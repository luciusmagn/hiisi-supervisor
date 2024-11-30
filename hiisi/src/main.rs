mod client;
mod display;
mod logs;

use clap::{Parser, Subcommand};
use client::Client;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a background process
    Run {
        /// Restart the process if it dies
        #[arg(long)]
        restart: bool,
        /// Command to run
        #[arg(required = true, num_args = 1.., last = true)]
        command: Vec<String>,
    },
    /// Stop a background process
    Stop {
        /// Process ID
        id: u32,
    },
    /// Show running processes
    Status,
    /// Show process logs
    Logs {
        /// Process ID
        id: u32,
    },
    /// Port management
    Port {
        #[command(subcommand)]
        cmd: PortCommands,
    },
}

#[derive(Subcommand)]
enum PortCommands {
    /// Allocate a port
    Allocate {
        /// Specific port to allocate
        port: Option<u16>,
    },
    /// Free a port
    Free {
        /// Port to free
        port: u16,
    },
    /// List allocated ports
    Lookup {
        /// Show ports for specific user
        user: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut client = Client::connect().await?;

    match cli.command {
        Commands::Run { restart, command } => {
            let cmd = command.join(" ");
            let id = client.run(cmd, restart).await?;
            println!(
                "{}",
                display::format_success(&format!(
                    "Started process {}",
                    id
                ))
            );
        }

        Commands::Stop { id } => {
            client.stop(id).await?;
            println!(
                "{}",
                display::format_success(&format!(
                    "Stopped process {}",
                    id
                ))
            );
        }

        Commands::Status => {
            let processes = client.status().await?;
            println!(
                "{}",
                display::format_processes(&processes)
            );
        }

        Commands::Logs { id } => {
            let (stdout, stderr) = client.logs(id).await?;
            logs::tail_logs_from_end(stdout, stderr).await?;
        }

        Commands::Port { cmd } => match cmd {
            PortCommands::Allocate { port } => {
                let allocated =
                    client.port_allocate(port).await?;
                println!(
                    "{}",
                    display::format_success(&format!(
                        "Allocated port {}",
                        allocated
                    ))
                );
            }

            PortCommands::Free { port } => {
                client.port_free(port).await?;
                println!(
                    "{}",
                    display::format_success(&format!(
                        "Freed port {}",
                        port
                    ))
                );
            }

            PortCommands::Lookup { user } => {
                let ports = client.port_lookup(user).await?;
                println!("{}", display::format_ports(&ports));
            }
        },
    }

    Ok(())
}
