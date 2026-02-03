#!/bin/bash
# Berry installation script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/geoffjay/berry-rs/main/scripts/install.sh | bash
#
# Environment variables:
#   BERRY_INSTALL_DIR - Installation directory (default: ~/.local/bin)
#   BERRY_VERSION     - Version to install (default: latest)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os
    local arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)       error "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="amd64" ;;
        arm64|aarch64) arch="arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get the latest version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/geoffjay/berry-rs/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install Berry
install_berry() {
    local platform="$1"
    local version="$2"
    local install_dir="$3"

    local base_url="https://github.com/geoffjay/berry-rs/releases/download"
    local asset_name="berry-${platform}"
    local download_url

    if [[ "$platform" == *"windows"* ]]; then
        download_url="${base_url}/${version}/${asset_name}.zip"
    else
        download_url="${base_url}/${version}/${asset_name}.tar.gz"
    fi

    info "Downloading Berry ${version} for ${platform}..."
    info "URL: ${download_url}"

    # Create temp directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    # Download
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "$tmp_dir/berry.tar.gz"
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "$tmp_dir/berry.tar.gz"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    # Extract
    info "Extracting..."
    cd "$tmp_dir"
    if [[ "$platform" == *"windows"* ]]; then
        unzip -q berry.tar.gz
    else
        tar -xzf berry.tar.gz
    fi

    # Install
    info "Installing to ${install_dir}..."
    mkdir -p "$install_dir"

    for binary in berry berry-server berry-mcp; do
        if [[ -f "$binary" ]]; then
            chmod +x "$binary"
            mv "$binary" "$install_dir/"
        fi
    done

    info "Installation complete!"
}

# Check if install directory is in PATH
check_path() {
    local install_dir="$1"

    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        warn "Installation directory is not in your PATH."
        warn "Add the following to your shell configuration:"
        echo ""
        echo "  export PATH=\"\$PATH:$install_dir\""
        echo ""
    fi
}

# Main
main() {
    local platform
    local version
    local install_dir

    platform=$(detect_platform)
    version="${BERRY_VERSION:-$(get_latest_version)}"
    install_dir="${BERRY_INSTALL_DIR:-$HOME/.local/bin}"

    info "Platform: ${platform}"
    info "Version: ${version}"
    info "Install directory: ${install_dir}"
    echo ""

    install_berry "$platform" "$version" "$install_dir"
    check_path "$install_dir"

    echo ""
    info "Run 'berry --help' to get started."
    info "Run 'berry init' to create a configuration file."
}

main "$@"
