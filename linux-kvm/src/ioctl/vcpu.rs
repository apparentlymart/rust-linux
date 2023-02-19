use linux_io::fd::ioctl::{
    ioctl_no_arg, ioctl_read, ioctl_write, IoctlReqNoArgs, IoctlReqRead, IoctlReqWrite, _IO, _IOR,
    _IOW,
};
use linux_unsafe::int;

use super::system::KVMIO;

/// The device type marker for the a KVM virtual CPU file descriptor.
#[derive(Debug)]
pub struct KvmVcpu;

impl linux_io::fd::ioctl::IoDevice for KvmVcpu {}

pub const KVM_RUN: IoctlReqNoArgs<KvmVcpu, int> = unsafe { ioctl_no_arg(_IO(KVMIO, 0x80)) };

pub const KVM_GET_REGS: IoctlReqRead<KvmVcpu, crate::raw::kvm_regs> = unsafe {
    ioctl_read(_IOR(
        KVMIO,
        0x81,
        core::mem::size_of::<crate::raw::kvm_regs>() as linux_unsafe::ulong,
    ))
};

pub const KVM_SET_REGS: IoctlReqWrite<KvmVcpu, crate::raw::kvm_regs> = unsafe {
    ioctl_write(_IOW(
        KVMIO,
        0x82,
        core::mem::size_of::<crate::raw::kvm_regs>() as linux_unsafe::ulong,
    ))
};
