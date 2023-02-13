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

/// Accept a connection on a socket.
#[cfg(have_syscall = "accept")]
#[inline(always)]
pub unsafe fn accept(sockfd: int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> Result<int> {
    syscall!(raw::ACCEPT, sockfd, addr, addrlen)
}

/// Accept a connection on a socket with additional flags.
#[cfg(have_syscall = "accept4")]
#[inline(always)]
pub unsafe fn accept4(
    sockfd: int,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: int,
) -> Result<int> {
    syscall!(raw::ACCEPT4, sockfd, addr, addrlen, flags)
}

/// Check user's permissions for a file.
#[cfg(have_syscall = "access")]
#[inline(always)]
pub unsafe fn access(pathname: *const char, mode: int) -> Result<int> {
    syscall!(raw::ACCESS, pathname, mode)
}

/// Switch process accounting on or off.
#[cfg(have_syscall = "acct")]
#[inline(always)]
pub unsafe fn acct(filename: *const char) -> Result<int> {
    syscall!(raw::ACCT, filename)
}

/// Set an alarm clock for delivery of a signal.
#[cfg(have_syscall = "alarm")]
#[inline(always)]
pub unsafe fn alarm(seconds: uint) -> uint {
    use crate::args::AsRawV;
    uint::from_raw_result(raw::syscall1(raw::ALARM, arg(seconds)))
}

/// Bind a name to a socket.
#[cfg(have_syscall = "bind")]
#[inline(always)]
pub unsafe fn bind(sockfd: int, addr: *const sockaddr, addrlen: socklen_t) -> Result<int> {
    syscall!(raw::BIND, sockfd, addr, addrlen)
}

/// Set the program break.
#[cfg(have_syscall = "brk")]
#[inline(always)]
pub unsafe fn brk(brk: ulong) -> long {
    use crate::args::AsRawV;
    long::from_raw_result(raw::syscall1(raw::BRK, arg(brk)))
}

/// Change working directory.
#[cfg(have_syscall = "chdir")]
#[inline(always)]
pub unsafe fn chdir(path: *const char) -> Result<int> {
    syscall!(raw::CHDIR, path)
}

/// Change permissions of a file.
#[cfg(have_syscall = "chmod")]
#[inline(always)]
pub unsafe fn chmod(pathname: *const char, mode: mode_t) -> Result<int> {
    syscall!(raw::CHMOD, pathname, mode)
}

/// Change ownership of a file.
#[cfg(have_syscall = "chown")]
#[inline(always)]
pub unsafe fn chown(pathname: *const char, owner: uid_t, group: gid_t) -> Result<int> {
    syscall!(raw::CHOWN, pathname, owner, group)
}

/// Change the root directory.
#[cfg(have_syscall = "chroot")]
#[inline(always)]
pub unsafe fn chroot(path: *const char) -> Result<int> {
    syscall!(raw::CHROOT, path)
}

/// Close a file.
#[cfg(have_syscall = "close")]
#[inline(always)]
pub unsafe fn close(fd: int) -> Result<int> {
    syscall!(raw::CLOSE, fd)
}

/// Close all file descriptors in a given range.
#[cfg(have_syscall = "close_range")]
#[inline(always)]
pub unsafe fn close_range(first: int, last: int, flags: uint) -> Result<int> {
    syscall!(raw::CLOSE_RANGE, first, last, flags)
}

/// Initiate a connection on a socket.
#[cfg(have_syscall = "connect")]
#[inline(always)]
pub unsafe fn connect(sockfd: int, addr: *const sockaddr, addrlen: socklen_t) -> Result<int> {
    syscall!(raw::CONNECT, sockfd, addr, addrlen)
}

/// Create a file.
#[cfg(have_syscall = "creat")]
#[inline(always)]
pub unsafe fn creat(pathname: *const char, mode: mode_t) -> Result<int> {
    syscall!(raw::CREAT, pathname, mode)
}

/// Duplicate a file descriptor.
#[cfg(have_syscall = "dup")]
#[inline(always)]
pub unsafe fn dup(oldfd: int) -> Result<int> {
    syscall!(raw::DUP, oldfd)
}

/// Duplicate a file descriptor.
#[cfg(have_syscall = "dup2")]
#[inline(always)]
pub unsafe fn dup2(oldfd: int, newfd: int) -> Result<int> {
    syscall!(raw::DUP2, oldfd, newfd)
}

/// Duplicate a file descriptor.
#[cfg(have_syscall = "dup3")]
#[inline(always)]
pub unsafe fn dup3(oldfd: int, newfd: int, flags: int) -> Result<int> {
    syscall!(raw::DUP3, oldfd, newfd, flags)
}

/// Open an epoll file descriptor.
#[cfg(have_syscall = "epoll_create")]
#[inline(always)]
pub unsafe fn epoll_create(size: int) -> Result<int> {
    syscall!(raw::EPOLL_CREATE, size)
}

/// Open an epoll file descriptor.
#[cfg(have_syscall = "epoll_create1")]
#[inline(always)]
pub unsafe fn epoll_create1(flags: int) -> Result<int> {
    syscall!(raw::EPOLL_CREATE1, flags)
}

/// Control interface for an epoll file descriptor.
#[cfg(have_syscall = "epoll_ctl")]
#[inline(always)]
pub unsafe fn epoll_ctl(epfd: int, op: int, fd: int, event: *const epoll_event) -> Result<int> {
    syscall!(raw::EPOLL_CTL, epfd, op, fd, event)
}

/// Wait for an I/O event on an epoll file descriptor.
#[cfg(have_syscall = "epoll_wait")]
#[inline(always)]
pub unsafe fn epoll_wait(
    epfd: int,
    events: *const epoll_event,
    maxevents: int,
    timeout: int,
) -> Result<int> {
    syscall!(raw::EPOLL_WAIT, epfd, events, maxevents, timeout)
}

/// Create a file descriptor for event notification.
#[cfg(have_syscall = "eventfd")]
#[inline(always)]
pub unsafe fn eventfd(initval: uint) -> Result<int> {
    syscall!(raw::EVENTFD, initval)
}

/// Create a file descriptor for event notification.
#[cfg(have_syscall = "eventfd2")]
#[inline(always)]
pub unsafe fn eventfd2(initval: uint, flags: int) -> Result<int> {
    syscall!(raw::EVENTFD2, initval, flags)
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

/// Check user's permissions for a file.
#[cfg(have_syscall = "faccessat")]
#[inline(always)]
pub unsafe fn faccessat(dirfd: int, pathname: *const char, mode: int) -> Result<int> {
    syscall!(raw::FACCESSAT, dirfd, pathname, mode)
}

/// Check user's permissions for a file.
#[cfg(have_syscall = "faccessat2")]
#[inline(always)]
pub unsafe fn faccessat2(dirfd: int, pathname: *const char, mode: int, flags: int) -> Result<int> {
    syscall!(raw::FACCESSAT2, dirfd, pathname, mode, flags)
}

/// Change working directory.
#[cfg(have_syscall = "fchdir")]
#[inline(always)]
pub unsafe fn fchdir(fd: int) -> Result<int> {
    syscall!(raw::FCHDIR, fd)
}

/// Change permissions of a file.
#[cfg(have_syscall = "fchmod")]
#[inline(always)]
pub unsafe fn fchmod(fd: int, mode: mode_t) -> Result<int> {
    syscall!(raw::FCHMOD, fd, mode)
}

/// Change permissions of a file.
#[cfg(have_syscall = "fchmodat")]
#[inline(always)]
pub unsafe fn fchmodat(dirfd: int, pathname: *const char, mode: mode_t) -> Result<int> {
    syscall!(raw::FCHMODAT, dirfd, pathname, mode)
}

/// Change ownership of a file.
#[cfg(have_syscall = "fchown")]
#[inline(always)]
pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> Result<int> {
    syscall!(raw::FCHOWN, fd, owner, group)
}

/// Change ownership of a file.
#[cfg(have_syscall = "fchownat")]
#[inline(always)]
pub unsafe fn fchownat(
    dirfd: int,
    pathname: *const char,
    owner: uid_t,
    group: gid_t,
) -> Result<int> {
    syscall!(raw::FCHOWN, dirfd, pathname, owner, group)
}

/// Manipulate characteristics of a file descriptor.
///
/// This system call is _particularly_ unsafe, because the final argument
/// gets interpreted in very different ways depending on the value of
/// the `cmd` argument. Callers must take care to ensure that `arg` is of
/// an appropriate type and value for the selected `cmd`.
///
/// Set `arg` to `()` (empty tuple) for commands whose argument is specified
/// as "void" in the documentation.
#[cfg(have_syscall = "fcntl")]
#[inline(always)]
pub unsafe fn fcntl(fd: int, cmd: int, arg: impl crate::args::AsRawV) -> Result<int> {
    if arg.raw_is_void() {
        syscall!(raw::FCNTL, fd, cmd)
    } else {
        syscall!(raw::FCNTL, fd, cmd, arg)
    }
}

/// Synchronize a file's in-core state with storage device.
#[cfg(have_syscall = "fdatasync")]
#[inline(always)]
pub unsafe fn fdatasync(fd: int) -> Result<int> {
    syscall!(raw::FDATASYNC, fd)
}

/// Synchronize a file's in-core state with storage device.
#[cfg(have_syscall = "fsync")]
#[inline(always)]
pub unsafe fn fsync(fd: int) -> Result<int> {
    syscall!(raw::FSYNC, fd)
}

/// Truncate a file to a specified length.
#[cfg(have_syscall = "ftruncate")]
#[inline(always)]
pub unsafe fn ftruncate(fd: int, length: off_t) -> Result<int> {
    syscall!(raw::FTRUNCATE, fd, length)
}

/// Determine CPU and NUMA node on which the calling thread is running.
#[cfg(have_syscall = "getcpu")]
#[inline(always)]
pub unsafe fn getcpu(cpu: *const uint, node: *const uint) -> Result<int> {
    syscall!(raw::GETCPU, cpu, node)
}

/// Get current working directory.
#[cfg(have_syscall = "getcwd")]
#[inline(always)]
pub unsafe fn getcwd(buf: *mut char, size: size_t) -> Result<*mut char> {
    syscall!(raw::GETCWD, buf, size)
}

/// Get the process id (PID) of the current process.
#[cfg(have_syscall = "getpid")]
#[inline(always)]
pub unsafe fn getpid() -> pid_t {
    raw::syscall0(raw::GETPID) as pid_t
}

/// Get a socket option.
#[cfg(have_syscall = "getsockopt")]
#[inline(always)]
pub unsafe fn getsockopt(
    sockfd: int,
    level: int,
    optname: int,
    optval: *mut void,
    optlen: *mut socklen_t,
) -> Result<int> {
    syscall!(raw::GETSOCKOPT, sockfd, level, optname, optval, optlen)
}

/// Arbitrary requests for file descriptors representing devices.
///
/// This system call is _particularly_ unsafe, because the final argument
/// gets interpreted in very different ways depending on the value of
/// the `request` argument. Callers must take care to ensure that `request` is
/// of an appropriate type and value for the selected `request`.
///
/// Set `arg` to `()` (empty tuple) for requests whose argument is specified
/// as "void" in the documentation.
#[cfg(have_syscall = "ioctl")]
#[inline(always)]
pub unsafe fn ioctl(fd: int, request: ulong, arg: impl crate::args::AsRawV) -> Result<int> {
    if arg.raw_is_void() {
        syscall!(raw::IOCTL, fd, request)
    } else {
        syscall!(raw::IOCTL, fd, request, arg)
    }
}

/// Listen for connections on a socket.
#[cfg(have_syscall = "listen")]
#[inline(always)]
pub unsafe fn listen(fd: int, backlog: int) -> Result<int> {
    syscall!(raw::LISTEN, fd, backlog)
}

/// Reposition the read/write offset for a file.
#[cfg(have_syscall = "lseek")]
#[inline(always)]
pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> Result<off_t> {
    syscall!(raw::LSEEK, fd, offset, whence)
}

/// Open a file.
#[cfg(have_syscall = "open")]
#[inline(always)]
pub unsafe fn open(pathname: *const char, flags: int, mode: mode_t) -> Result<int> {
    syscall!(raw::OPEN, pathname, flags, mode)
}

/// Create pipe.
#[cfg(have_syscall = "pipe")]
#[inline(always)]
pub unsafe fn pipe(fds: *mut int) -> Result<int> {
    syscall!(raw::PIPE, fds)
}

/// Create pipe.
#[cfg(have_syscall = "pipe2")]
#[inline(always)]
pub unsafe fn pipe2(fds: *mut int, flags: int) -> Result<int> {
    syscall!(raw::PIPE2, fds, flags)
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

/// Set a socket option.
#[cfg(have_syscall = "setsockopt")]
#[inline(always)]
pub unsafe fn setsockopt(
    sockfd: int,
    level: int,
    optname: int,
    optval: *const void,
    optlen: socklen_t,
) -> Result<int> {
    syscall!(raw::SETSOCKOPT, sockfd, level, optname, optval, optlen)
}

/// Create a socket endpoint for communication.
#[cfg(have_syscall = "socket")]
#[inline(always)]
pub unsafe fn socket(family: sa_family_t, typ: sock_type, protocol: int) -> Result<int> {
    syscall!(raw::SOCKET, family, typ as u32, protocol)
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

/// Truncate a file to a specified length.
#[cfg(have_syscall = "truncate")]
#[inline(always)]
pub unsafe fn truncate(path: *const char, length: off_t) -> Result<int> {
    syscall!(raw::TRUNCATE, path, length)
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
