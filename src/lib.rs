use std::{
    fs::File,
    io::{self, Seek, SeekFrom},
    net::TcpStream,
    os::fd::AsRawFd,
};

pub fn send_file(mut file: File, stream: &impl AsRawFd) -> io::Result<usize> {
    const MAX: usize = 0x7ffff0000;
    let len = file.seek(SeekFrom::End(0)).unwrap();

    let mut offset = 0;
    while offset < len as i64 {
        let ret = unsafe { libc::sendfile(stream.as_raw_fd(), file.as_raw_fd(), &mut offset, MAX) };
        if ret == 1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(len as usize)
}

trait SendFile {
    fn write_file(&self, file: File) -> io::Result<usize>;
}

impl SendFile for TcpStream {
    fn write_file(&self, file: File) -> io::Result<usize> {
        send_file(file, self)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    use super::*;

    fn simple_std(content: &str) {
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let expected = String::from(content);
        let handle = std::thread::spawn(move || {
            let mut conn = TcpStream::connect(("0.0.0.0", port)).unwrap();

            let mut buf = Vec::new();
            conn.read_to_end(&mut buf).unwrap();

            assert_eq!(String::from_utf8(buf).unwrap().as_str(), expected);
        });

        let mut file = tempfile::tempfile().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        if let Ok((mut stream, _)) = listener.accept() {
            stream.write_file(file).unwrap();
            stream.flush().unwrap();
        }

        handle.join().unwrap();
    }

    #[test]
    fn simple_std_cases() {
        simple_std("hello world!");
        simple_std("hello world!".repeat(100).as_str());
        simple_std("hello world!".repeat(10000).as_str());
    }
}
