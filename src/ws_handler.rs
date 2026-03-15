/// WebSocket handler — plain JSON messages, one WS connection per terminal tab.
///
/// Client → Server messages:
///   {"type":"start_local","session_id":"...","cols":220,"rows":50}
///   {"type":"start_ssh","session_id":"...","host_id":"...","cols":220,"rows":50}
///   {"type":"start_mosh","session_id":"...","host_id":"...","cols":220,"rows":50}
///   {"type":"input","data":"..."}
///   {"type":"resize","cols":80,"rows":24}
///   {"type":"close"}
///
/// Server → Client messages:
///   {"type":"connected"}
///   {"type":"output","data":"..."}
///   {"type":"disconnected"}
///   {"type":"error","message":"..."}

use axum::extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}};
use axum::response::Response;
use axum_extra::extract::CookieJar;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::store::{AppState, LiveSession, WsSinkSender};
use crate::pty_session::PtySession;
use crate::db::{Db, SessionRecord};
use crate::auth;

type WsSink = Arc<Mutex<SplitSink<WebSocket, Message>>>;
type SharedState = Arc<RwLock<AppState>>;

pub async fn ws_upgrade(
    State(state): State<SharedState>,
    jar: CookieJar,
    ws: WebSocketUpgrade,
) -> Response {
    let (username, db) = {
        let st = state.read().await;
        let u = jar.get("session")
            .and_then(|c| auth::verify_token(c.value(), &st.server_key))
            .unwrap_or_default();
        (u, st.db.clone())
    };
    ws.on_upgrade(move |socket| handle_ws(socket, username, db, state))
}

async fn send_json(sink: &WsSink, v: Value) {
    let txt = v.to_string();
    let mut s = sink.lock().await;
    let _ = s.send(Message::Text(txt.into())).await;
}

