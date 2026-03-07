# 🚀 5-Minute Setup Guide

## What You'll Get

Press `Ctrl+Enter` in any `.http` file to send the request at your cursor.

---

## Setup Steps

### 1. Install Extension (if not done)
```bash
cd /Users/tongzhou/poc/startup/zed/restclient
./install_to_zed.sh
```

### 2. Restart Zed
- Press `Cmd+Q` to quit
- Reopen Zed

### 3. Add Keyboard Shortcut

**a)** Press `Cmd+,` to open settings

**b)** Click **"Open Keymap"** (top right)

**c)** Add this to the file:
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

**d)** Save the file (`Cmd+S`)

### 4. Test It

**a)** Create `test.http`:
```http
### Test request
GET https://httpbin.org/get
```

**b)** Click anywhere in the request

**c)** Press `Ctrl+Enter`

**d)** See the response in terminal!

---

## Done! 🎉

Now in any `.http` file:
1. Place cursor in request
2. Press `Ctrl+Enter`
3. Get response

---

## Alternative (No Keyboard Shortcut)

If you don't want to set up the shortcut:

1. Place cursor in request
2. `Cmd+Shift+P` → Type "task spawn"
3. Select "Send Request at Cursor"

The keyboard shortcut is highly recommended though!

---

## Troubleshooting

**Ctrl+Enter doesn't work:**
- Make sure you're in a `.http` or `.rest` file
- Restart Zed after adding the keymap
- Check the file extension is `.http` not `.txt`

**Task not found:**
- Run `./install_to_zed.sh` again
- Restart Zed completely (`Cmd+Q`)
- Check extension is installed: `Cmd+Shift+P` → "extensions"

---

See **CURSOR_BASED.md** for full documentation.
