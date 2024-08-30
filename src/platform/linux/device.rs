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

use crate::configuration::configure;
use crate::{
    configuration::{Configuration, Layer},
    device::{AbstractDevice, ETHER_ADDR_LEN},
    error::{Error, Result},
    platform::linux::sys::*,
    platform::posix::{ipaddr_to_sockaddr, sockaddr_union, Fd, Tun},
    IntoAddress,
};
use libc::{
    self, c_char, c_short, ifreq, AF_INET, ARPHRD_ETHER, IFF_MULTI_QUEUE, IFF_NO_PI, IFF_RUNNING,
    IFF_TAP, IFF_TUN, IFF_UP, IFNAMSIZ, O_RDWR, SOCK_DGRAM,
};
use mac_address::mac_address_by_name;
use std::os::fd::FromRawFd;
use std::{
    ffi::CString,
    io::{self},
    mem,
    net::IpAddr,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
};

const OVERWRITE_SIZE: usize = std::mem::size_of::<libc::__c_anonymous_ifr_ifru>();

/// A TUN device using the TUN/TAP Linux driver.
pub struct Device {
    tun: Tun,
}
impl FromRawFd for Device {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let tun = Fd::new(fd, true).unwrap();
        Device {
            tun: Tun::new(tun, false),
        }
    }
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
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
        unsafe {
            let mut req: ifreq = mem::zeroed();

            if let Some(dev_name) = dev_name.as_ref() {
                ptr::copy_nonoverlapping(
                    dev_name.as_ptr() as *const c_char,
                    req.ifr_name.as_mut_ptr(),
                    dev_name.as_bytes_with_nul().len(),
                );
            }

            let device_type: c_short = config.layer.unwrap_or(Layer::L3).into();
            let queues_num = 1;
            let iff_no_pi = IFF_NO_PI as c_short;
            let iff_multi_queue = IFF_MULTI_QUEUE as c_short;
            let packet_information = config.platform_config.packet_information;
            req.ifr_ifru.ifru_flags = device_type
                | if packet_information { 0 } else { iff_no_pi }
                | if queues_num > 1 { iff_multi_queue } else { 0 };

            let fd = libc::open(b"/dev/net/tun\0".as_ptr() as *const _, O_RDWR);
            let tun_fd = Fd::new(fd, true)?;
            if let Err(err) = tunsetiff(tun_fd.inner, &mut req as *mut _ as *mut _) {
                return Err(io::Error::from(err).into());
            }

            let device = Device {
                tun: Tun::new(tun_fd, config.platform_config.packet_information),
            };
            configure(&device, config)?;
            Ok(device)
        }
    }

    /// Prepare a new request.
    unsafe fn request(&self) -> Result<ifreq> {
        request(&self.name()?)
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
    #[cfg(feature = "experimental")]
    pub(crate) fn shutdown(&self) -> io::Result<()> {
        self.tun.shutdown()
    }
    fn set_address(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_addr, OVERWRITE_SIZE);
            if let Err(err) = siocsifaddr(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }
    fn set_netmask(&self, value: IpAddr) -> Result<()> {
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_netmask, OVERWRITE_SIZE);
            if let Err(err) = siocsifnetmask(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn set_destination<A: IntoAddress>(&self, value: A) -> Result<()> {
        let value = value.into_address()?;
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_dstaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifdstaddr(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }
}
unsafe fn ctl() -> io::Result<Fd> {
    Ok(Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0), true)?)
}
unsafe fn name(fd: RawFd) -> io::Result<String> {
    let mut req: ifreq = mem::zeroed();
    if let Err(err) = tungetiff(fd, &mut req as *mut _ as *mut _) {
        return Err(io::Error::from(err));
    }
    let c_str = std::ffi::CStr::from_ptr(req.ifr_name.as_ptr() as *const c_char);
    let tun_name = c_str.to_string_lossy().into_owned();
    Ok(tun_name)
}
unsafe fn request(name: &str) -> Result<ifreq> {
    let mut req: ifreq = mem::zeroed();
    ptr::copy_nonoverlapping(
        name.as_ptr() as *const c_char,
        req.ifr_name.as_mut_ptr(),
        name.len(),
    );
    Ok(req)
}
impl AbstractDevice for Device {
    fn name(&self) -> Result<String> {
        unsafe { name(self.as_raw_fd()).map_err(|e| e.into()) }
    }

    fn set_name(&self, value: &str) -> Result<()> {
        unsafe {
            let tun_name = CString::new(value)?;

            if tun_name.as_bytes_with_nul().len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }

            let mut req = self.request()?;
            ptr::copy_nonoverlapping(
                tun_name.as_ptr() as *const c_char,
                req.ifr_ifru.ifru_newname.as_mut_ptr(),
                value.len(),
            );

            if let Err(err) = siocsifname(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }

            Ok(())
        }
    }

    fn enabled(&self, value: bool) -> Result<()> {
        unsafe {
            let ctl = ctl()?;
            let mut req = self.request()?;

            if let Err(err) = siocgifflags(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }

            if value {
                req.ifr_ifru.ifru_flags |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags &= !(IFF_UP as c_short);
            }

            if let Err(err) = siocsifflags(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifaddr(ctl()?.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn destination(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifdstaddr(ctl()?.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_dstaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn broadcast(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifbrdaddr(ctl()?.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_broadaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_broadcast<A: IntoAddress>(&self, value: A) -> Result<()> {
        let value = value.into_address()?;
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_broadaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifbrdaddr(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn netmask(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifnetmask(ctl()?.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_netmask);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_network_address<A: IntoAddress>(
        &self,
        address: A,
        netmask: A,
        destination: Option<A>,
    ) -> Result<()> {
        self.set_address(address.into_address()?)?;
        self.set_netmask(netmask.into_address()?)?;
        if let Some(destination) = destination {
            self.set_destination(destination.into_address()?)?;
        }
        Ok(())
    }

    fn mtu(&self) -> Result<u16> {
        unsafe {
            let mut req = self.request()?;

            if let Err(err) = siocgifmtu(ctl()?.as_raw_fd(), &mut req) {
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
            let mut req = self.request()?;
            req.ifr_ifru.ifru_mtu = value as i32;

            if let Err(err) = siocsifmtu(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn packet_information(&self) -> bool {
        self.tun.packet_information()
    }

    fn set_mac_address(&self, eth_addr: [u8; ETHER_ADDR_LEN as usize]) -> Result<()> {
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_hwaddr.sa_family = ARPHRD_ETHER;
            req.ifr_ifru.ifru_hwaddr.sa_data[0..ETHER_ADDR_LEN as usize]
                .copy_from_slice(eth_addr.map(|c| c as i8).as_slice());
            if let Err(err) = siocsifhwaddr(ctl()?.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn get_mac_address(&self) -> Result<[u8; ETHER_ADDR_LEN as usize]> {
        let mac = mac_address_by_name(&self.name()?)
            .map_err(|e| Error::String(e.to_string()))?
            .ok_or(Error::InvalidConfig)?;
        Ok(mac.bytes())
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
