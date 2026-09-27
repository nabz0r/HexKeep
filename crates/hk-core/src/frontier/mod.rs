//! The Confins campaign is a local save domain. Network matches retain their original simulation.
mod actions;
mod cinematics;
mod combat;
mod data;
mod entities;
mod missions;
#[cfg(test)]
mod tests;
mod world;
use super::{Game, Input};
use cinematics::Timeline;
use entities::*;
use missions::Progress;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use world::UNIT;
#[derive(Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub version: u8,
    pub unlocked: u8,
    pub quests: BTreeMap<String, Progress>,
    pub tracked: String,
    pub seen: BTreeSet<String>,
    pub exploration: Vec<Vec<bool>>,
    pub relics: [[bool; 4]; 3],
    pub rescued: [bool; 3],
    pub bosses: [bool; 3],
    pub run: Option<Run>,
    pub visits: u32,
}
impl Default for Campaign {
    fn default() -> Self {
        Self {
            version: 1,
            unlocked: 0,
            quests: BTreeMap::new(),
            tracked: String::new(),
            seen: BTreeSet::new(),
            exploration: vec![vec![false; world::CELLS]; 3],
            relics: [[false; 4]; 3],
            rescued: [false; 3],
            bosses: [false; 3],
            run: None,
            visits: 0,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Run {
    pub zone: u8,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub projectiles: Vec<Projectile>,
    pub effects: Vec<Effect>,
    pub beacons: [bool; 3],
    pub relics: [bool; 4],
    pub rescued: bool,
    pub explored: Vec<bool>,
    pub tick: u32,
    pub kills: u32,
    pub victory: bool,
    pub rewarded: bool,
    pub scene: Option<Timeline>,
    pub hitstop: u8,
    pub impact_id: u32,
    pub phase_events: u8,
    pub next_monster: u32,
    #[serde(skip)]
    pub pending: Controls,
}
impl Campaign {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.unlocked <= 2
            && self.exploration.len() == 3
            && self.exploration.iter().all(|v| v.len() == world::CELLS)
            && self.run.as_ref().is_none_or(|r| {
                r.zone <= self.unlocked
                    && r.zone < 3
                    && r.explored.len() == world::CELLS
                    && r.monsters.len() <= 48
                    && r.projectiles.len() <= 128
                    && r.effects.len() <= 64
                    && r.player.combo < 4
                    && r.player.frames < 180
                    && r.player.flasks <= 3
                    && r.player.hp >= 0
                    && r.player.hp <= r.player.max_hp
                    && world::free(data::zone(r.zone), r.player.pos)
                    && r.monsters
                        .iter()
                        .all(|m| m.kind < 5 && world::free(data::zone(r.zone), m.pos))
                    && r.scene.as_ref().is_none_or(Timeline::valid)
            })
    }
    fn scene(&mut self, id: String) {
        if self.seen.insert(id.clone()) {
            if let Some(r) = &mut self.run {
                r.scene = Timeline::new(id);
                r.pending = Controls::default();
            }
        }
    }
    fn remember(&mut self, r: &Run) {
        for (a, b) in self.exploration[r.zone as usize]
            .iter_mut()
            .zip(&r.explored)
        {
            *a |= *b;
        }
        self.relics[r.zone as usize] = r.relics;
        self.rescued[r.zone as usize] = r.rescued;
        self.bosses[r.zone as usize] |= r.victory;
    }
}
impl Run {
    fn new(zone: u8, stats: (i32, i32, i32, u16), campaign: &Campaign) -> Self {
        let z = data::zone(zone);
        let mut monsters = Vec::new();
        // Camps are authored around objectives. Each contains an identifiable mixture of behaviours.
        for (camp, p) in z
            .beacons
            .iter()
            .chain(std::iter::once(&z.rescue))
            .enumerate()
        {
            for (i, (dx, dy)) in [(-2, 1), (2, 2), (-1, 3), (3, -1)].into_iter().enumerate() {
                let mut pos = world::at([p[0] + dx, p[1] + dy]);
                if !world::free(z, pos) {
                    pos = world::at(*p);
                }
                let kind = ((i + camp + zone as usize) % 4) as u8;
                monsters.push(Monster::new(monsters.len() as u32, kind, pos, zone));
            }
        }
        monsters.push(Monster::new(16, 4, world::at(z.boss), zone));
        let mut run = Self {
            zone,
            player: Player::new(world::at(z.spawn), stats),
            monsters,
            projectiles: vec![],
            effects: vec![],
            beacons: [false; 3],
            relics: campaign.relics[zone as usize],
            rescued: campaign.rescued[zone as usize],
            explored: campaign.exploration[zone as usize].clone(),
            tick: 0,
            kills: 0,
            victory: false,
            rewarded: false,
            scene: None,
            hitstop: 0,
            impact_id: 0,
            phase_events: 0,
            next_monster: 17,
            pending: Controls::default(),
        };
        world::reveal(z, run.player.pos, &mut run.explored);
        run
    }
    fn interaction(&self) -> Option<(&'static str, usize, String)> {
        let z = data::zone(self.zone);
        let p = self.player.pos;
        let near = |v| p.dist2(world::at(v)) < (UNIT as i64 * 2).pow(2);
        for (i, pos) in z.beacons.iter().enumerate() {
            if !self.beacons[i] && near(*pos) {
                return Some(("beacon", i, "Rallumer la balise".into()));
            }
        }
        for (i, pos) in z.relics.iter().enumerate() {
            if !self.relics[i] && near(*pos) {
                return Some(("relic", i, "Recueillir la mémoire".into()));
            }
        }
        if !self.rescued && near(z.rescue) {
            return Some(("rescue", 0, "Secourir le voyageur".into()));
        }
        None
    }
}
impl Game {
    pub(crate) fn frontier_controls(
        &mut self,
        mx: i16,
        my: i16,
        attack: bool,
        dash: bool,
        skill: bool,
    ) {
        self.key_input = Input {
            move_x: mx.clamp(-1024, 1024),
            move_y: my.clamp(-1024, 1024),
            shoot: attack,
            dash,
            skill,
            ..Default::default()
        };
    }
    pub(super) fn frontier_screen(&mut self, screen: u8) {
        self.screen = screen;
        self.key_input = Input::default();
        self.touches.clear();
        if let Some(r) = &mut self.save.frontier.run {
            r.pending = Controls::default();
        }
    }
    pub(crate) fn frontier_tick(&mut self) {
        if !matches!(self.screen, 52 | 53) {
            return;
        }
        let Some(mut run) = self.save.frontier.run.take() else {
            self.screen = 50;
            return;
        };
        if self.screen == 53 {
            if run.scene.as_mut().is_some_and(|s| s.advance(false)) {
                run.scene = None;
                self.screen = if run.victory { 55 } else { 52 };
                self.dirty = true;
            }
        } else {
            let input = self.key_input;
            let kills = run.kills;
            let old_impact = run.impact_id;
            let old_hp = run.player.hp;
            let old_state = run.player.state;
            run.step(Controls {
                mx: input.move_x,
                my: input.move_y,
                attack: input.shoot,
                dash: input.dash,
                skill: input.skill,
            });
            if run.impact_id != old_impact {
                self.sound = if run.player.hp < old_hp { 2 } else { 3 };
                if self.save.settings.haptics {
                    self.haptic = 1;
                }
            } else if old_state != State::Strike && run.player.state == State::Strike {
                self.sound = 1;
            }
            self.key_input.dash = false;
            self.key_input.skill = false;
            if run.kills > kills {
                self.save
                    .frontier
                    .progress(run.zone, "kills", run.kills - kills);
                self.save.kills += run.kills - kills;
                // Deterministic drops: a camp gives one item and dust, already in the persistent bag.
                self.save.journey.dust += 2 * (run.kills - kills);
                if run.kills / 4 > kills / 4 {
                    let seed = self.save.frontier.visits as u64 * 97
                        + run.kills as u64 * 11
                        + run.zone as u64;
                    let item = self.save.journey.make_item(seed, (run.zone > 0) as u8);
                    self.save.journey.store(item);
                }
                self.dirty = true;
            }
            if run.victory && !run.rewarded {
                run.rewarded = true;
                self.save.frontier.unlocked =
                    self.save.frontier.unlocked.max((run.zone + 1).min(2));
                self.save.journey.victories += 1;
                self.save.journey.dust += 30;
                let item = self
                    .save
                    .journey
                    .make_item(self.save.frontier.visits as u64 * 137 + run.zone as u64, 2);
                self.save.journey.store(item);
                self.save.frontier.sync_main(&run);
                self.save.frontier.remember(&run);
                self.sound = 4;
                if self.save.settings.haptics {
                    self.haptic = 2;
                }
                self.dirty = true;
                let id = format!("outro-{}", run.zone);
                if self.save.frontier.seen.insert(id.clone()) {
                    run.scene = Timeline::new(id);
                    self.screen = 53;
                } else {
                    self.screen = 55;
                }
            }
            if run.player.hp <= 0 {
                self.screen = 55;
                self.dirty = true;
            }
            if self.ticks % 60 == 0 {
                self.save.frontier.remember(&run);
                self.dirty = true;
            }
        }
        self.save.frontier.run = Some(run);
    }
    pub(crate) fn frontier_view(&self) -> serde_json::Value {
        let f = &self.save.frontier;
        let (hp, power, guard, haste) = self.save.journey.stats();
        let loadout =
            serde_json::json!({"max_hp":260+hp,"power":28+power/2,"guard":guard,"haste":haste});
        let zones: Vec<_> = data::catalog()
            .zones
            .iter()
            .map(|z| {
                let mut v = serde_json::to_value(z).unwrap();
                v["unlocked"] = serde_json::json!(z.id <= f.unlocked);
                v["complete"] = serde_json::json!(f.bosses[z.id as usize]);
                v["exploration"] = serde_json::json!(
                    f.exploration[z.id as usize]
                        .iter()
                        .enumerate()
                        .filter(|(i, v)| **v
                            || f.run
                                .as_ref()
                                .is_some_and(|r| r.zone == z.id && r.explored[*i]))
                        .count()
                        * 100
                        / world::CELLS
                );
                v
            })
            .collect();
        let interaction = f.run.as_ref().and_then(|r| r.interaction()).map(
            |(kind, index, label)| serde_json::json!({"kind":kind,"index":index,"label":label}),
        );
        serde_json::json!({"title":data::catalog().title,"loadout":loadout,"zones":zones,"quests":f.quest_view(),"run":f.run,"scene":f.run.as_ref().and_then(|r|r.scene.as_ref().map(Timeline::view)),"interaction":interaction,"tracked":f.tracked,"monsters":data::catalog().monsters,"chronicles":data::CHRONICLES,"unlocked":f.unlocked,"completed":f.bosses.iter().all(|b|*b)})
    }
}
