//! Lightweight but safe abstractions around Linux system calls related to
//! file descriptors.
//!
//! This goal of this crate is to expose a convenient API while skipping any
//! unnecessary abstraction. In most cases calls to functions in this crate
//! should reduce to inline system calls and some minimal argument and result
//! conversion code, and the results should be generally unsurprising to anyone
//! who is familiar with the underlying system call behavior.
//!
//! The functions in this crate wrap functions in crate [`linux_unsafe`] to
//! actually make the system calls, and so the platform support for this
//! crate is limited to what that other crate supports.
//!
//! Implements standard library I/O traits by default, but can be made friendly
//! to `no_std` environments by disabling the default feature `std`.
//!
//! The initial versions of this crate are focused only on basic file
//! operations, until the API for that feels settled. In later releases the
//! scope will hopefully increase to cover most or all of the system calls
//! that work with file descriptors.
#![no_std]

/// Access to the "poll" system call.
pub mod poll;

/// An encapsulated Linux file descriptor.
///
/// The methods of `File` are largely just thin wrappers around Linux system
/// calls that work with file descriptors. Aside from type conversions to make
/// the API safer and more ergonomic there are no additional userspace
/// abstractions such as buffering.
///
/// When the `std` crate feature is enabled, a `File` also implements the
/// `std:io` traits `Read`, `Write`, and `Seek`.
#[repr(transparent)]
pub struct File {
    pub(crate) fd: linux_unsafe::int,
}

impl File {
    /// Wrap an existing raw file descriptor into a [`File`].
    ///
    /// Safety:
    /// - The given file descriptor must not belong to an active standard
    ///   library file or any similar wrapping abstraction.
    /// - The file descriptor must remain open and valid for the full lifetime
    ///   of the `File` object.
    /// - The same file descriptor must not be wrapped in instances of
    ///   `File`, because the first one to be dropped will close the file
    ///   descriptor.
    #[inline]
    pub unsafe fn from_raw_fd(fd: linux_unsafe::int) -> Self {
        File { fd }
    }

