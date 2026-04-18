// Batched HTTP sender to tracker-ingest edge function.

use crate::sampler::Event;

const INGEST_URL: &str =
    "https://bhluusqyqetongajeprh.supabase.co/functions/v1/tracker-ingest?action=ingest";

#[derive(serde::Serialize)]
struct Payload<'a> {
    events: &'a [Event],
}

pub async fn send_batch(device_key: &str, events: &[Event]) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("MacroSpendTracker/0.1 (tauri)")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(INGEST_URL)
        .header("Content-Type", "application/json")
        .header("X-Device-Key", device_key)
        .json(&Payload { events })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    log::info!("sent {} events", events.len());
    Ok(())
}
