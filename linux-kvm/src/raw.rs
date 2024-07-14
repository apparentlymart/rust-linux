//! Raw data types for use with various KVM `ioctl` requests.

/// The layout of the shared memory region used to communicate with the
/// `KVM_RUN` ioctl request, which is `mmap`ed from the VCPU's file descriptor.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct kvm_run {
    pub request_interrupt_window: u8,
    pub immediate_exit: u8,
    pub padding1: [u8; 6],
    pub exit_reason: u32,
    pub ready_for_interrupt_injection: u8,
    pub if_flag: u8,
    pub flags: u16,
    pub cr8: u64,
    pub apic_base: u64,
    pub exit_details: ExitDetails,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct kvm_userspace_memory_region {
    pub slot: u32,
    pub flags: u32,
    pub guest_phys_addr: u64,
    pub memory_size: u64,    // in bytes
    pub userspace_addr: u64, // start of the userspace allocated memory
}

#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct kvm_regs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct kvm_regs {
    pub regs: aarch64_user_pt_regs,
    pub sp_el1: u64,
    pub elr_el1: u64,
    pub spsr: [u64; 5],
    pub fp_regs: aarch64_user_fpsimd_state,
}

#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct aarch64_user_pt_regs {
    pub regs: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
}

#[cfg(target_arch = "aarch64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct aarch64_user_fpsimd_state {
    pub vregs: [u128; 32],
    pub fpsr: u32,
    pub fpcr: u32,
    pub __reserved: [u32; 2],
}

#[cfg(target_arch = "riscv64")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct kvm_regs {
    // The RISC-V port does not support getting/setting
    // all registers together. It only supports individual
    // register accesses using the GET_REG/SET_REG operations.
}

/// Used for the `exit_details` field of [`kvm_run`].
#[derive(Clone, Copy)]
#[repr(C)]
pub union ExitDetails {
    pub hw: ExitUnknown,
    pub fail_entry: ExitFailEntry,
    pub ex: ExitException,
    pub io: ExitIo,
    pub mmio: ExitMmio,
    // TODO: The rest of these
    pub padding: [linux_unsafe::char; 256],
}

impl core::fmt::Debug for ExitDetails {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExitDetails").finish()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExitUnknown {
    pub hardware_exit_reason: u64,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExitFailEntry {
    pub hardware_entry_failure_reason: u64,
    pub cpu: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExitException {
    pub exception: u32,
    pub error_code: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExitIo {
    pub direction: u8,
    pub size: u8,
    pub port: u16,
    pub count: u32,
    pub data_offset: u64,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ExitMmio {
    pub phys_addr: u64,
    pub data: [u8; 8],
    pub len: u32,
    pub is_write: u8,
}
