use super::*;
/// Complete co-signed proof; verification replays every tick before exposing playback.
pub struct Replay {
    pub battle: Battle,
    pub initial: Battle,
    pub final_tick: u32,
    pub hash: Hash,
    pub players: usize,
    pub inputs: BTreeMap<(u32, u8), Input>,
    pub reactions: Vec<Vec<u32>>,
}
impl Replay {
    pub fn verify(text: &str) -> Result<Self, String> {
        let mut v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        expand_proof(&mut v)?;
        let start: Packet =
            serde_json::from_value(v["start"].clone()).map_err(|e| e.to_string())?;
        if !start.valid() {
            return Err("Départ non signé".into());
        }
        let (cell, players, nonce, config) = match &start.message {
            Message::Start {
                cell,
                players,
                nonce,
            } => (*cell, players.clone(), *nonce, None),
            Message::StartV2 {
                cell,
                players,
                nonce,
                mode,
                codex,
                walls,
            } => (
                *cell,
                players.clone(),
                *nonce,
                Some((*mode, codex.clone(), *walls)),
            ),
            _ => return Err("Départ absent".into()),
        };
        if players.first().map(|p| p.key) != Some(start.author) {
            return Err("Organisateur absent".into());
        }
        let mut rb = Rollback::new(cell, players.clone(), nonce, start.author)?;
        if let Some((mode, codex, walls)) = config {
            rb.configure(mode, codex, walls)?;
        }
        let initial = rb.battle.clone();
        let tick = v["tick"].as_u64().ok_or("Tick absent")?;
        if tick == 0 || tick > 5401 {
            return Err("Durée invalide".into());
        }
        let final_tick = tick as u32;
        let mut inputs = BTreeMap::new();
        let mut links = BTreeMap::new();
        for t in 1..=2 {
            for id in 0..players.len() {
                inputs.insert((t, id as u8), Input::default());
            }
        }
        let packets: Vec<Packet> =
            serde_json::from_value(v["inputs"].clone()).map_err(|e| e.to_string())?;
        if packets.len() > 60000 {
            return Err("Trop d'entrées".into());
        }
        let mut verifier = CertificateVerifier::default();
        for p in packets {
            if !verifier.valid(&p) {
                return Err("Signature d'entrée invalide".into());
            }
            let id = players
                .iter()
                .position(|v| v.key == p.author)
                .ok_or("Auteur étranger")? as u8;
            if let Message::Input {
                room,
                tick,
                input,
                previous,
            } = p.message
            {
                if room != rb.room {
                    return Err("Champ étranger".into());
                }
                links.insert((tick, id), previous);
                if let Some(prev) = inputs.insert((tick, id), input) {
                    if prev != input {
                        return Err("Équivoque d'entrée".into());
                    }
                }
            }
        }
        let mut previous_hash = vec![[0; 32]; players.len()];
        let mut reactions = vec![vec![]; players.len()];
        for t in 1..=final_tick {
            let mut values = vec![];
            for id in 0..players.len() {
                let input = *inputs
                    .get(&(t, id as u8))
                    .ok_or(format!("Entrée manquante {t}/{id}"))?;
                if t >= 3 {
                    let previous = *links.get(&(t, id as u8)).ok_or("Lien manquant")?;
                    if previous != previous_hash[id] {
                        return Err("Chaîne d'entrées rompue".into());
                    }
                    previous_hash[id] = input_hash(rb.room, players[id].key, t, input, previous);
                }
                if input.shoot
                    && !inputs
                        .get(&(t.saturating_sub(1), id as u8))
                        .is_some_and(|i| i.shoot)
                {
                    reactions[id].push(t);
                }
                values.push((id as u8, input));
            }
            rb.battle.step(&values);
        }
        let hash = rb.battle.hash();
        if v["hash"].as_str() != Some(&hex(&hash)) {
            return Err("Hash de rejeu divergent".into());
        }
        let results: Vec<Packet> =
            serde_json::from_value(v["results"].clone()).map_err(|e| e.to_string())?;
        let mut signed = std::collections::BTreeSet::new();
        for p in results {
            if !p.valid() {
                return Err("Résultat invalide".into());
            }
            if let Message::Finish {
                room,
                tick,
                hash: h,
            } = p.message
            {
                if room == rb.room
                    && tick == final_tick
                    && h == hash
                    && players.iter().any(|v| v.key == p.author)
                {
                    signed.insert(p.author);
                }
            }
        }
        if signed.len() != players.len() {
            return Err("Preuve incomplète : signatures absentes".into());
        }
        Ok(Self {
            battle: initial.clone(),
            initial,
            final_tick,
            hash,
            players: players.len(),
            inputs,
            reactions,
        })
    }
    pub fn step(&mut self) {
        if self.battle.tick >= self.final_tick {
            return;
        }
        let t = self.battle.tick + 1;
        let values: Vec<_> = (0..self.players)
            .map(|id| (id as u8, self.inputs[&(t, id as u8)]))
            .collect();
        self.battle.step(&values);
    }
    pub fn rewind(&mut self) {
        self.battle = self.initial.clone();
    }
}

/// Share identical certificate chains once per proof, without changing signed bytes.
pub fn compact_proof(mut proof: serde_json::Value) -> serde_json::Value {
    let mut certs = serde_json::Map::new();
    fn packet(p: &mut serde_json::Value, certs: &mut serde_json::Map<String, serde_json::Value>) {
        if p["certificate"].is_object() {
            let cert = p["certificate"].take();
            let id = hex(blake3::hash(cert.to_string().as_bytes()).as_bytes());
            certs.entry(id.clone()).or_insert(cert);
            p["certificate_ref"] = id.into();
        }
    }
    packet(&mut proof["start"], &mut certs);
    for field in ["inputs", "results"] {
        if let Some(packets) = proof[field].as_array_mut() {
            for p in packets {
                packet(p, &mut certs);
            }
        }
    }
    proof["certificates"] = certs.into();
    proof["version"] = 2.into();
    proof
}
fn expand_proof(proof: &mut serde_json::Value) -> Result<(), String> {
    let certs = proof["certificates"].clone();
    fn packet(p: &mut serde_json::Value, certs: &serde_json::Value) -> Result<(), String> {
        if let Some(id) = p["certificate_ref"].as_str() {
            let cert = certs[id].clone();
            if !cert.is_object() || hex(blake3::hash(cert.to_string().as_bytes()).as_bytes()) != id
            {
                return Err("Certificat de preuve manquant".into());
            }
            p["certificate"] = cert;
        }
        Ok(())
    }
    packet(&mut proof["start"], &certs)?;
    for field in ["inputs", "results"] {
        if let Some(packets) = proof[field].as_array_mut() {
            for p in packets {
                packet(p, &certs)?;
            }
        }
    }
    Ok(())
}
