use linux_io::fd::ioctl::{ioctl_no_arg, IoctlReqNoArgs, _IO};
use linux_unsafe::int;

use super::system::KVMIO;

/// The device type marker for the a KVM virtual CPU file descriptor.
#[derive(Debug)]
pub struct KvmVcpu;

impl linux_io::fd::ioctl::IoDevice for KvmVcpu {}

pub const KVM_RUN: IoctlReqNoArgs<KvmVcpu, int> = unsafe { ioctl_no_arg(_IO(KVMIO, 0x80)) };
