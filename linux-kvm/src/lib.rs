//! This package wraps the lower-level crate [`linux_io`] to provide more
//! convenient access to the linux KVM API, which allows you to create and
//! run kernel-managed virtual machines on architectures that support that.
#![no_std]

pub mod ioctl;
