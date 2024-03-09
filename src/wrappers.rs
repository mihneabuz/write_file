use std::{fs::File, io, os::fd::AsRawFd};

use crate::MAX_CHUNK_SIZE;

pub fn send_file(
    file: &impl AsRawFd,
    stream: &impl AsRawFd,
    offset: usize,
    len: usize,
) -> io::Result<usize> {
    if len > MAX_CHUNK_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("len cannot be greater than {}", MAX_CHUNK_SIZE),
        ));
    }

    let mut offset = offset as i64;
    let ret = unsafe { libc::sendfile(stream.as_raw_fd(), file.as_raw_fd(), &mut offset, len) };

    if ret == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(ret as usize)
}

pub fn send_file_all_chunked(
    file: &File,
    stream: &impl AsRawFd,
    chunk_size: usize,
) -> io::Result<usize> {
    if chunk_size > MAX_CHUNK_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("chunk_size cannot be greater than {}", MAX_CHUNK_SIZE),
        ));
    }

    let len = file.metadata()?.len();

    let mut offset = 0;
    while offset < len as i64 {
        let ret = unsafe {
            libc::sendfile(
                stream.as_raw_fd(),
                file.as_raw_fd(),
                &mut offset,
                chunk_size,
            )
        };

        if ret == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(len as usize)
}

pub fn send_file_all(file: &File, stream: &impl AsRawFd) -> io::Result<usize> {
    send_file_all_chunked(file, stream, MAX_CHUNK_SIZE)
}
