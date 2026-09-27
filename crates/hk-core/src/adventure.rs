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
    #[serde(default)]
    pub motif: u8,
    #[serde(default)]
    pub lore: String,
    #[serde(default)]
    pub catalog: u16,
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
    #[serde(default)]
    pub victories: u32,
    #[serde(default)]
    pub difficulty: u8,
    #[serde(default)]
    pub secrets: BTreeSet<String>,
    #[serde(default)]
    pub bestiary: BTreeMap<u8, u32>,
    #[serde(default)]
    pub collection: BTreeSet<u16>,
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
            victories: 0,
            difficulty: 0,
            secrets: BTreeSet::new(),
            bestiary: BTreeMap::new(),
            collection: BTreeSet::new(),
        };
        for slot in 0..3 {
            let item = s.make_item(slot as u64, 0);
            s.equipped[slot] = item.id;
            s.collection.insert(item.catalog);
            s.items.push(item);
        }
        s
    }
}
impl Journey {
    pub fn migrate(&mut self) {
        if self.victories == 0 && !self.completed.is_empty() {
            self.victories = self.completed.len() as u32;
        }
        // Preserve every historical item's identity and values. Old names used four
        // models per slot; add only their new catalogue and narrative metadata.
        for item in &mut self.items {
            if item.lore.is_empty() {
                let model = match item.name.as_str() {
                    "Arc des chemins" | "Cuirasse des marches" | "Éclat de mémoire" => 1,
                    "Sceptre de braise" | "Cape du crépuscule" | "Boussole d’ambre" => 2,
                    "Hache des cloches" | "Voile de rosée" | "Cloche silencieuse" => 3,
                    _ => 0,
                };
                item.catalog = item.slot as u16 * 12 + model;
                item.lore = "Porté sur les premiers chemins. Sa braise a traversé la nuit.".into();
            }
            self.collection.insert(item.catalog);
        }
        self.difficulty = self.difficulty.min(self.max_difficulty());
    }
    pub fn make_item(&mut self, seed: u64, rarity: u8) -> Item {
        let slot = (seed % 3) as u8;
        let tier = rarity.min(3);
        let p = tier as i32 + 1;
        let names = [
            [
                "Lame du guetteur",
                "Arc des chemins",
                "Sceptre de braise",
                "Hache des cloches",
                "Aiguille du givre",
                "Lance du passeur",
                "Faux de l’éclipse",
                "Bâton des murmures",
                "Épée du dernier feu",
                "Arc des lucioles",
                "Dague du chat noir",
                "Marteau du serment",
            ],
            [
                "Manteau de veille",
                "Cuirasse des marches",
                "Cape du crépuscule",
                "Voile de rosée",
                "Armure des racines",
                "Pelisse du voyageur",
                "Étoffe des songes",
                "Plastron de verre",
                "Tunique des étoiles",
                "Cotte du carillon",
                "Cape des neuf vies",
                "Manteau du retour",
            ],
            [
                "Sceau de l’aube",
                "Éclat de mémoire",
                "Boussole d’ambre",
                "Cloche silencieuse",
                "Graine de lune",
                "Larme de givre",
                "Dé du vagabond",
                "Sablier de cendre",
                "Anneau du phare",
                "Clochette de Minuit",
                "Étoile oubliée",
                "Clé sans serrure",
            ],
        ];
        let model = ((seed / 3) % 12) as usize;
        let name = names[slot as usize][model].to_string();
        let motif = ((seed / 37) % 4) as u8;
        let id = self.next_id;
        self.next_id += 1;
        Item {
            id,
            name,
            slot,
            rarity: tier,
            vitality: (if slot == 1 { p * 12 } else { p * 3 }) + (seed % 4) as i32,
            power: (if slot == 0 { p * 8 } else { p * 2 }) + ((seed >> 4) % 3) as i32,
            guard: if slot == 1 { p * 8 } else { 0 },
            haste: if slot == 2 { tier as u16 + 1 } else { 0 },
            motif,
            catalog: slot as u16 * 12 + model as u16,
            lore: [
                "Une braise veille encore dans ses gravures.",
                "Le givre conserve les promesses anciennes.",
                "Ses fibres se souviennent des pas de la forêt.",
                "Une étoile minuscule indique toujours le retour.",
            ][motif as usize]
                .into(),
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
        for motif in 0..4 {
            if self
                .items
                .iter()
                .filter(|i| self.equipped.contains(&i.id) && i.motif == motif)
                .count()
                >= 2
            {
                v.0 += 12;
                v.1 += 8;
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
        self.collection.insert(item.catalog);
        self.recent.push(item.clone());
        if self.items.len() < 60 {
            self.items.push(item)
        } else {
            self.dust += 4 * (item.rarity as u32 + 1);
        }
    }
    pub fn max_difficulty(&self) -> u8 {
        if self.victories >= 9 {
            2
        } else if self.victories >= 3 {
            1
        } else {
            0
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
        "La longue veillée",
        "La piste du passeur",
    ][kind.min(4) as usize]
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
        let events:Vec<_>=(0..5).map(|kind|serde_json::json!({"kind":kind,"name":contract_name(kind),"done":j.completed.contains(&format!("{cell}:{cycle}:{kind}"))})).collect();
        let next_goal = if j.victories < 3 {
            format!(
                "Encore {} victoire(s) pour ouvrir les Marches périlleuses.",
                3 - j.victories
            )
        } else if j.victories < 9 {
            format!(
                "Encore {} victoire(s) pour ouvrir l’Éclipse.",
                9 - j.victories
            )
        } else {
            format!(
                "Retrouve les neuf secrets des marches : {} / 9.",
                j.secrets.len()
            )
        };
        serde_json::json!({"items":j.items,"equipped":j.equipped,"dust":j.dust,"discovered":j.discovered.len(),"outings":j.outings,"recent":j.recent,"level":1+self.save.expansion.campaign.xp/250,"vitality":base_hp+60+hp,"power":100+power,"armor":100+armor,"haste":haste,"region":region_name(cell),"events":events,"remaining":1800-self.now%1800,"victories":j.victories,"difficulty":j.difficulty,"max_difficulty":j.max_difficulty(),"secrets":j.secrets,"secret_entries":discoveries::secret_entries(j),"collection":j.collection.len(),"bestiary":j.bestiary,"next_goal":next_goal,"in_battle":self.battle.is_some(),"inventory_return":self.inventory_return,"story":(["La gardienne Éline a retrouvé une carte sans routes. Tes pas lui rendent ses chemins.","Les cloches ne sonnent plus pour les rois. Elles répondent aux lanternes des voyageurs.","Sous le verre repose la mémoire des trois serments. Aucun royaume ne peut veiller seul."][region(cell)])})
    }
    pub(crate) fn journey_reward(&mut self, b: &Battle, run: &Expedition) {
        let j = &mut self.save.journey;
        j.outings += 1;
        if run.victory {
            j.victories += 1;
        }
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
            if run.caches[i] && !run.secured[i] {
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
                if run.tier == 2 {
                    3
                } else if first {
                    if j.outings % 5 == 0 {
                        3
                    } else {
                        2
                    }
                } else if run.tier == 1 {
                    2
                } else {
                    1
                },
            );
            j.store(item);
        }
    }
    pub(crate) fn journey_action(&mut self, action: &str) -> bool {
        if action == "inventory" {
            if self.battle.is_none() || matches!(self.screen, 6 | 14) {
                self.inventory_return = if self.battle.is_some() {
                    self.screen
                } else {
                    7
                };
                self.key_input = Input::default();
                self.touches.clear();
                self.screen = 40;
            }
            return true;
        }
        if action == "inventory_back" || action == "codex_back" {
            self.screen = if self.battle.is_some() || self.inventory_return == 41 {
                self.inventory_return
            } else {
                7
            };
            self.key_input = Input::default();
            self.touches.clear();
            return true;
        }
        if action == "codex" {
            if self.screen != 40 {
                self.inventory_return = if self.battle.is_some() {
                    self.screen
                } else if self.screen == 41 {
                    41
                } else {
                    7
                };
            }
            self.screen = 42;
            self.key_input = Input::default();
            self.touches.clear();
            return true;
        }
        if let Some(tier) = action
            .strip_prefix("difficulty:")
            .and_then(|s| s.parse::<u8>().ok())
        {
            if self.battle.is_none() && tier <= self.save.journey.max_difficulty() {
                self.save.journey.difficulty = tier;
                self.dirty = true;
            }
            return true;
        }
        if action == "interact" {
            if self.screen == 6 && self.expedition.is_some() {
                if let Some(message) = discoveries::interact(
                    self.battle.as_mut().unwrap(),
                    self.expedition.as_mut().unwrap(),
                    &mut self.save.journey,
                ) {
                    self.toast(&message);
                    self.sound = 4;
                    self.haptic = 2;
                    self.dirty = true;
                }
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
            if kind <= 4 && self.save.created && self.battle.is_none() {
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
        if let Some(id) = action.strip_prefix("equip:").and_then(|s| s.parse().ok()) {
            if self.battle.is_some() && (self.expedition.is_none() || self.screen != 40) {
                return true;
            }
            if self.save.journey.equip(id) {
                if let Some(b) = &mut self.battle {
                    let (hp, power, armor, haste) = self.save.journey.stats();
                    let p = &mut b.fighters[0];
                    p.max_hp = b.codex.hp[p.realm.index()] + 60 + hp;
                    p.hp = p.hp.min(p.max_hp);
                    p.power = 100 + power;
                    p.armor = p.armor.min(100 + armor);
                    p.haste = haste;
                }
                self.dirty = true;
                self.toast(
                    "Équipement ajusté. Deux pièces du même serment : +12 vie, +8 puissance.",
                );
            }
            return true;
        }
        if self.battle.is_some() {
            return action == "forge" || action.starts_with("salvage:");
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
                j.recent.clear();
                j.store(item);
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
    fn rewards_are_consumed_once_and_inventory_resumes_combat() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("contract:2");
        g.expedition.as_mut().unwrap().victory = true;
        g.expedition.as_mut().unwrap().caches = [true; 3];
        g.ui_action("inventory");
        assert_eq!(g.screen, 40);
        let tick = g.battle.as_ref().unwrap().tick;
        g.tick(100);
        assert_eq!(g.battle.as_ref().unwrap().tick, tick);
        g.ui_action("inventory_back");
        assert_eq!(g.screen, 6);
        g.ui_action("finish");
        assert_eq!(g.save.journey.items.len(), 7);
        let xp = g.save.expansion.campaign.xp;
        g.ui_action("finish");
        assert_eq!(g.save.journey.items.len(), 7);
        assert_eq!(g.save.expansion.campaign.xp, xp);
    }
}

#[cfg(test)]
mod eveil_tests {
    use super::*;
    #[test]
    fn legacy_journey_migration_preserves_values_and_equipment() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.save.journey.completed = vec!["a".into(), "b".into(), "c".into()];
        let equipped = g.save.journey.equipped;
        let old = g.save.journey.items[0].clone();
        let mut value: serde_json::Value = serde_json::from_str(&g.snapshot()).unwrap();
        let j = value["journey"].as_object_mut().unwrap();
        for key in [
            "victories",
            "difficulty",
            "secrets",
            "bestiary",
            "collection",
        ] {
            j.remove(key);
        }
        for i in j["items"].as_array_mut().unwrap() {
            let item = i.as_object_mut().unwrap();
            for k in ["motif", "lore", "catalog"] {
                item.remove(k);
            }
        }
        let restored = Game::new(&value.to_string(), true);
        let j = &restored.save.journey;
        assert_eq!(j.equipped, equipped);
        assert_eq!(j.items[0].power, old.power);
        assert_eq!(j.items[0].id, old.id);
        assert_eq!(j.victories, 3);
        assert_eq!(j.collection.len(), 3);
        assert!(!j.items[0].lore.is_empty());
    }
    #[test]
    fn field_equipment_pauses_and_never_grants_free_health() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("contract:0");
        g.battle.as_mut().unwrap().fighters[0].hp = 50;
        let item = g.save.journey.make_item(1, 3);
        let id = item.id;
        g.save.journey.store(item);
        g.ui_action("inventory");
        let tick = g.battle.as_ref().unwrap().tick;
        g.ui_action(&format!("equip:{id}"));
        g.tick(100);
        assert_eq!(g.battle.as_ref().unwrap().tick, tick);
        assert_eq!(g.battle.as_ref().unwrap().fighters[0].hp, 50);
        let bag = g.save.journey.items.len();
        g.save.journey.dust = 100;
        g.ui_action("forge");
        assert_eq!(g.save.journey.items.len(), bag);
        g.ui_action("codex");
        g.tick(101);
        assert_eq!(g.battle.as_ref().unwrap().tick, tick);
        g.ui_action("codex_back");
        assert_eq!(g.screen, 6);
        g.tick(102);
        assert_eq!(g.battle.as_ref().unwrap().tick, tick + 1);
    }
    #[test]
    fn difficulty_unlocks_and_forging_updates_collection() {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("difficulty:2");
        assert_eq!(g.save.journey.difficulty, 0);
        g.save.journey.victories = 3;
        g.ui_action("difficulty:1");
        assert_eq!(g.save.journey.difficulty, 1);
        g.ui_action("difficulty:2");
        assert_eq!(g.save.journey.difficulty, 1);
        g.save.journey.victories = 9;
        g.ui_action("difficulty:2");
        g.ui_action("contract:0");
        assert_eq!(g.expedition.as_ref().unwrap().tier, 2);
        g.ui_action("difficulty:0");
        assert_eq!(g.save.journey.difficulty, 2);
        g.ui_action("home");
        g.save.journey.dust = 30;
        g.ui_action("forge");
        let item = g.save.journey.items.last().unwrap();
        assert_eq!(item.rarity, 2);
        assert!(g.save.journey.collection.contains(&item.catalog));
    }
}
