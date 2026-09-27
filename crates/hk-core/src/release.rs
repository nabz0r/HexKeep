//! A local adventure edition. This does not unlock production network authority.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    version: u8,
    battle: Battle,
    expedition: Option<Expedition>,
    mode: u8,
    tutorial: bool,
    lesson: u8,
    result: bool,
}

impl Game {
    pub fn new_offline(snapshot: &str) -> Self {
        let mut game = Self::new(snapshot, false);
        game.offline = true;
        game.save.world.gps = false;
        game.save.world.banner = false;
        game.save.expansion.phare = false;
        game
    }

    pub(crate) fn offline_action_allowed(action: &str) -> bool {
        matches!(
            action,
            "continue"
                | "prologue"
                | "expedition"
                | "finish"
                | "pause"
                | "resume"
                | "home"
                | "walk"
                | "settings"
                | "music"
                | "contrast"
                | "effects"
                | "haptics"
                | "inventory"
                | "inventory_back"
                | "codex"
                | "codex_back"
                | "journal"
                | "heal"
                | "interact"
                | "forge"
                | "bag_sort"
        ) || [
            "cell:",
            "difficulty:",
            "contract:",
            "equip:",
            "salvage:",
            "f:",
            "bag_move:",
        ]
        .iter()
        .any(|prefix| action.starts_with(prefix))
    }

    pub(crate) fn checkpoint(&self) -> Option<Checkpoint> {
        if self.online.is_some()
            || !matches!(self.battle_mode, 0 | 8)
            || self.expansion.court.is_some()
        {
            return None;
        }
        Some(Checkpoint {
            version: 1,
            battle: self.battle.as_ref()?.clone(),
            expedition: self.expedition.clone(),
            mode: self.battle_mode,
            tutorial: self.tutorial,
            lesson: self.lesson,
            result: self.screen == 12,
        })
    }

    pub(crate) fn restore_checkpoint(&mut self, checkpoint: Checkpoint) {
        // Never reinterpret a future checkpoint or mismatched battle as a new run.
        let valid = checkpoint.version == 1
            && matches!(checkpoint.mode, 0 | 8)
            && checkpoint.battle.fighters.len() >= 2
            && checkpoint
                .battle
                .fighters
                .iter()
                .enumerate()
                .all(|(i, f)| f.id as usize == i)
            && (checkpoint.mode == 8) == checkpoint.expedition.is_some()
            && checkpoint
                .expedition
                .as_ref()
                .is_none_or(|r| r.explored.len() == 480 && r.kind <= 4);
        if !valid {
            self.storage_error = true;
            return;
        }
        self.battle = Some(checkpoint.battle);
        self.expedition = checkpoint.expedition;
        self.battle_mode = checkpoint.mode;
        self.tutorial = checkpoint.tutorial;
        self.lesson = checkpoint.lesson;
        self.inventory_return = 14;
        // The title remains visible; Continue explicitly resumes to a paused view.
        if checkpoint.result {
            self.battle.as_mut().unwrap().finished = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn play_is_solo_and_does_not_unlock_the_production_network() {
        let mut g = Game::new_offline("");
        assert!(!g.is_dev);
        assert!(
            !serde_json::from_str::<serde_json::Value>(&g.presentation(584)).unwrap()["blocked"]
                .as_bool()
                .unwrap()
        );
        for action in [
            "connect", "network", "address", "gps", "shop", "campaign", "identity", "banner",
        ] {
            g.ui_action(action);
            assert_eq!(g.screen, 0);
            assert_eq!(g.native_action, 0);
        }
        g.start_network();
        g.throne_unlock();
        g.start_battle(5);
        assert!(g.node.is_none());
        assert!(g.battle.is_none());
        assert!(!g.expansion.throne_open);
        g.ui_action("prologue");
        g.controls(1024, 0, 0, 0, false, false, false);
        g.tick(1000);
        assert_eq!(g.screen, 6);
        assert!(g.battle.as_ref().unwrap().tick > 0);
        let mut prod = Game::new("", false);
        prod.ui_action("prologue");
        prod.tick(1000);
        assert!(prod.battle.is_none());
    }
    #[test]
    fn adventure_checkpoint_preserves_combat_loot_and_pause_without_double_reward() {
        let mut g = Game::new_offline("");
        g.save.created = true;
        g.ui_action("contract:2");
        for i in 0..70 {
            g.controls(700, 0, 0, 0, true, false, false);
            g.tick(1000 + i);
        }
        g.battle.as_mut().unwrap().fighters[0].hp = 41;
        g.expedition.as_mut().unwrap().flasks = 1;
        g.expedition.as_mut().unwrap().sites[0].opened = true;
        let hash = g.battle.as_ref().unwrap().hash();
        let mut restored = Game::new_offline(&g.snapshot());
        assert!(!restored.storage_error);
        assert_eq!(restored.screen, 0);
        restored.tick(9999);
        assert_eq!(restored.battle.as_ref().unwrap().hash(), hash);
        restored.ui_action("continue");
        assert_eq!(restored.screen, 14);
        assert_eq!(restored.expedition.as_ref().unwrap().flasks, 1);
        assert!(restored.expedition.as_ref().unwrap().sites[0].opened);
        restored.ui_action("resume");
        restored.tick(10000);
        assert_ne!(restored.battle.as_ref().unwrap().hash(), hash);
        restored.expedition.as_mut().unwrap().victory = true;
        restored.screen = 12;
        let mut result = Game::new_offline(&restored.snapshot());
        result.ui_action("continue");
        assert_eq!(result.screen, 12);
        result.ui_action("finish");
        let after = result.snapshot();
        result.ui_action("finish");
        assert_eq!(after, result.snapshot());
        assert_eq!(result.save.journey.victories, 1);
        assert!(Game::new_offline(&after).battle.is_none());
    }
    #[test]
    fn old_saves_migrate_and_invalid_checkpoints_fail_closed() {
        let old = Game::new("", true).snapshot();
        let mut value: serde_json::Value = serde_json::from_str(&old).unwrap();
        value.as_object_mut().unwrap().remove("checkpoint");
        assert!(!Game::new_offline(&value.to_string()).storage_error);
        let mut g = Game::new_offline("");
        g.ui_action("prologue");
        let mut value: serde_json::Value = serde_json::from_str(&g.snapshot()).unwrap();
        value["checkpoint"]["version"] = 999.into();
        assert!(Game::new_offline(&value.to_string()).storage_error);
    }
}
