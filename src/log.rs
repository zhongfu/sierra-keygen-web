use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

#[derive(Clone, Serialize)]
struct LogMetadata {
    level: LogLevel
}

#[derive(Clone, Serialize)]
pub struct LogMessage {
    metadata: LogMetadata,
    message: String,
}

#[derive(Serialize)]
struct LogflareBatch {
    batch: Vec<LogMessage>,
}

pub fn make_log(level: LogLevel, message: String) -> LogMessage {
    LogMessage {
        metadata: LogMetadata { level },
        message,
    }
}

// log to logflare
pub async fn log(api_key: &str, source_id: &str, messages: Vec<LogMessage>) {
    let payload = LogflareBatch { batch: messages };
    let client = reqwest::Client::new();
    let _ = client
        .post(format!("https://api.logflare.app/logs?source={}", source_id).as_str())
        .header("X-API-KEY", api_key)
        .json(&payload)
        .send()
        .await;
}
