#[cfg(feature = "std")]
extern crate std;

/// Used with [`File::seek`] to specify the starting point and offset.
///
/// This is just a copy of `std::io::SeekFrom`, included here to allow this
/// crate to work in `no_std` environments. If this crate's `std` feature
/// is enabled then `SeekFrom` can convert to and from `std::io::SeekFrom`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

impl SeekFrom {
    #[inline]
    pub const fn for_raw_offset(self) -> linux_unsafe::loff_t {
        match self {
            SeekFrom::Start(v) => v as linux_unsafe::loff_t,
            SeekFrom::End(v) => v as linux_unsafe::loff_t,
            SeekFrom::Current(v) => v as linux_unsafe::loff_t,
        }
    }

    #[inline]
    pub const fn for_raw_whence(self) -> linux_unsafe::int {
        match self {
            SeekFrom::Start(_) => linux_unsafe::SEEK_SET,
            SeekFrom::End(_) => linux_unsafe::SEEK_END,
            SeekFrom::Current(_) => linux_unsafe::SEEK_CUR,
        }
    }

    #[allow(dead_code)] // only used on 32-bit platforms
    #[inline]
    pub const fn for_raw_uwhence(self) -> linux_unsafe::uint {
        self.for_raw_whence() as linux_unsafe::uint
    }
}

#[cfg(feature = "std")]
impl From<std::io::SeekFrom> for SeekFrom {
    fn from(value: std::io::SeekFrom) -> Self {
        match value {
            std::io::SeekFrom::Start(v) => Self::Start(v),
            std::io::SeekFrom::End(v) => Self::End(v),
            std::io::SeekFrom::Current(v) => Self::Current(v),
        }
    }
}

#[cfg(feature = "std")]
impl Into<std::io::SeekFrom> for SeekFrom {
    fn into(self) -> std::io::SeekFrom {
        match self {
            SeekFrom::Start(v) => std::io::SeekFrom::Start(v),
            SeekFrom::End(v) => std::io::SeekFrom::End(v),
            SeekFrom::Current(v) => std::io::SeekFrom::Current(v),
        }
    }
}
