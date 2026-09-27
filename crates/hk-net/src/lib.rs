mod replay;
pub use replay::{compact_proof, Replay};
pub fn unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
// Encrypted, non-authoritative DEV transport.
use borsh::{BorshDeserialize, BorshSerialize};
use futures::StreamExt;
use hk_proto::*;
use hk_sim::{Battle, Fighter};
use libp2p::{
    autonat, dcutr, gossipsub, identify, kad, mdns, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{mpsc, Mutex},
    time::Duration,
};
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Participant {
    pub key: [u8; 32],
    pub name: String,
    pub realm: Realm,
    pub role: Role,
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Message {
    Presence {
        player: Participant,
        cell: u64,
        epoch: u64,
    },
    Start {
        cell: u64,
        players: Vec<Participant>,
        nonce: u64,
    },
    Input {
        room: Hash,
        tick: u32,
        input: Input,
        previous: Hash,
    },
    State {
        room: Hash,
        tick: u32,
        hash: Hash,
    },
    Finish {
        room: Hash,
        tick: u32,
        hash: Hash,
    },
    Ledger {
        events: Vec<hk_ledger::Event>,
    },
    StartV2 {
        cell: u64,
        players: Vec<Participant>,
        nonce: u64,
        mode: u8,
        codex: hk_crown::Codex,
        walls: u32,
    },
    Recovery {
        share: hk_crypto::identity::RecoveryShare,
    },
    Witness {
        witness: hk_crypto::identity::Witness,
    },
    Authority {
        data: String,
    },
    Inventory {
        ids: Vec<Hash>,
    },
    Missing {
        ids: Vec<Hash>,
    },
    Memory {
        realm: Realm,
        cell10: u64,
        time: u64,
    },
    MissingInputs {
        room: Hash,
        player: Hash,
        from: u32,
        to: u32,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Packet {
    pub author: [u8; 32],
    pub sequence: u64,
    pub message: Message,
    pub signature: Vec<u8>,
    #[serde(default)]
    pub certificate: Option<hk_crypto::identity::Certificate>,
    #[serde(default)]
    pub issued: u64,
}
impl Packet {
    pub fn new(secret: &[u8; 32], sequence: u64, message: Message) -> Self {
        let mut p = Self {
            author: hk_crypto::public(secret),
            sequence,
            message,
            signature: vec![],
            certificate: None,
            issued: 0,
        };
        p.signature = hk_crypto::sign(secret, "dev", "net", &p.payload());
        p
    }
    pub fn session(
        session: &hk_crypto::identity::Session,
        sequence: u64,
        message: Message,
        issued: u64,
    ) -> Self {
        let mut p = Self {
            author: session.certificate.device.nom,
            sequence,
            message,
            signature: vec![],
            certificate: Some(session.certificate.clone()),
            issued,
        };
        p.signature = hk_crypto::sign(&session.secret, "dev", "net", &p.payload());
        p
    }
    fn payload(&self) -> Vec<u8> {
        if let Some(c) = &self.certificate {
            return borsh::to_vec(&(self.author, self.sequence, &self.message, self.issued, c))
                .unwrap();
        }
        borsh::to_vec(&(self.author, self.sequence, &self.message)).unwrap()
    }
    pub fn valid(&self) -> bool {
        if let Some(c) = &self.certificate {
            return c.device.nom == self.author
                && c.valid(self.issued)
                && hk_crypto::verify(&c.session, "dev", "net", &self.payload(), &self.signature);
        }
        self.signature.len() == 64
            && hk_crypto::verify(&self.author, "dev", "net", &self.payload(), &self.signature)
    }
}
/// Bounded cache of fully verified certificate chains. Every packet signature and
/// certificate time window is still checked, including when its chain is cached.
#[derive(Default)]
struct CertificateVerifier {
    chains: std::collections::BTreeSet<Hash>,
}
impl CertificateVerifier {
    fn valid(&mut self, p: &Packet) -> bool {
        let Some(c) = &p.certificate else {
            return p.valid();
        };
        if c.device.nom != p.author || p.issued < c.issued || p.issued > c.expires {
            return false;
        }
        let id = digest(c);
        if !self.chains.contains(&id) {
            if !c.valid(p.issued) {
                return false;
            }
            if self.chains.len() >= 256 {
                self.chains.clear();
            }
            self.chains.insert(id);
        }
        hk_crypto::verify(&c.session, "dev", "net", &p.payload(), &p.signature)
    }
}
pub enum Event {
    Packet(Packet),
    Address(String),
    Peers(usize),
    Error(String),
    Ping(u64),
}
enum Command {
    Send(Packet),
    Dial(String),
    Reserve(String),
    Stop,
}
pub struct Node {
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
    rx: Mutex<mpsc::Receiver<Event>>,
    session: hk_crypto::identity::Session,
    sequence: u64,
}
impl Node {
    pub fn start(secret: [u8; 32], topic: String) -> Self {
        Self::with_session(
            hk_crypto::identity::Session::development(&secret, unix_time()),
            topic,
        )
    }
    pub fn with_session(session: hk_crypto::identity::Session, topic: String) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (out, events) = mpsc::channel();
        std::thread::Builder::new()
            .name("HEXKEEP peers".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                match runtime {
                    Ok(rt) => {
                        if let Err(e) = rt.block_on(run(rx, out.clone(), topic)) {
                            let _ = out.send(Event::Error(e));
                        }
                    }
                    Err(e) => {
                        let _ = out.send(Event::Error(e.to_string()));
                    }
                }
            })
            .expect("network thread");
        Self {
            tx,
            rx: Mutex::new(events),
            session,
            sequence: 0,
        }
    }
    pub fn send(&mut self, message: Message) {
        self.sequence += 1;
        let _ = self.tx.send(Command::Send(Packet::session(
            &self.session,
            self.sequence,
            message,
            unix_time(),
        )));
    }
    pub fn reserve(&self, address: String) {
        let _ = self.tx.send(Command::Reserve(address));
    }
    pub fn dial(&self, address: String) {
        let _ = self.tx.send(Command::Dial(address));
    }
    pub fn poll(&self) -> Vec<Event> {
        self.rx.lock().unwrap().try_iter().take(512).collect()
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
    }
}
#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::client::Behaviour,
    dcutr: dcutr::Behaviour,
    autonat: autonat::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
    gossip: gossipsub::Behaviour,
    mdns: libp2p::swarm::behaviour::toggle::Toggle<mdns::tokio::Behaviour>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}
