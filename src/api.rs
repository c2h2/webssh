use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::store::{AppState, HostEntry, KeyEntry};
use crate::db::{HostRecord, KeyRecord};
use crate::vault;
use crate::auth;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;
type SharedState = Arc<RwLock<AppState>>;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error": msg})))
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

fn session_user(jar: &CookieJar, server_key: &[u8]) -> Option<String> {
    let token = jar.get("session")?.value().to_string();
    auth::verify_token(&token, server_key)
}

fn set_session_cookie(username: &str, server_key: &[u8]) -> Cookie<'static> {
    let token = auth::create_token(username, server_key);
    Cookie::build(("session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .build()
}

// ── Auth routes ───────────────────────────────────────────────────────────────

pub async fn auth_status(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Json<Value> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key);
    let user_count = st.db.user_count();
    let is_admin = username.as_deref()
        .and_then(|u| st.db.get_user(u))
        .map(|u| u.is_admin)
        .unwrap_or(false);
    Json(json!({
        "logged_in": username.is_some(),
        "username":  username.unwrap_or_default(),
        "needs_register": user_count == 0,
        "is_admin": is_admin,
    }))
}

#[derive(Deserialize)]
pub struct AuthPayload { pub username: String, pub password: String }

pub async fn auth_register(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<AuthPayload>,
) -> Result<(CookieJar, Json<Value>), (StatusCode, Json<Value>)> {
    if p.username.is_empty() || p.password.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Username and password required"));
    }
    if p.password.len() < 8 {
        return Err(err(StatusCode::BAD_REQUEST, "Password must be at least 8 characters"));
    }
    let st = state.read().await;
    if st.db.user_count() > 0 {
        let caller = session_user(&jar, &st.server_key);
        let caller_is_admin = caller.as_deref()
            .and_then(|u| st.db.get_user(u))
            .map(|u| u.is_admin)
            .unwrap_or(false);
        if !caller_is_admin {
            // Non-admins can only register if registration is open
            let open = st.db.get_setting("registration_open", "true");
            if open != "true" {
                return Err(err(StatusCode::FORBIDDEN, "Registration is closed"));
            }
            if caller.is_none() {
                return Err(err(StatusCode::UNAUTHORIZED, "Must be logged in to register new users"));
            }
        }
    }
    auth::register(&st.db, &p.username, &p.password)
        .map_err(|e| err(StatusCode::BAD_REQUEST, &e.to_string()))?;
    let actor = session_user(&jar, &st.server_key).unwrap_or_else(|| p.username.clone());
    st.db.append_audit(&actor, "register", &p.username, "");
    let cookie = set_session_cookie(&p.username, &st.server_key);
    Ok((jar.add(cookie), Json(json!({"ok": true, "username": p.username}))))
}

pub async fn auth_login(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<AuthPayload>,
) -> Result<(CookieJar, Json<Value>), (StatusCode, Json<Value>)> {
    let st = state.read().await;
    auth::login(&st.db, &p.username, &p.password)
        .map_err(|e| {
            st.db.append_audit(&p.username, "login_fail", &p.username, &e.to_string());
            err(StatusCode::UNAUTHORIZED, "Invalid credentials")
        })?;
    st.db.append_audit(&p.username, "login", &p.username, "");
    let cookie = set_session_cookie(&p.username, &st.server_key);
    Ok((jar.add(cookie), Json(json!({"ok": true, "username": p.username}))))
}

