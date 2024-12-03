mod device;
mod ffi;
mod netsh;
mod tap;

use crate::configuration::Configuration;
use crate::error::Result;
pub use device::Driver;
pub use device::{Device, PacketVariant, Tun};
use std::ffi::OsString;
use std::net::IpAddr;

#[allow(dead_code)]
pub(crate) const WINTUN_PROVIDER: &str = "WireGuard LLC";

/// Windows-only interface configuration.
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub(crate) device_guid: Option<u128>,
    pub(crate) wintun_file: OsString,
    #[cfg(feature = "wintun-dns")]
    pub(crate) dns_servers: Option<Vec<IpAddr>>,
    pub(crate) ring_capacity: Option<u32>,
    pub(crate) metric: Option<u16>,
    pub(crate) delete_reg: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            device_guid: None,
            wintun_file: "wintun.dll".into(),
            #[cfg(feature = "wintun-dns")]
            dns_servers: None,
            ring_capacity: None,
            metric: None,
            delete_reg: true,
        }
    }
}

impl PlatformConfig {
    pub fn device_guid(&mut self, device_guid: u128) {
        log::trace!("Windows configuration device GUID");
        self.device_guid = Some(device_guid);
    }

    /// Use a custom path to the wintun.dll instead of looking in the working directory.
    /// Security note: It is up to the caller to ensure that the library can be safely loaded from
    /// the indicated path.
    ///
    /// [`wintun_file`](PlatformConfig::wintun_file) likes "path/to/wintun" or "path/to/wintun.dll".
    pub fn wintun_file<S: Into<OsString>>(&mut self, wintun_file: S) {
        self.wintun_file = wintun_file.into();
    }

    #[cfg(feature = "wintun-dns")]
    pub fn dns_servers(&mut self, dns_servers: &[IpAddr]) {
        self.dns_servers = Some(dns_servers.to_vec());
    }

    pub fn ring_capacity(&mut self, ring_capacity: u32) -> &mut Self {
        self.ring_capacity = Some(ring_capacity);
        self
    }

    pub fn metric(&mut self, metric: u16) -> &mut Self {
        self.metric = Some(metric);
        self
    }
    /// Delete the registry related to the current network card
    pub fn delete_reg(&mut self, delete_reg: bool) -> &mut Self {
        self.delete_reg = delete_reg;
        self
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
