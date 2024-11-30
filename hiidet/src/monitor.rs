use std::collections::HashMap;
use sysinfo::System;

pub struct SystemMonitor {
    sys: System,
    last_update: std::time::Instant,
}

#[derive(Debug)]
pub struct ProcessStats {
    pub cpu_usage: f32,
    pub memory_kb: u64,
}

#[derive(Debug)]
pub struct SystemStats {
    pub total_cpu: f32,
    pub total_memory_kb: u64,
    pub used_memory_kb: u64,
    pub process_stats: HashMap<u32, ProcessStats>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) -> SystemStats {
        // Only refresh if more than 1s passed
        if self.last_update.elapsed().as_secs() >= 1 {
            self.sys.refresh_all();
            self.last_update = std::time::Instant::now();
        }

        let total_cpu = self.sys.global_cpu_usage();

        let total_memory_kb = self.sys.total_memory();
        let used_memory_kb = self.sys.used_memory();

        let process_stats = self
            .sys
            .processes()
            .iter()
            .map(|(pid, process)| {
                (
                    pid.as_u32(),
                    ProcessStats {
                        cpu_usage: process.cpu_usage(),
                        memory_kb: process.memory(),
                    },
                )
            })
            .collect();

        SystemStats {
            total_cpu,
            total_memory_kb,
            used_memory_kb,
            process_stats,
        }
    }

    pub fn get_process_stats(
        &self,
        pid: u32,
    ) -> Option<ProcessStats> {
        self.sys.process(sysinfo::Pid::from_u32(pid)).map(
            |process| ProcessStats {
                cpu_usage: process.cpu_usage(),
                memory_kb: process.memory(),
            },
        )
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
