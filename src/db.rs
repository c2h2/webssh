/// Central database layer.
///
/// All persistent data — users, hosts, keys, sessions, scrollback — is stored
/// in SQLite.  Redis is an optional write-through cache for scrollback replay.
///
/// The public `Db` struct is cheap to clone (Arc-backed) and can be passed
/// across threads freely.
///
/// ─── Schema overview ───────────────────────────────────────────────────────
/// users(username, password_hash, vault_key_hex)
/// hosts(id, username, label, hostname, port, ssh_username, key_id, key_path,
///       jump_host, ssh_command, theme)
/// keys(id, username, name, path)
/// sessions(id, username, session_type, host_id, label, theme, updated_at)
/// scrollback(id AUTOINCREMENT, session_id, chunk, seq)
///   • capped at MAX_SCROLLBACK_LINES rows per session (oldest pruned)

use rusqlite::{params, Connection};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const MAX_SCROLLBACK_LINES: i64 = 1_000_000;

// ── Domain types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub username:      String,
    pub password_hash: String,
    /// Optional hex-encoded 32-byte vault key override (empty = server default).
    #[serde(default)]
    pub vault_key_hex: String,
    #[serde(default)]
    pub is_admin:    bool,
    #[serde(default)]
    pub is_disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRecord {
    pub id:          String,
    pub username:    String, // owner
    pub label:       String,
    pub hostname:    String,
    pub port:        u16,
    pub ssh_username: String,
    #[serde(default)] pub key_id:      String,
    #[serde(default)] pub key_path:    String,
    #[serde(default)] pub jump_host:   String,
    #[serde(default)] pub ssh_command: String,
    #[serde(default)] pub theme:       String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRecord {
    pub id:       String,
    pub username: String, // owner
    pub name:     String,
    pub path:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id:           String,
    pub username:     String,
    pub session_type: String, // "local" | "ssh" | "mosh"
    pub host_id:      String,
    pub label:        String,
    pub theme:        String,
    #[serde(default = "default_font_size")]
    pub font_size:    i64,
    pub updated_at:   i64,
}

fn default_font_size() -> i64 { 13 }

// ── Db handle ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Db {
    conn:  Arc<Mutex<Connection>>,
    redis: Option<Arc<redis::Client>>,
}

