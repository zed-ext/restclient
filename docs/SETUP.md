# Quick Setup Guide

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Zed IDE](https://zed.dev/)

## Setup Steps

### 1. Clone and Install

```bash
git clone https://github.com/anthropics/zed-restclient.git
cd zed-restclient
./install_to_zed.sh
```

### 2. Restart Zed

Completely quit Zed (`Cmd+Q` on macOS) and reopen it.

### 3. Test It

Create a file called `test.http`:

```http
### Test request
GET https://httpbin.org/get
```

You should see **▶ Send Request** and **⊕ Send in New Tab** buttons above the request. Click either one to execute.

## Troubleshooting

**▶ buttons don't appear:**
- Restart Zed completely (`Cmd+Q` and reopen)
- Check LSP is running: View → Server Logs → HTTP LSP
- Reinstall: `./install_to_zed.sh`

**Syntax highlighting not working:**
- Check file extension is `.http` or `.rest`
- Restart Zed completely

See the [README](../README.md) for full documentation.
