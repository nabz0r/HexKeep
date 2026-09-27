use super::*;
use hk_net::{Event, Message, Node, Packet, Participant, Rollback};
pub struct Peer {
    pub player: Participant,
    pub cell: u64,
    pub seen: u64,
}
pub struct Online {
    pub rollback: Rollback,
    pub start: Packet,
    pub countdown: u16,
    pub last_received: u64,
    pub peer_last_received: Vec<u64>,
    pub last_state: u32,
    pub finished_packets: Vec<Packet>,
    pub finish_sent: bool,
}
impl Game {
    fn participant(&self) -> Participant {
        Participant {
            key: hk_crypto::public(&self.save.secret),
            name: self.save.name.chars().take(18).collect(),
            realm: self.save.realm,
            role: self.save.role,
        }
    }
    pub fn start_network(&mut self) {
        if self.node.is_some() || !self.is_dev {
            return;
        }
        let region = marche(self.save.world.current).unwrap_or(0);
        self.node = Some(Node::with_session(
            self.session.clone(),
            format!("hk/dev/marche/{region:x}/v05"),
        ));
        self.save.world.banner = true;
        self.native_action = 3;
        self.dirty = true;
        self.toast("Bannière levée. Recherche sur le Wi-Fi local.");
    }
    pub fn connect(&mut self, address: String) {
        self.start_network();
        if let Some(n) = &self.node {
            n.dial(address);
        }
    }
    pub fn connect_text(&mut self) {
        let value = self.keyboard.trim();
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2
            || parts[0].parse::<std::net::Ipv4Addr>().is_err()
            || parts[1].parse::<u16>().is_err()
        {
            self.toast("Adresse attendue : 192.168.1.20:12345");
            return;
        }
        self.connect(format!("/ip4/{}/tcp/{}", parts[0], parts[1]));
        self.toast("Connexion directe demandée.");
    }
    fn presence(&mut self) {
        let player = self.participant();
        let cell = if self.save.world.banner {
            self.save.world.current
        } else {
            0
        };
        if let Some(n) = &mut self.node {
            for memory in &self.save.expansion.campaign.memories {
                if memory.carrier == Some(player.key) {
                    if let Some(cell10) = hk_world::public_memory_cell(memory.cell) {
                        n.send(Message::Memory {
                            realm: memory.realm,
                            cell10,
                            time: self.now,
                        });
                    }
                }
            }
            n.send(Message::Presence {
                player,
                cell,
                epoch: self.now,
            });
        }
    }
    pub fn poll_network(&mut self) {
        let events = self.node.as_ref().map(Node::poll).unwrap_or_default();
        for event in events {
            match event {
                Event::Address(addr) => {
                    if !self.addresses.contains(&addr) {
                        self.addresses.push(addr);
                    }
                }
                Event::Peers(n) => {
                    self.peer_count = n;
                    self.presence();
                }
                Event::Ping(ms) => self.latency = ms,
                Event::Error(_) => {
                    self.toast("Réseau local : connexion indisponible.");
                }
                Event::Packet(p) => self.receive_packet(p),
            }
        }
        if self.node.is_some() && self.ticks % 30 == 0 {
            self.presence();
            self.peers
                .retain(|_, p| self.ticks.saturating_sub(p.seen) < 300);
            if self.ticks % 150 == 0 {
                let eligible: Vec<_> = self
                    .save
                    .ledger
                    .events
                    .iter()
                    .filter(|e| {
                        e.body.time.saturating_add(900) <= self.now
                            && marche(e.body.cell) == marche(self.save.world.current)
                    })
                    .map(|e| e.id())
                    .collect();
                let page = (self.ticks / 150) as usize % eligible.len().div_ceil(512).max(1);
                let ids = eligible.into_iter().skip(page * 512).take(512).collect();
                if let Some(n) = &mut self.node {
                    n.send(Message::Inventory { ids });
                    let data = serde_json::to_string(&self.save.expansion.authority).unwrap();
                    if data.len() < 200000 {
                        n.send(Message::Authority { data });
                    }
                }
                let region = marche(self.save.world.current);
                let events = self
                    .save
                    .ledger
                    .events
                    .iter()
                    .filter(|e| e.body.time + 900 <= self.now && marche(e.body.cell) == region)
                    .rev()
                    .take(128)
                    .cloned()
                    .collect();
                if let Some(n) = &mut self.node {
                    n.send(Message::Ledger { events });
                }
            }
        }
    }
    fn receive_packet(&mut self, p: Packet) {
        // Only Node emits these packets, after verifying their session and signature.
        if p.author == hk_crypto::public(&self.save.secret) {
            return;
        }
        if self
            .save
            .expansion
            .authority
            .banned(&p.author, hk_crown::Scope::All, self.now)
        {
            return;
        }
        if let Some(c) = &p.certificate {
            self.expansion.peer_certificates.insert(p.author, c.clone());
        }
        match p.message.clone() {
            Message::StartV2 {
                cell,
                players,
                nonce,
                mode,
                codex,
                walls,
            } => {
                let own = self.participant();
                if self.online.is_some()
                    || !self.save.world.banner
                    || cell != self.save.world.current
                    || players.first().map(|v| v.key) != Some(p.author)
                    || !players
                        .iter()
                        .any(|v| v.key == own.key && v.realm == own.realm && v.role == own.role)
                    || players
                        .iter()
                        .any(|v| v.key != own.key && !self.peers.contains_key(&v.key))
                    || self.save.expansion.authority.banned(
                        &own.key,
                        hk_crown::Scope::Pvp,
                        self.now,
                    )
                {
                    return;
                }
                let local_codex = self
                    .save
                    .expansion
                    .authority
                    .state(&self.save.ledger)
                    .map(|s| s.codex)
                    .unwrap_or_default();
                if codex != local_codex {
                    return;
                }
                if let Ok(mut rb) = Rollback::new(cell, players, nonce, own.key) {
                    if rb.configure(mode, codex, walls).is_ok() {
                        rb.seed_initial_delay();
                        self.begin_online(rb, p);
                    }
                }
            }
            Message::Recovery { share } => {
                if share.owner == p.author
                    && share.recipient == hk_crypto::public(&self.save.secret)
                    && share.open(&self.save.secret).is_ok()
                {
                    self.save.expansion.held_shares.push(share);
                    self.dirty = true;
                    self.toast("Une part du Serment t'a été confiée.");
                }
            }
            Message::Witness { witness } => {
                if witness.observer == p.author
                    && witness.valid()
                    && self.now.abs_diff(witness.time) < 120
                {
                    self.save
                        .expansion
                        .witnesses
                        .retain(|w| w.observer != witness.observer || w.subject != witness.subject);
                    self.save.expansion.witnesses.push(witness);
                }
            }
            Message::Authority { data } => {
                if let Ok(other) = serde_json::from_str::<hk_crown::Authority>(&data) {
                    if self
                        .save
                        .expansion
                        .authority
                        .merge(&other, &self.save.ledger)
                        .is_ok()
                    {
                        self.apply_canonical();
                    }
                }
            }
            Message::Inventory { ids } => {
                if ids.len() > 512 {
                    return;
                }
                let known: std::collections::BTreeSet<_> =
                    self.save.ledger.events.iter().map(|e| e.id()).collect();
                let missing = ids
                    .into_iter()
                    .filter(|id| !known.contains(id))
                    .take(128)
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    if let Some(n) = &mut self.node {
                        n.send(Message::Missing { ids: missing });
                    }
                }
            }
            Message::Missing { ids } => {
                if ids.len() > 128 {
                    return;
                }
                let events = self
                    .save
                    .ledger
                    .events
                    .iter()
                    .filter(|e| {
                        ids.contains(&e.id())
                            && e.body.time.saturating_add(900) <= self.now
                            && marche(e.body.cell) == marche(self.save.world.current)
                    })
                    .cloned()
                    .collect();
                if let Some(n) = &mut self.node {
                    n.send(Message::Ledger { events });
                }
            }
            Message::Memory {
                realm,
                cell10,
                time,
            } => {
                if self.now.abs_diff(time) < 60 {
                    if let Some(m) = self.save.expansion.campaign.memories.iter_mut().find(|m| {
                        m.realm == realm && (m.carrier.is_none() || m.carrier == Some(p.author))
                    }) {
                        if let Some(cell) = hk_world::parent_cell(cell10) {
                            m.cell = cell;
                        }
                    }
                }
            }
            Message::Presence {
                player,
                cell,
                epoch,
            } => {
                if player.key != p.author
                    || player.name.chars().count() > 18
                    || epoch.abs_diff(self.now) > 120
                {
                    return;
                }
                if cell == 0 {
                    self.peers.remove(&p.author);
                    return;
                }
                if marche(cell) != marche(self.save.world.current) {
                    return;
                }
                if self.peers.len() < 64 || self.peers.contains_key(&p.author) {
                    self.peers.insert(
                        p.author,
                        Peer {
                            player,
                            cell,
                            seen: self.ticks,
                        },
                    );
                }
            }
            Message::Start {
                cell,
                players,
                nonce,
            } => {
                if self.online.is_some()
                    || !self.save.world.banner
                    || cell != self.save.world.current
                    || players.first().map(|v| v.key) != Some(p.author)
                {
                    return;
                }
                let own = self.participant();
                if !players
                    .iter()
                    .any(|v| v.key == own.key && v.realm == own.realm && v.role == own.role)
                {
                    return;
                }
                if players
                    .iter()
                    .any(|v| v.key != own.key && !self.peers.contains_key(&v.key))
                {
                    return;
                }
                if let Ok(mut rb) = Rollback::new(cell, players, nonce, own.key) {
                    rb.seed_initial_delay();
                    self.begin_online(rb, p);
                }
            }
            Message::MissingInputs {
                room,
                player,
                from,
                to,
            } => {
                if player != hk_crypto::public(&self.save.secret)
                    || from < 3
                    || to < from
                    || to - from > 15
                {
                    return;
                }
                if let Some(o) = &self.online {
                    if room != o.rollback.room
                        || !o.rollback.players.iter().any(|v| v.key == p.author)
                    {
                        return;
                    }
                    let messages: Vec<_> = o
                        .rollback
                        .transcript
                        .iter()
                        .filter(|v| v.author == player)
                        .filter_map(|v| match &v.message {
                            Message::Input { tick, .. } if *tick >= from && *tick <= to => {
                                Some(v.message.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    if let Some(n) = &mut self.node {
                        for message in messages {
                            n.send(message);
                        }
                    }
                }
            }
            Message::Input {
                room,
                tick,
                input,
                previous,
            } => {
                if let Some(o) = &mut self.online {
                    if o.rollback.room != room {
                        return;
                    }
                    if let Some(id) = o.rollback.players.iter().position(|v| v.key == p.author) {
                        o.last_received = self.ticks;
                        o.peer_last_received[id] = self.ticks;
                        let fresh = !o.rollback.inputs.contains_key(&(tick, id as u8));
                        let accepted = o.rollback.receive_linked(id as u8, tick, input, previous);
                        if fresh && accepted && o.rollback.transcript.len() < 60000 {
                            o.rollback.transcript.push(p);
                        }
                    }
                }
            }
            Message::State { room, tick, hash } => {
                if let Some(o) = &mut self.online {
                    if o.rollback.room != room
                        || !o.rollback.players.iter().any(|v| v.key == p.author)
                    {
                        return;
                    }
                    if tick <= o.rollback.confirmed() {
                        if let Some(state) = o.rollback.states.get(&tick) {
                            if state.hash() != hash {
                                o.rollback.error =
                                    Some("Désynchronisation : le combat est annulé.".into());
                            }
                        }
                    }
                }
            }
            Message::Finish {
                room,
                tick: _,
                hash: _,
            } => {
                if let Some(o) = &mut self.online {
                    if o.rollback.room == room
                        && o.rollback.players.iter().any(|v| v.key == p.author)
                        && !o.finished_packets.iter().any(|v| v.author == p.author)
                    {
                        o.finished_packets.push(p);
                    }
                }
            }
            Message::Ledger { events } => {
                if events.len() > 128 {
                    return;
                }
                let region = marche(self.save.world.current);
                let events: Vec<_> = events
                    .into_iter()
                    .filter(|e| {
                        e.valid("dev")
                            && e.body.time + 900 <= self.now
                            && marche(e.body.cell) == region
                    })
                    .collect();
                if self.save.ledger.events.len() + events.len() < 10000 {
                    self.save.ledger.merge(&Ledger { events });
                    let fold = self
                        .save
                        .ledger
                        .fold("dev", &std::collections::BTreeSet::new());
                    for (id, realm) in fold.bastions {
                        if let Some(c) = self.save.world.cells.get_mut(&id) {
                            c.bastion = Some(realm);
                        }
                    }
                    self.dirty = true;
                }
            }
        }
    }
    fn begin_online(&mut self, rb: Rollback, start: Packet) {
        self.battle = Some(rb.battle.clone());
        let participant_count = rb.players.len();
        self.online = Some(Online {
            rollback: rb,
            start,
            countdown: 60,
            last_received: self.ticks,
            peer_last_received: vec![self.ticks; participant_count],
            last_state: 0,
            finished_packets: vec![],
            finish_sent: false,
        });
        self.tutorial = false;
        self.expedition = None;
        self.screen = 6;
        self.touches.clear();
        self.toast("Le Champ s'ouvre. Combat provisoire, sans PR.");
    }
    pub fn challenge(&mut self, all: bool) {
        self.challenge_mode(if all { 1 } else { 0 });
    }
    pub fn challenge_mode(&mut self, mode: u8) {
        let all = mode != 0;
        let own = self.participant();
        if self
            .save
            .expansion
            .authority
            .banned(&own.key, hk_crown::Scope::Pvp, self.now)
        {
            self.toast("Un Édit suspend ton accès au PvP.");
            return;
        }
        let mut players = vec![own.clone()];
        players.extend(
            self.peers
                .values()
                .filter(|p| p.cell == self.save.world.current)
                .map(|p| p.player.clone())
                .take(if all { 9 } else { 1 }),
        );
        if players.len() < 2 {
            self.toast("Aucun pair à bannière levée dans ta cellule.");
            return;
        }
        let nonce = self.now ^ self.ticks;
        let codex = self
            .save
            .expansion
            .authority
            .state(&self.save.ledger)
            .map(|s| s.codex)
            .unwrap_or_default();
        let walls = self.walls();
        let msg = Message::StartV2 {
            mode,
            codex: codex.clone(),
            walls,
            cell: self.save.world.current,
            players: players.clone(),
            nonce,
        };
        let packet = Packet::session(&self.session, self.ticks, msg.clone(), self.now);
        if let Ok(mut rb) = Rollback::new(self.save.world.current, players, nonce, own.key) {
            if rb.configure(mode, codex, walls).is_err() {
                return;
            }
            rb.seed_initial_delay();
            if let Some(n) = &mut self.node {
                n.send(msg);
            }
            self.begin_online(rb, packet);
        }
    }
    pub fn step_online(&mut self, input: Input) {
        let mut notices = Vec::new();
        let Some(o) = &mut self.online else { return };
        if o.countdown > 0 {
            if o.countdown % 30 == 0 && o.rollback.local == 0 {
                if let Some(n) = &mut self.node {
                    n.send(o.start.message.clone());
                }
            }
            o.countdown -= 1;
            return;
        }
        if o.peer_last_received.iter().enumerate().any(|(id, seen)| {
            id != o.rollback.local as usize
                && !o.rollback.results.contains_key(&o.rollback.players[id].key)
                && self.ticks.saturating_sub(*seen) > 150
        }) {
            notices.push("Pair absent depuis 5 secondes : combat interrompu.");
            o.rollback.error = Some("Pair silencieux : forfait provisoire, sans finalité.".into());
        }
        if o.rollback.error.is_some() {
            self.screen = 17;
            return;
        }
        if let Some((tick, input)) = o.rollback.local_input(input) {
            let previous = o.rollback.sent_hash;
            o.rollback.sent_hash = hk_net::input_hash(
                o.rollback.room,
                hk_crypto::public(&self.save.secret),
                tick,
                input,
                previous,
            );
            let msg = Message::Input {
                room: o.rollback.room,
                tick,
                input,
                previous,
            };
            let p = Packet::session(&self.session, self.ticks, msg.clone(), self.now);
            o.rollback.transcript.push(p);
            if let Some(n) = &mut self.node {
                n.send(msg);
            }
        }
        o.rollback.step();
        // Request a bounded range when prediction is waiting at a missing input.
        if self.ticks % 15 == 0 {
            if let Some(n) = &mut self.node {
                for (id, highest) in o.rollback.highest.iter().enumerate() {
                    if id != o.rollback.local as usize && o.rollback.battle.tick >= highest + 5 {
                        n.send(Message::MissingInputs {
                            room: o.rollback.room,
                            player: o.rollback.players[id].key,
                            from: highest + 1,
                            to: (highest + 16).min(o.rollback.battle.tick + 2),
                        });
                    }
                }
                if o.finish_sent {
                    n.send(Message::Finish {
                        room: o.rollback.room,
                        tick: o.rollback.battle.tick,
                        hash: o.rollback.battle.hash(),
                    });
                }
            }
        }
        let confirmed = o.rollback.confirmed();
        for packet in &o.finished_packets {
            if let Message::Finish { tick, hash, .. } = packet.message {
                if tick <= confirmed {
                    if o.rollback
                        .states
                        .get(&tick)
                        .is_some_and(|s| s.hash() == hash)
                    {
                        o.rollback.results.insert(packet.author, hash);
                    } else {
                        o.rollback.error = Some("Résultat divergent".into());
                    }
                }
            }
        }

        if confirmed / 30 > o.last_state / 30 {
            let tick = confirmed / 30 * 30;
            if let Some(state) = o.rollback.states.get(&tick) {
                if let Some(n) = &mut self.node {
                    n.send(Message::State {
                        room: o.rollback.room,
                        tick,
                        hash: state.hash(),
                    });
                }
                o.last_state = tick;
            }
        }
        self.battle = Some(o.rollback.battle.clone());
        if o.rollback.battle.finished && confirmed >= o.rollback.battle.tick {
            if !o.finish_sent {
                let hash = o.rollback.battle.hash();
                let msg = Message::Finish {
                    room: o.rollback.room,
                    tick: o.rollback.battle.tick,
                    hash,
                };
                let p = Packet::session(&self.session, self.ticks, msg.clone(), self.now);
                o.finished_packets.push(p);
                o.rollback
                    .results
                    .insert(hk_crypto::public(&self.save.secret), hash);
                if let Some(n) = &mut self.node {
                    n.send(msg);
                }
                o.finish_sent = true;
            }
            if o.rollback.results.len() == o.rollback.players.len() {
                self.screen = 12;
            }
        }
        for notice in notices {
            self.toast(notice);
        }
    }
    pub fn finish_online(&mut self) {
        if let Some(o) = self.online.take() {
            let proof = serde_json::json!({"version":1,"network":"dev","start":o.start,"inputs":o.rollback.transcript,"results":o.finished_packets,"tick":o.rollback.battle.tick,"hash":hex(&o.rollback.battle.hash()),"complete":o.rollback.results.len()==o.rollback.players.len()&&o.rollback.error.is_none()});
            self.last_proof = hk_net::compact_proof(proof).to_string();
            if o.rollback.results.len() == o.rollback.players.len() && o.rollback.error.is_none() {
                if self.save.expansion.case_files.len() >= 3 {
                    self.save.expansion.case_files.remove(0);
                }
                self.save.expansion.case_files.push(self.last_proof.clone());
                self.dirty = true;
            }
            self.last_hash = hex(&o.rollback.battle.hash());
        }
        self.toast("Preuve disponible dans PAIRS. Aucun PR attribué.");
    }
    pub fn network_tap(&mut self, x: i32, y: i32) {
        let w = self.width;
        if y > 211 {
            self.screen = 7;
            return;
        }
        if (50..75).contains(&y) {
            self.start_network();
        } else if (83..108).contains(&y) {
            self.challenge(false);
        } else if (116..141).contains(&y) {
            self.challenge(true);
        } else if (149..174).contains(&y) {
            self.start_network();
            self.native_action = 14;
        } else if (180..205).contains(&y) {
            if x < w / 2 {
                self.screen = 9;
            } else if !self.last_proof.is_empty() {
                self.native_action = 6;
            } else {
                self.toast("Aucune preuve de combat à exporter.");
            }
        }
    }
    pub fn draw_network(&self, c: &mut Canvas) {
        let w = c.w;
        c.center(14, "LES LANTERNES VOISINES", GOLD, 2);
        c.text(
            15,
            38,
            &format!(
                "{} PAIRS · {} MS · RÉSEAU DEV",
                self.peer_count, self.latency
            ),
            MUTED,
            1,
        );
        c.button(
            15,
            52,
            204,
            if self.node.is_some() {
                "RECHERCHE ACTIVE (WI-FI)"
            } else {
                "LEVER BANNIÈRE / CONNECTER"
            },
            self.node.is_none(),
        );
        c.button(15, 85, 204, "DÉFIER LE PREMIER PAIR", false);
        c.button(15, 118, 204, "OUVRIR UN CHAMP À 10 MAX", false);
        c.button(15, 151, 204, "SAISIR UNE IP:PORT", false);
        c.button(15, 184, 98, "CODEX / AIDE", false);
        c.button(123, 184, 96, "EXPORT REJEU", false);
        c.text(234, 54, "BANNIÈRES DANS LA MARCHE", GOLD, 1);
        for (i, p) in self.peers.values().take(6).enumerate() {
            c.text(
                234,
                73 + i as i32 * 16,
                &format!("{} · {}", p.player.name, &hex(&p.player.key[..2])),
                REALMS[p.player.realm.index()],
                1,
            );
        }
        if self.peers.is_empty() {
            c.text(234, 82, "Aucun Veilleur découvert.", MUTED, 1);
            c.text(234, 100, "Même Wi-Fi, même cellule.", MUTED, 1);
            c.text(234, 118, "Lève ta bannière pour jouer.", MUTED, 1);
        }
        if let Some(addr) = self.addresses.iter().find(|a| !a.contains("127.0.0.1")) {
            c.text(234, 177, "ADRESSE LOCALE", MUTED, 1);
            c.text(234, 190, addr, WHITE, 1);
        }
        c.button(w / 2 - 80, 215, 160, "RETOUR À LA CARTE", true);
    }
}

#[cfg(test)]
mod silence_tests {
    use super::*;
    #[test]
    fn one_silent_participant_is_not_hidden_by_other_traffic() {
        let mut g = Game::new("", true);
        let own = hk_crypto::public(&g.save.secret);
        let players = vec![
            g.participant(),
            Participant {
                key: hk_crypto::public(&[7; 32]),
                name: "A".into(),
                realm: Realm::Skarn,
                role: Role::Foudre,
            },
            Participant {
                key: hk_crypto::public(&[8; 32]),
                name: "B".into(),
                realm: Realm::Vylde,
                role: Role::Foudre,
            },
        ];
        let rb = Rollback::new(g.save.world.current, players.clone(), 10, own).unwrap();
        let packet = Packet::session(
            &g.session,
            1,
            Message::Start {
                cell: g.save.world.current,
                players,
                nonce: 10,
            },
            hk_net::unix_time(),
        );
        g.begin_online(rb, packet);
        g.ticks = 200;
        let o = g.online.as_mut().unwrap();
        o.countdown = 0;
        o.last_received = 200;
        o.peer_last_received = vec![200, 200, 0];
        g.step_online(Input::default());
        assert_eq!(g.screen, 17);
        assert!(g.online.as_ref().unwrap().rollback.error.is_some());
    }
}
