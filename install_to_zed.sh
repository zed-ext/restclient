#!/bin/bash
# Build and install script

set -e

# Save project directory
PROJECT_DIR="$(pwd)"

echo "Building REST Client Extension..."
echo ""

# Install WASM target
echo "1. Installing wasm32-wasip1 target..."
rustup target add wasm32-wasip1 2>&1 | grep -v "info: component" || true

echo ""
echo "2. Building for WASM..."
if ! cargo build --release --target wasm32-wasip1 2>&1 | tail -20; then
    echo ""
    echo "❌ Build failed. See errors above."
    exit 1
fi

echo ""
echo "3. Copying WASM binary..."
cp target/wasm32-wasip1/release/zed_restclient.wasm extension.wasm

echo "✅ Built extension.wasm ($(ls -lh extension.wasm | awk '{print $5}'))"
echo ""

# Build grammar if needed
if [ ! -f "grammars/http.wasm" ]; then
    echo "4. Building tree-sitter grammar..."

    # Install tree-sitter CLI if not present
    if ! command -v tree-sitter &> /dev/null; then
        echo "   Installing tree-sitter CLI..."
        npm install -g tree-sitter-cli
    fi

    # Clone and build grammar
    echo "   Cloning tree-sitter-http..."
    cd /tmp
    rm -rf tree-sitter-http
    git clone --depth 1 https://github.com/rest-nvim/tree-sitter-http.git
    cd tree-sitter-http

    echo "   Compiling grammar to WASM..."
    tree-sitter build --wasm -o http.wasm

    # Copy back
    cd "$PROJECT_DIR"
    mkdir -p grammars
    cp /tmp/tree-sitter-http/http.wasm grammars/http.wasm

    echo "✅ Built grammar: grammars/http.wasm ($(ls -lh grammars/http.wasm | awk '{print $5}'))"
else
    echo "4. Grammar already built: grammars/http.wasm"
fi

echo ""

# Install
echo "4. Installing in Zed..."

# Determine the correct Zed extensions directory based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    ZED_EXT_DIR="$HOME/Library/Application Support/Zed/extensions/installed"
else
    ZED_EXT_DIR="$HOME/.config/zed/extensions"
fi

mkdir -p "$ZED_EXT_DIR"

TARGET="$ZED_EXT_DIR/restclient"
if [ -e "$TARGET" ]; then
    rm -rf "$TARGET"
fi

# Copy files instead of symlinking for dev extensions
mkdir -p "$TARGET"
cp extension.toml extension.wasm "$TARGET/"
cp -r languages grammars "$TARGET/"

echo "✅ Installed at: $TARGET"
echo ""
echo "=========================================="
echo "Installation Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. COMPLETELY QUIT Zed (Cmd+Q, not just close window)"
echo "2. Restart Zed"
echo "3. Open a .http or .rest file"
echo "4. You should see syntax highlighting!"
echo ""
echo "To test, create test.http with:"
echo "  GET https://httpbin.org/get"
echo ""
echo "Note: Commands (/restclient-execute) won't work yet - only syntax highlighting is implemented."
echo ""
