mod sys;

mod checksum;
mod device;
pub(crate) mod offload;
pub(crate) use device::DeviceInner;
pub use offload::ExpandBuffer;
pub use offload::GROTable;
pub use offload::IDEAL_BATCH_SIZE;
pub use offload::VIRTIO_NET_HDR_LEN;
