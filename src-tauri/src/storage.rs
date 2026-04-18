// Device key persistence via OS keychain
// macOS: Keychain, Windows: Credential Manager, Linux: Secret Service

use keyring::Entry;

const SERVICE: &str = "com.macrospend.tracker";
const ACCOUNT: &str = "device_key";

pub fn save_device_key(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    entry.set_password(key).map_err(|e| e.to_string())
}

pub fn load_device_key() -> Result<String, String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn clear_device_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())
}
