//! Types representing results from system call wrapper functions.

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

pub(crate) trait AsRawV: Copy {
    fn from_raw_result(raw: crate::raw::V) -> Self;
    fn to_raw_arg(self) -> crate::raw::V;
}

use crate::types;

macro_rules! trivial_raw_v {
    ($t:ty) => {
        impl AsRawV for $t {
            #[inline(always)]
            fn from_raw_result(raw: crate::raw::V) -> Self {
                raw as Self
            }
            #[inline(always)]
            fn to_raw_arg(self) -> crate::raw::V {
                self as _
            }
        }
    };
}

trivial_raw_v!(types::int);
trivial_raw_v!(types::uint);
trivial_raw_v!(types::short);
trivial_raw_v!(types::ushort);
trivial_raw_v!(types::long);
trivial_raw_v!(types::ulong);
trivial_raw_v!(types::size_t);
trivial_raw_v!(types::ssize_t);

impl<T> AsRawV for *const T {
    #[inline(always)]
    fn from_raw_result(raw: crate::raw::V) -> Self {
        raw as Self
    }
    #[inline(always)]
    fn to_raw_arg(self) -> crate::raw::V {
        self as _
    }
}

impl<T> AsRawV for *mut T {
    #[inline(always)]
    fn from_raw_result(raw: crate::raw::V) -> Self {
        raw as Self
    }
    #[inline(always)]
    fn to_raw_arg(self) -> crate::raw::V {
        self as _
    }
}
