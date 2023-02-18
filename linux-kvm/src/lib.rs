//! This package wraps the lower-level crate [`linux_io`] to provide more
//! convenient access to the linux KVM API, which allows you to create and
//! run kernel-managed virtual machines on architectures that support that.
//!
//! For now this crate is largely just serving as a prototype case for building
//! low-cost safe abstractions on top of [`linux_io`], so it doesn't support
//! the full KVM API. Hopefully over time it'll gain enough to be useful.
#![no_std]

pub mod ioctl;

pub use linux_io::result::Result;
use linux_io::{File, OpenOptions};
use linux_unsafe::int;

/// Represents the kernel's whole KVM subsystem.
///
/// This is the entry point for obtaining all other KVM objects, whether
/// directly or indirectly.
#[repr(transparent)]
pub struct Kvm {
    f: File<ioctl::system::KvmSystem>,
}

impl Kvm {
    /// Opens the KVM device `/dev/kvm` and returns a [`Kvm`] instance wrapping
    /// it.
    ///
    /// Fails with an error on a system where `/dev/kvm` doesn't exist for some
    /// reason, such as if KVM is not enabled in the kernel.
    ///
    /// **Warning:** The safety of this function relies on there being a
    /// reasonable device node at `/dev/kvm`. If the target system has some
    /// other unrelated device node or a non-device entry at that location
    /// then the returned object will allow issuing ioctl requests to that
    /// file that may cause memory corruption depending on how the opened
    /// device reacts to the KVM ioctl numbers.
    ///
    /// This function is not marked as `unsafe` because a system configured in
    /// that way is considered unreasonable, and this crate is optimized for
    /// reasonable Linux configurations that follow the filesystem layout given
    /// in the kernel documentation.
    pub fn open() -> Result<Self> {
        let path = unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(b"/dev/kvm\0") };
        let opts = OpenOptions::read_write().close_on_exec();
        let f = File::open(path, opts)?;

        // Safety: On any reasonable Linux system /dev/kvm should either not
        // exist (and we would've returned an error by now) or refer to the
        // main KVM system device, and should therefore be suitable to
        // accept the KvmSystem ioctls.
        let f = unsafe { f.to_device(ioctl::system::KvmSystem) };
        Ok(Self::from_file(f))
    }

    /// Wraps the given already-opened file in a `Kvm` object.
    #[inline(always)]
    pub const fn from_file(f: File<ioctl::system::KvmSystem>) -> Self {
        Self { f }
    }

    /// Identifies the version of the KVM API used by the current kernel.
    ///
    /// The stable API always returns version 12. The kernel documentation suggests
    /// that applications should always call this and refuse to run if it returns
    /// any value other than that; the version number is not expected to change
    /// in the future because future API additions will use [`Self::check_extension`]
    /// instead.
    #[inline(always)]
    pub fn get_api_version(&self) -> Result<int> {
        self.f.ioctl(ioctl::system::KVM_GET_API_VERSION, ())
    }

    /// Query whether the KVM subsystem in the current kernel supports a particular
    /// extension.
    ///
    /// A result of zero indicates a lack of support while nonzero indicates
    /// support. The nonzero value may carry additional meanings for some
    /// extensions.
    #[inline(always)]
    pub fn check_extension(&self, ext: int) -> Result<int> {
        self.f.ioctl(ioctl::system::KVM_CHECK_EXTENSION, &ext)
    }

    /// Create a new virtual machine.
    #[inline(always)]
    pub fn create_vm(&self) -> Result<VirtualMachine> {
        let f = self.f.ioctl(ioctl::system::KVM_CREATE_VM, ())?;
        Ok(VirtualMachine::from_file(f))
    }
}

/// An individual virtual machine created through a [`Kvm`] object.
#[repr(transparent)]
pub struct VirtualMachine {
    f: File<ioctl::vm::KvmVm>,
}

impl VirtualMachine {
    /// Wraps the given already-opened file in a `Kvm` object.
    #[inline(always)]
    pub const fn from_file(f: File<ioctl::vm::KvmVm>) -> Self {
        Self { f }
    }

    /// Query whether the KVM subsystem in the current kernel supports a particular
    /// extension for a specific VM.
    ///
    /// A result of zero indicates a lack of support while nonzero indicates
    /// support. The nonzero value may carry additional meanings for some
    /// extensions.
    #[inline(always)]
    pub fn check_extension(&self, ext: int) -> Result<int> {
        self.f.ioctl(ioctl::vm::KVM_CHECK_EXTENSION, &ext)
    }
}
