use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use tokio::fs::File;

use crate::MAX_CHUNK_SIZE;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct WriteFile<'a, 'b, S: ?Sized> {
        inner: &'a mut S,
        file: &'b File,
        offset: usize,
        len: usize
    }
}

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct WriteFileAll<'a, 'b, S: ?Sized> {
        inner: &'a mut S,
        file: &'b File,
        offset: usize,
        len: usize,
        chunk_size: usize,
    }
}

pub trait AsyncSendFile {
    fn poll_write_file(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        file: &File,
        offset: usize,
        len: usize,
    ) -> Poll<io::Result<usize>>;
}

pub trait AsyncSendFileExt: AsyncSendFile {
    fn write_file<'a, 'b>(
        &'a mut self,
        file: &'b File,
        offset: usize,
        len: usize,
    ) -> WriteFile<'a, 'b, Self> {
        WriteFile {
            inner: self,
            file,
            offset,
            len,
        }
    }

    fn write_file_all_chunked<'a, 'b>(
        &'a mut self,
        file: &'b File,
        chunk_size: usize,
    ) -> WriteFileAll<'a, 'b, Self> {
        WriteFileAll {
            inner: self,
            file,
            offset: 0,
            len: usize::MAX,
            chunk_size,
        }
    }

    fn write_file_all<'a, 'b>(&'a mut self, file: &'b File) -> WriteFileAll<'a, 'b, Self> {
        WriteFileAll {
            inner: self,
            file,
            offset: 0,
            len: usize::MAX,
            chunk_size: MAX_CHUNK_SIZE,
        }
    }
}

impl<S> AsyncSendFileExt for S where S: AsyncSendFile {}

impl<S> Future for WriteFile<'_, '_, S>
where
    S: AsyncSendFile + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let inner: Pin<&mut S> = Pin::new(*me.inner);
        inner.poll_write_file(cx, *me.file, *me.offset, *me.len)
    }
}

impl<S> Future for WriteFileAll<'_, '_, S>
where
    S: AsyncSendFile + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let mut inner: Pin<&mut S> = Pin::new(*me.inner);

        while *me.offset < *me.len {
            match inner
                .as_mut()
                .poll_write_file(cx, *me.file, *me.offset, *me.chunk_size)
            {
                Poll::Ready(written) => {
                    let written = written?;
                    *me.offset += written;
                    if written == 0 {
                        return Poll::Ready(Ok(*me.offset));
                    }
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(*me.len))
    }
}

impl AsyncSendFile for tokio::net::TcpStream {
    fn poll_write_file(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        file: &File,
        offset: usize,
        len: usize,
    ) -> Poll<io::Result<usize>> {
        use crate::wrappers::send_file;

        loop {
            let evt = match self.io.registration.poll_write_ready(cx) {
                Poll::Ready(t) => t?,
                Poll::Pending => return Poll::Pending,
            };

            match send_file(file, self.io.io.as_ref().unwrap(), offset, len) {
                Ok(n) => {
                    return Poll::Ready(Ok(n));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.io.registration.clear_readiness(evt);
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}
