//! Local PvE progression. Equipment is intentionally absent from network match setup.
use super::*;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub slot: u8,
    pub rarity: u8,
    pub vitality: i32,
    pub power: i32,
    pub guard: i32,
    pub haste: u16,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Journey {
    pub items: Vec<Item>,
    pub equipped: [u64; 3],
    pub next_id: u64,
    pub dust: u32,
    pub discovered: BTreeSet<u64>,
    pub completed: Vec<String>,
    pub outings: u32,
    pub recent: Vec<Item>,
}
impl Default for Journey {
    fn default() -> Self {
        let mut s = Self {
            items: vec![],
            equipped: [0; 3],
            next_id: 1,
            dust: 0,
            discovered: BTreeSet::new(),
            completed: vec![],
            outings: 0,
            recent: vec![],
        };
        for slot in 0..3 {
            let item = s.make_item(slot as u64, 0);
            s.equipped[slot] = item.id;
            s.items.push(item);
        }
        s
    }
}
impl Journey {
    pub fn make_item(&mut self, seed: u64, rarity: u8) -> Item {
        let slot = (seed % 3) as u8;
        let tier = rarity.min(3);
        let p = tier as i32 + 1;
        let names = [
            ["Lame du guetteur", "Arc des chemins", "Sceptre de braise"],
            [
                "Manteau de veille",
                "Cuirasse des marches",
                "Cape du crépuscule",
            ],
            ["Sceau de l’aube", "Éclat de mémoire", "Boussole d’ambre"],
        ];
        let name = names[slot as usize][((seed / 3) % 3) as usize].to_string();
        let id = self.next_id;
        self.next_id += 1;
        Item {
            id,
            name,
            slot,
            rarity: tier,
            vitality: if slot == 1 { p * 12 } else { p * 3 },
            power: if slot == 0 { p * 8 } else { p * 2 },
            guard: if slot == 1 { p * 8 } else { 0 },
            haste: if slot == 2 { tier as u16 + 1 } else { 0 },
        }
    }
    pub fn stats(&self) -> (i32, i32, i32, u16) {
        let mut v = (0, 0, 0, 0);
        for i in &self.items {
            if self.equipped[i.slot as usize] == i.id {
                v.0 += i.vitality;
                v.1 += i.power;
                v.2 += i.guard;
                v.3 += i.haste;
            }
        }
        v
    }
    pub fn equip(&mut self, id: u64) -> bool {
        if let Some(i) = self.items.iter().find(|i| i.id == id) {
            self.equipped[i.slot as usize] = id;
            true
        } else {
            false
        }
    }
    pub fn salvage(&mut self, id: u64) -> bool {
        if self.equipped.contains(&id) {
            return false;
        }
        if let Some(at) = self.items.iter().position(|i| i.id == id) {
            self.dust += 4 * (self.items.remove(at).rarity as u32 + 1);
            true
        } else {
            false
        }
    }
    pub fn store(&mut self, item: Item) {
        self.recent.push(item.clone());
        if self.items.len() < 60 {
            self.items.push(item)
        } else {
            self.dust += 4 * (item.rarity as u32 + 1);
        }
    }
}
pub fn region(cell: u64) -> usize {
    ((cell ^ (cell >> 17) ^ (cell >> 31)) % 3) as usize
}
pub fn region_name(cell: u64) -> &'static str {
    [
        "Les Jardins de cendre",
        "Le Val des cloches",
        "Les Hauts de verre",
    ][region(cell)]
}
pub fn contract_name(kind: u8) -> &'static str {
    [
        "Les trois feux",
        "La chasse aux ombres",
        "Les mémoires perdues",
    ][kind.min(2) as usize]
}
#[derive(Clone, Serialize)]
pub struct Drop {
    pub pos: Vec2,
    pub collected: bool,
    pub value: u32,
}
impl Game {
    pub(crate) fn journey_view(&self) -> serde_json::Value {
        let j = &self.save.journey;
        let (hp, power, armor, haste) = j.stats();
        let base_hp = self
            .save
            .expansion
            .authority
            .state(&self.save.ledger)
            .map(|s| s.codex.hp[self.save.realm.index()])
            .unwrap_or(self.save.realm.hp());
        let cycle = self.now / 1800;
        let cell = self.save.world.current;
        let events:Vec<_>=(0..3).map(|kind|serde_json::json!({"kind":kind,"name":contract_name(kind),"done":j.completed.contains(&format!("{cell}:{cycle}:{kind}"))})).collect();
        serde_json::json!({"items":j.items,"equipped":j.equipped,"dust":j.dust,"discovered":j.discovered.len(),"outings":j.outings,"recent":j.recent,"level":1+self.save.expansion.campaign.xp/250,"vitality":base_hp+60+hp,"power":100+power,"armor":100+armor,"haste":haste,"region":region_name(cell),"events":events,"remaining":1800-self.now%1800,"story":(["La gardienne Éline a retrouvé une carte sans routes. Tes pas lui rendent ses chemins.","Les cloches ne sonnent plus pour les rois. Elles répondent aux lanternes des voyageurs.","Sous le verre repose la mémoire des trois serments. Aucun royaume ne peut veiller seul."][region(cell)])})
    }
    pub(crate) fn journey_reward(&mut self, b: &Battle, run: &Expedition) {
        let j = &mut self.save.journey;
        j.recent.clear();
        j.outings += 1;
        j.dust += run.dust;
        let first = run.victory && !j.completed.contains(&run.event_id);
        if first {
            j.completed.push(run.event_id.clone());
            if j.completed.len() > 128 {
                j.completed.remove(0);
            }
            self.save.expansion.campaign.xp += 60;
        }
        for i in 0..3 {
            if run.caches[i] {
                let item = j.make_item(
                    b.seed.wrapping_add(j.outings as u64 * 31 + i as u64),
                    if i == 2 { 1 } else { 0 },
                );
                j.store(item);
            }
        }
        if run.victory {
            let item = j.make_item(
                b.seed.wrapping_add(j.outings as u64 * 997),
                if first {
                    if j.outings % 5 == 0 {
                        3
                    } else {
                        2
                    }
                } else {
                    1
                },
            );
            j.store(item);
        }
    }
    pub(crate) fn journey_action(&mut self, action: &str) -> bool {
        if action == "inventory" {
            if self.battle.is_none() {
                self.screen = 40;
            }
            return true;
        }
        if action == "journal" {
            if self.battle.is_none() {
                self.screen = 41;
            }
            return true;
        }
        if action == "heal" {
            if self.screen == 6 {
                if let (Some(b), Some(run)) = (&mut self.battle, &mut self.expedition) {
                    let p = &mut b.fighters[0];
                    if p.hp > 0 && p.hp < p.max_hp && run.flasks > 0 {
                        run.flasks -= 1;
                        p.hp = (p.hp + p.max_hp / 2).min(p.max_hp);
                        self.sound = 4;
                    }
                }
            }
            return true;
        }
        if let Some(kind) = action
            .strip_prefix("contract:")
            .and_then(|s| s.parse::<u8>().ok())
        {
            if kind <= 2 && self.save.created && self.battle.is_none() {
                self.start_battle(8);
                let cell = self.save.world.current;
                let run = self.expedition.as_mut().unwrap();
                run.kind = kind;
                run.event_id = format!("{cell}:{}:{kind}", self.now / 1800);
                self.save.journey.discovered.insert(cell);
                self.dirty = true;
            }
            return true;
        }
        if self.battle.is_some() {
            return false;
        }
        if let Some(id) = action.strip_prefix("equip:").and_then(|s| s.parse().ok()) {
            if self.save.journey.equip(id) {
                self.dirty = true;
                self.toast("Équipement ajusté pour ta prochaine expédition.");
            }
            return true;
        }
        if let Some(id) = action.strip_prefix("salvage:").and_then(|s| s.parse().ok()) {
            if self.save.journey.salvage(id) {
                self.dirty = true;
                self.toast("Objet recyclé en poussière de braise.");
            }
            return true;
        }
        if action == "forge" {
            if self.save.journey.dust >= 30 && self.save.journey.items.len() < 60 {
                let j = &mut self.save.journey;
                j.dust -= 30;
                let item = j.make_item(j.next_id * 17, 2);
                j.recent = vec![item.clone()];
                j.items.push(item);
                self.dirty = true;
                self.toast("La forge a façonné un objet rare.");
            } else {
                self.toast("La forge demande 30 poussières et une place libre.");
            }
            return true;
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leaving_a_replay_unlocks_the_refuge_without_awarding_its_kills() {
        let mut g = Game::new("", true);
        g.save.created = true;
        let mut b = Battle::new(10, Realm::Aurelon, Role::Foudre, 1, false);
        b.fighters[0].kills = 20;
        g.battle = Some(b);
        g.screen = 28;
        let xp = g.save.expansion.campaign.xp;
        g.ui_action("home");
        assert!(g.battle.is_none());
        assert_eq!(g.save.expansion.campaign.xp, xp);
        g.ui_action("inventory");
        assert_eq!(g.screen, 40);
        g.ui_action("contract:1");
        assert_eq!(g.screen, 6);
    }
    #[test]
    fn inventory_migration_equipment_salvage() {
        let mut j = Journey::default();
        assert_eq!(j.items.len(), 3);
        assert!(!j.salvage(1));
        let item = j.make_item(0, 3);
        let id = item.id;
        j.items.push(item);
        let old = j.stats();
        assert!(j.equip(id));
        assert!(j.stats().1 > old.1);
        assert!(j.salvage(1));
        assert!(!j.equip(1));
        let restored: Journey = serde_json::from_str(&serde_json::to_string(&j).unwrap()).unwrap();
        assert_eq!(restored.equipped, j.equipped);
    }
    #[test]
    fn pve_loadout_is_not_applied_to_pvp() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.start_battle(8);
        let hp = g.battle.as_ref().unwrap().fighters[0].max_hp;
        assert!(hp > g.save.realm.hp());
        g.ui_action("home");
        g.start_battle(1);
        assert_eq!(g.battle.as_ref().unwrap().fighters[0].power, 100);
        assert_eq!(
            g.battle.as_ref().unwrap().fighters[0].max_hp,
            g.save.realm.hp()
        );
    }
    #[test]
    fn rewards_are_consumed_once_and_inventory_locked_in_combat() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("contract:2");
        g.expedition.as_mut().unwrap().victory = true;
        g.expedition.as_mut().unwrap().caches = [true; 3];
        g.ui_action("inventory");
        assert_eq!(g.screen, 6);
        g.ui_action("finish");
        assert_eq!(g.save.journey.items.len(), 7);
        let xp = g.save.expansion.campaign.xp;
        g.ui_action("finish");
        assert_eq!(g.save.journey.items.len(), 7);
        assert_eq!(g.save.expansion.campaign.xp, xp);
    }
}
