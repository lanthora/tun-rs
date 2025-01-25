use crate::platform::{DeviceImpl, SyncDevice};
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};

/// TUN interface OSI layer of operation.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum Layer {
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    L2,
    #[default]
    L3,
}

/// Configuration for a TUN/TAP interface.
#[derive(Clone, Default, Debug)]
pub(crate) struct DeviceConfig {
    pub dev_name: Option<String>,
    #[allow(dead_code)]
    pub layer: Option<Layer>,
    #[cfg(windows)]
    pub device_guid: Option<u128>,
    #[cfg(windows)]
    pub wintun_file: Option<String>,
    #[cfg(windows)]
    pub ring_capacity: Option<u32>,
    /// switch of Enable/Disable packet information for network driver
    #[cfg(any(target_os = "ios", target_os = "macos", target_os = "linux"))]
    pub packet_information: Option<bool>,
    /// Enable/Disable TUN offloads.
    /// After enabling, use `recv_multiple`/`send_multiple` for data transmission.
    #[cfg(target_os = "linux")]
    pub offload: Option<bool>,
    /// Enable multi queue support
    #[cfg(target_os = "linux")]
    pub multi_queue: Option<bool>,
}

/// Builder for a TUN/TAP interface.
#[derive(Default)]
pub struct DeviceBuilder {
    dev_name: Option<String>,
    enabled: Option<bool>,
    mtu: Option<u16>,
    #[cfg(windows)]
    mtu_v6: Option<u16>,
    ipv4: Option<(Ipv4Addr, u8, Option<Ipv4Addr>)>,
    ipv6: Option<Vec<(Ipv6Addr, u8)>>,
    layer: Option<Layer>,
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    mac_addr: Option<[u8; 6]>,
    #[cfg(windows)]
    device_guid: Option<u128>,
    #[cfg(windows)]
    wintun_file: Option<String>,
    #[cfg(windows)]
    ring_capacity: Option<u32>,
    #[cfg(windows)]
    metric: Option<u16>,
    /// switch of Enable/Disable packet information for network driver
    #[cfg(any(target_os = "ios", target_os = "macos", target_os = "linux"))]
    packet_information: Option<bool>,
    #[cfg(target_os = "linux")]
    tx_queue_len: Option<u32>,
    /// Enable/Disable TUN offloads.
    /// After enabling, use `recv_multiple`/`send_multiple` for data transmission.
    #[cfg(target_os = "linux")]
    offload: Option<bool>,
    /// Enable multi queue support
    #[cfg(target_os = "linux")]
    multi_queue: Option<bool>,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn name<S: Into<String>>(mut self, dev_name: S) -> Self {
        self.dev_name = Some(dev_name.into());
        self
    }
    pub fn mtu(mut self, mtu: u16) -> Self {
        self.mtu = Some(mtu);
        self
    }
    #[cfg(windows)]
    pub fn mtu_v6(mut self, mtu: u16) -> Self {
        self.mtu = Some(mtu);
        self
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
    pub fn mac_addr(mut self, mac_addr: [u8; 6]) -> Self {
        self.mac_addr = Some(mac_addr);
        self
    }
    pub fn ipv4<Netmask: ToIpv4Netmask>(
        mut self,
        address: Ipv4Addr,
        mask: Netmask,
        destination: Option<Ipv4Addr>,
    ) -> Self {
        self.ipv4 = Some((address, mask.prefix(), destination));
        self
    }
    pub fn ipv6<Netmask: ToIpv6Netmask>(mut self, address: Ipv6Addr, mask: Netmask) -> Self {
        if let Some(v) = &mut self.ipv6 {
            v.push((address, mask.prefix()));
        } else {
            self.ipv6 = Some(vec![(address, mask.prefix())]);
        }

        self
    }
    pub fn ipv6_tuple<Netmask: ToIpv6Netmask>(mut self, addrs: Vec<(Ipv6Addr, Netmask)>) -> Self {
        if let Some(v) = &mut self.ipv6 {
            for (address, mask) in addrs {
                v.push((address, mask.prefix()));
            }
        } else {
            self.ipv6 = Some(
                addrs
                    .into_iter()
                    .map(|(ip, mask)| (ip, mask.prefix()))
                    .collect(),
            );
        }
        self
    }
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = Some(layer);
        self
    }
    #[cfg(windows)]
    pub fn device_guid(mut self, device_guid: u128) -> Self {
        self.device_guid = Some(device_guid);
        self
    }
    #[cfg(windows)]
    pub fn wintun_file(mut self, wintun_file: String) -> Self {
        self.wintun_file = Some(wintun_file);
        self
    }
    #[cfg(windows)]
    pub fn ring_capacity(mut self, ring_capacity: u32) -> Self {
        self.ring_capacity = Some(ring_capacity);
        self
    }
    #[cfg(windows)]
    pub fn metric(mut self, metric: u16) -> Self {
        self.metric = Some(metric);
        self
    }
    #[cfg(target_os = "linux")]
    pub fn tx_queue_len(mut self, tx_queue_len: u32) -> Self {
        self.tx_queue_len = Some(tx_queue_len);
        self
    }
    /// After enabling, use `recv_multiple`/`send_multiple` for data transmission.
    #[cfg(target_os = "linux")]
    pub fn offload(mut self, offload: bool) -> Self {
        self.offload = Some(offload);
        self
    }
    #[cfg(target_os = "linux")]
    pub fn multi_queue(mut self, multi_queue: bool) -> Self {
        self.multi_queue = Some(multi_queue);
        self
    }

