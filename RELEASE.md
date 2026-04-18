# First Release Checklist — MacroSpend Tracker v0.1.0

Ship an unsigned beta to ~10 internal users in roughly 1 working day.
This checklist assumes the `tauri-tracker/` scaffold from this repo has been moved to its own GitHub repository.

> Estimated time: 4-6 hours of focused work + 15 min CI build.

---

## 0. Prerequisites

You will need:
- A GitHub account with permissions to create a public repository
- A macOS machine (Intel or Apple Silicon) for local testing
- A Windows 10/11 machine (or VM) for local testing
- Node 20+, Git
- ~30 min for one-time toolchain installs per OS

---

## Step 1 — Move the scaffold to its own repo (10 min)

```bash
# From the web-app project root
cp -R tauri-tracker ~/macrospend-tracker
cd ~/macrospend-tracker

git init -b main
git add .
git commit -m "Initial Tauri tracker scaffold"

# Create the GitHub repo (using the gh CLI)
gh repo create macrospend-tracker --public --source=. --remote=origin --push
```

**Then delete `tauri-tracker/` from the web app repo** to avoid confusion.

---

## Step 2 — Local toolchain installs (one-time, 20-30 min per OS)

### macOS
```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
brew install node           # or use nvm
```

### Windows (in elevated PowerShell)
```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools `
  --override "--passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install Rustlang.Rustup
winget install OpenJS.NodeJS.LTS
# WebView2 is preinstalled on Win 11; on Win 10 install from microsoft.com/edge/webview2
```

After install, in a fresh shell on either OS:
```bash
node -v && npm -v && rustc -V && cargo -V
```

---

## Step 3 — Generate the Tauri updater signing key (5 min)

This is **NOT** the OS code-signing cert. It is a separate Ed25519 keypair Tauri uses to verify update bundles.
Without it, auto-update is disabled but the app still works.

```bash
cd ~/macrospend-tracker
npm install
npm run tauri signer generate -- -w ~/.tauri/macrospend.key
```

You will be prompted for a password. Choose a strong one and store it in 1Password.

The command prints a **public key** (a base64 blob ~120 chars). Copy it.

Open `tauri.conf.json` and replace the placeholder:
```diff
- "pubkey": "REPLACE_WITH_OUTPUT_OF_npm_run_tauri_signer_generate"
+ "pubkey": "<paste public key here>"
```

Also replace the org placeholder:
```diff
- "endpoints": ["https://github.com/REPLACE_ORG/macrospend-tracker/releases/latest/download/latest.json"]
+ "endpoints": ["https://github.com/<your-org>/macrospend-tracker/releases/latest/download/latest.json"]
```

Commit:
```bash
git add tauri.conf.json && git commit -m "Configure updater key + endpoint"
```

> **Backup**: The contents of `~/.tauri/macrospend.key` are irreplaceable. If lost, every existing install must be reinstalled. Save the file + password in 1Password as a secure note.

---

## Step 4 — Set GitHub secrets (5 min)

In your GitHub repo: **Settings → Secrets and variables → Actions → New repository secret**.

| Secret name | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Full contents of `~/.tauri/macrospend.key` (`cat ~/.tauri/macrospend.key`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The password you chose in Step 3 |

`GITHUB_TOKEN` is provided automatically by Actions. You do not need to add it.

---

## Step 5 — Add app icons (10 min)

Place a 1024x1024 source PNG (e.g. the MacroSpend logo on a transparent background) at `src-tauri/icons/source.png`, then:

```bash
cd ~/macrospend-tracker
npm run tauri icon src-tauri/icons/source.png
```

This generates every required size including `tray.png`, `icon.icns`, and `icon.ico`.

Commit:
```bash
git add src-tauri/icons && git commit -m "Add app icons"
```

---

## Step 6 — Verify local dev (10 min)

```bash
npm run tauri dev
```

You should see:
- Tray icon appears in the menu bar (Mac) or system tray (Windows)
- Right-click → menu shows Open / Pause / Resume / Check for updates / Quit
- Left-click opens the small status window

Quit with the tray menu, not Ctrl+C, to avoid orphan processes.

---

## Step 7 — Cut v0.1.0 (2 min)

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers `.github/workflows/release.yml`. Watch it run:
```bash
gh run watch
```

CI burns ~12-15 min producing:
- `MacroSpend-Tracker_0.1.0_universal.dmg` (Mac, both architectures)
- `MacroSpend-Tracker_0.1.0_x64_en-US.msi` (Windows)
- `latest.json` (signed updater manifest)
- `*.sig` files (one per installer)

---

## Step 8 — Verify the GitHub Release (5 min)

1. Open `https://github.com/<your-org>/macrospend-tracker/releases`
2. Confirm the v0.1.0 release is **draft** (the workflow creates drafts so you can review before publishing)
3. Confirm all 4 artifacts above are attached
4. Open `latest.json` in the browser and verify it contains:
   - `"version": "0.1.0"`
   - `"platforms"` with `darwin-aarch64`, `darwin-x86_64`, `windows-x86_64` entries
   - Each entry has a `signature` (base64) and `url`
