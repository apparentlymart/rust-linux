use linux_io::fd::ioctl::{
    ioctl_no_arg, ioctl_read, ioctl_write, IoctlReqNoArgs, IoctlReqRead, IoctlReqWrite, _IO, _IOR,
    _IOW, _IOWR,
};
use linux_unsafe::int;

use super::system::KVMIO;

/// The device type marker for the a KVM virtual machine file descriptor.
pub struct KvmVm;

impl linux_io::fd::ioctl::IoDevice for KvmVm {}

/// Query whether the KVM subsystem in the current kernel supports a particular
/// extension for a specific VM.
///
/// A result of zero indicates a lack of support while nonzero indicates
/// support. The nonzero value may carry additional meanings for some
/// extensions.
pub const KVM_CHECK_EXTENSION: IoctlReqWrite<KvmVm, int, int> =
    unsafe { ioctl_write(_IO(KVMIO, 0x03)) };
