use linux_io::fd::ioctl::{
    ioctl_no_arg, ioctl_read, ioctl_write, IoctlReqNoArgs, IoctlReqRead, IoctlReqWrite, _IO, _IOR,
    _IOW, _IOWR,
};
use linux_io::File;
use linux_unsafe::{int, ulong};

/// The device type marker for the main KVM file descriptor, typically obtained
/// by opening `/dev/kvm`.
pub struct KvmSystem;

impl linux_io::fd::ioctl::IoDevice for KvmSystem {}

pub(crate) const KVMIO: ulong = 0xAE;

/// Identifies the version of the KVM API used by the current kernel.
///
/// The stable API always returns version 12. The kernel documentation suggests
/// that applications should always call this and refuse to run if it returns
/// any value other than that; the version number is not expected to change
/// in the future because future API additions will use [`KVM_CHECK_EXTENSION`]
/// instead.
pub const KVM_GET_API_VERSION: IoctlReqNoArgs<KvmSystem, int> =
    unsafe { ioctl_no_arg(_IO(KVMIO, 0x00)) };

/// Create a new virtual machine and obtain the file that represents it.
///
/// The resulting file accepts the `ioctl` requests defined in [`super::vm`].
pub const KVM_CREATE_VM: IoctlReqNoArgs<KvmSystem, File<super::vm::KvmVm>> =
    unsafe { ioctl_no_arg(_IO(KVMIO, 0x01)) };

/// Query whether the KVM subsystem in the current kernel supports a particular
/// extension.
///
/// A result of zero indicates a lack of support while nonzero indicates
/// support. The nonzero value may carry additional meanings for some
/// extensions.
///
/// This is also supported for virtual machine file descriptors, but you must
/// use [`super::vm::KVM_CHECK_EXTENSION`] instead for those.
pub const KVM_CHECK_EXTENSION: IoctlReqWrite<KvmSystem, int, int> =
    unsafe { ioctl_write(_IO(KVMIO, 0x03)) };
