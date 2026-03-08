# REST Client Extension - Visual Usage Guide

## 📦 Installation

### Step 1: Install Extension
```
Cmd+Shift+P → "extensions" → Search "REST Client" → Click Install
```

### Step 2: Restart Zed
```
Cmd+Q (completely quit) → Reopen Zed
```

---

## ⚙️ Setup Keyboard Shortcut (Recommended)

### Step 1: Open Settings
```
Press: Cmd+, (comma)
```

### Step 2: Open Keymap
```
Click "Open Keymap" button in top right corner
```

### Step 3: Add Shortcut
Add this to your keymap file:
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

### Step 4: Save
```
Cmd+S to save the keymap file
```

**Result:** Now you can press `Ctrl+Enter` to execute requests! 🎉

---

## 🚀 Using the Extension

### Method 1: Using Keyboard Shortcut (Fastest)

#### Step 1: Create a .http file
```
File → New → Save as "test.http"
```

#### Step 2: Write a request
```http
### Get user info
GET https://jsonplaceholder.typicode.com/users/1
```

#### Step 3: Execute
```
1. Click anywhere in the request (on any line)
2. Press: Ctrl+Enter
3. See response in terminal! ✅
```

**Visual Flow:**
```
┌─────────────────────────────────────┐
│ test.http                           │
├─────────────────────────────────────┤
│ ### Get user info          ← Click here
│ GET https://api.com/users/1         │
│                            ← Or here │
└─────────────────────────────────────┘
         │
         │ Press: Ctrl+Enter
         ▼
┌─────────────────────────────────────┐
│ Terminal                            │
├─────────────────────────────────────┤
│ HTTP/1.1 200 OK                     │
│ {                                   │
│   "id": 1,                          │
│   "name": "Leanne Graham",          │
│   ...                               │
│ }                                   │
└─────────────────────────────────────┘
```

---

### Method 2: Using Command Palette (No Setup)

#### Step 1: Create a .http file
Same as above

#### Step 2: Write a request
Same as above

#### Step 3: Execute via Command Palette
```
1. Click anywhere in the request
2. Press: Cmd+Shift+P
3. Type: "task spawn"
4. Select: "Send Request at Cursor"
5. See response in terminal! ✅
```

---

## 📝 Request Examples

### Simple GET Request
```http
### Get all users
GET https://jsonplaceholder.typicode.com/users
```

### POST with JSON Body
```http
### Create a post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "My Post",
  "body": "Post content",
  "userId": 1
}
```

### Request with Headers
```http
### Authenticated request
GET https://api.example.com/data
Authorization: Bearer your-token-here
Accept: application/json
```

### Multiple Requests in One File
```http
### Request 1
GET https://jsonplaceholder.typicode.com/users/1

### Request 2
GET https://jsonplaceholder.typicode.com/users/2

### Request 3
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "Test"
}
```

**Tip:** Click anywhere in any request block and execute - only that request runs!

---

## 💡 Quick Reference

```
┌─────────────────────────────────────────────────────┐
│         REST Client - Quick Reference               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  🔥 EXECUTE REQUEST                                 │
│     Ctrl+Enter          (after keymap setup)        │
│     OR                                              │
│     Cmd+Shift+P → "task spawn" → Send Request      │
│                                                     │
│  📂 FILE TYPES                                      │
│     .http, .rest                                    │
│                                                     │
│  📝 REQUEST FORMAT                                  │
│     ### Comment                                     │
│     METHOD URL                                      │
│     Header: value                                   │
│                                                     │
│     { body }                                        │
│                                                     │
│  🔀 SEPARATOR                                       │
│     ###                                             │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

**Happy API Testing! 🚀**
