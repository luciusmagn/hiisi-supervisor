use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Run {
        cmd: String,
        cwd: PathBuf,
        env: HashMap<String, String>,
        restart: bool,
    },
    Stop {
        id: u32,
    },
    Status,
    Logs {
        id: u32,
    },
    PortAllocate {
        port: Option<u16>,
    },
    PortFree {
        port: u16,
    },
    PortLookup {
        user: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub cmd: Command,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessStatus {
    Running,
    Exited(i32),    // Exit code if we have it
    Failed(String), // Error message if process failed to start/crashed
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Exited(num) => write!(f, "exited({num})"),
            Self::Failed(err) => write!(f, "failed({err})"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: u32,
    pub user: String,
    pub uptime: Duration,
    pub cwd: PathBuf,
    pub cmd: String,
    pub status: ProcessStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub user: String,
    pub active: bool,
    pub allocated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseData {
    ProcessStarted { id: u32 },
    ProcessStopped,
    Status(Vec<ProcessInfo>),
    Logs { stdout: PathBuf, stderr: PathBuf },
    PortAllocated { port: u16 },
    PortFreed,
    PortList(Vec<PortInfo>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok(ResponseData),
    Error(String),
}
