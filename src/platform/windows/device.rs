use std::io;
use std::sync::Arc;

use wintun::{load_from_path, Packet, Session};

use crate::configuration::{configure, Configuration};
use crate::device::{AbstractDevice, ETHER_ADDR_LEN};
use crate::error::Result;
use crate::getifaddrs::Interface;
use crate::platform::windows::netsh;
use crate::platform::windows::tap::TapDevice;
use crate::{IntoAddress, Layer};
use std::net::IpAddr;

pub enum Driver {
    Tun(Tun),
    #[allow(dead_code)]
    Tap(TapDevice),
}
pub enum PacketVariant {
    Tun(Packet),
    Tap(Box<[u8]>),
}
impl Driver {
    pub(crate) fn index(&self) -> Result<u32> {
        match self {
            Driver::Tun(tun) => {
                let index = tun.session.get_adapter().get_adapter_index()?;
                Ok(index)
            }
            Driver::Tap(tap) => Ok(tap.index()),
        }
    }
    // pub fn name(&self) -> Result<String> {
    //     match self {
    //         Driver::Tun(tun) => {
    //             let name = tun.session.get_adapter().get_name()?;
    //             Ok(name)
    //         }
    //         Driver::Tap(tap) => {
    //             let name = tap.get_name()?;
    //             Ok(name)
    //         }
    //     }
    // }
    pub fn read_by_ref(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Driver::Tap(tap) => tap.read(buf),
            Driver::Tun(tun) => tun.read_by_ref(buf),
        }
    }
    pub fn try_read_by_ref(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Driver::Tap(tap) => tap.try_read(buf),
            Driver::Tun(tun) => tun.try_read_by_ref(buf),
        }
    }
    pub fn write_by_ref(&self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Driver::Tap(tap) => tap.write(buf),
            Driver::Tun(tun) => tun.write_by_ref(buf),
        }
    }
    pub fn try_write_by_ref(&self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Driver::Tap(tap) => tap.try_write(buf),
            Driver::Tun(tun) => tun.try_write_by_ref(buf),
        }
    }
    pub fn receive_blocking(&self) -> std::io::Result<PacketVariant> {
        match self {
            Driver::Tun(tun) => {
                let packet = tun.session.receive_blocking()?;
                Ok(PacketVariant::Tun(packet))
            }
            Driver::Tap(tap) => {
                let mut buf = [0u8; u16::MAX as usize];
                let len = tap.read(&mut buf)?;
                let mut vec = vec![];
                vec.extend_from_slice(&buf[..len]);
                Ok(PacketVariant::Tap(vec.into_boxed_slice()))
            }
        }
    }
    pub fn try_receive(&self) -> std::io::Result<Option<PacketVariant>> {
        match self {
            Driver::Tun(tun) => match tun.session.try_receive()? {
                None => Ok(None),
                Some(packet) => Ok(Some(PacketVariant::Tun(packet))),
            },
            Driver::Tap(tap) => {
                const MAX_LEN: usize = u16::MAX as usize;
                let mut buf = Vec::with_capacity(MAX_LEN);
                // guarantee all read bytes are initialized by the modification of the read function.
                #[allow(clippy::uninit_vec)]
                unsafe {
                    buf.set_len(MAX_LEN);
                };
                match tap.try_read(&mut buf) {
                    Ok(len) => {
                        buf.resize(len, 0);
                        Ok(Some(PacketVariant::Tap(buf.into_boxed_slice())))
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            return Ok(None);
                        }
                        Err(e)
                    }
                }
            }
        }
    }
}

/// A TUN device using the wintun driver.
pub struct Device {
    pub(crate) driver: Driver,
}

