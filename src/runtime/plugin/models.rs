use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeForMemoryRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWorkletRequest {
    pub memory_limit: usize,
    pub assets: Option<String>,
    pub on_poll: u32,
    pub on_suspend: u32,
    pub on_wakeup: u32,
    pub on_idle: u32,
    pub on_resume: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartFileRequest {
    pub id: u8,
    pub filename: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartUTF8Request {
    pub id: u8,
    pub filename: String,
    pub source: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartBytesRequest {
    pub id: u8,
    pub filename: String,
    pub source: Vec<u8>,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadRequest {
    pub id: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    pub id: u8,
    pub readable: bool,
    pub writable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuspendRequest {
    pub id: u8,
    pub linger: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeRequest {
    pub id: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeupRequest {
    pub id: u8,
    pub deadline: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminateRequest {
    pub id: u8,
}
