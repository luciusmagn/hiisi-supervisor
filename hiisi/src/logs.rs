use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::StreamExt;
use tracing::error;

use std::path::PathBuf;

pub async fn tail_logs(
    stdout_path: PathBuf,
    stderr_path: PathBuf,
) -> std::io::Result<()> {
    let stdout = File::open(&stdout_path).await?;
    let stderr = File::open(&stderr_path).await?;

    let stdout_lines = BufReader::new(stdout).lines();
    let stderr_lines = BufReader::new(stderr).lines();

    let mut stdout_stream =
        tokio_stream::wrappers::LinesStream::new(stdout_lines);
    let mut stderr_stream =
        tokio_stream::wrappers::LinesStream::new(stderr_lines);

    loop {
        tokio::select! {
            Some(line) = stdout_stream.next() => {
                match line {
                    Ok(content) => println!("{}", content),
                    Err(e) => error!("Error reading stdout: {}", e),
                }
            }
            Some(line) = stderr_stream.next() => {
                match line {
                    Ok(content) => eprintln!("\x1b[31m{}\x1b[0m", content),
                    Err(e) => error!("Error reading stderr: {}", e),
                }
            }
            else => break,
        }
    }

    Ok(())
}

pub async fn tail_logs_from_end(
    stdout_path: PathBuf,
    stderr_path: PathBuf,
) -> std::io::Result<()> {
    // Read last 100 lines from each file
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    if let Ok(content) =
        tokio::fs::read_to_string(&stdout_path).await
    {
        stdout_lines = content
            .lines()
            .rev()
            .take(100)
            .map(String::from)
            .collect::<Vec<_>>();
        stdout_lines.reverse();
    }

    if let Ok(content) =
        tokio::fs::read_to_string(&stderr_path).await
    {
        stderr_lines = content
            .lines()
            .rev()
            .take(100)
            .map(String::from)
            .collect::<Vec<_>>();
        stderr_lines.reverse();
    }

    // Print historical lines
    for line in stdout_lines {
        println!("{}", line);
    }
    for line in stderr_lines {
        eprintln!("\x1b[31m{}\x1b[0m", line);
    }

    // Then start tailing
    tail_logs(stdout_path, stderr_path).await
}
