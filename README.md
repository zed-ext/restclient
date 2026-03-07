# REST Client Extension for Zed

A REST client extension for Zed IDE that allows you to write and execute HTTP requests directly from `.http` files.

## 🎬 Demo

![Demo](docs/video/demo.gif)

---

## ⚡ Quick Start

### 1. Install the Extension

```bash
./install_to_zed.sh
```

### 2. Restart Zed

**Important**: Completely quit Zed (`Cmd+Q` on macOS) and reopen it.

### 3. Create a Test File

Create `test.http`:

```http
### Test request
GET https://httpbin.org/get
```

### 4. Execute the Request

**Option A: Using Command Palette (Works Immediately)**

1. Click anywhere in the request
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Windows/Linux)
3. Type "task spawn"
4. Select "Send Request at Cursor"
5. See response in terminal!

**Option B: Set Up Keyboard Shortcut (Recommended for Frequent Use)**

1. Press `Cmd+,` to open Zed settings
2. Click **"Open Keymap"** in the top right
3. Add this to your keymap:

```json
[
  {
    "context": "Editor",
    "bindings": {
      "ctrl-enter": ["task::Spawn", {"task_name": "Send Request at Cursor"}]
    }
  }
]
```

4. Save the file
5. Now just press `Ctrl+Enter` anywhere in a request to execute it!

---

## 🎯 How It Works

### Cursor-Based Execution

Just place your cursor **anywhere** in a request and run the command. The extension automatically:

1. Gets your cursor line number
2. Finds which request block contains that line
3. Executes that specific request

**No need to select text or number your requests!**

### Example

```http
### Get user info
GET https://jsonplaceholder.typicode.com/users/1

### Create new post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "userId": 1
}

### Delete user
DELETE https://jsonplaceholder.typicode.com/users/1
```

Click anywhere in the POST request (lines 5-12) and execute - only that request runs!

---

## ✨ Features

✅ **Cursor-Based Execution** - Click anywhere in a request to run it
✅ **Syntax Highlighting** - Full syntax highlighting for `.http` and `.rest` files
✅ **Multiple Request Formats** - Support for GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
✅ **Headers & Body** - Full support for custom headers and request bodies
✅ **Request Separator** - Use `###` to separate multiple requests in a file
✅ **Variable Support** - Use `{{variable}}` syntax (parsing ready, execution coming soon)
✅ **No Configuration Required** - Works out of the box

---

## 📖 Writing HTTP Requests

### Basic Request

```http
### Simple GET request
GET https://httpbin.org/get
```

### POST with JSON Body

```http
### Create a user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### Request with Headers

```http
### Authenticated request
GET https://api.example.com/data
Authorization: Bearer your-token-here
Accept: application/json
User-Agent: REST Client
```

### Multiple Requests in One File

```http
### Get all users
GET https://api.example.com/users

### Get specific user
GET https://api.example.com/users/123

### Create user
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "Jane"
}
```

Each `###` starts a new request block. Click anywhere in a block to execute just that request.

---

## 🎨 File Format

The extension supports the [JetBrains HTTP Client file format](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html):

```http
### Comment describing the request
METHOD URL [HTTP-version]
Header-Name: Header-Value
[Another-Header: Value]

[Request Body]
```

### Request Separators

Use `###` to separate multiple requests:

```http
### First request
GET https://example.com/api/users

###
### Second request
POST https://example.com/api/users
```

### Comments

Lines starting with `#` or `//` are comments:

```http
# This is a comment
// This is also a comment

### Actual request
GET https://api.example.com
```

### Headers

One header per line in `Name: Value` format:

```http
GET https://api.example.com
Authorization: Bearer abc123
Content-Type: application/json
Accept: application/json
```

### Request Body

Add request body after a blank line:

```http
POST https://api.example.com/data
Content-Type: application/json

{
  "key": "value"
}
```

### Variables (Coming Soon)

Declare variables with `@name = value`:

```http
@baseUrl = https://api.example.com
@userId = 123

### Use variables with {{name}}
GET {{baseUrl}}/users/{{userId}}
```

**Note**: Variable parsing works, but substitution is not yet implemented.

---

## 🔧 Alternative Execution Methods

### Method 1: Command Palette (Default)

```
Cmd+Shift+P → "task spawn" → "Send Request at Cursor"
```

**Pros**: No setup required, works immediately
**Best for**: Quick testing, first-time use

### Method 2: Keyboard Shortcut (Recommended)

Set up `Ctrl+Enter` (see Quick Start above)

**Pros**: Fastest, most convenient
**Best for**: Frequent API testing

### Method 3: Command Line

