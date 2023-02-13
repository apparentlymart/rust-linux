/// Socket address type for the IPv4 protocol family.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(8))]
pub struct SockAddrIpv4 {
    sin_family: linux_unsafe::sa_family_t,
    sin_port: u16, // (but in network byte order)
    sin_addr: Ipv4Addr,

    // The kernel expects sockaddr_in to be 16 bytes long, even though
    // only the first eight bytes are actually useful. We always set these
    // padding bytes to zero.
    sin_zero: [u8; 8],
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
            sin_zero: [0; 8],
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
#[derive(Clone, Copy, Debug)]
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

    /// Constructs an [`Ipv4Addr`] directly from a u32 value written in the
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

    /// Constructs an [`Ipv4Addr`] from the given octets which are interpreted
    /// in network byte order, meaning that the first element corresponds with
    /// the first decimal digit in the conventional four-segment dotted decimal
    /// IP address representation.
    #[inline(always)]
    pub const fn from_octets(raw: [u8; 4]) -> Self {
        // Note: we're first interpreting raw into the _host_ byte order, and
        // then from_u32 will reinterpret that into the network byte order.
        Self::from_u32(u32::from_be_bytes(raw))
    }

    /// Returns the raw u32 value of the address in host (_not_ network) byte order.
    #[inline(always)]
    pub const fn as_u32(&self) -> u32 {
        self.s_addr.to_be() // undoes the to_be we did on construction if we're on a little-endian system
    }

    /// Returns the raw octets of the address in network byte order.
    #[inline(always)]
    pub const fn as_octets(&self) -> [u8; 4] {
        self.as_u32().to_be_bytes()
    }

    /// Returns the same IP address in the "IPv6 mapped" form.
    pub const fn to_ipv6_mapped(&self) -> Ipv6Addr {
        let our_octets = self.as_octets();
        let mut new_octets = [0_u8; 16];
        new_octets[10] = 0xff;
        new_octets[11] = 0xff;
        new_octets[12] = our_octets[0];
        new_octets[13] = our_octets[1];
        new_octets[14] = our_octets[2];
        new_octets[15] = our_octets[3];
        Ipv6Addr::from_octets(new_octets)
    }
}

/// Socket address type for the IPv6 protocol family.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(8))]
pub struct SockAddrIpv6 {
    sin6_family: linux_unsafe::sa_family_t,
    sin6_port: u16, // (but in network byte order)
    sin6_flowinfo: u32,
    sin6_addr: Ipv6Addr,
    sin6_scope_id: u32,
}

impl SockAddrIpv6 {
    /// Create a new [`SockAddrIpv6`] with the specified IP address and port
    /// number.
    ///
    /// Port number should be provided in the host's native byte order. This
    /// function will convert it to network byte order where necessary.
    #[inline]
    pub const fn new(host_addr: Ipv6Addr, port: u16) -> Self {
        Self {
            sin6_family: AF_INET6,
            sin6_port: port.to_be(),
            sin6_addr: host_addr,

            sin6_flowinfo: 0,
            sin6_scope_id: 0,
        }
    }

    /// Create a new [`SockAddrIpv6`] with the specified IP address, port
    /// number, and scope id.
    ///
    /// Port number should be provided in the host's native byte order. This
    /// function will convert it to network byte order where necessary.
    #[inline]
    pub const fn new_with_scope(host_addr: Ipv6Addr, port: u16, scope_id: u32) -> Self {
        Self {
            sin6_family: AF_INET6,
            sin6_port: port.to_be(),
            sin6_addr: host_addr,

            sin6_flowinfo: 0,
            sin6_scope_id: scope_id,
        }
    }

    /// Returns the host address part of the socket address.
    #[inline(always)]
    pub const fn host_address(&self) -> Ipv6Addr {
        self.sin6_addr
    }

    /// Returns the port number in host (_not_ network) byte order.
    #[inline(always)]
    pub const fn port(&self) -> u16 {
        self.sin6_port.to_be() // Swaps the bytes if we're running on a little-endian system
    }

    /// Returns the scope ID of the address.
    #[inline(always)]
    pub const fn scope_id(&self) -> u32 {
        self.sin6_scope_id
    }
}

/// Representation of an IPv6 host address.
///
/// Note that this isn't an IPv6 _socket address_ type; use [`SockAddrIpv6`]
/// to represent both the host address and port number for an IPv4 socket.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Ipv6Addr {
    s6_addr: [u8; 16],
}

