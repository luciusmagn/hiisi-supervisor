use hiisi_common::frame::{read_frame, write_frame};
use hiisi_common::protocol::{
    Command, Message, Response, ResponseData,
};
use std::path::Path;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use crate::monitor::SystemMonitor;
use crate::ports::PortState;
use crate::process::{spawn_process, stop_process};
use crate::state::State;

pub struct Server {
    state: Arc<Mutex<State>>,
    ports: Arc<Mutex<PortState>>,
    monitor: Arc<Mutex<SystemMonitor>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::new())),
            ports: Arc::new(Mutex::new(PortState::load())),
            monitor: Arc::new(Mutex::new(SystemMonitor::new())),
        }
    }

    pub async fn run(
        &self,
        socket_path: &Path,
    ) -> std::io::Result<()> {
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;

        // Start port state saving task
        let ports = self.ports.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(
                    std::time::Duration::from_secs(30),
                )
                .await;
                ports.lock().await.check_save();
            }
        });

        loop {
            let (socket, _) = listener.accept().await?;
            let (mut reader, mut writer) = socket.into_split();

            let state = self.state.clone();
            let ports = self.ports.clone();
            let monitor = self.monitor.clone();

            tokio::spawn(async move {
                while let Ok(msg) =
                    read_frame::<_, Message>(&mut reader).await
                {
                    let response = handle_message(
                        msg, &state, &ports, &monitor,
                    )
                    .await;
                    if write_frame(&mut writer, &response)
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }
    }
}

async fn handle_message(
    msg: Message,
    state: &Arc<Mutex<State>>,
    ports: &Arc<Mutex<PortState>>,
    monitor: &Arc<Mutex<SystemMonitor>>,
) -> Response {
    match msg.cmd {
        Command::Run { cmd, cwd, env, restart } => {
            let mut state = state.lock().await;
            let id = state.next_id();

            match spawn_process(
                id, msg.user, cmd, cwd, env, restart,
            )
            .await
            {
                Ok(process) => {
                    state.add_process(process);
                    Response::Ok(ResponseData::ProcessStarted {
                        id,
                    })
                }
                Err(e) => Response::Error(format!(
                    "Failed to start process: {}",
                    e
                )),
            }
        }

        Command::Stop { id } => {
            let mut state = state.lock().await;

            match state.get_process(id) {
                Some(process) if process.user == msg.user => {
                    if let Some(mut process) =
                        state.remove_process(id)
                    {
                        match stop_process(&mut process).await {
                            Ok(_) => Response::Ok(
                                ResponseData::ProcessStopped,
                            ),
                            Err(e) => Response::Error(format!(
                                "Failed to stop process: {}",
                                e
                            )),
                        }
                    } else {
                        Response::Error(
                            "Process not found".into(),
                        )
                    }
                }
                Some(_) => Response::Error(
                    "Not authorized to stop this process".into(),
                ),
                None => {
                    Response::Error("Process not found".into())
                }
            }
        }

        Command::Logs { id } => {
            let state = state.lock().await;

            match state.get_process(id) {
                Some(process) if process.user == msg.user => {
                    Response::Ok(ResponseData::Logs {
                        stdout: process.stdout_path.clone(),
                        stderr: process.stderr_path.clone(),
                    })
                }
                Some(_) => Response::Error(
                    "Not authorized to view these logs".into(),
                ),
                None => {
                    Response::Error("Process not found".into())
                }
            }
        }

        Command::Status => {
            let state = state.lock().await;
            Response::Ok(ResponseData::Status(
                state.list_processes(),
            ))
        }

        Command::PortLookup { user } => {
            let ports = ports.lock().await;
            Response::Ok(ResponseData::PortList(
                ports.lookup(user),
            ))
        }

        Command::PortAllocate { port } => {
            let mut ports = ports.lock().await;
            match ports.allocate(msg.user, port) {
                Some(port) => {
                    Response::Ok(ResponseData::PortAllocated {
                        port,
                    })
                }
                None => Response::Error(
                    "Port allocation failed".into(),
                ),
            }
        }

        Command::PortFree { port } => {
            let mut ports = ports.lock().await;
            if ports.free(port) {
                Response::Ok(ResponseData::PortFreed)
            } else {
                Response::Error(
                    "Port not found or not owned by user".into(),
                )
            }
        }
    }
}
