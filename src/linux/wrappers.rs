use std::{
    fs::File,
    io,
    os::fd::{AsRawFd, RawFd},
};

use crate::MAX_CHUNK_SIZE;

pub(crate) fn sendfile(
    file: RawFd,
    stream: RawFd,
    offset: &mut i64,
    len: usize,
) -> io::Result<usize> {
    if len > MAX_CHUNK_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("len cannot be greater than {}", MAX_CHUNK_SIZE),
        ));
    }

    match unsafe { libc::sendfile(stream.as_raw_fd(), file.as_raw_fd(), offset, len) } {
        -1 => Err(io::Error::last_os_error()),
        written => Ok(written as usize),
    }
}

pub fn write_file_part(
    file: &impl AsRawFd,
    stream: &impl AsRawFd,
    offset: usize,
    len: usize,
) -> io::Result<usize> {
    let mut offset = offset as i64;
    sendfile(file.as_raw_fd(), stream.as_raw_fd(), &mut offset, len)
}

pub fn write_file_chunked(
    file: &File,
    stream: &impl AsRawFd,
    chunk_size: usize,
) -> io::Result<usize> {
    let len = file.metadata()?.len();

    let mut offset = 0;
    while offset < len as i64 {
        sendfile(
            file.as_raw_fd(),
            stream.as_raw_fd(),
            &mut offset,
            chunk_size,
        )?;
    }

    Ok(len as usize)
}

pub fn write_file(file: &File, stream: &impl AsRawFd) -> io::Result<usize> {
    write_file_chunked(file, stream, MAX_CHUNK_SIZE)
}
