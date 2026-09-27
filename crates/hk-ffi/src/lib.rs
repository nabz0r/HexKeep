use std::sync::{Arc, Mutex};
uniffi::setup_scaffolding!();
#[derive(uniffi::Object)]
pub struct Engine {
    game: Mutex<hk_core::Game>,
    audio: Mutex<hk_apu::Apu>,
}
#[uniffi::export]
impl Engine {
    #[uniffi::constructor]
    pub fn new(snapshot: String, dev: bool) -> Arc<Self> {
        Arc::new(Self {
            game: Mutex::new(hk_core::Game::new(&snapshot, dev)),
            audio: Mutex::new(hk_apu::Apu::default()),
        })
    }
    pub fn presentation(&self, width: i32) -> String {
        self.game.lock().unwrap().presentation(width)
    }
    pub fn ui_action(&self, action: String) {
        self.game.lock().unwrap().ui_action(&action);
    }
    pub fn hero(&self, realm: u8, role: u8) {
        self.game.lock().unwrap().hero(realm, role);
    }
    pub fn controls(
        &self,
        mx: i16,
        my: i16,
        ax: i16,
        ay: i16,
        auto: bool,
        dash: bool,
        skill: bool,
    ) {
        self.game
            .lock()
            .unwrap()
            .controls(mx, my, ax, ay, auto, dash, skill);
    }
    pub fn frame(&self, width: i32) -> Vec<u8> {
        self.game.lock().unwrap().frame(width)
    }
    pub fn tick(&self, time: u64) {
        self.game.lock().unwrap().tick(time);
    }
    pub fn touch(&self, id: i32, phase: u8, x: i32, y: i32) {
        self.game.lock().unwrap().touch(id, phase, x, y);
    }
    pub fn snapshot(&self) -> String {
        self.game.lock().unwrap().snapshot()
    }
    pub fn dirty(&self) -> bool {
        let mut g = self.game.lock().unwrap();
        let d = g.dirty;
        g.dirty = false;
        d && !g.storage_error
    }
    pub fn action(&self) -> u8 {
        let mut g = self.game.lock().unwrap();
        let a = g.native_action;
        g.native_action = 0;
        a
    }
    pub fn sensitive(&self) -> bool {
        self.game.lock().unwrap().sensitive()
    }
    pub fn haptic(&self) -> u8 {
        let mut g = self.game.lock().unwrap();
        let h = g.haptic;
        g.haptic = 0;
        h
    }
    pub fn location(&self, lat: i32, lng: i32, mock: bool) {
        self.game.lock().unwrap().gps(lat, lng, mock);
    }
    pub fn notice(&self, message: String) {
        self.game.lock().unwrap().toast(&message);
    }
    pub fn audio(&self, count: u32) -> Vec<u8> {
        let (enabled, effects, effect, theme) = {
            let mut g = self.game.lock().unwrap();
            let theme = if g.screen == 0 {
                0
            } else if g.screen == 27 {
                6
            } else if g.battle.as_ref().is_some_and(|b| b.siege.is_some()) {
                5
            } else if g.screen == 6 {
                4
            } else {
                g.save.realm.index() as u8 + 1
            };
            let pair = (g.save.settings.music, g.save.sound_effects, g.sound, theme);
            g.sound = 0;
            pair
        };
        let mut a = self.audio.lock().unwrap();
        a.enabled = enabled;
        a.effects_enabled = effects;
        a.theme = theme;
        if effect > 0 {
            a.trigger(effect);
        }
        a.samples(count.min(4096) as usize)
            .into_iter()
            .flat_map(|s| s.to_le_bytes())
            .collect()
    }
    pub fn set_storage_error(&self) {
        self.game.lock().unwrap().storage_error = true;
    }
    pub fn back(&self) {
        let mut g = self.game.lock().unwrap();
        if matches!(g.screen, 40 | 42) {
            g.ui_action("inventory_back");
        } else if g.screen == 6 {
            g.screen = 14;
            g.touches.clear();
        } else if g.save.created {
            g.ui_action("home");
        } else {
            g.screen = 0;
        }
    }
    pub fn debug_status(&self) -> String {
        let g = self.game.lock().unwrap();
        format!(
            "screen={} ticks={} cell={:x} events={} error={}",
            g.screen,
            g.ticks,
            g.save.world.current,
            g.save.ledger.events.len(),
            g.storage_error
        )
    }
    pub fn connect(&self, address: String) {
        self.game.lock().unwrap().connect(address);
    }
    pub fn network_addresses(&self) -> Vec<String> {
        self.game.lock().unwrap().addresses.clone()
    }
    pub fn network_peers(&self) -> u32 {
        self.game.lock().unwrap().peers.len() as u32
    }
    pub fn proof(&self) -> String {
        self.game.lock().unwrap().last_proof.clone()
    }
    pub fn determinism_check(&self) -> String {
        hk_proto::hex(&hk_sim::determinism_suite())
    }
    pub fn pause_network(&self) {
        let mut g = self.game.lock().unwrap();
        if g.online.is_some() {
            g.finish_online();
            g.screen = 17;
        }
        g.node = None;
        g.peers.clear();
        g.peer_count = 0;
        g.addresses.clear();
    }
    pub fn identity_public(&self) -> Vec<u8> {
        hk_crypto::public(&self.game.lock().unwrap().save.secret).to_vec()
    }
    pub fn identity_challenge(&self) -> Vec<u8> {
        blake3::hash(&self.identity_public()).as_bytes().to_vec()
    }
    pub fn session_request(&self, public: Vec<u8>, attestation: String) -> Vec<u8> {
        let mut g = self.game.lock().unwrap();
        let certs: Vec<Vec<u8>> = serde_json::from_str(&attestation).unwrap_or_default();
        let device = hk_crypto::identity::Device::bind(&g.save.secret, public, certs, false);
        g.session.certificate.device = device;
        g.session.certificate.issued = hk_net_time();
        g.session.certificate.expires = hk_net_time() + 86400;
        g.session.certificate.signature.clear();
        g.session.certificate.payload()
    }
    pub fn session_certify(&self, signature: Vec<u8>) -> bool {
        let mut g = self.game.lock().unwrap();
        g.session.certificate.signature = signature;
        let valid = g.session.certificate.valid(hk_net_time());
        if valid {
            g.expansion.device_status = "Clé P-256 Keystore liée au Nom".into();
            g.node = None;
        } else {
            g.expansion.device_status = "Certificat d'appareil refusé".into();
        }
        valid
    }
    pub fn throne_unlock(&self) {
        self.game.lock().unwrap().throne_unlock();
    }
    pub fn import_exchange(&self, data: String) -> String {
        let mut g = self.game.lock().unwrap();
        match g.import_exchange(&data) {
            Ok(()) => {
                g.toast("Dossier importé et vérifié.");
                String::new()
            }
            Err(e) => {
                g.toast(&e);
                e
            }
        }
    }
    pub fn beacon(&self) -> Vec<u8> {
        let g = self.game.lock().unwrap();
        let mut v = g.session.beacon(hk_net_time()).to_vec();
        v.push(g.save.realm.index() as u8);
        v
    }
    pub fn ble_observed(&self, bytes: Vec<u8>) {
        self.game.lock().unwrap().ble_observed(bytes);
    }
    pub fn ble_status(&self, message: String) {
        self.game.lock().unwrap().expansion.bluetooth_status = message;
    }
    pub fn memory_speed(&self, speed: u32) {
        let mut g = self.game.lock().unwrap();
        let cell = g.save.world.current;
        g.memory_location(cell, speed);
    }
    pub fn phare(&self) -> bool {
        self.game.lock().unwrap().save.expansion.phare
    }
    pub fn stop_phare(&self, message: String) {
        let mut g = self.game.lock().unwrap();
        g.save.expansion.phare = false;
        g.dirty = true;
        g.toast(&message);
    }
    pub fn native_text(&self) -> String {
        self.game.lock().unwrap().expansion.native_text.clone()
    }
    pub fn reserve_relay(&self, address: String) {
        let mut g = self.game.lock().unwrap();
        g.start_network();
        if let Some(n) = &g.node {
            n.reserve(address);
        }
    }
    pub fn network_report(&self) -> String {
        let g = self.game.lock().unwrap();
        if let Some(o) = &g.online {
            let tick = o.rollback.confirmed() / 30 * 30;
            let hash = o
                .rollback
                .states
                .get(&tick)
                .map(|s| hk_proto::hex(&s.hash()))
                .unwrap_or_default();
            serde_json::json!({"tick":tick,"hash":hash,"participants":o.rollback.players.len(),"error":o.rollback.error}).to_string()
        } else {
            serde_json::json!({"tick":0,"hash":"","participants":g.peers.len()+1,"error":null})
                .to_string()
        }
    }
}

fn hk_net_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
