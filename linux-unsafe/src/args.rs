//! Supporting traits for preparing values to be system call arguments.

use crate::types;

/// Trait implemented by types that can be used as raw system call arguments.
pub trait AsRawV: Copy {
    fn from_raw_result(raw: crate::raw::V) -> Self;
    fn to_raw_arg(self) -> crate::raw::V;

    /// Determines whether this value should represent the absense of a
    /// value when used in a context where that makes sense, such as
    /// in the final argument of either [`crate::ioctl`] or [`crate::fcntl`]
    /// when the operation does not use the final argument.
    #[inline(always)]
    fn raw_is_void(self) -> bool {
        false
    }
}

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

impl AsRawV for () {
    #[inline(always)]
    fn from_raw_result(_: crate::raw::V) -> Self {
        ()
    }
    #[inline(always)]
    fn to_raw_arg(self) -> crate::raw::V {
        0
    }
    #[inline(always)]
    fn raw_is_void(self) -> bool {
        true
    }
}
