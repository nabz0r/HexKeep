//! Offline action combat. This FSM never changes hk-sim's network protocol or hashes.
use super::{
    data,
    entities::*,
    world::{self, UNIT},
    Run,
};
fn dec(v: &mut u16) {
    *v = v.saturating_sub(1);
}
impl Run {
    pub fn effect(&mut self, p: Point, kind: u8, amount: i32) {
        if self.effects.len() < 64 {
            self.effects.push(Effect {
                pos: p,
                kind,
                amount,
                life: 24,
            });
        }
    }
    fn impact(&mut self, p: Point, heavy: bool, amount: i32) {
        self.impact_id = self.impact_id.wrapping_add(1);
        self.hitstop = if heavy { 4 } else { 2 };
        self.effect(p, if heavy { 2 } else { 0 }, amount);
    }
    fn hurt_player(&mut self, damage: i32) {
        let p = &mut self.player;
        if p.invulnerable > 0 || p.hp <= 0 {
            return;
        }
        let damage = (damage * p.stance.defense() / (100 + p.guard)).max(1);
        p.hp = (p.hp - damage).max(0);
        p.invulnerable = 18;
        p.state = if p.hp == 0 { State::Dead } else { State::Hurt };
        p.frames = if p.hp == 0 { 0 } else { 6 };
        let pos = p.pos;
        self.impact(pos, false, -damage);
    }
    fn swing(&mut self, skill: bool) {
        let p = &self.player;
        let heavy = p.combo == 3 || skill;
        let range = if skill {
            UNIT * 4
        } else if heavy {
            UNIT * 5 / 2
        } else {
            UNIT * 2
        };
        let damage = p.power * p.stance.damage() / 100
            * if skill {
                3
            } else if heavy {
                2
            } else {
                1
            };
        let origin = p.pos;
        let facing = p.facing;
        let zone = data::zone(self.zone);
        let mut hits = Vec::new();
        for m in &mut self.monsters {
            if !m.active
                || m.hp <= 0
                || origin.dist2(m.pos) > (range as i64).pow(2)
                || !world::visible(zone, origin, m.pos)
            {
                continue;
            }
            let dot = (m.pos.x - origin.x) as i64 * facing.x as i64
                + (m.pos.y - origin.y) as i64 * facing.y as i64;
            if !skill && dot < 0 {
                continue;
            }
            let guarded = m.stance == Stance::Bulwark && !heavy;
            let amount = if guarded { damage / 3 } else { damage };
            m.hp = (m.hp - amount).max(0);
            if m.hp == 0 {
                m.state = State::Dead;
                m.corpse = 45;
                self.kills += 1;
                if m.kind == 4 {
                    self.victory = true;
                }
            } else if m.kind != 4 && (heavy || m.kind != 2) {
                m.state = State::Hurt;
                m.frames = if heavy { 16 } else { 7 };
                if heavy {
                    m.stance = Stance::Balanced;
                    m.cooldown = 45;
                }
            }
            hits.push((m.pos, amount));
        }
        self.effect(
            origin,
            if skill {
                3
            } else if heavy {
                2
            } else {
                1
            },
            0,
        );
        if heavy && !skill {
            self.player.heavy_count += 1;
        }
        for (pos, damage) in hits {
            self.impact(pos, heavy, damage);
        }
    }
    /// Requests survive hitstop. A one-frame tap is buffered through recovery, never lost.
    pub fn step(&mut self, input: Controls) {
        self.pending.attack |= input.attack;
        self.pending.dash |= input.dash;
        self.pending.skill |= input.skill;
        if self.hitstop > 0 {
            self.hitstop -= 1;
            return;
        }
        self.tick += 1;
        for e in &mut self.effects {
            dec(&mut e.life);
        }
        self.effects.retain(|e| e.life > 0);
        if self.player.hp <= 0 || self.victory {
            return;
        }
        let zone = data::zone(self.zone);
        dec(&mut self.player.dash_cd);
        dec(&mut self.player.skill_cd);
        dec(&mut self.player.invulnerable);
        dec(&mut self.player.combo_window);
        if self.player.combo_window == 0 && matches!(self.player.state, State::Idle | State::Move) {
            self.player.combo = 0;
        }
        if self.player.frames > 0 {
            self.player.frames -= 1;
            if self.player.state == State::Dodge {
                let d = self.player.facing;
                let (dx, dy) = world::direction(Point::default(), d, 100);
                world::walk(zone, &mut self.player.pos, dx, dy);
            }
            if self.player.frames == 0 {
                match self.player.state {
                    State::Windup => {
                        self.player.state = State::Strike;
                        self.player.frames = 3;
                        self.swing(false);
                    }
                    State::Strike => {
                        self.player.state = State::Recovery;
                        self.player.frames = if self.player.combo == 3 { 14 } else { 7 };
                    }
                    State::Recovery => {
                        self.player.state = State::Idle;
                        self.player.combo = (self.player.combo + 1) % 4;
                        self.player.combo_window = 35;
                    }
                    _ => self.player.state = State::Idle,
                }
            }
        }
        if matches!(self.player.state, State::Idle | State::Move) {
            if self.pending.dash && self.player.dash_cd == 0 {
                self.pending.dash = false;
                self.player.state = State::Dodge;
                self.player.frames = 8;
                self.player.invulnerable = 12;
                self.player.dash_cd = 48;
                if input.mx.abs() + input.my.abs() > 80 {
                    self.player.facing = Point {
                        x: input.mx as i32,
                        y: input.my as i32,
                    };
                }
            } else if self.pending.skill && self.player.skill_cd == 0 {
                self.pending.skill = false;
                self.player.skill_cd = 180;
                self.player.state = State::Recovery;
                self.player.frames = 12;
                self.player.invulnerable = 12;
                self.swing(true);
            } else if self.pending.attack {
                self.pending.attack = false;
                if let Some(m) = self
                    .monsters
                    .iter()
                    .filter(|m| {
                        m.hp > 0 && m.active && world::visible(zone, self.player.pos, m.pos)
                    })
                    .min_by_key(|m| self.player.pos.dist2(m.pos))
                {
                    if self.player.pos.dist2(m.pos) < (6 * UNIT as i64).pow(2) {
                        self.player.facing = Point {
                            x: m.pos.x - self.player.pos.x,
                            y: m.pos.y - self.player.pos.y,
                        };
                    }
                }
                self.player.state = State::Windup;
                self.player.frames = ((if self.player.combo == 3 { 13 } else { 5 }) * 100
                    / self.player.stance.speed())
                .max(2) as u16;
                self.player.frames = self
                    .player
                    .frames
                    .saturating_sub(self.player.haste.min(3))
                    .max(2);
            } else {
                let dx = input.mx as i32;
                let dy = input.my as i32;
                if dx.abs() + dy.abs() > 80 {
                    let speed = 40 * self.player.stance.speed() / 100;
                    let (x, y) = world::direction(Point::default(), Point { x: dx, y: dy }, speed);
                    world::walk(zone, &mut self.player.pos, x, y);
                    self.player.state = State::Move;
                    self.player.facing = Point { x: dx, y: dy };
                } else {
                    self.player.state = State::Idle;
                }
            }
        }
        // Light footwork remains available while winding up and recovering a swing.
        if matches!(self.player.state, State::Windup | State::Recovery)
            && input.mx.abs() + input.my.abs() > 80
        {
            let (dx, dy) = world::direction(
                Point::default(),
                Point {
                    x: input.mx as i32,
                    y: input.my as i32,
                },
                16 * self.player.stance.speed() / 100,
            );
            world::walk(zone, &mut self.player.pos, dx, dy);
        }
        // An unavailable ability is not queued for several seconds after its tap.
        if self.player.dash_cd > 0 {
            self.pending.dash = false;
        }
        if self.player.skill_cd > 0 {
            self.pending.skill = false;
        }
        world::reveal(zone, self.player.pos, &mut self.explored);
        self.monster_step();
        let mut incoming = 0;
        for bolt in &mut self.projectiles {
            dec(&mut bolt.life);
            bolt.pos.x += bolt.dx;
            bolt.pos.y += bolt.dy;
            if !world::free(zone, bolt.pos) {
                bolt.life = 0;
            }
            if bolt.life > 0 && bolt.pos.dist2(self.player.pos) < (100i64).pow(2) {
                incoming += bolt.damage;
                bolt.life = 0;
            }
        }
        self.projectiles.retain(|p| p.life > 0);
        if incoming > 0 {
            self.hurt_player(incoming);
        }
        // A little recovery between encounters rewards exploration without trivialising boss pressure.
        if self.tick % 90 == 0
            && self.monsters.iter().all(|m| {
                m.hp <= 0 || !m.active || m.pos.dist2(self.player.pos) > (8 * UNIT as i64).pow(2)
            })
        {
            self.player.hp = (self.player.hp + 3).min(self.player.max_hp);
        }
    }
    fn monster_step(&mut self) {
        let zone = data::zone(self.zone);
        let player = self.player.pos;
        let flow = world::field(zone, player);
        let mut damage = 0;
        let mut bolts = Vec::new();
        let mut summon = Vec::new();
        for m in &mut self.monsters {
            if m.hp <= 0 {
                dec(&mut m.corpse);
                continue;
            }
            if !m.active {
                continue;
            }
            let dist = m.pos.dist2(player);
            if dist > (13 * UNIT as i64).pow(2) && m.kind != 4 {
                continue;
            }
            dec(&mut m.cooldown);
            if m.kind == 4 {
                let phase = if m.hp * 3 < m.max_hp {
                    2
                } else if m.hp * 3 < m.max_hp * 2 {
                    1
                } else {
                    0
                };
                if phase > m.phase {
                    m.phase = phase;
                    m.cooldown = 20;
                    self.phase_events += 1;
                    for i in 0..3 {
                        summon.push((
                            Point {
                                x: m.pos.x + (i - 1) * 300,
                                y: m.pos.y + 400,
                            },
                            3,
                        ));
                    }
                }
                m.stance = match m.phase {
                    0 => Stance::Bulwark,
                    1 => Stance::Balanced,
                    _ => Stance::Assault,
                };
            }
            if m.frames > 0 {
                m.frames -= 1;
                if m.frames == 0 {
                    if m.state == State::Windup {
                        m.state = State::Strike;
                        m.frames = 5;
                        match m.kind {
                            1 => {
                                let (dx, dy) = world::direction(m.pos, m.target, 80);
                                bolts.push(Projectile {
                                    pos: m.pos,
                                    dx,
                                    dy,
                                    life: 100,
                                    damage: (12 + self.zone as i32 * 3) * m.stance.damage() / 100,
                                });
                            }
                            4 => {
                                let count = if m.phase == 2 { 12 } else { 8 };
                                for i in 0..count {
                                    let angle = i as f64 * std::f64::consts::TAU / count as f64;
                                    bolts.push(Projectile {
                                        pos: m.pos,
                                        dx: (angle.cos() * 55.0) as i32,
                                        dy: (angle.sin() * 55.0) as i32,
                                        life: 115,
                                        damage: (17 + self.zone as i32 * 3) * m.stance.damage()
                                            / 100,
                                    });
                                }
                                if dist < (3 * UNIT as i64).pow(2) {
                                    damage += 24 * m.stance.damage() / 100;
                                }
                                if m.phase > 0 {
                                    let (dx, dy) = world::direction(m.pos, m.target, UNIT * 2);
                                    for _ in 0..8 {
                                        world::walk(zone, &mut m.pos, dx / 8, dy / 8);
                                    }
                                }
                            }
                            _ => {
                                if dist < (if m.kind == 2 { 460i64 } else { 370i64 }).pow(2) {
                                    damage += ([15, 12, 22, 7][m.kind as usize]
                                        + self.zone as i32 * 3)
                                        * m.stance.damage()
                                        / 100;
                                }
                            }
                        }
                    } else {
                        m.state = State::Idle;
                    }
                }
                continue;
            }
            let range = if m.kind == 1 {
                6 * UNIT
            } else if m.kind == 4 {
                4 * UNIT
            } else {
                310
            };
            if dist < (range as i64).pow(2)
                && m.cooldown == 0
                && world::visible(zone, m.pos, player)
            {
                m.state = State::Windup;
                m.target = player;
                m.frames = match m.kind {
                    0 => 16,
                    1 => 22,
                    2 => 30,
                    3 => 12,
                    _ => 36 - self.zone as u16 * 3,
                };
                m.cooldown = match m.kind {
                    0 => 50,
                    1 => 70,
                    2 => 95,
                    3 => 45,
                    _ => 100 - m.phase as u16 * 15,
                };
                if m.kind != 2 && m.kind != 4 {
                    m.stance = Stance::Assault;
                }
            } else {
                let mut target = world::seek(zone, m.pos, player, &flow);
                let mut speed = match m.kind {
                    0 => 24,
                    1 => 22,
                    2 => 13,
                    3 => 31,
                    _ => 17 + m.phase as i32 * 5,
                };
                if m.kind == 1 && dist < (4 * UNIT as i64).pow(2) {
                    target = Point {
                        x: m.pos.x + (m.pos.x - player.x),
                        y: m.pos.y + (m.pos.y - player.y),
                    };
                    m.stance = Stance::Balanced;
                } else if m.kind == 1
                    && dist < (5 * UNIT as i64).pow(2)
                    && world::visible(zone, m.pos, player)
                {
                    speed = 0;
                }
                if m.kind == 3
                    && dist > (2 * UNIT as i64).pow(2)
                    && world::visible(zone, m.pos, player)
                {
                    let offset = if m.id % 2 == 0 { 230 } else { -230 };
                    target.x += offset;
                    target.y -= offset;
                }
                let (dx, dy) = world::direction(m.pos, target, speed * m.stance.speed() / 100);
                world::walk(zone, &mut m.pos, dx, dy);
                m.state = if speed == 0 { State::Idle } else { State::Move };
                if m.kind == 2 && m.cooldown == 0 {
                    m.stance = Stance::Bulwark;
                }
            }
        }
        if damage > 0 {
            self.hurt_player(damage);
        }
        for b in bolts {
            if self.projectiles.len() < 128 {
                self.projectiles.push(b);
            }
        }
        self.monsters
            .retain(|m| m.hp > 0 || m.corpse > 0 || m.kind == 4);
        for (pos, kind) in summon {
            if self.monsters.len() < 48 && world::free(zone, pos) {
                self.monsters
                    .push(Monster::new(self.next_monster, kind, pos, self.zone));
                self.next_monster += 1;
            }
        }
    }
}
