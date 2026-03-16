# WebSSH

A self-hosted web-based SSH client built with Rust (Axum) and xterm.js.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/c2h2/webssh/main/install.sh | sh
```

Downloads the latest release for your OS/arch, installs the binary and assets, sets up a persistent data directory, and registers a system service (systemd on Linux, launchd on macOS) that starts automatically on boot.

After install, open **http://localhost:13337** in your browser.

To install a specific version:

```sh
curl -fsSL https://raw.githubusercontent.com/c2h2/webssh/main/install.sh | VERSION=v0.2.6 sh
```

**Requirements:** `curl`, `tar`, `sudo` (or run as root). No Rust toolchain needed — pre-built binaries for Linux x86\_64, Linux arm64, macOS x86\_64, macOS arm64.

---

## Features

- **Browser SSH** — full terminal emulation via xterm.js over WebSocket
- **Split panes** — 1, 2 (left/right), or 4-pane (2×2) layouts; layout persists across refreshes
- **Multi-tab sessions** — each tab gets its own PTY; sessions persist with full scrollback replay
- **Host manager** — save SSH host profiles (hostname, port, user, key, jump host, custom command, theme)
- **SSH key manager** — register key paths; keys are referenced by hosts
- **Encrypted vault** — credentials stored encrypted at rest (AES-GCM, PBKDF2)
- **User accounts** — register/login with Argon2-hashed passwords, cookie-based sessions
- **Admin panel** — user management, audit log, registration control
- **SQLite storage** — all data in a single `data/webssh.db` file; no external DB required
- **Redis scrollback cache** — optional; falls back to SQLite automatically if Redis is not running

## Build from source

**Requirements:** Rust 1.75+, OpenSSH client in `$PATH`

```bash
./compile.sh          # cargo build --release
./compile.sh clean    # cargo clean then build
```

Binary output: `target/release/webssh`

```bash
./target/release/webssh
```

Listens on `[::]:13337` (dual-stack IPv4+IPv6). Open `http://localhost:13337`.

The `data/` directory is created automatically on first run and contains:
- `webssh.db` — SQLite database (users, hosts, keys, sessions, scrollback)
- `server.key` — HMAC signing key for session tokens
- `vault.enc` / `vault.salt` — encrypted vault data

## Service management

**Linux (systemd)**
```bash
sudo systemctl status webssh
sudo systemctl restart webssh
sudo journalctl -u webssh -f
```

**macOS (launchd)**
```bash
sudo launchctl list | grep webssh
tail -f /var/log/webssh/webssh.log
```

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
| GET/POST | `/api/layout` | Get / set split layout prefs |
| GET | `/api/admin/users` | List users (admin) |
| GET | `/api/admin/audit` | Audit log (admin) |
| GET/POST | `/api/admin/settings` | Server settings (admin) |
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
- Bind behind a reverse proxy with TLS if exposed beyond localhost