impl Db {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                username      TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL,
                vault_key_hex TEXT NOT NULL DEFAULT '',
                is_admin      INTEGER NOT NULL DEFAULT 0,
                is_disabled   INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS hosts (
                id          TEXT PRIMARY KEY,
                username    TEXT NOT NULL,
                label       TEXT NOT NULL DEFAULT '',
                hostname    TEXT NOT NULL,
                port        INTEGER NOT NULL DEFAULT 22,
                ssh_username TEXT NOT NULL DEFAULT '',
                key_id      TEXT NOT NULL DEFAULT '',
                key_path    TEXT NOT NULL DEFAULT '',
                jump_host   TEXT NOT NULL DEFAULT '',
                ssh_command TEXT NOT NULL DEFAULT '',
                theme       TEXT NOT NULL DEFAULT 'hacker'
            );
            CREATE INDEX IF NOT EXISTS idx_hosts_user ON hosts(username);
            CREATE TABLE IF NOT EXISTS keys (
                id       TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                name     TEXT NOT NULL,
                path     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_keys_user ON keys(username);
            CREATE TABLE IF NOT EXISTS sessions (
                id           TEXT PRIMARY KEY,
                username     TEXT NOT NULL,
                session_type TEXT NOT NULL DEFAULT 'local',
                host_id      TEXT NOT NULL DEFAULT '',
                label        TEXT NOT NULL DEFAULT '',
                theme        TEXT NOT NULL DEFAULT 'hacker',
                font_size    INTEGER NOT NULL DEFAULT 13,
                updated_at   INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(username, updated_at);
            CREATE TABLE IF NOT EXISTS scrollback (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                chunk      TEXT NOT NULL,
                seq        INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_scrollback_session
                ON scrollback(session_id, seq);
            CREATE TABLE IF NOT EXISTS audit_log (
                id     INTEGER PRIMARY KEY AUTOINCREMENT,
                ts     INTEGER NOT NULL,
                actor  TEXT NOT NULL DEFAULT '',
                action TEXT NOT NULL,
                target TEXT NOT NULL DEFAULT '',
                detail TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_log(ts DESC);",
        )?;
        // Migrations for existing databases
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN font_size INTEGER NOT NULL DEFAULT 13;",
        ).ok();
        conn.execute_batch(
            "ALTER TABLE users ADD COLUMN is_admin    INTEGER NOT NULL DEFAULT 0;",
        ).ok();
        conn.execute_batch(
            "ALTER TABLE users ADD COLUMN is_disabled INTEGER NOT NULL DEFAULT 0;",
        ).ok();
        // If no admin exists yet (migration from pre-admin schema), promote the first user.
        conn.execute_batch(
            "UPDATE users SET is_admin=1 WHERE rowid=(SELECT MIN(rowid) FROM users)
             AND (SELECT COUNT(*) FROM users WHERE is_admin=1)=0;",
        ).ok();
        Ok(Self { conn: Arc::new(Mutex::new(conn)), redis: None })
    }

    pub fn with_redis(mut self, url: &str) -> Self {
        match redis::Client::open(url) {
            Ok(c) => { self.redis = Some(Arc::new(c)); }
            Err(e) => { tracing::warn!("Redis unavailable ({}), running without cache", e); }
        }
        self
    }

    // ── Users ─────────────────────────────────────────────────────────────

    pub fn user_count(&self) -> usize {
        self.conn.lock()
            .query_row("SELECT COUNT(*) FROM users", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) as usize
    }

    pub fn get_user(&self, username: &str) -> Option<UserRecord> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT username, password_hash, vault_key_hex, is_admin, is_disabled
             FROM users WHERE username=?1",
            params![username],
            |r| Ok(UserRecord {
                username:      r.get(0)?,
                password_hash: r.get(1)?,
                vault_key_hex: r.get(2)?,
                is_admin:    r.get::<_, i64>(3)? != 0,
                is_disabled: r.get::<_, i64>(4)? != 0,
            }),
        ).ok()
    }

    pub fn upsert_user(&self, rec: &UserRecord) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "INSERT INTO users (username, password_hash, vault_key_hex, is_admin, is_disabled)
             VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(username) DO UPDATE SET
               password_hash = excluded.password_hash,
               vault_key_hex = excluded.vault_key_hex",
            params![rec.username, rec.password_hash, rec.vault_key_hex,
                    rec.is_admin as i64, rec.is_disabled as i64],
        )?;
        Ok(())
    }

    pub fn list_users(&self) -> anyhow::Result<Vec<UserRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT username, password_hash, vault_key_hex, is_admin, is_disabled
             FROM users ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], |r| Ok(UserRecord {
            username:      r.get(0)?,
            password_hash: r.get(1)?,
            vault_key_hex: r.get(2)?,
            is_admin:    r.get::<_, i64>(3)? != 0,
            is_disabled: r.get::<_, i64>(4)? != 0,
        }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn set_user_disabled(&self, username: &str, disabled: bool) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "UPDATE users SET is_disabled=?1 WHERE username=?2",
            params![disabled as i64, username],
        )?;
        Ok(())
    }

    pub fn delete_user(&self, username: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM users WHERE username=?1 AND is_admin=0", params![username])?;
        Ok(())
    }

    // ── Settings ──────────────────────────────────────────────────────────

    pub fn get_setting(&self, key: &str, default: &str) -> String {
        self.conn.lock()
            .query_row("SELECT value FROM settings WHERE key=?1", params![key], |r| r.get(0))
            .unwrap_or_else(|_| default.to_string())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "INSERT INTO settings (key, value) VALUES (?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Audit log ─────────────────────────────────────────────────────────

    pub fn append_audit(&self, actor: &str, action: &str, target: &str, detail: &str) {
        let ts = now_secs() as i64;
        let _ = self.conn.lock().execute(
            "INSERT INTO audit_log (ts, actor, action, target, detail) VALUES (?1,?2,?3,?4,?5)",
            params![ts, actor, action, target, detail],
        );
    }

    pub fn list_audit(&self, limit: i64, offset: i64) -> Vec<serde_json::Value> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT id, ts, actor, action, target, detail
             FROM audit_log ORDER BY ts DESC, id DESC LIMIT ?1 OFFSET ?2",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![limit, offset], |r| {
            Ok(serde_json::json!({
                "id":     r.get::<_, i64>(0)?,
                "ts":     r.get::<_, i64>(1)?,
                "actor":  r.get::<_, String>(2)?,
                "action": r.get::<_, String>(3)?,
                "target": r.get::<_, String>(4)?,
                "detail": r.get::<_, String>(5)?,
            }))
        }).ok()
          .map(|rows| rows.filter_map(|r| r.ok()).collect())
          .unwrap_or_default()
    }

    pub fn audit_count(&self) -> i64 {
        self.conn.lock()
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap_or(0)
    }

    pub fn set_vault_key(&self, username: &str, hex: &str) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "UPDATE users SET vault_key_hex=?1 WHERE username=?2",
            params![hex, username],
        )?;
        Ok(())
    }

    // ── Hosts ─────────────────────────────────────────────────────────────

    pub fn list_hosts(&self, username: &str) -> anyhow::Result<Vec<HostRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, username, label, hostname, port, ssh_username,
                    key_id, key_path, jump_host, ssh_command, theme
             FROM hosts WHERE username=?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![username], row_to_host)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_host(&self, id: &str) -> Option<HostRecord> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, username, label, hostname, port, ssh_username,
                    key_id, key_path, jump_host, ssh_command, theme
             FROM hosts WHERE id=?1",
            params![id],
            row_to_host,
        ).ok()
    }

    pub fn upsert_host(&self, h: &HostRecord) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "INSERT INTO hosts (id, username, label, hostname, port, ssh_username,
                                key_id, key_path, jump_host, ssh_command, theme)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
             ON CONFLICT(id) DO UPDATE SET
               label=excluded.label, hostname=excluded.hostname, port=excluded.port,
               ssh_username=excluded.ssh_username, key_id=excluded.key_id,
               key_path=excluded.key_path, jump_host=excluded.jump_host,
               ssh_command=excluded.ssh_command, theme=excluded.theme",
            params![h.id, h.username, h.label, h.hostname, h.port as i64,
                    h.ssh_username, h.key_id, h.key_path, h.jump_host,
                    h.ssh_command, h.theme],
        )?;
        Ok(())
    }

    pub fn delete_host(&self, id: &str, username: &str) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "DELETE FROM hosts WHERE id=?1 AND username=?2",
            params![id, username],
        )?;
        Ok(())
    }

    // ── Keys ──────────────────────────────────────────────────────────────

    pub fn list_keys(&self, username: &str) -> anyhow::Result<Vec<KeyRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, username, name, path FROM keys WHERE username=?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![username], |r| Ok(KeyRecord {
            id:       r.get(0)?,
            username: r.get(1)?,
            name:     r.get(2)?,
            path:     r.get(3)?,
        }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn insert_key(&self, k: &KeyRecord) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "INSERT INTO keys (id, username, name, path) VALUES (?1,?2,?3,?4)",
            params![k.id, k.username, k.name, k.path],
        )?;
        Ok(())
    }

    pub fn delete_key(&self, id: &str, username: &str) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "DELETE FROM keys WHERE id=?1 AND username=?2",
            params![id, username],
        )?;
        Ok(())
    }

    // ── Sessions ──────────────────────────────────────────────────────────

    pub fn upsert_session(&self, rec: &SessionRecord) -> anyhow::Result<()> {
        let now = now_secs();
        self.conn.lock().execute(
            "INSERT INTO sessions (id, username, session_type, host_id, label, theme, font_size, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
               session_type=excluded.session_type, host_id=excluded.host_id,
               label=excluded.label, theme=excluded.theme, font_size=excluded.font_size,
               updated_at=excluded.updated_at",
            params![rec.id, rec.username, rec.session_type, rec.host_id,
                    rec.label, rec.theme, rec.font_size, now],
        )?;
        Ok(())
    }

    pub fn patch_session_prefs(&self, id: &str, username: &str, theme: &str, font_size: i64) -> anyhow::Result<()> {
        self.conn.lock().execute(
            "UPDATE sessions SET theme=?1, font_size=?2, updated_at=?3
             WHERE id=?4 AND username=?5",
            params![theme, font_size, now_secs(), id, username],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self, username: &str) -> anyhow::Result<Vec<SessionRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, username, session_type, host_id, label, theme, font_size, updated_at
             FROM sessions WHERE username=?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![username], |r| Ok(SessionRecord {
            id:           r.get(0)?,
            username:     r.get(1)?,
            session_type: r.get(2)?,
            host_id:      r.get(3)?,
            label:        r.get(4)?,
            theme:        r.get(5)?,
            font_size:    r.get(6)?,
            updated_at:   r.get(7)?,
        }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_session(&self, id: &str, username: &str) -> anyhow::Result<()> {
        {
            let conn = self.conn.lock();
            conn.execute("DELETE FROM sessions WHERE id=?1 AND username=?2", params![id, username])?;
            conn.execute("DELETE FROM scrollback WHERE session_id=?1", params![id])?;
        }
        self.redis_del_scrollback(id);
        Ok(())
    }

    // ── Scrollback ────────────────────────────────────────────────────────

    pub fn append_scrollback(&self, session_id: &str, chunk: &str) {
        // Redis write-through (fast, best-effort)
        if let Some(ref client) = self.redis {
            if let Ok(mut c) = client.get_connection() {
                let key = redis_key(session_id);
                let _: redis::RedisResult<()> = redis::cmd("RPUSH").arg(&key).arg(chunk).query(&mut c);
                let _: redis::RedisResult<()> = redis::cmd("LTRIM")
                    .arg(&key).arg(-(MAX_SCROLLBACK_LINES as isize)).arg(-1i64)
                    .query(&mut c);
                let _: redis::RedisResult<()> = redis::cmd("EXPIRE").arg(&key).arg(604800i64).query(&mut c);
            }
        }

        // SQLite write (durable)
        let conn = self.conn.lock();
        let seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq),0)+1 FROM scrollback WHERE session_id=?1",
                params![session_id], |r| r.get(0),
            ).unwrap_or(1);
        if let Err(e) = conn.execute(
            "INSERT INTO scrollback (session_id, chunk, seq) VALUES (?1,?2,?3)",
            params![session_id, chunk, seq],
        ) {
            tracing::warn!("scrollback insert: {e}");
            return;
        }
        // Prune oldest rows when over limit
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM scrollback WHERE session_id=?1",
                params![session_id], |r| r.get(0),
            ).unwrap_or(0);
        if count > MAX_SCROLLBACK_LINES {
            let excess = count - MAX_SCROLLBACK_LINES;
            let _ = conn.execute(
                "DELETE FROM scrollback WHERE session_id=?1 AND id IN (
                    SELECT id FROM scrollback WHERE session_id=?1 ORDER BY seq ASC LIMIT ?2
                )",
                params![session_id, excess],
            );
        }
    }

    /// Retrieve scrollback chunks (Redis → SQLite fallback).
    pub fn get_scrollback(&self, session_id: &str) -> Vec<String> {
        if let Some(ref client) = self.redis {
            if let Ok(mut c) = client.get_connection() {
                let key = redis_key(session_id);
                let res: redis::RedisResult<Vec<String>> =
                    redis::cmd("LRANGE").arg(&key).arg(0i64).arg(-1i64).query(&mut c);
                if let Ok(chunks) = res {
                    if !chunks.is_empty() { return chunks; }
                }
            }
        }
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare(
            "SELECT chunk FROM scrollback WHERE session_id=?1 ORDER BY seq ASC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![session_id], |r| r.get(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    fn redis_del_scrollback(&self, session_id: &str) {
        if let Some(ref client) = self.redis {
            if let Ok(mut c) = client.get_connection() {
                let _: redis::RedisResult<()> =
                    redis::cmd("DEL").arg(redis_key(session_id)).query(&mut c);
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn row_to_host(r: &rusqlite::Row<'_>) -> rusqlite::Result<HostRecord> {
    Ok(HostRecord {
        id:          r.get(0)?,
        username:    r.get(1)?,
        label:       r.get(2)?,
        hostname:    r.get(3)?,
        port:        r.get::<_, i64>(4)? as u16,
        ssh_username: r.get(5)?,
        key_id:      r.get(6)?,
        key_path:    r.get(7)?,
        jump_host:   r.get(8)?,
        ssh_command: r.get(9)?,
        theme:       r.get(10)?,
    })
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn redis_key(session_id: &str) -> String {
    format!("webssh:scrollback:{session_id}")
}
