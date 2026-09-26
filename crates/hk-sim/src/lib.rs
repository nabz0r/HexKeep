//! Combat uses signed integer subtiles (1/256 tile), with no float operations.
use borsh::{BorshDeserialize, BorshSerialize};
use hk_proto::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
pub const WIDTH: i32 = 30 * UNIT;
pub const HEIGHT: i32 = 16 * UNIT;
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Fighter {
    pub id: u8,
    pub realm: Realm,
    pub role: Role,
    pub pos: Vec2,
    pub aim: Vec2,
    pub hp: i32,
    pub armor: i32,
    pub cooldown: u16,
    pub dash_cd: u16,
    pub skill_cd: u16,
    pub shield: u16,
    pub invulnerable: u16,
    pub airborne: u16,
    pub charge: u16,
    pub rage: u16,
    pub respawn: u16,
    pub kills: u16,
    pub deaths: u16,
    pub bot: bool,
    pub genome: [u8; 16],
    pub velocity: Vec2,
}
impl Fighter {
    pub fn new(id: u8, realm: Realm, role: Role, bot: bool) -> Self {
        Self {
            id,
            realm,
            role,
            pos: spawn(id),
            aim: Vec2::new(UNIT, 0),
            hp: realm.hp(),
            armor: 100,
            cooldown: 0,
            dash_cd: 0,
            skill_cd: 0,
            shield: 0,
            invulnerable: 0,
            airborne: 0,
            charge: 0,
            rage: 0,
            respawn: 0,
            kills: 0,
            deaths: 0,
            bot,
            genome: [128; 16],
            velocity: Vec2::default(),
        }
    }
}
fn spawn(id: u8) -> Vec2 {
    let x = if id % 2 == 0 {
        3 + id as i32 % 3
    } else {
        25 - id as i32 % 3
    };
    let y = 3 + (id as i32 * 5) % 10;
    Vec2::new(x * UNIT, y * UNIT)
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Obstacle {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub kind: u8,
    pub life: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: u8,
    pub realm: Realm,
    pub kind: u8,
    pub damage: i32,
    pub life: u16,
    pub heavy: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Pickup {
    pub pos: Vec2,
    pub kind: u8,
    pub next: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Effect {
    pub pos: Vec2,
    pub radius: i32,
    pub life: u16,
    pub kind: u8,
    pub realm: Realm,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Battle {
    pub tick: u32,
    pub seed: u64,
    pub fighters: Vec<Fighter>,
    pub projectiles: Vec<Projectile>,
    pub obstacles: Vec<Obstacle>,
    pub pickups: Vec<Pickup>,
    pub effects: Vec<Effect>,
    pub duel: bool,
    pub round: [u8; 2],
    pub round_pause: u16,
    pub finished: bool,
}
fn touches(p: Vec2, o: &Obstacle, r: i32) -> bool {
    p.x + r > o.x && p.x - r < o.x + o.w && p.y + r > o.y && p.y - r < o.y + o.h
}
fn blocked(p: Vec2, obs: &[Obstacle], air: bool) -> bool {
    p.x < UNIT / 2
        || p.y < UNIT / 2
        || p.x > WIDTH - UNIT / 2
        || p.y > HEIGHT - UNIT / 2
        || obs
            .iter()
            .any(|o| !(air && o.kind != 0) && touches(p, o, 70))
}
impl Battle {
    pub fn new(seed: u64, realm: Realm, role: Role, bots: u8, duel: bool) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut fighters = vec![Fighter::new(0, realm, role, false)];
        for i in 1..=bots.min(9) {
            let mut f = Fighter::new(
                i,
                Realm::from_index((realm.index() + 1 + i as usize % 2) % 3),
                Role::from_index(i as usize),
                true,
            );
            for g in &mut f.genome {
                *g = rng.gen_range(32..=224);
            }
            fighters.push(f);
        }
        let mut obstacles = Vec::new();
        for (x, y, w, h, kind) in [
            (9, 3, 2, 3, 0),
            (19, 10, 2, 3, 0),
            (14, 6, 2, 4, 1),
            (5, 8, 3, 1, 1),
            (22, 5, 3, 1, 1),
            (10, 12, 4, 1, 2),
            (17, 2, 3, 1, 2),
        ] {
            obstacles.push(Obstacle {
                x: x * UNIT,
                y: y * UNIT,
                w: w * UNIT,
                h: h * UNIT,
                kind,
                life: 0,
            });
        }
        if seed % 2 == 1 {
            for o in &mut obstacles {
                o.x = WIDTH - o.x - o.w;
            }
        }
        let pickups = if duel {
            vec![]
        } else {
            vec![
                Pickup {
                    pos: Vec2::new(4 * UNIT, 8 * UNIT),
                    kind: 0,
                    next: 0,
                },
                Pickup {
                    pos: Vec2::new(26 * UNIT, 8 * UNIT),
                    kind: 1,
                    next: 0,
                },
                Pickup {
                    pos: Vec2::new(15 * UNIT, 13 * UNIT),
                    kind: 2,
                    next: 1350,
                },
            ]
        };
        Self {
            tick: 0,
            seed,
            fighters,
            projectiles: vec![],
            obstacles,
            pickups,
            effects: vec![],
            duel,
            round: [0, 0],
            round_pause: 0,
            finished: false,
        }
    }
    pub fn hash(&self) -> Hash {
        digest(self)
    }
    pub fn bot_input(&self, index: usize) -> Input {
        let f = &self.fighters[index];
        let target = self
            .fighters
            .iter()
            .filter(|g| g.id != f.id && g.realm != f.realm && g.hp > 0)
            .min_by_key(|g| (f.pos.dist2(g.pos), g.id));
        let Some(t) = target else {
            return Input::default();
        };
        let dx = t.pos.x - f.pos.x;
        let dy = t.pos.y - f.pos.y;
        let dist = isqrt(f.pos.dist2(t.pos) as u64) as i32;
        let preferred = if f.role == Role::Rempart {
            650
        } else {
            1000 + f.genome[1] as i32 * 3
        };
        let (mut mx, mut my) = if dist > preferred {
            (dx, dy)
        } else if dist < preferred - 300 {
            (-dx, -dy)
        } else {
            (-dy, dx)
        };
        if (self.tick / 90 + f.id as u32) % 2 == 0 {
            mx += dy / 2;
            my -= dx / 2;
        }
        let mv = Vec2::new(mx, my).scaled(1024);
        let aim = Vec2::new(dx, dy).scaled(1024);
        Input {
            move_x: mv.x as i16,
            move_y: mv.y as i16,
            aim_x: aim.x as i16,
            aim_y: aim.y as i16,
            shoot: self.tick % 90 > (15 + (255 - f.genome[3] as u32) / 8),
            dash: dist < 500 && self.tick % 120 == f.id as u32,
            skill: self.tick % 180 == f.id as u32,
        }
    }
    pub fn step(&mut self, inputs: &[(u8, Input)]) {
        if self.finished {
            return;
        }
        self.tick += 1;
        self.effects
            .iter_mut()
            .for_each(|e| e.life = e.life.saturating_sub(1));
        self.effects.retain(|e| e.life > 0);
        for o in &mut self.obstacles {
            if o.life > 0 {
                o.life -= 1;
                if o.life == 0 {
                    o.w = 0;
                }
            }
        }
        self.obstacles.retain(|o| o.w > 0);
        if self.round_pause > 0 {
            self.round_pause -= 1;
            if self.round_pause == 0 {
                for f in &mut self.fighters {
                    let kills = f.kills;
                    let deaths = f.deaths;
                    *f = Fighter::new(f.id, f.realm, f.role, f.bot);
                    f.kills = kills;
                    f.deaths = deaths;
                }
                self.projectiles.clear();
            }
            return;
        }
        let all: Vec<Input> = self
            .fighters
            .iter()
            .enumerate()
            .map(|(i, f)| {
                inputs
                    .iter()
                    .find(|(id, _)| *id == f.id)
                    .map(|(_, v)| *v)
                    .unwrap_or_else(|| {
                        if f.bot {
                            self.bot_input(i)
                        } else {
                            Input::default()
                        }
                    })
            })
            .collect();
        for (index, input) in all.iter().copied().enumerate() {
            let f = &mut self.fighters[index];
            if f.hp <= 0 {
                if !self.duel {
                    f.respawn = f.respawn.saturating_sub(1);
                    if f.respawn == 0 {
                        f.pos = spawn(f.id);
                        f.hp = f.realm.hp();
                        f.armor = 50;
                        f.invulnerable = 30;
                    }
                }
                continue;
            }
            for timer in [
                &mut f.cooldown,
                &mut f.dash_cd,
                &mut f.skill_cd,
                &mut f.shield,
                &mut f.invulnerable,
                &mut f.airborne,
                &mut f.rage,
            ] {
                *timer = timer.saturating_sub(1);
            }
            if f.hp > f.realm.hp() && self.tick % 30 == 0 {
                f.hp -= 1;
            }
            let aim = input.aim();
            if aim.x != 0 || aim.y != 0 {
                f.aim = aim.scaled(UNIT);
            }
            let mut velocity = input.movement().scaled(34);
            if self.effects.iter().any(|e| {
                e.kind == 3 && e.realm != f.realm && e.pos.dist2(f.pos) < (2 * UNIT).pow(2) as i64
            }) {
                velocity = velocity.scaled(20);
            }
            if input.dash && f.dash_cd == 0 {
                let direction = if velocity.x != 0 || velocity.y != 0 {
                    velocity
                } else {
                    f.aim
                };
                f.velocity = direction.scaled(154);
                f.invulnerable = 5;
                f.dash_cd = f.realm.dash();
                f.airborne = 6;
                self.effects.push(Effect {
                    pos: f.pos,
                    radius: UNIT / 2,
                    life: 12,
                    kind: 2,
                    realm: f.realm,
                });
            }
            velocity.x += f.velocity.x;
            velocity.y += f.velocity.y;
            f.velocity.x = f.velocity.x * 3 / 4;
            f.velocity.y = f.velocity.y * 3 / 4;
            let next = Vec2::new(f.pos.x + velocity.x, f.pos.y);
            if !blocked(next, &self.obstacles, f.airborne > 0) {
                f.pos = next;
            }
            let next = Vec2::new(f.pos.x, f.pos.y + velocity.y);
            if !blocked(next, &self.obstacles, f.airborne > 0) {
                f.pos = next;
            }
            if input.skill && f.skill_cd == 0 {
                f.skill_cd = 180;
                match f.role {
                    Role::Foudre => {
                        f.charge = 18;
                    }
                    Role::Rempart => {
                        if f.realm == Realm::Aurelon {
                            let center = Vec2::new(f.pos.x + f.aim.x * 2, f.pos.y + f.aim.y * 2);
                            let horizontal = f.aim.y.abs() > f.aim.x.abs();
                            self.obstacles.push(Obstacle {
                                x: center.x - if horizontal { 384 } else { 64 },
                                y: center.y - if horizontal { 64 } else { 384 },
                                w: if horizontal { 768 } else { 128 },
                                h: if horizontal { 128 } else { 768 },
                                kind: 0,
                                life: 120,
                            });
                        } else {
                            f.shield = 45;
                        }
                    }
                    Role::Lien => self.effects.push(Effect {
                        pos: Vec2::new(f.pos.x + f.aim.x * 3, f.pos.y + f.aim.y * 3),
                        radius: 2 * UNIT,
                        life: 90,
                        kind: 3,
                        realm: f.realm,
                    }),
                }
            }
            let heavy = f.charge == 1;
            if f.charge > 0 {
                f.charge -= 1;
            }
            if (input.shoot || heavy) && f.cooldown == 0 && f.charge == 0 {
                let base_speed = match f.realm {
                    Realm::Aurelon => 92,
                    Realm::Skarn => 65,
                    Realm::Vylde => 77,
                };
                f.cooldown = [24, 30, 36][f.role.index()];
                let dmg = if f.rage > 0 { 3 } else { 1 };
                let aim = if f.aim == Vec2::default() {
                    Vec2::new(UNIT, 0)
                } else {
                    f.aim
                };
                let start = Vec2::new(f.pos.x + aim.x / 2, f.pos.y + aim.y / 2);
                match f.role {
                    Role::Foudre => self.projectiles.push(Projectile {
                        pos: start,
                        vel: aim.scaled(if heavy {
                            base_speed * 7 / 10
                        } else {
                            base_speed
                        }),
                        owner: f.id,
                        realm: f.realm,
                        kind: 0,
                        damage: if heavy { 150 * dmg } else { 100 * dmg },
                        life: 100,
                        heavy,
                    }),
                    Role::Rempart => {
                        for s in -2..=2 {
                            let dir = Vec2::new(aim.x - aim.y * s / 6, aim.y + aim.x * s / 6);
                            self.projectiles.push(Projectile {
                                pos: start,
                                vel: dir.scaled(180),
                                owner: f.id,
                                realm: f.realm,
                                kind: 1,
                                damage: 12 * dmg,
                                life: 6,
                                heavy: false,
                            });
                        }
                    }
                    Role::Lien => self.projectiles.push(Projectile {
                        pos: start,
                        vel: aim.scaled(base_speed * 8 / 10),
                        owner: f.id,
                        realm: f.realm,
                        kind: 2,
                        damage: 60 * dmg,
                        life: 35,
                        heavy: false,
                    }),
                }
            }
        }
        let projectiles = std::mem::take(&mut self.projectiles);
        for mut p in projectiles {
            p.life = p.life.saturating_sub(1);
            if p.realm == Realm::Vylde && p.kind != 1 {
                if let Some(owner) = self.fighters.iter().find(|f| f.id == p.owner) {
                    let cross =
                        p.vel.x as i64 * owner.aim.y as i64 - p.vel.y as i64 * owner.aim.x as i64;
                    let sign = cross.signum() as i32;
                    let old = p.vel;
                    if sign != 0 {
                        let speed = if p.kind == 2 {
                            61
                        } else if p.heavy {
                            53
                        } else {
                            77
                        };
                        p.vel = Vec2::new(
                            (old.x * 65529 - old.y * 953 * sign) / 65536,
                            (old.y * 65529 + old.x * 953 * sign) / 65536,
                        )
                        .scaled(speed);
                    }
                }
            }
            p.pos.x += p.vel.x;
            p.pos.y += p.vel.y;
            let wall = p.pos.x < 0
                || p.pos.x > WIDTH
                || p.pos.y < 0
                || p.pos.y > HEIGHT
                || self
                    .obstacles
                    .iter()
                    .any(|o| o.kind == 0 && touches(p.pos, o, 20));
            let hit = self
                .fighters
                .iter()
                .find(|f| {
                    f.hp > 0
                        && f.id != p.owner
                        && (p.kind == 2 || f.realm != p.realm)
                        && f.pos.dist2(p.pos) < 120 * 120
                })
                .map(|f| f.id);
            if p.life == 0 || wall || hit.is_some() {
                if let Some(id) = hit {
                    let f = &self.fighters[id as usize];
                    let facing = f.aim.x as i64 * (p.pos.x - f.pos.x) as i64
                        + f.aim.y as i64 * (p.pos.y - f.pos.y) as i64;
                    if f.shield > 0 && facing >= 0 {
                        self.effects.push(Effect {
                            pos: p.pos,
                            radius: 70,
                            life: 6,
                            kind: 1,
                            realm: f.realm,
                        });
                        continue;
                    }
                }
                if p.kind == 1 {
                    if let Some(id) = hit {
                        self.damage(id, p.damage, p.owner, p.pos);
                    }
                } else {
                    let radius = if p.kind == 2 {
                        UNIT
                    } else {
                        [307, 499, 384][p.realm.index()]
                    };
                    self.effects.push(Effect {
                        pos: p.pos,
                        radius,
                        life: 10,
                        kind: 0,
                        realm: p.realm,
                    });
                    let victims: Vec<(u8, i32)> = self
                        .fighters
                        .iter()
                        .filter(|f| f.hp > 0)
                        .filter_map(|f| {
                            let d = isqrt(f.pos.dist2(p.pos) as u64) as i32;
                            if d > radius + 80 {
                                return None;
                            }
                            if f.realm == p.realm && f.id != p.owner {
                                return if p.kind == 2 { Some((f.id, -40)) } else { None };
                            }
                            let amount = if Some(f.id) == hit {
                                p.damage
                            } else {
                                (p.damage - 15) * (radius - d.min(radius)) / radius + 15
                            };
                            Some((f.id, if f.id == p.owner { amount / 2 } else { amount }))
                        })
                        .collect();
                    for (id, damage) in victims {
                        if damage < 0 {
                            let f = &mut self.fighters[id as usize];
                            f.hp = (f.hp - damage).min(f.realm.hp());
                        } else {
                            self.damage(id, damage, p.owner, p.pos);
                            let f = &mut self.fighters[id as usize];
                            let direction = Vec2::new(f.pos.x - p.pos.x, f.pos.y - p.pos.y)
                                .scaled(damage * [100, 140, 90][p.realm.index()] / 80);
                            f.velocity.x += direction.x;
                            f.velocity.y += direction.y;
                            if id == p.owner {
                                f.airborne = 15;
                            }
                        }
                    }
                }
            } else {
                self.projectiles.push(p);
            }
        }
        for pickup in &mut self.pickups {
            if self.tick < pickup.next {
                continue;
            }
            if let Some(f) = self
                .fighters
                .iter_mut()
                .find(|f| f.hp > 0 && f.pos.dist2(pickup.pos) < 180 * 180)
            {
                match pickup.kind {
                    0 => f.hp = (f.hp + 50).min(200),
                    1 => f.armor = (f.armor + 25).min(100),
                    _ => f.rage = 450,
                };
                pickup.next = self.tick + [1050, 750, 2700][pickup.kind as usize];
            }
        }
        if self.tick >= 5400 {
            self.finished = true;
        }
        if self.duel && self.fighters.len() == 2 {
            if let Some(loser) = self
                .fighters
                .iter()
                .find(|f| f.hp <= 0)
                .map(|f| f.id as usize)
            {
                let winner = 1 - loser;
                self.round[winner] += 1;
                if self.round[winner] >= 3 {
                    self.finished = true
                } else {
                    self.round_pause = 60;
                }
            }
        }
    }
    fn damage(&mut self, id: u8, amount: i32, owner: u8, _origin: Vec2) {
        let f = &mut self.fighters[id as usize];
        if f.hp <= 0 || f.invulnerable > 0 {
            return;
        }
        let absorbed = (amount * 2 / 3).min(f.armor);
        f.armor -= absorbed;
        f.hp -= amount - absorbed;
        if f.hp <= 0 {
            f.hp = 0;
            f.deaths += 1;
            f.respawn = 120;
            f.rage = 0;
            if id != owner {
                self.fighters[owner as usize].kills += 1;
            }
        }
    }
}
/// Bounded deterministic crossover; the seed must derive from a sealed generation.
pub fn evolve(pop: &[([u8; 16], i64)], seed: [u8; 32]) -> Vec<[u8; 16]> {
    let mut ranked = pop.to_vec();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    if ranked.is_empty() {
        return vec![];
    }
    let mut rng = ChaCha8Rng::from_seed(seed);
    let elite = ranked.len().div_ceil(2);
    (0..ranked.len())
        .map(|i| {
            let mut g = ranked[i % elite].0;
            for j in 0..16 {
                if rng.gen_bool(0.5) {
                    g[j] = ranked[(i + 1) % elite].0[j];
                }
                if rng.gen_ratio(1, 8) {
                    g[j] = (g[j] as i16 + rng.gen_range(-16..=16)).clamp(16, 240) as u8;
                }
            }
            g
        })
        .collect()
}
pub fn determinism_suite() -> Hash {
    let mut combined = blake3::Hasher::new();
    for seed in 0..10000u64 {
        let mut a = Battle::new(
            seed,
            Realm::from_index(seed as usize),
            Role::from_index(seed as usize / 3),
            1,
            true,
        );
        let mut b = a.clone();
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x4845584b);
        for tick in 0..180 {
            let input = Input {
                move_x: rng.gen_range(-1024..=1024),
                move_y: rng.gen_range(-1024..=1024),
                aim_x: rng.gen_range(-1024..=1024),
                aim_y: rng.gen_range(-1024..=1024),
                shoot: tick % 3 != 0,
                dash: tick % 40 == 0,
                skill: tick % 75 == 0,
            };
            a.step(&[(0, input)]);
            b.step(&[(0, input)]);
        }
        assert_eq!(a.hash(), b.hash(), "seed {seed}");
        combined.update(&a.hash());
    }
    *combined.finalize().as_bytes()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ten_thousand_replays() {
        let h = determinism_suite();
        println!("DETERMINISM {}", hex(&h));
    }
    #[test]
    fn roles_and_realms() {
        for r in 0..3 {
            for c in 0..3 {
                let mut b = Battle::new(10, Realm::from_index(r), Role::from_index(c), 9, false);
                for _ in 0..180 {
                    b.step(&[(
                        0,
                        Input {
                            aim_x: 1024,
                            shoot: true,
                            ..Input::default()
                        },
                    )]);
                }
                assert!(b.fighters.iter().all(|f| f.hp >= 0 && f.armor >= 0));
                assert!(b.tick == 180);
            }
        }
    }
    #[test]
    fn armor_damage() {
        let mut b = Battle::new(1, Realm::Aurelon, Role::Foudre, 1, false);
        b.damage(0, 100, 1, Vec2::default());
        assert_eq!(b.fighters[0].hp, 66);
        assert_eq!(b.fighters[0].armor, 34);
    }
    #[test]
    fn evolution_bounded() {
        let p = vec![([20; 16], 1), ([230; 16], 9)];
        assert_eq!(evolve(&p, [4; 32]), evolve(&p, [4; 32]));
        assert!(evolve(&p, [4; 32])
            .iter()
            .flatten()
            .all(|v| (16..=240).contains(v)));
    }
}
