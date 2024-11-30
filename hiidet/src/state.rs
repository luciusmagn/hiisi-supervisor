use hiisi_common::protocol::ProcessInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::process::Child;

pub struct Process {
    pub id: u32,
    pub user: String,
    pub cmd: String,
    pub cwd: PathBuf,
    pub started_at: SystemTime,
    pub restart: bool,
    pub child: Child,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

impl Process {
    pub fn to_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: self.id,
            user: self.user.clone(),
            uptime: SystemTime::now()
                .duration_since(self.started_at)
                .unwrap_or(Duration::from_secs(0)),
            cwd: self.cwd.clone(),
            cmd: self.cmd.clone(),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub processes: HashMap<u32, Process>,
    pub next_id: u32,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_process(&mut self, process: Process) {
        self.processes.insert(process.id, process);
    }

    pub fn remove_process(
        &mut self,
        id: u32,
    ) -> Option<Process> {
        self.processes.remove(&id)
    }

    pub fn get_process(&self, id: u32) -> Option<&Process> {
        self.processes.get(&id)
    }

    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.values().map(Process::to_info).collect()
    }
}
