use borsh::{BorshDeserialize, BorshSerialize};
use hk_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Kind {
    Watch,
    Foundation,
    Victory,
    Banner,
    Generation,
    Capture,
    Memory,
    Lantern,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Body {
    pub version: u8,
    pub network: String,
    pub cell: u64,
    pub parents: Vec<Hash>,
    pub seal: Hash,
    pub lamport: u64,
    pub kind: Kind,
    pub realm: Realm,
    pub value: u32,
    pub time: u64,
    pub author: [u8; 32],
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Event {
    pub body: Body,
    pub signature: Vec<u8>,
}
impl Event {
    pub fn id(&self) -> Hash {
        digest(self)
    }
    pub fn valid(&self, network: &str) -> bool {
        self.body.version == 1
            && self.body.network == network
            && self.body.parents.len() <= 8
            && self.signature.len() == 64
            && self.body.parents.windows(2).all(|p| p[0] < p[1])
            && hk_crypto::verify(
                &self.body.author,
                network,
                "event",
                &borsh::to_vec(&self.body).unwrap(),
                &self.signature,
            )
    }
}
#[derive(Clone, Default, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Ledger {
    pub events: Vec<Event>,
}
#[derive(Clone, Default, Debug, PartialEq, Eq, BorshSerialize)]
pub struct Fold {
    pub bastions: BTreeMap<u64, Realm>,
    pub points: BTreeMap<[u8; 32], u32>,
    pub rejected: Vec<Hash>,
    pub accepted: BTreeSet<Hash>,
}
impl Ledger {
    pub fn append(
        &mut self,
        secret: &[u8; 32],
        cell: u64,
        kind: Kind,
        realm: Realm,
        value: u32,
        time: u64,
    ) -> Hash {
        let known = self.fold("dev", &BTreeSet::new());
        let used: BTreeSet<Hash> = self
            .events
            .iter()
            .flat_map(|e| e.body.parents.iter().copied())
            .collect();
        let mut heads: Vec<Hash> = self
            .events
            .iter()
            .filter(|e| {
                e.body.cell == cell && known.accepted.contains(&e.id()) && !used.contains(&e.id())
            })
            .map(Event::id)
            .collect();
        heads.sort();
        heads.truncate(8);
        let lamport = self
            .events
            .iter()
            .map(|e| e.body.lamport)
            .max()
            .unwrap_or(0)
            + 1;
        let body = Body {
            version: 1,
            network: "dev".into(),
            cell,
            parents: heads,
            seal: [0; 32],
            lamport,
            kind,
            realm,
            value,
            time,
            author: hk_crypto::public(secret),
        };
        let signature = hk_crypto::sign(secret, "dev", "event", &borsh::to_vec(&body).unwrap());
        let event = Event { body, signature };
        let id = event.id();
        self.events.push(event);
        id
    }
    pub fn merge(&mut self, other: &Ledger) {
        let mut ids: BTreeSet<Hash> = self.events.iter().map(Event::id).collect();
        for e in &other.events {
            if ids.insert(e.id()) {
                self.events.push(e.clone());
            }
        }
        self.events.sort_by_key(|e| (e.body.lamport, e.id()));
    }
    pub fn fold(&self, network: &str, banned: &BTreeSet<[u8; 32]>) -> Fold {
        let mut out = Fold::default();
        let mut sorted: Vec<&Event> = self.events.iter().collect();
        sorted.sort_by_key(|e| (e.body.lamport, e.id()));
        let mut seen = BTreeSet::new();
        for event in sorted {
            let h = event.id();
            if !seen.insert(h) {
                continue;
            }
            let b = &event.body;
            let parents_ok = b.parents.iter().all(|p| {
                out.accepted.contains(p)
                    && self.events.iter().any(|e| {
                        e.id() == *p && e.body.cell == b.cell && e.body.lamport < b.lamport
                    })
            });
            if !event.valid(network) || !parents_ok || banned.contains(&b.author) || b.lamport == 0
            {
                out.rejected.push(h);
                continue;
            }
            match b.kind {
                Kind::Foundation => {
                    out.bastions.entry(b.cell).or_insert(b.realm);
                }
                Kind::Capture => {
                    out.bastions.insert(b.cell, b.realm);
                }
                Kind::Victory | Kind::Memory | Kind::Lantern => {
                    let p = out.points.entry(b.author).or_default();
                    *p = p.saturating_add(b.value.min(100));
                }
                _ => {}
            }
            out.accepted.insert(h);
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merge_order_and_tamper() {
        let s = [7; 32];
        let mut a = Ledger::default();
        a.append(&s, 7, Kind::Watch, Realm::Aurelon, 1, 100);
        let mut b = a.clone();
        a.append(&s, 7, Kind::Foundation, Realm::Aurelon, 0, 101);
        b.append(&s, 7, Kind::Victory, Realm::Aurelon, 10, 102);
        let mut c = a.clone();
        a.merge(&b);
        b.merge(&c);
        assert_eq!(
            a.fold("dev", &BTreeSet::new()),
            b.fold("dev", &BTreeSet::new())
        );
        c.events[1].body.realm = Realm::Skarn;
        assert_eq!(c.fold("dev", &BTreeSet::new()).rejected.len(), 1);
        assert_eq!(a.fold("prod", &BTreeSet::new()).accepted.len(), 0);
    }
    #[test]
    fn missing_parent() {
        let mut a = Ledger::default();
        a.append(&[1; 32], 1, Kind::Watch, Realm::Aurelon, 1, 0);
        a.append(&[1; 32], 1, Kind::Watch, Realm::Aurelon, 1, 1);
        a.events.remove(0);
        assert!(a.fold("dev", &BTreeSet::new()).accepted.is_empty());
    }
}
