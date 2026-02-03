use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhatProfile {
    #[serde(rename = "dhatFileVersion")]
    pub dhat_file_version: u32,
    pub mode: String,
    pub verb: String,
    pub bklt: bool,
    pub bkacc: bool,
    pub tu: String,
    #[serde(rename = "Mtu")]
    pub mtu: String,
    pub tuth: u64,
    pub cmd: String,
    pub pid: u32,
    pub tg: u64,
    pub te: u64,
    pub pps: Vec<ProfilePoint>,
    pub ftbl: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePoint {
    pub tb: u64,
    pub tbk: u64,
    pub tl: u64,
    pub mb: u64,
    pub mbk: u64,
    pub gb: u64,
    pub gbk: u64,
    pub eb: u64,
    pub ebk: u64,
    pub fs: Vec<usize>,
}
