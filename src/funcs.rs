use core::ffi;

use super::raw;
use super::result::{prepare_arg as arg, prepare_standard_result as mkresult, Result};
use super::types::*;

macro_rules! syscall {
    ($n:expr) => {
        mkresult(raw::syscall0($n))
    };
    ($n:expr, $a0:expr) => {
        mkresult(raw::syscall1($n, arg($a0)))
    };
    ($n:expr, $a0:expr, $a1:expr) => {
        mkresult(raw::syscall2($n, arg($a0), arg($a1)))
    };
    ($n:expr, $a0:expr, $a1:expr, $a2:expr) => {
        mkresult(raw::syscall3($n, arg($a0), arg($a1), arg($a2)))
    };
    ($n:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr) => {
        mkresult(raw::syscall4($n, arg($a0), arg($a1), arg($a2), arg($a3)))
    };
    ($n:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        mkresult(raw::syscall5(
            $n,
            arg($a0),
            arg($a1),
            arg($a2),
            arg($a3),
            arg($a4),
        ))
    };
    ($n:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        mkresult(raw::syscall6(
            $n,
            arg($a0),
            arg($a1),
            arg($a2),
            arg($a3),
            arg($a4),
            arg($a5),
        ))
    };
}

/// Close a file.
#[cfg(have_syscall = "close")]
#[inline(always)]
pub unsafe fn close(fd: int) -> Result<int> {
    syscall!(raw::CLOSE, fd)
}

/// Create a file.
#[cfg(have_syscall = "creat")]
#[inline(always)]
pub unsafe fn creat(pathname: *const char, mode: mode_t) -> Result<int> {
    syscall!(raw::CREAT, pathname, mode)
}

/// Immediately terminate the current thread, without giving Rust or libc
/// any opportunity to run destructors or other cleanup code.
#[cfg(have_syscall = "exit")]
#[inline(always)]
pub unsafe fn exit(status: int) -> ! {
    raw::syscall1(raw::EXIT, arg(status));
    unreachable!()
}

/// Immediately terminate all threads in the current process's thread group,
/// without giving Rust or libc any opportunity to run destructors or other
/// cleanup code.
#[cfg(have_syscall = "exit_group")]
#[inline(always)]
pub unsafe fn exit_group(status: int) -> ! {
    raw::syscall1(raw::EXIT_GROUP, arg(status));
    unreachable!()
}

/// Get the process id (PID) of the current process.
#[cfg(have_syscall = "getpid")]
#[inline(always)]
pub unsafe fn getpid() -> pid_t {
    raw::syscall0(raw::GETPID) as pid_t
}

/// Open a file.
#[cfg(have_syscall = "open")]
#[inline(always)]
pub unsafe fn open(pathname: *const char, flags: int, mode: mode_t) -> Result<int> {
    syscall!(raw::OPEN, pathname, flags, mode)
}

/// Wait for events on one or more file descriptors.
#[cfg(have_syscall = "poll")]
#[inline(always)]
pub unsafe fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: int) -> Result<int> {
    syscall!(raw::POLL, fds, nfds, timeout)
}

/// Read from a file descriptor.
#[cfg(have_syscall = "read")]
#[inline(always)]
pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> Result<ssize_t> {
    syscall!(raw::READ, fd, buf, count)
}

/// Read from a file descriptor into multiple buffers.
#[cfg(have_syscall = "readv")]
#[inline(always)]
pub unsafe fn readv(fd: int, iov: *mut iovec, iovcount: int) -> Result<size_t> {
    syscall!(raw::READV, fd, iov, iovcount)
}

/// Commit all filesystem caches to disk.
#[cfg(have_syscall = "sync")]
#[inline(always)]
pub unsafe fn sync() {
    raw::syscall0(raw::SYNC);
}

/// Commit filesystem caches to disk for the filesystem containing a particular file.
#[cfg(have_syscall = "syncfs")]
#[inline(always)]
pub unsafe fn syncfs(fd: int) -> Result<int> {
    syscall!(raw::SYNCFS, fd)
}

/// Reposition the read/write offset for a file.
#[cfg(have_syscall = "lseek")]
#[inline(always)]
pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> Result<off_t> {
    syscall!(raw::LSEEK, fd, offset, whence)
}

/// Write to a file descriptor.
#[cfg(have_syscall = "write")]
#[inline(always)]
pub unsafe fn write(fd: int, buf: *const ffi::c_void, count: size_t) -> Result<ssize_t> {
    syscall!(raw::WRITE, fd, buf, count)
}

/// Write to a file descriptor from multiple buffers.
#[cfg(have_syscall = "writev")]
#[inline(always)]
pub unsafe fn writev(fd: int, iov: *const iovec, iovcount: int) -> Result<size_t> {
    syscall!(raw::WRITEV, fd, iov, iovcount)
}

/// A special variant of [`lseek`] for 32-bit platforms that need the 64-bit
/// offset split into two arguments.
///
/// This function is not available at all on 64-bit platforms, because
/// `lseek` is sufficient for 64-bit offsets there.
#[cfg(have_syscall = "_llseek")]
#[inline(always)]
pub unsafe fn _llseek(
    fd: int,
    offset_high: ffi::c_ulong,
    offset_low: ffi::c_ulong,
    result: *mut loff_t,
    whence: uint,
) -> Result<int> {
    syscall!(raw::_LLSEEK, fd, offset_high, offset_low, result, whence)
}
