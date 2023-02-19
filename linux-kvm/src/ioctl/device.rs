/// The device type marker for the a KVM device file descriptor.
#[derive(Debug)]
pub struct KvmDevice;

impl linux_io::fd::ioctl::IoDevice for KvmDevice {}
