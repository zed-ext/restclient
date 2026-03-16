#!/bin/bash
# Build and install script

set -e

# Save project directory
PROJECT_DIR="$(pwd)"

echo "Building REST Client Extension..."
echo ""

echo "Building HTTP LSP Server..."
echo ""
(cd lsp && cargo build --release 2>&1 | tail -20) || { echo "❌ LSP build failed."; exit 1; }

# Copy LSP binaries from lsp/target to root target for the install steps below
mkdir -p target/release
cp lsp/target/release/http-lsp target/release/ 2>/dev/null || true
cp lsp/target/release/http-client target/release/ 2>/dev/null || true
echo "✅ Built LSP server and http-client"
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
echo "5. Installing in Zed..."

# Determine the correct Zed extensions directory based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    ZED_EXT_DIR="$HOME/Library/Application Support/Zed/extensions/installed"
else
    ZED_EXT_DIR="$HOME/.config/zed/extensions"
fi

mkdir -p "$ZED_EXT_DIR"

INSTALL_DIR="$ZED_EXT_DIR/restclient"
TARGET="$INSTALL_DIR"
if [ -e "$TARGET" ]; then
    rm -rf "$TARGET"
fi

# Copy files instead of symlinking for dev extensions
mkdir -p "$TARGET"
cp extension.toml extension.wasm "$TARGET/"
cp -r languages "$TARGET/"
# Only copy the compiled grammar WASM, not the source tree
mkdir -p "$TARGET/grammars"
cp grammars/http.wasm "$TARGET/grammars/"

echo ""
echo "6. Copying LSP binary..."
if [ -f "target/release/http-lsp" ]; then
    mkdir -p "$INSTALL_DIR/lsp/target/release"
    cp target/release/http-lsp "$INSTALL_DIR/lsp/target/release/"
    # Re-sign after copy (macOS invalidates code signatures on copied binaries)
    codesign --force --sign - "$INSTALL_DIR/lsp/target/release/http-lsp" 2>/dev/null || true
    SIZE=$(ls -lh target/release/http-lsp | awk '{print $5}')
    echo "✅ Copied http-lsp ($SIZE)"

    mkdir -p "$HOME/.local/bin"
    cp target/release/http-lsp "$HOME/.local/bin/http-lsp"
    codesign --force --sign - "$HOME/.local/bin/http-lsp" 2>/dev/null || true
    echo "✅ Also installed to ~/.local/bin/http-lsp"
else
    echo "⚠️  LSP binary not found - code lens won't work"
fi

echo ""
echo "6b. Copying HTTP Client binary..."
if [ -f "target/release/http-client" ]; then
    mkdir -p "$HOME/.local/bin"
    cp target/release/http-client "$HOME/.local/bin/http-client"
    codesign --force --sign - "$HOME/.local/bin/http-client" 2>/dev/null || true
    SIZE=$(ls -lh target/release/http-client | awk '{print $5}')
    echo "✅ Installed http-client to ~/.local/bin/ ($SIZE)"
else
    echo "⚠️  http-client binary not found - task-based execution won't work"
    echo "   Build with: cargo build --release --package http-lsp --bin http-client"
fi
echo ""

# Configure Zed settings for optimal HTTP completion experience
echo "7. Configuring Zed settings for HTTP completions..."

# Determine settings file path
if [[ "$OSTYPE" == "darwin"* ]]; then
    ZED_SETTINGS="$HOME/Library/Application Support/Zed/settings.json"
else
    ZED_SETTINGS="$HOME/.config/zed/settings.json"
fi

configure_zed_settings() {
    local settings_file="$1"
    local settings_dir
    settings_dir="$(dirname "$settings_file")"

    # Ensure directory exists
    mkdir -p "$settings_dir"

    # If file doesn't exist or is empty, create with our settings
    if [ ! -f "$settings_file" ] || [ ! -s "$settings_file" ]; then
        cat > "$settings_file" << 'SETTINGS_EOF'
{
  "languages": {
    "HTTP": {
      "completions": {
        "words_min_length": 1
      }
    }
  }
}
SETTINGS_EOF
        echo "✅ Created Zed settings with HTTP completion config"
        return 0
    fi

    # File exists — need to merge. Use python3 for safe JSON handling
    # (jq can't handle JSONC comments that Zed settings may contain)
    if command -v python3 &> /dev/null; then
        python3 - "$settings_file" << 'PYTHON_EOF'
import json
import sys
import re
import os

settings_file = sys.argv[1]

with open(settings_file, "r") as f:
    raw = f.read()

# Strip JSONC comments (// and /* */) before parsing
# Remove single-line comments (but not inside strings)
cleaned = re.sub(r'(?m)^\s*//.*$', '', raw)        # full-line // comments
cleaned = re.sub(r'(?<!:)//[^\n"]*$', '', cleaned, flags=re.MULTILINE)  # trailing // comments
cleaned = re.sub(r'/\*.*?\*/', '', cleaned, flags=re.DOTALL)  # block comments

# Remove trailing commas before } or ]
cleaned = re.sub(r',(\s*[}\]])', r'\1', cleaned)

# Handle empty file after stripping
cleaned = cleaned.strip()
if not cleaned:
    cleaned = '{}'

try:
    settings = json.loads(cleaned)
except json.JSONDecodeError as e:
    print(f"⚠️  Could not parse {settings_file}: {e}", file=sys.stderr)
    print("   Please add manually to your Zed settings:", file=sys.stderr)
    print('   "languages": { "HTTP": { "completions": { "words_min_length": 1 } } }', file=sys.stderr)
    sys.exit(1)

# Check if already configured correctly
langs = settings.get("languages", {})
http_cfg = langs.get("HTTP", {})
comp_cfg = http_cfg.get("completions", {})
if comp_cfg.get("words_min_length") == 1:
    print("✅ HTTP completion settings already configured")
    sys.exit(0)

# Deep merge: set languages.HTTP.completions.words_min_length = 1
if "languages" not in settings:
    settings["languages"] = {}
if "HTTP" not in settings["languages"]:
    settings["languages"]["HTTP"] = {}
if "completions" not in settings["languages"]["HTTP"]:
    settings["languages"]["HTTP"]["completions"] = {}
settings["languages"]["HTTP"]["completions"]["words_min_length"] = 1

# Write back — we lose comments but preserve all other settings
# Back up original first
backup = settings_file + ".backup"
import shutil
shutil.copy2(settings_file, backup)

with open(settings_file, "w") as f:
    json.dump(settings, f, indent=2)
    f.write("\n")

print(f"✅ Configured HTTP completions (backup: {os.path.basename(backup)})")
PYTHON_EOF
    else
        echo "⚠️  python3 not found. Please add this to your Zed settings manually:"
        echo '  "languages": { "HTTP": { "completions": { "words_min_length": 1 } } }'
    fi
}

configure_zed_settings "$ZED_SETTINGS"

echo ""

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
echo "4. You should see syntax highlighting and ▶ Run buttons!"
echo ""
echo "To test, create test.http with:"
echo "  GET https://httpbin.org/get"
echo ""
