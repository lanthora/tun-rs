use crate::platform::DeviceImpl;
use ::tokio::io::unix::AsyncFd as TokioAsyncFd;
use ::tokio::io::Interest;
use std::io;
use std::io::{Error, IoSlice, IoSliceMut};
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct AsyncFd(TokioAsyncFd<DeviceImpl>);
impl AsyncFd {
    pub fn new(device: DeviceImpl) -> io::Result<Self> {
        device.set_nonblocking(true)?;
        Ok(Self(TokioAsyncFd::new(device)?))
    }
    pub fn into_device(self) -> io::Result<DeviceImpl> {
        Ok(self.0.into_inner())
    }
    pub async fn readable(&self) -> io::Result<()> {
        _ = self.0.readable().await?;
        Ok(())
    }
    pub fn poll_readable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.0.poll_read_ready(cx) {
            Poll::Ready(rs) => Poll::Ready(rs.map(|_| ())),
            Poll::Pending => Poll::Pending,
        }
    }
    pub async fn writable(&self) -> io::Result<()> {
        _ = self.0.writable().await?;
        Ok(())
    }
    pub fn poll_writable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.0.poll_write_ready(cx) {
            Poll::Ready(rs) => Poll::Ready(rs.map(|_| ())),
            Poll::Pending => Poll::Pending,
        }
    }
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0
            .async_io(Interest::READABLE.add(Interest::ERROR), |device| {
                device.recv(buf)
            })
            .await
    }
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .async_io(Interest::WRITABLE, |device| device.send(buf))
            .await
    }
    pub async fn send_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0
            .async_io(Interest::WRITABLE, |device| device.send_vectored(bufs))
            .await
    }
    pub async fn recv_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0
            .async_io(Interest::READABLE.add(Interest::ERROR), |device| {
                device.recv_vectored(bufs)
            })
            .await
    }

    pub fn get_ref(&self) -> &DeviceImpl {
        self.0.get_ref()
    }
}

impl AsyncRead for AsyncFd {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.0.poll_read_ready(cx))?;
            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|inner| inner.get_ref().recv(unfilled)) {
                Ok(Ok(len)) => {
                    buf.advance(len);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncFd {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        loop {
            let mut guard = ready!(self.0.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().send(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        #[cfg(feature = "experimental")]
        self.0.get_ref().tun.shutdown()?;
        Poll::Ready(Ok(()))
    }
}
