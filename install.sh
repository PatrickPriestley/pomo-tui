#!/bin/bash

# pomo-tui Installation Script
# This script installs the minimal working version of pomo-tui

set -e

echo "🍅 Installing pomo-tui - ADHD-focused Pomodoro Timer"
echo "=================================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✅ Rust installed successfully"
else
    echo "✅ Rust is already installed"
fi

# Build the project
echo "🔨 Building pomo-tui..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Test the binary
echo "🧪 Testing the binary..."
./target/release/pomo-tui --version

# Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy binary
echo "📦 Installing binary to $INSTALL_DIR..."
cp target/release/pomo-tui "$INSTALL_DIR/"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "⚠️  Adding $HOME/.local/bin to PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
    export PATH="$HOME/.local/bin:$PATH"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "You can now run:"
echo "  pomo-tui --version"
echo "  pomo-tui --help" 
echo "  pomo-tui          # Interactive mode"
echo ""
echo "If the command is not found, restart your terminal or run:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Note: This is a minimal version. The full TUI with all features"
echo "requires additional dependencies and implementation."