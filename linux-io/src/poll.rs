use core::{cell::UnsafeCell, ptr::null};

use crate::result::{Error, Result};

/// A request for events related to a particular file descriptor in a call to
/// [`poll`].
#[repr(C)]
pub struct PollRequest<'a> {
    // Note: The layout of this struct must exactly match the kernel's
    // struct pollfd, because we'll be passing a pointer to an array of
    // instances of this struct directly to the kernel.
    fd: linux_unsafe::int,
    events: linux_unsafe::short,
    revents: UnsafeCell<linux_unsafe::short>, // Kernel will write in here
    _phantom: core::marker::PhantomData<&'a super::File>,
}

impl<'a> PollRequest<'a> {
    /// Begin constructing a [`PollRequest`] that describes events to request
    /// for the given file to use in a subsequent call to [`poll`].
    ///
    /// The the given file must outlive the returned poll request.
    #[inline]
    pub const fn new<'f: 'a>(file: &'f super::File) -> Self {
        let fd = file.fd;
        Self {
            fd,
            events: 0,
            revents: UnsafeCell::new(0),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Directly set the events bitmask for this request.
    ///
    /// Safety: The given bitmask must be a valid value for the `events`
    /// field of `struct pollfd` in the kernel's C API.
    #[inline(always)]
    pub unsafe fn events_raw(mut self, bitmask: core::ffi::c_short) -> Self {
        self.events = bitmask;
        self
    }

    /// Merge the given events bitmask with the existing event bits in the
    /// object using the bitwise OR operation.
    ///
    /// Safety: The given bitmask must be a valid value for the `events`
    /// field of `struct pollfd` in the kernel's C API.
    #[inline(always)]
    pub unsafe fn or_events_raw(mut self, bitmask: core::ffi::c_short) -> Self {
        self.events |= bitmask;
        self
    }

    /// Returns the result written by the latest call to [`poll`] that included
    /// this poll request object. If [`poll`] hasn't yet been called then the
    /// response indicates that no events have occurred.
    #[inline(always)]
    pub fn response(&self) -> PollResponse {
        PollResponse::new(unsafe { *self.revents.get() })
    }
}

/// Represents the bitmask of event flags produced for a [`PollRequest`] when
/// passed to [`poll`].
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PollResponse {
    revents: core::ffi::c_short,
}

impl PollResponse {
    #[inline(always)]
    const fn new(raw: core::ffi::c_short) -> Self {
        Self { revents: raw }
    }

    /// Returns the raw bitmask representing the events that the kernel
    /// reported. The meaning of this bitmask may be architecture-dependent.
    #[inline(always)]
    pub const fn raw_result(&self) -> core::ffi::c_short {
        self.revents
    }

    /// Returns true if the response indicates that there is data to read.
    #[inline(always)]
    pub const fn readable(&self) -> bool {
        (self.revents & linux_unsafe::POLLIN) != 0
    }

    /// Returns true if the response indicates that the file is writable.
    ///
    /// This only indicates that _some_ amount of writing is possible, but
    /// does not guarantee that a write of any given size will succeed.
    #[inline(always)]
    pub const fn writable(&self) -> bool {
        (self.revents & linux_unsafe::POLLOUT) != 0
    }

    /// Returns true if there is an error condition on the file descriptor.
    ///
    /// This condition is also used for the write end of a pipe once the read
    /// end has been closed.
    #[inline(always)]
    pub const fn error(&self) -> bool {
        (self.revents & linux_unsafe::POLLERR) != 0
    }

    /// Returns true if the other end of a channel has been closed.
    ///
    /// There might still be data in the read buffer, which can be read until
    /// reaching EOF.
    #[inline(always)]
    pub const fn hung_up(&self) -> bool {
        (self.revents & linux_unsafe::POLLERR) != 0
    }

    /// Returns true if there is an exceptional condition on the file descriptor.
    #[inline(always)]
    pub const fn exception(&self) -> bool {
        (self.revents & linux_unsafe::POLLPRI) != 0
    }

    /// Returns true if the kernel deemed the corresponding request to be invalid.
    #[inline(always)]
    pub const fn invalid(&self) -> bool {
        (self.revents & linux_unsafe::POLLNVAL) != 0
    }
}

/// `poll` wraps the Linux system call of the same name, or at least one
/// with a similar name, passing all of the given requests to the kernel.
///
/// *Warning:* This abstraction doesn't really work because the lifetime
/// bounds on `PollRequest` make the `File` objects unusable once the are
/// used here with a mutable borrow. We'll hopefully fix this in a future
/// release.
///
/// The kernel will modify the request objects in-place with updated event
/// flags, which you can then retrieve using [`PollRequest::response`].
/// The successful return value is the number of entries in `reqs` that now
/// have at least one response flag set.
///
/// The kernel may have an upper limit on the number of entries in reqs that
/// is smaller than the maximum value of `usize`. If the given slice is too
/// long then this function will return the EINVAL error code.
pub fn poll(reqs: &mut [PollRequest], timeout: linux_unsafe::int) -> Result<linux_unsafe::int> {
    // NOTE: We're effectively transmuting our PollRequest type into
    // the kernel's struct pollfd here. This is safe because the layout
    // of our struct should exactly match the kernel's, and the kernel
    // will only write into our "revents" field, and any bits written
    // in there are valid for type "short".
    let reqs_ptr = reqs.as_ptr() as *mut linux_unsafe::pollfd;
    if reqs.len() > (!(0 as linux_unsafe::nfds_t)) as usize {
        // More file descriptors than the kernel can physicall support on this
        // platform, so we'll return a synthetic EINVAL to mimic how the
        // kernel would behave if it had a smaller soft limit.
        return Err(Error::new(22)); // hard-coded EINVAL value (TODO: expose this as a constant from linux-unsafe instead?)
    }
    let reqs_count = reqs.len() as linux_unsafe::nfds_t;
    // We actually use ppoll rather than poll, because poll is not
    // available on recently-added architectures like riscv64.
    let tmo = linux_unsafe::timespec {
        tv_sec: (timeout / 1000) as linux_unsafe::long,
        tv_nsec: ((timeout % 1000) * 1_000_000) as linux_unsafe::long,
    };
    let tmo_p = &tmo as *const _;
    let result = unsafe { linux_unsafe::ppoll(reqs_ptr, reqs_count, tmo_p, null()) };
    result.map(|count| count as _).map_err(|e| e.into())
}
