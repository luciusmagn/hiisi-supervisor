use hiisi_common::protocol::PortInfo;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

const MIN_PORT: u16 = 1024;
const MAX_PORT: u16 = 65535;
const SAVE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Serialize, Deserialize)]
pub struct PortAllocation {
    pub user: String,
    pub allocated_at: SystemTime,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PortState {
    allocations: HashMap<u16, PortAllocation>,
    #[serde(skip)]
    last_save: Option<SystemTime>,
}

impl PortState {
    pub fn load() -> Self {
        std::fs::read_to_string("/etc/hiisi/ports.ron")
            .map(|contents| ron::from_str(&contents).unwrap())
            .unwrap_or_default()
    }

    fn save(&self) {
        let contents = ron::to_string(self).unwrap();
        std::fs::write("/etc/hiisi/ports.ron", contents).ok();
    }

    pub fn check_save(&mut self) {
        let now = SystemTime::now();
        if self
            .last_save
            .and_then(|t| now.duration_since(t).ok())
            .map_or(true, |d| d >= SAVE_INTERVAL)
        {
            self.save();
            self.last_save = Some(now);
        }
    }

    pub fn allocate(
        &mut self,
        user: String,
        requested_port: Option<u16>,
    ) -> Option<u16> {
        match requested_port {
            Some(port) if self.is_available(port) => {
                self.allocate_specific(user, port);
                Some(port)
            }
            None => self.allocate_random(user),
            _ => None,
        }
    }

    fn allocate_specific(&mut self, user: String, port: u16) {
        self.allocations.insert(
            port,
            PortAllocation {
                user,
                allocated_at: SystemTime::now(),
            },
        );
    }

    fn allocate_random(&mut self, user: String) -> Option<u16> {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let port = rng.gen_range(MIN_PORT..=MAX_PORT);
            if self.is_available(port) {
                self.allocate_specific(user, port);
                return Some(port);
            }
        }
        None
    }

    pub fn free(&mut self, port: u16) -> bool {
        self.allocations.remove(&port).is_some()
    }

    fn is_available(&self, port: u16) -> bool {
        port >= MIN_PORT
            && port <= MAX_PORT
            && !self.allocations.contains_key(&port)
    }

    pub fn lookup(&self, user: Option<String>) -> Vec<PortInfo> {
        self.allocations
            .iter()
            .filter(|(_, alloc)| {
                user.as_ref().map_or(true, |u| alloc.user == *u)
            })
            .map(|(&port, alloc)| PortInfo {
                port,
                user: alloc.user.clone(),
                active: false, // TODO: implement port activity check
                allocated_at: alloc.allocated_at.into(),
            })
            .collect()
    }
}
