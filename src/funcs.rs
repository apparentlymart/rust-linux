use core::ffi;

use super::raw;
use super::types::*;

/// Get the process id (PID) of the current process.
#[inline(always)]
pub unsafe fn getpid() -> pid_t {
    raw::syscall0(raw::GETPID) as pid_t
}

/// Read from a file descriptor.
#[inline(always)]
pub unsafe fn read(fd: int, buf: *mut ffi::c_void, count: size_t) -> ssize_t {
    raw::syscall3(raw::READ, fd as raw::V, buf as raw::V, count as raw::V) as ssize_t
}

/// Reposition the read/write offset for a file.
#[inline(always)]
pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    raw::syscall3(raw::LSEEK, fd as raw::V, offset as raw::V, whence as raw::V) as off_t
}

/// Write to a file descriptor.
#[inline(always)]
pub unsafe fn write(fd: int, buf: *const ffi::c_void, count: size_t) -> ssize_t {
    raw::syscall3(raw::WRITE, fd as raw::V, buf as raw::V, count as raw::V) as ssize_t
}