macro_rules! driver_case {
    ($driver:expr; $tun:ident =>  $tun_branch:block; $tap:ident => $tap_branch:block) => {
        match $driver {
            Driver::Tun($tun) => $tun_branch
            Driver::Tap($tap) => $tap_branch
        }
    };
}
fn hash_name(input_str: &str) -> u128 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    8765028472139845610u64.hash(&mut hasher);
    input_str.hash(&mut hasher);
    let front = hasher.finish();

    let mut hasher = std::hash::DefaultHasher::new();
    12874056902134875693u64.hash(&mut hasher);
    input_str.hash(&mut hasher);
    let back = hasher.finish();
    (u128::from(front) << 64) | u128::from(back)
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let layer = config.layer.unwrap_or(Layer::L3);
        let mut count = 0;
        let device = if layer == Layer::L3 {
            let wintun_file = &config.platform_config.wintun_file;
            let wintun = unsafe { load_from_path(wintun_file)? };
            let mut attempts = 0;
            let adapter = loop {
                let default_name = format!("tun{count}");
                let name = config.name.as_deref().unwrap_or(&default_name);

                match wintun::Adapter::open(&wintun, name) {
                    Ok(a) => {
                        if config.name.is_none() {
                            count += 1;
                            continue;
                        }
                        break a;
                    }
                    Err(_e) => {
                        let mut guid = config.platform_config.device_guid;
                        if guid.is_none() {
                            guid.replace(hash_name(name));
                        }
                        if config.platform_config.delete_reg {
                            if let Err(e) = delete_reg(name) {
                                log::warn!("delete_reg {e:?}")
                            }
                        }
                        match wintun::Adapter::create(&wintun, name, name, guid) {
                            Ok(adapter) => break adapter,
                            Err(e) => {
                                if attempts > 3 {
                                    Err(e)?
                                } else {
                                    count += 1;
                                    attempts += 1;
                                }
                            }
                        }
                    }
                }
            };

            #[cfg(feature = "wintun-dns")]
            if let Some(dns_servers) = &config.platform_config.dns_servers {
                adapter.set_dns_servers(dns_servers)?;
            }

            let session = adapter.start_session(
                config
                    .platform_config
                    .ring_capacity
                    .unwrap_or(wintun::MAX_RING_CAPACITY),
            )?;
            Device {
                driver: Driver::Tun(Tun { session }),
            }
        } else if layer == Layer::L2 {
            const HARDWARE_ID: &str = "tap0901";
            let tap = loop {
                let default_name = format!("tap{count}");
                let name = config.name.as_deref().unwrap_or(&default_name);
                if let Ok(tap) = TapDevice::open(HARDWARE_ID, name) {
                    if config.name.is_none() {
                        count += 1;
                        continue;
                    }
                    break tap;
                } else {
                    let tap = TapDevice::create(HARDWARE_ID)?;
                    if let Err(e) = tap.set_name(name) {
                        if config.name.is_some() {
                            Err(e)?
                        }
                    }
                    break tap;
                }
            };
            Device {
                driver: Driver::Tap(tap),
            }
        } else {
            panic!("unknown layer {:?}", layer);
        };
        configure(&device, config)?;
        if let Some(metric) = config.platform_config.metric {
            netsh::set_interface_metric(device.driver.index()?, metric)?;
        }
        Ok(device)
    }

    /// Recv a packet from tun device
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.read_by_ref(buf)
    }
    pub fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.driver.try_read_by_ref(buf)
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.driver.write_by_ref(buf)
    }
    pub fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
        self.driver.try_write_by_ref(buf)
    }
    pub fn shutdown(&self) -> io::Result<()> {
        driver_case!(
            &self.driver;
            tun=>{
                tun.get_session().shutdown().map_err(|e|io::Error::new(io::ErrorKind::Other,format!("{:?}",e)))
            };
            tap=>{
               tap.down()
            }
        )
    }
    pub fn get_all_adapter_address(&self) -> Result<Vec<Interface>, crate::Error> {
        crate::getifaddrs::windows::get_adapters_addresses()
    }
}

impl AbstractDevice for Device {
    fn name(&self) -> Result<String> {
        driver_case!(
            &self.driver;
            tun=>{
                Ok(tun.session.get_adapter().get_name()?)
            };
            tap=>{
               Ok(tap.get_name()?)
            }
        )
    }

    fn set_name(&self, value: &str) -> Result<()> {
        driver_case!(
            &self.driver;
            tun=>{
                tun.session.get_adapter().set_name(value)?;
            };
            tap=>{
               tap.set_name(value)?
            }
        );
        Ok(())
    }

    fn if_index(&self) -> Result<u32> {
        self.driver.index()
    }

    fn enabled(&self, value: bool) -> Result<()> {
        driver_case!(
            &self.driver;
            tun=>{
                if !value{
                    tun.session.shutdown()?;
                }
            };
            tap=>{
                 if value{
                    tap.up()?
                 }else{
                     tap.down()?
                }
            }
        );
        Ok(())
    }

