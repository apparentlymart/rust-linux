/// The device type marker for the a KVM virtual CPU file descriptor.
pub struct KvmVcpu;

impl linux_io::fd::ioctl::IoDevice for KvmVcpu {}