```bash
# Build the CLI tool
cargo build --release --features cli --bin http-execute

# Execute request at specific line
cargo run --release --features cli --bin http-execute test.http --line 10

# Execute first request (default)
./target/release/http-execute test.http
```

**Pros**: Scriptable, CI/CD integration
**Best for**: Automation, testing pipelines

---

## 🐛 Troubleshooting

### Task doesn't appear in command palette

1. Make sure you're typing "task spawn" (not just "send" or "request")
2. Restart Zed completely (`Cmd+Q` and reopen)
3. Verify extension is installed: `Cmd+Shift+P` → "extensions"

### "No HTTP request found at line X"

Your cursor is outside any request block. Move it inside a request:

```http
###                    ← Don't put cursor here
# Comment line         ← Or here
                       ← Or here
GET https://...        ← ✅ Put cursor here
Content-Type: ...      ← ✅ Or here
                       ← ✅ Or here
{ "data": "value" }    ← ✅ Or here
                       ← ✅ Or here
###                    ← Don't put cursor here
```

### Keyboard shortcut doesn't work

1. Make sure you're in a `.http` or `.rest` file
2. Check you added the keymap correctly (see Quick Start)
3. Restart Zed after editing keymap
4. Try the command palette method to verify the extension works

### Syntax highlighting not working

1. Check file extension is `.http` or `.rest` (not `.txt`)
2. Restart Zed completely (`Cmd+Q`)
3. Reinstall: `./install_to_zed.sh`

### Seeing duplicate tasks in the list

This is normal Zed behavior - the task history shows previous runs. Just select any "Send Request at Cursor" - they're all identical.

**Solution**: Set up the keyboard shortcut to skip the task list entirely!

---

## 📋 Example File

See `test.http` for comprehensive examples including:

- Simple GET requests
- POST with JSON bodies
- Requests with custom headers
- Form data submissions
- Query parameters
- Variable declarations

---

## 🚀 Roadmap

- [x] Syntax highlighting
- [x] HTTP request parsing
- [x] CLI tool for executing requests
- [x] Zed task integration
- [x] Cursor-based execution
- [ ] Variable substitution in requests
- [ ] Environment management (`.http-client/environments.json`)
- [ ] Response handlers with JavaScript
- [ ] File reference support (`< ./file.json`)
- [ ] Response formatting and pretty-printing
- [ ] Request history
- [ ] Certificate and authentication helpers

---

## 🛠️ Development

### Building

```bash
# Build the WASM extension
cargo build --release --target wasm32-wasip1

# Build the CLI tool
cargo build --release --features cli --bin http-execute
```

### Testing

```bash
# Test the parser
cargo test

# Test the CLI with a sample file
cargo run --release --features cli --bin http-execute test.http --line 19
```

### Project Structure

```
restclient/
├── src/
│   ├── lib.rs              # Extension entry point
│   ├── parser.rs           # HTTP request parser
│   ├── bin/
│   │   └── http_execute.rs # CLI tool for executing requests
│   └── ...
├── languages/http/         # Language configuration
│   ├── config.toml         # Language settings
│   ├── highlights.scm      # Syntax highlighting rules
│   └── tasks.json          # Task definitions
├── grammars/http.wasm      # Compiled Tree-sitter grammar
└── extension.toml          # Extension manifest
```

---

## 📚 Documentation

- **[docs/SETUP.md](docs/SETUP.md)** - 5-minute setup guide
- **[docs/CURSOR_BASED.md](docs/CURSOR_BASED.md)** - Complete guide to cursor-based execution
- **[docs/SLASH_COMMANDS.md](docs/SLASH_COMMANDS.md)** - Using slash commands (requires LLM)
- **[docs/QUICK_START_SLASH.md](docs/QUICK_START_SLASH.md)** - Quick reference for slash commands

---

## 📝 License

MIT

## 🤝 Contributing

Contributions welcome! Please open an issue or PR.

---

## 💡 Tips

### Tip 1: Use Variables for Common Values

```http
@host = api.example.com
@token = abc123

GET https://{{host}}/users
Authorization: Bearer {{token}}
```

### Tip 2: Organize Requests with Comments

```http
### ============================================
### User Management API
### ============================================

### Get all users
GET https://api.example.com/users

### Get specific user
GET https://api.example.com/users/1

### ============================================
### Post Management API
### ============================================

### Get all posts
GET https://api.example.com/posts
```

### Tip 3: Keep Environment-Specific Files

```
api.http           # Shared requests
dev.http          # Development endpoints
staging.http      # Staging endpoints
production.http   # Production endpoints
```

### Tip 4: Version Control Your Requests

`.http` files are just text - commit them to git alongside your code!

```bash
git add api.http
git commit -m "Add user authentication endpoints"
```

---

**Happy API Testing! 🚀**
