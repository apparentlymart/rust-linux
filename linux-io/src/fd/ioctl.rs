//! A safer abstraction for `ioctl`.
//!
//! The `ioctl` system call is very open-ended, supporting a variety of
//! different operations with different argument and result types, with the
//! valid operations varying drastically based on which underlying driver or
//! type of device the file represents. Passing the wrong kind of value to the
//! wrong request can cause memory corruption.
//!
//! To make `ioctl` a _little_ safer to use, this module provides the building
//! blocks for request constants that also include information about the
//! argument and result types, so that the [`super::File::ioctl`] method can
//! then provide a type-safe interface as long as these constants are defined
//! correctly. [`IoctlReq`] implementations can be defined in other crates that
//! provide support for particular device types or device drivers.

use core::mem::MaybeUninit;

use linux_unsafe::args::AsRawV;
use linux_unsafe::{int, ulong};

/// Represents a device type that can have `ioctl` requests implemented for it.
///
/// Implementations of this type should typically be zero-sized types, because
/// they are used only as markers for annotating `File` objects using
/// [`super::File::to_device`].
pub trait IoDevice {}

/// Represents a particular request that can be issue with the `ioctl` system call.
///
/// Safety: Implementers must ensure that they only generate valid combinations
/// of `ioctl` request and raw argument.
pub unsafe trait IoctlReq<'a, Device: IoDevice> {
    /// The type that the caller will provide when using this `ioctl` command.
    ///
    /// Use `()` for requests that don't need a caller-provided argument, such
    /// as those which only return some data.
    type ExtArg
    where
        Self: 'a;

    /// The type of some temporary memory that the request needs to do its
    /// work.
    ///
    /// For a request that only returns data, this will typically
    /// describe the layout of the returned data, which the kernel will
    /// then populate. For requests that don't need this, use `()`.
    type TempMem;

    /// The type of argument that will be passed to the raw system call.
    type RawArg: AsRawV;

    /// The type of the result of the `fcntl` call.
    type Result
    where
        Self: 'a;

    /// Prepare the `cmd` and `arg` values for a `ioctl` system call.
    ///
    /// The `arg` parameter is the argument provided by the caller of the
    /// [`super::File::ioctl`] function. `temp_mem` is a reference to
    /// uninitialized memory of appropriate size and alignment for
    /// [`Self::TempMem`], which the implementer can either leave uninitialized
    /// for the kernel to populate or pre-initialize with data the
    /// kernel will expect to find there.
    fn prepare_ioctl_args(
        &self,
        arg: &Self::ExtArg,
        temp_mem: &mut MaybeUninit<Self::TempMem>,
    ) -> (ulong, Self::RawArg);

    /// Prepare a raw successful result from a `ioctl` call to be returned.
    fn prepare_ioctl_result(
        &self,
        raw: int,
        arg: &Self::ExtArg,
        temp_mem: &MaybeUninit<Self::TempMem>,
    ) -> Self::Result;
}

/// Constructs a new read-only [`IoctlReq`] with a fixed request code that
/// passes no payload to `ioctl` and returns its result in the system call
/// return value.
///
/// Safety: Callers must ensure that the given `request` is valid.
pub const unsafe fn ioctl_no_arg<Device, Result>(request: ulong) -> IoctlReqNoArgs<Device, Result>
where
    *mut Result: AsRawV,
    Device: IoDevice,
    Result: FromIoctlResult<int>,
{
    IoctlReqNoArgs::<Device, Result> {
        request,
        _phantom: core::marker::PhantomData,
    }
}

/// Constructs a new read-only [`IoctlReq`] with a fixed request code and
/// a result type that maps directly to the data the kernel will
/// provide.
///
/// Safety: Callers must ensure that the given `request` is valid, that
/// type `T` describes what this request expects to get a pointer to, and
/// that the kernel will populate the given `T` object with data that is
/// consistent with Rust's expectations for the given type.
pub const unsafe fn ioctl_read<Device, Result>(request: ulong) -> IoctlReqRead<Device, Result>
where
    *mut Result: AsRawV,
    Device: IoDevice,
    Result: Copy,
{
    IoctlReqRead::<Device, Result> {
        request,
        _phantom: core::marker::PhantomData,
    }
}

/// Constructs a new write-only [`IoctlReq`] with a fixed request code and
/// an argument type that maps directly to the data the kernel excepts
/// to receive a pointer to.
///
/// Safety: Callers must ensure that the given `request` is valid, that
/// type `T` describes what this request expects to get a pointer to, and
/// that it isn't possible for any value of that type to cause the kernel
/// to violate memory safety. In particular, the kernel must not modify
/// the given memory, because the safe caller will provide a shared reference.
pub const unsafe fn ioctl_write<Device, Arg, Result>(
    request: ulong,
) -> IoctlReqWrite<Device, Arg, Result>
where
    *const Arg: AsRawV,
    Device: IoDevice,
    Result: FromIoctlResult<int>,
{
    IoctlReqWrite::<Device, Arg, Result> {
        request,
        _phantom: core::marker::PhantomData,
    }
}

/// Implementation of [`IoctlReq`] with a fixed `cmd` and passing no arguments
/// at all, just returning the kernel's result value.
///
/// This is for the less common `ioctl` requests that indicate more than just
/// success in their result, and so callers need to obtain that result.
#[repr(transparent)]
pub struct IoctlReqNoArgs<Device: IoDevice, Result> {
    request: ulong,
    _phantom: core::marker::PhantomData<(Device, Result)>,
}

