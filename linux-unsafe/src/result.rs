//! Types representing results from system call wrapper functions.

use crate::args::AsRawV;

/// The result type used for all of the system call wrapper functions to
/// distinguish between success and error results.
pub type Result<T> = core::result::Result<T, Error>;

/// Represents an error code directly from the kernel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Error(pub i32);

impl Error {
    #[inline(always)]
    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }
}

#[inline(always)]
pub(crate) fn prepare_standard_result<T: AsRawV>(raw: crate::raw::V) -> Result<T> {
    crate::raw::unpack_standard_result(raw)
        .map(|raw| T::from_raw_result(raw))
        .map_err(|raw| Error::new(raw))
}

#[inline(always)]
pub(crate) fn prepare_arg<T: AsRawV>(arg: T) -> crate::raw::V {
    arg.to_raw_arg()
}

pub use crate::raw::errno::*;
