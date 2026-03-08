# REST Client - Animated Workflow Guide

## 🎬 Setup Keymap (One-Time)

```
Step 1: Open Settings
┌─────────────────────────────┐
│ Press: Cmd + ,              │
└─────────────────────────────┘
         ▼
Step 2: Click "Open Keymap"
┌─────────────────────────────┐
│ Zed Settings                │
│ ┌─────────────────────────┐ │
│ │   [Open Keymap]  ◄─────┼─┼─ Click this
│ └─────────────────────────┘ │
└─────────────────────────────┘
         ▼
Step 3: Add this code
┌─────────────────────────────────────────┐
│ keymap.json                             │
├─────────────────────────────────────────┤
│ [                                       │
│   {                                     │
│     "context": "Editor",                │
│     "bindings": {                       │
│       "ctrl-enter": [                   │
│         "task::Spawn",                  │
│         {                               │
│           "task_name":                  │
│             "Send Request at Cursor"    │
│         }                               │
│       ]                                 │
│     }                                   │
│   }                                     │
│ ]                                       │
└─────────────────────────────────────────┘
         ▼
Step 4: Save
┌─────────────────────────────┐
│ Press: Cmd + S              │
└─────────────────────────────┘

✅ Done! Now Ctrl+Enter executes requests
```

---

## 🚀 Execute Request (Every Time)

```
Step 1: Create .http file
┌─────────────────────────────┐
│ File → New                  │
│ Save as: "test.http"        │
└─────────────────────────────┘
         ▼
Step 2: Write request
┌──────────────────────────────────────────┐
│ test.http                                │
├──────────────────────────────────────────┤
│                                          │
│ ### Get user info                        │
│ GET https://api.example.com/users/1      │
│                                          │
└──────────────────────────────────────────┘
         ▼
Step 3: Click anywhere in request
┌──────────────────────────────────────────┐
│ test.http                                │
├──────────────────────────────────────────┤
│                                          │
│ ### Get user info   ◄─────── Click here │
│ GET https://...     ◄─────── Or here    │
│                     ◄─────── Or here    │
└──────────────────────────────────────────┘
         ▼
Step 4: Press Ctrl+Enter
┌─────────────────────────────┐
│                             │
│    Press: Ctrl + Enter      │
│                             │
└─────────────────────────────┘
         ▼
Step 5: See response in terminal!
┌──────────────────────────────────────────┐
│ Terminal                                 │
├──────────────────────────────────────────┤
│ Executing request at line 2...           │
│                                          │
│ HTTP/1.1 200 OK                          │
│ Content-Type: application/json           │
│                                          │
│ {                                        │
│   "id": 1,                               │
│   "name": "Leanne Graham",               │
│   "username": "Bret",                    │
│   "email": "Sincere@april.biz"           │
│ }                                        │
│                                          │
│ ✅ Request completed in 245ms            │
└──────────────────────────────────────────┘
```

---

## 🎯 Alternative: Use Command Palette

```
Step 1-2: Same as above (create file, write request)
         ▼
Step 3: Open Command Palette
┌─────────────────────────────┐
│ Press: Cmd + Shift + P      │
└─────────────────────────────┘
         ▼
Step 4: Type "task spawn"
┌──────────────────────────────────────────┐
│ 🔍 task spawn                            │
├──────────────────────────────────────────┤
│                                          │
│ ▶ task: Spawn                            │
│                                          │
└──────────────────────────────────────────┘
         ▼
Step 5: Select "Send Request at Cursor"
┌──────────────────────────────────────────┐
│ Select a task to spawn:                  │
├──────────────────────────────────────────┤
│                                          │
│ ▶ Send Request at Cursor  ◄───── Select │
│   cargo build                            │
│   cargo test                             │
│                                          │
└──────────────────────────────────────────┘
         ▼
Response appears in terminal (same as above)
```

---

## 📝 Multiple Requests Example

```
┌──────────────────────────────────────────┐
│ api.http                                 │
├──────────────────────────────────────────┤
│                                          │
│ ### Get all users         ◄─── Request 1│
│ GET https://api.example.com/users        │
│                                          │
│ ### Get user by ID        ◄─── Request 2│
│ GET https://api.example.com/users/1      │
│                                          │
│ ### Create new user       ◄─── Request 3│
│ POST https://api.example.com/users       │
│ Content-Type: application/json           │
│                                          │
│ {                                        │
│   "name": "John Doe"                     │
│ }                                        │
│                                          │
└──────────────────────────────────────────┘

To execute Request 1:
  1. Click anywhere in lines 3-4
  2. Press Ctrl+Enter

To execute Request 2:
  1. Click anywhere in lines 6-7
  2. Press Ctrl+Enter

To execute Request 3:
  1. Click anywhere in lines 9-15
  2. Press Ctrl+Enter

Each request executes independently! 🎯
```

