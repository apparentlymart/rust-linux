#![allow(non_camel_case_types)]

use core::ffi;

/// The primary unsigned integer type for the current platform.
pub type int = ffi::c_int;

/// The signed size type (or "pointer difference" type) for the current platform.
pub type ssize_t = isize;

/// The unsigned size type for the current platform.
pub type size_t = usize;

/// The type used for void pointers on the current platform.
pub type void = ffi::c_void;

/// The type used for process identifiers (PIDs) on the current platform.
pub type pid_t = int;

/// The type used to represent file sizes and offsets into files on the current platform.
pub type off_t = ffi::c_long;

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
