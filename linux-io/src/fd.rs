use linux_unsafe::args::AsRawV;
use linux_unsafe::void;

use crate::result::{self, Result};
use crate::seek::SeekFrom;

use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::mem::MaybeUninit;

use self::ioctl::SubDevice;

pub mod fcntl;
pub mod ioctl;
pub mod sockopt;

mod direntry;
pub use direntry::*;

/// An encapsulated Linux file descriptor.
///
/// The methods of `File` are largely just thin wrappers around Linux system
/// calls that work with file descriptors. Aside from type conversions to make
/// the API safer and more ergonomic there are no additional userspace
/// abstractions such as buffering.
///
/// When the `std` crate feature is enabled, a `File` also implements the
/// `std:io` traits `Read`, `Write`, and `Seek`.
///
/// A `File` can have an optional `Device` type parameter, which if set to
/// an implementation of `IODevice` enables the `ioctl` method to accept
/// request constants that are declared as being compatible with that device.
/// Otherwise, the `ioctl` method is unavailable.
#[repr(transparent)]
pub struct File<Device = ()> {
    pub(crate) fd: linux_unsafe::int,
    _phantom: core::marker::PhantomData<Device>,
}

impl File<()> {
    /// Open an existing file.
    ///
    /// Use this function for `OpenOptions` that don't require a mode. If you
    /// set the "create" option then you will need to use
    /// [`Self::open_with_mode`] instead, to specify the mode of the new file.
    #[inline(always)]
    pub fn open(path: &CStr, options: OpenOptions<OpenWithoutMode>) -> Result<Self> {
        Self::open_raw(path, options.flags, 0)
    }

    /// Open a file, creating it if necessary using the given file mode.
    ///
    /// Use this function only for `OpenOptions` that require a mode. For
    /// most options you can use [`Self::open`] instead.
    #[inline(always)]
    pub fn open_with_mode(
        path: &CStr,
        options: OpenOptions<OpenWithMode>,
        mode: linux_unsafe::mode_t,
    ) -> Result<Self> {
        Self::open_raw(path, options.flags, mode)
    }

