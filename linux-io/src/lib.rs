//! Lightweight but safe abstractions around Linux system calls related to
//! file descriptors.
//!
//! This goal of this crate is to expose a convenient API while skipping any
//! unnecessary abstraction. In most cases calls to functions in this crate
//! should reduce to inline system calls and some minimal argument and result
//! conversion code, and the results should be generally unsurprising to anyone
//! who is familiar with the underlying system call behavior.
//!
//! The functions in this crate wrap functions in crate [`linux_unsafe`] to
//! actually make the system calls, and so the platform support for this
//! crate is limited to what that other crate supports.
//!
//! Implements standard library I/O traits by default, but can be made friendly
//! to `no_std` environments by disabling the default feature `std`.
//!
//! The initial versions of this crate are focused only on basic file
//! operations, until the API for that feels settled. In later releases the
//! scope will hopefully increase to cover most or all of the system calls
//! that work with file descriptors.
#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

/// Access to the "poll" system call.
pub mod poll;

/// Types for representing system call results and errors.
pub mod result;

/// Types for use with "seek" operations.
pub mod seek;

/// The main `File` type and its supporting utilities for working safely with file descriptors.
pub mod fd;
pub use fd::{File, OpenOptions, OPEN_READ_ONLY, OPEN_READ_WRITE, OPEN_WRITE_ONLY};

/// For interacting with tty devices.
pub mod tty;

/// Socket address manipulation, socket device ioctls, etc.
pub mod socket;

/// Synchronization primitives built using Linux kernel features.
pub mod sync;

/// For safely representing pointers in `ioctl` request types, and similar.
pub mod ptr;

#[cfg(test)]
mod tests;
