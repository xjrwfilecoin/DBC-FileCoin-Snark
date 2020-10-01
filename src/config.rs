use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JobLimit {
    pub name: String,
    pub limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub auth: bool,
    pub listen_addr: String,
    pub private_cert: Option<String>,
    pub cert_chain: Option<String>,
    pub job_limits: Vec<JobLimit>,
    pub allow_tokens: Vec<String>,
}
