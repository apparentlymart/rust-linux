//! Thin wrappers around the CPU instructions for making system calls on ARM.

use core::arch::asm;

/// The type of all system call arguments and return values on this platform.
pub type V = u32;

/// Call into a system function with one argument.
#[inline(always)]
pub unsafe fn syscall0(n: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        out("r0") ret,
    );
    ret
}

/// Call into a system function with one argument.
#[inline(always)]
pub unsafe fn syscall1(n: V, a0: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
    );
    ret
}

/// Call into a system function with two arguments.
#[inline(always)]
pub unsafe fn syscall2(n: V, a0: V, a1: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
    );
    ret
}

/// Call into a system function with three arguments.
#[inline(always)]
pub unsafe fn syscall3(n: V, a0: V, a1: V, a2: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
        in("r2") a2,
    );
    ret
}

/// Call into a system function with four arguments.
#[inline(always)]
pub unsafe fn syscall4(n: V, a0: V, a1: V, a2: V, a3: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
        in("r2") a2,
        in("r3") a3,
    );
    ret
}

/// Call into a system function with five arguments.
#[inline(always)]
pub unsafe fn syscall5(n: V, a0: V, a1: V, a2: V, a3: V, a4: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
        in("r2") a2,
        in("r3") a3,
        in("r4") a4,
    );
    ret
}

/// Call into a system function with six arguments.
#[inline(always)]
pub unsafe fn syscall6(n: V, a0: V, a1: V, a2: V, a3: V, a4: V, a5: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
        in("r2") a2,
        in("r3") a3,
        in("r4") a4,
        in("r5") a5,
    );
    ret
}

/// Call into a system function with seven arguments.
#[inline(always)]
pub unsafe fn syscall7(n: V, a0: V, a1: V, a2: V, a3: V, a4: V, a5: V, a6: V) -> V {
    let ret: V;
    asm!(
        "svc 0",
        in("r7") n,
        inout("r0") a0 => ret,
        in("r1") a1,
        in("r2") a2,
        in("r3") a3,
        in("r4") a4,
        in("r5") a5,
        in("r6") a6,
    );
    ret
}

include!(concat!(env!("OUT_DIR"), "/syscall_nrs_arm.rs"));
