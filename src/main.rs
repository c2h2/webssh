mod vault;
mod store;
mod pty_session;
mod ws_handler;
mod api;
mod auth;
mod db;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put, delete},
    response::Html,
};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

pub use store::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    std::fs::create_dir_all("data").ok();

    let server_key = auth::load_or_create_server_key();

    let db = db::Db::open("data/webssh.db")
        .expect("Failed to open SQLite DB")
        .with_redis("redis://127.0.0.1/");

    let state = Arc::new(RwLock::new(AppState::new(server_key, db)));

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(root_handler))
        // Auth
        .route("/api/auth/status",   get(api::auth_status))
        .route("/api/auth/register", post(api::auth_register))
        .route("/api/auth/login",    post(api::auth_login))
        .route("/api/auth/logout",   post(api::auth_logout))
        .route("/api/auth/settings", post(api::auth_settings))
        // Vault
        .route("/api/vault/status", get(api::vault_status))
        .route("/api/vault/init",   post(api::vault_init))
        .route("/api/vault/unlock", post(api::vault_unlock))
        .route("/api/vault/lock",   post(api::vault_lock))
        // Hosts
        .route("/api/hosts",     get(api::get_hosts).post(api::add_host))
        .route("/api/hosts/:id", put(api::update_host).delete(api::delete_host))
        // Keys
        .route("/api/keys",     get(api::get_keys).post(api::add_key))
        .route("/api/keys/:id", delete(api::delete_key))
        // Admin
        .route("/api/admin/settings",              get(api::admin_get_settings).post(api::admin_set_settings))
        .route("/api/admin/users",                 get(api::admin_list_users))
        .route("/api/admin/users/:username/disable", post(api::admin_set_user_disabled))
        .route("/api/admin/users/:username",       delete(api::admin_delete_user))
        .route("/api/admin/audit",                 get(api::admin_get_audit))
        // Persistent sessions
        .route("/api/sessions",                    get(api::list_sessions))
        .route("/api/sessions/:id",                delete(api::delete_session).patch(api::patch_session))
        .route("/api/sessions/:id/scrollback",     get(api::get_scrollback))
        // WebSocket
        .route("/ws", get(ws_handler::ws_upgrade))
        .with_state(state);

    // Bind [::] to accept both IPv6 and IPv4 (dual-stack) on port 13337.
    // On Linux the kernel maps IPv4 connections to ::ffff:x.x.x.x by default.
    // On macOS dual-stack on [::] also works out of the box.
    let addr = "[::]:13337";
    tracing::info!("WebSSH listening on http://0.0.0.0:13337 and http://[::]:13337");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}
