use core::ffi;

use super::raw;
use super::types::*;

/// Close a file.
#[cfg(have_syscall = "close")]
#[inline(always)]
pub unsafe fn close(fd: int) -> int {
    raw::syscall1(raw::CLOSE, fd as raw::V) as int
}

/// Create a file.
#[cfg(have_syscall = "creat")]
#[inline(always)]
pub unsafe fn creat(pathname: *const char, mode: mode_t) -> int {
    raw::syscall2(raw::CREAT, pathname as raw::V, mode as raw::V) as int
}

/// Immediately terminate the current thread, without giving Rust or libc
/// any opportunity to run destructors or other cleanup code.
#[cfg(have_syscall = "exit")]
#[inline(always)]
pub unsafe fn exit(status: int) -> ! {
    raw::syscall1(raw::EXIT, status as raw::V);
    unreachable!()
}

/// Immediately terminate all threads in the current process's thread group,
/// without giving Rust or libc any opportunity to run destructors or other
/// cleanup code.
#[cfg(have_syscall = "exit_group")]
#[inline(always)]
pub unsafe fn exit_group(status: int) -> ! {
    raw::syscall1(raw::EXIT_GROUP, status as raw::V);
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
pub unsafe fn open(pathname: *const char, flags: int, mode: mode_t) -> int {
    raw::syscall3(
        raw::OPEN,
        pathname as raw::V,
        flags as raw::V,
        mode as raw::V,
    ) as int
}

/// Wait for events on one or more file descriptors.
#[cfg(have_syscall = "poll")]
#[inline(always)]
pub unsafe fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: int) -> int {
    raw::syscall3(raw::POLL, fds as raw::V, nfds as raw::V, timeout as raw::V) as int
}

/// Read from a file descriptor.
#[cfg(have_syscall = "read")]
#[inline(always)]
pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    raw::syscall3(raw::READ, fd as raw::V, buf as raw::V, count as raw::V) as ssize_t
}

/// Read from a file descriptor into multiple buffers.
#[cfg(have_syscall = "readv")]
#[inline(always)]
pub unsafe fn readv(fd: int, iov: *mut iovec, iovcount: int) -> size_t {
    raw::syscall3(raw::READV, fd as raw::V, iov as raw::V, iovcount as raw::V) as size_t
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
pub unsafe fn syncfs(fd: int) -> int {
    raw::syscall1(raw::SYNCFS, fd as raw::V) as int
}

/// Reposition the read/write offset for a file.
#[cfg(have_syscall = "lseek")]
#[inline(always)]
pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    raw::syscall3(raw::LSEEK, fd as raw::V, offset as raw::V, whence as raw::V) as off_t
}

/// Write to a file descriptor.
#[cfg(have_syscall = "write")]
#[inline(always)]
pub unsafe fn write(fd: int, buf: *const ffi::c_void, count: size_t) -> ssize_t {
    raw::syscall3(raw::WRITE, fd as raw::V, buf as raw::V, count as raw::V) as ssize_t
}

/// Write to a file descriptor from multiple buffers.
#[cfg(have_syscall = "writev")]
#[inline(always)]
pub unsafe fn writev(fd: int, iov: *const iovec, iovcount: int) -> size_t {
    raw::syscall3(raw::WRITEV, fd as raw::V, iov as raw::V, iovcount as raw::V) as size_t
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
) -> int {
    raw::syscall5(
        raw::_LLSEEK,
        fd as raw::V,
        offset_high as raw::V,
        offset_low as raw::V,
        result as raw::V,
        whence as raw::V,
    ) as int
}
