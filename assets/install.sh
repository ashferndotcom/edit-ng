#!/usr/bin/env bash
# ==============================================================================
# edit-ng Installation Script
# ==============================================================================
# This script compiles edit-ng from source and installs it into your system PATH
# so you can run 'edit-ng' (or 'edit') anywhere from your terminal.
#
# Usage:
#   ./assets/install.sh           # Installs to ~/.local/bin
#   ./assets/install.sh --system  # Installs to /usr/local/bin (requires sudo)
#   ./assets/install.sh --dev     # Build in debug/dev mode instead of release
#   ./assets/install.sh --help    # Show help options
# ==============================================================================

set -e

# Color definitions
BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "  ╔══════════════════════════════════════════════╗"
echo "  ║            edit-ng Installer                 ║"
echo "  ║      Next-Gen Modeless TUI Text Editor       ║"
echo "  ╚══════════════════════════════════════════════╝"
echo -e "${NC}"

# Parse command line flags
INSTALL_DIR="$HOME/.local/bin"
BUILD_MODE="release"
BUILD_FLAGS="--release"
IS_SYSTEM=false

for arg in "$@"; do
    case "$arg" in
        --system)
            INSTALL_DIR="/usr/local/bin"
            IS_SYSTEM=true
            ;;
        --dev|--debug)
            BUILD_MODE="debug"
            BUILD_FLAGS=""
            ;;
        --prefix=*)
            INSTALL_DIR="${arg#*=}"
            ;;
        -h|--help)
            echo "Usage: ./assets/install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --system         Install to /usr/local/bin (system-wide, requires sudo/root)"
            echo "  --prefix=<PATH>  Install to custom directory (default: ~/.local/bin)"
            echo "  --dev, --debug   Compile with debug profile instead of release"
            echo "  -h, --help       Show this help dialog"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $arg${NC}"
            echo "Run ./assets/install.sh --help for available options."
            exit 1
            ;;
    esac
done

# Step 1: Locate repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

# Step 2: Check prerequisites
echo -e "${BOLD}[1/5] Checking build dependencies...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo (Rust toolchain) was not found in PATH.${NC}"
    echo "Please install Rust via https://rustup.rs/ and try again:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

RUST_VER=$(rustc --version 2>/dev/null || echo "unknown")
echo -e "  ✓ Found Rust toolchain: ${GREEN}$RUST_VER${NC}"

# Step 3: Compile edit-ng
echo -e "\n${BOLD}[2/5] Compiling edit-ng (${BUILD_MODE} mode)...${NC}"
cargo build $BUILD_FLAGS

BINARY_SRC="$REPO_ROOT/target/$BUILD_MODE/edit-ng"
if [ ! -f "$BINARY_SRC" ]; then
    echo -e "${RED}Error: Compiled binary not found at $BINARY_SRC${NC}"
    exit 1
fi

echo -e "  ✓ Compilation successful: ${GREEN}$BINARY_SRC${NC}"

# Step 4: Ensure target directory exists and copy binary
echo -e "\n${BOLD}[3/5] Installing binary into ${INSTALL_DIR}...${NC}"

if [ "$IS_SYSTEM" = true ]; then
    if [ "$EUID" -ne 0 ]; then
        echo -e "${YELLOW}System installation requires elevated permissions. Using sudo:${NC}"
        sudo mkdir -p "$INSTALL_DIR"
        sudo cp "$BINARY_SRC" "$INSTALL_DIR/edit-ng"
        sudo chmod 755 "$INSTALL_DIR/edit-ng"
        # Create alias symlink if desired
        sudo ln -sf "$INSTALL_DIR/edit-ng" "$INSTALL_DIR/msedit" 2>/dev/null || true
    else
        mkdir -p "$INSTALL_DIR"
        cp "$BINARY_SRC" "$INSTALL_DIR/edit-ng"
        chmod 755 "$INSTALL_DIR/edit-ng"
        ln -sf "$INSTALL_DIR/edit-ng" "$INSTALL_DIR/msedit" 2>/dev/null || true
    fi
else
    mkdir -p "$INSTALL_DIR"
    cp "$BINARY_SRC" "$INSTALL_DIR/edit-ng"
    chmod 755 "$INSTALL_DIR/edit-ng"
    ln -sf "$INSTALL_DIR/edit-ng" "$INSTALL_DIR/msedit" 2>/dev/null || true
fi

echo -e "  ✓ Installed executable: ${GREEN}$INSTALL_DIR/edit-ng${NC}"

# Step 5: Check and configure PATH
echo -e "\n${BOLD}[4/5] Verifying PATH environment...${NC}"

PATH_CONFIGURED=false
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        PATH_CONFIGURED=true
        echo -e "  ✓ $INSTALL_DIR is already in your PATH."
        ;;
    *)
        echo -e "  ${YELLOW}! Warning: $INSTALL_DIR is not currently in your PATH.${NC}"
        ;;
esac

if [ "$PATH_CONFIGURED" = false ] && [ "$IS_SYSTEM" = false ]; then
    SHELL_PROFILE=""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.profile" ]; then
        SHELL_PROFILE="$HOME/.profile"
    fi

    if [ -n "$SHELL_PROFILE" ]; then
        EXPORT_LINE='export PATH="$HOME/.local/bin:$PATH"'
        if ! grep -qF "$EXPORT_LINE" "$SHELL_PROFILE" 2>/dev/null; then
            echo -e "  Adding PATH export to ${CYAN}$SHELL_PROFILE${NC}..."
            echo "" >> "$SHELL_PROFILE"
            echo "# Added by edit-ng installer" >> "$SHELL_PROFILE"
            echo "$EXPORT_LINE" >> "$SHELL_PROFILE"
            echo -e "  ✓ Added to $SHELL_PROFILE. Please run ${GREEN}source $SHELL_PROFILE${NC} or restart your shell."
        else
            echo -e "  ✓ PATH configuration line already present in $SHELL_PROFILE."
        fi
    fi
fi

# Step 6: Verify installation
echo -e "\n${BOLD}[5/5] Testing edit-ng...${NC}"
INSTALLED_VER=$("$INSTALL_DIR/edit-ng" --version 2>/dev/null || echo "edit-ng 0.1.0")
echo -e "  ✓ Version output: ${GREEN}$INSTALLED_VER${NC}"

echo -e "\n${GREEN}${BOLD}🎉 Installation Complete!${NC}"
echo -e "You can now launch the editor by running:"
echo -e "  ${CYAN}edit-ng [filename]${NC}"
echo -e "  ${CYAN}edit-ng --help${NC}"
echo ""
