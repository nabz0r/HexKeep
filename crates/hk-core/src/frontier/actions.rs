//! Validated UI commands, travel gates, equipment and interaction transactions.
use super::*;
impl Game {
    pub(crate) fn frontier_action(&mut self, action: &str) -> bool {
        let active = self.save.frontier.run.is_some();
        if action == "continue" && active {
            self.frontier_screen(54);
            return true;
        }
        if matches!(action, "pause" | "f:pause") && active {
            self.frontier_screen(54);
            return true;
        }
        if matches!(action, "resume" | "f:resume") && active {
            let r = self.save.frontier.run.as_ref().unwrap();
            let target = if r.scene.is_some() {
                53
            } else if r.victory || r.player.hp == 0 {
                55
            } else {
                52
            };
            self.frontier_screen(target);
            return true;
        }
        if action == "inventory" && active {
            self.inventory_return = 54;
            self.frontier_screen(40);
            return true;
        }
        if matches!(action, "inventory_back" | "codex_back") && active {
            self.frontier_screen(54);
            return true;
        }
        if action == "inventory" && !active && matches!(self.screen, 50 | 51 | 56) {
            self.inventory_return = self.screen;
            self.frontier_screen(40);
            return true;
        }
        if matches!(action, "inventory_back" | "codex_back")
            && !active
            && matches!(self.inventory_return, 50 | 51 | 56)
            && matches!(self.screen, 40 | 42)
        {
            self.frontier_screen(self.inventory_return);
            return true;
        }
        if active
            && (action.starts_with("contract:")
                || matches!(
                    action,
                    "prologue" | "expedition" | "network" | "campaign" | "forge" | "journal"
                ))
        {
            self.toast("Rentre au refuge depuis la pause avant de commencer une autre aventure.");
            return true;
        }
        if action == "home" && active {
            self.frontier_screen(7);
            return true;
        }
        if let Some(id) = action.strip_prefix("equip:").and_then(|s| s.parse().ok()) {
            if active {
                if self.screen == 40 && self.save.journey.equip(id) {
                    let stats = self.save.journey.stats();
                    self.save
                        .frontier
                        .run
                        .as_mut()
                        .unwrap()
                        .player
                        .equipment(stats);
                    self.dirty = true;
                }
                return true;
            }
        }
        if action == "bag_sort" {
            self.save.journey.items.sort_by_key(|i| {
                (
                    i.slot,
                    std::cmp::Reverse(i.rarity),
                    std::cmp::Reverse(i.power + i.vitality + i.guard),
                    i.id,
                )
            });
            self.dirty = true;
            return true;
        }
        if let Some(ids) = action.strip_prefix("bag_move:") {
            let ids: Vec<u64> = ids.split(':').filter_map(|s| s.parse().ok()).collect();
            if ids.len() == 2 {
                let items = &mut self.save.journey.items;
                if let (Some(a), Some(b)) = (
                    items.iter().position(|i| i.id == ids[0]),
                    items.iter().position(|i| i.id == ids[1]),
                ) {
                    items.swap(a, b);
                    self.dirty = true;
                }
            }
            return true;
        }
        let Some(command) = action.strip_prefix("f:") else {
            return false;
        };
        // The campaign cannot replace a network or legacy battle in progress.
        if self.battle.is_some() {
            self.toast("Termine l’aventure actuelle avant de rejoindre les Confins.");
            return true;
        }
        match command {
            "atlas" => self.frontier_screen(50),
            "journal" => self.frontier_screen(51),
            "chronicles" => self.frontier_screen(56),
            "home" => self.frontier_screen(7),
            "leave" => {
                if let Some(r) = self.save.frontier.run.take() {
                    self.save.frontier.remember(&r);
                }
                self.frontier_screen(50);
                self.dirty = true;
            }
            "stance" => {
                if self.screen == 52 {
                    if let Some(r) = &mut self.save.frontier.run {
                        if !matches!(r.player.state, State::Dead | State::Windup | State::Strike) {
                            r.player.stance = r.player.stance.next();
                            self.dirty = true;
                        }
                    }
                }
            }
            "heal" => {
                if self.screen == 52 {
                    if let Some(r) = &mut self.save.frontier.run {
                        if r.player.flasks > 0 && r.player.hp > 0 && r.player.hp < r.player.max_hp {
                            r.player.flasks -= 1;
                            r.player.hp = (r.player.hp + r.player.max_hp / 2).min(r.player.max_hp);
                            self.dirty = true;
                            self.sound = 4;
                        }
                    }
                }
            }
            "next" | "skip" => {
                if self.screen == 53 {
                    if let Some(r) = &mut self.save.frontier.run {
                        let done =
                            command == "skip" || r.scene.as_mut().is_some_and(|s| s.advance(true));
                        if done {
                            r.scene = None;
                            self.screen = if r.victory { 55 } else { 52 };
                        }
                        self.dirty = true;
                    }
                }
            }
            "interact" => self.frontier_interact(),
            _ => {
                if let Some(zone) = command
                    .strip_prefix("travel:")
                    .and_then(|s| s.parse::<u8>().ok())
                {
                    if zone <= self.save.frontier.unlocked
                        && zone < 3
                        && matches!(self.screen, 50 | 7)
                        && !active
                    {
                        self.save.created = true;
                        self.save.introduction_seen = true;
                        self.save.journey.recent.clear();
                        self.save.journey.outings += 1;
                        self.save.frontier.visits += 1;
                        let r = Run::new(zone, self.save.journey.stats(), &self.save.frontier);
                        self.save.frontier.run = Some(r);
                        self.save.frontier.accept(&format!("main-{zone}"));
                        self.save.frontier.scene(format!("intro-{zone}"));
                        self.frontier_screen(
                            if self.save.frontier.run.as_ref().unwrap().scene.is_some() {
                                53
                            } else {
                                52
                            },
                        );
                        self.dirty = true;
                    }
                } else if let Some(id) = command.strip_prefix("accept:") {
                    if self.save.frontier.accept(id) {
                        if let Some(q) = data::catalog().quests.iter().find(|q| q.id == id) {
                            let amount = match q.objective.as_str() {
                                "relics" => self.save.frontier.relics[q.zone as usize]
                                    .iter()
                                    .filter(|v| **v)
                                    .count() as u32,
                                "rescue" => u32::from(self.save.frontier.rescued[q.zone as usize]),
                                _ => 0,
                            };
                            if amount > 0 {
                                self.save.frontier.progress(q.zone, &q.objective, amount);
                            }
                        }
                        self.dirty = true;
                    }
                } else if let Some(id) = command.strip_prefix("track:") {
                    if self
                        .save
                        .frontier
                        .quests
                        .get(id)
                        .is_some_and(|p| p.accepted && !p.claimed)
                    {
                        self.save.frontier.tracked = id.into();
                        self.dirty = true;
                    }
                } else if let Some(id) = command.strip_prefix("claim:") {
                    if let Some(q) = data::catalog().quests.iter().find(|q| q.id == id) {
                        if let Some(p) = self.save.frontier.quests.get_mut(id) {
                            if p.accepted && !p.claimed && p.value >= q.target {
                                p.claimed = true;
                                self.save.journey.dust += q.dust;
                                self.save.expansion.campaign.xp += q.xp;
                                let item = self.save.journey.make_item(
                                    q.zone as u64 * 31 + q.xp as u64,
                                    if q.main { 2 } else { 1 },
                                );
                                self.save.journey.store(item);
                                self.toast("Mission accomplie · récompenses rangées dans le sac.");
                                self.dirty = true;
                            }
                        }
                    }
                }
            }
        }
        true
    }
    fn frontier_interact(&mut self) {
        if self.screen != 52 {
            return;
        }
        let Some(mut r) = self.save.frontier.run.take() else {
            return;
        };
        if let Some((kind, index, _)) = r.interaction() {
            let guarded = kind != "relic"
                && r.monsters.iter().any(|m| {
                    m.active && m.hp > 0 && m.pos.dist2(r.player.pos) < (UNIT as i64 * 4).pow(2)
                });
            if guarded {
                self.toast("Écarte les ombres proches avant d’interagir.");
            } else {
                match kind {
                    "beacon" => {
                        r.beacons[index] = true;
                        r.player.hp = (r.player.hp + r.player.max_hp / 4).min(r.player.max_hp);
                        self.save.frontier.sync_main(&r);
                        self.toast("Une balise répond. La route retrouve sa lumière.");
                    }
                    "relic" => {
                        r.relics[index] = true;
                        self.save.frontier.progress(r.zone, "relics", 1);
                        self.save.journey.dust += 8;
                        self.toast("Mémoire recueillie · +8 poussières.");
                    }
                    "rescue" => {
                        r.rescued = true;
                        self.save.frontier.progress(r.zone, "rescue", 1);
                        r.player.flasks = (r.player.flasks + 1).min(3);
                        self.toast("Le voyageur rejoint le refuge. Une fiole t’est offerte.");
                    }
                    _ => {}
                }
                self.save.frontier.remember(&r);
                self.dirty = true;
                self.sound = 4;
            }
        }
        if r.beacons.iter().all(|v| *v) {
            if let Some(boss) = r.monsters.iter_mut().find(|m| m.kind == 4 && !m.active) {
                boss.active = true;
                let id = format!("boss-{}", r.zone);
                self.save.frontier.seen.insert(id.clone());
                r.scene = Timeline::new(id);
                self.screen = 53;
            }
        }
        if r.zone == 1
            && r.beacons.iter().filter(|v| **v).count() == 2
            && self.save.frontier.seen.insert("revelation".into())
        {
            r.scene = Timeline::new("revelation".into());
            self.screen = 53;
        }
        r.pending = Controls::default();
        self.save.frontier.run = Some(r);
    }
}
