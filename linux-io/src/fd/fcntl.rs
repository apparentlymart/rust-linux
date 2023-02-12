//! A safer abstraction for `fcntl`.
//!
//! The `fcntl` system call is very open-ended, supporting a variety of
//! different operations with different argument and result types. Passing
//! the wrong kind of value to the wrong command can cause memory corruption.
//!
//! To make `fcntl` a _little_ safer to use, this module provides command
//! constants that also include information about the argument and result
//! types, so that the [`super::File::fcntl`] method can then provide a
//! type-safe interface as long as these constants are defined correctly.

use linux_unsafe::args::AsRawV;
use linux_unsafe::int;

/// Duplicate the file descriptor fd using the lowest-numbered
/// available file descriptor greater than or equal to `arg`.
pub const F_DUPFD: DirectFcntlCmd<int, super::File> = unsafe { fcntl_cmd(0) };

/// Retrieve the function descriptor flags.
pub const F_GETFD: DirectFcntlCmd<(), int> = unsafe { fcntl_cmd(1) };

/// Set the function descriptor flags.
pub const F_SETFD: DirectFcntlCmd<int, ()> = unsafe { fcntl_cmd(2) };

/// Retrieve the file access mode and the file status flags.
pub const F_GETFL: DirectFcntlCmd<(), int> = unsafe { fcntl_cmd(3) };

/// Set the file status flags.
pub const F_SETFL: DirectFcntlCmd<int, ()> = unsafe { fcntl_cmd(4) };

/// Place a lock on the file.
pub const F_GETLK: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(5) };

/// Acquire a lock or release a lock on the file.
pub const F_SETLK: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(6) };

/// Acquire a lock or release a lock on the file, waiting for any conflicting
/// lock to be released.
pub const F_SETLKW: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(7) };

/// Place a lock on the file description of the file.
pub const F_OFD_GETLK: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(36) };

/// Acquire a lock or release a lock on the file description of the file.
pub const F_OFD_SETLK: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(37) };

/// Acquire a lock or release a lock on the file description of the file,
/// waiting for any conflicting lock to be released.
pub const F_OFD_SETLKW: MutPtrFcntlCmd<linux_unsafe::flock, ()> = unsafe { fcntl_cmd_mut_ptr(38) };

const F_LINUX_SPECIFIC_BASE: int = 1024;

/// Duplicate the file descriptor fd using the lowest-numbered
/// available file descriptor greater than or equal to `arg`,
/// setting the `FD_CLOEXEC` flag on the new descriptor.
pub const F_DUPFD_CLOEXEC: DirectFcntlCmd<int, int> =
    unsafe { fcntl_cmd(F_LINUX_SPECIFIC_BASE + 6) };

/// Represents a particular command that can be used with the `fcntl` system call.
///
/// Safety: Implementers must ensure that they only generate valid combinations
/// of `fcntl` command and raw argument.
pub unsafe trait FcntlCmd<'a> {
    /// The type that the caller will provide when using this `fcntl` command.
    type ExtArg
    where
        Self: 'a;

    /// The type of argument that will be passed to the raw system call.
    type RawArg: AsRawV;

    /// The type of the result of the `fcntl` call.
    type Result;

    /// Prepare the `cmd` and `arg` values for a `fcntl` system call.
    fn prepare_fcntl_args(&self, arg: Self::ExtArg) -> (int, Self::RawArg);

    /// Prepare a raw successful result from a `fcntl` call to be returned.
    fn prepare_fcntl_result(&self, raw: int) -> Self::Result;
}

/// Constructs a new "simple" [`FcntlCmd`] with a fixed command code and
/// an argument type that maps directly to the raw system call argument.
///
/// Safety: Callers must ensure that the given `cmd` is valid and that
/// type `T` is the type that command expects.
pub const unsafe fn fcntl_cmd<Arg, Result>(cmd: int) -> DirectFcntlCmd<Arg, Result>
where
    Arg: AsRawV,
    Result: FromFcntlResult,
{
    DirectFcntlCmd::<Arg, Result> {
        cmd,
        _phantom: core::marker::PhantomData,
    }
}