    #[cfg(any(target_os = "ios", target_os = "macos", target_os = "linux"))]
    pub fn packet_information(mut self, packet_information: bool) -> Self {
        self.packet_information = Some(packet_information);
        self
    }
    pub fn enable(mut self, enable: bool) -> Self {
        self.enabled = Some(enable);
        self
    }
    pub(crate) fn build_config(&mut self) -> DeviceConfig {
        DeviceConfig {
            dev_name: self.dev_name.take(),
            layer: self.layer.take(),
            #[cfg(windows)]
            device_guid: self.device_guid.take(),
            #[cfg(windows)]
            wintun_file: self.wintun_file.take(),
            #[cfg(windows)]
            ring_capacity: self.ring_capacity.take(),
            #[cfg(any(target_os = "ios", target_os = "macos", target_os = "linux"))]
            packet_information: self.packet_information.take(),
            #[cfg(target_os = "linux")]
            offload: self.offload.take(),
            #[cfg(target_os = "linux")]
            multi_queue: self.multi_queue.take(),
        }
    }
    pub(crate) fn config(self, device: &DeviceImpl) -> io::Result<()> {
        if let Some(mtu) = self.mtu {
            device.set_mtu(mtu)?;
        }
        #[cfg(windows)]
        if let Some(mtu) = self.mtu_v6 {
            device.set_mtu_v6(mtu)?;
        }
        #[cfg(windows)]
        if let Some(metric) = self.metric {
            device.set_metric(metric)?;
        }
        #[cfg(target_os = "linux")]
        if let Some(tx_queue_len) = self.tx_queue_len {
            device.set_tx_queue_len(tx_queue_len)?;
        }
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd"))]
        if let Some(mac_addr) = self.mac_addr {
            if self.layer.unwrap_or_default() == Layer::L2 {
                device.set_mac_address(mac_addr)?;
            }
        }

        if let Some((address, netmask, destination)) = self.ipv4 {
            device.set_network_address(address, netmask, destination)?;
        }
        if let Some(ipv6) = self.ipv6 {
            for (ip, prefix) in ipv6 {
                device.add_address_v6(ip, prefix)?;
            }
        }
        device.enabled(self.enabled.unwrap_or(true))?;
        Ok(())
    }
    pub fn build_sync(mut self) -> io::Result<SyncDevice> {
        let device = DeviceImpl::new(self.build_config())?;
        self.config(&device)?;
        Ok(SyncDevice(device))
    }
    #[cfg(any(feature = "async_std", feature = "async_tokio"))]
    pub fn build_async(self) -> io::Result<crate::AsyncDevice> {
        let sync_device = self.build_sync()?;
        let device = crate::AsyncDevice::new_dev(sync_device.0)?;
        Ok(device)
    }
}
pub trait ToIpv4Netmask {
    fn prefix(&self) -> u8;
    fn netmask(&self) -> Ipv4Addr {
        let ip = u32::MAX.checked_shl(32 - self.prefix() as u32).unwrap_or(0);
        Ipv4Addr::from(ip)
    }
}
impl ToIpv4Netmask for u8 {
    fn prefix(&self) -> u8 {
        *self
    }
}
impl ToIpv4Netmask for Ipv4Addr {
    fn prefix(&self) -> u8 {
        u32::from_be_bytes(self.octets()).count_ones() as u8
    }
}
pub trait ToIpv6Netmask {
    fn prefix(self) -> u8;
}
impl ToIpv6Netmask for u8 {
    fn prefix(self) -> u8 {
        self
    }
}
impl ToIpv6Netmask for Ipv6Addr {
    fn prefix(self) -> u8 {
        u128::from_be_bytes(self.octets()).count_ones() as u8
    }
}
