#!/bin/bash
# Installation script for roadmap-cli
# Usage: curl -fsSL https://raw.githubusercontent.com/siovos/roadmap-cli/main/install.sh | bash

set -e

REPO="siovos/roadmap-cli"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)
    case "$ARCH" in
      x86_64) BINARY="roadmap-cli-darwin-x86_64" ;;
      arm64)  BINARY="roadmap-cli-darwin-arm64" ;;
      *)      echo "Architecture non supportée: $ARCH"; exit 1 ;;
    esac
    ;;
  linux)
    case "$ARCH" in
      x86_64) BINARY="roadmap-cli-linux-x86_64" ;;
      *)      echo "Architecture non supportée: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "OS non supporté: $OS"
    exit 1
    ;;
esac

# Get latest release
echo "📦 Téléchargement de roadmap-cli..."
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
  echo "❌ Impossible de récupérer la dernière version"
  exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$BINARY.tar.gz"

# Download and install
TMPDIR=$(mktemp -d)
curl -fsSL "$URL" -o "$TMPDIR/roadmap-cli.tar.gz"
tar -xzf "$TMPDIR/roadmap-cli.tar.gz" -C "$TMPDIR"

echo "📁 Installation dans $INSTALL_DIR..."
sudo mv "$TMPDIR/roadmap-cli" "$INSTALL_DIR/roadmap"
sudo chmod +x "$INSTALL_DIR/roadmap"

# Cleanup
rm -rf "$TMPDIR"

echo "✅ roadmap-cli installé avec succès!"
echo ""
echo "Utilisation:"
echo "  roadmap init          # Initialiser dans un projet"
echo "  roadmap --help        # Voir toutes les commandes"
