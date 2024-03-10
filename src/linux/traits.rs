use std::{fs::File, io, net::TcpStream};

use crate::{write_file_chunked, write_file_part, MAX_CHUNK_SIZE};

pub trait SendFile {
    fn write_file_part(&self, file: &File, offset: usize, len: usize) -> io::Result<usize>;

    fn write_file_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize>;

    fn write_file(&self, file: &File) -> io::Result<usize> {
        self.write_file_chunked(file, MAX_CHUNK_SIZE)
    }
}

impl SendFile for TcpStream {
    fn write_file_part(&self, file: &File, offset: usize, len: usize) -> io::Result<usize> {
        write_file_part(file, self, offset, len)
    }

    fn write_file_chunked(&self, file: &File, chunk_size: usize) -> io::Result<usize> {
        write_file_chunked(file, self, chunk_size)
    }
}
