#!/bin/bash
set -e

echo "🚀 Bootstrapping macup..."
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "📦 Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add brew to PATH (for Apple Silicon Macs)
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
    fi
    
    echo "✓ Homebrew installed"
else
    echo "✓ Homebrew is installed"
fi
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed"
    echo ""
fi

echo "✓ Rust is installed"
echo ""

# Build macup
echo "🔨 Building macup..."
cargo build --release

# Install binary
echo "📦 Installing macup to ~/.cargo/bin..."
cargo install --path .

echo ""
echo "=========================================="
echo "✅ Bootstrap complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Create or edit your config: vim macup.toml"
echo "  2. Preview changes:  macup apply --dry-run"
echo "  3. Apply setup:      macup apply"
echo ""
echo "Example config has been created at: macup.toml"
echo ""
