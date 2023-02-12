/// Address types for the IPv4 and IPv6 protocol families.
pub mod ip;

pub use linux_unsafe::sock_type;

/// A trait implemented by all socket address types.
///
/// **Safety:**
/// - Implementers must ensure that the two raw methods always return
/// valid pointers and lengths for the kernel to refer to. The pointers should
/// always be to a sockaddr-shaped structure, which always starts with
/// an `sa_family_t` field describing the protocol family that the address
/// belongs to.
/// - Both methods must return a pointer to the same memory so that writes
/// through the mut pointer will be visible to reads through the const pointer.
pub unsafe trait SockAddr {
    /// Returns a raw const pointer and the length of what it points to for
    /// use when sending a socket address to the kernel.
    ///
    /// **Safety:** Caller must ensure that `self` remains valid throughout
    /// all use of the returned pointer and that use of it is consistent
    /// with a shared borrow.
    unsafe fn sockaddr_raw_const(&self) -> (*const linux_unsafe::void, linux_unsafe::socklen_t);

    /// Returns a raw mut pointer and the length of what it points to for
    /// use when retrieving a socket address from the kernel.
    ///
    /// **Safety:** Caller must ensure that `self` remains valid throughout
    /// all use of the returned pointer and that use of it is consistent
    /// with a mutable borrow.
    unsafe fn sockaddr_raw_mut(&mut self) -> (*mut linux_unsafe::void, linux_unsafe::socklen_t);
}
