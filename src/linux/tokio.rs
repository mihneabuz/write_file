use std::{io, os::fd::AsRawFd};

use tokio::{fs::File, io::Interest, net::TcpStream};

use crate::{sendfile, MAX_CHUNK_SIZE};

pub async fn write_file_part(
    file: &File,
    stream: &TcpStream,
    offset: usize,
    len: usize,
) -> io::Result<usize> {
    let mut offset = offset as i64;

    let ret = stream
        .async_io(Interest::WRITABLE, || {
            sendfile(file.as_raw_fd(), stream.as_raw_fd(), &mut offset, len)
        })
        .await?;

    Ok(ret as usize)
}

pub async fn write_file_chunked(
    file: &File,
    stream: &TcpStream,
    chunk_size: usize,
) -> io::Result<usize> {
    let len = file.metadata().await?.len();

    let mut offset = 0;
    while offset < len as i64 {
        stream
            .async_io(Interest::WRITABLE, || {
                sendfile(
                    file.as_raw_fd(),
                    stream.as_raw_fd(),
                    &mut offset,
                    chunk_size,
                )
            })
            .await?;
    }

    Ok(len as usize)
}

pub async fn write_file(file: &File, stream: &TcpStream) -> io::Result<usize> {
    write_file_chunked(file, stream, MAX_CHUNK_SIZE).await
}
