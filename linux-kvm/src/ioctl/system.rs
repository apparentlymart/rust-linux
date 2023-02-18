use linux_io::fd::ioctl::{
    ioctl_no_arg, ioctl_read, ioctl_write, IoctlReqNoArgs, IoctlReqRead, IoctlReqWrite, _IO, _IOR,
    _IOW, _IOWR,
};
use linux_io::File;
use linux_unsafe::{int, ulong};

const KVMIO: ulong = 0xAE;

pub const KVM_GET_API_VERSION: IoctlReqNoArgs<int> = unsafe { ioctl_no_arg(_IO(KVMIO, 0x00)) };
pub const KVM_CREATE_VM: IoctlReqNoArgs<File> = unsafe { ioctl_no_arg(_IO(KVMIO, 0x01)) };
