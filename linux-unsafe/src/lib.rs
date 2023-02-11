//! A low-level, unsafe Rust interface to Linux system calls.
//!
//! The [`raw`] module provides functions wrapping platform-specific assembly
//! language stubs for making arbitrary system calls by providing a system
//! call number and arbitrary number of arguments.
//!
//! This crate currently supports the following architectures:
//!
//! - x86_64
//! - x86 (32-bit)
//! - arm
//! - riscv64
//!
//! For this initial release, x86_64 has seen some limited testing and the
//! other platforms have been barely tested at all. Over time I intend to
//! support all architectures that Linux supports that are also supported
//! by Rust inline assembly, but we'll see how it goes.
//!
//! The functions in the root of the crate then wrap those stubs with thin
//! wrappers that just lightly convert their arguments to what the kernel
//! expects for a particular system call and then delegate to one of the system
//! call stubs in [`raw`].
//!
//! This crate also includes a number of types and type aliases representing
//! the memory layout of objects the kernel will interpret. For those which
//! are aliases, calling code must always use the aliases rather than their
//! underlying types because their exact definitions may vary on different
//! platforms and in future versions of this crate.
//!
//! Where possible the wrapping functions and types are portable across
//! architectures, as long as callers use the argument types and type aliases
//! defined in this crate. The raw system call interface has considerable
//! overlap between platforms but is ultimately architecture-specific and this
//! crate does not attempt to hide differences at that layer.
//!
//! # Be careful mixing with `std`
//!
//! The Rust `std` crate has lots of functionality that wraps the target's
//! libc functions. On Linux systems libc is a wrapper around the same system
//! call interface this crate is exposing, but also adds other state and
//! abstractions such as buffers and error codes. Making direct system calls
//! may violate the assumptions being made by libc.
//!
//! To avoid strange problems, avoid interacting with the same system resources
//! through both the standard library and though direct system calls.
#![no_std]

mod funcs;
mod types;

pub use funcs::*;
pub use types::*;
pub mod result;

pub mod args;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[path = "raw/x86_64.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "x86"))]
#[path = "raw/x86.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "arm"))]
#[path = "raw/arm.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "riscv64"))]
#[path = "raw/riscv64.rs"]
pub mod raw;

#[cfg(test)]
mod tests;
