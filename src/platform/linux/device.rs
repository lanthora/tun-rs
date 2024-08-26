//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::{
    ffi::{CStr, CString},
    io::{self},
    mem,
    net::IpAddr,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
    sync::RwLock,
};

use libc::{
    self, c_char, c_short, ifreq, AF_INET, IFF_MULTI_QUEUE, IFF_NO_PI, IFF_RUNNING, IFF_TAP,
    IFF_TUN, IFF_UP, IFNAMSIZ, O_RDWR, SOCK_DGRAM,
};

use crate::configuration::configure;
use crate::{
    configuration::{Configuration, Layer},
    device::AbstractDevice,
    error::{Error, Result},
    platform::linux::sys::*,
    platform::posix::{self, ipaddr_to_sockaddr, sockaddr_union, Fd, Tun},
};

const OVERWRITE_SIZE: usize = std::mem::size_of::<libc::__c_anonymous_ifr_ifru>();

/// A TUN device using the TUN/TAP Linux driver.
pub struct Device {
    name: RwLock<String>,
    tun: Tun,
    ctl: Fd,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let layer = config.layer.unwrap_or(Layer::L3);
        if let Some(fd) = config.raw_fd {
            let close_fd_on_drop = config.close_fd_on_drop.unwrap_or(true);
            let tun = Fd::new(fd, close_fd_on_drop).map_err(|_| io::Error::last_os_error())?;
            let device = Device {
                name: RwLock::new(String::new()),
                tun: posix::Tun::new(tun, config.platform_config.packet_information),
                ctl: Fd::new(unsafe { libc::socket(AF_INET, SOCK_DGRAM, 0) }, true)?,
            };
            return Ok(device);
        }
        let device = unsafe {
            let dev_name = match config.name.as_ref() {
                Some(tun_name) => {
                    let tun_name = CString::new(tun_name.clone())?;

                    if tun_name.as_bytes_with_nul().len() > IFNAMSIZ {
                        return Err(Error::NameTooLong);
                    }

                    Some(tun_name)
                }

                None => None,
            };

            let mut req: ifreq = mem::zeroed();

            if let Some(dev_name) = dev_name.as_ref() {
                ptr::copy_nonoverlapping(
                    dev_name.as_ptr() as *const c_char,
                    req.ifr_name.as_mut_ptr(),
                    dev_name.as_bytes_with_nul().len(),
                );
            }

            let device_type: c_short = layer.into();
            let queues_num = 1;
            let iff_no_pi = IFF_NO_PI as c_short;
            let iff_multi_queue = IFF_MULTI_QUEUE as c_short;
            let packet_information = config.platform_config.packet_information;
            req.ifr_ifru.ifru_flags = device_type
                | if packet_information { 0 } else { iff_no_pi }
                | if queues_num > 1 { iff_multi_queue } else { 0 };

            let tun_fd = {
                let fd = libc::open(b"/dev/net/tun\0".as_ptr() as *const _, O_RDWR);
                let tun_fd = Fd::new(fd, true).map_err(|_| io::Error::last_os_error())?;
                if let Err(err) = tunsetiff(tun_fd.inner, &mut req as *mut _ as *mut _) {
                    return Err(io::Error::from(err).into());
                }
                tun_fd
            };

            let ctl = Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0), true)?;

            let tun_name = CStr::from_ptr(req.ifr_name.as_ptr())
                .to_string_lossy()
                .to_string();
            Device {
                name: RwLock::new(tun_name),
                tun: Tun::new(tun_fd, packet_information),
                ctl,
            }
        };

        configure(&device, config)?;
        Ok(device)
    }

    /// Prepare a new request.
    unsafe fn request(&self) -> ifreq {
        let mut req: ifreq = mem::zeroed();
        let tun_name = self.name.read().unwrap();
        let tun_name = &*tun_name;
        ptr::copy_nonoverlapping(
            tun_name.as_ptr() as *const c_char,
            req.ifr_name.as_mut_ptr(),
            tun_name.len(),
        );

        req
    }

    /// Make the device persistent.
    pub fn persist(&self) -> Result<()> {
        unsafe {
            if let Err(err) = tunsetpersist(self.as_raw_fd(), &1) {
                Err(io::Error::from(err).into())
            } else {
                Ok(())
            }
        }
    }

    /// Set the owner of the device.
    pub fn user(&self, value: i32) -> Result<()> {
        unsafe {
            if let Err(err) = tunsetowner(self.as_raw_fd(), &value) {
                Err(io::Error::from(err).into())
            } else {
                Ok(())
            }
        }
    }

    /// Set the group of the device.
    pub fn group(&self, value: i32) -> Result<()> {
        unsafe {
            if let Err(err) = tunsetgroup(self.as_raw_fd(), &value) {
                Err(io::Error::from(err).into())
            } else {
                Ok(())
            }
        }
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.tun.set_nonblock()
    }

    /// Recv a packet from tun device
    pub(crate) fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.tun.recv(buf)
    }

    /// Send a packet to tun device
    pub(crate) fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.tun.send(buf)
    }
}

impl AbstractDevice for Device {
    fn tun_name(&self) -> Result<String> {
        Ok(self.name.read().unwrap().clone())
    }

    fn set_tun_name(&self, value: &str) -> Result<()> {
        unsafe {
            let tun_name = CString::new(value)?;

            if tun_name.as_bytes_with_nul().len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }

            let mut req = self.request();
            ptr::copy_nonoverlapping(
                tun_name.as_ptr() as *const c_char,
                req.ifr_ifru.ifru_newname.as_mut_ptr(),
                value.len(),
            );

            if let Err(err) = siocsifname(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            *self.name.write().unwrap() = value.into();

            Ok(())
        }
    }

    fn enabled(&self, value: bool) -> Result<()> {
        unsafe {
            let mut req = self.request();

            if let Err(err) = siocgifflags(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }

            if value {
                req.ifr_ifru.ifru_flags |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags &= !(IFF_UP as c_short);
            }

            if let Err(err) = siocsifflags(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_address(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_addr, OVERWRITE_SIZE);
            if let Err(err) = siocsifaddr(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn destination(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifdstaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_dstaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_destination(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_dstaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifdstaddr(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn broadcast(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifbrdaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_broadaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_broadcast(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_broadaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifbrdaddr(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn netmask(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifnetmask(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_netmask);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_netmask(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_netmask, OVERWRITE_SIZE);
            if let Err(err) = siocsifnetmask(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn set_network_address(
        &self,
        address: IpAddr,
        netmask: IpAddr,
        destination: Option<IpAddr>,
    ) -> Result<()> {
        self.set_address(address)?;
        self.set_netmask(netmask)?;
        if let Some(destination) = destination {
            self.set_destination(destination)?;
        }
        Ok(())
    }

    fn mtu(&self) -> Result<u16> {
        unsafe {
            let mut req = self.request();

            if let Err(err) = siocgifmtu(self.ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }

            req.ifr_ifru
                .ifru_mtu
                .try_into()
                .map_err(|_| Error::TryFromIntError)
        }
    }

    fn set_mtu(&self, value: u16) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_mtu = value as i32;

            if let Err(err) = siocsifmtu(self.ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn packet_information(&self) -> bool {
        self.tun.packet_information()
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.tun.as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.tun.into_raw_fd()
    }
}

impl From<Layer> for c_short {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::L2 => IFF_TAP as c_short,
            Layer::L3 => IFF_TUN as c_short,
        }
    }
}
