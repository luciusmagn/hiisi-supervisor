use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::process::Command;

use crate::state::Process;

pub fn create_log_paths(
    user: &str,
    cwd: &Path,
    cmd: &str,
) -> (PathBuf, PathBuf) {
    let slug_cwd = slug::slugify(cwd.to_string_lossy());
    let slug_cmd = slug::slugify(cmd);
    let base = PathBuf::from("/home")
        .join(user)
        .join(".logs")
        .join(slug_cwd);

    std::fs::create_dir_all(&base).unwrap_or_default();

    (
        base.join(format!("{}.stdout", slug_cmd)),
        base.join(format!("{}.stderr", slug_cmd)),
    )
}

pub async fn spawn_process(
    id: u32,
    user: String,
    cmd: String,
    cwd: PathBuf,
    env: HashMap<String, String>,
    restart: bool,
) -> std::io::Result<Process> {
    let (stdout_path, stderr_path) =
        create_log_paths(&user, &cwd, &cmd);

    let stdout_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stdout_path)?;

    let stderr_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stderr_path)?;

    // Split command into program and args
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap();
    let args: Vec<_> = parts.collect();

    let child = Command::new(program)
        .args(args)
        .current_dir(&cwd)
        .envs(env.clone())
        .stdout(stdout_file)
        .stderr(stderr_file)
        // Run as the specified user
        .uid(users::get_user_by_name(&user).unwrap().uid())
        .spawn()?;

    Ok(Process {
        id,
        user,
        cmd,
        cwd,
        started_at: SystemTime::now(),
        restart,
        child,
        stdout_path,
        stderr_path,
        env,
    })
}

pub async fn stop_process(
    process: &mut Process,
) -> std::io::Result<()> {
    // Try SIGINT first
    process.child.start_kill()?;

    // Wait up to 15 seconds for graceful shutdown
    tokio::select! {
        _ = process.child.wait() => Ok(()),
        _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
            // Try SIGTERM
            process.child.kill().await?;

            // Wait another 15 seconds
            tokio::select! {
                _ = process.child.wait() => Ok(()),
                _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                    // Force SIGKILL
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(process.child.id().unwrap() as i32),
                        nix::sys::signal::Signal::SIGKILL,
                    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    Ok(())
                }
            }
        }
    }
}
