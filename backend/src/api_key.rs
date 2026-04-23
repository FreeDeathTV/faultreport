use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use hex;

pub fn generate_api_key() -> String {
    let mut rng = OsRng;
    let mut bytes = [0u8; 24]; // 192 bits
    rng.fill_bytes(&mut bytes);
    let base62 = encode_base62(&bytes);
    format!("frp_{}", &base62[..32])
}

pub fn hash_api_key(key: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

fn encode_base62(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut num = 0u128;
    for &byte in bytes.iter().take(16) {
        num = num * 256 + (byte as u128);
    }
    let mut result = String::new();
    while num > 0 {
        result.push(ALPHABET[(num % 62) as usize] as char);
        num /= 62;
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_format() {
        let key = generate_api_key();
        assert!(key.starts_with("frp_"));
        assert_eq!(key.len(), 36);
    }

    #[test]
    fn test_hash_deterministic() {
        let key = "testkey";
        let salt = "testsalt";
        let hash1 = hash_api_key(key, salt);
        let hash2 = hash_api_key(key, salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_unique_keys() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }
}

