use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::db::{Db, HostRecord, KeyRecord};
use crate::pty_session::PtySession;

// ── API-facing types (JSON serialization shapes) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEntry {
    pub id:           String,
    pub label:        String,
    pub hostname:     String,
    pub port:         u16,
    pub username:     String,
    #[serde(default)] pub key_id:      String,
    #[serde(default)] pub key_path:    String,
    #[serde(default)] pub jump_host:   String,
    #[serde(default)] pub ssh_command: String,
    #[serde(default)] pub theme:       String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub id:   String,
    pub name: String,
    pub path: String,
}

// ── Conversions ─────────────────────────────────────────────────────────────

impl From<HostRecord> for HostEntry {
    fn from(r: HostRecord) -> Self {
        Self {
            id:          r.id,
            label:       r.label,
            hostname:    r.hostname,
            port:        r.port,
            username:    r.ssh_username,
            key_id:      r.key_id,
            key_path:    r.key_path,
            jump_host:   r.jump_host,
            ssh_command: r.ssh_command,
            theme:       r.theme,
        }
    }
}

impl From<KeyRecord> for KeyEntry {
    fn from(r: KeyRecord) -> Self {
        Self { id: r.id, name: r.name, path: r.path }
    }
}

// ── AppState ─────────────────────────────────────────────────────────────────

/// Shared live PTY handle — kept alive across browser refresh/disconnect.
/// The WsSink for broadcasting output is swapped when a new WS reattaches.
pub struct LiveSession {
    pub pty: Arc<Mutex<Option<PtySession>>>,
    /// Channel to send a new sink to the background reader task when a client reconnects.
    pub sink_tx: tokio::sync::watch::Sender<Option<WsSinkSender>>,
}

/// Type-erased sender that the background task uses to forward PTY output to the WS sink.
pub type WsSinkSender = Arc<dyn Fn(String) + Send + Sync + 'static>;

pub struct AppState {
    pub server_key: Vec<u8>,
    pub db: Db,
    /// Vault passwords cached per username for the duration of the server session.
    pub vault_passwords: HashMap<String, String>,
    /// Live PTY sessions keyed by session_id — survive browser refresh.
    pub live_sessions: HashMap<String, LiveSession>,
}

impl AppState {
    pub fn new(server_key: Vec<u8>, db: Db) -> Self {
        Self { server_key, db, vault_passwords: HashMap::new(), live_sessions: HashMap::new() }
    }
}

