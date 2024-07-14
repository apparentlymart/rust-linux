#[cfg(feature = "std")]
extern crate std;

/// Represents a result from a kernel call that might fail.
pub type Result<T> = core::result::Result<T, Error>;

/// Represents an error code directly from the kernel.
///
/// This is a lower-level representation of an error for `no_std` situations.
/// If the crate feature `std` is enabled then `Error` can convert to
/// `std::io::Error`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Error(pub i32);

impl Error {
    #[inline(always)]
    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn into_std_io_error(self) -> std::io::Error {
        std::io::Error::from_raw_os_error(self.0)
    }
}

impl From<i32> for Error {
    #[inline(always)]
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<linux_unsafe::result::Error> for Error {
    #[inline(always)]
    fn from(value: linux_unsafe::result::Error) -> Self {
        Self::new(value.0)
    }
}

impl Into<core::fmt::Error> for Error {
    #[inline(always)]
    fn into(self) -> core::fmt::Error {
        core::fmt::Error
    }
}

#[cfg(feature = "std")]
impl Into<std::io::Error> for Error {
    #[inline(always)]
    fn into(self) -> std::io::Error {
        self.into_std_io_error()
    }
}

linux_unsafe::result::errno_derived_consts!(Error, Error);
