# ✨ Cursor-Based HTTP Request Execution

## 🎯 The Best Way to Send Requests in Zed

**No buttons needed!** Just place your cursor anywhere in a request and run one command.

---

## 🚀 How It Works

1. **Open your `.http` file** (e.g., `test.http`)
2. **Click anywhere** in the request you want to send
3. **Press `Ctrl+Enter`** (after setting up keyboard shortcut)
4. **Done!** Response appears in terminal

---

## ⚡ RECOMMENDED: Direct Keyboard Shortcut

**Skip the command palette entirely!** Set up `Ctrl+Enter` to directly execute the request at your cursor:

### Setup (One Time Only):

1. Press `Cmd+,` to open Zed settings
2. Click **"Open Keymap"** in the top right
3. Add this to your keymap.json:

```json
[
  {
    "context": "Editor && (path_extension(http) || path_extension(rest))",
    "bindings": {
      "ctrl-enter": ["task::Spawn", {"task_name": "Send Request at Cursor"}]
    }
  }
]
```

4. Save the file

### Usage:

Now in any `.http` file:
1. Place cursor anywhere in a request
2. Press `Ctrl+Enter`
3. Request executes immediately!

**No command palette, no typing, no selection needed!**

---

## Alternative: Command Palette Method

If you prefer not to use keyboard shortcuts:

1. **Open your `.http` file** (e.g., `test.http`)
2. **Click anywhere** in the request you want to send
3. **Press `Cmd+Shift+P`** → Type "task spawn"
4. Select "Send Request at Cursor"
5. **Done!** Response appears in terminal

---

## 📖 Example

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

**To send the POST request:**
- Click anywhere on line 6, 7, 8, 9, 10, or 11 (inside the POST block)
- Press `Ctrl+Enter` (if you set up the shortcut)
- Response appears!

---

## 🎨 How The Task Finds Your Request

The extension automatically:
1. Gets your cursor line number
2. Parses all requests in the file
3. Finds which request block contains that line
4. Executes that specific request

**No need to name requests or number them manually!**

---

## 💡 Pro Tips

### Tip 1: Customize the Shortcut

Don't like `Ctrl+Enter`? Change it to anything:

```json
[
  {
    "context": "Editor && (path_extension(http) || path_extension(rest))",
    "bindings": {
      "cmd-r": ["task::Spawn", {"task_name": "Send Request at Cursor"}]
    }
  }
]
```

Popular choices:
- `"cmd-r"` - Cmd+R (like "Run")
- `"f5"` - F5 (common in many IDEs)
- `"cmd-shift-r"` - Cmd+Shift+R
- `"alt-enter"` - Alt+Enter

### Tip 2: Request Blocks

Each request is a "block" separated by `###`:
```http
###          ← Separator (starts new block)
GET /users   ← Request block starts
             ← Still part of GET block

###          ← New separator (GET block ends, new block starts)
POST /posts  ← New request block
```

Your cursor can be **anywhere** in the block!

### Tip 3: Works with Complex Requests

```http
### Login and save token
POST https://api.example.com/auth/login
Content-Type: application/json
Authorization: Basic abc123

{
  "username": "user",
  "password": "pass"
}

> {%
  client.global.set("token", response.body.token);
%}
```

Cursor can be on any of these lines ↑ and it will execute the whole request!

---

## 🆚 Comparison with Other Methods

### Cursor-Based with Keyboard Shortcut (Best! ⭐⭐⭐)
- **How**: Place cursor → Press `Ctrl+Enter`
- **Pros**: ✅ Fastest, ✅ No typing needed, ✅ Just like IDE "Run" buttons
- **Best for**: Everyone doing API testing

### Cursor-Based with Command Palette
- **How**: Place cursor → `Cmd+Shift+P` → "task spawn" → Select task
- **Pros**: ✅ No configuration needed, ✅ Works immediately
- **Cons**: ⚠️ More steps than keyboard shortcut
- **Best for**: Quick testing without setup

### Slash Commands
- **How**: Open assistant → `/send` or `/http file.http 2`
- **Pros**: ✅ Can specify request number
- **Cons**: ❌ Requires LLM configuration
- **Best for**: If you already use Zed assistant

### Command Line
- **How**: `cargo run --bin http-execute test.http --line 15`
- **Pros**: ✅ Scriptable, ✅ CI/CD integration
- **Best for**: Automation

---

## 🐛 Troubleshooting

### "No HTTP request found at line X"

**Cause**: Your cursor is outside any request block (in comments or between separators)

**Solution**: Move cursor inside a request:
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

**Solutions**:
1. Check that you're in a `.http` or `.rest` file (not `.txt`)
2. Make sure your keymap.json has the correct context filter
3. Restart Zed after editing keymap.json
4. Check for conflicts: `Cmd+Shift+P` → "keymap" → Look for other `ctrl-enter` bindings

### Task doesn't appear in command palette

**Solution**: Just type "task spawn" first, then you'll see "Send Request at Cursor"

**Or better**: Set up the keyboard shortcut so you never need the command palette!

### Wrong request executes

**Cause**: Request blocks overlap or separator is missing

**Solution**: Make sure each request has `###` separator:
```http
### Request 1
GET /users

###              ← Need this!
### Request 2
POST /posts
```

---

## 📋 File Requirements

Your `.http` file should have `.http` or `.rest` extension:
- ✅ `test.http`
- ✅ `api.http`
- ✅ `requests.rest`
- ❌ `test.txt`
- ❌ `api.md`

---

## 🎯 Complete Workflow Example

```http
# My API Tests

### 1. Get all users
GET https://jsonplaceholder.typicode.com/users

### 2. Get specific user
GET https://jsonplaceholder.typicode.com/users/1

### 3. Create user
POST https://jsonplaceholder.typicode.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

### 4. Update user
PUT https://jsonplaceholder.typicode.com/users/1
Content-Type: application/json

{
  "name": "Jane Doe"
}

### 5. Delete user
DELETE https://jsonplaceholder.typicode.com/users/1
```

**To test:**
1. Click in request #3 (anywhere in the POST block)
2. Press `Ctrl+Enter`
3. See response in terminal
4. Click in request #5
5. Press `Ctrl+Enter`
6. Different response!

---

## ✅ Summary

**This is the closest experience to "Send" buttons without Zed having to support custom UI!**

✅ Click where you want
✅ Press one key (`Ctrl+Enter`)
✅ Get instant results
✅ No configuration needed (works via command palette)
✅ Optional shortcut for power users
✅ No file clutter

---

**Try it now:**

### Quick Start (No Setup):
1. Restart Zed
2. Open `test.http`
3. Click in any request
4. `Cmd+Shift+P` → "task spawn" → "Send Request at Cursor"

### Power User Setup:
1. `Cmd+,` → "Open Keymap"
2. Add the keyboard shortcut (see above)
3. Now just press `Ctrl+Enter` in any request!

🎉 Welcome to cursor-based HTTP testing!
