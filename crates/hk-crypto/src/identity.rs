//! Nom -> P-256 device -> ephemeral Ed25519 session. DEV software devices are explicit.
use crate::{new_secret, public, sign, verify};
use borsh::{BorshDeserialize, BorshSerialize};
use hk_proto::{domain, Hash};
use p256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    pkcs8::DecodePublicKey,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Device {
    pub nom: Hash,
    pub public: Vec<u8>,
    pub attestation: Vec<Vec<u8>>,
    pub software_dev: bool,
    pub signature: Vec<u8>,
}
impl Device {
    pub fn payload(&self) -> Vec<u8> {
        borsh::to_vec(&(self.nom, &self.public, &self.attestation, self.software_dev)).unwrap()
    }
    pub fn bind(
        nom: &Hash,
        public: Vec<u8>,
        attestation: Vec<Vec<u8>>,
        software_dev: bool,
    ) -> Self {
        let mut d = Self {
            nom: crate::public(nom),
            public,
            attestation,
            software_dev,
            signature: vec![],
        };
        d.signature = sign(nom, "dev", "device", &d.payload());
        d
    }
    pub fn valid(&self) -> bool {
        self.public.len() <= 512
            && self.attestation.len() <= 8
            && verify(&self.nom, "dev", "device", &self.payload(), &self.signature)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Certificate {
    pub device: Device,
    pub session: Hash,
    pub issued: u64,
    pub expires: u64,
    pub signature: Vec<u8>,
}
impl Certificate {
    pub fn payload(&self) -> Vec<u8> {
        domain(
            "dev",
            "session",
            &borsh::to_vec(&(
                self.device.nom,
                &self.device.public,
                self.session,
                self.issued,
                self.expires,
            ))
            .unwrap(),
        )
    }
    pub fn valid(&self, time: u64) -> bool {
        if !self.device.valid()
            || self.expires <= self.issued
            || self.expires - self.issued > 86400
            || time < self.issued
            || time > self.expires
        {
            return false;
        }
        let key = VerifyingKey::from_sec1_bytes(&self.device.public)
            .or_else(|_| VerifyingKey::from_public_key_der(&self.device.public));
        let signature = Signature::from_der(&self.signature)
            .or_else(|_| Signature::from_slice(&self.signature));
        matches!((key,signature),(Ok(k),Ok(s)) if k.verify(&self.payload(),&s).is_ok())
    }
}
#[derive(Clone)]
pub struct Session {
    pub secret: Hash,
    pub certificate: Certificate,
}
impl Session {
    pub fn development(nom: &Hash, now: u64) -> Self {
        let key = SigningKey::random(&mut rand::rngs::OsRng);
        let device = Device::bind(
            nom,
            key.verifying_key()
                .to_encoded_point(false)
                .as_bytes()
                .to_vec(),
            vec![],
            true,
        );
        let secret = new_secret();
        let mut certificate = Certificate {
            device,
            session: public(&secret),
            issued: now,
            expires: now.saturating_add(86400),
            signature: vec![],
        };
        let signature: Signature = key.sign(&certificate.payload());
        certificate.signature = signature.to_der().as_bytes().to_vec();
        Self {
            secret,
            certificate,
        }
    }
    pub fn beacon(&self, epoch: u64) -> [u8; 16] {
        beacon(&self.certificate.session, epoch)
    }
}
pub fn beacon(session: &Hash, epoch: u64) -> [u8; 16] {
    let mut p = session.to_vec();
    p.extend_from_slice(&(epoch / 600).to_le_bytes());
    blake3::hash(&p).as_bytes()[..16].try_into().unwrap()
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Witness {
    pub observer: Hash,
    pub subject: Hash,
    pub token: [u8; 16],
    pub time: u64,
    pub cell: u64,
    pub signature: Vec<u8>,
}
impl Witness {
    pub fn new(secret: &Hash, subject: Hash, token: [u8; 16], time: u64, cell: u64) -> Self {
        let mut w = Self {
            observer: public(secret),
            subject,
            token,
            time,
            cell,
            signature: vec![],
        };
        w.signature = sign(secret, "dev", "witness", &w.payload());
        w
    }
    fn payload(&self) -> Vec<u8> {
        borsh::to_vec(&(
            self.observer,
            self.subject,
            self.token,
            self.time,
            self.cell,
        ))
        .unwrap()
    }
    pub fn valid(&self) -> bool {
        self.subject != self.observer
            && verify(
                &self.observer,
                "dev",
                "witness",
                &self.payload(),
                &self.signature,
            )
    }
    pub fn mutual(a: &Self, b: &Self, now: u64) -> bool {
        a.valid()
            && b.valid()
            && a.observer == b.subject
            && a.subject == b.observer
            && a.cell == b.cell
            && now.abs_diff(a.time) <= 120
            && now.abs_diff(b.time) <= 120
    }
}
#[derive(Serialize, Deserialize)]
pub struct SealedShare {
    pub ephemeral: Hash,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}
pub fn seal_share(recipient: Hash, share: &[u8]) -> Result<SealedShare, String> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305,
    };
    let sk = x25519_dalek::StaticSecret::from(new_secret());
    let ephemeral = x25519_dalek::PublicKey::from(&sk).to_bytes();
    let shared = sk.diffie_hellman(&recipient.into());
    if !shared.was_contributory() {
        return Err("Dépositaire invalide".into());
    }
    let key = blake3::derive_key("HEXKEEP/v1/dev/recovery", shared.as_bytes());
    let nonce: [u8; 12] = new_secret()[..12].try_into().unwrap();
    let ciphertext = ChaCha20Poly1305::new((&key).into())
        .encrypt((&nonce).into(), share)
        .map_err(|_| "Chiffrement")?;
    Ok(SealedShare {
        ephemeral,
        nonce,
        ciphertext,
    })
}
pub fn open_share(secret: Hash, share: &SealedShare) -> Result<Vec<u8>, String> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305,
    };
    let shared = x25519_dalek::StaticSecret::from(secret).diffie_hellman(&share.ephemeral.into());
    if !shared.was_contributory() {
        return Err("Part invalide".into());
    }
    let key = blake3::derive_key("HEXKEEP/v1/dev/recovery", shared.as_bytes());
    ChaCha20Poly1305::new((&key).into())
        .decrypt((&share.nonce).into(), share.ciphertext.as_ref())
        .map_err(|_| "Part non authentique".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hierarchy_expiry_tamper() {
        let mut s = Session::development(&[7; 32], 100);
        assert!(s.certificate.valid(101));
        assert!(!s.certificate.valid(86501));
        s.certificate.session[0] ^= 1;
        assert!(!s.certificate.valid(101));
    }
    #[test]
    fn encrypted_recovery() {
        let secret = [4; 32];
        let recipient =
            x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(secret)).to_bytes();
        let pieces = crate::split(&[6; 32]);
        let encrypted: Vec<_> = pieces
            .iter()
            .map(|p| seal_share(recipient, p).unwrap())
            .collect();
        let recovered: Vec<_> = encrypted[..3]
            .iter()
            .map(|s| open_share(secret, s).unwrap())
            .collect();
        assert_eq!(crate::recover(&recovered).unwrap(), [6; 32]);
        assert!(open_share([5; 32], &encrypted[0]).is_err());
    }
    #[test]
    fn beacon_rotates() {
        let s = Session::development(&[3; 32], 0);
        assert_eq!(s.beacon(1), s.beacon(599));
        assert_ne!(s.beacon(599), s.beacon(600));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RecoveryShare {
    pub owner: Hash,
    pub recipient: Hash,
    pub created: u64,
    pub ephemeral: Hash,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub signature: Vec<u8>,
}
impl RecoveryShare {
    fn payload(&self) -> Vec<u8> {
        borsh::to_vec(&(
            self.owner,
            self.recipient,
            self.created,
            self.ephemeral,
            self.nonce,
            &self.ciphertext,
        ))
        .unwrap()
    }
    pub fn new(owner: &Hash, recipient: Hash, piece: &[u8], now: u64) -> Result<Self, String> {
        let ed = ed25519_dalek::VerifyingKey::from_bytes(&recipient)
            .map_err(|_| "Destinataire invalide")?;
        let encrypted = seal_share(ed.to_montgomery().to_bytes(), piece)?;
        let mut s = Self {
            owner: public(owner),
            recipient,
            created: now,
            ephemeral: encrypted.ephemeral,
            nonce: encrypted.nonce,
            ciphertext: encrypted.ciphertext,
            signature: vec![],
        };
        s.signature = sign(owner, "dev", "recovery-share", &s.payload());
        Ok(s)
    }
    pub fn valid(&self) -> bool {
        self.ciphertext.len() < 1024
            && verify(
                &self.owner,
                "dev",
                "recovery-share",
                &self.payload(),
                &self.signature,
            )
    }
    pub fn open(&self, recipient: &Hash) -> Result<Vec<u8>, String> {
        if public(recipient) != self.recipient || !self.valid() {
            return Err("Part destinée à un autre Nom ou altérée".into());
        }
        let secret = ed25519_dalek::SigningKey::from_bytes(recipient).to_scalar_bytes();
        open_share(
            secret,
            &SealedShare {
                ephemeral: self.ephemeral,
                nonce: self.nonce,
                ciphertext: self.ciphertext.clone(),
            },
        )
    }
}
#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn five_guardians_three_recovers_owner() {
        let owner = [42; 32];
        let parts = crate::split(&owner);
        let guardians: Vec<_> = (1..=5).map(|i| [i; 32]).collect();
        let encrypted: Vec<_> = parts
            .iter()
            .zip(&guardians)
            .map(|(p, g)| RecoveryShare::new(&owner, public(g), p, 100).unwrap())
            .collect();
        let opened: Vec<_> = [0, 2, 4]
            .iter()
            .map(|i| encrypted[*i].open(&guardians[*i]).unwrap())
            .collect();
        assert_eq!(crate::recover(&opened).unwrap(), owner);
        assert!(encrypted[0].open(&guardians[1]).is_err());
        let mut tampered = encrypted[0].clone();
        tampered.recipient = public(&guardians[1]);
        assert!(!tampered.valid());
    }
}
