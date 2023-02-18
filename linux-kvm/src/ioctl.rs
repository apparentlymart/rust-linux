//! ioctl constants for use with [`linux_io::File::ioctl`].
//!
//! This exposes the lower-level `ioctl` constants that the higher-level API
//! is implemented in terms of, in case you need to do something the wrapping
//! API cannot yet do, or in case you just prefer working this way.
//!
//! The constants are categorized the same way as they are in
//! [the kernel API docs](https://docs.kernel.org/virt/kvm/api.html),
//! split into:
//! - [`system`] for system ioctls
//! - [`vm`] for VM ioctls
//! - [`vcpu`] for VCPU ioctls
//! - [`device`] for device ioctls

/// Constants for system ioctls.
pub mod system;

/// Constants for VM ioctls.
pub mod vm;

/// Constants for VCPU ioctls.
pub mod vcpu;

/// Constants for device ioctls.
pub mod device;
