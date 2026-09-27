use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hk_proto::*;
use rand::{rngs::OsRng, RngCore};
pub fn new_secret() -> [u8; 32] {
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    seed
}
pub fn public(secret: &[u8; 32]) -> [u8; 32] {
    SigningKey::from_bytes(secret).verifying_key().to_bytes()
}
pub fn sign(secret: &[u8; 32], network: &str, kind: &str, data: &[u8]) -> Vec<u8> {
    SigningKey::from_bytes(secret)
        .sign(&domain(network, kind, data))
        .to_bytes()
        .to_vec()
}
pub fn verify(key: &[u8; 32], network: &str, kind: &str, data: &[u8], sig: &[u8]) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(key) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(sig) else {
        return false;
    };
    key.verify_strict(&domain(network, kind, data), &sig)
        .is_ok()
}
pub fn mnemonic(secret: &[u8; 32]) -> String {
    bip39::Mnemonic::from_entropy(secret).unwrap().to_string()
}
pub fn restore(phrase: &str) -> Result<[u8; 32], String> {
    let words = bip39::Mnemonic::parse(phrase)
        .map_err(|_| "Les 24 mots ou leur contrôle sont incorrects.")?;
    words
        .to_entropy()
        .try_into()
        .map_err(|_| "24 mots sont nécessaires.".into())
}
pub fn split(secret: &[u8; 32]) -> Vec<Vec<u8>> {
    sharks::Sharks(3)
        .dealer(secret)
        .take(5)
        .map(|s| Vec::from(&s))
        .collect()
}
pub fn recover(shares: &[Vec<u8>]) -> Result<[u8; 32], String> {
    let pieces: Result<Vec<sharks::Share>, _> = shares
        .iter()
        .map(|s| sharks::Share::try_from(s.as_slice()))
        .collect();
    let bytes = sharks::Sharks(3)
        .recover(&pieces.map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    bytes.try_into().map_err(|_| "Longueur invalide".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signatures_are_separated() {
        let s = [9; 32];
        let sig = sign(&s, "dev", "event", b"hello");
        assert!(verify(&public(&s), "dev", "event", b"hello", &sig));
        assert!(!verify(&public(&s), "prod", "event", b"hello", &sig));
        assert!(!verify(&public(&s), "dev", "seal", b"hello", &sig));
        assert!(!verify(&public(&s), "dev", "event", b"x", &sig));
    }
    #[test]
    fn twenty_four_words() {
        let s = new_secret();
        assert_eq!(mnemonic(&s).split_whitespace().count(), 24);
        assert_eq!(restore(&mnemonic(&s)).unwrap(), s);
        assert!(restore("hello").is_err());
    }
    #[test]
    fn all_three_of_five() {
        let s = new_secret();
        let shares = split(&s);
        for a in 0..3 {
            for b in a + 1..4 {
                for c in b + 1..5 {
                    assert_eq!(
                        recover(&[shares[a].clone(), shares[b].clone(), shares[c].clone()])
                            .unwrap(),
                        s
                    );
                }
            }
        }
        assert!(recover(&shares[..2]).is_err());
    }
}
pub mod identity;
