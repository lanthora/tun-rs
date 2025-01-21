use crate::device::ETHER_ADDR_LEN;
use crate::platform::Device;
use crate::{ToIpv4Netmask, ToIpv6Netmask};
use std::future::Future;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

/// An async TUN device wrapper around a TUN device.
pub struct AsyncDevice {
    inner: Arc<Device>,
    recv_task_lock: Arc<Mutex<Option<blocking::Task<io::Result<(Vec<u8>, usize)>>>>>,
    send_task_lock: Arc<Mutex<Option<blocking::Task<io::Result<usize>>>>>,
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> io::Result<AsyncDevice> {
        let inner = Arc::new(device);

        Ok(AsyncDevice {
            inner,
            recv_task_lock: Arc::new(Mutex::new(None)),
            send_task_lock: Arc::new(Mutex::new(None)),
        })
    }
    pub fn poll_recv(&self, cx: &mut Context<'_>, mut buf: &mut [u8]) -> Poll<io::Result<usize>> {
        match self.try_recv(buf) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            rs => return Poll::Ready(rs),
        }
        let mut guard = self.recv_task_lock.lock().unwrap();
        let mut task = if let Some(task) = guard.take() {
            task
        } else {
            let device = self.inner.clone();
            let size = buf.len();
            blocking::unblock(move || {
                let mut in_buf = vec![0; size];
                let n = device.recv(&mut in_buf)?;
                Ok((in_buf, n))
            })
        };
        match Pin::new(&mut task).poll(cx) {
            Poll::Ready(Ok((packet, n))) => {
                drop(guard);
                let mut packet: &[u8] = &packet[..n];
                match io::copy(&mut packet, &mut buf) {
                    Ok(n) => Poll::Ready(Ok(n as usize)),
                    Err(e) => Poll::Ready(Err(e)),
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => {
                guard.replace(task);
                Poll::Pending
            }
        }
    }
    pub fn poll_send(&self, cx: &mut Context<'_>, src: &[u8]) -> Poll<io::Result<usize>> {
        match self.try_send(src) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            rs => return Poll::Ready(rs),
        }
        let mut guard = self.send_task_lock.lock().unwrap();
        loop {
            if let Some(task) = guard.as_mut() {
                match Pin::new(task).poll(cx) {
                    Poll::Ready(rs) => {
                        _ = guard.take();
                        // If the previous write was successful, continue.
                        // Otherwise, error.
                        rs?;
                        continue;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            } else {
                let device = self.inner.clone();
                let buf = src.to_vec();
                let task = blocking::unblock(move || device.send(&buf));
                guard.replace(task);
                return Poll::Ready(Ok(src.len()));
            };
        }
    }

    /// Recv a packet from tun device - Not implemented for windows
    pub async fn recv(&self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.try_recv(buf) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            rs => return rs,
        }
        let device = self.inner.clone();
        let size = buf.len();
        let (packet, n) = blocking::unblock(move || {
            let mut in_buf = vec![0; size];
            let n = device.recv(&mut in_buf)?;
            Ok::<(Vec<u8>, usize), io::Error>((in_buf, n))
        })
        .await?;
        let mut packet: &[u8] = &packet[..n];

        match io::copy(&mut packet, &mut buf) {
            Ok(n) => Ok(n as usize),
            Err(e) => Err(e),
        }
    }
    pub fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.try_recv(buf)
    }

    /// Send a packet to tun device - Not implemented for windows
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.try_send(buf) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            rs => return rs,
        }
        let buf = buf.to_vec();
        let device = self.inner.clone();
        blocking::unblock(move || device.send(&buf)).await
    }
    pub fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        self.inner.try_send(buf)
    }
    pub fn shutdown(&self) -> io::Result<()> {
        self.inner.shutdown()
    }
    pub fn name(&self) -> io::Result<String> {
        self.inner.name()
    }

    pub fn set_name(&self, name: &str) -> io::Result<()> {
        self.inner.set_name(name)
    }

    pub fn if_index(&self) -> io::Result<u32> {
        self.inner.if_index()
    }

    pub fn enabled(&self, value: bool) -> io::Result<()> {
        self.inner.enabled(value)
    }

    pub fn addresses(&self) -> io::Result<Vec<IpAddr>> {
        self.inner.addresses()
    }

    pub fn set_network_address<Netmask: ToIpv4Netmask>(
        &self,
        address: Ipv4Addr,
        netmask: Netmask,
        destination: Option<Ipv4Addr>,
    ) -> io::Result<()> {
        self.inner
            .set_network_address(address, netmask, destination)
    }

    pub fn add_address_v6<Netmask: ToIpv6Netmask>(
        &self,
        addr: Ipv6Addr,
        netmask: Netmask,
    ) -> io::Result<()> {
        self.inner.add_address_v6(addr, netmask)
    }

    pub fn remove_address(&self, addr: IpAddr) -> io::Result<()> {
        self.inner.remove_address(addr)
    }

    pub fn mtu(&self) -> io::Result<u16> {
        self.inner.mtu()
    }
    pub fn mtu_v6(&self) -> io::Result<u16> {
        self.inner.mtu_v6()
    }

    pub fn set_mtu(&self, value: u16) -> io::Result<()> {
        self.inner.set_mtu(value)
    }
    pub fn set_mtu_v6(&self, mtu: u16) -> io::Result<()> {
        self.inner.set_mtu_v6(mtu)
    }

    pub fn set_mac_address(&self, eth_addr: [u8; ETHER_ADDR_LEN as usize]) -> io::Result<()> {
        self.inner.set_mac_address(eth_addr)
    }

    pub fn mac_address(&self) -> io::Result<[u8; ETHER_ADDR_LEN as usize]> {
        self.inner.mac_address()
    }
    pub fn set_metric(&self, metric: u16) -> io::Result<()> {
        self.inner.set_metric(metric)
    }
    pub fn version(&self) -> io::Result<String> {
        self.inner.version()
    }
}