impl Ipv6Addr {
    /// Equivalent to the constant `IN6ADDR_ANY_INIT` in C.
    pub const ANY: Self = Self {
        s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    /// Equivalent to the constant `IN6ADDR_LOOPBACK_INIT` in C.
    pub const LOOPBACK: Self = Self {
        s6_addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    };

    /// Equivalent to the constant `IN6ADDR_LINKLOCAL_ALLNODES_INIT` in C.
    pub const LINKLOCAL_ALLNODES: Self = Self {
        s6_addr: [0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    };

    /// Equivalent to the constant `IN6ADDR_LINKLOCAL_ALLROUTERS_INIT` in C.
    pub const LINKLOCAL_ALLROUTERS: Self = Self {
        s6_addr: [0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    };

    /// Equivalent to the constant `IN6ADDR_INTERFACELOCAL_ALLNODES_INIT` in C.
    pub const INTERFACELOCAL_ALLNODES: Self = Self {
        s6_addr: [0xff, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    };

    /// Equivalent to the constant `IN6ADDR_INTERFACELOCAL_ALLROUTERS_INIT` in C.
    pub const INTERFACELOCAL_ALLROUTERS: Self = Self {
        s6_addr: [0xff, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    };

    /// Equivalent to the constant `IN6ADDR_SITELOCAL_ALLROUTERS_INIT` in C.
    pub const SITELOCAL_ALLROUTERS: Self = Self {
        s6_addr: [0xff, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    };

    /// Constructs an [`Ipv6Addr`] from the given octets which are interpreted
    /// in network byte order, meaning that the first element corresponds with
    /// the first decimal digit in the conventional four-segment dotted decimal
    /// IP address representation.
    #[inline(always)]
    pub const fn from_octets(raw: [u8; 16]) -> Self {
        // Note: we're first interpreting raw into the _host_ byte order, and
        // then from_u32 will reinterpret that into the network byte order.
        Self { s6_addr: raw }
    }

    /// Returns the raw octets of the address in network byte order.
    #[inline(always)]
    pub const fn as_octets(&self) -> [u8; 16] {
        self.s6_addr
    }
}

/// Represents a socket address that can be for either an IPv4 socket or an
/// IPv6 socket, chosen dynamically at runtime.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SockAddrIp(SockAddrIpInner);

#[derive(Clone, Copy)]
#[repr(C, align(8))]
union SockAddrIpInner {
    // A struct that covers only the "family" field that both real address
    // types have in common. We use this only to retrieve the tag before
    // deciding which of the other two fields is the active one.
    jt: SockAddrJustTag,

    // Every `SockAddrIpInner` has one of these, with the "family" field
    // of both serving as the discriminator.
    v4: SockAddrIpv4,
    v6: SockAddrIpv6,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, align(8))]
struct SockAddrJustTag {
    family: linux_unsafe::sa_family_t,
}

impl SockAddrIp {
    pub fn new(host_address: impl Into<IpAddr>, port: u16) -> Self {
        let host_address = host_address.into();
        match host_address {
            IpAddr::V4(addr) => Self(SockAddrIpInner {
                v4: SockAddrIpv4::new(addr, port),
            }),
            IpAddr::V6(addr) => Self(SockAddrIpInner {
                v6: SockAddrIpv6::new(addr, port),
            }),
        }
    }

    /// Get the address family that this address is for.
    ///
    /// The result is either [`AF_INET`] or [`AF_INET6`].
    pub const fn address_family(&self) -> linux_unsafe::sa_family_t {
        // Safe because `new` populates either v4 or v6, both of which
        // have the family field as the first field with compatible
        // storage and both always initialize it to the appropriate constant.
        unsafe { self.0.jt.family }
    }

    /// Get the IP address associated with the socket address.
    ///
    /// The result can be either an IPv4 or IPv6 address depending on the
    /// address family of the socket address.
    pub const fn host_address(&self) -> IpAddr {
        match self.address_family() {
            AF_INET => IpAddr::V4(unsafe { self.0.v4 }.host_address()),
            AF_INET6 => IpAddr::V6(unsafe { self.0.v6 }.host_address()),
            _ => unreachable!(),
        }
    }

    /// Returns the port number in host (_not_ network) byte order.
    #[inline(always)]
    pub const fn port(&self) -> u16 {
        match self.address_family() {
            AF_INET => unsafe { self.0.v4 }.port(),
            AF_INET6 => unsafe { self.0.v6 }.port(),
            _ => unreachable!(),
        }
    }
}

impl core::fmt::Debug for SockAddrIp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.address_family() {
            AF_INET => unsafe { self.0.v4 }.fmt(f),
            AF_INET6 => unsafe { self.0.v6 }.fmt(f),
            _ => unreachable!(),
        }
    }
}

/// Represents a host address that can be either an IPv4 address or an IPv6
/// address chosen dynamically at runtime.
#[derive(Clone, Copy, Debug)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    /// Returns the equivalent IPv6 address, either directly (when the source
    /// is already IPv6) or as an IPv4-mapped-in-IPv6 address.
    ///
    /// The Linux IPv6 implementation can support IPv4 too when using a
    /// mapped address, so using this method can allow the rest of the program
    /// to use `AF_INET6` exclusively, if desired.
    pub const fn to_ipv6_mapped(self) -> Ipv6Addr {
        match self {
            IpAddr::V4(addr) => addr.to_ipv6_mapped(),
            IpAddr::V6(addr) => addr,
        }
    }
}

/// Represents the IPv4 address family.
pub const AF_INET: linux_unsafe::sa_family_t = 2;

/// Represents the IPv6 address family.
pub const AF_INET6: linux_unsafe::sa_family_t = 10;

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

unsafe impl super::SockAddr for SockAddrIpv6 {
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

unsafe impl super::SockAddr for SockAddrIp {
    #[inline(always)]
    unsafe fn sockaddr_raw_const(&self) -> (*const linux_unsafe::void, linux_unsafe::socklen_t) {
        // Our inner is a union over the possible address types, and we can use
        // the required "family" field as the tag for our union without needing
        // any additional storage.
        match self.address_family() {
            AF_INET => self.0.v4.sockaddr_raw_const(),
            AF_INET6 => self.0.v6.sockaddr_raw_const(),
            _ => unreachable!(), // af field is not publicly exposed so cannot have any other value
        }
    }

    #[inline(always)]
    unsafe fn sockaddr_raw_mut(&mut self) -> (*mut linux_unsafe::void, linux_unsafe::socklen_t) {
        // Our inner is a union over the possible address types, and we can use
        // the required "family" field as the tag for our union without needing
        // any additional storage.
        match self.address_family() {
            AF_INET => self.0.v4.sockaddr_raw_mut(),
            AF_INET6 => self.0.v6.sockaddr_raw_mut(),
            _ => unreachable!(), // af field is not publicly exposed so cannot have any other value
        }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(value: Ipv4Addr) -> Self {
        IpAddr::V4(value)
    }
}

impl From<Ipv6Addr> for IpAddr {
    fn from(value: Ipv6Addr) -> Self {
        IpAddr::V6(value)
    }
}

#[cfg(feature = "std")]
extern crate std;

/// Additional functions available when the `std` feature is active, for
/// integrating with the standard library.
#[cfg(feature = "std")]
impl Ipv4Addr {
    /// Converts the standard library's representation of IPv4 addresses into
    /// our representation.
    #[inline]
    pub const fn from_std(addr: std::net::Ipv4Addr) -> Self {
        Self::from_octets(addr.octets())
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv4Addr> for Ipv4Addr {
    fn from(value: std::net::Ipv4Addr) -> Self {
        Self::from_std(value)
    }
}

/// Additional functions available when the `std` feature is active, for
/// integrating with the standard library.
#[cfg(feature = "std")]
impl Ipv6Addr {
    /// Converts the standard library's representation of IPv4 addresses into
    /// our representation.
    #[inline]
    pub const fn from_std(addr: std::net::Ipv6Addr) -> Self {
        Self::from_octets(addr.octets())
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv6Addr> for Ipv6Addr {
    fn from(value: std::net::Ipv6Addr) -> Self {
        Self::from_std(value)
    }
}

/// Additional functions available when the `std` feature is active, for
/// integrating with the standard library.
#[cfg(feature = "std")]
impl IpAddr {
    /// Converts the standard library's representation of IPv4 addresses into
    /// our representation.
    #[inline]
    pub const fn from_std(addr: std::net::IpAddr) -> Self {
        match addr {
            std::net::IpAddr::V4(addr) => Self::V4(Ipv4Addr::from_octets(addr.octets())),
            std::net::IpAddr::V6(addr) => Self::V6(Ipv6Addr::from_octets(addr.octets())),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::net::IpAddr> for IpAddr {
    fn from(value: std::net::IpAddr) -> Self {
        Self::from_std(value)
    }
}
