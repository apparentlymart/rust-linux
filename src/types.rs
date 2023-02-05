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
