#!/bin/sh
set -e

REPO="c2h2/webssh"
INSTALL_DIR="/usr/local/bin"
BIN="webssh"
SHARE_DIR="/usr/local/share/webssh"

# ── Detect OS and arch ────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  OS_NAME="linux"  ;;
  Darwin) OS_NAME="macos"  ;;
  *)
    echo "Unsupported OS: $OS. Only Linux and macOS are supported."
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_NAME="x86_64" ;;
  aarch64|arm64) ARCH_NAME="arm64"  ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Data directory: persistent, outside the install tree
if [ "$OS_NAME" = "linux" ]; then
  DATA_DIR="/var/lib/webssh"
else
  DATA_DIR="/Library/Application Support/webssh"
fi

ARTIFACT="webssh-${OS_NAME}-${ARCH_NAME}"

# ── Resolve version ───────────────────────────────────────────────────────────
if [ -z "$VERSION" ]; then
  echo "Fetching latest release..."
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
fi

if [ -z "$VERSION" ]; then
  echo "Could not determine latest version. Set VERSION=vX.Y.Z and retry."
  exit 1
fi

echo "Installing WebSSH ${VERSION} (${OS_NAME}/${ARCH_NAME})..."

# ── Sudo helper ───────────────────────────────────────────────────────────────
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  if command -v sudo >/dev/null 2>&1; then
    SUDO="sudo"
  else
    echo "Not running as root and sudo not found. Re-run as root."
    exit 1
  fi
fi

# ── Download ──────────────────────────────────────────────────────────────────
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
TARBALL="${ARTIFACT}.tar.gz"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading ${TARBALL}..."
curl -fsSL "${BASE_URL}/${TARBALL}"        -o "${TMP}/${TARBALL}"
curl -fsSL "${BASE_URL}/${TARBALL}.sha256" -o "${TMP}/${TARBALL}.sha256"

# ── Verify checksum ───────────────────────────────────────────────────────────
echo "Verifying checksum..."
cd "$TMP"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c "${TARBALL}.sha256"
elif command -v shasum >/dev/null 2>&1; then
  # macOS ships shasum, not sha256sum. The .sha256 file has the filename in it,
  # so we rewrite the path to match the local file.
  sed "s|${ARTIFACT}.tar.gz|${TMP}/${TARBALL}|" "${TARBALL}.sha256" | shasum -a 256 -c
else
  echo "Warning: no sha256 tool found, skipping checksum verification"
fi

# ── Extract ───────────────────────────────────────────────────────────────────
tar xzf "$TARBALL"

# ── Install binary ────────────────────────────────────────────────────────────
echo "Installing binary to ${INSTALL_DIR}/${BIN}..."
$SUDO install -m 755 "${TMP}/${ARTIFACT}" "${INSTALL_DIR}/${BIN}"

# ── Install static assets ─────────────────────────────────────────────────────
echo "Installing assets to ${SHARE_DIR}..."
$SUDO mkdir -p "$SHARE_DIR"
$SUDO cp -r "${TMP}/static" "${TMP}/templates" "$SHARE_DIR/"

# ── Data directory ────────────────────────────────────────────────────────────
# The app uses a relative path "data/" from its working dir.
# We create the real data dir in a safe system location and symlink it.
echo "Setting up data directory at ${DATA_DIR}..."
$SUDO mkdir -p "$DATA_DIR"
$SUDO chmod 750 "$DATA_DIR"

# Symlink SHARE_DIR/data -> DATA_DIR so the app finds it at its relative path
if [ ! -L "${SHARE_DIR}/data" ]; then
  $SUDO ln -sf "$DATA_DIR" "${SHARE_DIR}/data"
fi

# ── Platform-specific service setup ──────────────────────────────────────────
if [ "$OS_NAME" = "linux" ]; then
  # ── systemd ──────────────────────────────────────────────────────────────
  if [ ! -d /etc/systemd/system ]; then
    echo "systemd not found, skipping service installation."
  else
    echo "Installing systemd service..."
    $SUDO tee /etc/systemd/system/webssh.service >/dev/null <<EOF
[Unit]
Description=WebSSH — browser-based SSH client
After=network.target

[Service]
ExecStart=${INSTALL_DIR}/${BIN}
WorkingDirectory=${SHARE_DIR}
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=${DATA_DIR}

[Install]
WantedBy=multi-user.target
EOF
    $SUDO systemctl daemon-reload
    $SUDO systemctl enable webssh
    $SUDO systemctl restart webssh
    echo ""
    echo "Service status:"
    $SUDO systemctl status webssh --no-pager || true
  fi

elif [ "$OS_NAME" = "macos" ]; then
  # ── launchd plist ─────────────────────────────────────────────────────────
  PLIST_DIR="/Library/LaunchDaemons"
  PLIST="${PLIST_DIR}/com.c2h2.webssh.plist"
  LOG_DIR="/var/log/webssh"

  echo "Installing launchd service..."
  $SUDO mkdir -p "$LOG_DIR"
  $SUDO tee "$PLIST" >/dev/null <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>             <string>com.c2h2.webssh</string>
  <key>ProgramArguments</key>  <array><string>${INSTALL_DIR}/${BIN}</string></array>
  <key>WorkingDirectory</key>  <string>${SHARE_DIR}</string>
  <key>RunAtLoad</key>         <true/>
  <key>KeepAlive</key>         <true/>
  <key>StandardOutPath</key>   <string>${LOG_DIR}/webssh.log</string>
  <key>StandardErrorPath</key> <string>${LOG_DIR}/webssh.err</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key> <string>info</string>
  </dict>
</dict>
</plist>
EOF
  # Unload first in case it was already loaded (upgrade scenario)
  $SUDO launchctl unload "$PLIST" 2>/dev/null || true
  $SUDO launchctl load -w "$PLIST"
  echo "Service loaded. Logs: ${LOG_DIR}/"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "WebSSH ${VERSION} installed successfully."
echo "  Binary : ${INSTALL_DIR}/${BIN}"
echo "  Assets : ${SHARE_DIR}"
echo "  Data   : ${DATA_DIR}"
echo "  URL    : http://localhost:3000"
echo ""
