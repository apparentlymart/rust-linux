//! Lightweight but safe abstractions around Linux system calls related to
//! file descriptors.
#[no_std]

/// An encapsulated Linux file descriptor.
pub struct File {
    fd: linux_unsafe::int,
}

impl File {
    #[inline]
    pub unsafe fn from_raw_fd(fd: linux_unsafe::int) -> Self {
        File { fd }
    }

    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, linux_unsafe::raw::V> {
        let buf_ptr = buf.as_mut_ptr() as *mut linux_unsafe::void;
        let buf_size = buf.len();
        let result = unsafe { linux_unsafe::read(self.fd, buf_ptr, buf_size) };
        if result < 0 {
            Err(-result as linux_unsafe::raw::V)
        } else {
            Ok(result as usize)
        }
    }

    #[inline]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, linux_unsafe::raw::V> {
        let buf_ptr = buf.as_ptr() as *const linux_unsafe::void;
        let buf_size = buf.len();
        let result = unsafe { linux_unsafe::write(self.fd, buf_ptr, buf_size) };
        if result < 0 {
            Err(-result as linux_unsafe::raw::V)
        } else {
            Ok(result as usize)
        }
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

#[cfg(feature = "std")]
impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

#[cfg(feature = "std")]
impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
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
        self.fd as std::os::fd::RawFd;
    }
}
