/// The device type marker for the a KVM device file descriptor.
pub struct KvmDevice;

impl linux_io::fd::ioctl::IoDevice for KvmDevice {}
