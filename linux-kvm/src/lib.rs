//! This package wraps the lower-level crate [`linux_io`] to provide more
//! convenient access to the linux KVM API, which allows you to create and
//! run kernel-managed virtual machines on architectures that support that.
//!
//! For now this crate is largely just serving as a prototype case for building
//! low-cost safe abstractions on top of [`linux_io`], so it doesn't support
//! the full KVM API. Hopefully over time it'll gain enough to be useful.
#![no_std]

pub mod ioctl;
pub mod raw;

pub use linux_io::result::Result;
use linux_io::{File, OpenOptions};
use linux_unsafe::int;

/// Represents the kernel's whole KVM subsystem.
///
/// This is the entry point for obtaining all other KVM objects, whether
/// directly or indirectly.
#[derive(Debug)]
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
        Ok(VirtualMachine::from_file(f, &self))
    }

    /// Determine the size of the shared memory regions that will be used
    /// between kernel and userspace for each VCPU.
    #[inline(always)]
    pub fn get_vcpu_mmap_size(&self) -> Result<int> {
        self.f.ioctl(ioctl::system::KVM_GET_VCPU_MMAP_SIZE, ())
    }
}

/// An individual virtual machine created through a [`Kvm`] object.
#[derive(Debug)]
pub struct VirtualMachine<'a> {
    f: File<ioctl::vm::KvmVm>,
    kvm: &'a Kvm,
}

impl<'a> VirtualMachine<'a> {
    /// Wraps the given already-opened file in a `VirtualMachine` object.
    #[inline(always)]
    const fn from_file(f: File<ioctl::vm::KvmVm>, kvm: &'a Kvm) -> Self {
        Self { f, kvm }
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

    /// Create a new VCPU for this VM.
    ///
    /// If creating multiple VCPUs in the same VM, start with `cpu_id` zero
    /// and then increment for each new VM. The kernel enforces a
    /// platform-specific limit on VCPUs per VM, which you can determine by
    /// querying extensions using [`Self::check_extension`].
    #[inline(always)]
    pub fn create_vcpu(&self, cpu_id: linux_unsafe::int) -> Result<VirtualCpu> {
        self.f
            .ioctl(ioctl::vm::KVM_CREATE_VCPU, cpu_id)
            .map(|f| VirtualCpu::from_file(f, &self.kvm))
    }

    /// Sets one of the VM's memory region slots to refer to the given
    /// memory region, which must outlive this VCPU.
    pub fn set_guest_memory_region<'r: 'a>(
        &mut self,
        slot: u32,
        flags: u32,
        guest_phys_addr: u64,
        host_region: &'r mut MemoryRegion,
    ) -> Result<()> {
        let desc = raw::kvm_userspace_memory_region {
            slot,
            flags,
            guest_phys_addr,
            memory_size: host_region.length as u64,
            userspace_addr: host_region.addr as u64,
        };
        self.f
            .ioctl(ioctl::vm::KVM_SET_USER_MEMORY_REGION, &desc)
            .map(|_| ())
    }
}

/// A virtual CPU belonging to a [`VirtualMachine`].
#[derive(Debug)]
pub struct VirtualCpu<'a> {
    f: File<ioctl::vcpu::KvmVcpu>,
    kvm: &'a Kvm,
}

impl<'a> VirtualCpu<'a> {
    /// Wraps the given already-opened file in a `VirtualCpu` object.
    #[inline(always)]
    const fn from_file(f: File<ioctl::vcpu::KvmVcpu>, kvm: &'a Kvm) -> Self {
        Self { f, kvm }
    }

    /// Get the architecture-specific representation of the current register
    /// values of this vCPU.
    #[inline(always)]
    pub fn get_regs(&self) -> Result<raw::kvm_regs> {
        self.f.ioctl(ioctl::vcpu::KVM_GET_REGS, ())
    }

    /// Set the architecture-specific representation of the current register
    /// values of this vCPU.
    #[inline(always)]
    pub fn set_regs(&self, new: &raw::kvm_regs) -> Result<()> {
        self.f.ioctl(ioctl::vcpu::KVM_SET_REGS, new).map(|_| ())
    }

    /// Wrap this CPU into an object that has the necessary extra state to
    /// run it.
    ///
    /// This encapsulates the step of using `mmap` on the VCPU file descriptor
    /// to establish a shared memory space with the KVM subsystem, so failure
    /// here represents failure of either that `mmap` operation or the
    /// `ioctl` call to discover its parameters.
    pub fn to_runner(self) -> Result<VirtualCpuRunner<'a>> {
        let mmap_size = self.kvm.get_vcpu_mmap_size()?;
        VirtualCpuRunner::new(self, mmap_size as linux_unsafe::size_t)
    }
}

