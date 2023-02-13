//! A safer abstraction for `setsockopt` and `getsockopt`.
//!
//! These system calls are very open-ended, supporting a variety of
//! different operations with different argument and result types. Passing
//! the wrong kind of value to the wrong command can cause memory corruption.
//!
//! To make these a _little_ safer to use, this module provides option
//! constants that also include information about the argument and result
//! types, so that the [`super::File::setsockopt`] and
//! [`super::File::getsockopt`] methods can then provide a type-safe interface
//! as long as these constants are defined correctly.
//!
//! This module defines constants for the main sockets API options.
//! Protocol-specific options might also be available in other modules,
//! or potentially in other crates.

use linux_unsafe::int;

/// The sockopt "level" for general socket options that are not protocol-specific.
pub const SOL_SOCKET: int = 1;

/// Indicates whether or not this socket has been marked to accept connections
/// e.g. using [`super::File::listen`].
///
/// `1` indicates that the socket is listening, and `0` that it is not.
pub const SO_ACCEPTCONN: DirectSockOptReadOnly<int> = unsafe { sockopt_readonly(SOL_SOCKET, 30) };

/// Returns the protocol/address family for this socket.
///
/// The result can be converted to [`linux_unsafe::sa_family_t`] to compare
/// with the address family constants defined elsewhere in this crate.
pub const SO_DOMAIN: DirectSockOptReadOnly<int> = unsafe { sockopt_readonly(SOL_SOCKET, 39) };

/// Send only to directly-connected hosts, ignoring any configured gateway.
///
/// `1` indicates direct connections only, while `0` allows using gateways.
pub const SO_DONTROUTE: DirectSockOpt<int> = unsafe { sockopt(SOL_SOCKET, 5) };

/// Send period keepalive messages on connection-oriented sockets.
///
/// `1` enables keepalive messages, while `0` disables them.
pub const SO_KEEPALIVE: DirectSockOpt<int> = unsafe { sockopt(SOL_SOCKET, 9) };

/// Implemented by options that can be used with `setsockopt`.
///
/// Safety: Implementers must ensure that they only generate valid combinations
/// of `setsockopt` level, optname, optval, and optlen.
pub unsafe trait SetSockOpt<'a> {
    /// The type that the caller will provide when setting this option.
    type ExtArg
    where
        Self: 'a;

    /// The type that "optval" will be a pointer to in the call.
    type OptVal;

    /// The type of the result of the `setsockopt` call.
    type Result;

    /// Prepare the arguments for a `setsockopt` system call. The tuple
    /// elements of the result are `(level, optname, optval, optlen)`.
    fn prepare_setsockopt_args(
        &self,
        arg: &Self::ExtArg,
    ) -> (int, int, *const Self::OptVal, linux_unsafe::socklen_t);

    /// Prepare a raw successful result from a `setsockopt` call to be returned.
    fn prepare_setsockopt_result(&self, raw: int) -> Self::Result;
}

/// Implemented by options that can be used with `getsockopt`.
///
/// Safety: Implementers must ensure that they only generate valid combinations
/// of `getsockopt` level, optname, optval, and optlen.
pub unsafe trait GetSockOpt<'a> {
    /// The type that "optval" will be a pointer to in the call.
    type OptVal;

    /// The type of the result of the `getsockopt` call.
    type Result
    where
        Self: 'a;

    /// Prepare the arguments for a `getsockopt` system call. The tuple
    /// elements of the result are `(level, optname, optlen)`.
    fn prepare_getsockopt_args(&self) -> (int, int);

    /// Prepare a raw successful result from a `getsockopt` call to be returned.
    fn prepare_getsockopt_result(&self, retval: int, optval: Self::OptVal) -> Self::Result;
}

/// Constructs a new "simple" socket option whose safe-facing argument
/// type is the same as its internal type and whose level and option name
/// are fixed.
///
/// Types used with this implementation should typically be `repr(C)` and
/// designed to exactly match the layout of the option's kernel structure.
///
/// Safety: Callers must ensure that the given `level` and `optname` are valid
/// and that type `T` is the type that the corresponding option expects.
pub const unsafe fn sockopt<T>(level: int, optname: int) -> DirectSockOpt<T> {
    DirectSockOpt::<T> {
        level,
        optname,
        _phantom: core::marker::PhantomData,
    }
}

/// Constructs a new "simple" socket option that is read-only.
///
/// Aside from the result only supporting `getsockopt`, this is the same
/// as [`sockopt`].
pub const unsafe fn sockopt_readonly<T>(level: int, optname: int) -> DirectSockOptReadOnly<T> {
    DirectSockOptReadOnly(sockopt::<T>(level, optname))
}

/// Implementation of both [`SetSockOpt`] and [`GetSockOpt`] with fixed `level`
/// and `optname` values, passing the arg type directly through to the
/// underlying system calls.
pub struct DirectSockOpt<T> {
    level: int,
    optname: int,
    _phantom: core::marker::PhantomData<T>,
}

/// Implementation of just [`GetSockOpt`] with fixed `level` and `optname`
/// values, similar to [`DirectSockOpt`] but for read-only options.
#[repr(transparent)]
pub struct DirectSockOptReadOnly<T>(DirectSockOpt<T>);

unsafe impl<'a, T: 'a> SetSockOpt<'a> for DirectSockOpt<T> {
    type ExtArg = T;
    type OptVal = T;
    type Result = int;

    fn prepare_setsockopt_args(
        &self,
        arg: &Self::ExtArg,
    ) -> (int, int, *const Self::OptVal, linux_unsafe::socklen_t) {
        (
            self.level,
            self.optname,
            arg as *const Self::OptVal,
            core::mem::size_of::<Self::OptVal>() as linux_unsafe::socklen_t,
        )
    }

    fn prepare_setsockopt_result(&self, raw: int) -> Self::Result {
        raw
    }
}

unsafe impl<'a, T: 'a> GetSockOpt<'a> for DirectSockOpt<T> {
    type OptVal = T;
    type Result = T;

    fn prepare_getsockopt_args(&self) -> (int, int) {
        (self.level, self.optname)
    }

    fn prepare_getsockopt_result(&self, _: int, optval: T) -> Self::Result {
        optval
    }
}

unsafe impl<'a, T: 'a> GetSockOpt<'a> for DirectSockOptReadOnly<T> {
    type OptVal = T;
    type Result = T;

    fn prepare_getsockopt_args(&self) -> (int, int) {
        self.0.prepare_getsockopt_args()
    }

    fn prepare_getsockopt_result(&self, ret: int, optval: T) -> Self::Result {
        self.0.prepare_getsockopt_result(ret, optval)
    }
}