async fn handle_ws(socket: WebSocket, username: String, db: Db, state: SharedState) {
    let (sink, mut stream) = socket.split();
    let sink = Arc::new(Mutex::new(sink));

    // The PTY handle for this connection (only set when we own a new spawn;
    // for reattaches we use the live_sessions map).
    let local_pty: Arc<Mutex<Option<Arc<Mutex<Option<PtySession>>>>>> =
        Arc::new(Mutex::new(None));
    // session_id active for this WS connection
    let active_session_id: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    while let Some(Ok(msg)) = stream.next().await {
        let text = match msg {
            Message::Text(t)   => t.to_string(),
            Message::Binary(b) => match String::from_utf8(b.to_vec()) {
                Ok(s) => s,
                Err(_) => continue,
            },
            Message::Close(_) => break,
            _ => continue,
        };

        let v: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = v["type"].as_str().unwrap_or("").to_string();

        match msg_type.as_str() {
            "start_local" | "start_ssh" | "start_mosh" => {
                let cols = v["cols"].as_u64().unwrap_or(220) as u16;
                let rows = v["rows"].as_u64().unwrap_or(50) as u16;
                let session_id = v["session_id"].as_str().unwrap_or("").to_string();
                let host_id = v["host_id"].as_str().unwrap_or("").to_string();

                *active_session_id.lock().await = session_id.clone();

                // Check if there's already a live PTY for this session_id
                let existing_sink_tx = if !session_id.is_empty() {
                    let st = state.read().await;
                    st.live_sessions.get(&session_id).map(|ls| ls.sink_tx.clone())
                } else {
                    None
                };

                if let Some(sink_tx) = existing_sink_tx {
                    // Reattach: wire the existing background reader to this new WS sink
                    let sink_clone = Arc::clone(&sink);
                    let sender: WsSinkSender = Arc::new(move |data: String| {
                        let sink = Arc::clone(&sink_clone);
                        let _ = tokio::runtime::Handle::current().block_on(async move {
                            send_json(&sink, serde_json::json!({"type":"output","data": data})).await;
                        });
                    });
                    let _ = sink_tx.send(Some(sender));

                    send_json(&sink, serde_json::json!({"type":"connected"})).await;

                    // Store a ref so resize/input can find the PTY
                    let pty_ref = {
                        let st = state.read().await;
                        st.live_sessions.get(&session_id).map(|ls| Arc::clone(&ls.pty))
                    };
                    *local_pty.lock().await = pty_ref;

                    if let Some(pty_arc) = local_pty.lock().await.as_ref() {
                        let guard = pty_arc.lock().await;
                        if let Some(p) = guard.as_ref() {
                            p.resize(cols, rows);
                        }
                    }
                } else {
                    // New session — spawn a PTY
                    let argv = match msg_type.as_str() {
                        "start_local" => build_local_argv(),
                        "start_ssh"   => build_ssh_argv(&v, &username, &db),
                        "start_mosh"  => build_mosh_argv(&v, &username, &db),
                        _ => unreachable!(),
                    };

                    if argv.is_empty() {
                        send_json(&sink, serde_json::json!({"type":"error","message":"Could not resolve command"})).await;
                        continue;
                    }

                    // Determine label + theme for the session record
                    let (label, theme) = {
                        let hosts = db.list_hosts(&username).unwrap_or_default();
                        hosts.into_iter()
                            .find(|h| h.id == host_id)
                            .map(|h| {
                                let lbl = if h.label.is_empty() { h.hostname.clone() } else { h.label.clone() };
                                (lbl, h.theme.clone())
                            })
                            .unwrap_or_else(|| ("local".to_string(), "hacker".to_string()))
                    };

                    // Persist session metadata (upsert)
                    if !session_id.is_empty() && !username.is_empty() {
                        let rec = SessionRecord {
                            id:           session_id.clone(),
                            username:     username.clone(),
                            session_type: msg_type.clone(),
                            host_id:      host_id.clone(),
                            label:        label.clone(),
                            theme:        theme.clone(),
                            font_size:    13,
                            updated_at:   0,
                        };
                        if let Err(e) = db.upsert_session(&rec) {
                            tracing::warn!("upsert_session: {e}");
                        }
                    }

                    let argv_str: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
                    match PtySession::spawn(&argv_str, cols, rows) {
                        Ok(session) => {
                            let pty_arc: Arc<Mutex<Option<PtySession>>> =
                                Arc::new(Mutex::new(Some(session)));

                            // Create a watch channel so the background reader can swap sinks on reconnect
                            let sink_clone = Arc::clone(&sink);
                            let initial_sender: WsSinkSender = Arc::new(move |data: String| {
                                let sink = Arc::clone(&sink_clone);
                                let _ = tokio::runtime::Handle::current().block_on(async move {
                                    send_json(&sink, serde_json::json!({"type":"output","data": data})).await;
                                });
                            });
                            let (sink_tx, sink_rx) =
                                tokio::sync::watch::channel(Some(initial_sender));

                            // Register in live_sessions
                            if !session_id.is_empty() {
                                let mut st = state.write().await;
                                st.live_sessions.insert(session_id.clone(), LiveSession {
                                    pty: Arc::clone(&pty_arc),
                                    sink_tx,
                                });
                            }

                            *local_pty.lock().await = Some(Arc::clone(&pty_arc));

                            send_json(&sink, serde_json::json!({"type":"connected"})).await;

                            // Spawn background reader — lives as long as the PTY
                            let pty_bg   = Arc::clone(&pty_arc);
                            let db_clone = db.clone();
                            let sid_clone = session_id.clone();
                            let state_clone = Arc::clone(&state);

                            tokio::task::spawn_blocking(move || {
                                let rt = tokio::runtime::Handle::current();
                                let mut buf = [0u8; 4096];
                                loop {
                                    let result = {
                                        let guard = rt.block_on(pty_bg.lock());
                                        guard.as_ref().map(|p| p.try_read(&mut buf))
                                    };
                                    match result {
                                        None => break,
                                        Some(Err(())) => break,
                                        Some(Ok(None)) => {
                                            std::thread::sleep(std::time::Duration::from_millis(5));
                                        }
                                        Some(Ok(Some(n))) => {
                                            let s = String::from_utf8_lossy(&buf[..n]).into_owned();
                                            if !sid_clone.is_empty() {
                                                db_clone.append_scrollback(&sid_clone, &s);
                                            }
                                            // Forward to current sink (if any client is connected)
                                            let current_sender = sink_rx.borrow().clone();
                                            if let Some(sender) = current_sender {
                                                sender(s);
                                            }
                                        }
                                    }
                                }
                                // PTY exited — remove from live_sessions
                                if !sid_clone.is_empty() {
                                    rt.block_on(async {
                                        let mut st = state_clone.write().await;
                                        st.live_sessions.remove(&sid_clone);
                                    });
                                }
                            });
                        }
                        Err(e) => {
                            send_json(&sink, serde_json::json!({"type":"error","message": e.to_string()})).await;
                        }
                    }
                }
            }

            "input" => {
                let data = v["data"].as_str().unwrap_or("").as_bytes().to_vec();
                if let Some(pty_arc) = local_pty.lock().await.as_ref() {
                    let guard = pty_arc.lock().await;
                    if let Some(p) = guard.as_ref() {
                        let _ = p.write_bytes(&data);
                    }
                }
            }

            "resize" => {
                let cols = v["cols"].as_u64().unwrap_or(80) as u16;
                let rows = v["rows"].as_u64().unwrap_or(24) as u16;
                if let Some(pty_arc) = local_pty.lock().await.as_ref() {
                    let guard = pty_arc.lock().await;
                    if let Some(p) = guard.as_ref() {
                        p.resize(cols, rows);
                    }
                }
            }

            "close" => {
                // Explicit close: kill the PTY and clean up live_sessions
                let sid = active_session_id.lock().await.clone();
                if !sid.is_empty() {
                    let mut st = state.write().await;
                    if let Some(ls) = st.live_sessions.remove(&sid) {
                        *ls.pty.lock().await = None; // Drop PtySession → SIGTERM
                    }
                }
                break;
            }

            _ => {}
        }
    }

    // WS closed (browser refresh / tab hidden / network drop) — detach sink but keep PTY alive.
    // Signal the background reader that there's no current sink.
    let sid = active_session_id.lock().await.clone();
    if !sid.is_empty() {
        let st = state.read().await;
        if let Some(ls) = st.live_sessions.get(&sid) {
            let _ = ls.sink_tx.send(None);
        }
    }
}

