/// User authentication: register, login, session tokens, settings.
///
/// All user records are stored in SQLite via the shared Db handle.
/// The server generates a random master signing key in data/server.key on first run.
/// Session tokens are signed with HMAC-SHA256: base64(username:timestamp):signature

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{Db, UserRecord};

const SERVER_KEY_FILE: &str = "data/server.key";
const TOKEN_TTL_SECS: u64   = 60 * 60 * 24 * 30; // 30 days

// ── Server signing key ────────────────────────────────────────────────────────

pub fn load_or_create_server_key() -> Vec<u8> {
    if Path::new(SERVER_KEY_FILE).exists() {
        if let Ok(h) = std::fs::read_to_string(SERVER_KEY_FILE) {
            if let Ok(b) = hex::decode(h.trim()) {
                if b.len() == 32 { return b; }
            }
        }
    }
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    std::fs::write(SERVER_KEY_FILE, hex::encode(&key)).ok();
    key.to_vec()
}

// ── Password hashing ──────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash: {e}"))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else { return false; };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

// ── Session tokens ────────────────────────────────────────────────────────────

fn sign(msg: &str, key: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key");
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn create_token(username: &str, server_key: &[u8]) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let payload = B64.encode(format!("{username}:{ts}"));
    let sig = sign(&payload, server_key);
    format!("{payload}.{sig}")
}

pub fn verify_token(token: &str, server_key: &[u8]) -> Option<String> {
    let (payload, sig) = token.split_once('.')?;
    if sign(payload, server_key) != sig { return None; }
    let decoded = B64.decode(payload).ok()?;
    let s = String::from_utf8(decoded).ok()?;
    let (username, ts_str) = s.split_once(':')?;
    let ts: u64 = ts_str.parse().ok()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.saturating_sub(ts) > TOKEN_TTL_SECS { return None; }
    Some(username.to_string())
}

// ── Public API (all take &Db) ─────────────────────────────────────────────────

pub fn register(db: &Db, username: &str, password: &str) -> anyhow::Result<()> {
    if db.get_user(username).is_some() {
        anyhow::bail!("Username already taken");
    }
    let is_admin = db.user_count() == 0; // first user becomes admin
    let hash = hash_password(password)?;
    db.upsert_user(&UserRecord {
        username:      username.to_string(),
        password_hash: hash,
        vault_key_hex: String::new(),
        is_admin,
        is_disabled: false,
    })
}

pub fn login(db: &Db, username: &str, password: &str) -> anyhow::Result<()> {
    let user = db.get_user(username)
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;
    if !verify_password(password, &user.password_hash) {
        anyhow::bail!("Invalid credentials");
    }
    if user.is_disabled {
        anyhow::bail!("Account disabled");
    }
    Ok(())
}

pub fn set_vault_key(db: &Db, username: &str, key_hex: &str) -> anyhow::Result<()> {
    if !key_hex.is_empty() {
        let bytes = hex::decode(key_hex).map_err(|_| anyhow::anyhow!("Invalid hex key"))?;
        if bytes.len() != 32 { anyhow::bail!("Key must be 32 bytes (64 hex chars)"); }
    }
    db.set_vault_key(username, key_hex)
}