pub async fn auth_logout(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> (CookieJar, Json<Value>) {
    let st = state.read().await;
    if let Some(username) = session_user(&jar, &st.server_key) {
        st.db.append_audit(&username, "logout", &username, "");
        drop(st);
        state.write().await.vault_passwords.remove(&username);
    }
    let jar = jar.remove(Cookie::from("session"));
    (jar, Json(json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct SettingsPayload { pub vault_key_hex: String }

pub async fn auth_settings(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<SettingsPayload>,
) -> ApiResult<Value> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    auth::set_vault_key(&st.db, &username, &p.vault_key_hex)
        .map_err(|e| err(StatusCode::BAD_REQUEST, &e.to_string()))?;
    Ok(Json(json!({"ok": true})))
}

// ── Vault ─────────────────────────────────────────────────────────────────────

pub async fn vault_status(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Json<Value> {
    let st = state.read().await;
    let user = session_user(&jar, &st.server_key);
    let unlocked = user.as_deref()
        .map(|u| st.vault_passwords.contains_key(u))
        .unwrap_or(false);
    Json(json!({ "exists": vault::exists(), "logged_in": user.is_some(), "unlocked": unlocked }))
}

#[derive(Deserialize)]
pub struct PwPayload { pub password: String }

pub async fn vault_init(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<PwPayload>,
) -> ApiResult<Value> {
    let username = {
        let st = state.read().await;
        session_user(&jar, &st.server_key)
            .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?
    };
    if p.password.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Password required"));
    }
    if vault::exists() {
        return Err(err(StatusCode::BAD_REQUEST, "Vault already exists"));
    }
    vault::init(&p.password).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    state.write().await.vault_passwords.insert(username, p.password);
    Ok(Json(json!({"ok": true})))
}

pub async fn vault_unlock(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<PwPayload>,
) -> ApiResult<Value> {
    let username = {
        let st = state.read().await;
        session_user(&jar, &st.server_key)
            .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?
    };
    vault::unlock(&p.password)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "Wrong password"))?;
    state.write().await.vault_passwords.insert(username, p.password);
    Ok(Json(json!({"ok": true})))
}

pub async fn vault_lock(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Json<Value> {
    let st = state.read().await;
    if let Some(username) = session_user(&jar, &st.server_key) {
        drop(st);
        state.write().await.vault_passwords.remove(&username);
    }
    Json(json!({"ok": true}))
}

// ── Hosts ─────────────────────────────────────────────────────────────────────

pub async fn get_hosts(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let hosts: Vec<HostEntry> = st.db.list_hosts(&username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .into_iter().map(HostEntry::from).collect();
    Ok(Json(json!(hosts)))
}

#[derive(Deserialize)]
pub struct AddHostPayload {
    pub label:          Option<String>,
    pub hostname:       String,
    pub port:           Option<u16>,
    pub username:       String,
    pub password:       Option<String>,
    pub key_id:         Option<String>,
    pub key_path:       Option<String>,
    pub key_passphrase: Option<String>,
    pub jump_host:      Option<String>,
    pub ssh_command:    Option<String>,
    pub theme:          Option<String>,
    pub vault_password: Option<String>,
}

pub async fn add_host(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<AddHostPayload>,
) -> ApiResult<Value> {
    let st = state.read().await;
    let owner = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let id = Uuid::new_v4().to_string();

    let rec = HostRecord {
        id:          id.clone(),
        username:    owner.clone(),
        label:       p.label.unwrap_or_default(),
        hostname:    p.hostname,
        port:        p.port.unwrap_or(22),
        ssh_username: p.username,
        key_id:      p.key_id.unwrap_or_default(),
        key_path:    p.key_path.unwrap_or_default(),
        jump_host:   p.jump_host.unwrap_or_default(),
        ssh_command: p.ssh_command.unwrap_or_default(),
        theme:       p.theme.unwrap_or_else(|| "hacker".into()),
    };

    let vp = p.vault_password.as_deref()
        .or_else(|| st.vault_passwords.get(&owner).map(|s| s.as_str()));
    if let Some(vp) = vp {
        save_credentials_to_vault(&id, vp, p.password.as_deref(), p.key_passphrase.as_deref());
    }

    st.db.upsert_host(&rec)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!(HostEntry::from(rec))))
}

#[derive(Deserialize)]
pub struct UpdateHostPayload {
    pub label:          Option<String>,
    pub hostname:       Option<String>,
    pub port:           Option<u16>,
    pub username:       Option<String>,
    pub password:       Option<String>,
    pub key_id:         Option<String>,
    pub key_path:       Option<String>,
    pub key_passphrase: Option<String>,
    pub jump_host:      Option<String>,
    pub ssh_command:    Option<String>,
    pub theme:          Option<String>,
    pub vault_password: Option<String>,
}

pub async fn update_host(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(p): Json<UpdateHostPayload>,
) -> ApiResult<Value> {
    let st = state.read().await;
    let owner = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let mut rec = st.db.get_host(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Host not found"))?;
    if rec.username != owner {
        return Err(err(StatusCode::FORBIDDEN, "Not your host"));
    }
    if let Some(v) = p.label       { rec.label        = v; }
    if let Some(v) = p.hostname    { rec.hostname      = v; }
    if let Some(v) = p.port        { rec.port          = v; }
    if let Some(v) = p.username    { rec.ssh_username  = v; }
    if let Some(v) = p.key_id      { rec.key_id        = v; }
    if let Some(v) = p.key_path    { rec.key_path      = v; }
    if let Some(v) = p.jump_host   { rec.jump_host     = v; }
    if let Some(v) = p.ssh_command { rec.ssh_command   = v; }
    if let Some(v) = p.theme       { rec.theme         = v; }

    let vp = p.vault_password.as_deref()
        .or_else(|| st.vault_passwords.get(&owner).map(|s| s.as_str()));
    if let Some(vp) = vp {
        save_credentials_to_vault(&id, vp, p.password.as_deref(), p.key_passphrase.as_deref());
    }

    st.db.upsert_host(&rec)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!(HostEntry::from(rec))))
}

pub async fn delete_host(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    st.db.delete_host(&id, &username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"ok": true})))
}

// ── Keys ──────────────────────────────────────────────────────────────────────

