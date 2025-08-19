use linux_io::fd::ioctl::{
    ioctl_const_arg, ioctl_read, ioctl_write, ioctl_writeread, IoctlReqConstArg, IoctlReqRead,
    IoctlReqWrite, IoctlReqWriteRead, _IO, _IOR, _IOW,
};
use linux_unsafe::int;

use super::system::KVMIO;

/// The device type marker for the a KVM virtual CPU file descriptor.
#[derive(Debug)]
pub struct KvmVcpu;

impl linux_io::fd::ioctl::IoDevice for KvmVcpu {}

pub const KVM_RUN: IoctlReqConstArg<KvmVcpu, int, 0> = unsafe { ioctl_const_arg(_IO(KVMIO, 0x80)) };

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

pub const KVM_GET_ONE_REG: IoctlReqWriteRead<KvmVcpu, crate::raw::kvm_one_reg> = unsafe {
    ioctl_writeread(_IOR(
        KVMIO,
        0xab,
        core::mem::size_of::<crate::raw::kvm_one_reg>() as linux_unsafe::ulong,
    ))
};

pub const KVM_SET_ONE_REG: IoctlReqWrite<KvmVcpu, crate::raw::kvm_one_reg> = unsafe {
    ioctl_write(_IOW(
        KVMIO,
        0xac,
        core::mem::size_of::<crate::raw::kvm_one_reg>() as linux_unsafe::ulong,
    ))
};