// ── Argv builders ──────────────────────────────────────────────────────────

fn build_local_argv() -> Vec<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    vec![shell]
}

fn build_ssh_argv_from_host(host: &crate::db::HostRecord) -> Vec<String> {
    if !host.ssh_command.is_empty() {
        return shell_split(&host.ssh_command);
    }
    let ssh = which("ssh").unwrap_or_else(|| "ssh".into());
    let mut argv = vec![ssh, "-tt".to_string()];
    if host.port != 22 {
        argv.push("-p".to_string());
        argv.push(host.port.to_string());
    }
    if !host.key_path.is_empty() {
        argv.push("-i".to_string());
        argv.push(host.key_path.clone());
    }
    if !host.jump_host.is_empty() {
        argv.push("-J".to_string());
        argv.push(host.jump_host.clone());
    }
    argv.push("-o".to_string());
    argv.push("StrictHostKeyChecking=accept-new".to_string());
    argv.push("-o".to_string());
    argv.push("BatchMode=no".to_string());
    argv.push("-o".to_string());
    argv.push("PasswordAuthentication=yes".to_string());
    argv.push(format!("{}@{}", host.ssh_username, host.hostname));
    argv
}

fn build_ssh_argv(v: &Value, username: &str, db: &Db) -> Vec<String> {
    let host_id = v["host_id"].as_str().unwrap_or("").to_string();
    let hosts = db.list_hosts(username).unwrap_or_default();
    let Some(host) = hosts.into_iter().find(|h| h.id == host_id) else {
        return vec![];
    };
    build_ssh_argv_from_host(&host)
}

fn build_mosh_argv(v: &Value, username: &str, db: &Db) -> Vec<String> {
    let host_id = v["host_id"].as_str().unwrap_or("").to_string();
    let hosts = db.list_hosts(username).unwrap_or_default();
    let Some(host) = hosts.into_iter().find(|h| h.id == host_id) else {
        return vec![];
    };
    let mosh = which("mosh").unwrap_or_else(|| "mosh".into());
    let mut argv = vec![mosh];
    if host.port != 22 {
        argv.push("--ssh".to_string());
        argv.push(format!("ssh -p {}", host.port));
    }
    argv.push(format!("{}@{}", host.ssh_username, host.hostname));
    argv
}

fn shell_split(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    for ch in s.chars() {
        match ch {
            '"' => in_q = !in_q,
            ' ' | '\t' if !in_q => {
                if !cur.is_empty() { args.push(cur.clone()); cur.clear(); }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() { args.push(cur); }
    args
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        let p = format!("{dir}/{name}");
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}
