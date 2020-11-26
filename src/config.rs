use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub auth: bool,
    pub listen_addr: String,
    pub private_cert: Option<String>,
    pub cert_chain: Option<String>,
    #[serde(default)]
    pub job_limits: HashMap<String, u64>,
    #[serde(default)]
    pub allow_tokens: Vec<String>,
}
