use crate::result::Result;
use crate::seek::SeekFrom;

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
        let result = unsafe { linux_unsafe::fsync(self.fd) };
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
