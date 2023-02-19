use linux_io::fd::ioctl::{
    ioctl_no_arg, ioctl_read, ioctl_write, IoctlReqNoArgs, IoctlReqRead, IoctlReqWrite, _IO, _IOR,
    _IOW, _IOWR,
};
use linux_unsafe::{int, ulong};

use super::system::KVMIO;
use linux_io::File;

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
    unsafe { ioctl_write(_IOW(KVMIO, 0x03, core::mem::size_of::<int>() as ulong)) };

/// Create a new virtual CPU for an existing virtual machine and obtain the
/// file that represents it.
///
/// The argument is a VCPU ID, which ranges from zero to the maximum number of
/// supported VCPUs per VM, which is a kernel-decided limit.
///
/// The resulting file accepts the `ioctl` requests defined in [`super::vcpu`].
pub const KVM_CREATE_VCPU: IoctlReqWrite<KvmVm, int, File<super::vcpu::KvmVcpu>> =
    unsafe { ioctl_write(_IOW(KVMIO, 0x41, core::mem::size_of::<int>() as ulong)) };

/// Create, modify or delete a guest physical memory slot.
pub const KVM_SET_USER_MEMORY_REGION: IoctlReqWrite<
    KvmVm,
    crate::raw::kvm_userspace_memory_region,
    int,
> = unsafe {
    ioctl_write(_IOW(
        KVMIO,
        0x46,
        core::mem::size_of::<crate::raw::kvm_userspace_memory_region>() as ulong,
    ))
};