pub async fn get_keys(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let keys: Vec<KeyEntry> = st.db.list_keys(&username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .into_iter().map(KeyEntry::from).collect();
    Ok(Json(json!(keys)))
}

#[derive(Deserialize)]
pub struct AddKeyPayload { pub name: String, pub path: String }

pub async fn add_key(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(p): Json<AddKeyPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let rec = KeyRecord { id: Uuid::new_v4().to_string(), username, name: p.name, path: p.path };
    st.db.insert_key(&rec)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!(KeyEntry::from(rec))))
}

pub async fn delete_key(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    st.db.delete_key(&id, &username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"ok": true})))
}

// ── Persistent sessions ───────────────────────────────────────────────────────

pub async fn list_sessions(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let sessions = st.db.list_sessions(&username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!(sessions)))
}

pub async fn delete_session(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (username, db) = {
        let st = state.read().await;
        let username = session_user(&jar, &st.server_key)
            .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
        (username, st.db.clone())
    };
    db.delete_session(&id, &username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    // Also kill the live PTY if it's running
    {
        let mut st = state.write().await;
        if let Some(ls) = st.live_sessions.remove(&id) {
            *ls.pty.lock().await = None;
        }
    }
    Ok(Json(json!({"ok": true})))
}

pub async fn patch_session(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let username = session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let theme     = body["theme"].as_str().unwrap_or("").to_string();
    let font_size = body["font_size"].as_i64().unwrap_or(13);
    st.db.patch_session_prefs(&id, &username, &theme, font_size)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(json!({"ok": true})))
}

pub async fn get_scrollback(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    session_user(&jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let chunks = st.db.get_scrollback(&id);
    Ok(Json(json!({ "chunks": chunks })))
}

// ── Admin routes ──────────────────────────────────────────────────────────────

fn require_admin(jar: &CookieJar, st: &AppState) -> Result<String, (StatusCode, Json<Value>)> {
    let username = session_user(jar, &st.server_key)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not logged in"))?;
    let user = st.db.get_user(&username)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "User not found"))?;
    if !user.is_admin {
        return Err(err(StatusCode::FORBIDDEN, "Admin only"));
    }
    Ok(username)
}

pub async fn admin_get_settings(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    require_admin(&jar, &st)?;
    let open = st.db.get_setting("registration_open", "true") == "true";
    Ok(Json(json!({ "registration_open": open })))
}

pub async fn admin_set_settings(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let caller = require_admin(&jar, &st)?;
    let open = body["registration_open"].as_bool().unwrap_or(true);
    let val = if open { "true" } else { "false" };
    st.db.set_setting("registration_open", val)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    st.db.append_audit(&caller, "set_registration", "", &format!("registration_open={val}"));
    Ok(Json(json!({"ok": true})))
}

pub async fn admin_list_users(
    State(state): State<SharedState>,
    jar: CookieJar,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    require_admin(&jar, &st)?;
    let users = st.db.list_users()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let out: Vec<Value> = users.iter().map(|u| json!({
        "username":    u.username,
        "is_admin":    u.is_admin,
        "is_disabled": u.is_disabled,
    })).collect();
    Ok(Json(json!(out)))
}

pub async fn admin_set_user_disabled(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(username): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let caller = require_admin(&jar, &st)?;
    if username == caller {
        return Err(err(StatusCode::BAD_REQUEST, "Cannot disable yourself"));
    }
    let disabled = body["disabled"].as_bool().unwrap_or(false);
    st.db.set_user_disabled(&username, disabled)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let action = if disabled { "disable_user" } else { "enable_user" };
    st.db.append_audit(&caller, action, &username, "");
    Ok(Json(json!({"ok": true})))
}

pub async fn admin_delete_user(
    State(state): State<SharedState>,
    jar: CookieJar,
    Path(username): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    let caller = require_admin(&jar, &st)?;
    if username == caller {
        return Err(err(StatusCode::BAD_REQUEST, "Cannot delete yourself"));
    }
    st.db.delete_user(&username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    st.db.append_audit(&caller, "delete_user", &username, "");
    Ok(Json(json!({"ok": true})))
}

pub async fn admin_get_audit(
    State(state): State<SharedState>,
    jar: CookieJar,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let st = state.read().await;
    require_admin(&jar, &st)?;
    let limit  = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(200i64);
    let offset = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0i64);
    let entries = st.db.list_audit(limit, offset);
    let total   = st.db.audit_count();
    Ok(Json(json!({ "entries": entries, "total": total })))
}

// ── Vault helpers ─────────────────────────────────────────────────────────────

fn save_credentials_to_vault(
    host_id: &str,
    vault_pw: &str,
    password: Option<&str>,
    passphrase: Option<&str>,
) {
    if !vault::exists() { return; }
    if let Ok(mut vdata) = vault::unlock(vault_pw) {
        if let Some(pw) = password {
            if !pw.is_empty() { vdata[format!("host_{host_id}_password")] = json!(pw); }
        }
        if let Some(pp) = passphrase {
            if !pp.is_empty() { vdata[format!("host_{host_id}_passphrase")] = json!(pp); }
        }
        vault::save(vault_pw, &vdata).ok();
    }
}
