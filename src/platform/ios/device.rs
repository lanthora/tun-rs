use crate::platform::unix::Tun;

/// A TUN device for iOS.
pub struct DeviceImpl {
    pub(crate) tun: Tun,
}
impl DeviceImpl {
    pub(crate) fn from_tun(tun: Tun) -> Self {
        Self { tun }
    }

    pub fn ignore_packet_info(&self) -> bool {
        self.tun.ignore_packet_info()
    }

    pub fn set_ignore_packet_info(&self, ign: bool) {
        self.tun.set_ignore_packet_info(ign)
    }
}
