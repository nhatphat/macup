#!/bin/bash
set -euo pipefail

REPO="nhatphat/macup"
ASSET="macup-aarch64-apple-darwin.tar.gz"
INSTALL_DIR="${MACUP_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="$HOME/.config/macup"
CONFIG_FILE="$CONFIG_DIR/config.toml"

info() {
    printf '%s\n' "$1"
}

fail() {
    printf 'Error: %s\n' "$1" >&2
    exit 1
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

if [[ "$(uname -s)" != "Darwin" ]]; then
    fail "macup release installer currently supports macOS only"
fi

if [[ "$(uname -m)" != "arm64" ]]; then
    fail "macup release installer currently supports Apple Silicon (arm64) only"
fi

require_cmd curl
require_cmd tar
require_cmd shasum

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BASE_URL="https://github.com/${REPO}/releases/latest/download"

info "Downloading macup latest release..."
curl -fsSL "${BASE_URL}/${ASSET}" -o "${TMP_DIR}/${ASSET}"
curl -fsSL "${BASE_URL}/${ASSET}.sha256" -o "${TMP_DIR}/${ASSET}.sha256"

info "Verifying checksum..."
(
    cd "$TMP_DIR"
    shasum -a 256 -c "${ASSET}.sha256"
)

info "Installing/updating macup to ${INSTALL_DIR}..."
mkdir -p "$INSTALL_DIR"
tar -xzf "${TMP_DIR}/${ASSET}" -C "$TMP_DIR"
install -m 0755 "${TMP_DIR}/macup-aarch64-apple-darwin/macup" "$INSTALL_DIR/macup"

mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_FILE" ]]; then
    cp "${TMP_DIR}/macup-aarch64-apple-darwin/config.example.toml" "$CONFIG_FILE"
    info "Created default config at ${CONFIG_FILE}"
else
    info "Config already exists at ${CONFIG_FILE}"
fi

info "macup installed successfully."
info "Binary: ${INSTALL_DIR}/macup"
info "Config: ${CONFIG_FILE}"

case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        info ""
        info "Add this to your shell profile if macup is not found:"
        info "export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac
