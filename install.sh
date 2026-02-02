#!/bin/bash

# ftrack - Automated Installer for Linux
# --------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================${NC}"
echo -e "${BLUE}    ftrack: Industrial Web Archiver    ${NC}"
echo -e "${BLUE}=======================================${NC}"

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo -e "${RED}[!] Gagal mendeteksi sistem operasi.${NC}"
    exit 1
fi

echo -e "[*] Mendeteksi OS: ${GREEN}$OS${NC}"

# Install dependencies based on OS
case $OS in
    ubuntu|debian)
        echo -e "[*] Menginstal dependensi (Sudo required)..."
        sudo apt-get update
        sudo apt-get install -y pkg-config libssl-dev libsqlite3-dev build-essential chromium
        ;;
    fedora)
        echo -e "[*] Menginstal dependensi (Sudo required)..."
        sudo dnf install -y pkg-config openssl-devel sqlite-devel gcc-c++ chromium
        ;;
    arch)
        echo -e "[*] Menginstal dependensi (Sudo required)..."
        sudo pacman -Sy --noconfirm pkg-config openssl sqlite gcc chromium
        ;;
    *)
        echo -e "${RED}[!] OS tidak didukung secara otomatis. Mohon instal pkg-config, openssl, sqlite, dan chromium secara manual.${NC}"
        ;;
esac

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "[*] Rust belum terinstal. Menginstal Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo -e "${GREEN}[V] Rust sudah terinstal.${NC}"
fi

# Build Project
echo -e "[*] Membangun ftrack (ini mungkin memakan waktu beberapa menit)..."
cargo build --release

# Setup local binary
echo -e "[*] Menyiapkan executable..."
cp target/release/ftrack .
chmod +x ftrack

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}  INSTALASI SELESAI!                   ${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "Gunakan: ${BLUE}./ftrack <URL>${NC}"
echo -e "Dokumentasi lengkap: ${BLUE}README.md${NC}"
