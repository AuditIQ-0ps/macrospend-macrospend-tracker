// Active-window sampler. Runs every 60s, batches every 5 min.

use crate::{ingest, AppState};
use chrono::Utc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const POLL_SECS: u64 = 60;
const FLUSH_SECS: u64 = 5 * 60;
const MAX_BATCH: usize = 100;

#[derive(serde::Serialize, Clone)]
pub struct Event {
    pub app_name: String,
    pub window_title: Option<String>,
    pub category: String,
    pub duration_seconds: u64,
    pub recorded_at: String,
}

pub async fn run_loop(handle: AppHandle) {
    let state = handle.state::<AppState>();
    let mut buffer: Vec<Event> = Vec::new();
    let mut last_flush = std::time::Instant::now();

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_SECS)).await;

        if *state.paused.read() {
            continue;
        }

        let (app_name, title) = active_window();
        let category = categorise(&app_name, title.as_deref().unwrap_or(""));
        buffer.push(Event {
            app_name,
            window_title: title,
            category,
            duration_seconds: POLL_SECS,
            recorded_at: Utc::now().to_rfc3339(),
        });

        let should_flush = last_flush.elapsed().as_secs() >= FLUSH_SECS || buffer.len() >= MAX_BATCH;
        if should_flush {
            let key_opt = state.device_key.read().clone();
            if let Some(key) = key_opt {
                let chunk: Vec<Event> = buffer.drain(..buffer.len().min(MAX_BATCH)).collect();
                match ingest::send_batch(&key, &chunk).await {
                    Ok(_) => {
                        *state.connected.write() = true;
                        last_flush = std::time::Instant::now();
                    }
                    Err(e) => {
                        *state.connected.write() = false;
                        log::warn!("ingest failed: {e}");
                        // Put events back at the front, capped
                        buffer.splice(0..0, chunk);
                        if buffer.len() > 1000 {
                            buffer.drain(0..buffer.len() - 1000);
                        }
                    }
                }
            } else {
                log::debug!("no device key set, dropping {} events", buffer.len());
                buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn active_window() -> (String, Option<String>) {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSWorkspace;
    autoreleasepool(|_| {
        let ws = unsafe { NSWorkspace::sharedWorkspace() };
        let app = unsafe { ws.frontmostApplication() };
        let name = app
            .and_then(|a| unsafe { a.localizedName() })
            .map(|n| n.to_string())
            .unwrap_or_else(|| "Unknown".into());
        // Window title sampling requires accessibility permission; omit by default.
        (name, None)
    })
}

#[cfg(target_os = "windows")]
fn active_window() -> (String, Option<String>) {
    use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return ("Unknown".into(), None);
        }
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = if len > 0 {
            Some(String::from_utf16_lossy(&title_buf[..len as usize]))
        } else {
            None
        };

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        );
        let app_name = match handle {
            Ok(h) => {
                let mut name_buf = [0u16; MAX_PATH as usize];
                let n = GetModuleBaseNameW(h, None, &mut name_buf);
                let _ = CloseHandle(h);
                if n > 0 {
                    let mut s = String::from_utf16_lossy(&name_buf[..n as usize]);
                    if let Some(stripped) = s.strip_suffix(".exe") {
                        s = stripped.to_string();
                    }
                    s
                } else {
                    "Unknown".into()
                }
            }
            Err(_) => "Unknown".into(),
        };
        (app_name, title)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn active_window() -> (String, Option<String>) {
    ("Unknown".into(), None)
}

fn categorise(app: &str, title: &str) -> String {
    let blob = format!("{app} {title}").to_lowercase();
    let rules: &[(&str, &[&str])] = &[
        ("communication", &["slack", "teams", "discord", "zoom", "webex", "meet"]),
        ("email", &["mail", "outlook", "gmail", "spark", "thunderbird"]),
        ("productivity", &["notion", "asana", "trello", "monday", "linear", "jira", "confluence"]),
        ("design", &["figma", "sketch", "framer", "photoshop", "illustrator"]),
        ("dev", &["code", "vscode", "xcode", "intellij", "pycharm", "iterm", "terminal", "warp"]),
        ("ai", &["chatgpt", "claude", "perplexity", "copilot", "cursor"]),
        ("browser", &["chrome", "safari", "firefox", "arc", "edge", "brave"]),
        ("storage", &["dropbox", "drive", "onedrive", "box", "icloud"]),
        ("crm", &["salesforce", "hubspot", "pipedrive", "zoho"]),
    ];
    for (cat, kws) in rules {
        for k in *kws {
            if blob.contains(k) {
                return (*cat).into();
            }
        }
    }
    "other".into()
}
