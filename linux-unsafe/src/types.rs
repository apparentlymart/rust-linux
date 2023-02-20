#![allow(non_camel_case_types)]

use core::ffi;

/// The primary signed integer type for the current platform.
pub type int = ffi::c_int;

/// The primary unsigned integer type for the current platform.
pub type uint = ffi::c_uint;

/// The short signed integer type for the current platform.
pub type short = ffi::c_short;

/// The short unsigned integer type for the current platform.
pub type ushort = ffi::c_ushort;

/// The signed long integer type for the current platform.
pub type long = ffi::c_long;

/// The unsigned long integer type for the current platform.
pub type ulong = ffi::c_ulong;

/// The signed long long integer type for the current platform.
pub type longlong = ffi::c_long;

/// The unsigned long long integer type for the current platform.
pub type ulonglong = ffi::c_ulong;

/// The signed size type (or "pointer difference" type) for the current platform.
pub type ssize_t = isize;

/// The unsigned size type for the current platform.
pub type size_t = usize;

/// The type used for characters on the current platform.
pub type char = ffi::c_char;

/// The type used for unsigned characters on the current platform.
pub type uchar = ffi::c_uchar;

/// The type used for void pointers on the current platform.
pub type void = ffi::c_void;

/// The type used to represent file modes on the current platform.
pub type mode_t = uint;

/// The type used to represent file sizes and offsets into files on the current platform.
pub type off_t = long;

/// The type used to represent larger file sizes and offsets into files on the current platform.
pub type loff_t = ffi::c_longlong;

/// The type used for process identifiers (PIDs) on the current platform.
pub type pid_t = int;

/// The type used for representing socket addresses in the raw system calls.
///
/// This is a type of unknown length, because the actual length of a socket
/// address depends on the address family. When reading a socket address of
/// an unknown address family, use a pointer to a [`sockaddr_storage`] as a
/// placeholder type and then convert based on the returned `family`.
///
/// **Warning:** It is not meaningful to ask the compiler for the size of
/// this type; it will return an arbitrary placeholder value. If you need
/// sufficient storage for an arbitrary address, use [`sockaddr_storage`]
/// instead.
#[repr(C)]
pub struct sockaddr {
    pub family: sa_family_t,

    // Intentionally not public to discourage using values of this type;
    // it's here primarily as a placeholder type for arguments that are
    // pointers to arbitrary addresses.
    data: [u8; 14],
}

/// Represents the upper limit for the size of any [`sockaddr`] value, across
/// all address families.
///
/// This is a reasonable default type to use when retrieving a socket address
/// from the kernel without knowledge of its address family. It is guaranteed
/// at least as large as the largest address type the kernel can return.
/// After the value is populated, use `family` to convert to a more specific
/// address type.
#[repr(C, align(8))]
pub struct sockaddr_storage {
    pub family: sa_family_t,
    pub data: [u8; 128 - core::mem::size_of::<sa_family_t>()],
}

/// The type used for representing the length of a socket address.
pub type socklen_t = int;

/// The type used to represent user ids.
pub type uid_t = uint;

/// The type used to represent group ids.
pub type gid_t = uint;

/// Seek relative to the beginning of the file.
pub const SEEK_SET: int = 0;

/// Seek relative to the current file position.
pub const SEEK_CUR: int = 1;

/// Seek relative to the end of the file.
pub const SEEK_END: int = 2;

/// Seek to the next data.
pub const SEEK_DATA: int = 3;

/// Seek to the next hole.
pub const SEEK_HOLE: int = 4;

