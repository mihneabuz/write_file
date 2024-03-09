use std::{fs::File, io, os::fd::AsRawFd};

pub fn send_file(file: File, stream: &impl AsRawFd) -> io::Result<usize> {
    const MAX: usize = 0x7ffff0000;
    let len = file.metadata()?.len();

    let mut offset = 0;
    while offset < len as i64 {
        let ret = unsafe { libc::sendfile(stream.as_raw_fd(), file.as_raw_fd(), &mut offset, MAX) };
        if ret == 1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(len as usize)
}
