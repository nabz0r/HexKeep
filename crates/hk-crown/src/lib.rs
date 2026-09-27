//! Offline Crown, chained edicts and independently recomputable sealed state.
use borsh::{BorshDeserialize, BorshSerialize};
use hk_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Codex {
    pub version: u32,
    pub hp: [i32; 3],
    pub dash: [u16; 3],
    pub projectile: [i32; 3],
    pub season_start: u64,
    pub season_days: u32,
}
impl Default for Codex {
    fn default() -> Self {
        Self {
            version: 1,
            hp: [100, 110, 90],
            dash: [90, 105, 72],
            projectile: [92, 65, 77],
            season_start: 0,
            season_days: 70,
        }
    }
}
impl Codex {
    pub fn valid(&self) -> bool {
        self.version > 0
            && self.hp.iter().all(|v| (50..=200).contains(v))
            && self.dash.iter().all(|v| (30..=180).contains(v))
            && self.projectile.iter().all(|v| (40..=180).contains(v))
            && self.season_days == 70
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Signed<T> {
    pub network: String,
    pub author: Hash,
    pub body: T,
    pub signature: Vec<u8>,
}
impl<T: BorshSerialize> Signed<T> {
    pub fn new(secret: &Hash, network: &str, kind: &str, body: T) -> Self {
        let mut s = Self {
            network: network.into(),
            author: hk_crypto::public(secret),
            body,
            signature: vec![],
        };
        s.signature = hk_crypto::sign(secret, network, kind, &borsh::to_vec(&s.body).unwrap());
        s
    }
    pub fn valid(&self, network: &str, kind: &str, key: &Hash) -> bool {
        self.network == network
            && self.author == *key
            && hk_crypto::verify(
                key,
                network,
                kind,
                &borsh::to_vec(&self.body).unwrap(),
                &self.signature,
            )
    }
    pub fn hash(&self) -> Hash {
        digest(self)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GenesisBody {
    pub network: String,
    pub root: Hash,
    pub codex: Codex,
    pub relays: Vec<String>,
    pub created: u64,
}
pub type Genesis = Signed<GenesisBody>;
pub fn genesis(secret: &Hash, network: &str, now: u64, codex: Codex) -> Genesis {
    Signed::new(
        secret,
        network,
        "genesis",
        GenesisBody {
            network: network.into(),
            root: hk_crypto::public(secret),
            codex,
            relays: vec![],
            created: now,
        },
    )
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Throne {
    pub public: Hash,
    pub issued: u64,
    pub expires: u64,
}
pub type Certificate = Signed<Throne>;
pub fn certify(root: &Hash, network: &str, key: Hash, now: u64) -> Certificate {
    Signed::new(
        root,
        network,
        "throne",
        Throne {
            public: key,
            issued: now,
            expires: now + 30 * 86400,
        },
    )
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum Scope {
    All,
    Pvp,
    Shop,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EdictKind {
    Ban {
        nom: Hash,
        scope: Scope,
        until: u64,
    },
    Unban {
        nom: Hash,
    },
    Rollback {
        cells: Vec<u64>,
        seal: Hash,
    },
    AssignBastion {
        cell: u64,
        realm: Realm,
    },
    GrantCosmetic {
        nom: Hash,
        item: String,
    },
    RevokeCosmetic {
        nom: Hash,
        item: String,
    },
    Verdict {
        proof: Hash,
        accepted: bool,
        reason: String,
    },
    Proclamation {
        text: String,
    },
    Foundation {
        cell: u64,
        realm: Realm,
        kind: u8,
    },
    WorldEvent {
        cell: u64,
        kind: u8,
        until: u64,
    },
    Codex {
        codex: Codex,
    },
    SeasonKey {
        season: u32,
        key: Hash,
    },
    RelayList {
        addresses: Vec<String>,
    },
    RevokeDevice {
        device: Hash,
    },
    RevokeThrone {
        throne: Hash,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EdictBody {
    pub sequence: u64,
    pub previous: Hash,
    pub time: u64,
    pub kind: EdictKind,
}
pub type Edict = Signed<EdictBody>;
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct CellState {
    pub events: BTreeSet<Hash>,
    pub bastion: Option<Realm>,
    pub victories: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SealBody {
    pub epoch: u64,
    pub previous: Hash,
    pub time: u64,
    pub cells_root: Hash,
    pub edict_head: Hash,
    pub codex_version: u32,
    pub throne_cert: Hash,
    pub event_ids: BTreeSet<Hash>,
}
pub type Seal = Signed<SealBody>;
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Canonical {
    pub cells: BTreeMap<u64, CellState>,
    pub bans: BTreeMap<Hash, (Scope, u64)>,
    pub devices: BTreeSet<Hash>,
    pub thrones: BTreeSet<Hash>,
    pub cosmetics: BTreeMap<Hash, BTreeSet<String>>,
    pub proclamations: Vec<String>,
    pub foundations: BTreeMap<u64, (Realm, u8)>,
    pub world_events: BTreeMap<u64, (u8, u64)>,
    pub codex: Codex,
    pub season: u32,
    pub season_key: Hash,
    pub relays: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authority {
    pub genesis: Genesis,
    pub certificates: Vec<Certificate>,
    pub edicts: Vec<Edict>,
    pub seals: Vec<Seal>,
}
impl Authority {
    pub fn new(genesis: Genesis) -> Result<Self, String> {
        if !genesis.valid(&genesis.body.network, "genesis", &genesis.body.root)
            || !genesis.body.codex.valid()
        {
            return Err("Genèse invalide".into());
        }
        Ok(Self {
            genesis,
            certificates: vec![],
            edicts: vec![],
            seals: vec![],
        })
    }
    pub fn network(&self) -> &str {
        &self.genesis.body.network
    }
    pub fn head(&self) -> Hash {
        self.seals
            .last()
            .map(Signed::hash)
            .unwrap_or(self.genesis.hash())
    }
    pub fn add_certificate(&mut self, c: Certificate) -> Result<(), String> {
        if !c.valid(self.network(), "throne", &self.genesis.body.root)
            || c.body.expires <= c.body.issued
            || c.body.expires - c.body.issued > 30 * 86400
        {
            return Err("Certificat invalide".into());
        }
        if !self.certificates.iter().any(|v| v.hash() == c.hash()) {
            self.certificates.push(c)
        }
        Ok(())
    }
    fn certified(&self, key: Hash, time: u64, revoked: &BTreeSet<Hash>) -> Option<&Certificate> {
        self.certificates.iter().find(|c| {
            c.body.public == key
                && c.body.issued <= time
                && time <= c.body.expires
                && !revoked.contains(&key)
                && c.valid(self.network(), "throne", &self.genesis.body.root)
        })
    }
    pub fn propose(&mut self, secret: &Hash, kind: EdictKind, now: u64) -> Result<Hash, String> {
        let e = Signed::new(
            secret,
            self.network(),
            "edict",
            EdictBody {
                sequence: self.edicts.len() as u64 + 1,
                previous: self
                    .edicts
                    .last()
                    .map(Signed::hash)
                    .unwrap_or(self.genesis.hash()),
                time: now,
                kind,
            },
        );
        self.append_edict(e)
    }
    pub fn append_edict(&mut self, e: Edict) -> Result<Hash, String> {
        let revoked = self.revoked_thrones();
        let root_revoke = matches!(e.body.kind, EdictKind::RevokeThrone { .. })
            && e.author == self.genesis.body.root;
        if e.body.sequence != self.edicts.len() as u64 + 1
            || e.body.previous
                != self
                    .edicts
                    .last()
                    .map(Signed::hash)
                    .unwrap_or(self.genesis.hash())
            || !e.valid(self.network(), "edict", &e.author)
            || (!root_revoke && self.certified(e.author, e.body.time, &revoked).is_none())
        {
            return Err("Édit non autorisé ou chaîne divergente".into());
        }
        if let EdictKind::Codex { ref codex } = e.body.kind {
            if !codex.valid() {
                return Err("Codex hors limites".into());
            }
        }
        let id = e.hash();
        self.edicts.push(e);
        Ok(id)
    }
    fn revoked_thrones(&self) -> BTreeSet<Hash> {
        let head = self.seals.last().map(|s| s.body.edict_head);
        let n = head
            .and_then(|h| self.edicts.iter().position(|e| e.hash() == h))
            .map(|i| i + 1)
            .unwrap_or(0);
        self.edicts[..n]
            .iter()
            .filter_map(|e| {
                if let EdictKind::RevokeThrone { throne } = e.body.kind {
                    Some(throne)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn banned(&self, nom: &Hash, scope: Scope, now: u64) -> bool {
        let n = self
            .seals
            .last()
            .and_then(|s| {
                self.edicts
                    .iter()
                    .position(|e| e.hash() == s.body.edict_head)
            })
            .map(|i| i + 1)
            .unwrap_or(0);
        let mut ban = None;
        for e in &self.edicts[..n] {
            match &e.body.kind {
                EdictKind::Ban {
                    nom: n,
                    scope,
                    until,
                } if n == nom => ban = Some((scope, *until)),
                EdictKind::Unban { nom: n } if n == nom => ban = None,
                _ => {}
            }
        }
        ban.is_some_and(|(s, u)| (u == 0 || now < u) && (*s == Scope::All || *s == scope))
    }
    fn compute(
        &self,
        ledger: &hk_ledger::Ledger,
        ids: &BTreeSet<Hash>,
        edict_count: usize,
        now: u64,
    ) -> Result<Canonical, String> {
        let mut c = Canonical {
            codex: self.genesis.body.codex.clone(),
            relays: self.genesis.body.relays.clone(),
            ..Default::default()
        };
        for e in &self.edicts[..edict_count] {
            match &e.body.kind {
                EdictKind::Ban { nom, scope, until } => {
                    c.bans.insert(*nom, (scope.clone(), *until));
                }
                EdictKind::Unban { nom } => {
                    c.bans.remove(nom);
                }
                EdictKind::RevokeDevice { device } => {
                    c.devices.insert(*device);
                }
                EdictKind::RevokeThrone { throne } => {
                    c.thrones.insert(*throne);
                }
                EdictKind::GrantCosmetic { nom, item } => {
                    c.cosmetics.entry(*nom).or_default().insert(item.clone());
                }
                EdictKind::RevokeCosmetic { nom, item } => {
                    c.cosmetics.entry(*nom).or_default().remove(item);
                }
                EdictKind::Proclamation { text } => c.proclamations.push(text.clone()),
                EdictKind::Foundation { cell, realm, kind } => {
                    c.foundations.insert(*cell, (*realm, *kind));
                }
                EdictKind::WorldEvent { cell, kind, until } => {
                    c.world_events.insert(*cell, (*kind, *until));
                }
                EdictKind::Codex { codex } => {
                    if codex.version <= c.codex.version {
                        return Err("Version Codex non croissante".into());
                    }
                    c.codex = codex.clone();
                }
                EdictKind::SeasonKey { season, key } => {
                    c.season = *season;
                    c.season_key = *key;
                }
                EdictKind::RelayList { addresses } => c.relays = addresses.clone(),
                _ => {}
            }
        }
        let banned = c
            .bans
            .iter()
            .filter(|(_, (s, u))| *s == Scope::All && (*u == 0 || now < *u))
            .map(|(n, _)| *n)
            .collect();
        let subset = hk_ledger::Ledger {
            events: ledger
                .events
                .iter()
                .filter(|e| ids.contains(&e.id()))
                .cloned()
                .collect(),
        };
        if subset
            .events
            .iter()
            .map(|e| e.id())
            .collect::<BTreeSet<_>>()
            != *ids
        {
            return Err("Registre incomplet pour ce Sceau".into());
        }
        let folded = subset.fold(self.network(), &banned);
        for e in &subset.events {
            if folded.accepted.contains(&e.id()) {
                let s = c.cells.entry(e.body.cell).or_default();
                s.events.insert(e.id());
                if e.body.kind == hk_ledger::Kind::Victory {
                    s.victories = s.victories.saturating_add(e.body.value.min(100));
                }
            }
        }
        for (id, r) in folded.bastions {
            c.cells.entry(id).or_default().bastion = Some(r)
        }
        for e in &self.edicts[..edict_count] {
            match &e.body.kind {
                EdictKind::AssignBastion { cell, realm } => {
                    c.cells.entry(*cell).or_default().bastion = Some(*realm)
                }
                EdictKind::Rollback { cells, seal } => {
                    let index = self
                        .seals
                        .iter()
                        .position(|s| s.hash() == *seal)
                        .ok_or("Sceau rollback inconnu")?;
                    let target = &self.seals[index];
                    let count = self
                        .edicts
                        .iter()
                        .position(|v| v.hash() == target.body.edict_head)
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    if count >= edict_count {
                        return Err("Rollback cyclique".into());
                    }
                    let old =
                        self.compute(ledger, &target.body.event_ids, count, target.body.time)?;
                    for id in cells {
                        if let Some(s) = old.cells.get(id) {
                            c.cells.insert(*id, s.clone());
                        } else {
                            c.cells.remove(id);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(c)
    }
    pub fn state(&self, ledger: &hk_ledger::Ledger) -> Result<Canonical, String> {
        match self.seals.last() {
            Some(s) => {
                let n = self
                    .edicts
                    .iter()
                    .position(|e| e.hash() == s.body.edict_head)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                self.compute(ledger, &s.body.event_ids, n, s.body.time)
            }
            None => Ok(Canonical {
                codex: self.genesis.body.codex.clone(),
                ..Default::default()
            }),
        }
    }
    pub fn seal(
        &mut self,
        secret: &Hash,
        ledger: &hk_ledger::Ledger,
        now: u64,
    ) -> Result<Hash, String> {
        let cert = self
            .certified(hk_crypto::public(secret), now, &self.revoked_thrones())
            .ok_or("Trône absent, expiré ou révoqué")?;
        let ids = ledger
            .events
            .iter()
            .filter(|e| e.valid(self.network()))
            .map(|e| e.id())
            .collect();
        let c = self.compute(ledger, &ids, self.edicts.len(), now)?;
        let s = Signed::new(
            secret,
            self.network(),
            "seal",
            SealBody {
                epoch: self.seals.len() as u64 + 1,
                previous: self.head(),
                time: now,
                cells_root: merkle(&c.cells),
                edict_head: self
                    .edicts
                    .last()
                    .map(Signed::hash)
                    .unwrap_or(self.genesis.hash()),
                codex_version: c.codex.version,
                throne_cert: cert.hash(),
                event_ids: ids,
            },
        );
        self.append_seal(s, ledger)
    }
    pub fn append_seal(&mut self, s: Seal, ledger: &hk_ledger::Ledger) -> Result<Hash, String> {
        let cert = self
            .certified(s.author, s.body.time, &self.revoked_thrones())
            .ok_or("Trône invalide")?;
        let n = if s.body.edict_head == self.genesis.hash() {
            0
        } else {
            self.edicts
                .iter()
                .position(|e| e.hash() == s.body.edict_head)
                .ok_or("Édits manquants")?
                + 1
        };
        let last_count = self
            .seals
            .last()
            .and_then(|s| {
                self.edicts
                    .iter()
                    .position(|e| e.hash() == s.body.edict_head)
            })
            .map(|i| i + 1)
            .unwrap_or(0);
        if s.body.epoch != self.seals.len() as u64 + 1
            || s.body.previous != self.head()
            || s.body.throne_cert != cert.hash()
            || s.body.time
                < self
                    .seals
                    .last()
                    .map(|s| s.body.time)
                    .unwrap_or(self.genesis.body.created)
            || n < last_count
            || self.edicts[..n].iter().any(|e| e.body.time > s.body.time)
            || !s.valid(self.network(), "seal", &s.author)
        {
            return Err("Chaîne de Sceaux invalide".into());
        }
        let c = self.compute(ledger, &s.body.event_ids, n, s.body.time)?;
        if merkle(&c.cells) != s.body.cells_root || c.codex.version != s.body.codex_version {
            return Err("Racine du monde divergente".into());
        }
        let id = s.hash();
        self.seals.push(s);
        Ok(id)
    }
    pub fn merge(&mut self, other: &Self, ledger: &hk_ledger::Ledger) -> Result<(), String> {
        if self.genesis.hash() != other.genesis.hash() {
            return Err("Couronne étrangère".into());
        }
        let mut next = self.clone();
        for c in &other.certificates {
            next.add_certificate(c.clone())?
        }
        for e in &other.edicts {
            if e.body.sequence == 0 {
                return Err("Séquence nulle".into());
            }
            if e.body.sequence > next.edicts.len() as u64 {
                next.append_edict(e.clone())?;
            } else if next.edicts[e.body.sequence as usize - 1].hash() != e.hash() {
                return Err("Fourche d'Édits".into());
            }
        }
        for s in &other.seals {
            if s.body.epoch == 0 {
                return Err("Époque nulle".into());
            }
            if s.body.epoch > next.seals.len() as u64 {
                next.append_seal(s.clone(), ledger)?;
            } else if next.seals[s.body.epoch as usize - 1].hash() != s.hash() {
                return Err("Fourche de Sceaux".into());
            }
        }
        *self = next;
        Ok(())
    }
    pub fn interregnum(&self, now: u64) -> bool {
        now.saturating_sub(
            self.seals
                .last()
                .map(|s| s.body.time)
                .unwrap_or(self.genesis.body.created),
        ) > 72 * 3600
    }
}
/// Sparse binary Merkle tree, 64-bit H3 index. Empty leaves and branch nodes domain-separated.
pub fn merkle(cells: &BTreeMap<u64, CellState>) -> Hash {
    let mut empty = *blake3::hash(b"HEXKEEP/v1/empty-cell").as_bytes();
    let mut level: BTreeMap<u64, Hash> = cells
        .iter()
        .map(|(k, v)| (*k, digest(&(0u8, *k, v))))
        .collect();
    for _ in 0..64 {
        let mut next = BTreeMap::new();
        for key in level.keys() {
            let parent = *key >> 1;
            next.entry(parent).or_insert_with(|| {
                digest(&(
                    1u8,
                    level.get(&(parent * 2)).unwrap_or(&empty),
                    level.get(&(parent * 2 + 1)).unwrap_or(&empty),
                ))
            });
        }
        empty = digest(&(1u8, empty, empty));
        level = next;
    }
    level.get(&0).copied().unwrap_or(empty)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Authority, Hash, hk_ledger::Ledger) {
        let root = [1; 32];
        let throne = [2; 32];
        let mut a = Authority::new(genesis(&root, "dev", 100, Codex::default())).unwrap();
        a.add_certificate(certify(&root, "dev", hk_crypto::public(&throne), 100))
            .unwrap();
        (a, throne, hk_ledger::Ledger::default())
    }
    #[test]
    fn ban_only_after_seal_and_rollback() {
        let (mut a, t, mut l) = setup();
        l.append(
            &[3; 32],
            9,
            hk_ledger::Kind::Foundation,
            Realm::Skarn,
            0,
            100,
        );
        let first = a.seal(&t, &l, 101).unwrap();
        a.propose(
            &t,
            EdictKind::Ban {
                nom: hk_crypto::public(&[3; 32]),
                scope: Scope::All,
                until: 0,
            },
            102,
        )
        .unwrap();
        assert!(!a.banned(&hk_crypto::public(&[3; 32]), Scope::All, 102));
        a.seal(&t, &l, 103).unwrap();
        assert!(a.banned(&hk_crypto::public(&[3; 32]), Scope::All, 103));
        assert!(a.state(&l).unwrap().cells.is_empty());
        a.propose(
            &t,
            EdictKind::Rollback {
                cells: vec![9],
                seal: first,
            },
            104,
        )
        .unwrap();
        a.seal(&t, &l, 105).unwrap();
        assert_eq!(a.state(&l).unwrap().cells[&9].bastion, Some(Realm::Skarn));
    }
    #[test]
    fn partition_merge_and_forgery() {
        let (mut a, t, mut l) = setup();
        let mut b = a.clone();
        l.append(&[4; 32], 8, hk_ledger::Kind::Watch, Realm::Vylde, 1, 100);
        a.seal(&t, &l, 101).unwrap();
        b.merge(&a, &l).unwrap();
        assert_eq!(a.head(), b.head());
        let mut forged = a.seals[0].clone();
        forged.body.cells_root[0] ^= 1;
        let (mut c, _, _) = setup();
        assert!(c.append_seal(forged, &l).is_err());
        assert!(c.merge(&a, &hk_ledger::Ledger::default()).is_err());
    }
    #[test]
    fn expiry_and_revoke() {
        let (mut a, t, l) = setup();
        assert!(a.seal(&t, &l, 31 * 86400).is_err());
        a.propose(
            &[1; 32],
            EdictKind::RevokeThrone {
                throne: hk_crypto::public(&t),
            },
            101,
        )
        .unwrap();
        a.seal(&t, &l, 102).unwrap();
        assert!(a.seal(&t, &l, 103).is_err());
    }
}
