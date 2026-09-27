use super::*;
use hk_crown::{Authority, EdictKind, Scope};
use hk_season::{Campaign, House, Oath};
#[derive(Clone, Serialize, Deserialize)]
pub struct DevelopmentFixture {
    pub authority: Authority,
    pub throne: Hash,
}
pub fn fixture() -> DevelopmentFixture {
    serde_json::from_str(include_str!("../../../assets/dev/crown.json"))
        .expect("built DEV ceremony")
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ExpansionSave {
    #[serde(default)]
    pub held_shares: Vec<hk_crypto::identity::RecoveryShare>,
    #[serde(default)]
    pub test_guardians: Vec<Hash>,
    #[serde(default)]
    pub test_shares: Vec<hk_crypto::identity::RecoveryShare>,
    pub campaign: Campaign,
    pub authority: Authority,
    pub throne: Option<Hash>,
    pub witnesses: Vec<hk_crypto::identity::Witness>,
    pub case_files: Vec<String>,
    pub phare: bool,
    pub phare_anchor: u64,
}
impl Default for ExpansionSave {
    fn default() -> Self {
        Self {
            held_shares: vec![],
            test_guardians: vec![],
            test_shares: vec![],
            campaign: Campaign::default(),
            authority: fixture().authority,
            throne: None,
            witnesses: vec![],
            case_files: vec![],
            phare: false,
            phare_anchor: 0,
        }
    }
}
#[derive(Default)]
pub struct Expansion {
    pub codex_draft: Option<hk_crown::Codex>,
    pub codex_realm: usize,
    pub page: usize,
    pub item: usize,
    pub brush: u8,
    pub throne_open: bool,
    pub pending_edict: u8,
    pub memory_ready: bool,
    pub selected_memory: usize,
    pub bluetooth_status: String,
    pub device_status: String,
    pub court: Option<hk_net::Replay>,
    pub court_play: bool,
    pub court_speed: usize,
    pub court_pov: usize,
    pub native_text: String,
    pub hidden_taps: u8,
    pub peer_certificates: BTreeMap<Hash, hk_crypto::identity::Certificate>,
    pub ble_seen: BTreeMap<[u8; 16], u64>,
}
impl Game {
    pub fn guard_at(&self, cell: u64) -> u32 {
        let realm = self.save.realm;
        let nom = hk_crypto::public(&self.save.secret);
        let uptime = (self.save.world.seconds.min(86400) * 65536 / 86400) as u32;
        let witnesses = self
            .save
            .expansion
            .witnesses
            .iter()
            .filter(|w| w.subject == nom)
            .map(|w| w.observer)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let proximity = if neighbors(self.save.world.current, 2).contains(&cell) {
            65536
        } else {
            0
        };
        let device = hk_season::GuardDevice {
            nom,
            device: *blake3::hash(&self.session.certificate.device.public).as_bytes(),
            realm,
            confidence: hk_season::confidence(false, false, witnesses, 0, true),
            uptime,
            proximity,
        };
        let mut devices = vec![device];
        for (nom, peer) in &self.peers {
            if let Some(cert) = self.expansion.peer_certificates.get(nom) {
                let uptime = self
                    .save
                    .ledger
                    .events
                    .iter()
                    .filter(|e| {
                        e.body.author == *nom
                            && e.body.kind == Kind::Watch
                            && self.now.saturating_sub(e.body.time) <= 86400
                    })
                    .map(|e| e.body.value.min(60) as u64)
                    .sum::<u64>()
                    .min(86400);
                devices.push(hk_season::GuardDevice {
                    nom: *nom,
                    device: *blake3::hash(&cert.device.public).as_bytes(),
                    realm: peer.player.realm,
                    confidence: hk_season::confidence(
                        false,
                        self.save
                            .expansion
                            .authority
                            .banned(nom, Scope::All, self.now),
                        0,
                        0,
                        true,
                    ),
                    uptime: (uptime * 65536 / 86400) as u32,
                    proximity: if neighbors(peer.cell, 2).contains(&cell) {
                        65536
                    } else {
                        0
                    },
                });
            }
        }
        self.save
            .expansion
            .campaign
            .guard_bonus(realm, hk_season::guard_for(&devices, realm))
    }
    pub fn walls(&self) -> u32 {
        let cells = neighbors(self.save.world.current, 1);
        cells.iter().map(|c| self.guard_at(*c)).sum::<u32>() / cells.len().max(1) as u32
    }
    pub fn expansion_tick(&mut self) {
        if !self.save.created {
            return;
        }
        if self.ticks % 30 == 0 {
            let start = self
                .save
                .expansion
                .authority
                .genesis
                .body
                .codex
                .season_start;
            self.save.expansion.campaign.tick(self.now, start);
            if let Some(secret) = self.save.expansion.throne {
                let last = self
                    .save
                    .expansion
                    .authority
                    .seals
                    .last()
                    .map(|s| s.body.time)
                    .unwrap_or(0);
                if self.now.saturating_sub(last) >= 900 {
                    if self
                        .save
                        .expansion
                        .authority
                        .seal(&secret, &self.save.ledger, self.now)
                        .is_ok()
                    {
                        self.apply_canonical();
                        self.dirty = true;
                    }
                }
            }
        }
        if self.screen == 28 && self.expansion.court_play {
            let steps = match self.expansion.court_speed {
                0 => {
                    if self.ticks % 4 == 0 {
                        1
                    } else {
                        0
                    }
                }
                1 => {
                    if self.ticks % 2 == 0 {
                        1
                    } else {
                        0
                    }
                }
                2 => 1,
                3 => 2,
                _ => 4,
            };
            if let Some(r) = &mut self.expansion.court {
                for _ in 0..steps {
                    r.step();
                }
                self.battle = Some(r.battle.clone());
            }
        }
    }
    pub fn apply_canonical(&mut self) {
        if let Ok(c) = self.save.expansion.authority.state(&self.save.ledger) {
            let sealed_ids = self
                .save
                .expansion
                .authority
                .seals
                .last()
                .map(|s| s.body.event_ids.clone())
                .unwrap_or_default();
            let sealed_cells: std::collections::BTreeSet<_> = self
                .save
                .ledger
                .events
                .iter()
                .filter(|e| sealed_ids.contains(&e.id()))
                .map(|e| e.body.cell)
                .collect();
            for cell in self.save.world.cells.values_mut() {
                if sealed_cells.contains(&cell.id) || c.cells.contains_key(&cell.id) {
                    cell.bastion = c.cells.get(&cell.id).and_then(|s| s.bastion);
                }
            }
            let nom = hk_crypto::public(&self.save.secret);
            if let Some(owned) = c.cosmetics.get(&nom) {
                self.save
                    .expansion
                    .campaign
                    .owned
                    .extend(owned.iter().cloned());
            }
            let head = self
                .save
                .expansion
                .authority
                .seals
                .last()
                .map(|s| s.body.edict_head);
            let count = head
                .and_then(|h| {
                    self.save
                        .expansion
                        .authority
                        .edicts
                        .iter()
                        .position(|e| e.hash() == h)
                })
                .map(|i| i + 1)
                .unwrap_or(0);
            for edict in &self.save.expansion.authority.edicts[..count] {
                match &edict.body.kind {
                    EdictKind::GrantCosmetic { nom: n, item } if *n == nom => {
                        self.save.expansion.campaign.owned.insert(item.clone());
                    }
                    EdictKind::RevokeCosmetic { nom: n, item } if *n == nom => {
                        self.save.expansion.campaign.owned.remove(item);
                    }
                    _ => {}
                }
                if Some(edict.hash()) == head {
                    break;
                }
            }
            if !self
                .save
                .expansion
                .campaign
                .owned
                .contains(&self.save.expansion.campaign.equipped)
            {
                self.save.expansion.campaign.equipped = "cape_veille".into();
            }
            for (cell, (kind, until)) in &c.world_events {
                if *until > self.now && *kind == 0 {
                    for local in self
                        .save
                        .world
                        .cells
                        .values_mut()
                        .filter(|c| marche(c.id) == marche(*cell))
                    {
                        local.clear = false;
                        local.continuous = 0;
                    }
                }
            }
            for (cell, (realm, kind)) in c.foundations {
                if kind == 0 {
                    if let Some(m) = self
                        .save
                        .expansion
                        .campaign
                        .memories
                        .iter_mut()
                        .find(|m| m.realm == realm)
                    {
                        m.sanctuary = cell;
                    }
                }
            }
            if c.season > self.save.expansion.campaign.season {
                self.save.expansion.campaign.season = c.season;
                self.save.expansion.campaign.seal_season(
                    &self
                        .save
                        .expansion
                        .authority
                        .state(&self.save.ledger)
                        .unwrap(),
                    self.save.expansion.authority.head(),
                );
            }
            self.dirty = true;
        }
    }
    pub fn crown_action(&mut self, kind: EdictKind) {
        let Some(secret) = self
            .save
            .expansion
            .throne
            .filter(|_| self.expansion.throne_open)
        else {
            self.toast("Le Trône doit être déverrouillé.");
            return;
        };
        match self
            .save
            .expansion
            .authority
            .propose(&secret, kind, self.now)
        {
            Ok(_) => {
                self.dirty = true;
                self.toast("Édit signé. Effet au prochain Sceau.");
            }
            Err(e) => self.toast(&e),
        }
    }
    pub fn throne_unlock(&mut self) {
        if !self.is_dev {
            return;
        }
        self.save.expansion.throne.get_or_insert(fixture().throne);
        self.expansion.throne_open = true;
        self.screen = 27;
        self.dirty = true;
        self.toast("Trône DEV ouvert après authentification.");
    }
    pub fn export_authority(&mut self) {
        self.last_proof=serde_json::json!({"format":"HEXKEEP/v2/dev","kind":"authority","authority":self.save.expansion.authority,"ledger":self.save.ledger}).to_string();
        self.native_action = 6;
    }
    pub fn import_exchange(&mut self, text: &str) -> Result<(), String> {
        if text.len() > 16 * 1024 * 1024 {
            return Err("Dossier trop volumineux".into());
        }
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if v["body"]["hp"].is_array() {
            let signed: hk_crown::Signed<hk_crown::Codex> =
                serde_json::from_value(v).map_err(|e| e.to_string())?;
            if !signed.valid(
                "dev",
                "codex",
                &self.save.expansion.authority.genesis.body.root,
            ) || !signed.body.valid()
            {
                return Err("Codex racine invalide".into());
            }
            self.crown_action(EdictKind::Codex { codex: signed.body });
            return Ok(());
        }
        match v["kind"].as_str() {
            Some("recovery-share") => {
                let share: hk_crypto::identity::RecoveryShare =
                    serde_json::from_value(v["share"].clone()).map_err(|e| e.to_string())?;
                share.open(&self.save.secret)?;
                self.save.expansion.held_shares.push(share);
                self.screen = 35;
            }
            Some("authority") => {
                let other: Authority =
                    serde_json::from_value(v["authority"].clone()).map_err(|e| e.to_string())?;
                let ledger: Ledger =
                    serde_json::from_value(v["ledger"].clone()).map_err(|e| e.to_string())?;
                if ledger.events.len() > 50000 || ledger.events.iter().any(|e| !e.valid("dev")) {
                    return Err("Registre invalide".into());
                }
                let mut combined = self.save.ledger.clone();
                combined.merge(&ledger);
                self.save.expansion.authority.merge(&other, &combined)?;
                self.save.ledger = combined;
                self.apply_canonical();
            }
            Some("recovery") => {
                let pieces: Vec<Vec<u8>> =
                    serde_json::from_value(v["shares"].clone()).map_err(|e| e.to_string())?;
                let secret = hk_crypto::recover(&pieces)?;
                self.save.secret = secret;
                self.session = hk_crypto::identity::Session::development(&secret, self.now);
                self.native_action = 10;
                self.dirty = true;
            }
            _ => {
                let replay = hk_net::Replay::verify(text)?;
                if self.save.expansion.case_files.len() >= 16 {
                    self.save.expansion.case_files.remove(0);
                }
                self.save.expansion.case_files.push(text.into());
                self.battle = Some(replay.battle.clone());
                self.expansion.court = Some(replay);
                self.screen = 28;
                self.expansion.court_speed = 2;
                self.expansion.court_play = false;
            }
        }
        self.dirty = true;
        Ok(())
    }
    pub fn expansion_battle_end(&mut self, b: &Battle) {
        self.save.expansion.campaign.xp = self
            .save
            .expansion
            .campaign
            .xp
            .saturating_add(b.fighters[0].kills as u32 * 30 + 10);
        let population: Vec<_> = b
            .fighters
            .iter()
            .filter(|f| f.bot)
            .map(|f| {
                (
                    f.genome,
                    f.kills as i64 * 100 + b.tick as i64 - f.deaths as i64 * 50,
                )
            })
            .collect();
        if !population.is_empty() {
            let seed = digest(&(
                self.save.expansion.authority.head(),
                marche(self.save.world.current),
                self.save.expansion.campaign.generation,
            ));
            self.save.expansion.campaign.genomes = hk_sim::evolve(&population, seed);
            self.save.expansion.campaign.generation += 1;
            self.event(Kind::Generation, self.save.expansion.campaign.generation);
        }
        if b.siege.as_ref().is_some_and(|s| s.captured) {
            if let Some(c) = self.save.world.cells.get_mut(&self.save.world.current) {
                c.bastion = Some(self.save.realm);
            }
            self.event(Kind::Capture, 100);
            self.save.expansion.campaign.xp += 100;
        }
        if self.battle_mode == 6 && b.fighters[0].kills >= 1 {
            self.expansion.memory_ready = true;
        }
        if self.battle_mode == 7 && b.fighters[0].kills >= 1 {
            self.save.expansion.campaign.nuit_lanterns += 1;
            self.save.expansion.campaign.xp += 50;
            self.event(Kind::Lantern, 1);
        }
        if b.fighters[0].deaths > 0 {
            let nom = hk_crypto::public(&self.save.secret);
            for m in &mut self.save.expansion.campaign.memories {
                if m.carrier == Some(nom) {
                    m.drop();
                }
            }
        }
    }
    pub fn memory_location(&mut self, cell: u64, speed: u32) {
        let nom = hk_crypto::public(&self.save.secret);
        let mut dropped = false;
        for m in &mut self.save.expansion.campaign.memories {
            if m.carrier == Some(nom) {
                dropped |= m.update(cell, speed, self.now);
            }
        }
        if dropped {
            self.toast("La Mémoire est tombée. Marche sous 14 km/h.");
        }
        self.dirty = true;
    }
    pub fn ble_observed(&mut self, bytes: Vec<u8>) {
        if bytes.len() < 16 {
            return;
        }
        let token: [u8; 16] = bytes[..16].try_into().unwrap();
        self.expansion.ble_seen.insert(token, self.now);
        let subjects: Vec<_> = self
            .expansion
            .peer_certificates
            .iter()
            .filter(|(_, c)| hk_crypto::identity::beacon(&c.session, self.now) == token)
            .map(|(n, _)| *n)
            .collect();
        for subject in subjects {
            let witness = hk_crypto::identity::Witness::new(
                &self.save.secret,
                subject,
                token,
                self.now,
                self.save.world.current,
            );
            self.save
                .expansion
                .witnesses
                .retain(|w| w.observer != witness.observer || w.subject != subject);
            self.save.expansion.witnesses.push(witness.clone());
            if let Some(n) = &mut self.node {
                n.send(hk_net::Message::Witness { witness });
            }
        }
    }
    pub fn recovery_prepare(&mut self, test: bool) -> Result<(), String> {
        let owner = self.save.secret;
        let nom = hk_crypto::public(&owner);
        let parts = hk_crypto::split(&owner);
        let recipients: Vec<Hash> = if test {
            self.save.expansion.test_guardians = (0..5).map(|_| hk_crypto::new_secret()).collect();
            self.save
                .expansion
                .test_guardians
                .iter()
                .map(hk_crypto::public)
                .collect()
        } else {
            self.peers
                .keys()
                .filter(|subject| {
                    self.save.expansion.witnesses.iter().any(|a| {
                        a.observer == nom
                            && a.subject == **subject
                            && self
                                .save
                                .expansion
                                .witnesses
                                .iter()
                                .any(|b| hk_crypto::identity::Witness::mutual(a, b, self.now))
                    })
                })
                .take(5)
                .copied()
                .collect()
        };
        if recipients.len() != 5 {
            return Err("Cinq compagnons co-présents sont nécessaires.".into());
        }
        let shares = recipients
            .iter()
            .zip(parts)
            .map(|(r, p)| hk_crypto::identity::RecoveryShare::new(&owner, *r, &p, self.now))
            .collect::<Result<Vec<_>, _>>()?;
        if test {
            self.save.expansion.test_shares = shares;
        } else if let Some(n) = &mut self.node {
            for share in shares {
                n.send(hk_net::Message::Recovery { share });
            }
        }
        self.dirty = true;
        Ok(())
    }
    pub fn recovery_test_restore(&mut self) -> Result<(), String> {
        let e = &self.save.expansion;
        if e.test_shares.len() < 3 || e.test_guardians.len() < 3 {
            return Err("Prépare d'abord le Serment DEV.".into());
        }
        let parts = e
            .test_shares
            .iter()
            .zip(&e.test_guardians)
            .take(3)
            .map(|(s, g)| s.open(g))
            .collect::<Result<Vec<_>, _>>()?;
        let secret = hk_crypto::recover(&parts)?;
        if hk_crypto::public(&secret) != e.test_shares[0].owner {
            return Err("Nom reconstruit incohérent".into());
        }
        self.save.secret = secret;
        self.session = hk_crypto::identity::Session::development(&secret, self.now);
        self.native_action = 10;
        self.dirty = true;
        Ok(())
    }
    pub fn expansion_tap(&mut self, x: i32, y: i32) {
        let w = self.width;
        if self.screen == 36 && y < 30 && x > w - 180 && x < w - 72 {
            if let Some(mut codex) = self.expansion.codex_draft.take() {
                codex.version = self
                    .save
                    .expansion
                    .authority
                    .edicts
                    .iter()
                    .filter_map(|e| {
                        if let EdictKind::Codex { codex } = &e.body.kind {
                            Some(codex.version)
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(codex.version)
                    + 1;
                self.crown_action(EdictKind::Codex { codex });
                self.screen = 32;
            }
            return;
        }

        if y < 30 && x > w - 70 {
            self.screen = if self.screen == 20 { 7 } else { 20 };
            return;
        }
        let left = x < w / 2;
        let row = ((y - 52) / 45).clamp(0, 3) as usize;
        if y < 45 {
            return;
        }
        match self.screen {
            20 => {
                let i = row * 2 + if left { 0 } else { 1 };
                self.expansion.page = 0;
                self.screen = [21, 22, 23, 24, 25, 26, 27, 30][i];
                if self.screen == 27 && !self.expansion.throne_open {
                    self.native_action = 8;
                    self.screen = 20;
                }
            }
            21 => {
                if row == 0 {
                    self.start_battle(5);
                } else if row == 1 {
                    self.start_network();
                    self.challenge_mode(2);
                } else if row == 2 {
                    self.native_action = 11;
                } else {
                    self.screen = 7;
                }
            }
            22 => {
                if row == 0 {
                    self.expansion.selected_memory = (self.expansion.selected_memory + 1) % 3;
                    self.expansion.memory_ready = false;
                } else if row == 1 {
                    let cell =
                        self.save.expansion.campaign.memories[self.expansion.selected_memory].cell;
                    self.save.world.enter(cell, self.now);
                    self.selected = cell;
                    self.start_battle(6);
                } else if row == 2 {
                    let i = self.expansion.selected_memory;
                    let ready = self.expansion.memory_ready;
                    let result = self.save.expansion.campaign.memories[i].take(
                        hk_crypto::public(&self.save.secret),
                        self.save.realm,
                        self.save.world.current,
                        true,
                        ready,
                        self.now,
                    );
                    match result {
                        Ok(()) => {
                            self.save.world.banner = true;
                            self.toast("Course consentie : cellule publique rés. 10.");
                            self.dirty = true;
                        }
                        Err(e) => self.toast(&e),
                    }
                } else {
                    let i = self.expansion.selected_memory;
                    let own = self.save.realm;
                    let nom = hk_crypto::public(&self.save.secret);
                    if left {
                        let cell = self
                            .save
                            .world
                            .cells
                            .values()
                            .find(|c| c.bastion == Some(own))
                            .map(|c| c.id);
                        if let Some(cell) = cell {
                            self.save.world.enter(cell, self.now);
                            self.memory_location(cell, 1200);
                            if self.save.expansion.campaign.memories[i].deliver(
                                nom,
                                own,
                                cell,
                                Some(own),
                            ) {
                                self.save.expansion.campaign.xp += 150;
                                self.event(Kind::Memory, 100);
                                self.toast("Mémoire livrée. Garde collective +10 %.");
                            }
                        } else {
                            self.toast("Fonde d'abord un bastion de ton royaume.");
                        }
                    } else {
                        self.memory_location(self.save.world.current, 10000);
                    }
                }
            }
            23 => {
                if row == 0 {
                    let n = self.save.expansion.campaign.claim();
                    self.dirty = true;
                    self.toast(&format!("{n} récompenses gratuites réclamées."));
                } else if row == 1 {
                    self.start_battle(7);
                } else if row == 2 {
                    self.screen = 29;
                } else {
                    self.expansion.page = (self.expansion.page + 1) % 5;
                }
            }
            24 => {
                if row == 0 {
                    self.expansion.item = (self.expansion.item + 1) % hk_season::catalog().len();
                } else if row == 1 {
                    match self.save.expansion.campaign.buy_dev(
                        &self.save.secret,
                        self.expansion.item,
                        self.now,
                    ) {
                        Ok(()) => {
                            self.dirty = true;
                            self.toast("Achat TEST reçu. Aucun paiement effectué.");
                        }
                        Err(e) => self.toast(&e),
                    }
                } else if row == 2 {
                    let item = hk_season::catalog()[self.expansion.item].id;
                    if self.save.expansion.campaign.owned.contains(item) {
                        self.save.expansion.campaign.equipped = item.into();
                        self.dirty = true;
                        self.toast("Apparence équipée.");
                    } else {
                        self.toast("Ajoute cet objet à ta collection de test.");
                    }
                } else {
                    self.expansion.native_text =
                        hk_season::catalog()[self.expansion.item].id.into();
                    self.native_action = 12;
                }
            }
            25 => {
                if x < 212 && y >= 40 && y < 200 {
                    let col = ((x - 20) / 12).clamp(0, 15);
                    let row = ((y - 40) / 10).clamp(0, 15);
                    self.save.expansion.campaign.draft[(row * 16 + col) as usize] =
                        self.expansion.brush % 4;
                    self.dirty = true;
                } else if y < 82 {
                    self.expansion.brush = (self.expansion.brush + 1) % 4;
                } else if y < 127 {
                    let emblem = self.save.expansion.campaign.draft.clone();
                    let id = digest(&(self.save.secret, self.now, &emblem));
                    let mut members =
                        vec![Oath::new(&self.save.secret, self.save.realm, id, self.now)];
                    for _ in 0..2 {
                        members.push(Oath::new(
                            &hk_crypto::new_secret(),
                            self.save.realm,
                            id,
                            self.now,
                        ));
                    }
                    match House::found(
                        "Trois Lanternes".into(),
                        self.save.realm,
                        emblem,
                        members,
                        &[],
                        self.now,
                        true,
                    ) {
                        Ok(h) => {
                            self.save.expansion.campaign.houses.push(h);
                            self.dirty = true;
                            self.toast("Maison DEV fondée avec 2 compagnons de test.");
                        }
                        Err(e) => self.toast(&e),
                    }
                } else if y < 173 {
                    self.last_proof=serde_json::json!({"kind":"house","houses":self.save.expansion.campaign.houses}).to_string();
                    self.native_action = 6;
                } else {
                    self.screen = 35;
                }
            }
            26 => {
                if row == 0 {
                    self.start_network();
                    self.native_action = 11;
                } else if row == 1 {
                    self.save.expansion.phare = !self.save.expansion.phare;
                    if self.save.expansion.phare {
                        let banner = self.save.world.banner;
                        self.start_network();
                        self.save.world.banner = banner;
                        self.save.world.expand_watch(5, self.now);
                    }
                    self.save.expansion.phare_anchor = self.save.world.current;
                    self.native_action = 13;
                    self.dirty = true;
                } else if row == 2 {
                    self.screen = 16;
                } else {
                    self.native_action = 9;
                }
            }
            27 => {
                if !self.expansion.throne_open {
                    return;
                }
                if row == 0 && left {
                    if let Some(secret) = self.save.expansion.throne {
                        match self.save.expansion.authority.seal(
                            &secret,
                            &self.save.ledger,
                            self.now,
                        ) {
                            Ok(_) => {
                                self.apply_canonical();
                                self.toast("Sceau accepté. Histoire finalisée.");
                            }
                            Err(e) => self.toast(&e),
                        }
                    }
                } else if row == 0 {
                    self.native_action = 9;
                } else if row == 1 && left {
                    self.screen = 31;
                } else if row == 1 {
                    self.export_authority();
                } else if row == 2 && left {
                    self.screen = 28;
                    if let Some(text) = self.save.expansion.case_files.last().cloned() {
                        let _ = self.import_exchange(&text);
                    }
                } else if row == 2 {
                    self.screen = 32;
                } else if left {
                    self.screen = 29;
                } else {
                    self.screen = 33;
                }
            }
            28 => {
                if x < w - 198 {
                    if y > 210 {
                        self.native_action = 9;
                    }
                    return;
                }
                let control = ((y - 40) / 30).clamp(0, 5);
                match control {
                    0 => self.expansion.court_play = !self.expansion.court_play,
                    1 => {
                        if let Some(r) = &mut self.expansion.court {
                            r.step();
                            self.battle = Some(r.battle.clone());
                        }
                    }
                    2 => self.expansion.court_speed = (self.expansion.court_speed + 1) % 5,
                    3 => {
                        self.expansion.court_pov = (self.expansion.court_pov + 1)
                            % self
                                .expansion
                                .court
                                .as_ref()
                                .map(|r| r.players)
                                .unwrap_or(1)
                    }
                    _ => {
                        if let Some(r) = &self.expansion.court {
                            let proof = r.hash;
                            self.crown_action(EdictKind::Verdict {
                                proof,
                                accepted: control == 4,
                                reason: if control == 4 {
                                    "Rejeu conforme"
                                } else {
                                    "Preuve contestée"
                                }
                                .into(),
                            });
                        }
                    }
                }
            }
            29 => {
                if row == 3 {
                    self.screen = 23;
                }
            }
            30 => {
                if row == 0 {
                    self.native_action = 10;
                } else if row == 1 {
                    self.screen = 13;
                    self.native_action = 2;
                } else if row == 2 {
                    self.native_action = 9;
                } else {
                    self.last_proof=serde_json::json!({"kind":"identity","certificate":self.session.certificate}).to_string();
                    self.native_action = 6;
                }
            }
            31 => {
                let nom = self
                    .peers
                    .keys()
                    .next()
                    .copied()
                    .unwrap_or_else(|| hk_crypto::public(&self.save.secret));
                if row == 0 {
                    if left {
                        self.crown_action(EdictKind::Ban {
                            nom,
                            scope: Scope::Pvp,
                            until: self.now + 86400,
                        });
                    } else {
                        self.crown_action(EdictKind::Unban { nom });
                    }
                } else if row == 1 {
                    if left {
                        self.crown_action(EdictKind::AssignBastion {
                            cell: self.save.world.current,
                            realm: self.save.realm,
                        });
                    } else if let Some(s) = self.save.expansion.authority.seals.first() {
                        let seal = s.hash();
                        self.crown_action(EdictKind::Rollback {
                            cells: vec![self.save.world.current],
                            seal,
                        });
                    }
                } else if row == 2 {
                    let item = "skin_pierre".into();
                    if left {
                        self.crown_action(EdictKind::GrantCosmetic { nom, item });
                    } else {
                        self.crown_action(EdictKind::RevokeCosmetic { nom, item });
                    }
                } else {
                    self.crown_action(EdictKind::Proclamation {
                        text: "Le monde est à qui le tient. Veillez.".into(),
                    });
                }
            }
            32 => {
                if row == 0 {
                    self.expansion.codex_draft = Some(
                        self.save
                            .expansion
                            .authority
                            .state(&self.save.ledger)
                            .map(|s| s.codex)
                            .unwrap_or_default(),
                    );
                    self.screen = 36;
                } else if row == 1 {
                    self.crown_action(EdictKind::Foundation {
                        cell: self.save.world.current,
                        realm: self.save.realm,
                        kind: 0,
                    });
                } else if row == 2 {
                    self.crown_action(EdictKind::WorldEvent {
                        cell: self.save.world.current,
                        kind: if left { 0 } else { 1 },
                        until: self.now + 600,
                    });
                } else {
                    self.crown_action(EdictKind::SeasonKey {
                        season: self.save.expansion.campaign.season + 1,
                        key: hk_crypto::new_secret(),
                    });
                }
            }
            36 => {
                let index = self.expansion.codex_realm;
                let delta = if left { -1 } else { 1 };
                if row == 0 {
                    self.expansion.codex_realm = (index + 1) % 3;
                } else if let Some(c) = &mut self.expansion.codex_draft {
                    match row {
                        1 => c.hp[index] = (c.hp[index] + delta * 5).clamp(50, 200),
                        2 => c.projectile[index] = (c.projectile[index] + delta).clamp(40, 180),
                        _ => {
                            c.dash[index] = (c.dash[index] as i32 + delta * 3).clamp(30, 180) as u16
                        }
                    }
                }
            }
            35 => {
                let result = match row {
                    0 => self.recovery_prepare(false),
                    1 => self.recovery_prepare(true),
                    2 => self.recovery_test_restore(),
                    _ => {
                        self.native_action = 9;
                        Ok(())
                    }
                };
                match result {
                    Ok(()) => self.toast("Serment vérifié. Les parts restent chiffrées."),
                    Err(e) => self.toast(&e),
                }
            }
            33 => {
                if row == 3 {
                    self.screen = 27;
                }
            }
            _ => {}
        }
    }
    pub fn draw_expansion(&self, c: &mut Canvas) {
        let w = c.w;
        let e = &self.save.expansion;
        let p = &e.campaign;
        c.rect(0, 0, w, 30, PANEL);
        let title = match self.screen {
            20 => "LA VEILLE • V0.2",
            21 => "SIÈGE DU BASTION",
            22 => "COURSE DE MÉMOIRE",
            23 => "SAISON I • 40 PALIERS",
            24 => "BOUTIQUE • TEST DEV",
            25 => "MAISON • HÉRALDIQUE",
            26 => "LANTERNE ET PHARE",
            27 => "LE TRÔNE • DEV",
            28 => "LA HAUTE COUR",
            29 => "LA CHRONIQUE",
            30 => "LE NOM VRAI",
            35 => "LE SERMENT DES CINQ",
            36 => "ÉQUILIBRER LE CODEX",
            31 => "ÉDITS DE LA COURONNE",
            32 => "CODEX ET MONDE",
            _ => "OBSERVATOIRE",
        };
        c.text(12, 10, title, GOLD, 1);
        c.text(w - 58, 10, "RETOUR", MUTED, 1);
        let button = |c: &mut Canvas, row: usize, left: bool, text: &str| {
            let x = if left { 16 } else { w / 2 + 8 };
            c.button(x, 52 + row as i32 * 45, w / 2 - 24, text, true);
        };
        let line = |c: &mut Canvas, row: usize, text: &str| {
            c.text(18, 36 + row as i32 * 45, text, MUTED, 1)
        };
        match self.screen {
            20 => {
                for (i, label) in [
                    "SIÈGES",
                    "MÉMOIRES",
                    "SAISON ET NUIT LONGUE",
                    "BOUTIQUE",
                    "MAISONS",
                    "VEILLE ET PHARE",
                    "TRÔNE DEV",
                    "IDENTITÉ",
                ]
                .iter()
                .enumerate()
                {
                    button(c, i / 2, i % 2 == 0, label);
                }
            }
            21 => {
                line(c, 0, "Brise la Porte. Tiens la Cour sans ennemi : 60 s.");
                button(c, 0, true, "SIÈGE SOLO");
                button(c, 0, false, "PORTE = 500 + 20 X MURS");
                button(c, 1, true, "SIÈGE AVEC LES PAIRS");
                button(c, 2, true, "ACTIVER LE BLUETOOTH");
                line(c, 3, "Les Mémoires renforcent la garde, jamais tes tirs.");
                button(c, 3, true, "CARTE");
            }
            22 => {
                let m = &p.memories[self.expansion.selected_memory];
                line(
                    c,
                    0,
                    &format!(
                        "{} • {}",
                        m.realm.name(),
                        if m.carrier.is_some() {
                            "EN COURSE"
                        } else if m.held_by.is_some() {
                            "DÉTENUE"
                        } else {
                            "SANCTUAIRE"
                        }
                    ),
                );
                button(c, 0, true, "CHOISIR LA MÉMOIRE >");
                button(c, 1, true, "DEV : ALLER AUX GARDIENS");
                line(
                    c,
                    2,
                    "Saisir = accepter la diffusion de ta cellule rés. 10.",
                );
                button(c, 2, true, "J'ACCEPTE ET JE SAISIS");
                line(c, 3, "DEV : trajet à pied / essai à 36 km/h.");
                button(c, 3, true, "MARCHER ET LIVRER");
                button(c, 3, false, "TESTER LA CHUTE");
            }
            23 => {
                line(
                    c,
                    0,
                    &format!(
                        "Semaine {}/10 • {} XP • palier {}/40",
                        p.week,
                        p.xp,
                        p.tier()
                    ),
                );
                button(c, 0, true, "RÉCLAMER LES COSMÉTIQUES");
                button(c, 1, true, "NUIТ LONGUE : COMBATTRE");
                button(c, 2, true, "LIRE LA CHRONIQUE");
                let start = self.expansion.page * 8 + 1;
                line(
                    c,
                    3,
                    &format!(
                        "Paliers {}–{} : cape, motif, trace, lumière",
                        start,
                        start + 7
                    ),
                );
                button(c, 3, true, "PARCOURIR LA PISTE >");
            }
            24 => {
                let products = hk_season::catalog();
                let item = &products[self.expansion.item];
                line(
                    c,
                    0,
                    &format!(
                        "{} • {}.{:02} EUR",
                        item.name,
                        item.cents / 100,
                        item.cents % 100
                    ),
                );
                c.panel(w / 2 + 16, 62, w / 2 - 36, 96);
                c.sprite(
                    w * 3 / 4 - 32,
                    76,
                    if item.id.starts_with("lantern") { 2 } else { 0 },
                    self.save.realm.index(),
                    4,
                    self.ticks as u32,
                );
                let hue = [GOLD, BLUE, GREEN, WHITE]
                    [(blake3::hash(item.id.as_bytes()).as_bytes()[0] % 4) as usize];
                c.rect(w * 3 / 4 - 14, 116, 28, 22, hue);
                c.text(w / 2 + 26, 144, "APERÇU COSMÉTIQUE", MUTED, 1);
                button(c, 0, true, "OBJET SUIVANT >");
                button(c, 1, true, "OBTENIR EN TEST • 0 EUR");
                line(
                    c,
                    2,
                    if p.owned.contains(item.id) {
                        "Dans ta collection. Apparence uniquement."
                    } else {
                        "Aucun avantage de combat. Aucun tirage au sort."
                    },
                );
                button(c, 2, true, "ÉQUIPER");
                button(c, 3, true, "VÉRIFIER GOOGLE PLAY");
                line(c, 3, "Les achats réels sont désactivés dans cette DEV.");
            }
            25 => {
                for y in 0..16 {
                    for x in 0..16 {
                        let v = p.draft[(y * 16 + x) as usize] as usize;
                        let color = [INK, GOLD, REALMS[self.save.realm.index()], WHITE][v.min(3)];
                        c.rect(20 + x * 12, 40 + y * 10, 11, 9, color);
                    }
                }
                c.button(230, 52, w - 250, "COULEUR >", true);
                c.button(230, 97, w - 250, "FONDER • 2 ALLIÉS DEV", true);
                c.button(230, 142, w - 250, "EXPORTER LA MAISON", false);
                c.button(230, 187, w - 250, "SERMENT DES CINQ", false);
                c.text(
                    22,
                    209,
                    &format!("{} maison(s) • dessin 16x16", p.houses.len()),
                    MUTED,
                    1,
                );
            }
            26 => {
                line(c, 0, &self.expansion.bluetooth_status);
                button(c, 0, true, "BALISES BLUETOOTH");
                line(
                    c,
                    1,
                    if e.phare {
                        "Phare actif • ancre fixe • secteur + Wi-Fi"
                    } else {
                        "Phare arrêté"
                    },
                );
                button(c, 1, true, "ACTIVER / ARRÊTER PHARE");
                button(c, 2, true, "PAIRS ET RELAIS");
                button(c, 3, true, "IMPORTER UN DOSSIER");
            }
            27 => {
                line(
                    c,
                    0,
                    &format!(
                        "{} Sceaux • {} Édits • automatique : 15 min",
                        e.authority.seals.len(),
                        e.authority.edicts.len()
                    ),
                );
                for (i, label) in [
                    "SCELLER MAINTENANT",
                    "IMPORTER DOSSIER",
                    "ÉDITS",
                    "EXPORTER HISTOIRE",
                    "HAUTE COUR",
                    "CODEX ET MONDE",
                    "CHRONIQUE",
                    "OBSERVATOIRE",
                ]
                .iter()
                .enumerate()
                {
                    button(c, i / 2, i % 2 == 0, label);
                }
            }
            28 => {
                if let Some(r) = &self.expansion.court {
                    let mut battle = Canvas::new(w, 240);
                    self.draw_battle(&mut battle);
                    let outw = (w - 224).min(360);
                    let outh = outw * 192 / 360;
                    let ox = (w - 360) / 2;
                    for y in 0..outh {
                        for x in 0..outw {
                            let source =
                                (((27 + y * 192 / outh) * w + ox + x * 360 / outw) * 4) as usize;
                            let dst = (((40 + y) * w + 12 + x) * 4) as usize;
                            c.px[dst..dst + 4].copy_from_slice(&battle.px[source..source + 4]);
                        }
                    }
                    c.text(
                        14,
                        201,
                        &format!(
                            "{}/{} TICKS • VUE {}",
                            r.battle.tick,
                            r.final_tick,
                            self.expansion.court_pov + 1
                        ),
                        WHITE,
                        1,
                    );
                    let f = &r.battle.fighters[self.expansion.court_pov % r.players];
                    c.text(
                        14,
                        215,
                        &format!("PV {} • {} K / {} D • IMPORTER >", f.hp, f.kills, f.deaths),
                        MUTED,
                        1,
                    );
                } else {
                    c.text(14, 75, "Dépose une preuve co-signée.", WHITE, 1);
                    c.button(14, 211, w - 222, "IMPORTER UN DOSSIER", true);
                }
                for (i, label) in [
                    if self.expansion.court_play {
                        "PAUSE"
                    } else {
                        "LIRE"
                    },
                    "PAS À PAS",
                    [
                        "VITESSE X0.25",
                        "VITESSE X0.5",
                        "VITESSE X1",
                        "VITESSE X2",
                        "VITESSE X4",
                    ][self.expansion.court_speed],
                    "POINT DE VUE >",
                    "VERDICT : CONFORME",
                    "VERDICT : CONTESTÉ",
                ]
                .iter()
                .enumerate()
                {
                    c.button(w - 198, 40 + i as i32 * 30, 182, label, i == 0);
                }
            }
            29 => {
                if p.chronicle.is_empty() {
                    c.text(18, 55, "L'histoire attend son Sceau de Saison.", WHITE, 1);
                    c.text(
                        18,
                        80,
                        "Le Trône clôt la saison par Édit puis Sceau.",
                        MUTED,
                        1,
                    );
                } else {
                    for (i, l) in p.chronicle.lines().enumerate() {
                        c.text(18, 50 + i as i32 * 22, l, WHITE, 1);
                    }
                }
                button(c, 3, true, "RETOUR SAISON");
            }
            30 => {
                line(c, 0, &self.expansion.device_status);
                button(c, 0, true, "CERTIFIER CET APPAREIL");
                line(
                    c,
                    1,
                    &format!(
                        "Session : {} • valable 24 h",
                        hex(&self.session.certificate.session[..4])
                    ),
                );
                button(c, 1, true, "REVOIR MES 24 MOTS");
                button(c, 2, true, "RESTAURER / IMPORTER");
                button(c, 3, true, "EXPORTER CERTIFICAT PUBLIC");
            }
            31 => {
                line(c, 0, "Cible : premier pair connu ; sinon ton Nom DEV.");
                for (i, label) in [
                    "BANNIR PVP 24 H",
                    "GRACIER",
                    "ATTRIBUER BASTION",
                    "ROLLBACK CELLULE",
                    "OFFRIR APPARENCE",
                    "RÉVOQUER APPARENCE",
                    "PROCLAMER",
                    "PROCLAMER",
                ]
                .iter()
                .enumerate()
                {
                    button(c, i / 2, i % 2 == 0, label);
                }
            }
            32 => {
                button(c, 0, true, "PUBLIER CODEX SUIVANT");
                button(c, 1, true, "FONDER SANCTUAIRE ICI");
                button(c, 2, true, "TEMPÊTE 10 MIN");
                button(c, 2, false, "ÉVEILLER SANS-NOM");
                button(c, 3, true, "CLÔTURER SAISON");
                line(c, 3, "Chaque décision attend le prochain Sceau.");
            }
            36 => {
                c.text(w - 176, 10, "PUBLIER >", GOLD, 1);
                if let Some(codex) = &self.expansion.codex_draft {
                    let i = self.expansion.codex_realm;
                    line(c, 0, Realm::from_index(i).name());
                    button(c, 0, true, "ROYAUME SUIVANT >");
                    line(c, 1, &format!("PV MAX : {}", codex.hp[i]));
                    button(c, 1, true, "-5 PV");
                    button(c, 1, false, "+5 PV");
                    line(
                        c,
                        2,
                        &format!("VITESSE PROJECTILE : {}", codex.projectile[i]),
                    );
                    button(c, 2, true, "-1");
                    button(c, 2, false, "+1");
                    line(c, 3, &format!("RECHARGE DASH : {} TICKS", codex.dash[i]));
                    button(c, 3, true, "-3 TICKS");
                    button(c, 3, false, "+3 TICKS");
                }
            }
            35 => {
                line(
                    c,
                    0,
                    &format!("{} parts confiées à ton Nom", e.held_shares.len()),
                );
                button(c, 0, true, "CONFIER AUX 5 PAIRS BLE");
                line(c, 1, "Essai DEV : cinq identités locales de test.");
                button(c, 1, true, "PRÉPARER LE SERMENT DEV");
                button(c, 2, true, "RESTAURER AVEC TROIS PARTS");
                button(c, 3, true, "IMPORTER UNE PART CHIFFRÉE");
            }
            _ => {
                let canonical = e.authority.state(&self.save.ledger).ok();
                for (i, t) in [
                    format!("Vue limitée aux données synchronisées."),
                    format!(
                        "{} cellules connues / {} pairs",
                        self.save.world.cells.len(),
                        self.peers.len()
                    ),
                    format!(
                        "{} événements / {} scellés",
                        self.save.ledger.events.len(),
                        canonical
                            .as_ref()
                            .map(|s| s.cells.values().map(|c| c.events.len()).sum::<usize>())
                            .unwrap_or(0)
                    ),
                    format!("Génération des Sans-Témoin : {}", p.generation),
                    format!("{} témoignages BLE", e.witnesses.len()),
                ]
                .iter()
                .enumerate()
                {
                    c.text(18, 48 + i as i32 * 27, t, WHITE, 1);
                }
                button(c, 3, true, "RETOUR AU TRÔNE");
            }
        }
    }
}
