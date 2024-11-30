use std::io;
use tokio::io::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

const MAX_FRAME_SIZE: u32 = 16 * 1024 * 1024; // 16MB limit

#[derive(Debug)]
pub enum FrameError {
    Io(io::Error),
    TooLarge(u32),
}

impl From<io::Error> for FrameError {
    fn from(err: io::Error) -> Self {
        FrameError::Io(err)
    }
}

pub async fn write_frame<W, T>(
    writer: &mut W,
    value: &T,
) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
    T: serde::Serialize,
{
    let data = serde_json::to_vec(value).unwrap();
    let len = data.len() as u32;

    if len > MAX_FRAME_SIZE {
        return Err(FrameError::TooLarge(len));
    }

    writer.write_u32(len).await?;
    writer.write_all(&data).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read_frame<R, T>(
    reader: &mut R,
) -> Result<T, FrameError>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let len = reader.read_u32().await?;

    if len > MAX_FRAME_SIZE {
        return Err(FrameError::TooLarge(len));
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;

    Ok(serde_json::from_slice(&buf).unwrap())
}
