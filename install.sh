#!/usr/bin/env bash
# Warning: AI Generated content
# Installation script for git-polyp

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

check_dependencies() {
    print_info "Checking dependencies..."
    
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo is not installed or not in PATH"
        print_info "Please install Rust and Cargo: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v git >/dev/null 2>&1; then
        print_error "git is not installed or not in PATH"
        print_info "Please install Git"
        exit 1
    fi
    
    print_success "Dependencies check passed"
}

build_release() {
    print_info "Building release binary..."
    
    cd "$SCRIPT_DIR"
    
    # Clean previous build artifacts if requested
    if [ "$1" = "--clean" ]; then
        print_info "Cleaning previous build artifacts..."
        cargo clean
    fi
    
    # Build release
    if ! cargo build --release; then
        print_error "Failed to build release binary"
        exit 1
    fi
    
    # Verify binary exists
    if [ ! -f "$SCRIPT_DIR/target/release/git-polyp" ]; then
        print_error "Release binary not found at target/release/git-polyp"
        exit 1
    fi
    
    print_success "Release binary built successfully"
}

install_binary() {
    print_info "Installing binary to /usr/local/bin..."
    
    local binary_path="$SCRIPT_DIR/target/release/git-polyp"
    local install_path="/usr/local/bin/git-polyp"
    
    # Check if /usr/local/bin exists
    if [ ! -d "/usr/local/bin" ]; then
        print_error "/usr/local/bin directory does not exist"
        print_info "Please create it with: sudo mkdir -p /usr/local/bin"
        exit 1
    fi
    
    # Try to copy without sudo first
    if cp "$binary_path" "$install_path" 2>/dev/null; then
        print_success "Installed to $install_path"
    else
        # Need sudo
        print_info "Administrator privileges required to install to /usr/local/bin"
        if sudo cp "$binary_path" "$install_path"; then
            print_success "Installed to $install_path (with sudo)"
        else
            print_error "Failed to install binary to $install_path"
            exit 1
        fi
    fi
    
    # Make sure it's executable
    if ! chmod +x "$install_path" 2>/dev/null; then
        sudo chmod +x "$install_path"
    fi
    
    # Verify installation
    if command -v git-polyp >/dev/null 2>&1; then
        local version=$(git-polyp --version 2>/dev/null || echo "unknown")
        print_success "git-polyp is now available in PATH ($version)"
    else
        print_warning "git-polyp installed but not found in PATH"
        print_info "You may need to restart your shell or add /usr/local/bin to your PATH"
    fi
}

install_completions() {
    print_info "Installing shell completions..."
    
    local completions_script="$SCRIPT_DIR/completions/install.sh"
    
    if [ ! -f "$completions_script" ]; then
        print_error "Completions install script not found at $completions_script"
        exit 1
    fi
    
    # Make sure the script is executable
    chmod +x "$completions_script"
    
    # Run the completions installer
    if "$completions_script"; then
        print_success "Shell completions installed"
    else
        print_warning "Shell completions installation failed, but continuing..."
    fi
}

show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Install git-polyp binary and shell completions.

Options:
  --clean           Clean build artifacts before building
  --binary-only     Install only the binary, skip completions
  --completions-only Install only completions, skip building and binary installation
  -h, --help        Show this help message

Examples:
  $0                # Build and install everything
  $0 --clean        # Clean build, then install everything
  $0 --binary-only  # Build and install only the binary
  $0 --completions-only # Install only completions

EOF
}

main() {
    local clean_build=false
    local binary_only=false
    local completions_only=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                clean_build=true
                shift
                ;;
            --binary-only)
                binary_only=true
                shift
                ;;
            --completions-only)
                completions_only=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    echo "🚀 git-polyp Installation Script"
    echo "=================================="
    echo ""
    
    if [ "$completions_only" = true ]; then
        install_completions
    else
        check_dependencies
        
        if [ "$clean_build" = true ]; then
            build_release --clean
        else
            build_release
        fi
        
        install_binary
        
        if [ "$binary_only" != true ]; then
            echo ""
            install_completions
        fi
    fi
    
    echo ""
    print_success "Installation complete!"
    echo ""
    print_info "You can now use git-polyp by running:"
    echo "  git polyp <command>"
    echo "  git-polyp <command>"
    echo ""
    print_info "For help, run:"
    echo "  git polyp --help"
    echo "  git-polyp --help"
}

main "$@"