5. Click **Publish release**

---

## Step 9 — Test install on macOS (5 min)

1. Download the `.dmg` from the release page
2. Open it, drag MacroSpend Tracker to Applications
3. **First launch will be blocked** by Gatekeeper. To open:
   - Open Finder → Applications
   - Right-click MacroSpend Tracker → **Open**
   - Click **Open** in the warning dialog
4. The tray icon should appear in the menu bar within 2 seconds
5. macOS may prompt for accessibility permission. **Do not approve yet** — the v0.1.0 sampler reads app names only via NSWorkspace, which does not need that permission

Verify with logs:
```bash
log stream --predicate 'subsystem == "com.macrospend.tracker"' --info
```

---

## Step 10 — Test install on Windows (5 min)

1. Download the `.msi` from the release page
2. Double-click. **SmartScreen will warn** — click **More info → Run anyway**
3. Step through the installer (Next → Next → Install)
4. The tray icon should appear in the system tray (may be hidden in the overflow caret `^`)

Verify it is running:
```powershell
Get-Process | Where-Object {$_.ProcessName -like "macrospend*"}
```

---

## Step 11 — Test deep-link registration (10 min)

This is the magic flow: clicking a link in the web app registers the device without any copy-paste.

1. In the web app, sign in and open `/tracker`
2. Click **Register a device** → choose OS → click **Open in desktop app**
   *(This is the new button added in Step 12 of the web wiring below. If your web app has not been deployed yet, manually craft a test URL):*
   ```
   macrospend://register?key=<api_key_from_tracked_devices_table>
   ```
3. The OS will ask "Open MacroSpend Tracker?" → click **Open**
4. The tracker stores the key in the OS keychain and starts sampling
5. Confirm in the tracker's status window: dot turns green, "Connected"

If the deep link does not open the app:
- **macOS**: confirm the app is in `/Applications` (not just running from a download folder). Try `open "macrospend://register?key=test"` in Terminal
- **Windows**: re-run the installer (the URI registration happens during MSI install)

---

## Step 12 — Verify first sync into `usage_events` (5 min)

After registration, the tracker samples every 60s and flushes every 5 min.
Wait 5-7 minutes after first sync, then in the web app's database:

```sql
select count(*), min(recorded_at), max(recorded_at)
from usage_events
where device_id = '<your test device id>';
```

You should see at least 5 rows. If empty after 10 min:
- Check the tracker log:
  - Mac: `~/Library/Logs/com.macrospend.tracker/`
  - Win: `%APPDATA%\com.macrospend.tracker\logs\`
- Common cause: device key not saved (deep link did not fire). Re-trigger from `/tracker`

---

## Step 13 — Verify tracker health in the web app (2 min)

1. Open `/tracker` in the web app
2. Scroll to **Tracker health**
3. Your device should appear with:
   - Green dot (synced in last 10 min)
   - Event count > 0 in the last 24h column
   - OS icon matching your machine
4. Open the install funnel widget — you should see counts: downloaded → installed → registered → first sync

---

## You are done.

Beta-ready installers are now live at the GitHub Release page.
Share the URL with your beta users along with this one-liner:

> macOS: download the .dmg, drag to Applications, **right-click → Open** the first time.
> Windows: download the .msi, **More info → Run anyway** when SmartScreen warns.
> After install, return to /tracker and click **Open in desktop app** to register.

---

## Phase 6 (later) — Code signing

When you are ready to remove the Gatekeeper / SmartScreen warnings, see `PLAN.md` Phase 6.
You will need:
- $99/year Apple Developer Program enrollment
- $200-700/year Windows code signing cert
- Add 6 more GitHub secrets (Apple cert, Apple ID, team ID, password, Win cert, Win password)

Until then, document the right-click-Open and SmartScreen workarounds prominently in your install instructions.