/// Wraps a [`VirtualCpu`] with some extra state required to run it.
#[derive(Debug)]
pub struct VirtualCpuRunner<'a> {
    vcpu: VirtualCpu<'a>,
    run: *mut raw::kvm_run,
    run_len: linux_unsafe::size_t,
}

impl<'a> VirtualCpuRunner<'a> {
    fn new(cpu: VirtualCpu<'a>, mmap_size: linux_unsafe::size_t) -> Result<Self> {
        if core::mem::size_of::<raw::kvm_run>() > (mmap_size as usize) {
            // We can't safely use our struct type over the mmap region if
            // the region isn't long enough. This shouldn't happen because
            // we're using the documented structure.
            return Err(linux_io::result::Error::new(12 /* ENOMEM */));
        }

        let run_ptr = unsafe {
            cpu.f.mmap_raw(
                0,
                mmap_size,
                core::ptr::null_mut(),
                0x1 | 0x2, // PROT_READ | PROT_WRITE
                0x1,       // MAP_SHARED
            )
        }? as *mut raw::kvm_run;

        // Safety: We assume that the kernel has placed valid initial values for
        // all of the fields of kvm_run in this shared memory area before
        // returning it, so we don't need to initialize it further here.

        Ok(Self {
            vcpu: cpu,
            run: run_ptr,
            run_len: mmap_size,
        })
    }

    /// Get the architecture-specific representation of the current register
    /// values of this vCPU.
    #[inline(always)]
    pub fn get_regs(&self) -> Result<raw::kvm_regs> {
        self.vcpu.get_regs()
    }

    /// Set the architecture-specific representation of the current register
    /// values of this vCPU.
    #[inline(always)]
    pub fn set_regs(&self, new: &raw::kvm_regs) -> Result<()> {
        self.vcpu.set_regs(new)
    }

    /// Modify in place the architecturte-specific register values of this vCPU.
    #[inline]
    pub fn modify_regs<R>(&self, f: impl FnOnce(&mut raw::kvm_regs) -> R) -> Result<R> {
        let mut regs = self.get_regs()?;
        let ret = f(&mut regs);
        self.set_regs(&regs)?;
        Ok(ret)
    }

    #[inline]
    pub fn with_raw_run_state<R>(&mut self, f: impl FnOnce(&mut raw::kvm_run) -> R) -> R {
        f(unsafe { &mut *self.run })
    }

    /// Run the VCPU until it exits.
    #[inline(always)]
    pub fn run_raw(&mut self) -> Result<()> {
        self.vcpu.f.ioctl(ioctl::vcpu::KVM_RUN, ())?;
        Ok(())
    }
}

impl<'a> Drop for VirtualCpuRunner<'a> {
    /// [`VirtualCpuRunner`] automatically releases its kernel shared memory
    /// mapping when dropped, and will panic if that fails.
    fn drop(&mut self) {
        unsafe { linux_unsafe::munmap(self.run as *mut linux_unsafe::void, self.run_len) }.unwrap();
    }
}

/// A page-aligned host memory region that can be mapped into the guest memory
/// space of a [`VirtualMachine`].
#[derive(Debug)]
pub struct MemoryRegion {
    addr: *mut linux_unsafe::void,
    length: linux_unsafe::size_t,
}

impl MemoryRegion {
    /// Attempts to allocate a new memory region of a given size.
    #[inline]
    pub fn new(length: linux_unsafe::size_t) -> Result<Self> {
        let addr = unsafe {
            linux_unsafe::mmap(
                core::ptr::null_mut(),
                length,
                0x1 | 0x2,  // PROT_READ | PROT_WRITE
                0x1 | 0x20, // MAP_SHARED | MAP_ANONYMOUS
                -1,         // no fd, because MAP_ANONYMOUS
                0,
            )
        }?;
        Ok(Self { addr, length })
    }

    /// Returns a view of the memory region as a mutable slice, which
    /// the caller can then modify to populate the memory area.
    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        // Safety: Caller can't interact with the memory region in any other
        // way while still holding the mutable borrow we return here, so
        // nothing else should access it.
        unsafe { core::slice::from_raw_parts_mut(self.addr as *mut u8, self.length) }
    }
}

impl<'a> Drop for MemoryRegion {
    /// [`MemoryRegion`] automatically releases its memory mapping when dropped,
    /// and will panic if that fails.
    fn drop(&mut self) {
        unsafe { linux_unsafe::munmap(self.addr, self.length) }.unwrap();
    }
}
