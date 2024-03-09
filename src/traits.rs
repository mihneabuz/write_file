use std::{fs::File, io, net::TcpStream};

use crate::{send_file, send_file_all_chunked, wrappers::send_file_all, MAX_CHUNK_SIZE};

pub trait SendFile {
    fn write_file(&self, file: &File, offset: usize, len: usize) -> io::Result<usize>;

    fn write_file_all_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize>;

    fn write_file_all(&self, file: &File) -> io::Result<usize> {
        self.write_file_all_chunked(file, MAX_CHUNK_SIZE)
    }
}

impl SendFile for TcpStream {
    fn write_file(&self, file: &File, offset: usize, len: usize) -> io::Result<usize> {
        send_file(file, self, offset, len)
    }

    fn write_file_all_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize> {
        send_file_all_chunked(file, self, chunk_size)
    }

    fn write_file_all(&self, file: &File) -> io::Result<usize> {
        send_file_all(file, self)
    }
}