---

## 🎨 Request Anatomy

```
┌──────────────────────────────────────────┐
│                                          │
│  ### Comment or description              │
│  ▲                                       │
│  └─ Optional: Describes the request      │
│                                          │
│  GET https://api.example.com/endpoint    │
│  ▲   ▲                                   │
│  │   └─ URL (required)                   │
│  └─ HTTP Method (required)               │
│                                          │
│  Authorization: Bearer token123          │
│  Content-Type: application/json          │
│  ▲                                       │
│  └─ Headers (optional, one per line)     │
│                                          │
│  {                                       │
│    "key": "value"                        │
│  }                                       │
│  ▲                                       │
│  └─ Request Body (optional)              │
│                                          │
│  ###  ◄─── Separator (starts new request)│
│                                          │
└──────────────────────────────────────────┘
```

---

## ⚡ Pro Tips

### Tip 1: Click Anywhere
```
┌──────────────────────────────────────────┐
│ ### Get user    ◄─── ✅ Works            │
│                 ◄─── ✅ Works            │
│ GET https://... ◄─── ✅ Works            │
│                 ◄─── ✅ Works            │
│ Header: value   ◄─── ✅ Works            │
│                 ◄─── ✅ Works            │
│ { "body": 1 }   ◄─── ✅ Works            │
│                 ◄─── ✅ Works            │
│ ###             ◄─── ❌ Don't click here │
└──────────────────────────────────────────┘

Click anywhere EXCEPT the ### separator!
```

### Tip 2: Quick Iteration
```
Write request → Ctrl+Enter → See response
     ▲                              │
     │                              │
     └──────── Modify ──────────────┘

No need to restart Zed! Just edit and re-execute.
```

### Tip 3: Organize Multiple Requests
```
api.http
  ├─ ### User API
  │   ├─ GET /users
  │   ├─ GET /users/:id
  │   └─ POST /users
  │
  ├─ ### Post API
  │   ├─ GET /posts
  │   └─ POST /posts
  │
  └─ ### Auth API
      └─ POST /login
```

---

## 🐛 Troubleshooting

### Problem: "No HTTP request found at line X"
```
❌ Bad:
┌──────────────────────────────┐
│ ### Comment                  │
│                              │ ◄─ Cursor here (empty line)
│ GET https://...              │
└──────────────────────────────┘

✅ Good:
┌──────────────────────────────┐
│ ### Comment                  │
│ GET https://...              │ ◄─ Cursor here
└──────────────────────────────┘
```

### Problem: Shortcut not working
```
Checklist:
□ Did you save keymap.json? (Cmd+S)
□ Did you restart Zed? (Cmd+Q, reopen)
□ Are you in a .http or .rest file?
□ Try Command Palette method first
```

---

## 📊 Full Workflow Diagram

```
┌────────────────────────────────────────────────────┐
│                   ONE-TIME SETUP                   │
└────────────────────────────────────────────────────┘
                        │
                        ▼
         ┌──────────────────────────┐
         │   Setup Keyboard Shortcut│
         │   (Cmd+, → Open Keymap)  │
         └──────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────┐
│                  DAILY WORKFLOW                    │
└────────────────────────────────────────────────────┘
                        │
         ┌──────────────┴──────────────┐
         │                             │
         ▼                             ▼
┌─────────────────┐          ┌─────────────────┐
│ Create .http    │          │ Or open existing│
│ file            │          │ .http file      │
└─────────────────┘          └─────────────────┘
         │                             │
         └──────────────┬──────────────┘
                        ▼
         ┌──────────────────────────┐
         │   Write HTTP request     │
         └──────────────────────────┘
                        │
                        ▼
         ┌──────────────────────────┐
         │   Click in request       │
         └──────────────────────────┘
                        │
                        ▼
         ┌──────────────────────────┐
         │   Press Ctrl+Enter       │
         └──────────────────────────┘
                        │
                        ▼
         ┌──────────────────────────┐
         │   See response!          │
         └──────────────────────────┘
                        │
         ┌──────────────┴──────────────┐
         │                             │
         ▼                             ▼
┌─────────────────┐          ┌─────────────────┐
│ Modify request  │          │ Try another     │
│ Re-execute      │          │ request         │
└─────────────────┘          └─────────────────┘
         │                             │
         └──────────────┬──────────────┘
                        │
                        ▼
         ┌──────────────────────────┐
         │   Done! 🎉               │
         └──────────────────────────┘
```

---

**Created with ❤️ for Zed users**
