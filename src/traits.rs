use std::{fs::File, io, net::TcpStream};

use crate::{send_file_chunked, wrappers::send_file, MAX_CHUNK_SIZE};

pub trait SendFile {
    fn write_file_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize>;

    fn write_file(&self, file: &File) -> io::Result<usize> {
        self.write_file_chunked(file, MAX_CHUNK_SIZE)
    }
}

impl SendFile for TcpStream {
    fn write_file_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize> {
        send_file_chunked(file, self, chunk_size)
    }

    fn write_file(&self, file: &File) -> io::Result<usize> {
        send_file(file, self)
    }
}
