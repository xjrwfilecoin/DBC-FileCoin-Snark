use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub auth: bool,
    pub listen_addr: String,
    pub private_cert: Option<String>,
    pub cert_chain: Option<String>,
    pub job_limits: HashMap<String, u64>,
    pub allow_tokens: Vec<String>,
}
