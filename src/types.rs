#![allow(non_camel_case_types)]

use core::ffi;

/// The primary signed integer type for the current platform.
pub type int = ffi::c_int;

/// The primary unsigned integer type for the current platform.
pub type uint = ffi::c_uint;

/// The signed long integer type for the current platform.
pub type long = ffi::c_long;

/// The unsigned long integer type for the current platform.
pub type ulong = ffi::c_ulong;

/// The signed size type (or "pointer difference" type) for the current platform.
pub type ssize_t = isize;

/// The unsigned size type for the current platform.
pub type size_t = usize;

/// The type used for characters on the current platform.
pub type char = ffi::c_char;

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

// Also include architecture-specific types.
pub use crate::raw::types::*;
