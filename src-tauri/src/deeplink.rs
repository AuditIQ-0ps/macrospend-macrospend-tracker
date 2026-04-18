// Deep-link handler: macrospend://register?key=<api_key>
// The web app generates the key, then opens this URL. Tracker stores it
// in the OS keychain and starts sampling without the user copying anything.

use crate::{storage, AppState};
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn register_handler(handle: AppHandle) {
    let h = handle.clone();
    handle.deep_link().on_open_url(move |event| {
        for url in event.urls() {
            log::info!("deep link received: {url}");
            if url.scheme() != "macrospend" {
                continue;
            }
            // Accept either macrospend://register?key=... or macrospend://register/<key>
            if url.host_str() != Some("register") && url.path() != "/register" {
                continue;
            }
            let key_opt = url
                .query_pairs()
                .find(|(k, _)| k == "key")
                .map(|(_, v)| v.into_owned());

            if let Some(key) = key_opt {
                match storage::save_device_key(&key) {
                    Ok(_) => {
                        let state = h.state::<AppState>();
                        *state.device_key.write() = Some(key);
                        log::info!("device key saved via deep link");
                        if let Some(w) = h.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    Err(e) => log::error!("failed to save device key: {e}"),
                }
            }
        }
    });
}