    fn addresses(&self) -> Result<Vec<Interface>> {
        driver_case!(
            &self.driver;
            tun=>{
                let tun_index = tun.session.get_adapter().get_adapter_index()?;
                let r = self.get_all_adapter_address()?.into_iter().filter(|v|v.index == Some(tun_index)).collect();
                Ok(r)
            };
            tap=>{
                let tap_index = tap.index();
                let r = self.get_all_adapter_address()?.into_iter().filter(|v|v.index == Some(tap_index)).collect();
                Ok(r)
            }
        )
    }

    fn set_network_address<A: IntoAddress>(
        &self,
        address: A,
        netmask: A,
        destination: Option<A>,
    ) -> Result<()> {
        let destination = if let Some(destination) = destination {
            Some(destination.into_address()?)
        } else {
            None
        };
        if let Ok(addr) = self.addresses() {
            for e in addr {
                if e.address.is_ipv6() {
                    if let Err(e) = netsh::delete_interface_ip(self.driver.index()?, e.address) {
                        log::error!("{e:?}");
                    }
                }
            }
        }

        netsh::set_interface_ip(
            self.driver.index()?,
            address.into_address()?,
            netmask.into_address()?,
            destination,
        )?;
        Ok(())
    }

    fn remove_network_address(&self, addrs: Vec<(IpAddr, u8)>) -> Result<()> {
        for addr in addrs {
            if let Err(e) = netsh::delete_interface_ip(self.driver.index()?, addr.0) {
                return Err(crate::Error::String(e.to_string()));
            }
        }
        Ok(())
    }

    fn add_address_v6(&self, addr: IpAddr, prefix: u8) -> Result<()> {
        if !addr.is_ipv6() {
            return Err(crate::Error::InvalidAddress);
        }
        let network_addr =
            ipnet::IpNet::new(addr, prefix).map_err(|e| crate::Error::String(e.to_string()))?;
        let mask = network_addr.netmask();
        netsh::set_interface_ip(self.driver.index()?, addr, mask, None)?;
        Ok(())
    }

    /// The return value is always `Ok(65535)` due to wintun
    fn mtu(&self) -> Result<u16> {
        driver_case!(
              &self.driver;
            tun=>{
                let mtu = tun.session.get_adapter().get_mtu()?;
                Ok(mtu as _)
            };
            tap=>{
                let mtu = tap.get_mtu()?;
                 Ok(mtu as _)
            }
        )
    }

    /// This setting has no effect since the mtu of wintun is always 65535
    fn set_mtu(&self, mtu: u16) -> Result<()> {
        driver_case!(
            &self.driver;
            tun=>{
                tun.session.get_adapter().set_mtu(mtu as _)?;
                Ok(())
            };
            tap=>{
                tap.set_mtu(mtu).map_err(|e|e.into())
            }
        )
    }

    fn set_mac_address(&self, eth_addr: [u8; ETHER_ADDR_LEN as usize]) -> Result<()> {
        driver_case!(
            &self.driver;
            _tun=>{
                Err(io::Error::from(io::ErrorKind::Unsupported))?
            };
            tap=>{
                tap.set_mac(&eth_addr).map_err(|e|e.into())
            }
        )
    }

    fn get_mac_address(&self) -> Result<[u8; ETHER_ADDR_LEN as usize]> {
        driver_case!(
            &self.driver;
            _tun=>{
                Err(io::Error::from(io::ErrorKind::Unsupported))?
            };
            tap=>{
                tap.get_mac().map_err(|e|e.into())
            }
        )
    }

    fn set_metric(&self, metric: u16) -> Result<()> {
        netsh::set_interface_metric(self.if_index()?, metric)?;
        Ok(())
    }
}

pub struct Tun {
    session: Arc<Session>,
}

