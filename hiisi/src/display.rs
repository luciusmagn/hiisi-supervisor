use hiisi_common::protocol::{PortInfo, ProcessInfo};
use std::time::Duration;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ProcessRow {
    #[tabled(rename = "ID")]
    id: u32,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "UPTIME")]
    uptime: String,
    #[tabled(rename = "CWD")]
    cwd: String,
    #[tabled(rename = "COMMAND")]
    cmd: String,
}

#[derive(Tabled)]
struct PortRow {
    #[tabled(rename = "PORT")]
    port: u16,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "ALLOCATED")]
    allocated: String,
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

pub fn format_processes(processes: &[ProcessInfo]) -> String {
    let rows: Vec<ProcessRow> = processes
        .iter()
        .map(|p| ProcessRow {
            id: p.id,
            user: p.user.clone(),
            uptime: format_duration(p.uptime),
            cwd: p.cwd.to_string_lossy().into_owned(),
            cmd: p.cmd.clone(),
        })
        .collect();

    Table::new(rows).to_string()
}

pub fn format_ports(ports: &[PortInfo]) -> String {
    let rows: Vec<PortRow> = ports
        .iter()
        .map(|p| PortRow {
            port: p.port,
            user: p.user.clone(),
            status: if p.active { "ACTIVE" } else { "IDLE" }
                .into(),
            allocated: humantime::format_rfc3339(
                p.allocated_at.into(),
            )
            .to_string(),
        })
        .collect();

    Table::new(rows).to_string()
}

pub fn format_error(err: &str) -> String {
    format!("Error: {}", err)
}

pub fn format_success(msg: &str) -> String {
    format!("Success: {}", msg)
}
