# WebSSH

A self-hosted web-based SSH client built with Rust (Axum) and xterm.js.

## Features

- **Browser SSH** — full terminal emulation via xterm.js over WebSocket
- **Multi-tab sessions** — each browser tab gets its own PTY; sessions persist across page reloads with full scrollback replay
- **Host manager** — save, edit, and delete SSH host profiles (hostname, port, user, key, jump host, custom command, theme)
- **SSH key manager** — register key paths; keys are referenced by hosts
- **Encrypted vault** — credentials stored encrypted at rest (AES-GCM, PBKDF2)
- **User accounts** — register/login with Argon2-hashed passwords, cookie-based sessions
- **SQLite storage** — all data in a single `data/webssh.db` file; no external DB required
- **Redis scrollback cache** — optional; falls back to SQLite automatically if Redis is not running

## Requirements

- Rust 1.75+
- OpenSSH client available on the host (`ssh` in `$PATH`)
- Redis (optional, for scrollback caching)

## Build

```bash
./compile.sh          # cargo build --release
./compile.sh clean    # cargo clean then build
```

Binary output: `target/release/webssh`

## Run

```bash
./target/release/webssh
```

Listens on `0.0.0.0:5001`. Open `http://localhost:5001` in your browser.

The `data/` directory is created automatically on first run and contains:
- `webssh.db` — SQLite database (users, hosts, keys, sessions, scrollback)
- `server.key` — HMAC signing key for session tokens
- `vault.enc` / `vault.salt` — encrypted vault data

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/auth/status` | Check login state |
| POST | `/api/auth/register` | Create account |
| POST | `/api/auth/login` | Log in |
| POST | `/api/auth/logout` | Log out |
| GET/POST | `/api/hosts` | List / add hosts |
| PUT/DELETE | `/api/hosts/:id` | Update / delete host |
| GET/POST | `/api/keys` | List / add keys |
| DELETE | `/api/keys/:id` | Delete key |
| GET | `/api/sessions` | List open sessions |
| DELETE | `/api/sessions/:id` | Close session |
| GET | `/api/sessions/:id/scrollback` | Fetch scrollback |
| GET | `/ws` | WebSocket terminal |

## Stack

- **Backend:** Rust, Axum 0.7, Tokio, rusqlite (bundled SQLite), redis
- **Crypto:** aes-gcm, argon2, pbkdf2, sha2, hmac
- **PTY:** nix (fork/execve)
- **Frontend:** xterm.js, vanilla JS SPA

## Security notes

- Runs SSH as the OS user that started the process
- The vault must be unlocked after each server restart before encrypted credentials are usable
- `data/` contains sensitive files — keep it out of version control (already in `.gitignore`)
