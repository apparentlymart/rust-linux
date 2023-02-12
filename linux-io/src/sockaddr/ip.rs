/// Socket address type for the IPv4 protocol family.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SockAddrIpv4 {
    sin_family: linux_unsafe::sa_family_t,
    sin_port: u16, // (but in network byte order)
    sin_addr: Ipv4Addr,
}

impl SockAddrIpv4 {
    /// Create a new [`SockAddrIpv4`] with the specified IP address and port
    /// number.
    ///
    /// Port number should be provided in the host's native byte order. This
    /// function will convert it to network byte order where necessary.
    #[inline]
    pub const fn new(host_addr: Ipv4Addr, port: u16) -> Self {
        Self {
            sin_family: AF_INET,
            sin_port: port.to_be(),
            sin_addr: host_addr,
        }
    }

    /// Returns the host address part of the socket address.
    #[inline(always)]
    pub const fn host_address(&self) -> Ipv4Addr {
        self.sin_addr
    }

    /// Returns the port number in host (_not_ network) byte order.
    #[inline(always)]
    pub const fn port(&self) -> u16 {
        self.sin_port.to_be() // Swaps the bytes if we're running on a little-endian system
    }
}

/// Representation of an IPv4 host address.
///
/// Note that this isn't an IPv4 _socket address_ type; use [`SockAddrIpv4`]
/// to represent both the host address and port number for an IPv4 socket.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ipv4Addr {
    s_addr: u32, // (but in network byte order)
}

impl Ipv4Addr {
    /// Equivalent to the constant `INADDR_ANY` in C.
    pub const ANY: Self = Self { s_addr: 0x00000000 };

    /// Equivalent to the constant `INADDR_NONE` in C.
    pub const NONE: Self = Self { s_addr: 0xffffffff };

    /// Equivalent to the constant `INADDR_BROADCAST` in C.
    pub const BROADCAST: Self = Self { s_addr: 0xffffffff };

    /// Equivalent to the constant `INADDR_DUMMY` in C.
    pub const DUMMY: Self = Self {
        s_addr: 0xc0000008_u32.to_be(),
    };

    /// Equivalent to the constant `INADDR_LOOPBACK` in C.
    pub const LOOPBACK: Self = Self {
        s_addr: 0x7f000001_u32.to_be(),
    };

    /// Equivalent to the constant `INADDR_UNSPEC_GROUP` in C.
    pub const UNSPEC_GROUP: Self = Self {
        s_addr: 0xe0000000_u32.to_be(),
    };
    /// Equivalent to the constant `INADDR_ALLHOSTS_GROUP` in C.
    pub const ALLHOSTS_GROUP: Self = Self {
        s_addr: 0xe0000001_u32.to_be(),
    };
    /// Equivalent to the constant `INADDR_ALLRTRS_GROUP` in C.
    pub const ALLRTRS_GROUP: Self = Self {
        s_addr: 0xe0000002_u32.to_be(),
    };
    /// Equivalent to the constant `INADDR_ALLSNOOPERS_GROUP` in C.
    pub const ALLSNOOPERS_GROUP: Self = Self {
        s_addr: 0xe000006a_u32.to_be(),
    };

    /// Constructs an Ipv4Addr directly from a u32 value written in the
    /// host byte order.
    ///
    /// For example, the standard loopback address `127.0.0.1` should be
    /// provided as `0x7f000001` on all platforms, which would be encoded as
    /// `[0x01, 0x00, 0x00, 0x7f]` on a little-endian system but this
    /// function will then convert it to network byte order automatically.
    #[inline(always)]
    pub const fn from_u32(raw: u32) -> Self {
        Self {
            s_addr: raw.to_be(),
        }
    }

    /// Returns the raw u32 value of the address in host (_not_ network) byte order.
    #[inline(always)]
    pub const fn as_u32(&self) -> u32 {
        self.s_addr.to_be() // undoes the to_be we did on construction if we're on a little-endian system
    }
}

/// Represents the IPv4 address family.
pub const AF_INET: linux_unsafe::sa_family_t = 2;

pub const IPPROTO_ICMP: linux_unsafe::int = 1;
pub const IPPROTO_IGMP: linux_unsafe::int = 4;
pub const IPPROTO_TCP: linux_unsafe::int = 6;
pub const IPPROTO_EGP: linux_unsafe::int = 8;
pub const IPPROTO_PUP: linux_unsafe::int = 12;
pub const IPPROTO_UDP: linux_unsafe::int = 17;
pub const IPPROTO_IDP: linux_unsafe::int = 22;
pub const IPPROTO_TP: linux_unsafe::int = 29;
pub const IPPROTO_DCCP: linux_unsafe::int = 33;
pub const IPPROTO_IPV6: linux_unsafe::int = 41;
pub const IPPROTO_RSVP: linux_unsafe::int = 46;
pub const IPPROTO_GRE: linux_unsafe::int = 47;
pub const IPPROTO_ESP: linux_unsafe::int = 50;
pub const IPPROTO_AH: linux_unsafe::int = 51;
pub const IPPROTO_MTP: linux_unsafe::int = 92;
pub const IPPROTO_ENCAP: linux_unsafe::int = 98;
pub const IPPROTO_PIM: linux_unsafe::int = 103;
pub const IPPROTO_COMP: linux_unsafe::int = 108;
pub const IPPROTO_L2TP: linux_unsafe::int = 115;
pub const IPPROTO_SCTP: linux_unsafe::int = 132;
pub const IPPROTO_UDPLITE: linux_unsafe::int = 136;
pub const IPPROTO_MPLS: linux_unsafe::int = 137;
pub const IPPROTO_ETHERNET: linux_unsafe::int = 143;
pub const IPPROTO_RAW: linux_unsafe::int = 255;
pub const IPPROTO_MPTCP: linux_unsafe::int = 262;

unsafe impl super::SockAddr for SockAddrIpv4 {
    #[inline(always)]
    unsafe fn sockaddr_raw_const(&self) -> (*const linux_unsafe::void, linux_unsafe::socklen_t) {
        (
            self as *const Self as *const _,
            core::mem::size_of::<Self>() as linux_unsafe::socklen_t,
        )
    }

    #[inline(always)]
    unsafe fn sockaddr_raw_mut(&mut self) -> (*mut linux_unsafe::void, linux_unsafe::socklen_t) {
        (
            self as *mut Self as *mut _,
            core::mem::size_of::<Self>() as linux_unsafe::socklen_t,
        )
    }
}