/// Constructs a new [`FcntlCmd`] with a fixed command code and
/// an argument type that will be passed as a raw const pointer.
///
/// Safety: Callers must ensure that the given `cmd` is valid and that
/// type `T` is the type that command expects.
pub const unsafe fn fcntl_cmd_const_ptr<Arg, Result>(cmd: int) -> ConstPtrFcntlCmd<Arg, Result>
where
    *const Arg: AsRawV,
    Result: FromFcntlResult,
{
    ConstPtrFcntlCmd::<Arg, Result> {
        cmd,
        _phantom: core::marker::PhantomData,
    }
}

/// Constructs a new [`FcntlCmd`] with a fixed command code and
/// an argument type that will be passed as a raw mut pointer.
///
/// Safety: Callers must ensure that the given `cmd` is valid and that
/// type `T` is the type that command expects.
pub const unsafe fn fcntl_cmd_mut_ptr<Arg, Result>(cmd: int) -> MutPtrFcntlCmd<Arg, Result>
where
    *mut Arg: AsRawV,
    Result: FromFcntlResult,
{
    MutPtrFcntlCmd::<Arg, Result> {
        cmd,
        _phantom: core::marker::PhantomData,
    }
}

/// Implementation of [`FcntlCmd`] with a fixed `cmd` value and passing the
/// arg directly through to the underlying system call.
#[repr(transparent)]
pub struct DirectFcntlCmd<Arg: AsRawV, Result: FromFcntlResult> {
    cmd: int,
    _phantom: core::marker::PhantomData<(Arg, Result)>,
}

unsafe impl<'a, Arg: AsRawV, Result: FromFcntlResult> FcntlCmd<'a> for DirectFcntlCmd<Arg, Result> {
    type ExtArg = Arg where Self: 'a;
    type RawArg = Arg;

    fn prepare_fcntl_args(&self, arg: Arg) -> (int, Self::RawArg) {
        (self.cmd, arg)
    }

    type Result = Result;

    fn prepare_fcntl_result(&self, raw: int) -> Self::Result {
        unsafe { Self::Result::prepare_result(raw) }
    }
}

/// Implementation of [`FcntlCmd`] with a fixed `cmd` value and passing the
/// arg through to the underlying system call as a raw const pointer.
#[repr(transparent)]
pub struct ConstPtrFcntlCmd<Arg, Result: FromFcntlResult> {
    cmd: int,
    _phantom: core::marker::PhantomData<(Arg, Result)>,
}

unsafe impl<'a, Arg, Result: FromFcntlResult> FcntlCmd<'a> for ConstPtrFcntlCmd<Arg, Result> {
    type ExtArg = &'a Arg where Self: 'a;
    type RawArg = *const Arg;

    fn prepare_fcntl_args(&self, arg: &'a Arg) -> (int, Self::RawArg) {
        (self.cmd, arg as *const Arg)
    }

    type Result = Result;

    fn prepare_fcntl_result(&self, raw: int) -> Self::Result {
        unsafe { Self::Result::prepare_result(raw) }
    }
}

/// Implementation of [`FcntlCmd`] with a fixed `cmd` value and passing the
/// arg through to the underlying system call as a raw mut pointer.
#[repr(transparent)]
pub struct MutPtrFcntlCmd<Arg, Result: FromFcntlResult> {
    cmd: int,
    _phantom: core::marker::PhantomData<(Arg, Result)>,
}

unsafe impl<'a, Arg, Result: FromFcntlResult> FcntlCmd<'a> for MutPtrFcntlCmd<Arg, Result> {
    type ExtArg = &'a mut Arg where Self: 'a;
    type RawArg = *mut Arg;

    fn prepare_fcntl_args(&self, arg: &'a mut Arg) -> (int, Self::RawArg) {
        (self.cmd, arg as *mut Arg)
    }

    type Result = Result;

    fn prepare_fcntl_result(&self, raw: int) -> Self::Result {
        unsafe { Self::Result::prepare_result(raw) }
    }
}

/// Trait for types that can be returned by `fcntl` calls.
pub trait FromFcntlResult {
    unsafe fn prepare_result(raw: int) -> Self;
}

impl FromFcntlResult for int {
    #[inline(always)]
    unsafe fn prepare_result(raw: int) -> Self {
        raw
    }
}

impl FromFcntlResult for () {
    #[inline(always)]
    unsafe fn prepare_result(_: int) -> () {
        ()
    }
}

impl FromFcntlResult for super::File {
    #[inline(always)]
    unsafe fn prepare_result(raw: int) -> Self {
        super::File::from_raw_fd(raw)
    }
}