impl Tun {
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }
    fn read_by_ref(&self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.session.receive_blocking() {
            Ok(pkt) => match io::copy(&mut pkt.bytes(), &mut buf) {
                Ok(n) => Ok(n as usize),
                Err(e) => Err(e),
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, e)),
        }
    }
    fn try_read_by_ref(&self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.session.try_receive() {
            Ok(Some(pkt)) => match io::copy(&mut pkt.bytes(), &mut buf) {
                Ok(n) => Ok(n as usize),
                Err(e) => Err(e),
            },
            Ok(None) => Err(io::Error::from(io::ErrorKind::WouldBlock)),
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, e)),
        }
    }
    fn write_by_ref(&self, mut buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        match self.session.allocate_send_packet(size as u16) {
            Err(e) => match e {
                // if (GetLastError() != ERROR_BUFFER_OVERFLOW) // Silently drop packets if the ring is full
                wintun::Error::Io(io_err) => Err(io_err),
                e => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
            },
            Ok(mut packet) => match io::copy(&mut buf, &mut packet.bytes_mut()) {
                Ok(s) => {
                    self.session.send_packet(packet);
                    Ok(s as usize)
                }
                Err(e) => Err(e),
            },
        }
    }
    fn try_write_by_ref(&self, mut buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        match self.session.allocate_send_packet(size as u16) {
            Err(e) => match e {
                wintun::Error::Io(io_err) => {
                    if io_err.raw_os_error().unwrap_or(0)
                        == windows_sys::Win32::Foundation::ERROR_BUFFER_OVERFLOW as i32
                    {
                        Err(io::Error::from(io::ErrorKind::WouldBlock))
                    } else {
                        Err(io_err)
                    }
                }
                e => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
            },
            Ok(mut packet) => match io::copy(&mut buf, &mut packet.bytes_mut()) {
                Ok(s) => {
                    self.session.send_packet(packet);
                    Ok(s as usize)
                }
                Err(e) => Err(e),
            },
        }
    }
}

pub fn delete_reg(dev_name: &str) -> io::Result<()> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, enums::KEY_ALL_ACCESS, RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let profiles_key = hklm.open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\NetworkList\\Profiles",
        KEY_ALL_ACCESS,
    )?;
    let unmanaged_key = hklm.open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\NetworkList\\Signatures\\Unmanaged",
        KEY_ALL_ACCESS,
    )?;

    // let network_key = hklm.open_subkey_with_flags(
    //     "SYSTEM\\CurrentControlSet\\Control\\Network",
    //     KEY_ALL_ACCESS,
    // )?;
    for sub_key_name in profiles_key.enum_keys().filter_map(Result::ok) {
        let sub_key = profiles_key.open_subkey(&sub_key_name)?;
        match sub_key.get_value::<String, _>("ProfileName") {
            Ok(profile_name) => {
                if dev_name == profile_name {
                    match profiles_key.delete_subkey_all(&sub_key_name) {
                        Ok(_) => {
                            log::info!("Successfully deleted Profiles sub_key: {}", sub_key_name)
                        }
                        Err(e) => {
                            log::warn!("Failed to delete Profiles sub_key {}: {}", sub_key_name, e)
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to read ProfileName for sub_key {}: {}",
                    sub_key_name,
                    e
                );
            }
        }
    }
    for sub_key_name in unmanaged_key.enum_keys().filter_map(Result::ok) {
        let sub_key = unmanaged_key.open_subkey(&sub_key_name)?;
        match sub_key.get_value::<String, _>("Description") {
            Ok(description) => {
                if dev_name == description {
                    match unmanaged_key.delete_subkey_all(&sub_key_name) {
                        Ok(_) => {
                            log::info!("Successfully deleted Unmanaged sub_key: {}", sub_key_name)
                        }
                        Err(e) => {
                            log::warn!("Failed to delete Unmanaged sub_key {}: {}", sub_key_name, e)
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to read Description for sub_key {}: {}",
                    sub_key_name,
                    e
                );
            }
        }
    }
    // for sub_key_name1 in network_key.enum_keys().filter_map(Result::ok) {
    //     if let Ok(sub_key1) = network_key.open_subkey(&sub_key_name1){
    //         for sub_key_name2 in sub_key1.enum_keys().filter_map(Result::ok) {
    //             if let Ok(sub_key2) = sub_key1.open_subkey(&sub_key_name2){
    //                 for sub_key_name3 in sub_key2.enum_keys().filter_map(Result::ok) {
    //                     if let Ok(sub_key3) = sub_key2.open_subkey(&sub_key_name3){
    //                         if let Ok(name) = sub_key3.get_value::<String, _>("Name"){
    //                             if dev_name == name {
    //                                 match sub_key1.delete_subkey_all(&sub_key_name2) {
    //                                     Ok(_) => {
    //                                         log::info!("Successfully deleted Network sub_key: {}", sub_key_name2)
    //                                     }
    //                                     Err(e) => {
    //                                         log::warn!("Failed to delete Network sub_key {}: {}", sub_key_name2, e)
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //
    //             }
    //         }
    //     }
    // }
    Ok(())
}
