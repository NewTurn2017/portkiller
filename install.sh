#!/bin/sh
set -e

REPO="NewTurn2017/portkiller"
BINARY="pk"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_TAG="apple-darwin" ;;
  Linux)  OS_TAG="unknown-linux-gnu" ;;
  *)
    echo "Error: Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_TAG="x86_64" ;;
  arm64|aarch64) ARCH_TAG="aarch64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="${ARCH_TAG}-${OS_TAG}"
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Error: Could not fetch latest release"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}.tar.gz"

echo "Installing pk ${LATEST} (${TARGET})..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "${TMPDIR}/pk.tar.gz"
tar -xzf "${TMPDIR}/pk.tar.gz" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
  echo "Need sudo to install to ${INSTALL_DIR}"
  sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Done! pk installed to ${INSTALL_DIR}/${BINARY}"
echo "Run 'pk --help' to get started."