    /// Open a file using the `openat` system call.
    ///
    /// This function exposes the raw `flags` and `mode` arguments from the
    /// underlying system call, which the caller must populate appropriately.
    #[inline]
    pub fn open_raw(
        path: &CStr,
        flags: linux_unsafe::int,
        mode: linux_unsafe::mode_t,
    ) -> Result<Self> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let result = unsafe {
            linux_unsafe::openat(
                linux_unsafe::AT_FDCWD,
                path_raw,
                flags as linux_unsafe::int,
                mode as linux_unsafe::mode_t,
            )
        };
        result
            .map(|fd| unsafe { Self::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    /// Create a new file using the `openat` system call.
    ///
    /// This function exposes the raw `mode` argument from the underlying
    /// system call, which the caller must populate appropriately.
    #[inline]
    pub fn create_raw(path: &CStr, mode: linux_unsafe::mode_t) -> Result<Self> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let result = unsafe {
            linux_unsafe::openat(
                linux_unsafe::AT_FDCWD,
                path_raw,
                linux_unsafe::O_CREAT | linux_unsafe::O_WRONLY | linux_unsafe::O_TRUNC,
                mode as linux_unsafe::mode_t,
            )
        };
        result
            .map(|fd| unsafe { Self::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    /// Create a new socket using the `socket` system call.
    ///
    /// The protocol is specifed as a special typed constant which carries
    /// both the protocol number expected by the kernel and the device type
    /// to use for the returned file, so the result can accept `ioctl`
    /// requests that are defined for that specific protocol.
    #[inline]
    pub fn socket<Protocol: super::socket::SocketProtocol>(
        domain: linux_unsafe::sa_family_t,
        typ: crate::socket::sock_type,
        protocol: Protocol,
    ) -> Result<File<Protocol::Device>> {
        let result = unsafe { linux_unsafe::socket(domain, typ, protocol.raw_protocol_num()) };
        result
            .map(|fd| unsafe { File::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    /// Create a new socket using the `socket` system call without automatically
    /// assigning a device type based on the protocol.
    ///
    /// This is similar to [`Self::socket`] but allows specifying any arbitrary
    /// protocol number without needing a special implementation of
    /// [`super::socket::SocketProtocol`]. However, that means that the result
    /// will be typed only as a generic socket and so will not accept any
    /// protocol-specific `ioctl` requests.
    #[inline]
    pub fn socket_raw<Protocol: super::socket::SocketProtocol>(
        domain: linux_unsafe::sa_family_t,
        typ: crate::socket::sock_type,
        protocol: linux_unsafe::int,
    ) -> Result<File<crate::socket::SocketDevice>> {
        let result = unsafe { linux_unsafe::socket(domain, typ, protocol) };
        result
            .map(|fd| unsafe { File::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }
}

impl<Device> File<Device> {
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
        File {
            fd,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Creates a new file descriptor referring to the same underlying file
    /// description as `self`.
    ///
    /// Note that the read/write position of a file is a property of its file
    /// description rather than its descriptor, and so modifying the position
    /// (and some other aspects) of the new file will also affect the original.
    #[inline]
    pub fn duplicate(&self) -> Result<Self> {
        let result = unsafe { linux_unsafe::dup(self.fd) };
        result
            .map(|fd| unsafe { Self::from_raw_fd(fd) })
            .map_err(|e| e.into())
    }

    /// Open a file relative to the current file, which must represent a
    /// directory.
    #[inline]
    pub fn open_relative(
        &self,
        path: &CStr,
        options: OpenOptions<OpenWithoutMode>,
    ) -> Result<File<()>> {
        self.open_relative_raw(path, options.flags, 0)
    }

    /// Open a file relative to the current file, which must represent a
    /// directory.
    #[inline]
    pub fn open_relative_with_mode(
        &self,
        path: &CStr,
        options: OpenOptions<OpenWithMode>,
        mode: linux_unsafe::mode_t,
    ) -> Result<File<()>> {
        self.open_relative_raw(path, options.flags, mode)
    }

    /// Open a file using the `openat` system call.
    ///
    /// This function exposes the raw `flags` and `mode` arguments from the
    /// underlying system call, which the caller must populate appropriately.
    #[inline]
    pub fn open_relative_raw(
        &self,
        path: &CStr,
        flags: linux_unsafe::int,
        mode: linux_unsafe::mode_t,
    ) -> Result<File<()>> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let result = unsafe {
            linux_unsafe::openat(
                self.fd,
                path_raw,
                flags as linux_unsafe::int,
                mode as linux_unsafe::mode_t,
            )
        };
        result
            .map(|fd| unsafe { File::from_raw_fd(fd as linux_unsafe::int) })
            .map_err(|e| e.into())
    }

    #[inline(always)]
    pub fn fd(&self) -> linux_unsafe::int {
        self.fd
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
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
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
        &self,
        buf: *mut linux_unsafe::void,
        count: linux_unsafe::size_t,
    ) -> Result<linux_unsafe::size_t> {
        let result = unsafe { linux_unsafe::read(self.fd, buf, count) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Read some directory entries from the directory into the given buffer,
    /// and obtain an iterator over those directory entries.
    ///
    /// The caller **must** fully-consume the returned iterator; any items
    /// not retrieved will be lost.
    ///
    /// Once the iterator is dropped the original buffer contains the raw
    /// directory entries returned from the kernel, and can be used again for
    /// a subsequent call to this function.
    #[inline(always)]
    pub fn getdents<'a>(&self, buf: &'a mut [u8]) -> Result<DirEntries<'a>> {
        let buf_ptr = buf.as_mut_ptr() as *mut linux_unsafe::void;
        let buf_size = buf.len();
        if buf_size > (linux_unsafe::int::MAX as usize) {
            return Err(result::EINVAL);
        }
        let populated_size =
            unsafe { self.getdents_raw(buf_ptr, buf_size as linux_unsafe::int) }? as usize;
        Ok(DirEntries::from_getdents64_buffer(&buf[..populated_size]))
    }

    /// Read a transformation of every directory entry from the directory.
    ///
    /// This wrapper around [`Self::getdents`] returns an iterator that can
    /// call `Self::getdents` multiple times to visit all of the entries in
    /// the directory.
    ///
    /// However, since all of the calls to `Self::getdents` write into the
    /// same buffer `buf` it isn't possible for the iterator to directly
    /// return the borrowed directory entries. Instead, each entry is passed
    /// to the given function `transform`, which must then return a
    /// representation of that entry that can outlive the buffer contents.
    ///
    /// This admittedly-awkward compromise means that the caller can decide
    /// how and whether to allocate memory to retain ownership of the directory
    /// entry names, so that this crate can avoid imposing any particular
    /// opinion about that.
    #[inline(always)]
    pub fn getdents_all<'file, 'buf, TF, R>(
        &'file self,
        buf: &'buf mut [u8],
        transform: TF,
    ) -> AllDirEntries<'file, 'buf, TF, R, Device>
    where
        TF: for<'tmp> FnMut(DirEntry<'tmp>) -> R,
        'buf: 'file,
    {
        AllDirEntries::new(self, buf, transform)
    }

    /// A thin wrapper around the raw `getdents64` system call against this
    /// file's file descriptor.
    ///
    /// Use [`File::getdents`] as a more convenient alternative.
    #[inline]
    pub unsafe fn getdents_raw(
        &self,
        buf: *mut linux_unsafe::void,
        buf_len: linux_unsafe::int,
    ) -> Result<linux_unsafe::size_t> {
        let result = unsafe { linux_unsafe::getdents64(self.fd, buf, buf_len) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Read the content of the symbolic link that the file refers to.
    ///
    /// This makes sense only for a file that was opened with the "path only"
    /// and "no follow symlinks" options.
    #[inline(always)]
    pub fn readlink<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8]> {
        let path = c"";
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let buf_ptr = buf.as_mut_ptr() as *mut linux_unsafe::char;
        let buf_size = buf.len();
        if buf_size > (linux_unsafe::int::MAX as usize) {
            return Err(result::EINVAL);
        }
        let result = unsafe { linux_unsafe::readlinkat(self.fd, path_raw, buf_ptr, buf_size) };
        match result {
            Ok(populated_size) => Ok(&buf[..populated_size as usize]),
            Err(e) => Err(e.into()),
        }
    }

    /// Read the content of a symbolic link relative to the file, which
    /// must represent a directory.
    #[inline(always)]
    pub fn readlink_relative<'a>(&self, path: &CStr, buf: &'a mut [u8]) -> Result<&'a [u8]> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let buf_ptr = buf.as_mut_ptr() as *mut linux_unsafe::char;
        let buf_size = buf.len();
        if buf_size > (linux_unsafe::int::MAX as usize) {
            return Err(result::EINVAL);
        }
        let result = unsafe { linux_unsafe::readlinkat(self.fd, path_raw, buf_ptr, buf_size) };
        match result {
            Ok(populated_size) => Ok(&buf[..populated_size as usize]),
            Err(e) => Err(e.into()),
        }
    }

    /// Use a `statx` system call to determine whether a particular path exists
    /// relative to the file, which must represent a directory.
    #[inline(always)]
    pub fn exists_relative(&self, path: &CStr) -> Result<bool> {
        let path_raw = path.as_ptr() as *const linux_unsafe::char;
        let mut tmp = unsafe { core::mem::zeroed::<linux_unsafe::statx>() };
        let result = unsafe { linux_unsafe::statx(self.fd, path_raw, 0, 0, &mut tmp as *mut _) };
        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.0 == linux_unsafe::result::ENOENT {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Change the current read/write position of the file.
    #[inline]
    pub fn seek(&self, pos: impl Into<SeekFrom>) -> Result<u64> {
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
    pub fn sync(&self) -> Result<()> {
        let result = unsafe { linux_unsafe::fsync(self.fd) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Write bytes from the given buffer to the file, returning how many bytes
    /// were written.
    #[inline(always)]
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
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
        &self,
        buf: *const linux_unsafe::void,
        count: linux_unsafe::size_t,
    ) -> Result<linux_unsafe::size_t> {
        let result = unsafe { linux_unsafe::write(self.fd, buf, count) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Safe wrapper for the `fcntl` system call.
    ///
    /// The safety of this wrapper relies on being passed only correct
    /// implementations of [`fcntl::FcntlCmd`], some of which are predefined
    /// as constants in the [`fcntl`] module.
    ///
    /// The type of the argument depends on which `cmd` you choose.
    #[inline]
    pub fn fcntl<'a, Cmd: fcntl::FcntlCmd<'a>>(
        &'a self,
        cmd: Cmd,
        arg: Cmd::ExtArg,
    ) -> Result<Cmd::Result> {
        let (raw_cmd, raw_arg) = cmd.prepare_fcntl_args(arg);
        let raw_result = unsafe { self.fcntl_raw(raw_cmd, raw_arg) };
        raw_result.map(|r| cmd.prepare_fcntl_result(r))
    }

    /// Direct wrapper around the raw `fcntl` system call.
    ///
    /// This system call is particularly unsafe because it interprets its
    /// last argument differently depending on the value of `cmd`.
    /// [`Self::fcntl`] provides a slightly safer abstraction around this
    /// operation.
    #[inline]
    pub unsafe fn fcntl_raw(
        &self,
        cmd: linux_unsafe::int,
        arg: impl AsRawV,
    ) -> Result<linux_unsafe::int> {
        let result = unsafe { linux_unsafe::fcntl(self.fd, cmd, arg) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Adds a device type parameter to the type of a file, allowing the
    /// [`Self::ioctl`] method to accept request constants that are compatible
    /// with that device type.
    ///
    /// **Safety:**
    /// - Caller must guarantee that the underlying file descriptor really
    ///   is representing a device of the given type, because the kernel
    ///   has some overloaded ioctl request numbers that have different meaning
    ///   depending on driver and using the wrong one can corrupt memory.
    pub unsafe fn to_device<T: ioctl::IoDevice>(
        self,
        #[allow(unused_variables)] devty: T,
    ) -> File<T> {
        let ret = File {
            fd: self.fd,
            _phantom: core::marker::PhantomData,
        };
        core::mem::forget(self); // don't call self's "drop" implementation
        ret
    }

    /// Direct wrapper around the raw `ioctl` system call.
    ///
    /// This system call is particularly unsafe because it interprets its
    /// last argument differently depending on the value of `request`.
    /// [`Self::ioctl`] provides a slightly safer abstraction around this
    /// operation.
    #[inline]
    pub unsafe fn ioctl_raw(
        &self,
        request: linux_unsafe::ulong,
        arg: impl AsRawV,
    ) -> Result<linux_unsafe::int> {
        let result = unsafe { linux_unsafe::ioctl(self.fd, request, arg) };
        result.map(|v| v as _).map_err(|e| e.into())
    }

    /// Bind an address to a socket.
    #[inline]
    pub fn bind(&self, addr: impl crate::socket::SockAddr) -> Result<()> {
        let (raw_ptr, raw_len) = unsafe { addr.sockaddr_raw_const() };
        unsafe { self.bind_raw(raw_ptr, raw_len) }
    }

    /// Bind an address to a socket using a raw pointer.
    #[inline]
    pub unsafe fn bind_raw(
        &self,
        addr: *const linux_unsafe::sockaddr,
        addrlen: linux_unsafe::socklen_t,
    ) -> Result<()> {
        let result = unsafe { linux_unsafe::bind(self.fd, addr, addrlen) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Initiate a connection on a socket.
    #[inline]
    pub fn connect(&self, addr: impl crate::socket::SockAddr) -> Result<()> {
        let (raw_ptr, raw_len) = unsafe { addr.sockaddr_raw_const() };
        unsafe { self.connect_raw(raw_ptr, raw_len) }
    }

    /// Initiate a connection on a socket using a raw pointer.
    #[inline]
    pub unsafe fn connect_raw(
        &self,
        addr: *const linux_unsafe::sockaddr,
        addrlen: linux_unsafe::socklen_t,
    ) -> Result<()> {
        let result = unsafe { linux_unsafe::connect(self.fd, addr, addrlen) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Listen for incoming connections on this socket.
    #[inline]
    pub fn listen(&self, backlog: linux_unsafe::int) -> Result<()> {
        let result = unsafe { linux_unsafe::listen(self.fd, backlog) };
        result.map(|_| ()).map_err(|e| e.into())
    }

    /// Get a socket option for a file descriptor representing a socket.
    ///
    /// The value for `opt` is typically a constant defined elsewhere in this
    /// crate, or possibly in another crate, which describes both the level
    /// and optname for the underlying call and the type of the result.
    #[inline(always)]
    pub fn getsockopt<'a, O: sockopt::GetSockOpt<'a>>(&self, opt: O) -> Result<O::Result> {
        let (level, optname) = opt.prepare_getsockopt_args();
        let mut buf: MaybeUninit<O::OptVal> = MaybeUninit::zeroed();
        let optlen = core::mem::size_of::<O::OptVal>() as linux_unsafe::socklen_t;
        let mut optlen_out = UnsafeCell::new(optlen);
        let result = unsafe {
            self.getsockopt_raw(
                level,
                optname,
                buf.as_mut_ptr() as *mut linux_unsafe::void,
                optlen_out.get(),
            )
        }?;
        if *optlen_out.get_mut() != optlen {
            // If the length isn't what we expected then we'll assume this
            // was an invalid GetSockOpt implementation.
            return Err(crate::result::Error::new(22)); // EINVAL
        }
        let buf = unsafe { buf.assume_init() };
        Ok(opt.prepare_getsockopt_result(result, buf))
    }

    /// Get a socket option for a file descriptor representing a socket using
    /// the raw arguments to the `getsockopt` system call.
    #[inline]
    pub unsafe fn getsockopt_raw(
        &self,
        level: linux_unsafe::int,
        optname: linux_unsafe::int,
        optval: *mut linux_unsafe::void,
        optlen: *mut linux_unsafe::socklen_t,
    ) -> Result<linux_unsafe::int> {
        let result = unsafe { linux_unsafe::getsockopt(self.fd, level, optname, optval, optlen) };
        result.map_err(|e| e.into())
    }

    /// Set a socket option for a file descriptor representing a socket.
    ///
    /// The value for `opt` is typically a constant defined elsewhere in this
    /// crate, or possibly in another crate, which describes both the level
    /// and optname for the underlying call and the type of the argument.
    #[inline(always)]
    pub fn setsockopt<'a, O: sockopt::SetSockOpt<'a>>(
        &self,
        opt: O,
        arg: O::ExtArg,
    ) -> Result<O::Result> {
        let (level, optname, optval, optlen) = opt.prepare_setsockopt_args(&arg);
        let result = unsafe {
            self.setsockopt_raw(level, optname, optval as *mut linux_unsafe::void, optlen)
        }?;
        Ok(opt.prepare_setsockopt_result(result))
    }

    /// Set a socket option for a file descriptor representing a socket using
    /// the raw arguments to the `setsockopt` system call.
    #[inline]
    pub unsafe fn setsockopt_raw(
        &self,
        level: linux_unsafe::int,
        optname: linux_unsafe::int,
        optval: *const linux_unsafe::void,
        optlen: linux_unsafe::socklen_t,
    ) -> Result<linux_unsafe::int> {
        let result = unsafe { linux_unsafe::setsockopt(self.fd, level, optname, optval, optlen) };
        result.map_err(|e| e.into())
    }

    /// Map the file into memory using the `mmap` system call.
    ///
    /// There is no safe wrapper for this because mapping a file into memory
    /// is inherently unsafe. Callers must take care to ensure they use the
    /// returned pointer in a safe way and to release the mapping with
    /// [`linux_unsafe::munmap`] when it's no longer needed.
    #[inline(always)]
    pub unsafe fn mmap_raw(
        &self,
        offset: linux_unsafe::off_t,
        length: linux_unsafe::size_t,
        addr: *mut void,
        prot: linux_unsafe::int,
        flags: linux_unsafe::int,
    ) -> Result<*mut void> {
        let result = unsafe { linux_unsafe::mmap(addr, length, prot, flags, self.fd, offset) };
        result.map_err(|e| e.into())
    }
}

/// Files that have been marked as representing a particular device type using
/// [`File::to_device`] can support `ioctl` requests that are designated for
/// that device.
impl<Device: ioctl::IoDevice> File<Device> {
    /// Safe wrapper for the `ioctl` system call.
    ///
    /// The safety of this wrapper relies on being passed only correct
    /// implementations of [`ioctl::IoctlReq`], some of which are predefined
    /// as constants elsewhere in this crate, while others will appear in
    /// device-specific support crates.
    ///
    /// The type of the argument depends on which `request` you choose.
    /// Some requests expect no argument, in which case you should pass
    /// `()`.
    #[inline]
    pub fn ioctl<'a, ReqDevice: ioctl::IoDevice, Req: ioctl::IoctlReq<'a, ReqDevice>>(
        &'a self,
        request: Req,
        arg: Req::ExtArg,
    ) -> Result<Req::Result>
    where
        Device: SubDevice<ReqDevice>,
    {
        // Some ioctl requests need some temporary memory space for the
        // kernel to write data into. It's the request implementation's
        // responsibility to initialize it if needed, but we'll at least
        // zero it so that any unused padding will start as zero.
        let mut temp_mem: MaybeUninit<Req::TempMem> = MaybeUninit::zeroed();
        let (raw_req, raw_arg) = request.prepare_ioctl_args(&arg, &mut temp_mem);
        let raw_result = unsafe { self.ioctl_raw(raw_req, raw_arg) };
        raw_result.map(|r| request.prepare_ioctl_result(r, &arg, &temp_mem))
    }
}

impl<Device> Drop for File<Device> {
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

impl<Device> core::fmt::Debug for File<Device> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File").field("fd", &self.fd).finish()
    }
}

/// [`File`] implements [`core::fmt::Write`] by passing UTF-8 encoded bytes
/// directly to the `write` method.
impl<T> core::fmt::Write for File<T> {
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
impl<Device> std::io::Read for File<Device> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Self::read(self, buf).map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl<Device> std::io::Write for File<Device> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Self::write(self, buf).map_err(|e| e.into())
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        Self::sync(self).map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl<Device> std::io::Seek for File<Device> {
    #[inline(always)]
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Self::seek(self, pos).map_err(|e| e.into())
    }
}

#[cfg(feature = "std")]
impl From<std::os::fd::OwnedFd> for File<()> {
    fn from(value: std::os::fd::OwnedFd) -> Self {
        use std::os::fd::IntoRawFd;

        Self {
            fd: value.into_raw_fd().into(),
            _phantom: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "std")]
impl<Device> std::os::fd::AsFd for File<Device> {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        unsafe { std::os::fd::BorrowedFd::borrow_raw(self.fd) }
    }
}

/// Use with [`File::open`] to open a file only for reading.
///
/// Use the methods of this type to add additional options for `open`.
pub const OPEN_READ_ONLY: OpenOptions<OpenWithoutMode> =
    OpenOptions::<OpenWithoutMode>::read_only();

/// Use with [`File::open`] to open a file only for writing.
///
/// Use the methods of this type to add additional options for `open`.
pub const OPEN_WRITE_ONLY: OpenOptions<OpenWithoutMode> =
    OpenOptions::<OpenWithoutMode>::write_only();

/// Use with [`File::open`] to open a file for both reading and writing.
///
/// Use the methods of this type to add additional options for `open`.
pub const OPEN_READ_WRITE: OpenOptions<OpenWithoutMode> =
    OpenOptions::<OpenWithoutMode>::read_write();

/// Encapsulates the various options for the `open` system call behind a
/// builder API.
///
/// Use [`OPEN_READ_ONLY`], [`OPEN_WRITE_ONLY`], or [`OPEN_READ_WRITE`] as
/// a starting value of this type and then refine as necessary using the
/// methods to set additional flags.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct OpenOptions<NeedMode: OpenMode> {
    flags: linux_unsafe::int,
    _phantom: core::marker::PhantomData<NeedMode>,
}

impl OpenOptions<OpenWithoutMode> {
    #[inline(always)]
    pub const fn read_only() -> Self {
        Self {
            flags: linux_unsafe::O_RDONLY, // NOTE: This is really just zero
            _phantom: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub const fn write_only() -> Self {
        Self {
            flags: linux_unsafe::O_WRONLY,
            _phantom: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub const fn read_write() -> Self {
        Self {
            flags: linux_unsafe::O_RDWR,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<NeedMode: OpenMode> OpenOptions<NeedMode> {
    #[inline(always)]
    const fn bit_or(self, new: linux_unsafe::int) -> Self {
        Self {
            flags: self.flags | new,
            _phantom: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub const fn append(self) -> Self {
        self.bit_or(linux_unsafe::O_APPEND)
    }

    #[inline(always)]
    pub const fn close_on_exec(self) -> Self {
        self.bit_or(linux_unsafe::O_CLOEXEC)
    }

    #[inline(always)]
    pub const fn create(self) -> OpenOptions<OpenWithMode> {
        OpenOptions {
            flags: self.bit_or(linux_unsafe::O_CREAT).flags,
            _phantom: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub const fn direct(self) -> Self {
        self.bit_or(linux_unsafe::O_DIRECT)
    }

    #[inline(always)]
    pub const fn directory(self) -> Self {
        self.bit_or(linux_unsafe::O_DIRECTORY)
    }

    #[inline(always)]
    pub const fn excl(self) -> Self {
        self.bit_or(linux_unsafe::O_EXCL)
    }

    #[inline(always)]
    pub const fn no_atime(self) -> Self {
        self.bit_or(linux_unsafe::O_NOATIME)
    }

    #[inline(always)]
    pub const fn no_controlling_tty(self) -> Self {
        self.bit_or(linux_unsafe::O_NOCTTY)
    }

    #[inline(always)]
    pub const fn no_follow_symlinks(self) -> Self {
        self.bit_or(linux_unsafe::O_NOFOLLOW)
    }

    #[inline(always)]
    pub const fn nonblocking(self) -> Self {
        self.bit_or(linux_unsafe::O_NONBLOCK)
    }

    #[inline(always)]
    pub const fn path_only(self) -> Self {
        self.bit_or(linux_unsafe::O_PATH)
    }

    #[inline(always)]
    pub const fn sync(self) -> Self {
        self.bit_or(linux_unsafe::O_SYNC)
    }

    #[inline(always)]
    pub const fn temp_file(self) -> OpenOptions<OpenWithMode> {
        OpenOptions {
            flags: self.bit_or(linux_unsafe::O_TMPFILE).flags,
            _phantom: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub const fn truncate(self) -> Self {
        self.bit_or(linux_unsafe::O_TRUNC)
    }

    /// Convert the options wrapper into the corresponding raw flags value
    /// to use with the `open` system call.
    #[inline(always)]
    pub const fn into_raw_flags(self) -> linux_unsafe::int {
        self.flags
    }
}

impl<NeedMode: OpenMode> Into<linux_unsafe::int> for OpenOptions<NeedMode> {
    #[inline(always)]
    fn into(self) -> linux_unsafe::int {
        self.into_raw_flags()
    }
}

/// A marker type used with [`OpenOptions`] to represent situations where
/// opening the file would require a `mode` argument.
pub enum OpenWithMode {}

/// A marker type used with [`OpenOptions`] to represent situations where
/// opening the file would require a `mode` argument.
pub enum OpenWithoutMode {}

/// A marker trait used with [`OpenOptions`] to represent whether a particular
/// set of options must be opened with an additional `mode` argument.
pub trait OpenMode {}

impl OpenMode for OpenWithMode {}
impl OpenMode for OpenWithoutMode {}
