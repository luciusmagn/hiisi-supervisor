use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio_stream::StreamExt;
use tracing::error;

pub async fn tail_logs(
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    history: bool,
) -> std::io::Result<()> {
    println!("Full logs available at:");
    println!("  stdout: {}", stdout_path.display());
    println!("  stderr: {}", stderr_path.display());
    println!("---");

    // If history requested, show last 100 lines first
    if history {
        if let Ok(content) =
            tokio::fs::read_to_string(&stdout_path).await
        {
            let mut lines: Vec<_> =
                content.lines().rev().take(100).collect();
            lines.reverse();
            for line in lines {
                println!("out: {}", line);
            }
        }

        if let Ok(content) =
            tokio::fs::read_to_string(&stderr_path).await
        {
            let mut lines: Vec<_> =
                content.lines().rev().take(100).collect();
            lines.reverse();
            for line in lines {
                eprintln!("err: {}", line);
            }
        }
    }

    // Open files and seek to end
    let mut stdout = File::open(&stdout_path).await?;
    let mut stderr = File::open(&stderr_path).await?;

    stdout.seek(SeekFrom::End(0)).await?;
    stderr.seek(SeekFrom::End(0)).await?;

    // Start streaming new lines
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
                    Ok(content) => println!("out: {}", content),
                    Err(e) => error!("Error reading stdout: {}", e),
                }
            }
            Some(line) = stderr_stream.next() => {
                match line {
                    Ok(content) => eprintln!("err: {}", content),
                    Err(e) => error!("Error reading stderr: {}", e),
                }
            }
            else => break,
        }
    }

    Ok(())
}

// Just a wrapper for backward compatibility
pub async fn tail_logs_from_end(
    stdout_path: PathBuf,
    stderr_path: PathBuf,
) -> std::io::Result<()> {
    tail_logs(stdout_path, stderr_path, true).await
}
