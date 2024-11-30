use std::path::PathBuf;
use hiisi_common::frame::{read_frame, write_frame};
use hiisi_common::protocol::{Command, Message, Response};
use tokio::net::UnixStream;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn connect() -> std::io::Result<Self> {
        let stream =
            UnixStream::connect("/run/hiisi/hiisi.sock").await?;
        Ok(Self { stream })
    }

    async fn send_command(
        &mut self,
        cmd: Command,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let msg = Message {
            cmd,
            user: users::get_current_username()
                .ok_or("Couldn't get username")?
                .into_string()
                .map_err(|_| "Invalid username")?,
        };

        write_frame(&mut self.stream, &msg).await?;
        let response: Response =
            read_frame(&mut self.stream).await?;
        Ok(response)
    }

    pub async fn run(
        &mut self,
        cmd: String,
        restart: bool,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let cwd = std::env::current_dir()?;
        let env = std::env::vars().collect();

        match self.send_command(Command::Run { cmd, cwd, env, restart }).await? {
           Response::Ok(hiisi_common::protocol::ResponseData::ProcessStarted { id }) => Ok(id),
           Response::Error(e) => Err(e.into()),
           _ => Err("Unexpected response".into()),
       }
    }

    pub async fn stop(
        &mut self,
        id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.send_command(Command::Stop { id }).await? {
           Response::Ok(hiisi_common::protocol::ResponseData::ProcessStopped) => Ok(()),
           Response::Error(e) => Err(e.into()),
           _ => Err("Unexpected response".into()),
       }
    }

    pub async fn status(
        &mut self,
    ) -> Result<
        Vec<hiisi_common::protocol::ProcessInfo>,
        Box<dyn std::error::Error>,
    > {
        match self.send_command(Command::Status).await? {
            Response::Ok(
                hiisi_common::protocol::ResponseData::Status(
                    info,
                ),
            ) => Ok(info),
            Response::Error(e) => Err(e.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn logs(
        &mut self,
        id: u32,
    ) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>>
    {
        match self.send_command(Command::Logs { id }).await? {
            Response::Ok(
                hiisi_common::protocol::ResponseData::Logs {
                    stdout,
                    stderr,
                },
            ) => Ok((stdout, stderr)),
            Response::Error(e) => Err(e.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn port_allocate(
        &mut self,
        port: Option<u16>,
    ) -> Result<u16, Box<dyn std::error::Error>> {
        match self.send_command(Command::PortAllocate { port }).await? {
           Response::Ok(hiisi_common::protocol::ResponseData::PortAllocated { port }) => Ok(port),
           Response::Error(e) => Err(e.into()),
           _ => Err("Unexpected response".into()),
       }
    }

    pub async fn port_free(
        &mut self,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self
            .send_command(Command::PortFree { port })
            .await?
        {
            Response::Ok(
                hiisi_common::protocol::ResponseData::PortFreed,
            ) => Ok(()),
            Response::Error(e) => Err(e.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn port_lookup(
        &mut self,
        user: Option<String>,
    ) -> Result<
        Vec<hiisi_common::protocol::PortInfo>,
        Box<dyn std::error::Error>,
    > {
        match self
            .send_command(Command::PortLookup { user })
            .await?
        {
            Response::Ok(
                hiisi_common::protocol::ResponseData::PortList(
                    info,
                ),
            ) => Ok(info),
            Response::Error(e) => Err(e.into()),
            _ => Err("Unexpected response".into()),
        }
    }
}