pub const O_ACCMODE: int = 0o00000003;
pub const O_RDONLY: int = 0o00000000;
pub const O_WRONLY: int = 0o00000001;
pub const O_RDWR: int = 0o00000002;
pub const O_CREAT: int = 0o00000100;
pub const O_EXCL: int = 0o00000200;
pub const O_NOCTTY: int = 0o00000400;
pub const O_TRUNC: int = 0o00001000;
pub const O_APPEND: int = 0o00002000;
pub const O_NONBLOCK: int = 0o00004000;
pub const O_DSYNC: int = 0o00010000;
pub const O_DIRECT: int = 0o00040000;
pub const O_LARGEFILE: int = 0o00100000;
pub const O_DIRECTORY: int = 0o00200000;
pub const O_NOFOLLOW: int = 0o00400000;
pub const O_NOATIME: int = 0o01000000;
pub const O_CLOEXEC: int = 0o02000000;
pub const O_SYNC: int = 0o04000000 | O_DSYNC;
pub const O_PATH: int = 0o010000000;
pub const O_TMPFILE: int = 0o020000000 | O_DIRECTORY;
pub const O_TMPFILE_MASK: int = 0o020000000 | O_DIRECTORY | O_CREAT;
pub const O_NDELAY: int = O_NONBLOCK;

/// A file descriptor request object for use with [`crate::poll`].
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct pollfd {
    pub fd: int,
    pub events: short,
    pub revents: short,
}

// The type used to specify the number of file descriptors when calling [`crate::poll`].
pub type nfds_t = uint;

pub const POLLIN: short = 0x0001;
pub const POLLPRI: short = 0x0002;
pub const POLLOUT: short = 0x0004;
pub const POLLERR: short = 0x0008;
pub const POLLHUP: short = 0x0010;
pub const POLLNVAL: short = 0x0020;

/// A type used with [`crate::readv`] and [`crate::writev`].
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct iovec {
    pub iov_base: *mut void,
    pub iov_len: size_t,
}

/// A type used with [`crate::epoll_ctl`].
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct epoll_event {
    pub events: u32,
    pub data: epoll_data,
}

/// A type used with [`crate::epoll_ctl`].
#[derive(Clone, Copy)]
#[repr(C)]
pub union epoll_data {
    pub ptr: *mut void,
    pub fd: int,
    pub u32: u32,
    pub u64: u64,
}

impl core::fmt::Debug for epoll_data {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("epoll_data").finish_non_exhaustive()
    }
}

/// A type used with some [`crate::fcntl`] commands.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct flock {
    pub l_type: short,
    pub l_whence: short,
    pub l_start: off_t,
    pub l_len: off_t,
    pub l_pid: pid_t,
    // TODO: MIPS Linux has an extra field l_sysid and some padding.
    // We don't support MIPS yet so we're ignoring that, but we'll
    // need to deal with that if we add MIPS support later.
    // Sparc also has padding, but no other extra fields.
}

/// The type for representing socket address families.
pub type sa_family_t = ushort;

/// The type for representing socket communication model types.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum sock_type {
    SOCK_STREAM = 1,
    SOCK_DGRAM = 2,
    SOCK_RAW = 3,
    SOCK_RDM = 4,
    SOCK_SEQPACKET = 5,
    SOCK_DCCP = 6,
    SOCK_PACKET = 10,
}

/// Used for time in seconds.
pub type time_t = long;

/// Used for time in microseconds.
pub type suseconds_t = long;

/// Representation of time as separate seconds and subseconds.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct timeval {
    tv_sec: long,
    tv_uec: suseconds_t,
}

/// Used for [`crate::getdents`].
#[derive(Debug)]
#[repr(C)]
pub struct linux_dirent {
    pub d_ino: ulong,
    pub d_off: ulong,
    pub d_reclen: ushort,
    pub d_name: [char],
}

/// 64-bit offset.
pub type off64_t = longlong;

// 64-bit inode number.
pub type ino64_t = ulonglong;

/// Used for [`crate::getdents64`].
#[derive(Debug)]
#[repr(C)]
pub struct linux_dirent64 {
    pub d_ino: ino64_t,
    pub d_off: off64_t,
    pub d_reclen: ushort,
    pub d_type: uchar,
    pub d_name: [char],
}

// Also include architecture-specific types.
pub use crate::raw::types::*;
