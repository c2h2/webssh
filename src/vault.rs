/// AES-256-GCM vault with PBKDF2 key derivation.
/// vault.enc = 12-byte nonce || ciphertext+tag
/// vault.salt = 16 random bytes

use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::RngCore;
use std::path::Path;

const SALT_FILE: &str = "data/vault.salt";
const VAULT_FILE: &str = "data/vault.enc";
const ITER: u32 = 480_000;

pub fn exists() -> bool {
    Path::new(VAULT_FILE).exists() && Path::new(SALT_FILE).exists()
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, ITER, &mut key);
    key
}

pub fn init(password: &str) -> anyhow::Result<()> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    std::fs::write(SALT_FILE, &salt)?;
    let data = serde_json::to_vec(&serde_json::json!({}))?;
    save_raw(password, &salt, &data)?;
    Ok(())
}

fn save_raw(password: &str, salt: &[u8], plaintext: &[u8]) -> anyhow::Result<()> {
    let key_bytes = derive_key(password, salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encrypt: {e}"))?;
    let mut out = nonce_bytes.to_vec();
    out.extend(ciphertext);
    std::fs::write(VAULT_FILE, &out)?;
    Ok(())
}

pub fn unlock(password: &str) -> anyhow::Result<serde_json::Value> {
    let salt = std::fs::read(SALT_FILE)?;
    let blob = std::fs::read(VAULT_FILE)?;
    if blob.len() < 12 {
        anyhow::bail!("vault corrupt");
    }
    let (nonce_bytes, ct) = blob.split_at(12);
    let key_bytes = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ct)
        .map_err(|_| anyhow::anyhow!("wrong password"))?;
    Ok(serde_json::from_slice(&plaintext)?)
}

pub fn save(password: &str, data: &serde_json::Value) -> anyhow::Result<()> {
    let salt = std::fs::read(SALT_FILE)?;
    let bytes = serde_json::to_vec(data)?;
    save_raw(password, &salt, &bytes)
}