    /// Create a new file using the `creat` system call.
    ///
    /// This function exposes the raw `mode` argument from the underlying
    /// system call, which the caller must populate appropriately.
    #[inline]
    pub fn create_raw(path: &[u8], mode: linux_unsafe::mode_t) -> Result<Self> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let result = unsafe { linux_unsafe::creat(path_raw, mode as linux_unsafe::mode_t) };
        result
            .map(|fd| unsafe { Self::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    /// Open a file using the `open` system call.
    ///
    /// This function exposes the raw `flags` and `mode` arguments from the
    /// underlying system call, which the caller must populate appropriately.
    #[inline]
    pub fn open_raw(
        path: &[u8],
        flags: linux_unsafe::int,
        mode: linux_unsafe::mode_t,
    ) -> Result<Self> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let result = unsafe {
            linux_unsafe::open(
                path_raw,
                flags as linux_unsafe::int,
                mode as linux_unsafe::mode_t,
            )
        };
        result
            .map(|fd| unsafe { Self::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    /// Consumes the file object and returns the underlying file descriptor
    /// without closing it.
    #[inline(always)]
    pub fn into_raw_fd(self) -> linux_unsafe::int {
        let ret = self.fd;
        core::mem::forget(self);
        ret
    }

    /// Consumes the file object and closes the underlying file descriptor.
    ///
    /// If `close` fails then the file descriptor is always leaked, because
    /// there is no way to recover it once consumed.
    #[inline]
    pub fn close(mut self) -> Result<()> {
        unsafe { self.close_mut() }?;
        // Must "forget" the file because otherwise the Drop impl will
        // try to close it again, and perhaps close an unrelated file that
        // has been allocated the same fd in the meantime.
        core::mem::forget(self);
        Ok(())
    }

    /// Closes the underlying file descriptor without consuming it.
    ///
    /// Safety:
    /// - Callers must pass the file to [`core::mem::forget`] immediately
    ///   after calling this function to prevent the implicit `close` in
    ///   the [`Drop`] implementation.
    /// - Callers must not use the file object again after calling this
    ///   method; file descriptor will either be dangling or will be referring
    ///   to some other unrelated file.
    #[inline(always)]
    pub unsafe fn close_mut(&mut self) -> Result<()> {
        let result = unsafe { linux_unsafe::close(self.fd) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Read some bytes from the file into the given buffer, returning the
    /// number of bytes that were read.
    #[inline(always)]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut linux_unsafe::void;
        let buf_size = buf.len();
        unsafe { self.read_raw(buf_ptr, buf_size) }.map(|v| v as _)
    }

    /// A thin wrapper around the raw `read` system call against this file's
    /// file descriptor.
    ///
    /// Use [`File::read`] as a safe alternative.
    #[inline]
    pub unsafe fn read_raw(
        &mut self,
        buf: *mut linux_unsafe::void,
        count: linux_unsafe::size_t,
    ) -> Result<linux_unsafe::size_t> {
        let result = unsafe { linux_unsafe::read(self.fd, buf, count) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Change the current read/write position of the file.
    #[inline]
    pub fn seek(&mut self, pos: impl Into<SeekFrom>) -> Result<u64> {
        let pos = pos.into();
        let raw_offs = pos.for_raw_offset();

        #[cfg(not(target_pointer_width = "32"))]
        {
            // For 64-bit platforms we can just use lseek, because off_t is
            // bit enough for all offsets.
            let raw_whence = pos.for_raw_whence();
            let result = unsafe { linux_unsafe::lseek(self.fd, raw_offs, raw_whence) };
            result.map(|v| v as u64).map_err(|e| e.into())
        }

        #[cfg(target_pointer_width = "32")]
        {
            // For 32-bit platforms we need to use _llseek instead, which
            // splits the offset across two arguments.
            let raw_offs_high = ((raw_offs as u64) >> 32) as linux_unsafe::ulong;
            let raw_offs_low = (raw_offs as u64) as linux_unsafe::ulong;
            use core::cell::UnsafeCell;
            let result: UnsafeCell<linux_unsafe::loff_t> = UnsafeCell::new(0);
            let result_ptr = result.get();
            let raw_whence = pos.for_raw_uwhence();
            let result = unsafe {
                linux_unsafe::_llseek(self.fd, raw_offs_high, raw_offs_low, result_ptr, raw_whence)
            };
            match result {
                Ok(_) => {
                    let result_offs = unsafe { *result_ptr } as u64;
                    Ok(result_offs)
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Tell the kernel to flush any in-memory buffers and caches for the
    /// file.
    #[inline]
    pub fn sync(&mut self) -> Result<()> {
        let result = unsafe { linux_unsafe::syncfs(self.fd) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Write bytes from the given buffer to the file, returning how many bytes
    /// were written.
    #[inline(always)]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let buf_ptr = buf.as_ptr() as *const linux_unsafe::void;
        let buf_size = buf.len();
        unsafe { self.write_raw(buf_ptr, buf_size) }.map(|v| v as _)
    }

    /// A thin wrapper around the raw `write` system call against this file's
    /// file descriptor.
    ///
    /// Use [`File::write`] as a safe alternative.
    #[inline]
    pub unsafe fn write_raw(
        &mut self,
        buf: *const linux_unsafe::void,
        count: linux_unsafe::size_t,
    ) -> Result<linux_unsafe::size_t> {
        let result = unsafe { linux_unsafe::write(self.fd, buf, count) };
        result.map(|v| v as _).map_err(|e| e.into())
    }
}

impl Drop for File {
    /// Attempts to close the file when it's no longer in scope.
    ///
    /// This implicit close ignores errors, which might cause data loss if
    /// the final commit of data to disk fails. Use [`File::close`] explicitly
    /// if you need to detect errors.
    #[allow(unused_must_use)] // intentionally discarding close result
    fn drop(&mut self) {
        unsafe { self.close_mut() };
    }
}

/// [`File`] implements [`core::fmt::Write`] by passing UTF-8 encoded bytes
/// directly to the `write` method.
impl core::fmt::Write for File {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut bytes = s.as_bytes();
        while !bytes.is_empty() {
            let n = match self.write(bytes) {
                Ok(n) => n,
                Err(e) => return Err(e.into()),
            };
            bytes = &bytes[n..];
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read(buf).map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf).map_err(|e| e.into())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.sync().map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.seek(pos).map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl std::os::fd::FromRawFd for File {
    unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
        Self {
            fd: fd as linux_unsafe::int,
        }
    }
}

#[cfg(feature = "std")]
impl std::os::fd::IntoRawFd for File {
    fn into_raw_fd(self) -> std::os::fd::RawFd {
        self.into_raw_fd() as std::os::fd::RawFd
    }
}

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
    const fn for_raw_offset(self) -> linux_unsafe::loff_t {
        match self {
            SeekFrom::Start(v) => v as linux_unsafe::loff_t,
            SeekFrom::End(v) => v as linux_unsafe::loff_t,
            SeekFrom::Current(v) => v as linux_unsafe::loff_t,
        }
    }

    #[inline]
    const fn for_raw_whence(self) -> linux_unsafe::int {
        match self {
            SeekFrom::Start(_) => linux_unsafe::SEEK_SET,
            SeekFrom::End(_) => linux_unsafe::SEEK_END,
            SeekFrom::Current(_) => linux_unsafe::SEEK_CUR,
        }
    }

    #[allow(dead_code)] // only used on 32-bit platforms
    #[inline]
    const fn for_raw_uwhence(self) -> linux_unsafe::uint {
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

#[cfg(test)]
mod tests;
