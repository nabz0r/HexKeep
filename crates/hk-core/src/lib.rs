mod adventure;
mod expansion;
mod network;
mod presentation;
use hk_ledger::{Kind, Ledger};
use hk_ppu::*;
use hk_proto::*;
use hk_sim::*;
use hk_world::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub music: bool,
    pub haptics: bool,
    pub english: bool,
    pub accessible: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            music: true,
            haptics: true,
            english: false,
            accessible: false,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Save {
    pub version: u8,
    pub secret: [u8; 32],
    pub created: bool,
    pub realm: Realm,
    pub role: Role,
    pub name: String,
    pub world: World,
    pub ledger: Ledger,
    pub kills: u32,
    pub tutorial_done: bool,
    pub settings: Settings,
    #[serde(default)]
    pub expansion: expansion::ExpansionSave,
    #[serde(default)]
    pub introduction_seen: bool,
    #[serde(default = "default_sound")]
    pub sound_effects: bool,
    #[serde(default)]
    pub journey: adventure::Journey,
}
fn default_sound() -> bool {
    true
}
impl Default for Save {
    fn default() -> Self {
        let secret = hk_crypto::new_secret();
        let suffix = hex(&hk_crypto::public(&secret)[..2]);
        Self {
            version: 1,
            secret,
            created: false,
            realm: Realm::Aurelon,
            role: Role::Foudre,
            name: format!("Veilleur {suffix}"),
            world: World::default(),
            ledger: Ledger::default(),
            kills: 0,
            tutorial_done: false,
            settings: Settings::default(),
            expansion: Default::default(),
            introduction_seen: false,
            sound_effects: true,
            journey: Default::default(),
        }
    }
}
#[derive(Clone, serde::Serialize)]
pub struct Expedition {
    pub charges: [u16; 3],
    pub wave: u8,
    pub victory: bool,
    pub kind: u8,
    pub event_id: String,
    pub cell: u64,
    pub caches: [bool; 3],
    pub drops: Vec<adventure::Drop>,
    pub dust: u32,
    pub flasks: u8,
    pub explored: Vec<bool>,
}
pub struct Game {
    pub save: Save,
    pub expansion: expansion::Expansion,
    pub session: hk_crypto::identity::Session,
    pub screen: u8,
    pub ticks: u64,
    pub now: u64,
    pub width: i32,
    pub battle: Option<Battle>,
    pub tutorial: bool,
    pub lesson: u8,
    pub touches: BTreeMap<i32, (i32, i32, u8)>,
    pub selected: u64,
    pub message: String,
    pub message_until: u64,
    pub dirty: bool,
    pub native_action: u8,
    pub haptic: u8,
    pub sound: u8,
    pub storage_error: bool,
    pub is_dev: bool,
    quiz: u8,
    keyboard: String,
    keyboard_mode: u8,
    keyboard_return: u8,
    pub key_input: Input,
    pub battle_mode: u8,
    pub expedition: Option<Expedition>,
    battle_start_kills: u16,
    pub last_hash: String,
    pub node: Option<hk_net::Node>,
    pub peers: BTreeMap<[u8; 32], network::Peer>,
    pub online: Option<network::Online>,
    pub addresses: Vec<String>,
    pub peer_count: usize,
    pub latency: u64,
    pub last_proof: String,
}
impl Game {
    pub fn new(snapshot: &str, is_dev: bool) -> Self {
        let parsed = if snapshot.is_empty() {
            Ok(Save::default())
        } else {
            serde_json::from_str::<Save>(snapshot)
        };
        let error = parsed.is_err();
        let save = parsed.unwrap_or_default();
        let mut tampered = save.version != 1 || save.ledger.events.iter().any(|e| !e.valid("dev"));
        let mut authority = expansion::fixture().authority;
        if authority
            .merge(&save.expansion.authority, &save.ledger)
            .is_err()
        {
            tampered = true;
        }
        let last_proof = save
            .expansion
            .case_files
            .last()
            .cloned()
            .unwrap_or_default();
        let selected = save.world.current;
        let session = hk_crypto::identity::Session::development(&save.secret, hk_net::unix_time());
        Self {
            save,
            session,
            expansion: expansion::Expansion {
                brush: 1,
                ..Default::default()
            },
            screen: 0,
            ticks: 0,
            now: 0,
            width: 520,
            battle: None,
            tutorial: false,
            lesson: 0,
            touches: BTreeMap::new(),
            selected,
            message: String::new(),
            message_until: 0,
            dirty: false,
            native_action: 0,
            haptic: 0,
            sound: 0,
            storage_error: error || tampered,
            is_dev,
            quiz: 0,
            keyboard: String::new(),
            keyboard_mode: 0,
            keyboard_return: 0,
            key_input: Input::default(),
            battle_mode: 0,
            expedition: None,
            battle_start_kills: 0,
            last_hash: String::new(),
            node: None,
            peers: BTreeMap::new(),
            online: None,
            addresses: vec![],
            peer_count: 0,
            latency: 0,
            last_proof,
        }
    }
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.save).unwrap()
    }
    pub fn sensitive(&self) -> bool {
        matches!(self.screen, 4 | 5 | 13) || self.screen == 11 && self.keyboard_mode == 1
    }
    pub fn tr<'a>(&self, fr: &'a str, en: &'a str) -> &'a str {
        if self.save.settings.english {
            en
        } else {
            fr
        }
    }
    pub fn toast(&mut self, msg: &str) {
        self.message = msg.into();
        self.message_until = self.ticks + 150;
    }
    fn event(&mut self, kind: Kind, value: u32) {
        self.save.ledger.append(
            &self.save.secret,
            self.save.world.current,
            kind,
            self.save.realm,
            value,
            self.now,
        );
        self.dirty = true;
    }
    pub fn start_battle(&mut self, mode: u8) {
        self.expansion.court = None;
        self.online = None;
        self.battle_mode = mode;
        self.expedition = if mode == 8 {
            Some(Expedition {
                charges: [0; 3],
                wave: 0,
                victory: false,
                kind: 0,
                cell: self.save.world.current,
                event_id: format!("{}:{}:0", self.save.world.current, self.now / 1800),
                caches: [false; 3],
                drops: vec![],
                dust: 0,
                flasks: 2,
                explored: vec![false; 480],
            })
        } else {
            None
        };
        self.tutorial = mode == 0;
        self.lesson = 0;
        let seed = self.save.world.current
            ^ (self.save.kills as u64 * 997)
            ^ (self.save.journey.outings as u64 * 7919);
        let mut b = Battle::new(
            seed,
            self.save.realm,
            self.save.role,
            if mode == 3 || mode == 5 || mode == 7 {
                9
            } else if mode == 2 || mode == 8 {
                3
            } else {
                1
            },
            mode == 4,
        );
        if mode == 8 {
            b.fighters[0].pos = Vec2::new(4 * UNIT, 8 * UNIT);
        }
        if self.tutorial {
            b.fighters[0].pos = Vec2::new(7 * UNIT, 8 * UNIT);
            b.fighters[1].pos = Vec2::new(21 * UNIT, 8 * UNIT);
            b.fighters[1].armor = 0;
            b.fighters[1].genome = [32; 16];
        }
        if let Ok(state) = self.save.expansion.authority.state(&self.save.ledger) {
            b.apply_codex(state.codex);
        }
        if mode == 8 {
            let (health, power, armor, haste) = self.save.journey.stats();
            b.fighters[0].hp += 60 + health;
            b.fighters[0].max_hp = b.fighters[0].hp;
            b.fighters[0].power += power;
            b.fighters[0].armor += armor;
            b.fighters[0].haste = haste;
            for f in b.fighters.iter_mut().skip(1) {
                f.realm = Realm::from_index((self.save.realm.index() + 1) % 3);
                f.hp = 75;
                f.max_hp = 75;
                f.power = 55;
                f.armor = 20;
            }
        }
        if mode == 5 {
            b.begin_siege(self.save.realm, self.walls());
        }
        if mode >= 5 {
            for f in &mut b.fighters {
                if f.id > 0 {
                    f.genome = self.save.expansion.campaign.genomes
                        [f.id as usize % self.save.expansion.campaign.genomes.len()];
                }
            }
        }
        if mode == 7 {
            let population = self.save.expansion.campaign.genomes.clone();
            for f in &mut b.fighters {
                if f.id > 0 {
                    for j in 0..16 {
                        f.genome[j] = population
                            .iter()
                            .map(|g| g[j] as u32)
                            .sum::<u32>()
                            .checked_div(population.len() as u32)
                            .unwrap_or(128) as u8;
                    }
                }
            }
            b.fighters[1].hp = 300;
        }
        self.battle_start_kills = 0;
        self.battle = Some(b);
        self.screen = 6;
        self.touches.clear();
    }
    fn end_battle(&mut self) {
        if self.online.is_some() {
            self.finish_online();
            self.screen = 7;
            self.battle = None;
            self.touches.clear();
            return;
        }
        if let Some(b) = self.battle.take() {
            self.expansion_battle_end(&b);
            if let Some(run) = self.expedition.take() {
                self.journey_reward(&b, &run);
                let lit = run.charges.iter().filter(|v| **v == 90).count() as u32;
                self.save.expansion.campaign.xp += lit * 20 + if run.victory { 80 } else { 0 };
                if run.victory {
                    if let Some(c) = self.save.world.cells.get_mut(&run.cell) {
                        c.clear = true;
                    }
                    self.event(Kind::Lantern, lit);
                }
            }
            self.last_hash = hex(&b.hash());
            let kills = b.fighters[0].kills as u32;
            self.save.kills += kills;
            if kills > 0 {
                self.event(Kind::Victory, kills * 10);
                if let Some(c) = self.save.world.cells.get_mut(&self.save.world.current) {
                    c.battles += 1;
                }
            }
        }
        if self.tutorial && self.lesson >= 4 {
            self.save.tutorial_done = true;
        }
        self.screen = 7;
        self.touches.clear();
        self.dirty = true;
    }
    pub fn tick(&mut self, now: u64) {
        self.ticks += 1;
        self.now = now;
        self.poll_network();
        self.expansion_tick();
        if self.storage_error || !self.is_dev {
            return;
        }
        if self.save.created && self.ticks % 30 == 0 {
            if self.save.journey.discovered.insert(self.save.world.current) {
                self.dirty = true;
            }
            if self.save.world.seconds == 0 {
                for c in self.save.world.cells.values_mut() {
                    c.last_watch = now;
                }
            }
            self.save
                .world
                .elapse_radius(now, 1, if self.save.expansion.phare { 5 } else { 2 });
            if self.ticks % 300 == 0 {
                self.dirty = true;
            }
            if self.ticks % 1800 == 0 {
                self.event(Kind::Watch, 60);
            }
        }
        if self.screen != 6 {
            // Opening an in-match menu cannot pause the other participants.
            if self.screen == 14 && self.online.is_some() {
                self.step_online(Input::default());
            }
            return;
        }
        let mut input = self.key_input;
        for (x, y, zone) in self.touches.values() {
            match zone {
                1 => {
                    input.move_x = ((x - 48) * 35).clamp(-1024, 1024) as i16;
                    input.move_y = ((y - 190) * 35).clamp(-1024, 1024) as i16;
                }
                2 => {
                    input.aim_x = ((x - (self.width - 48)) * 40).clamp(-1024, 1024) as i16;
                    input.aim_y = ((y - 190) * 40).clamp(-1024, 1024) as i16;
                    input.shoot = input.aim_x.abs() + input.aim_y.abs() > 120;
                }
                3 => input.dash = true,
                4 => input.skill = true,
                _ => {}
            }
        }
        if self.online.is_some() {
            self.step_online(input);
            return;
        }
        let b = self.battle.as_mut().unwrap();
        let old_hp = b.fighters[0].hp;
        let old_shots = b.projectiles.len();
        let old_kills = b.fighters[0].kills;
        if self.tutorial {
            match self.lesson {
                0 => {
                    if input.move_x.abs() + input.move_y.abs() > 200 && b.tick > 20 {
                        self.lesson = 1
                    }
                }
                1 => {
                    if input.shoot {
                        self.lesson = 2
                    }
                }
                2 => {
                    if input.dash {
                        self.lesson = 3
                    }
                }
                3 => {
                    if input.skill {
                        self.lesson = 4
                    }
                }
                _ => {}
            }
        }
        if self.tutorial && self.lesson < 4 {
            b.fighters[1].invulnerable = 2;
            b.step(&[(0, input), (1, Input::default())]);
            b.fighters[0].hp = self.save.realm.hp();
        } else {
            b.step(&[(0, input)]);
        }
        if b.fighters[0].hp < old_hp && self.save.settings.haptics {
            self.haptic = 1;
            self.sound = 2;
        } else if b.projectiles.len() > old_shots {
            self.sound = 1;
        }
        if b.fighters[0].kills > old_kills {
            self.sound = 3;
        }
        if let Some(run) = &mut self.expedition {
            let player_pos = b.fighters[0].pos;
            for i in 0..480 {
                let p = Vec2::new(
                    (i % 30) as i32 * UNIT + UNIT / 2,
                    (i / 30) as i32 * UNIT + UNIT / 2,
                );
                if p.dist2(player_pos) < (5 * UNIT).pow(2) as i64 {
                    run.explored[i] = true;
                }
            }
            if b.fighters[0].kills > old_kills {
                for f in b
                    .fighters
                    .iter()
                    .skip(1)
                    .filter(|f| f.hp <= 0 && f.respawn == 120)
                {
                    run.drops.push(adventure::Drop {
                        pos: f.pos,
                        collected: false,
                        value: 3,
                    });
                }
            }
            for drop in &mut run.drops {
                if !drop.collected && drop.pos.dist2(player_pos) < (2 * UNIT).pow(2) as i64 {
                    drop.collected = true;
                    run.dust += drop.value;
                    self.sound = 4;
                }
            }
            for (i, (x, y)) in [(3, 13), (16, 2), (27, 12)].iter().enumerate() {
                if !run.caches[i]
                    && player_pos.dist2(Vec2::new(x * UNIT, y * UNIT))
                        < (UNIT * 3 / 2).pow(2) as i64
                    && b.fighters[0].hp > 0
                {
                    run.caches[i] = true;
                    run.dust += 2;
                    self.sound = 4;
                    self.haptic = 2;
                }
            }

            for f in b.fighters.iter_mut().skip(1) {
                if f.hp <= 0 {
                    f.respawn = u16::MAX;
                }
            }
            let points = [
                Vec2::new(6 * UNIT, 4 * UNIT),
                Vec2::new(15 * UNIT, 12 * UNIT),
                Vec2::new(25 * UNIT, 5 * UNIT),
            ];
            for (i, pos) in points.iter().enumerate() {
                let d = Vec2::new(b.fighters[0].pos.x - pos.x, b.fighters[0].pos.y - pos.y);
                if run.charges[i] < 90
                    && (if run.kind == 1 {
                        b.fighters[0].kills as usize >= (i + 1) * 2
                    } else {
                        d.x * d.x + d.y * d.y < (UNIT * 3 / 2).pow(2)
                    })
                    && b.fighters[0].hp > 0
                {
                    run.charges[i] = (run.charges[i]
                        + if run.kind == 1 {
                            90
                        } else if run.kind == 2 {
                            6
                        } else {
                            2
                        })
                    .min(90);
                    if run.charges[i] == 90 {
                        self.sound = 4;
                        self.haptic = 2;
                        if i < 2 {
                            for j in 0..2 {
                                let id = b.fighters.len() as u8;
                                let mut enemy = Fighter::new(
                                    id,
                                    Realm::from_index((self.save.realm.index() + 1) % 3),
                                    Role::from_index((i + j) % 3),
                                    true,
                                );
                                enemy.pos = Vec2::new(
                                    if b.fighters[0].pos.x < 15 * UNIT {
                                        26 * UNIT
                                    } else {
                                        3 * UNIT
                                    },
                                    (5 + j as i32 * 6) * UNIT,
                                );
                                enemy.hp = 65;
                                enemy.max_hp = 65;
                                enemy.power = 55;
                                enemy.armor = 10;
                                enemy.invulnerable = 20;
                                b.fighters.push(enemy);
                            }
                        }
                        b.fighters[0].hp = (b.fighters[0].hp + 35).min(b.fighters[0].max_hp);
                    }
                }
            }
            if run.charges.iter().all(|c| *c == 90) && run.wave == 0 {
                run.wave = 1;
                let mut boss = Fighter::new(
                    1,
                    Realm::from_index((self.save.realm.index() + 1) % 3),
                    Role::Rempart,
                    true,
                );
                boss.pos = Vec2::new(25 * UNIT, 8 * UNIT);
                boss.hp = 360;
                boss.max_hp = 360;
                boss.power = 70;
                boss.armor = 60;
                boss.genome = [160; 16];
                b.fighters.truncate(1);
                b.fighters.push(boss);
                self.sound = 5;
            } else if run.wave == 1 && b.fighters[1].hp <= 0 {
                run.victory = true;
                b.finished = true;
            }
            if b.fighters[0].deaths >= 3 {
                b.finished = true;
            }
        }
        if (self.tutorial && b.fighters[0].kills > 0)
            || (self.battle_mode == 6 && b.fighters[0].kills > 0)
            || (self.battle_mode == 7 && b.fighters[1].deaths > 0)
            || b.finished
        {
            self.screen = 12;
            self.touches.clear();
        }
    }
    pub fn touch(&mut self, id: i32, phase: u8, x: i32, y: i32) {
        if phase == 3 {
            self.touches.clear();
            return;
        }
        if phase == 2 {
            self.touches.remove(&id);
            return;
        }
        if self.screen == 6 {
            if phase == 0 && y < 22 && x > self.width - 64 {
                self.screen = 14;
                self.touches.clear();
                return;
            }
            let zone = if let Some(t) = self.touches.get(&id) {
                t.2
            } else if x < 106 && y > 139 {
                1
            } else if x > self.width - 94 && y > 139 {
                2
            } else if (x - (self.width - 123)).pow(2) + (y - 202).pow(2) < 20 * 20 {
                3
            } else if (x - (self.width - 123)).pow(2) + (y - 155).pow(2) < 20 * 20 {
                4
            } else {
                0
            };
            self.touches.insert(id, (x, y, zone));
            return;
        }
        if phase == 0 {
            self.tap(x, y);
        }
    }
    pub fn tap(&mut self, x: i32, y: i32) {
        let w = self.width;
        if self.storage_error {
            return;
        }
        if !self.is_dev {
            return;
        }
        self.sound = 1;
        if self.screen >= 20 {
            self.expansion_tap(x, y);
            return;
        }
        match self.screen {
            0 => {
                if y < 100 {
                    self.expansion.hidden_taps += 1;
                    if self.expansion.hidden_taps >= 7 {
                        self.native_action = 8;
                    }
                }
                if y >= 150 && y <= 184 {
                    if self.save.created {
                        self.screen = 7
                    } else {
                        self.screen = 1
                    }
                } else if y >= 188 && y <= 217 {
                    self.screen = 10;
                }
            }
            1 => {
                if y >= 188 && x < w / 2 {
                    self.screen = 2;
                } else if y >= 188 {
                    self.screen = 0;
                }
            }
            2 => {
                let card = (w - 48) / 3;
                if (59..180).contains(&y) {
                    let i = ((x - 16) / card).clamp(0, 2);
                    self.save.realm = Realm::from_index(i as usize);
                } else if y > 192 && x > w / 2 {
                    self.screen = 3;
                } else if y > 192 {
                    self.screen = 1;
                }
            }
            3 => {
                let card = (w - 48) / 3;
                if (52..183).contains(&y) {
                    self.save.role = Role::from_index(((x - 16) / card).clamp(0, 2) as usize);
                } else if y > 190 && x > w / 2 {
                    if self.save.created {
                        self.screen = 7;
                        self.dirty = true;
                    } else {
                        self.screen = 4;
                    }
                } else if y > 190 {
                    self.screen = if self.save.created { 7 } else { 2 };
                }
            }
            4 => {
                if y > 204 {
                    if x < w / 3 {
                        self.keyboard_mode = 1;
                        self.keyboard_return = 4;
                        self.keyboard.clear();
                        self.screen = 11;
                    } else {
                        self.quiz = 0;
                        self.screen = 5;
                    }
                }
            }
            5 => {
                if y > 99 && y < 192 {
                    let chosen = ((y - 100) / 29).clamp(0, 2) as usize;
                    let correct = self.quiz_correct();
                    if chosen == correct {
                        self.quiz += 1;
                        if self.quiz == 3 {
                            self.save.created = true;
                            self.dirty = true;
                            self.event(Kind::Watch, 1);
                            self.start_battle(0);
                        }
                    } else {
                        self.screen = 4;
                        self.toast("Revois tes mots puis réessaie.");
                    }
                } else if y > 205 {
                    self.screen = 4;
                }
            }
            7 => {
                if y < 25 {
                    if x < 180 {
                        self.screen = 20;
                    } else if x > w - 44 {
                        self.screen = 10;
                    } else if x > w - 107 {
                        self.screen = 8;
                    } else if x > w - 163 {
                        self.screen = 16;
                    }
                } else if x > w - 158 {
                    if (44..72).contains(&y) {
                        self.start_battle(1);
                    } else if (76..104).contains(&y) {
                        self.start_battle(2);
                    } else if (108..136).contains(&y) {
                        let msg = match self.save.world.found(self.save.realm) {
                            Ok(()) => {
                                self.event(Kind::Foundation, 0);
                                "Ton bastion se dresse. État provisoire."
                            }
                            Err(e) => e,
                        };
                        self.toast(msg);
                    } else if (140..168).contains(&y) {
                        self.save.world.toggle_banner(self.now);
                        self.event(Kind::Banner, self.save.world.banner as u32);
                    } else if (172..198).contains(&y) {
                        if self.selected != self.save.world.current {
                            self.save.world.gps = false;
                            self.save.world.enter(self.selected, self.now);
                            self.dirty = true;
                            self.toast("Déplacement de développement.");
                        } else {
                            self.native_action = 1;
                        }
                    } else if y > 206 {
                        self.screen = 3;
                    }
                } else {
                    let cx = (w - 164) / 2;
                    let cy = 111;
                    if let Some((id, _, _)) = offsets(self.save.world.current, 2)
                        .into_iter()
                        .min_by_key(|(_, i, j)| {
                            let xx = cx + (i - j) * 36;
                            let yy = cy + (i + j) * 21;
                            (xx - x).pow(2) + (yy - y).pow(2)
                        })
                    {
                        self.selected = id;
                    }
                }
            }
            8 | 9 | 15 => {
                if y > 205 {
                    self.screen = 7;
                }
            }
            10 => {
                if y < 33 && x > w - 65 {
                    self.screen = if self.save.created { 7 } else { 0 };
                } else if (48..73).contains(&y) {
                    self.save.settings.music = !self.save.settings.music;
                    self.dirty = true;
                } else if (78..103).contains(&y) {
                    self.save.settings.haptics = !self.save.settings.haptics;
                    self.dirty = true;
                } else if (108..133).contains(&y) {
                    self.save.settings.english = !self.save.settings.english;
                    self.dirty = true;
                } else if (138..163).contains(&y) {
                    self.save.settings.accessible = !self.save.settings.accessible;
                    self.dirty = true;
                } else if (176..201).contains(&y) && self.save.created {
                    if x < w / 2 {
                        self.screen = 13;
                        self.native_action = 2;
                    } else {
                        self.keyboard_mode = 0;
                        self.keyboard_return = 10;
                        self.keyboard = self.save.name.clone();
                        self.screen = 11;
                    }
                } else if y > 210 {
                    self.screen = if self.save.created { 7 } else { 0 };
                }
            }
            11 => {
                if y > 202 {
                    if x < w / 3 {
                        self.screen = self.keyboard_return;
                    } else if x > 2 * w / 3 {
                        if self.keyboard_mode == 2 {
                            self.connect_text();
                            self.screen = 16;
                        } else if self.keyboard_mode == 0 {
                            if !self.keyboard.trim().is_empty() {
                                self.save.name = self.keyboard.trim().chars().take(18).collect();
                                self.dirty = true;
                                self.screen = self.keyboard_return;
                            }
                        } else {
                            match hk_crypto::restore(&self.keyboard) {
                                Ok(secret) => {
                                    self.save.secret = secret;
                                    self.session = hk_crypto::identity::Session::development(
                                        &secret,
                                        hk_net::unix_time(),
                                    );
                                    self.native_action = 10;
                                    self.save.created = true;
                                    self.dirty = true;
                                    self.screen = 7;
                                    self.toast("Nom restauré sur cet appareil.");
                                }
                                Err(e) => self.toast(&e),
                            }
                        }
                    } else {
                        self.keyboard.push(' ');
                    }
                } else if y >= 80 && y < 198 {
                    let row = ((y - 80) / 30) as usize;
                    let col = ((x - (w - 300) / 2) / 30).clamp(0, 9) as usize;
                    let rows = ["qwertyuiop", "asdfghjkl<", "zxcvbnm.:<", "0123456789"];
                    if row < 4 {
                        let ch = rows[row].as_bytes()[col] as char;
                        if ch == '<' {
                            self.keyboard.pop();
                        } else if self.keyboard.len() < 280 {
                            self.keyboard.push(ch);
                        }
                    }
                }
            }
            12 => {
                if y > 155 {
                    self.end_battle();
                }
            }
            13 => {
                if y > 205 {
                    self.screen = 10;
                }
            }
            14 => {
                if y >= 65 && y < 96 {
                    self.screen = 6;
                } else if y >= 106 && y < 136 {
                    self.end_battle();
                } else if y >= 146 && y < 177 {
                    self.start_battle(self.battle_mode);
                }
            }
            16 => self.network_tap(x, y),
            17 => {
                if y > 180 {
                    self.finish_online();
                    self.screen = 7;
                    self.battle = None;
                }
            }
            _ => {}
        }
    }
    fn quiz_index(&self) -> usize {
        let b = self.save.secret[self.quiz as usize] as usize;
        (b + self.quiz as usize * 7) % 24
    }
    fn quiz_correct(&self) -> usize {
        self.save.secret[10 + self.quiz as usize] as usize % 3
    }
    pub fn gps(&mut self, lat: i32, lng: i32, mock: bool) {
        if !self.is_dev && mock {
            return;
        }
        if let Some(cell) = cell_at(lat, lng) {
            self.save.world.enter(cell, self.now);
            self.selected = cell;
            self.save.world.gps = true;
            self.dirty = true;
            self.toast(if mock {
                "GPS de développement actif."
            } else {
                "Position locale actualisée."
            });
        }
    }
    pub fn frame(&mut self, w: i32) -> Vec<u8> {
        self.canvas(w, false).px
    }
    pub fn canvas(&mut self, w: i32, recording: bool) -> Canvas {
        self.width = w.clamp(400, 640) / 8 * 8;
        let w = self.width;
        let mut c = if recording {
            Canvas::recording(w, 240)
        } else {
            Canvas::new(w, 240)
        };
        c.clear(INK);
        if !self.is_dev {
            c.center(44, "HEXKEEP", GOLD, 4);
            c.center(111, "PRODUCTION VERROUILLÉE", WHITE, 1);
            c.center(136, "Genèse et attestation officielles requises.", MUTED, 1);
            c.center(170, "Utilise l'APK DEV pour jouer.", GOLD, 1);
            return c;
        }
        if self.storage_error {
            c.center(80, "SAUVEGARDE INDISPONIBLE", RED, 2);
            c.center(116, "Aucun fichier n'a été remplacé.", WHITE, 1);
            c.center(
                136,
                "Déverrouille l'appareil puis relance le jeu.",
                MUTED,
                1,
            );
            return c;
        }
        match self.screen {
            0 => {
                c.landscape(self.ticks as u32);
                c.center(26, self.tr("LA PREMIÈRE NUIT", "THE FIRST NIGHT"), GOLD, 1);
                c.center(48, "HEXKEEP", WHITE, 5);
                c.center(
                    105,
                    self.tr("Là où personne ne veille,", "Where no one keeps watch,"),
                    MUTED,
                    1,
                );
                c.center(
                    117,
                    self.tr("le monde s'éteint.", "the world goes dark."),
                    MUTED,
                    1,
                );
                c.button(
                    w / 2 - 86,
                    153,
                    172,
                    if self.save.created {
                        self.tr("REPRENDRE LA VEILLE", "KEEP WATCH")
                    } else {
                        self.tr("ALLUMER MA LANTERNE", "LIGHT MY LANTERN")
                    },
                    true,
                );
                c.button(
                    w / 2 - 64,
                    187,
                    128,
                    self.tr("RÉGLAGES / SETTINGS", "SETTINGS / RÉGLAGES"),
                    false,
                );
                c.center(
                    226,
                    self.tr(
                        "DEV · CAMPAGNE ET COURONNE · 0.2",
                        "DEV · CAMPAIGN AND CROWN · 0.2",
                    ),
                    MUTED,
                    1,
                );
            }
            1 => {
                c.center(
                    22,
                    self.tr("AVANT LA PREMIÈRE NUIT", "BEFORE THE FIRST NIGHT"),
                    GOLD,
                    2,
                );
                c.sprite(w / 2 - 16, 48, 2, 0, 2, 0);
                for (i, line) in [
                    "J'ai 16 ans ou plus.",
                    "Je reste attentif au monde autour de moi.",
                    "Je ne joue jamais en conduisant.",
                    "Je n'entre pas dans les propriétés privées.",
                    "Ma position GPS reste sur cet appareil.",
                ]
                .iter()
                .enumerate()
                {
                    c.center(95 + i as i32 * 15, line, WHITE, 1);
                }
                c.button(w / 2 - 164, 194, 158, "J'ACCEPTE · CONTINUER", true);
                c.button(w / 2 + 6, 194, 158, "RETOUR", false);
            }
            2 => {
                c.center(
                    18,
                    self.tr("CHOISIS TON ROYAUME", "CHOOSE YOUR REALM"),
                    GOLD,
                    2,
                );
                c.center(39, "Un serment pour toute la saison.", MUTED, 1);
                let cw = (w - 48) / 3;
                let mottos = [
                    "CE QUI EST BÂTI TIENT.",
                    "TIENS, OU TOMBE.",
                    "TOUT REVIENT.",
                ];
                let lines = [
                    [
                        "PIERRE ET SERMENT",
                        "100 PV · TIRS VIFS",
                        "DASH : 3 SECONDES",
                    ],
                    ["FER ET HIVER", "110 PV · RECUL FORT", "DASH : 3,5 SECONDES"],
                    [
                        "BRUME ET SÈVE",
                        "90 PV · TIRS COURBES",
                        "DASH : 2,4 SECONDES",
                    ],
                ];
                for i in 0..3 {
                    let x = 16 + i as i32 * (cw + 8);
                    c.panel(x, 58, cw, 125);
                    if self.save.realm.index() == i {
                        c.frame(x, 58, cw, 125, REALMS[i]);
                    }
                    c.text(x + 8, 68, Realm::from_index(i).name(), REALMS[i], 2);
                    c.sprite(x + cw / 2 - 16, 93, 0, i, 2, 0);
                    for (j, line) in lines[i].iter().enumerate() {
                        c.text(
                            x + 8,
                            134 + j as i32 * 12,
                            line,
                            if j == 0 { REALMS[i] } else { MUTED },
                            1,
                        );
                    }
                    let _ = mottos;
                }
                c.button(16, 202, 90, "RETOUR", false);
                c.button(w - 186, 202, 170, "PRÊTER SERMENT >", true);
            }
            3 => {
                c.center(
                    19,
                    self.tr("COMMENT VAS-TU VEILLER ?", "HOW WILL YOU KEEP WATCH?"),
                    GOLD,
                    2,
                );
                let cw = (w - 48) / 3;
                let lines = [
                    [
                        "ROQUETTE · 100 DÉGÂTS",
                        "TIR CHARGÉ / SOUFFLE",
                        "EXPLOSION ET PROPULSION",
                    ],
                    [
                        "5 PLOMBS · 12 DÉGÂTS",
                        "BOUCLIER / MUR",
                        "DÉFENSE RAPPROCHÉE",
                    ],
                    ["ORBE · 60 DÉGÂTS", "SOINS ALLIÉS : 40", "ENTRAVE DE ZONE"],
                ];
                for i in 0..3 {
                    let x = 16 + i as i32 * (cw + 8);
                    c.panel(x, 57, cw, 123);
                    if self.save.role.index() == i {
                        c.frame(x, 57, cw, 123, GOLD);
                    }
                    c.text(x + 8, 68, Role::from_index(i).name(), WHITE, 2);
                    c.sprite(x + cw / 2 - 8, 96, 0, self.save.realm.index(), 1, i as u32);
                    c.circle(
                        x + cw / 2 + 15,
                        104,
                        3 + i as i32,
                        REALMS[self.save.realm.index()],
                    );
                    for (j, line) in lines[i].iter().enumerate() {
                        c.text(x + 6, 130 + j as i32 * 13, line, MUTED, 1);
                    }
                }
                c.button(16, 201, 94, "RETOUR", false);
                c.button(w - 186, 201, 170, "CHOISIR CE RÔLE >", true);
            }
            4 | 13 => {
                c.center(15, "TON NOM VRAI", GOLD, 2);
                c.center(38, "Note ces 24 mots à l'abri des regards.", WHITE, 1);
                c.center(
                    51,
                    "Ils restaurent ton identité. Ne les partage pas.",
                    MUTED,
                    1,
                );
                let phrase = hk_crypto::mnemonic(&self.save.secret);
                let words: Vec<_> = phrase.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    let col = i / 8;
                    let row = i % 8;
                    let x = 24 + col as i32 * ((w - 48) / 3);
                    c.text(
                        x,
                        70 + row as i32 * 15,
                        &format!("{:02} {}", i + 1, word),
                        WHITE,
                        1,
                    );
                }
                if self.screen == 4 {
                    c.button(16, 210, 116, "RESTAURER", false);
                    c.button(w - 216, 210, 200, "MES MOTS SONT NOTÉS >", true);
                } else {
                    c.button(w / 2 - 65, 210, 130, "FERMER", true);
                }
            }
            5 => {
                c.center(24, "LE SERMENT DU NOM", GOLD, 2);
                c.center(57, &format!("Vérification {}/3", self.quiz + 1), MUTED, 1);
                c.center(
                    80,
                    &format!("Quel est ton mot numéro {} ?", self.quiz_index() + 1),
                    WHITE,
                    1,
                );
                let phrase = hk_crypto::mnemonic(&self.save.secret);
                let words: Vec<_> = phrase.split_whitespace().collect();
                let right = self.quiz_correct();
                for i in 0..3 {
                    let index = if i == right {
                        self.quiz_index()
                    } else {
                        (self.quiz_index() + i + 5) % 24
                    };
                    c.button(
                        w / 2 - 100,
                        100 + i as i32 * 29,
                        200,
                        words[index],
                        i == right && false,
                    );
                }
                c.button(w / 2 - 90, 209, 180, "REVOIR MES MOTS", false);
            }
            6 => {
                if !recording {
                    self.draw_battle(&mut c)
                }
            }
            7 => self.draw_map(&mut c),
            8 => {
                c.center(16, "LA CHRONIQUE", GOLD, 2);
                c.center(40, "Ton histoire locale · aucun Sceau reçu", MUTED, 1);
                c.text(
                    20,
                    62,
                    &format!("{} · {}", self.save.name, self.save.realm.name()),
                    WHITE,
                    1,
                );
                c.text(
                    20,
                    80,
                    &format!(
                        "{} SANS-TÉMOIN VAINCUS  /  {} MIN DE VEILLE",
                        self.save.kills,
                        self.save.world.seconds / 60
                    ),
                    MUTED,
                    1,
                );
                let count = self.save.ledger.events.len();
                for (i, e) in self
                    .save
                    .ledger
                    .events
                    .iter()
                    .skip(count.saturating_sub(6))
                    .enumerate()
                {
                    let kind = match e.body.kind {
                        Kind::Watch => "UNE LANTERNE TIENT LA NUIT",
                        Kind::Foundation => "UN BASTION SORT DU NOIR",
                        Kind::Victory => "LE NOIR A CÉDÉ DU TERRAIN",
                        Kind::Banner => "UNE BANNIÈRE CHANGE LE VENT",
                        Kind::Generation => "LE NOIR APPREND",
                        Kind::Capture => "BASTION CAPTURÉ",
                        Kind::Memory => "MÉMOIRE LIVRÉE",
                        Kind::Lantern => "LANTERNE DE LA NUIT",
                    };
                    c.text(
                        20,
                        103 + i as i32 * 15,
                        kind,
                        if e.body.kind == Kind::Foundation {
                            GOLD
                        } else {
                            WHITE
                        },
                        1,
                    );
                }
                c.button(w / 2 - 65, 210, 130, "RETOUR CARTE", true);
            }
            9 => {
                c.center(14, "LE CODEX DU VEILLEUR", GOLD, 2);
                for (i, line) in [
                    "GAUCHE : déplace-toi avec le premier stick.",
                    "DROITE : oriente le second stick pour tirer.",
                    "A : dash. B : compétence du rôle choisi.",
                    "FOUDRE : ton souffle te propulse au-dessus",
                    "des obstacles bas. Garde tes distances.",
                    "COEUR : +50 PV. ÉCAILLE : +25 armure.",
                    "BRAISE : dégâts triplés. Compte de tête.",
                    "VEILLE : le Noir s'éclaire après 10 minutes.",
                    "BÂTIS : un bastion par site clair.",
                    "Aucun achat ni rang n'augmente ta puissance.",
                ]
                .iter()
                .enumerate()
                {
                    c.text(
                        22,
                        43 + i as i32 * 15,
                        line,
                        if i == 9 { GOLD } else { WHITE },
                        1,
                    );
                }
                c.button(w / 2 - 65, 210, 130, "RETOUR CARTE", true);
            }
            10 => {
                c.center(16, self.tr("RÉGLAGES", "SETTINGS"), GOLD, 2);
                let x = w / 2 - 156;
                for (i, (fr, en, value)) in [
                    (
                        "MUSIQUE ET SONS",
                        "MUSIC AND SOUND",
                        self.save.settings.music,
                    ),
                    ("VIBRATIONS", "HAPTICS", self.save.settings.haptics),
                    (
                        "LANGUE : ENGLISH",
                        "LANGUAGE : ENGLISH",
                        self.save.settings.english,
                    ),
                    (
                        "PALETTE CONTRASTÉE",
                        "CONTRAST PALETTE",
                        self.save.settings.accessible,
                    ),
                ]
                .iter()
                .enumerate()
                {
                    c.button(
                        x,
                        48 + i as i32 * 30,
                        312,
                        &format!(
                            "{} : {}",
                            self.tr(fr, en),
                            if *value { "ON" } else { "OFF" }
                        ),
                        *value,
                    );
                }
                if self.save.created {
                    c.button(x, 178, 151, "REVOIR MON NOM", false);
                    c.button(x + 161, 178, 151, "NOM D'AFFICHAGE", false);
                }
                c.button(
                    w / 2 - 90,
                    210,
                    180,
                    self.tr("ENREGISTRER / RETOUR", "SAVE / BACK"),
                    true,
                );
            }
            11 => {
                c.center(
                    12,
                    if self.keyboard_mode == 0 {
                        "NOM D'AFFICHAGE"
                    } else if self.keyboard_mode == 2 {
                        "ADRESSE IP:PORT DU PAIR"
                    } else {
                        "RESTAURER LES 24 MOTS"
                    },
                    GOLD,
                    2,
                );
                let s: Vec<char> = self.keyboard.chars().collect();
                let start = s.len().saturating_sub((w as usize - 40) / 6);
                c.text(20, 43, &s[start..].iter().collect::<String>(), WHITE, 1);
                c.text(
                    20,
                    60,
                    &format!("{} mots", self.keyboard.split_whitespace().count()),
                    MUTED,
                    1,
                );
                for (r, row) in ["qwertyuiop", "asdfghjkl<", "zxcvbnm.:<", "0123456789"]
                    .iter()
                    .enumerate()
                {
                    for (col, ch) in row.chars().enumerate() {
                        c.button(
                            (w - 300) / 2 + col as i32 * 30,
                            80 + r as i32 * 30,
                            27,
                            &ch.to_string(),
                            false,
                        );
                    }
                }
                c.button(15, 211, w / 3 - 22, "ANNULER", false);
                c.button(w / 3 + 7, 211, w / 3 - 14, "ESPACE", false);
                c.button(2 * w / 3 + 7, 211, w / 3 - 22, "VALIDER", true);
            }
            12 => {
                c.landscape(self.ticks as u32);
                c.panel(w / 2 - 171, 44, 342, 147);
                c.text(
                    w / 2 - 147,
                    61,
                    if self.tutorial {
                        "TA LANTERNE EST ALLUMÉE"
                    } else {
                        if self.battle_mode == 5 {
                            "LE SIÈGE EST TERMINÉ"
                        } else if self.battle_mode == 7 {
                            "UNE FLAMME DANS LA NUIT"
                        } else {
                            "LE COMBAT EST TERMINÉ"
                        }
                    },
                    GOLD,
                    2,
                );
                c.center(96, "Le monde tient tant que tu veilles.", WHITE, 1);
                let stats = self
                    .battle
                    .as_ref()
                    .map(|b| {
                        format!(
                            "{} éliminations • {} chutes • {} s",
                            b.fighters[0].kills,
                            b.fighters[0].deaths,
                            b.tick / 30
                        )
                    })
                    .unwrap_or_default();
                c.center(119, &stats, MUTED, 1);
                c.button(w / 2 - 125, 159, 250, "RETOURNER À LA CARTE >", true);
            }
            14 => {
                self.draw_battle(&mut c);
                c.panel(w / 2 - 140, 36, 280, 168);
                c.center(45, "UNE HALTE DANS LA NUIT", GOLD, 1);
                c.button(w / 2 - 110, 68, 220, "REPRENDRE LE COMBAT", true);
                c.button(w / 2 - 110, 108, 220, "RETOURNER À LA CARTE", false);
                c.button(w / 2 - 110, 148, 220, "RECOMMENCER L'ENTRAÎNEMENT", false);
            }
            16 => self.draw_network(&mut c),
            17 => {
                c.center(48, "LE CHAMP S'EST REFERMÉ", GOLD, 2);
                c.center(95, "Le combat n'a pas été validé.", WHITE, 1);
                c.center(115, "Un pair est absent ou les états divergent.", MUTED, 1);
                c.center(140, "Aucun point ni territoire n'est attribué.", MUTED, 1);
                c.button(w / 2 - 100, 195, 200, "RETOUR À LA CARTE", true);
            }
            _ => {}
        }
        if self.screen >= 20 {
            self.draw_expansion(&mut c);
        }
        if self.message_until > self.ticks && !self.sensitive() {
            let n = self.message.chars().count() as i32 * 6 + 16;
            let x = (w - n) / 2;
            c.rect(x, 219, n, 17, PANEL);
            c.frame(x, 219, n, 17, GOLD);
            c.text(x + 8, 224, &self.message, WHITE, 1);
        }
        if self.save.settings.accessible {
            for px in c.px.chunks_exact_mut(4) {
                let color = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
                let replacement = if color == GREEN {
                    Some(0xdabaed)
                } else if color == RED {
                    Some(0xff9568)
                } else {
                    None
                };
                if let Some(v) = replacement {
                    px[0] = (v >> 16) as u8;
                    px[1] = (v >> 8) as u8;
                    px[2] = v as u8;
                }
            }
        }
        c
    }
    fn draw_map(&self, c: &mut Canvas) {
        let w = c.w;
        let world = &self.save.world;
        c.rect(0, 0, w, 26, PANEL);
        c.text(12, 9, "VEILLE >", GOLD, 1);
        c.text(
            68,
            9,
            self.save.realm.name(),
            REALMS[self.save.realm.index()],
            1,
        );
        c.text(w - 155, 9, "PAIRS", MUTED, 1);
        c.text(w - 100, 9, "RÉCIT", MUTED, 1);
        c.text(w - 41, 9, "MENU", MUTED, 1);
        let cx = (w - 164) / 2;
        let cy = 111;
        for (id, i, j) in offsets(world.current, 3) {
            let x = cx + (i - j) * 36;
            let y = cy + (i + j) * 21;
            if x < -20 || x > w - 154 || y < 36 || y > 193 {
                continue;
            }
            let cell = world.cells.get(&id);
            let clear = cell.is_some_and(|v| v.clear);
            let watch = cell.is_some_and(|v| v.continuous > 0);
            let selected = id == self.selected;
            let color = if clear {
                0x2f4844
            } else if watch {
                0x1b2835
            } else {
                0x121a28
            };
            c.hex(
                x,
                y,
                23,
                color,
                if selected {
                    GOLD
                } else if clear {
                    0x648574
                } else {
                    0x33404f
                },
            );
            if clear {
                for n in 0..3 {
                    let xx = x - 14 + ((id >> n * 3) as i32 & 15);
                    let yy = y - 9 + n * 8;
                    c.rect(xx, yy, 3, 1, 0x4b6559);
                }
            } else {
                let dx = (id as i32).rem_euclid(17) - 8;
                c.rect(x + dx, y - 3, 6, 1, 0x444058);
                c.pixel(x - dx, y + 4, 0x5b526c);
            }
            if let Some(realm) = cell.and_then(|v| v.bastion) {
                c.sprite(x - 8, y - 10, 3, realm.index(), 1, 0);
            }
            if id == world.current {
                c.sprite(
                    x - 8,
                    y - 8,
                    2,
                    self.save.realm.index(),
                    1,
                    self.ticks as u32,
                );
                c.rect(x - 2, y + 10, 4, 2, GOLD);
                if self.save.expansion.campaign.equipped == "lantern_ambre" {
                    c.ring(x, y, 15, GOLD);
                }
            }
            if selected {
                c.text(x - 11, y + 16, "ICI", GOLD, 1);
            }
        }
        if self.save.expansion.authority.interregnum(self.now) {
            c.text(12, 230, "INTERRÈGNE • HISTOIRE PROVISOIRE", GOLD, 1);
        }
        c.rect(w - 163, 27, 163, 213, INK);
        c.button(
            w - 151,
            45,
            139,
            self.tr("CHASSER LE NOIR", "HUNT THE DARK"),
            true,
        );
        c.button(
            w - 151,
            78,
            139,
            self.tr("3 SANS-TÉMOIN", "3 UNWITNESSED"),
            false,
        );
        c.button(
            w - 151,
            111,
            139,
            self.tr("FONDER UN BASTION", "FOUND A BASTION"),
            false,
        );
        c.button(
            w - 151,
            144,
            139,
            if world.lower_at.is_some() {
                "BANNIÈRE : RETRAIT"
            } else if world.banner {
                "BAISSER BANNIÈRE"
            } else {
                "LEVER BANNIÈRE"
            },
            false,
        );
        c.button(
            w - 151,
            177,
            139,
            if self.selected != world.current {
                "MARCHER (DEV)"
            } else {
                "UTILISER MON GPS"
            },
            false,
        );
        c.text(
            w - 150,
            214,
            &format!("RÔLE : {} >", self.save.role.name()),
            REALMS[self.save.realm.index()],
            1,
        );
        c.rect(0, 197, w - 164, 43, PANEL);
        c.text(
            12,
            204,
            if world.gps {
                "POSITION GPS LOCALE"
            } else {
                "CARTE D'ENTRAÎNEMENT · LUXEMBOURG"
            },
            MUTED,
            1,
        );
        c.text(
            12,
            216,
            &format!(
                "{} · {} MIN · {} VICTOIRES",
                self.save.role.name(),
                world.seconds / 60,
                self.save.kills
            ),
            WHITE,
            1,
        );
        c.text(
            12,
            228,
            if self.node.is_some() {
                "PROVISOIRE · AUCUN SCEAU · LAN"
            } else {
                "PROVISOIRE · AUCUN SCEAU · SOLO"
            },
            GOLD,
            1,
        );
    }
    fn draw_battle(&self, c: &mut Canvas) {
        let Some(b) = &self.battle else { return };
        let w = c.w;
        let ox = (w - 360) / 2;
        let oy = 27;
        for y in 0..16 {
            for x in 0..30 {
                let shade = if (x + y) % 2 == 0 { 0x26343b } else { 0x29383e };
                c.rect(ox + x * 12, oy + y * 12, 12, 12, shade);
                if (x * 17 + y * 7 + b.seed as i32).rem_euclid(13) == 0 {
                    c.rect(ox + x * 12 + 3, oy + y * 12 + 4, 2, 1, 0x40514f);
                }
            }
        }
        c.frame(ox - 1, oy - 1, 362, 194, EDGE);
        let point = |p: Vec2| (ox + p.x * 12 / UNIT, oy + p.y * 12 / UNIT);
        for o in &b.obstacles {
            let x = ox + o.x * 12 / UNIT;
            let y = oy + o.y * 12 / UNIT;
            let ww = o.w * 12 / UNIT;
            let hh = o.h * 12 / UNIT;
            let color = if o.kind == 2 {
                0x335969
            } else if o.kind == 1 {
                0x4a5a52
            } else {
                0x5c6a70
            };
            c.rect(x + 2, y + 3, ww, hh, 0x19242d);
            c.rect(x, y, ww, hh, color);
            c.rect(x, y, ww, 2, if o.kind == 2 { 0x74a4b2 } else { 0x8c998c });
            if o.kind == 0 {
                for yy in (y + 6..y + hh).step_by(7) {
                    c.rect(x, yy, ww, 1, 0x34434b);
                }
            } else if o.kind == 2 {
                for yy in (y + 4..y + hh).step_by(4) {
                    c.rect(
                        x + 2 + (self.ticks as i32 / 10 % 3),
                        yy,
                        (ww - 5).max(1),
                        1,
                        BLUE,
                    );
                }
            }
        }
        if let Some(s) = &b.siege {
            let (x, y) = point(Vec2::new(20 * UNIT, 8 * UNIT));
            c.rect(
                x - 12,
                y - 18,
                24,
                36,
                if s.gate_hp > 0 { GOLD } else { EDGE },
            );
            let (cx, cy) = point(Vec2::new(24 * UNIT, 8 * UNIT));
            c.ring(cx, cy, 24, GOLD);
            c.text(
                ox + 80,
                oy + 4,
                &format!("PORTE {}  COUR {}/60", s.gate_hp, s.capture / 30),
                WHITE,
                1,
            );
        }
        for p in &b.pickups {
            if b.tick >= p.next {
                let (x, y) = point(p.pos);
                let color = [RED, BLUE, GOLD][p.kind as usize];
                c.ring(x, y, 7, EDGE);
                if p.kind == 0 {
                    c.rect(x - 4, y - 1, 9, 3, color);
                    c.rect(x - 1, y - 4, 3, 9, color);
                } else if p.kind == 1 {
                    c.hex(x, y, 4, color, WHITE);
                } else {
                    c.circle(x, y, 4, GOLD);
                    c.pixel(x, y - 6, WHITE);
                }
            }
        }
        for e in &b.effects {
            let (x, y) = point(e.pos);
            let r = e.radius * 12 / UNIT;
            if e.kind == 3 {
                c.ring(x, y, r, GREEN);
                c.ring(x, y, r - 2, EDGE);
            } else {
                c.ring(
                    x,
                    y,
                    (r * (11 - e.life.min(10) as i32) / 10).max(2),
                    if e.kind == 2 { BLUE } else { GOLD },
                );
            }
        }
        for f in &b.fighters {
            if f.hp <= 0 {
                continue;
            }
            let (x, y) = point(f.pos);
            c.rect(x - 6, y + 5, 12, 3, 0x142029);
            if f.invulnerable == 0 || self.ticks % 4 < 2 {
                c.sprite(
                    x - 8,
                    y - 10,
                    if f.bot { 1 } else { 0 },
                    f.realm.index(),
                    1,
                    b.tick,
                );
            }
            if f.id == self.online.as_ref().map(|o| o.rollback.local).unwrap_or(0) {
                let item = &self.save.expansion.campaign.equipped;
                let index = (blake3::hash(item.as_bytes()).as_bytes()[0] % 4) as usize;
                let color = [GOLD, BLUE, GREEN, WHITE][index];
                c.rect(x - 4, y + 2, 8, 6, color);
                c.rect(x - 1, y + 2, 2, 6, INK);
                if item.starts_with("skin_") {
                    c.rect(x - 5, y - 7, 10, 2, color);
                }
                if item == "patron_month" {
                    c.ring(x, y, 12, GOLD);
                }
                if let Some(house) = self.save.expansion.campaign.houses.last() {
                    for yy in 0..4 {
                        for xx in 0..4 {
                            if house.emblem[(yy * 4 * 16 + xx * 4) as usize] > 0 {
                                c.pixel(x - 2 + xx, y + 2 + yy, WHITE);
                            }
                        }
                    }
                }
            }
            let color = if f.id == self.online.as_ref().map(|o| o.rollback.local).unwrap_or(0) {
                GOLD
            } else {
                RED
            };
            c.rect(x - 8, y - 14, 16, 2, INK);
            c.rect(x - 8, y - 14, (16 * f.hp / f.realm.hp()).min(16), 2, color);
            if f.shield > 0 {
                c.ring(x, y, 12, BLUE);
            }
            if f.id == self.online.as_ref().map(|o| o.rollback.local).unwrap_or(0) {
                let aim = f.aim.scaled(16);
                c.pixel(x + aim.x, y + aim.y, GOLD);
                c.pixel(x + aim.x + 1, y + aim.y, GOLD);
            }
        }
        for p in &b.projectiles {
            let (x, y) = point(p.pos);
            c.rect(x - p.vel.x / 28, y - p.vel.y / 28, 2, 2, RED);
            c.circle(
                x,
                y,
                if p.kind == 1 { 1 } else { 2 },
                if p.kind == 2 { GREEN } else { GOLD },
            );
            c.pixel(x, y, WHITE);
        }
        c.rect(0, 0, w, 23, INK);
        let local = self.online.as_ref().map(|o| o.rollback.local).unwrap_or(0);
        let f = &b.fighters[local as usize];
        c.text(
            8,
            8,
            &format!("PV {:03}", f.hp),
            if f.hp < 30 { RED } else { WHITE },
            1,
        );
        c.rect(51, 7, 61, 7, EDGE);
        c.rect(51, 7, (f.hp.min(200) * 60 / 200).max(0), 7, GREEN);
        c.text(122, 8, &format!("ARM {:03}", f.armor), BLUE, 1);
        c.text(
            188,
            8,
            &format!("{} · {}", self.save.role.name(), f.kills),
            GOLD,
            1,
        );
        c.text(w - 46, 8, "PAUSE", MUTED, 1);
        if self.tutorial {
            let lessons = [
                "1/5  BOUGE AVEC LE STICK GAUCHE",
                "2/5  VISE ET TIRE AVEC LE STICK DROIT",
                "3/5  APPUIE SUR A POUR LE DASH",
                "4/5  APPUIE SUR B POUR LA COMPÉTENCE",
                "5/5  VAINCS LE SANS-TÉMOIN",
            ];
            let s = lessons[self.lesson.min(4) as usize];
            c.rect(w / 2 - 126, 27, 252, 17, INK);
            c.center(32, s, WHITE, 1);
        } else {
            c.text(
                ox + 5,
                31,
                if self.battle_mode == 4 {
                    "DUEL D'ENTRAÎNEMENT"
                } else {
                    if self.online.is_some() {
                        "COMBAT ENTRE PAIRS · DEV"
                    } else {
                        "CHASSE AUX SANS-TÉMOIN"
                    }
                },
                MUTED,
                1,
            );
        }
        for (x, y, label) in [(48, 190, "BOUGER"), (w - 48, 190, "VISER")] {
            c.circle(x, y, 29, INK);
            c.ring(x, y, 29, EDGE);
            c.ring(x, y, 20, EDGE);
            let zone = if x == 48 { 1 } else { 2 };
            let (dx, dy) = self
                .touches
                .values()
                .find(|v| v.2 == zone)
                .map(|v| {
                    let a = Vec2::new(v.0 - x, v.1 - y).scaled(
                        (isqrt(((v.0 - x).pow(2) + (v.1 - y).pow(2)) as u64) as i32).min(23),
                    );
                    (a.x, a.y)
                })
                .unwrap_or((0, 0));
            c.circle(x + dx, y + dy, 8, if zone == 1 { MUTED } else { GOLD });
            c.text(x - 18, 226, label, MUTED, 1);
        }
        for (x, y, label, ready) in [
            (w - 123, 202, "A", f.dash_cd == 0),
            (w - 123, 155, "B", f.skill_cd == 0),
        ] {
            c.circle(x, y, 17, INK);
            c.ring(x, y, 17, if ready { GOLD } else { EDGE });
            c.text(x - 2, y - 3, label, if ready { WHITE } else { MUTED }, 1);
        }
        c.text(w - 138, 225, "DASH", MUTED, 1);
        c.text(w - 144, 132, "POUVOIR", MUTED, 1);
        if f.hp == 0 {
            c.panel(w / 2 - 110, 94, 220, 45);
            c.center(105, "LA LANTERNE VACILLE", RED, 1);
            c.center(121, "Retour au Champ dans un instant.", WHITE, 1);
        }
        if b.duel {
            c.center(
                226,
                &format!("ROUNDS {} / {}", b.round[0], b.round[1]),
                GOLD,
                1,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn solo_save_roundtrip() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.now = 1234;
        g.save.world.found(Realm::Aurelon).unwrap();
        g.event(Kind::Foundation, 0);
        let saved = g.snapshot();
        let h = Game::new(&saved, true);
        assert!(!h.storage_error);
        assert_eq!(h.save.world.current, g.save.world.current);
        assert_eq!(
            h.save.world.cells[&h.save.world.current].bastion,
            Some(Realm::Aurelon)
        );
        assert_eq!(
            h.save.ledger.fold("dev", &BTreeSet::new()).bastions.len(),
            1
        );
    }
    #[test]
    fn malformed_save_never_overwritten() {
        let g = Game::new("garbage", true);
        assert!(g.storage_error);
        assert!(!g.dirty);
    }
    #[test]
    fn onboarding_and_touch() {
        let mut g = Game::new("", true);
        g.frame(520);
        g.tap(260, 161);
        assert_eq!(g.screen, 1);
        g.tap(100, 204);
        g.tap(410, 208);
        g.tap(410, 208);
        assert_eq!(g.screen, 4);
        g.tap(410, 218);
        for _ in 0..3 {
            let y = 111 + g.quiz_correct() as i32 * 29;
            g.tap(260, y);
        }
        assert!(g.save.created);
        assert_eq!(g.screen, 6);
        g.touch(1, 0, 66, 190);
        g.tick(123);
        assert!(g.battle.as_ref().unwrap().fighters[0].pos.x > 7 * UNIT);
        g.touch(1, 2, 66, 190);
        assert!(g.touches.is_empty());
    }
    #[test]
    fn all_screens_render() {
        let mut g = Game::new("", true);
        for screen in 0..16 {
            g.screen = screen;
            g.battle = Some(Battle::new(7, Realm::Aurelon, Role::Foudre, 3, false));
            assert_eq!(g.frame(520).len(), 520 * 240 * 4);
        }
    }
    #[test]
    fn tutorial_can_be_won_without_changing_combat_stats() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.start_battle(0);
        for tick in 0..4800u64 {
            if g.screen == 12 {
                break;
            }
            let b = g.battle.as_ref().unwrap();
            let mut input = b.bot_input(0);
            match g.lesson {
                0 => {
                    input.move_x = 500;
                }
                1 => input.shoot = true,
                2 => input.dash = true,
                3 => input.skill = true,
                _ => {}
            }
            g.key_input = input;
            g.tick(1000 + tick / 30);
        }
        assert_eq!(
            g.screen, 12,
            "A normal-input agent must be able to finish the tutorial"
        );
        assert!(g.battle.as_ref().unwrap().fighters[0].kills > 0);
        g.tap(260, 168);
        assert_eq!(g.screen, 7);
        assert!(g.save.tutorial_done);
    }
}
