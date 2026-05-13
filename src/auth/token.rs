use sha2::{Digest, Sha256};

/// Generate a random 32-byte token and return (`raw_bytes`, `hex_hash`).
pub fn generate() -> ([u8; 32], String) {
    let raw: [u8; 32] = rand::random();
    let hash = hex::encode(Sha256::digest(raw));
    (raw, hash)
}

/// Hash raw token bytes with SHA-256 and return the hex-encoded hash.
pub fn hash(raw: &[u8]) -> String {
    hex::encode(Sha256::digest(raw))
}
