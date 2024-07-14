//! Thin wrappers around the CPU instructions for making system calls on x86_64.

use core::arch::asm;

/// The type of all system call arguments and return values on this platform.
pub type V = u64;

/// Call into a system function with no arguments.
#[inline(always)]
pub unsafe fn syscall0(n: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with one argument.
#[inline(always)]
pub unsafe fn syscall1(n: V, a0: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with two arguments.
#[inline(always)]
pub unsafe fn syscall2(n: V, a0: V, a1: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        in("rsi") a1,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with three arguments.
#[inline(always)]
pub unsafe fn syscall3(n: V, a0: V, a1: V, a2: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with four arguments.
#[inline(always)]
pub unsafe fn syscall4(n: V, a0: V, a1: V, a2: V, a3: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("r10") a3,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with five arguments.
#[inline(always)]
pub unsafe fn syscall5(n: V, a0: V, a1: V, a2: V, a3: V, a4: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("r10") a3,
        in("r8") a4,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Call into a system function with six arguments.
#[inline(always)]
pub unsafe fn syscall6(n: V, a0: V, a1: V, a2: V, a3: V, a4: V, a5: V) -> V {
    let ret: V;
    asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("r10") a3,
        in("r8") a4,
        in("r9") a5,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Given a result value from a system call that follows the standard error
/// return convention for this platform, returns either the given value
/// verbatim or the kernel error code extracted from it.
///
/// For x86_64, the standard way to signal an error is to return a result
/// between -4095 and -1 inclusive, with all other values representing
/// successful results.
///
/// A small number of system calls signal errors in different ways. This
/// function is not compatible with the results from those calls.
#[inline]
pub fn unpack_standard_result(raw: V) -> Result<V, i32> {
    if (raw as u64) >= ((-4095 as i64) as u64) {
        let err = -(raw as i32);
        Err(err)
    } else {
        Ok(raw)
    }
}

include!(concat!(env!("OUT_DIR"), "/syscall_nrs_x86_64.rs"));
pub(crate) mod errno {
    include!(concat!(env!("OUT_DIR"), "/errnos_x86_64.rs"));
}

// Architecture-specific types and constants
pub(crate) mod types {}
