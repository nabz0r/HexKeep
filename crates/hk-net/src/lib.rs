//! Development LAN transport. No coordinator stores or authoritatively decides game state.
use borsh::{BorshDeserialize, BorshSerialize};
use futures::StreamExt;
use hk_proto::*;
use hk_sim::{Battle, Fighter};
use libp2p::{
    gossipsub, identify, mdns, noise, ping,
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
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Packet {
    pub author: [u8; 32],
    pub sequence: u64,
    pub message: Message,
    pub signature: Vec<u8>,
}
impl Packet {
    pub fn new(secret: &[u8; 32], sequence: u64, message: Message) -> Self {
        let mut p = Self {
            author: hk_crypto::public(secret),
            sequence,
            message,
            signature: vec![],
        };
        p.signature = hk_crypto::sign(secret, "dev", "net", &p.payload());
        p
    }
    fn payload(&self) -> Vec<u8> {
        borsh::to_vec(&(self.author, self.sequence, &self.message)).unwrap()
    }
    pub fn valid(&self) -> bool {
        self.signature.len() == 64
            && hk_crypto::verify(&self.author, "dev", "net", &self.payload(), &self.signature)
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
    Stop,
}
pub struct Node {
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
    rx: Mutex<mpsc::Receiver<Event>>,
    secret: [u8; 32],
    sequence: u64,
}
impl Node {
    pub fn start(secret: [u8; 32], topic: String) -> Self {
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
            secret,
            sequence: 0,
        }
    }
    pub fn send(&mut self, message: Message) {
        self.sequence += 1;
        let _ = self.tx.send(Command::Send(Packet::new(
            &self.secret,
            self.sequence,
            message,
        )));
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
    gossip: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}
async fn run(
    mut commands: tokio::sync::mpsc::UnboundedReceiver<Command>,
    out: mpsc::Sender<Event>,
    topic_name: String,
) -> Result<(), String> {
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        .with_behaviour(|key| {
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
            Ok(Behaviour {
                gossip,
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config {
                        query_interval: Duration::from_secs(2),
                        ..Default::default()
                    },
                    key.public().to_peer_id(),
                )?,
                identify: identify::Behaviour::new(identify::Config::new(
                    "/hexkeep/dev/1".into(),
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
    let mut peers = std::collections::BTreeSet::new();
    let mut dedup = std::collections::BTreeSet::new();
    loop {
        tokio::select! {
        command=commands.recv()=>match command{Some(Command::Send(p))=>{let _=swarm.behaviour_mut().gossip.publish(topic.clone(),borsh::to_vec(&p).unwrap());},Some(Command::Dial(addr))=>{if let Ok(addr)=addr.parse::<libp2p::Multiaddr>(){if let Err(e)=swarm.dial(addr){let _=out.send(Event::Error(e.to_string()));}}},Some(Command::Stop)|None=>break},
        event=swarm.select_next_some()=>match event{
        SwarmEvent::NewListenAddr{address,..}=>{let _=out.send(Event::Address(address.to_string()));},
        SwarmEvent::ConnectionEstablished{peer_id,..}=>{peers.insert(peer_id);swarm.behaviour_mut().gossip.add_explicit_peer(&peer_id);let _=out.send(Event::Peers(peers.len()));},
        SwarmEvent::ConnectionClosed{peer_id,num_established:0,..}=>{peers.remove(&peer_id);let _=out.send(Event::Peers(peers.len()));},
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list)))=>{for(peer,addr)in list{swarm.behaviour_mut().gossip.add_explicit_peer(&peer);let _=swarm.dial(addr);}},
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list)))=>{for(peer,_)in list{swarm.behaviour_mut().gossip.remove_explicit_peer(&peer);}},
        SwarmEvent::Behaviour(BehaviourEvent::Gossip(gossipsub::Event::Message{message,..}))=>{if message.data.len()<256*1024{if let Ok(p)=borsh::from_slice::<Packet>(&message.data){let id=*blake3::hash(&message.data).as_bytes();if p.valid()&&dedup.insert(id){if dedup.len()>20000{dedup.clear();}let _=out.send(Event::Packet(p));}}}},
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
            results: BTreeMap::new(),
        })
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
        self.highest[id as usize] = self.highest[id as usize].max(tick);
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
        let mut tick = 1;
        while tick <= self.battle.tick {
            if (0..self.players.len()).any(|p| !self.inputs.contains_key(&(tick, p as u8))) {
                break;
            }
            tick += 1;
        }
        tick - 1
    }
    pub fn seed_initial_delay(&mut self) {
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
                        let port = addr.rsplit('/').next().unwrap();
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
