//! Collective campaign and cosmetics are separate from combat statistics.
use borsh::{BorshDeserialize, BorshSerialize};
use hk_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub realm: Realm,
    pub sanctuary: u64,
    pub cell: u64,
    pub carrier: Option<Hash>,
    pub held_by: Option<Realm>,
    pub last_update: u64,
    pub samples: VecDeque<(u64, u32)>,
}
impl Memory {
    pub fn take(
        &mut self,
        nom: Hash,
        realm: Realm,
        cell: u64,
        consent: bool,
        guards_defeated: bool,
        now: u64,
    ) -> Result<(), String> {
        if !consent
            || !guards_defeated
            || self.realm == realm
            || cell != self.cell
            || self.carrier.is_some()
        {
            return Err("Gardiens, présence et consentement requis".into());
        }
        self.carrier = Some(nom);
        self.held_by = None;
        self.last_update = now;
        self.samples.clear();
        Ok(())
    }
    pub fn update(&mut self, cell: u64, speed: u32, now: u64) -> bool {
        if self.carrier.is_none() {
            return false;
        }
        if now.saturating_sub(self.last_update) > 60 {
            self.drop();
            return true;
        }
        self.samples.push_back((now, speed));
        while self
            .samples
            .front()
            .is_some_and(|(t, _)| now.saturating_sub(*t) > 20)
        {
            self.samples.pop_front();
        }
        let average = self.samples.iter().map(|(_, s)| *s as u64).sum::<u64>()
            / self.samples.len().max(1) as u64;
        if average > 3888 {
            self.drop();
            return true;
        }
        self.cell = cell;
        self.last_update = now;
        false
    }
    pub fn timeout(&mut self, now: u64) -> bool {
        if self.carrier.is_some() && now.saturating_sub(self.last_update) > 60 {
            self.drop();
            true
        } else {
            false
        }
    }
    pub fn drop(&mut self) {
        self.carrier = None;
        self.samples.clear();
    }
    pub fn deliver(&mut self, nom: Hash, realm: Realm, cell: u64, bastion: Option<Realm>) -> bool {
        if self.carrier == Some(nom) && self.cell == cell && bastion == Some(realm) {
            self.carrier = None;
            self.held_by = Some(realm);
            true
        } else {
            false
        }
    }
    pub fn reset(&mut self) {
        self.cell = self.sanctuary;
        self.carrier = None;
        self.held_by = None;
        self.samples.clear();
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub memories: Vec<Memory>,
    pub season: u32,
    pub xp: u32,
    pub claimed: BTreeSet<u8>,
    pub owned: BTreeSet<String>,
    pub equipped: String,
    pub houses: Vec<House>,
    pub draft: Vec<u8>,
    pub color: u8,
    pub week: u8,
    pub nuit_lanterns: u32,
    pub nuit_complete: bool,
    pub chronicle: String,
    pub genomes: Vec<[u8; 16]>,
    pub generation: u32,
    pub dev_receipts: Vec<Receipt>,
    pub watch_seconds: u64,
}
impl Default for Campaign {
    fn default() -> Self {
        let cells = hk_world::neighbors(hk_world::pilot_cell(), 2);
        Self {
            memories: (0..3)
                .map(|i| Memory {
                    realm: Realm::from_index(i),
                    sanctuary: cells[i * 6],
                    cell: cells[i * 6],
                    carrier: None,
                    held_by: None,
                    last_update: 0,
                    samples: VecDeque::new(),
                })
                .collect(),
            season: 1,
            xp: 0,
            claimed: BTreeSet::new(),
            owned: BTreeSet::from(["cape_veille".into()]),
            equipped: "cape_veille".into(),
            houses: vec![],
            draft: vec![0; 256],
            color: 1,
            week: 1,
            nuit_lanterns: 0,
            nuit_complete: false,
            chronicle: String::new(),
            genomes: vec![[128; 16]; 8],
            generation: 0,
            dev_receipts: vec![],
            watch_seconds: 0,
        }
    }
}
impl Campaign {
    pub fn tier(&self) -> u8 {
        (self.xp / 100).min(40) as u8
    }
    pub fn claim(&mut self) -> usize {
        let mut n = 0;
        for tier in 1..=self.tier() {
            if self.claimed.insert(tier) {
                self.owned.insert(format!("s1_free_{tier:02}"));
                n += 1;
            }
            if self.owned.contains("season_pass") {
                self.owned.insert(format!("s1_pass_{tier:02}"));
            }
        }
        n
    }
    pub fn guard_bonus(&self, realm: Realm, base: u32) -> u32 {
        if self
            .memories
            .iter()
            .any(|m| m.realm != realm && m.held_by == Some(realm))
        {
            (base * 110 / 100).min(100)
        } else {
            base
        }
    }
    pub fn tick(&mut self, now: u64, start: u64) {
        self.watch_seconds += 1;
        if self.watch_seconds % 60 == 0 {
            self.xp = self.xp.saturating_add(1);
        }
        self.week = ((now.saturating_sub(start) / (7 * 86400)) + 1).min(10) as u8;
        for m in &mut self.memories {
            m.timeout(now);
        }
    }
    pub fn buy_dev(&mut self, secret: &Hash, index: usize, now: u64) -> Result<(), String> {
        let item = catalog().get(index).cloned().ok_or("Objet inconnu")?;
        if self.owned.contains(item.id) {
            return Err("Déjà dans ta collection".into());
        }
        let receipt = Receipt::new(secret, item.id, now);
        self.owned.insert(item.id.into());
        self.dev_receipts.push(receipt);
        Ok(())
    }
    pub fn seal_season(&mut self, canonical: &hk_crown::Canonical, head: Hash) {
        let bastions = canonical
            .cells
            .values()
            .filter(|c| c.bastion.is_some())
            .count();
        let events = canonical
            .cells
            .values()
            .map(|c| c.events.len())
            .sum::<usize>();
        self.chronicle=format!("LA PREMIÈRE NUIT\nSous le Sceau {}, {} actes demeurent.\n{} bastions ont tenu leurs lanternes.\n{} flammes ont encerclé le Sans-Nom.\nLes Mémoires regagnent leur pierre.\nCe que nous avons veillé porte encore nos noms.",hex(&head[..4]),events,bastions,self.nuit_lanterns);
        for m in &mut self.memories {
            m.reset();
        }
        self.nuit_complete = true;
    }
}
#[derive(Clone)]
pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub cents: u32,
}
pub fn catalog() -> Vec<Product> {
    vec![
        Product {
            id: "season_pass",
            name: "Pass : 40 paliers",
            cents: 499,
        },
        Product {
            id: "skin_pierre",
            name: "Veilleur de pierre",
            cents: 199,
        },
        Product {
            id: "skin_givre",
            name: "Veilleur de givre",
            cents: 399,
        },
        Product {
            id: "skin_seve",
            name: "Veilleur de sève",
            cents: 599,
        },
        Product {
            id: "lantern_ambre",
            name: "Lanterne d'ambre",
            cents: 299,
        },
        Product {
            id: "fanfare_aube",
            name: "Fanfare de l'aube",
            cents: 149,
        },
        Product {
            id: "signature_serment",
            name: "Signature du serment",
            cents: 99,
        },
        Product {
            id: "heraldry_extended",
            name: "Héraldique étendue",
            cents: 399,
        },
        Product {
            id: "patron_month",
            name: "Mécène / mois",
            cents: 299,
        },
    ]
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub nom: Hash,
    pub item: String,
    pub time: u64,
    pub signature: Vec<u8>,
    pub development: bool,
}
impl Receipt {
    pub fn new(secret: &Hash, item: &str, time: u64) -> Self {
        let mut r = Self {
            nom: hk_crypto::public(secret),
            item: item.into(),
            time,
            signature: vec![],
            development: true,
        };
        r.signature = hk_crypto::sign(
            secret,
            "dev",
            "test-receipt",
            &borsh::to_vec(&(r.nom, &r.item, time)).unwrap(),
        );
        r
    }
    pub fn valid(&self) -> bool {
        self.development
            && hk_crypto::verify(
                &self.nom,
                "dev",
                "test-receipt",
                &borsh::to_vec(&(self.nom, &self.item, self.time)).unwrap(),
                &self.signature,
            )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Oath {
    pub nom: Hash,
    pub realm: Realm,
    pub house: Hash,
    pub time: u64,
    pub signature: Vec<u8>,
}
impl Oath {
    pub fn new(secret: &Hash, realm: Realm, house: Hash, time: u64) -> Self {
        let nom = hk_crypto::public(secret);
        let signature = hk_crypto::sign(
            secret,
            "dev",
            "house-oath",
            &borsh::to_vec(&(nom, realm, house, time)).unwrap(),
        );
        Self {
            nom,
            realm,
            house,
            time,
            signature,
        }
    }
    pub fn valid(&self) -> bool {
        hk_crypto::verify(
            &self.nom,
            "dev",
            "house-oath",
            &borsh::to_vec(&(self.nom, self.realm, self.house, self.time)).unwrap(),
            &self.signature,
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct House {
    pub id: Hash,
    pub name: String,
    pub realm: Realm,
    pub emblem: Vec<u8>,
    pub members: Vec<Oath>,
    pub development_witnesses: bool,
}
impl House {
    pub fn found(
        name: String,
        realm: Realm,
        emblem: Vec<u8>,
        members: Vec<Oath>,
        witnesses: &[hk_crypto::identity::Witness],
        now: u64,
        dev: bool,
    ) -> Result<Self, String> {
        if emblem.len() != 256
            || emblem.iter().any(|v| *v > 3)
            || members.len() < 3
            || members.len() > 64
            || name.chars().count() > 18
        {
            return Err("Maison : format invalide".into());
        }
        let id = members[0].house;
        let mut noms = BTreeSet::new();
        if members.iter().any(|o| {
            o.house != id
                || o.realm != realm
                || !o.valid()
                || !noms.insert(o.nom)
                || now.abs_diff(o.time) > 120
        }) {
            return Err("Serments invalides".into());
        }
        if !dev {
            for i in 0..3 {
                for j in i + 1..3 {
                    if !witnesses.iter().any(|a| {
                        a.observer == members[i].nom
                            && a.subject == members[j].nom
                            && witnesses
                                .iter()
                                .any(|b| hk_crypto::identity::Witness::mutual(a, b, now))
                    }) {
                        return Err("Trois Veilleurs co-présents requis".into());
                    }
                }
            }
        }
        Ok(Self {
            id,
            name,
            realm,
            emblem,
            members,
            development_witnesses: dev,
        })
    }
}
#[derive(Clone, Debug)]
pub struct GuardDevice {
    pub nom: Hash,
    pub device: Hash,
    pub realm: Realm,
    pub confidence: u32,
    pub uptime: u32,
    pub proximity: u32,
}
pub fn guard_for(devices: &[GuardDevice], realm: Realm) -> u32 {
    let mut counts = BTreeMap::<Hash, u8>::new();
    let mut seen = BTreeSet::new();
    let mut sorted = devices.to_vec();
    sorted.sort_by_key(|d| (d.nom, d.device));
    let weights: Vec<_> = sorted
        .iter()
        .filter(|d| d.realm == realm && seen.insert(d.device))
        .filter_map(|d| {
            let n = counts.entry(d.nom).or_default();
            if *n >= 3 {
                return None;
            }
            *n += 1;
            Some(
                (d.confidence.min(65536) as u64 * d.uptime.min(65536) as u64 / 65536
                    * d.proximity.min(65536) as u64
                    / 65536) as u32,
            )
        })
        .collect();
    hk_world::guard(&weights)
}
pub fn confidence(attested: bool, banned: bool, witnesses: usize, age: u64, dev: bool) -> u32 {
    if banned || (!attested && !dev) {
        return 0;
    }
    (19661 + (witnesses.min(10) as u32 * 3277) + if age >= 14 * 86400 { 13107 } else { 0 })
        .min(65536)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn memory_drops_and_collective_bonus_only() {
        let mut c = Campaign::default();
        let m = &mut c.memories[1];
        m.take([1; 32], Realm::Aurelon, m.cell, true, true, 100)
            .unwrap();
        assert!(!m.update(m.cell, 1000, 101));
        assert!(m.update(m.cell, 9000, 102));
        assert!(m.carrier.is_none());
        m.take([1; 32], Realm::Aurelon, m.cell, true, true, 200)
            .unwrap();
        assert!(m.timeout(261));
        assert!(!m
            .take([1; 32], Realm::Aurelon, m.cell, false, true, 300)
            .is_ok());
        m.held_by = Some(Realm::Aurelon);
        assert_eq!(c.guard_bonus(Realm::Aurelon, 50), 55);
    }
    #[test]
    fn tiers_and_test_receipts() {
        let mut c = Campaign::default();
        c.xp = 4000;
        assert_eq!(c.claim(), 40);
        c.buy_dev(&[9; 32], 0, 100).unwrap();
        c.claim();
        assert_eq!(c.owned.len(), 82);
        assert!(c.dev_receipts[0].valid());
        c.dev_receipts[0].item = "power".into();
        assert!(!c.dev_receipts[0].valid());
    }
    #[test]
    fn sybil_three_devices_limit() {
        let d: Vec<_> = (0..10)
            .map(|i| GuardDevice {
                nom: [1; 32],
                device: [i; 32],
                realm: Realm::Skarn,
                confidence: 65536,
                uptime: 65536,
                proximity: 65536,
            })
            .collect();
        assert_eq!(
            guard_for(&d, Realm::Skarn),
            guard_for(&d[..3], Realm::Skarn)
        );
    }
    #[test]
    fn house_requires_distinct_signed_members() {
        let id = [9; 32];
        let o = Oath::new(&[1; 32], Realm::Aurelon, id, 100);
        assert!(House::found(
            "Maison".into(),
            Realm::Aurelon,
            vec![0; 256],
            vec![o.clone(); 3],
            &[],
            100,
            true
        )
        .is_err());
        let members = (1..=3)
            .map(|i| Oath::new(&[i; 32], Realm::Aurelon, id, 100))
            .collect::<Vec<_>>();
        assert!(House::found(
            "Maison".into(),
            Realm::Aurelon,
            vec![0; 256],
            members.clone(),
            &[],
            100,
            false
        )
        .is_err());
        assert!(House::found(
            "Maison".into(),
            Realm::Aurelon,
            vec![0; 256],
            members,
            &[],
            100,
            true
        )
        .is_ok());
    }
}