unsafe impl<'a, Device, Result> IoctlReq<'a, Device> for IoctlReqNoArgs<Device, Result>
where
    Result: 'a + FromIoctlResult<int>,
    Device: 'a + IoDevice,
{
    type ExtArg = ();
    type TempMem = ();
    type RawArg = ();
    type Result = Result;

    #[inline(always)]
    fn prepare_ioctl_args(
        &self,
        _: &Self::ExtArg,
        _: &mut MaybeUninit<Self::TempMem>,
    ) -> (ulong, Self::RawArg) {
        (self.request, ())
    }

    #[inline(always)]
    fn prepare_ioctl_result(
        &self,
        raw: int,
        _: &Self::ExtArg,
        _: &MaybeUninit<Self::TempMem>,
    ) -> Self::Result {
        Self::Result::from_ioctl_result(&raw)
    }
}

/// Implementation of [`IoctlReq`] with a fixed `cmd` value and passing a
/// pointer to a zeroed memory block of type `Result` directly through to the
/// underlying system call and then returnin a copy of that memory.
#[repr(transparent)]
pub struct IoctlReqRead<Device: IoDevice, Result>
where
    *mut Result: AsRawV,
    Result: Copy,
{
    request: ulong,
    _phantom: core::marker::PhantomData<(Device, Result)>,
}

unsafe impl<'a, Device, Result> IoctlReq<'a, Device> for IoctlReqRead<Device, Result>
where
    *mut Result: AsRawV,
    Result: 'a + Copy,
    Device: 'a + IoDevice,
{
    type ExtArg = ();
    type TempMem = Result;
    type RawArg = *mut Result;
    type Result = Result;

    #[inline(always)]
    fn prepare_ioctl_args(
        &self,
        _: &Self::ExtArg,
        temp_mem: &mut MaybeUninit<Self::TempMem>,
    ) -> (ulong, Self::RawArg) {
        (self.request, temp_mem.as_mut_ptr())
    }

    #[inline(always)]
    fn prepare_ioctl_result(
        &self,
        _: int,
        _: &Self::ExtArg,
        temp_mem: &MaybeUninit<Self::TempMem>,
    ) -> Self::Result {
        unsafe { temp_mem.assume_init() }
    }
}

/// Implementation of [`IoctlReq`] with a fixed `cmd` value and passing a
/// .
#[repr(transparent)]
pub struct IoctlReqWrite<Device: IoDevice, Arg, Result = int>
where
    *const Arg: AsRawV,
{
    request: ulong,
    _phantom: core::marker::PhantomData<(Device, Arg, Result)>,
}

unsafe impl<'a, Device, Arg, Result> IoctlReq<'a, Device> for IoctlReqWrite<Device, Arg, Result>
where
    *const Arg: AsRawV,
    Arg: 'a,
    Result: 'a + FromIoctlResult<int>,
    Device: 'a + IoDevice,
{
    type ExtArg = &'a Arg;
    type TempMem = ();
    type RawArg = *const Arg;
    type Result = Result;

    #[inline(always)]
    fn prepare_ioctl_args(
        &self,
        arg: &Self::ExtArg,
        _: &mut MaybeUninit<Self::TempMem>,
    ) -> (ulong, *const Arg) {
        (self.request, (*arg) as *const Arg)
    }

    #[inline(always)]
    fn prepare_ioctl_result(
        &self,
        ret: int,
        _: &Self::ExtArg,
        _: &MaybeUninit<Self::TempMem>,
    ) -> Self::Result {
        Result::from_ioctl_result(&ret)
    }
}

/// Trait for types that can be constructed automatically from `ioctl` results
/// from requests with a given argument type and temporary value type.
pub trait FromIoctlResult<Raw> {
    fn from_ioctl_result(raw: &Raw) -> Self;
}

impl FromIoctlResult<int> for int {
    fn from_ioctl_result(raw: &int) -> Self {
        *raw
    }
}

impl<Device: IoDevice> FromIoctlResult<int> for super::File<Device> {
    fn from_ioctl_result(raw: &int) -> Self {
        unsafe { super::File::from_raw_fd(*raw) }
    }
}

#[allow(non_snake_case)]
const fn _IOC(dir: ulong, typ: ulong, nr: ulong, size: ulong) -> ulong {
    (dir << 30) | (typ << 8) | (nr << 0) | (size << 16)
}

/// Equivalent to the kernel macro `_IO` for defining ioctl request numbers that
/// neither read nor write within the standard numbering scheme.
#[allow(non_snake_case)]
pub const fn _IO(typ: ulong, nr: ulong) -> ulong {
    _IOC(0, typ, nr, 0)
}

/// Equivalent to the kernel macro `_IOR` for defining ioctl request numbers
/// where userspace reads data from the kernel.
#[allow(non_snake_case)]
pub const fn _IOR(typ: ulong, nr: ulong, size: ulong) -> ulong {
    _IOC(2, typ, nr, size)
}

/// Equivalent to the kernel macro `_IOW` for defining ioctl request numbers
/// where userspace writes data to the kernel.
#[allow(non_snake_case)]
pub const fn _IOW(typ: ulong, nr: ulong, size: ulong) -> ulong {
    _IOC(1, typ, nr, size)
}

/// Equivalent to the kernel macro `_IOWR` for defining ioctl request numbers
/// where userspace writes data to the kernel _and_ the kernel returns data
/// back to userspace.
#[allow(non_snake_case)]
pub const fn _IOWR(typ: ulong, nr: ulong, size: ulong) -> ulong {
    _IOC(1 | 2, typ, nr, size)
}