async fn run(
    mut commands: tokio::sync::mpsc::UnboundedReceiver<Command>,
    out: mpsc::Sender<Event>,
    topic_name: String,
) -> Result<(), String> {
    let builder = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        .with_quic();
    // Android has no /etc/resolv.conf. Its platform DNS is used by the Kotlin bridge.
    #[cfg(target_os = "android")]
    let builder = builder.with_dns_config(libp2p::dns::ResolverConfig::new(), Default::default());
    #[cfg(not(target_os = "android"))]
    let builder = builder.with_dns().map_err(|e| e.to_string())?;
    let mut swarm = builder
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| e.to_string())?
        .with_behaviour(|key, relay| {
            let config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_millis(250))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .max_transmit_size(256 * 1024)
                .flood_publish(true)
                .build()?;
            let gossip = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                config,
            )?;
            let peer = key.public().to_peer_id();
            Ok(Behaviour {
                relay,
                dcutr: dcutr::Behaviour::new(peer),
                autonat: autonat::Behaviour::new(peer, Default::default()),
                kad: kad::Behaviour::new(peer, kad::store::MemoryStore::new(peer)),
                gossip,
                mdns: if std::env::var_os("HK_TEST_DISABLE_MDNS").is_some() {
                    None
                } else {
                    Some(mdns::tokio::Behaviour::new(
                        mdns::Config {
                            query_interval: Duration::from_secs(2),
                            ..Default::default()
                        },
                        key.public().to_peer_id(),
                    )?)
                }
                .into(),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/hexkeep/dev/3".into(),
                    key.public(),
                )),
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(2)),
                ),
            })
        })
        .map_err(|e| e.to_string())?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(90)))
        .build();
    let topic = gossipsub::IdentTopic::new(topic_name);
    swarm
        .behaviour_mut()
        .gossip
        .subscribe(&topic)
        .map_err(|e| e.to_string())?;
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .map_err(|e| e.to_string())?;
    swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
        .map_err(|e| e.to_string())?;
    let memory_topic = gossipsub::IdentTopic::new("hk/dev/v03/memories");
    swarm
        .behaviour_mut()
        .gossip
        .subscribe(&memory_topic)
        .map_err(|e| e.to_string())?;
    let mut peers = std::collections::BTreeSet::new();
    let mut dedup = std::collections::BTreeSet::new();
    let mut verifier = CertificateVerifier::default();
    loop {
        tokio::select! {
        command=commands.recv()=>match command{Some(Command::Send(p))=>{let _=swarm.behaviour_mut().gossip.publish(if matches!(p.message,Message::Memory{..}){memory_topic.clone()}else{topic.clone()},borsh::to_vec(&p).unwrap());},Some(Command::Dial(addr))=>{if let Ok(addr)=addr.parse::<libp2p::Multiaddr>(){if let Err(e)=swarm.dial(addr){let _=out.send(Event::Error(e.to_string()));}}},Some(Command::Reserve(addr))=>{if let Ok(mut a)=addr.parse::<libp2p::Multiaddr>(){a.push(libp2p::multiaddr::Protocol::P2pCircuit);if let Err(e)=swarm.listen_on(a){let _=out.send(Event::Error(e.to_string()));}}},Some(Command::Stop)|None=>break},
        event=swarm.select_next_some()=>match event{
        SwarmEvent::NewListenAddr{mut address,..}=>{let own=*swarm.local_peer_id();if !matches!(address.iter().last(),Some(libp2p::multiaddr::Protocol::P2p(peer)) if peer==own){address.push(libp2p::multiaddr::Protocol::P2p(own));}let _=out.send(Event::Address(address.to_string()));},
        SwarmEvent::ConnectionEstablished{peer_id,..}=>{peers.insert(peer_id);swarm.behaviour_mut().gossip.add_explicit_peer(&peer_id);let _=out.send(Event::Peers(peers.len()));},
        SwarmEvent::ConnectionClosed{peer_id,num_established:0,..}=>{peers.remove(&peer_id);let _=out.send(Event::Peers(peers.len()));},
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list)))=>{for(peer,addr)in list{swarm.behaviour_mut().gossip.add_explicit_peer(&peer);let _=swarm.dial(addr);}},
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list)))=>{for(peer,_)in list{swarm.behaviour_mut().gossip.remove_explicit_peer(&peer);}},
        SwarmEvent::Behaviour(BehaviourEvent::Gossip(gossipsub::Event::Message{message,..}))=>{if message.data.len()<256*1024{if let Ok(p)=borsh::from_slice::<Packet>(&message.data){let id=*blake3::hash(&message.data).as_bytes();if p.certificate.is_some()&&p.issued.abs_diff(unix_time())<=120&&!dedup.contains(&id)&&verifier.valid(&p)&&dedup.insert(id){if dedup.len()>20000{dedup.clear();}let _=out.send(Event::Packet(p));}}}},
        SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received{peer_id,info,..}))=>{for addr in info.listen_addrs{swarm.behaviour_mut().kad.add_address(&peer_id,addr);}swarm.add_external_address(info.observed_addr);},
        SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event{result:Ok(rtt),..}))=>{let _=out.send(Event::Ping(rtt.as_millis()as u64));},
        _=>{}
        }
        }
    }
    Ok(())
}
/// All peers independently run the same simulation. Late input replays up to eight ticks.
#[derive(Clone)]
pub struct Rollback {
    pub room: Hash,
    pub players: Vec<Participant>,
    pub local: u8,
    pub battle: Battle,
    pub inputs: BTreeMap<(u32, u8), Input>,
    pub states: BTreeMap<u32, Battle>,
    pub highest: Vec<u32>,
    pub sent: u32,
    pub error: Option<String>,
    pub rollbacks: u32,
    pub last_progress: u64,
    pub transcript: Vec<Packet>,
    pub links: BTreeMap<(u32, u8), (Hash, Hash)>,
    pub sent_hash: Hash,
    pub results: BTreeMap<[u8; 32], Hash>,
}
impl Rollback {
    pub fn new(
        cell: u64,
        players: Vec<Participant>,
        nonce: u64,
        local_key: [u8; 32],
    ) -> Result<Self, String> {
        if !(2..=10).contains(&players.len()) {
            return Err("2 à 10 joueurs requis".into());
        }
        let mut unique = std::collections::BTreeSet::new();
        if players.iter().any(|p| !unique.insert(p.key)) {
            return Err("Joueur dupliqué".into());
        }
        let local = players
            .iter()
            .position(|p| p.key == local_key)
            .ok_or("Joueur absent")? as u8;
        let room = digest(&(cell, &players, nonce));
        let seed = u64::from_le_bytes(room[..8].try_into().unwrap());
        let mut battle = Battle::new(
            seed,
            players[0].realm,
            players[0].role,
            0,
            players.len() == 2,
        );
        battle.fighters = players
            .iter()
            .enumerate()
            .map(|(i, p)| Fighter::new(i as u8, p.realm, p.role, false))
            .collect();
        let states = BTreeMap::from([(0, battle.clone())]);
        let count = players.len();
        Ok(Self {
            room,
            players,
            local,
            battle,
            inputs: BTreeMap::new(),
            states,
            highest: vec![0; count],
            sent: 0,
            error: None,
            rollbacks: 0,
            last_progress: 0,
            transcript: vec![],
            links: BTreeMap::new(),
            sent_hash: [0; 32],
            results: BTreeMap::new(),
        })
    }
    pub fn configure(
        &mut self,
        mode: u8,
        codex: hk_crown::Codex,
        walls: u32,
    ) -> Result<(), String> {
        if mode > 2 || !codex.valid() || walls > 100 {
            return Err("Configuration de Champ invalide".into());
        }
        self.room = digest(&(self.room, mode, &codex, walls));
        self.battle.apply_codex(codex);
        self.battle.duel = mode == 0;
        if mode == 2 {
            self.battle.begin_siege(self.players[0].realm, walls);
        }
        self.states = BTreeMap::from([(0, self.battle.clone())]);
        Ok(())
    }
    pub fn receive_linked(&mut self, id: u8, tick: u32, input: Input, previous: Hash) -> bool {
        if id as usize >= self.players.len() || tick < 3 {
            return false;
        }
        let hash = input_hash(
            self.room,
            self.players[id as usize].key,
            tick,
            input,
            previous,
        );
        let prev_ok = if tick == 3 {
            previous == [0; 32]
        } else {
            self.links
                .get(&(tick - 1, id))
                .is_none_or(|(_, h)| *h == previous)
        };
        let next_ok = self
            .links
            .get(&(tick + 1, id))
            .is_none_or(|(p, _)| *p == hash);
        if !prev_ok || !next_ok {
            self.error = Some("Chaîne d'entrées rompue".into());
            return false;
        }
        self.links.insert((tick, id), (previous, hash));
        self.receive(id, tick, input)
    }
    pub fn receive(&mut self, id: u8, tick: u32, input: Input) -> bool {
        if id as usize >= self.players.len() || tick > self.battle.tick + 180 || tick == 0 {
            return false;
        }
        if let Some(prev) = self.inputs.get(&(tick, id)) {
            if *prev != input {
                self.error = Some("Entrées contradictoires signées".into());
                return false;
            }
            return true;
        }
        self.inputs.insert((tick, id), input);
        // A later packet is not evidence that every earlier input arrived.
        // Prediction must stay within the rollback window of the first gap.
        while self
            .inputs
            .contains_key(&(self.highest[id as usize] + 1, id))
        {
            self.highest[id as usize] += 1;
        }
        if tick <= self.battle.tick {
            if self.battle.tick - tick >= 8 {
                self.error = Some("Entrée au-delà de la fenêtre de rollback".into());
                return false;
            }
            if let Some(state) = self.states.get(&(tick - 1)).cloned() {
                let target = self.battle.tick;
                self.battle = state;
                while self.battle.tick < target && !self.battle.finished {
                    self.advance();
                }
                self.rollbacks += 1;
            }
        }
        true
    }
    fn advance(&mut self) {
        let tick = self.battle.tick + 1;
        let inputs: Vec<(u8, Input)> = (0..self.players.len())
            .map(|i| {
                let id = i as u8;
                let input = self.inputs.get(&(tick, id)).copied().unwrap_or_default();
                (id, input)
            })
            .collect();
        self.battle.step(&inputs);
        self.states.insert(self.battle.tick, self.battle.clone());
        let oldest = self.battle.tick.saturating_sub(120);
        self.states.retain(|tick, _| *tick >= oldest);
    }
    pub fn local_input(&mut self, input: Input) -> Option<(u32, Input)> {
        let tick = self.battle.tick + 3;
        if tick <= self.sent {
            return None;
        }
        self.sent = tick;
        self.receive(self.local, tick, input);
        Some((tick, input))
    }
    pub fn step(&mut self) -> bool {
        if self.error.is_some() || self.battle.finished {
            return false;
        }
        let min = *self.highest.iter().min().unwrap_or(&0);
        if self.battle.tick > min + 6 {
            return false;
        }
        self.advance();
        true
    }
    pub fn confirmed(&self) -> u32 {
        self.highest
            .iter()
            .copied()
            .min()
            .unwrap_or(0)
            .min(self.battle.tick)
    }
    pub fn seed_initial_delay(&mut self) {
        self.highest.fill(2);
        for tick in 1..=2 {
            for id in 0..self.players.len() {
                self.inputs.insert((tick, id as u8), Input::default());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn players() -> Vec<Participant> {
        (0..2)
            .map(|i| Participant {
                key: hk_crypto::public(&[i + 1; 32]),
                name: format!("TEST {i}"),
                realm: Realm::from_index(i as usize),
                role: Role::Foudre,
            })
            .collect()
    }
    #[test]
    fn rollback_matches_ordered() {
        let p = players();
        let mut a = Rollback::new(7, p.clone(), 9, p[0].key).unwrap();
        let mut b = a.clone();
        a.seed_initial_delay();
        b.seed_initial_delay();
        for tick in 3..120 {
            let input = Input {
                move_x: 400,
                aim_x: 900,
                shoot: true,
                ..Input::default()
            };
            a.receive(0, tick, input);
            a.receive(1, tick, input);
            a.step();
            b.receive(0, tick, input);
            b.step();
            b.receive(1, tick, input);
        }
        assert_eq!(a.battle.hash(), b.battle.hash());
    }
    #[test]
    fn reordered_input_cannot_escape_rollback_window() {
        let p = players();
        let mut ordered = Rollback::new(7, p.clone(), 9, p[0].key).unwrap();
        ordered.seed_initial_delay();
        let mut delayed = ordered.clone();
        let moving = Input {
            move_x: 700,
            ..Input::default()
        };
        for tick in 3..=40 {
            for id in 0..2 {
                ordered.receive(id, tick, moving);
                if !(id == 1 && tick == 3) {
                    delayed.receive(id, tick, moving);
                }
            }
        }
        for _ in 0..25 {
            ordered.step();
            delayed.step();
        }
        assert_eq!(delayed.highest[1], 2);
        assert!(delayed.battle.tick <= 9);
        assert!(delayed.receive(1, 3, moving));
        while delayed.battle.tick < ordered.battle.tick {
            assert!(delayed.step());
        }
        assert!(delayed.error.is_none());
        assert_eq!(ordered.battle.hash(), delayed.battle.hash());
    }
    #[test]
    fn cached_chain_does_not_accept_modified_or_expired_packets() {
        let session = hk_crypto::identity::Session::development(&[12; 32], 100);
        let message = Message::State {
            room: [1; 32],
            tick: 30,
            hash: [2; 32],
        };
        let p = Packet::session(&session, 1, message.clone(), 101);
        let mut verifier = CertificateVerifier::default();
        assert!(verifier.valid(&p));
        assert!(verifier.valid(&p));
        let mut forged = p.clone();
        forged.sequence += 1;
        assert!(!verifier.valid(&forged));
        let mut changed = p.clone();
        changed.certificate.as_mut().unwrap().device.nom = [3; 32];
        assert!(!verifier.valid(&changed));
        let expired = Packet::session(&session, 2, message, 100 + 86401);
        assert!(!verifier.valid(&expired));
        assert_eq!(verifier.chains.len(), 1);
    }
    #[test]
    fn packet_authentication() {
        let p = Packet::new(
            &[8; 32],
            1,
            Message::State {
                room: [2; 32],
                tick: 30,
                hash: [3; 32],
            },
        );
        assert!(p.valid());
        let mut q = p.clone();
        q.sequence += 1;
        assert!(!q.valid());
    }
    #[test]
    fn conflicting_inputs_abort() {
        let p = players();
        let mut r = Rollback::new(7, p.clone(), 0, p[0].key).unwrap();
        r.receive(1, 3, Input::default());
        assert!(!r.receive(
            1,
            3,
            Input {
                shoot: true,
                ..Input::default()
            }
        ));
        assert!(r.error.is_some());
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    #[test]
    fn real_noise_tcp_transport() {
        let topic = format!("hk/dev/test/{}", std::process::id());
        let mut a = Node::start([51; 32], topic.clone());
        let b = Node::start([52; 32], topic);
        let deadline = std::time::Instant::now() + Duration::from_secs(12);
        let mut received = false;
        let mut dialed = false;
        while std::time::Instant::now() < deadline {
            for e in a.poll() {
                if let Event::Address(addr) = e {
                    if !dialed {
                        let parts: Vec<_> = addr.split('/').collect();
                        let Some(i) = parts.iter().position(|v| *v == "tcp") else {
                            continue;
                        };
                        let port = parts[i + 1];
                        b.dial(format!("/ip4/127.0.0.1/tcp/{port}"));
                        dialed = true;
                    }
                }
            }
            a.send(Message::State {
                room: [0; 32],
                tick: 30,
                hash: [7; 32],
            });
            for e in b.poll() {
                if let Event::Packet(p) = e {
                    assert!(p.valid());
                    if let Message::State { hash, .. } = p.message {
                        assert_eq!(hash, [7; 32]);
                        received = true;
                    }
                }
            }
            if received {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(received, "No authenticated payload received over TCP/Noise");
    }
}

pub fn input_hash(room: Hash, author: Hash, tick: u32, input: Input, previous: Hash) -> Hash {
    digest(&(room, author, tick, input, previous))
}
