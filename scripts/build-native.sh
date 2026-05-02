#!/usr/bin/env bash
# Build overdrive native library from OverDrive-DB server source (Linux/macOS)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$(dirname "$SCRIPT_DIR")"
SERVER_DIR="$(dirname "$SDK_DIR")/OverDrive-DB"

# Detect OS and arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  darwin) OS_NAME="macos" ;;
  linux)  OS_NAME="linux" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64)  ARCH_NAME="x64" ;;
  aarch64|arm64) ARCH_NAME="arm64" ;;
  *)       echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

PLATFORM="${OS_NAME}-${ARCH_NAME}"
LIB_DIR="${SDK_DIR}/lib/${PLATFORM}"
LIB_FILE="liboverdrive.${OS_NAME == macos && echo dylib || echo so}"
[ "$OS_NAME" = "macos" ] && EXT="dylib" || EXT="so"
OUT="${LIB_DIR}/liboverdrive.${EXT}"

echo "Building for: ${PLATFORM}"
echo "Server source: ${SERVER_DIR}"

if [ ! -d "$SERVER_DIR" ]; then
  echo "ERROR: Server not found at $SERVER_DIR"
  echo "  Set SERVER_PATH env var to point to the OverDrive-DB source."
  SERVER_DIR="${SERVER_PATH:-}"
fi

cd "$SERVER_DIR"
cargo build --features ffi --release

mkdir -p "$LIB_DIR"
cp "target/release/liboverdrive_db.${EXT}" "$OUT"
SIZE=$(du -sh "$OUT" | cut -f1)
echo "✅ liboverdrive.${EXT} -> $OUT ($SIZE)"
