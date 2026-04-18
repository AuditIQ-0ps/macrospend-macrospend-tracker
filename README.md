# MacroSpend Tracker — Desktop Agent (Tauri)

Native macOS + Windows tray app that replaces the Python tracker.
Same privacy model: only active app name, window title, duration. No keystrokes, no screenshots, no file content.

## Why Tauri (not Electron)

- ~3 MB installer vs ~80 MB Electron
- Native system tray, deep links, autostart out of the box
- Built-in updater with Ed25519 signature verification
- Rust core means no Python dependency on user machines

## Architecture

```
┌──────────────────────────────────────────────────┐
│ Tray icon (always visible)                       │
│   ├─ Status: Running / Paused / Disconnected     │
│   ├─ Pause / Resume                              │
│   ├─ Open MacroSpend dashboard                   │
│   ├─ Check for updates                           │
│   └─ Quit                                        │
└──────────────────────────────────────────────────┘
            │
            │ every 60s
            ▼
┌──────────────────────────────────────────────────┐
│ Sampler (Rust)                                   │
│   macOS: AppKit NSWorkspace.frontmostApplication │
│   Win:   GetForegroundWindow + QueryFullProcess  │
└──────────────────────────────────────────────────┘
            │
            │ batch every 5 min
            ▼
┌──────────────────────────────────────────────────┐
│ POST tracker-ingest                              │
│   X-Device-Key: <stored in OS keychain>          │
└──────────────────────────────────────────────────┘
```

## Folder layout

```
tauri-tracker/
├── README.md                 # this file
├── PLAN.md                   # full delivery plan + timeline
├── package.json              # tauri CLI + minimal frontend
├── tauri.conf.json           # bundle, updater, deep-link config
├── src/                      # tray HTML/JS UI (60 lines)
│   └── index.html
├── src-tauri/
│   ├── Cargo.toml            # Rust deps
│   ├── tauri.conf.json       # → see root tauri.conf.json
│   ├── build.rs
│   ├── icons/                # add app icons here (see below)
│   └── src/
│       ├── main.rs           # tray, deep link, lifecycle
│       ├── sampler.rs        # platform active-window sampler
│       ├── ingest.rs         # batched HTTP sender
│       ├── storage.rs        # device key in OS keychain
│       └── deeplink.rs       # macrospend://register?key=...
└── .github/workflows/
    └── release.yml           # cross-build mac + win, attach to GH Release
```

## Local development

Prereqs (one-time):
- **Rust**: https://rustup.rs
- **Node 18+**
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Visual Studio Build Tools 2022 (Desktop C++ workload), WebView2 Runtime

```bash
cd tauri-tracker
npm install
npm run tauri dev          # launches the tray app locally
```

## Building unsigned beta installers

```bash
npm run tauri build
# Output:
#   macOS  → src-tauri/target/release/bundle/dmg/MacroSpend Tracker_0.1.0_x64.dmg
#   Win    → src-tauri/target/release/bundle/msi/MacroSpend Tracker_0.1.0_x64_en-US.msi
```

## Beta install warnings (unsigned)

**macOS** — Gatekeeper will block on first launch:
1. Right-click the app in Applications → **Open**
2. Click **Open** again in the warning dialog
3. After this, it launches normally

**Windows** — SmartScreen will warn:
1. Click **More info** in the blue popup
2. Click **Run anyway**

These warnings disappear once you add real signing certs (see PLAN.md).

## Deep-link registration

The web app links to `macrospend://register?key=<api_key>`.
The tracker stores the key in the OS keychain (Keychain on Mac, Credential Manager on Win) and starts sampling immediately. No copy-paste of UUIDs.

## Auto-update

On launch the app checks `https://github.com/<your-org>/macrospend-tracker/releases/latest/download/latest.json` and prompts the user if a new version is available. Bundles are signed with Ed25519 (Tauri's own signing, separate from OS code-signing).

See `PLAN.md` for the full delivery checklist.
