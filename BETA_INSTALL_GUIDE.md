# MacroSpend Tracker — Beta Install Guide

A short guide for the controlled pilot rollout of the desktop tracker.
Two audiences: **admins** running the pilot, and **employees** installing the app.

---

## Who needs what

| Role | Dashboard access | Tracker install |
|---|---|---|
| **Admin / decision-maker** | Yes — runs the pilot, reviews findings | Optional (only if you also want your own usage measured) |
| **Employee in the pilot** | No — they never sign in to MacroSpend | Yes — one-time install |
| **Employee NOT in the pilot** | No | No — leave them alone |

The tracker is **opt-in per device**. An employee outside the pilot does not need an account, an install, or any awareness of MacroSpend. Keep the beta narrow (5-15 machines) so you can debug install issues fast.

---

## For admins — running the pilot (10 min)

1. Sign in to the dashboard at https://macrospend.com.
2. Open **Tracker** in the sidebar.
3. Click **Register a device** for each pilot user. You'll get:
   - A download link for their OS (.dmg or .msi)
   - A one-click "Open in desktop app" deep link that registers their device automatically
4. Send each pilot user the **employee install instructions** below, along with their personal deep link.
5. Within 10 min of install, their device shows up in **Tracker health** with a green dot. If it doesn't, see [Troubleshooting](#troubleshooting).
6. After 24h, the **Install funnel** widget shows your conversion: downloaded → installed → registered → first sync.

### Admin checklist before inviting users
- [ ] You've personally installed the tracker on one of your own devices and confirmed first sync
- [ ] At least one Mac and one Windows install has been verified end-to-end
- [ ] You've decided who is in the pilot (5-15 people max for beta)
- [ ] You've told pilot users **what is collected** (app name, window title, duration) and **what is not** (keystrokes, screenshots, file contents)

---

## For employees — installing the tracker (5 min)

You should have received a download link and an "Open in desktop app" link from your admin.

### macOS
1. Click the **Download for Mac** link — saves `MacroSpend-Tracker_universal.dmg`
2. Open the .dmg, drag MacroSpend Tracker to **Applications**
3. **First launch:** open Finder → Applications → **right-click MacroSpend Tracker → Open** → click **Open** in the warning dialog
   *(This bypasses Gatekeeper. Only needed once. The app is unsigned during beta.)*
4. The tray icon (a small dot) appears in your menu bar within 2 seconds
5. Click your admin's **"Open in desktop app"** link — macOS asks "Open MacroSpend Tracker?" → click **Open**
6. The tray icon's status window should now show **Connected** (green dot)

### Windows
1. Click the **Download for Windows** link — saves `MacroSpend-Tracker_x64_en-US.msi`
2. Double-click the .msi. **SmartScreen warning** appears → click **More info → Run anyway**
   *(The app is unsigned during beta. Only needed once.)*
3. Step through the installer (Next → Next → Install)
4. The tray icon appears in your system tray (may be hidden under the `^` overflow caret)
5. Click your admin's **"Open in desktop app"** link — Windows asks "Open MacroSpend Tracker?" → click **Open**
6. The tray icon's status window should now show **Connected** (green dot)

### What it does and doesn't collect

✅ Captures: name of the active app (e.g. "Slack"), window title (e.g. "general - Acme Corp"), duration in seconds
❌ Never captures: keystrokes, mouse clicks, screenshots, file contents, audio, video, web page contents, passwords

You can pause sampling any time from the tray menu (Pause sampling). You can quit the app entirely from the same menu.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Tray icon never appears after install | App didn't launch | macOS: open Applications, right-click → Open. Windows: search "MacroSpend Tracker" in Start menu and run it. |
| Deep link does nothing | App not in standard install location | macOS: confirm app is in `/Applications` (not Downloads). Windows: re-run the .msi installer. |
| Status shows "Disconnected" after registration | Network issue or stale device key | Open the tray menu → Quit → relaunch. Click the "Open in desktop app" link again. |
| Admin sees device in dashboard but no usage data after 10 min | First flush takes up to 5 min | Wait. If still empty after 15 min, check tracker logs (Mac: `~/Library/Logs/com.macrospend.tracker/`, Win: `%APPDATA%\com.macrospend.tracker\logs\`). |
| Device shows "Stale" in Tracker health | Last sync > 24h ago | Employee's machine has been off, or app was quit. Ask them to relaunch. |

---

## Removing the tracker

**Employee uninstall:**
- macOS: drag MacroSpend Tracker from Applications to Trash
- Windows: Settings → Apps → MacroSpend Tracker → Uninstall

**Admin revocation (kills the device's API key immediately):**
1. Open **Tracker** in the dashboard
2. Find the device in **Tracker health**
3. Click **Revoke** — the device can no longer push events even if the app is still installed

Stale devices (no sync in 24h) are automatically logged in the install funnel as `device_stale` for visibility, but not auto-revoked.

---

## Beta scope reminder

This is a **controlled pilot**, not company-wide deployment. Expected scale: 5-15 devices.
For general public launch, we still need code-signing certificates (so the Gatekeeper / SmartScreen workarounds disappear) and a published privacy policy. Until then, keep the rollout to people who have agreed to be on the beta.
