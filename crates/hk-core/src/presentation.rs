use super::*;
impl Game {
    /// Presentation-only data: never includes the saved secret or private device material.
    pub fn presentation(&mut self, width: i32) -> String {
        let ui = self.canvas(width, true).ui;
        let cells:Vec<_>=offsets(self.save.world.current,3).into_iter().map(|(id,q,r)|{
            let c=self.save.world.cells.get(&id);
            serde_json::json!({"id":format!("{id:x}"),"q":q,"r":r,"clear":c.is_some_and(|v|v.clear),"bastion":c.and_then(|v|v.bastion).map(|v|v.index()),"discovered":self.save.journey.discovered.contains(&id),"region":adventure::region_name(id),"poi":adventure::region(id),"current":id==self.save.world.current,"selected":id==self.selected})
        }).collect();
        serde_json::json!({"screen":self.screen,"width":self.width,"ticks":self.ticks,"created":self.save.created,
            "intro_seen":self.save.introduction_seen,"tutorial":self.tutorial,"lesson":self.lesson,
            "realm":self.save.realm.index(),"role":self.save.role.index(),"name":self.save.name,
            "realm_name":self.save.realm.name(),"role_name":self.save.role.name(),"music":self.save.settings.music,
            "accessible":self.save.settings.accessible,"effects":self.save.sound_effects,"haptics":self.save.settings.haptics,
            "journey":self.journey_view(),"xp":self.save.expansion.campaign.xp,"kills":self.save.kills,"equipped":self.save.expansion.campaign.equipped,
            "battle":self.battle,"battle_mode":self.battle_mode,"expedition":self.expedition,
            "guidance":self.battle.as_ref().zip(self.expedition.as_ref()).map(|(b,run)|discoveries::view(b,run)),
            "bestiary":(0..7).map(|i|serde_json::json!({"kind":i,"name":discoveries::NAMES[i],"tactic":discoveries::TACTICS[i],"defeated":self.save.journey.bestiary.get(&(i as u8)).copied().unwrap_or(0)})).collect::<Vec<_>>(),
            "local":self.online.as_ref().map(|o|o.rollback.local).unwrap_or(0),"online":self.online.is_some(),
            "banner":self.save.world.banner,"selected_current":self.selected==self.save.world.current,"gps":self.save.world.gps,
            "peers":self.peer_count,"latency":self.latency,"cells":cells,"emblem":self.save.expansion.campaign.draft,
            "message":if self.ticks<self.message_until{&self.message}else{""},"ui":ui,
            "party":self.peers.values().map(|p|serde_json::json!({"name":p.player.name,"realm":p.player.realm.name()})).collect::<Vec<_>>(),"searching":self.node.is_some(),"addresses":self.addresses,"blocked":self.storage_error||!self.is_dev}).to_string()
    }
    pub fn ui_action(&mut self, action: &str) {
        if self.storage_error || !self.is_dev {
            return;
        }
        if self.journey_action(action) {
            return;
        }
        self.key_input = Input::default();
        self.touches.clear();
        match action {
            "continue" => self.screen = if self.save.created { 7 } else { 0 },
            "prologue" => {
                if self.battle.is_some() {
                    self.end_battle();
                }
                if !self.save.created {
                    self.save.created = true;
                    self.event(Kind::Watch, 1);
                }
                self.save.introduction_seen = true;
                self.dirty = true;
                self.start_battle(0);
            }
            "expedition" => {
                if self.save.created {
                    if self.screen == 12 {
                        self.end_battle();
                    }
                    self.start_battle(8);
                }
            }
            "finish" => self.end_battle(),
            "pause" => {
                if self.screen == 6 {
                    self.screen = 14;
                }
            }
            "resume" => {
                if self.battle.is_some() {
                    self.screen = 6;
                }
            }
            "home" => {
                if self.screen == 28
                    || (!matches!(self.screen, 6 | 14 | 12) && self.expansion.court.is_some())
                {
                    self.battle = None;
                    self.expedition = None;
                    self.expansion.court = None;
                    self.screen = 7;
                } else if self.battle.is_some() && matches!(self.screen, 6 | 14 | 12 | 40 | 42) {
                    self.end_battle();
                } else {
                    self.screen = 7;
                }
            }
            "campaign" => self.screen = 20,
            "chronicle" => self.screen = 8,
            "gps" => self.native_action = 1,
            "walk" => {
                if self.selected != self.save.world.current {
                    self.save.world.gps = false;
                    self.native_action = 15;
                    self.save.world.enter(self.selected, self.now);
                    self.dirty = true;
                    self.toast("Déplacement de développement.");
                }
            }
            "found" => {
                let msg = match self.save.world.found(self.save.realm) {
                    Ok(()) => {
                        self.event(Kind::Foundation, 0);
                        "Ton bastion se dresse."
                    }
                    Err(e) => e,
                };
                self.toast(msg);
            }
            "banner" => {
                self.save.world.toggle_banner(self.now);
                self.event(Kind::Banner, self.save.world.banner as u32);
            }
            "shop" => self.screen = 24,
            "season" => self.screen = 23,
            "house" => self.screen = 25,
            "network" => self.screen = 16,
            "connect" => self.start_network(),
            "duel" => self.challenge(false),
            "field" => self.challenge(true),
            "address" => {
                self.start_network();
                self.native_action = 14;
            }
            "identity" => self.screen = 30,
            "settings" => self.screen = 10,
            "codex" => self.screen = 9,
            "role" => self.screen = 3,
            "music" => {
                self.save.settings.music = !self.save.settings.music;
                self.dirty = true;
            }
            "contrast" => {
                self.save.settings.accessible = !self.save.settings.accessible;
                self.dirty = true;
            }
            "effects" => {
                self.save.sound_effects = !self.save.sound_effects;
                self.dirty = true;
            }
            "haptics" => {
                self.save.settings.haptics = !self.save.settings.haptics;
                self.dirty = true;
            }
            "siege" => self.start_battle(5),
            "night" => self.start_battle(7),
            _ => {
                if let Some(id) = action
                    .strip_prefix("cell:")
                    .and_then(|v| u64::from_str_radix(v, 16).ok())
                {
                    if offsets(self.save.world.current, 3)
                        .iter()
                        .any(|(cell, _, _)| *cell == id)
                    {
                        self.selected = id;
                    }
                }
            }
        }
    }
    pub fn hero(&mut self, realm: u8, role: u8) {
        if self.screen == 0
            || self.screen == 2
            || self.screen == 3
            || self.screen == 7
            || self.screen == 10
        {
            self.save.realm = Realm::from_index(realm as usize);
            self.save.role = Role::from_index(role as usize);
            self.dirty = true;
        }
    }
    pub fn controls(
        &mut self,
        mx: i16,
        my: i16,
        ax: i16,
        ay: i16,
        auto: bool,
        dash: bool,
        skill: bool,
    ) {
        if self.screen != 6 {
            self.key_input = Input::default();
            return;
        }
        let (mx, my, ax, ay) = (
            mx.clamp(-1024, 1024),
            my.clamp(-1024, 1024),
            ax.clamp(-1024, 1024),
            ay.clamp(-1024, 1024),
        );
        let mut input = Input {
            move_x: mx,
            move_y: my,
            aim_x: ax,
            aim_y: ay,
            shoot: ax.abs() > 100 || ay.abs() > 100,
            dash,
            skill,
        };
        if auto && !input.shoot {
            if let Some(b) = &self.battle {
                let local = self
                    .online
                    .as_ref()
                    .map(|o| o.rollback.local as usize)
                    .unwrap_or(0);
                let f = &b.fighters[local];
                if let Some(t) = b
                    .fighters
                    .iter()
                    .filter(|v| {
                        v.id != f.id
                            && v.realm != f.realm
                            && v.hp > 0
                            && f.pos.dist2(v.pos) < (10 * UNIT) as i64 * (10 * UNIT) as i64
                            && hk_sim::navigation::clear_line(f.pos, v.pos, &b.obstacles, 24)
                    })
                    .min_by_key(|v| f.pos.dist2(v.pos))
                {
                    let lead = (isqrt(f.pos.dist2(t.pos) as u64) as i32
                        / b.codex.projectile[f.realm.index()].max(1))
                    .min(8);
                    let predicted =
                        Vec2::new(t.pos.x + t.stride.x * lead, t.pos.y + t.stride.y * lead);
                    let target =
                        if hk_sim::navigation::clear_line(f.pos, predicted, &b.obstacles, 24) {
                            predicted
                        } else {
                            t.pos
                        };
                    let a = Vec2::new(target.x - f.pos.x, target.y - f.pos.y).scaled(1024);
                    input.aim_x = a.x as i16;
                    input.aim_y = a.y as i16;
                    input.shoot = true;
                }
            }
        }
        self.key_input = input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_save_migrates_without_changing_identity_or_progress() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.save.kills = 87;
        g.save.expansion.campaign.xp = 320;
        let mut saved = serde_json::to_value(&g.save).unwrap();
        saved.as_object_mut().unwrap().remove("introduction_seen");
        saved.as_object_mut().unwrap().remove("sound_effects");
        saved.as_object_mut().unwrap().remove("journey");
        let restored = Game::new(&saved.to_string(), true);
        assert_eq!(restored.save.secret, g.save.secret);
        assert_eq!(restored.save.kills, 87);
        assert_eq!(restored.save.expansion.campaign.xp, 320);
        assert!(restored.save.sound_effects);
        assert_eq!(restored.save.journey.items.len(), 3);
        assert!(!restored.save.introduction_seen);
    }
    #[test]
    fn guided_start_and_manual_controls_are_safe() {
        let mut g = Game::new("", true);
        g.hero(2, 2);
        g.ui_action("prologue");
        assert!(g.save.created && g.save.introduction_seen);
        assert_eq!(g.screen, 6);
        g.controls(i16::MIN, 0, i16::MIN, 0, false, true, true);
        assert_eq!(g.key_input.move_x, -1024);
        assert!(g.key_input.shoot);
        g.tick(100);
        g.ui_action("pause");
        let pos = g.battle.as_ref().unwrap().fighters[0].pos;
        g.controls(1024, 0, 0, 0, true, true, true);
        g.tick(101);
        assert_eq!(pos, g.battle.as_ref().unwrap().fighters[0].pos);
        assert_eq!(g.key_input, Input::default());
        let p: serde_json::Value = serde_json::from_str(&g.presentation(584)).unwrap();
        assert!(p.get("secret").is_none());
        assert!(p["ui"].is_array());
    }
    #[test]
    fn all_three_realms_can_complete_expedition_and_claim_once() {
        for realm in 0..3 {
            let mut g = Game::new("", true);
            g.save.created = true;
            g.save.realm = Realm::from_index(realm);
            g.ui_action("expedition");
            let points = [(6, 4), (15, 12), (25, 5)];
            for (x, y) in points {
                for _ in 0..45 {
                    let b = g.battle.as_mut().unwrap();
                    b.fighters[0].pos = Vec2::new(x * UNIT, y * UNIT);
                    b.fighters[0].invulnerable = 10;
                    for f in b.fighters.iter_mut().skip(1) {
                        f.hp = 0;
                        f.respawn = u16::MAX;
                    }
                    g.tick(100);
                }
            }
            assert_eq!(g.expedition.as_ref().unwrap().wave, 1);
            let b = g.battle.as_mut().unwrap();
            assert_ne!(b.fighters[0].realm, b.fighters[1].realm);
            assert_eq!(b.fighters[1].hp, 420);
            b.fighters[1].hp = 0;
            b.fighters[1].respawn = 120;
            g.tick(101);
            assert_eq!(g.screen, 12);
            assert!(g.expedition.as_ref().unwrap().victory);
            g.ui_action("finish");
            assert_eq!(g.screen, 7);
            let xp = g.save.expansion.campaign.xp;
            assert!(xp >= 140);
            g.ui_action("finish");
            assert_eq!(g.save.expansion.campaign.xp, xp);
        }
    }
    #[test]
    fn expedition_stops_after_three_lost_lights() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("expedition");
        g.battle.as_mut().unwrap().fighters[0].deaths = 3;
        g.tick(100);
        assert_eq!(g.screen, 12);
        assert!(!g.expedition.as_ref().unwrap().victory);
    }
}
