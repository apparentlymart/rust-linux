/// Address types for the IPv4 and IPv6 protocol families.
pub mod ip;

use core::mem::size_of;

pub use linux_unsafe::sock_type;

use crate::fd::ioctl::{ioctl_read, IoctlReqRead, _IOR};

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

/// Represents a socket protocol that is compatible with sockets belonging to
/// the domain/family `FAMILY`.
///
/// This trait allows [`super::File::socket`] to return a file of the
/// appropriate device type for the selected protocol so that the relevant
/// socket `ioctl` requests will be supported on its result.
pub trait SocketProtocol {
    type Device: crate::fd::ioctl::IoDevice;

    fn raw_protocol_num(&self) -> linux_unsafe::int;
}

/// Builds a reasonable default implementation of [`SocketProtocol`] with
/// a fixed protocol number and device type.
///
/// **Safety:** Caller must ensure that the specified device marker type
/// is suitable for the type of socket the kernel will return when requesting
/// this protocol. This works in conjunction with the safety rules for
/// implementing ioctl requests on device types; the designated device should
/// only have ioctl request constants that can be called against a file
/// descriptor of this protocol without the risk of memory corruption caused
/// by an incorrect argument type.
#[inline(always)]
pub const unsafe fn socket_protocol<Device: crate::fd::ioctl::IoDevice>(
    num: linux_unsafe::int,
) -> SocketProtocolFixed<Device> {
    SocketProtocolFixed::numbered(num)
}

/// A reasonable default implementation of [`SocketProtocol`] with a fixed
/// protocol number and device type.
///
/// This is the return type of [`socket_protocol`].
#[repr(transparent)]
pub struct SocketProtocolFixed<Device: crate::fd::ioctl::IoDevice> {
    num: linux_unsafe::int,
    _phantom: core::marker::PhantomData<Device>,
}

/// A convenience implementation of [`SocketProtocol`] for simple protocols
/// that belong to only a single family and have a fixed protocol number.
impl<Device: crate::fd::ioctl::IoDevice> SocketProtocolFixed<Device> {
    #[inline(always)]
    pub(crate) const unsafe fn numbered(num: linux_unsafe::int) -> Self {
        // The value of a `SocketProtocolFixed` is always just its protocol
        // number directly, making it have identical layout to the raw
        // protocol argument on the `socket` system call.
        Self {
            num,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<Device: crate::fd::ioctl::IoDevice> SocketProtocol for SocketProtocolFixed<Device> {
    type Device = Device;

    #[inline(always)]
    fn raw_protocol_num(&self) -> linux_unsafe::int {
        self.num
    }
}

/// Device type marker for [`crate::File`] instances that represent sockets.
///
/// In practice there should not typically be a `File<SocketDevice>` directly,
/// but instead should use a protocol-specific device type that also has a
/// blanket impl to make all of the `SocketDevice` ioctl requests available
/// too.
pub struct SocketDevice;

impl crate::fd::ioctl::IoDevice for SocketDevice {}

const SOCK_IOC_TYPE: linux_unsafe::ulong = 0x89;

/// `ioctl` request to retrieve a `struct timeval` with the receive timestamp
/// of the last packet passed to the user.
pub const SIOCGSTAMP: IoctlReqRead<SocketDevice, linux_unsafe::timeval> = unsafe {
    ioctl_read(_IOR(
        SOCK_IOC_TYPE,
        0x06,
        // This size is expressed in the kernel as sizeof(long long[2]), rather
        // than as "struct timeval".
        (size_of::<core::ffi::c_longlong>() * 2) as linux_unsafe::ulong,
    ))
};
