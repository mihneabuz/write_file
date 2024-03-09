use std::{fs::File, io, net::TcpStream};

use crate::wrappers::send_file;

pub trait SendFile {
    fn write_file(&self, file: File) -> io::Result<usize>;
}

impl SendFile for TcpStream {
    fn write_file(&self, file: File) -> io::Result<usize> {
        send_file(file, self)
    }
}
