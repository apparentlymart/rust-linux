/// Device type marker for [`crate::File`] instances that represent TCP sockets.
#[derive(Clone, Copy)]
pub struct TcpSocketDevice;

impl crate::fd::ioctl::IoDevice for TcpSocketDevice {}
unsafe impl crate::fd::ioctl::SubDevice<super::Ipv4SocketDevice> for TcpSocketDevice {}
unsafe impl crate::fd::ioctl::SubDevice<super::Ipv6SocketDevice> for TcpSocketDevice {}
unsafe impl crate::fd::ioctl::SubDevice<super::super::SocketDevice> for TcpSocketDevice {}

use crate::fd::ioctl::{ioctl_read, IoctlReqRead};
use linux_unsafe::int;

/// Returns the amount of queued unread data in the receive buffer.
///
/// The socket must not be in listen state, otherwise an error (`EINVAL`) is
/// returned.
pub const SIOCINQ: IoctlReqRead<TcpSocketDevice, int> = unsafe { ioctl_read(0x541B) };

/// Returns true (i.e., value is nonzero) if the inbound data stream is at the
/// urgent mark.
pub const SIOCATMARK: IoctlReqRead<TcpSocketDevice, int> = unsafe { ioctl_read(0x8905) };

/// Returns the amount of unsent data in the socket send queue.
///
/// The socket must not be in listen state, otherwise an error (`EINVAL`) is
/// returned.
pub const SIOCOUTQ: IoctlReqRead<TcpSocketDevice, int> = unsafe { ioctl_read(0x5411) };
