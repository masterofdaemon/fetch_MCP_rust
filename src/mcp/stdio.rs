use anyhow::{bail, Context, Result};
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

// Read a Content-Length framed message over stdio (like LSP/MCP)
pub async fn read_message<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    // We need to read headers until empty line, then read body based on Content-Length
    // Use a small buffer and incremental parsing to keep memory small
    let mut buf_reader = BufReader::new(reader);
    let mut content_length: Option<usize> = None;
    let mut header_line = String::new();

    loop {
        header_line.clear();
        let n = buf_reader.read_line(&mut header_line).await?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF while reading headers"));
        }
        if header_line == "\r\n" {
            break;
        }
        if let Some((name, value)) = header_line.split_once(":") {
            if name.eq_ignore_ascii_case("Content-Length") {
                let v = value.trim();
                let v = v.trim_start_matches(':').trim();
                content_length = Some(v.parse::<usize>().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Content-Length"))?);
            }
        }
    }

    let len = content_length.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing Content-Length"))?;
    let mut body = vec![0u8; len];
    buf_reader.read_exact(&mut body).await?;
    Ok(body)
}

pub async fn write_message<W: AsyncWrite + Unpin>(writer: &mut W, body: &[u8]) -> io::Result<()> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(body).await?;
    writer.flush().await
}
