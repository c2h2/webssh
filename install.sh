#!/bin/sh
set -e

REPO="c2h2/webssh"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/webssh"
BIN="webssh"

# ── Detect OS and arch ────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  OS_NAME="linux" ;;
  Darwin) OS_NAME="macos" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)   ARCH_NAME="x86_64" ;;
  aarch64|arm64)  ARCH_NAME="arm64"  ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

ARTIFACT="webssh-${OS_NAME}-${ARCH_NAME}"

# ── Resolve version ───────────────────────────────────────────────────────────
if [ -z "$VERSION" ]; then
  echo "Fetching latest release..."
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
fi

if [ -z "$VERSION" ]; then
  echo "Could not determine latest version. Set VERSION= and retry."
  exit 1
fi

echo "Installing WebSSH ${VERSION} (${OS_NAME}/${ARCH_NAME})..."

# ── Download ──────────────────────────────────────────────────────────────────
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
TARBALL="${ARTIFACT}.tar.gz"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "${BASE_URL}/${TARBALL}"        -o "${TMPDIR}/${TARBALL}"
curl -fsSL "${BASE_URL}/${TARBALL}.sha256" -o "${TMPDIR}/${TARBALL}.sha256"

# ── Verify checksum ───────────────────────────────────────────────────────────
cd "$TMPDIR"
# sha256sum on Linux, shasum on macOS
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c "${TARBALL}.sha256"
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c "${TARBALL}.sha256"
else
  echo "Warning: no sha256 tool found, skipping checksum verification"
fi

# ── Extract and install binary ────────────────────────────────────────────────
tar xzf "$TARBALL"

NEEDS_SUDO=""
if [ ! -w "$INSTALL_DIR" ]; then
  NEEDS_SUDO="sudo"
fi

$NEEDS_SUDO install -m 755 "${ARTIFACT}" "${INSTALL_DIR}/${BIN}"

# ── Install static assets ─────────────────────────────────────────────────────
# WebSSH serves static/ and templates/ relative to its working directory.
# Install them to /usr/local/share/webssh so the systemd unit can find them.
SHARE_DIR="/usr/local/share/webssh"
$NEEDS_SUDO mkdir -p "$SHARE_DIR"
$NEEDS_SUDO cp -r static templates "$SHARE_DIR/"

# ── Data directory ────────────────────────────────────────────────────────────
$NEEDS_SUDO mkdir -p "$DATA_DIR"

echo ""
echo "WebSSH ${VERSION} installed to ${INSTALL_DIR}/${BIN}"
echo ""
echo "Run it:"
echo "  cd /usr/local/share/webssh && webssh"
echo ""

# ── Optional: install systemd unit (Linux only) ───────────────────────────────
if [ "$OS_NAME" = "linux" ] && [ -d /etc/systemd/system ] && [ -n "$NEEDS_SUDO" ]; then
  $NEEDS_SUDO tee /etc/systemd/system/webssh.service >/dev/null <<EOF
[Unit]
Description=WebSSH
After=network.target

[Service]
ExecStart=${INSTALL_DIR}/${BIN}
WorkingDirectory=${SHARE_DIR}
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
  $NEEDS_SUDO systemctl daemon-reload
  echo "Systemd unit installed. Enable with:"
  echo "  sudo systemctl enable --now webssh"
  echo ""
fi
