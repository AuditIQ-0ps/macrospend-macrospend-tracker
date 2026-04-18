# Delivery Plan — MacroSpend Tracker (Tauri)

## TL;DR

| Phase | Effort | Output |
|---|---|---|
| 1. Move scaffold to its own repo | 30 min | `github.com/<org>/macrospend-tracker` |
| 2. Local dev working on Mac + Win | 1 day | Tray app sends events to staging |
| 3. CI builds unsigned installers | 1 day | `.dmg` + `.msi` attached to GitHub Releases |
| 4. Deep-link + updater wired | 1 day | One-click install from /tracker |
| 5. Beta with ~10 users | 1 week | Real-world bug surface |
| 6. Code signing + notarization | 1 week + cert wait | No more Gatekeeper/SmartScreen |
| **Total to public-ready** | **~3 weeks + cert procurement** | |

## Phase 1 — Move to its own repo

This scaffold lives inside the web app for convenience. **It does not belong here long-term** — Tauri builds need GitHub Actions runners with macOS + Windows, separate from your web Vercel/Lovable deploy.

```bash
# From the tauri-tracker/ folder:
mkdir ~/macrospend-tracker && cp -R . ~/macrospend-tracker/
cd ~/macrospend-tracker && git init && gh repo create macrospend-tracker --public --source=.
```

Then **delete `tauri-tracker/` from the web app repo**.

## Phase 2 — Local dev

Verify each platform once before touching CI.

**Mac**
```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd macrospend-tracker && npm install && npm run tauri dev
```

**Windows** (in PowerShell)
```powershell
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools  # add "Desktop development with C++"
cd macrospend-tracker; npm install; npm run tauri dev
```

Confirm: tray icon appears, "Pause" stops the sampler, events arrive in `usage_events` table when you set a real device key.

## Phase 3 — CI: unsigned builds

`.github/workflows/release.yml` (already scaffolded) runs on `git push --tags` and produces:
- `MacroSpend-Tracker_<v>_x64.dmg` (Mac Intel)
- `MacroSpend-Tracker_<v>_aarch64.dmg` (Apple Silicon)
- `MacroSpend-Tracker_<v>_x64_en-US.msi` (Windows)
- `latest.json` (updater manifest, signed with Ed25519)

To cut a release:
```bash
git tag v0.1.0 && git push --tags
```

GitHub Actions burns ~15 min per release.

### Required GitHub secrets (for unsigned builds)

| Secret | Purpose |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 key for updater bundle signatures (≠ OS code signing). Generate once: `npm run tauri signer generate`. Store the **private** key here, the **public** key in `tauri.conf.json`. |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password you set when generating. |

**Without these, auto-update will not work** — but the installers themselves still build fine.

## Phase 4 — Web app wiring

Add to the web `/tracker` page (separate task, ask me to build it):

1. **"Download desktop app" CTA** — links to latest GitHub Release `.dmg` / `.msi` based on `navigator.userAgent`.
2. **"Register this device" deep link** — after the user installs, page generates a device key and opens `macrospend://register?key=<key>&org=<id>`. Tauri picks it up via the registered protocol handler.
3. **Fallback** — keep the Python script for Linux + edge cases, demoted below the desktop app.

Estimated: 1 React file change (~80 lines) in `src/components/tracker/DeviceRegistration.tsx`.

## Phase 5 — Beta

Ship to ~10 internal users. Collect:
- Crash logs (Tauri writes to `~/Library/Logs/com.macrospend.tracker/` on Mac, `%APPDATA%\com.macrospend.tracker\logs\` on Win)
- Install friction reports (the Gatekeeper/SmartScreen workaround is the main one)
- Battery impact on laptops (sampler should be <0.1% CPU)

## Phase 6 — Code signing (optional, but real beta needs this)

### macOS — Apple Developer Program

| Item | Cost | Time |
|---|---|---|
| Apple Developer Program enrollment | $99/year | 1-2 days approval |
| Developer ID Application certificate | included | instant after enrollment |
| App-specific password for notarization | included | instant |

GitHub secrets to add:
- `APPLE_CERTIFICATE` (base64 of `.p12`)
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY` (e.g. `Developer ID Application: Your Co (TEAMID)`)
- `APPLE_ID`
- `APPLE_PASSWORD` (app-specific)
- `APPLE_TEAM_ID`

Tauri's bundler then signs + notarizes automatically.

### Windows — Code Signing Certificate

| Option | Cost | Trust |
|---|---|---|
| Standard OV cert (Sectigo, DigiCert) | $200-400/year | Builds reputation slowly; SmartScreen warning persists for first ~thousand installs |
| EV cert | $400-700/year + USB token | Instant SmartScreen trust, no warning |

For <500 beta users, **OV is fine**. EV only matters if you're shipping at scale.

GitHub secrets:
- `WINDOWS_CERTIFICATE` (base64 of `.pfx`)
- `WINDOWS_CERTIFICATE_PASSWORD`

### What I (the AI) cannot do

- Buy or store certificates
- Run `tauri signer generate` (you must, locally — it produces a private key)
- Notarize from the sandbox (needs Apple credentials)
- Push tags to your GitHub

Everything above is documented; you run it once.

## Risks & open questions

1. **Tray icons on Windows 11** sometimes hide in the overflow menu. Document this for users.
2. **Ed25519 key rotation** — if you lose `TAURI_SIGNING_PRIVATE_KEY`, every installed client must reinstall manually. Back it up offline.
3. **macOS Sequoia accessibility prompt** — sampling the active window via NSWorkspace doesn't need accessibility, but reading window *titles* does. The scaffold uses NSWorkspace only (app name only) by default; flip a flag to enable title sampling.
4. **Windows ARM64** — not built. Add later if needed.

## Decision log

- **2026-04-17**: Scaffold lives in web repo temporarily. Move to own repo before any CI runs.
- **2026-04-17**: Beta ships unsigned. Add Apple + Windows certs before public launch.
- **2026-04-17**: Updater feed = GitHub Releases (free, standard). Re-evaluate at >1k DAU.
