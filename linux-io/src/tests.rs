extern crate std;
use core::ffi::CStr;
use std::vec::Vec;
use std::{os::unix::prelude::OsStrExt, path::PathBuf};

use fd::{DirEntry, DirEntryType};
use tempfile::tempdir;

use super::*;
use std::ffi::CString;

#[test]
fn create_write_close_open_read_close() {
    use std::io::{Read, Write};
    use std::println;

    let message = b"Hello!\n";

    let dir = tempdir().unwrap();
    let mut filename: PathBuf = dir.path().into();
    filename.push("test.txt");
    println!("temporary file is {:?}", filename);
    // This crate is only for Linux systems, so it's safe to assume that
    // an OsStr is raw filename bytes as the kernel will expect.
    let filename_raw = CString::new(filename.as_os_str().as_bytes()).unwrap();

    let mut f = File::create_raw(&filename_raw, 0o666)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create file");
    f.write_all(message).expect("failed to write to file");
    f.close()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to close file");

    let mut f = File::open_raw(&filename_raw, linux_unsafe::O_RDONLY, 0)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to reopen file");
    let mut v: Vec<u8> = Vec::new();
    f.read_to_end(&mut v).expect("failed to read from file");
    f.close()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to close file the second time");

    dir.close().expect("failed to clean temporary directory");

    assert_eq!(v.as_slice(), message, "wrong file contents");
}

#[test]
fn dup() {
    use std::io::{Read, Write};

    let dir = tempdir().unwrap();
    let mut filename: PathBuf = dir.path().into();
    filename.push("test.txt");
    use std::println;
    println!("temporary file is {:?}", filename);

    // This crate is only for Linux systems, so it's safe to assume that
    // an OsStr is raw filename bytes as the kernel will expect.
    let filename_raw = CString::new(filename.as_os_str().as_bytes()).unwrap();

    let f = File::create_raw(&filename_raw, 0o666)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create file");

    let mut f2 = f
        .duplicate()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to duplicate file");

    let message = b"Hello!\n";
    f2.write_all(message).expect("failed to write to file");
    f2.close()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to close dup file");
    f.close()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to close original file");

    let mut f = File::open_raw(&filename_raw, linux_unsafe::O_RDONLY, 0)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to reopen file");
    let mut v: Vec<u8> = Vec::new();
    f.read_to_end(&mut v).expect("failed to read from file");
    f.close()
        .map_err(|e| e.into_std_io_error())
        .expect("failed to close file the second time");

    dir.close().expect("failed to clean temporary directory");

    assert_eq!(v.as_slice(), message, "wrong file contents");
}

#[test]
fn fcntl_dup() {
    let dir = tempdir().unwrap();
    let mut filename: PathBuf = dir.path().into();
    filename.push("test.txt");
    use std::println;
    println!("temporary file is {:?}", filename);

    // This crate is only for Linux systems, so it's safe to assume that
    // an OsStr is raw filename bytes as the kernel will expect.
    let filename_raw = CString::new(filename.as_os_str().as_bytes()).unwrap();

    let f = File::create_raw(&filename_raw, 0o666)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create file");

    let f2 = f
        .fcntl(crate::fd::fcntl::F_DUPFD, 0)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create file");
    drop(f2); // close the dup
    drop(f); // close the original

    dir.close().expect("failed to clean temporary directory");
}

#[test]
fn getdents() {
    let dirf = File::open(c"testdata/simpledir", OpenOptions::read_only().directory()).unwrap();

    let mut buf = std::vec![0_u8; 256];
    let mut got: Vec<DirEntry> = dirf.getdents(&mut buf).unwrap().collect();
    got.sort_unstable_by_key(|e| e.name);
    for entry in got.iter_mut() {
        // inode and offset are implementation-dependent, so
        // we'll just fix them and focus on testing the others.
        entry.ino = 0xfeedface;
        entry.off = 0xbeeebeee;
    }
    let want = std::vec![
        DirEntry {
            ino: 0xfeedface,
            off: 0xbeeebeee,
            entry_type: DirEntryType::Dir,
            name: c".",
        },
        DirEntry {
            ino: 0xfeedface,
            off: 0xbeeebeee,
            entry_type: DirEntryType::Dir,
            name: c"..",
        },
        DirEntry {
            ino: 0xfeedface,
            off: 0xbeeebeee,
            entry_type: DirEntryType::Reg,
            name: c"bar",
        },
        DirEntry {
            ino: 0xfeedface,
            off: 0xbeeebeee,
            entry_type: DirEntryType::Dir,
            name: c"foo",
        },
    ];

    assert_eq!(got, want);
}

#[test]
fn getdents_all() {
    let dirf = File::open(c"testdata/simpledir", OpenOptions::read_only().directory()).unwrap();

    #[derive(Debug, PartialEq, Eq)]
    struct Item {
        entry_type: DirEntryType,
        name: CString,
    }
    let mut buf = std::vec![0_u8; 28];
    let mut got: Vec<_> = dirf
        .getdents_all(&mut buf, |entry| Item {
            entry_type: entry.entry_type,
            name: entry.name.into(),
        })
        .map(|result| result.unwrap())
        .collect();
    got.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    let want = std::vec![
        Item {
            entry_type: DirEntryType::Dir,
            name: c".".into(),
        },
        Item {
            entry_type: DirEntryType::Dir,
            name: c"..".into(),
        },
        Item {
            entry_type: DirEntryType::Reg,
            name: c"bar".into(),
        },
        Item {
            entry_type: DirEntryType::Dir,
            name: c"foo".into(),
        },
    ];

    assert_eq!(got, want);
}

#[test]
fn getdents_all_fnmut() {
    // This is a rather esotaric use of `getdents_all` which accumulates
    // the entry names all into a single Vec and then handles them as
    // immutable CStr views into that Vec. The main point of this is just
    // to make sure getdents_all allows the transform function to mutate
    // symbols that it captures.

    let dirf = File::open(c"testdata/simpledir", OpenOptions::read_only().directory()).unwrap();

    #[derive(Debug, PartialEq, Eq)]
    struct Item<'a> {
        entry_type: DirEntryType,
        name: &'a CStr,
    }

    let mut buf = std::vec![0_u8; 28];
    let mut names_buf = Vec::<u8>::new();
    let tmp: Vec<_> = dirf
        .getdents_all(&mut buf, |entry| {
            let start_idx = names_buf.len();
            names_buf.extend(entry.name.to_bytes_with_nul());
            (entry.entry_type, start_idx, names_buf.len())
        })
        .map(|result| result.unwrap())
        .collect();
    let mut got: Vec<_> = tmp
        .iter()
        .map(|(entry_type, start_idx, end_idx)| {
            let name_bytes = &names_buf[*start_idx..*end_idx];
            Item {
                entry_type: *entry_type,
                name: CStr::from_bytes_with_nul(name_bytes).unwrap(),
            }
        })
        .collect();
    got.sort_unstable_by_key(|e| e.name);

    let want = std::vec![
        Item {
            entry_type: DirEntryType::Dir,
            name: c".",
        },
        Item {
            entry_type: DirEntryType::Dir,
            name: c"..",
        },
        Item {
            entry_type: DirEntryType::Reg,
            name: c"bar",
        },
        Item {
            entry_type: DirEntryType::Dir,
            name: c"foo",
        },
    ];

    assert_eq!(got, want);
}

#[test]
fn socket_ipv4_bind_tcp() {
    use crate::socket;
    use std::println;

    let f = File::socket(
        socket::ip::AF_INET,
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    let addr = socket::ip::SockAddrIpv4::new(socket::ip::Ipv4Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");

    // While we're here we'll also make sure we can call a TCP-specific
    // ioctl request on this file, since File::socket should've set that up.
    f.ioctl(crate::socket::ip::tcp::SIOCATMARK, ())
        .map_err(|e| e.into_std_io_error())
        .expect("failed to ioctl SIOCATMARK");
    // (We don't actually care about the result, only that it wasn't an error)
}

#[test]
fn socket_ipv6_bind_tcp() {
    use crate::socket;
    use std::println;

    let f = File::socket(
        socket::ip::AF_INET6,
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    let addr = socket::ip::SockAddrIpv6::new(socket::ip::Ipv6Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");

    // While we're here we'll also make sure we can call a TCP-specific
    // ioctl request on this file, since File::socket should've set that up.
    f.ioctl(crate::socket::ip::tcp::SIOCATMARK, ())
        .map_err(|e| e.into_std_io_error())
        .expect("failed to ioctl SIOCATMARK");
    // (We don't actually care about the result, only that it wasn't an error)
}

#[test]
fn socket_dynipv4_bind_tcp() {
    use crate::socket;
    use std::println;

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    // Passing an IPv4 address to SockAddrIp::new causes it to return an
    // IPv4 socket address.
    let addr = socket::ip::SockAddrIp::new(socket::ip::Ipv4Addr::LOOPBACK, 0);
    assert_eq!(addr.address_family(), crate::socket::ip::AF_INET);

    let f = File::socket(
        addr.address_family(),
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");
}

#[test]
fn socket_dynipv6_bind_tcp() {
    use crate::socket;
    use std::println;

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    // Passing an IPv6 address to SockAddrIp::new causes it to return an
    // IPv6 socket address.
    let addr = socket::ip::SockAddrIp::new(socket::ip::Ipv6Addr::LOOPBACK, 0);
    assert_eq!(addr.address_family(), crate::socket::ip::AF_INET6);

    let f = File::socket(
        addr.address_family(),
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");
}

#[test]
fn socket_dynipv6mappedv4_bind_tcp() {
    use crate::socket;
    use std::println;

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    // This is the IPv4 loopback address represented as an IPv6 address
    // using the "mapped" addressing scheme.
    let addr = socket::ip::SockAddrIp::new(socket::ip::Ipv4Addr::LOOPBACK.to_ipv6_mapped(), 0);
    assert_eq!(addr.address_family(), crate::socket::ip::AF_INET6);

    let f = File::socket(
        addr.address_family(),
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");
}

#[test]
fn socket_getsockopt() {
    use crate::socket;
    use std::println;

    let f = File::socket(
        socket::ip::AF_INET,
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    let addr = socket::ip::SockAddrIpv4::new(socket::ip::Ipv4Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");

    let acceptconn = f
        .getsockopt(crate::fd::sockopt::SO_ACCEPTCONN)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to getsockopt");
    assert_eq!(
        acceptconn, 0,
        "socket is already accepting connections somehow"
    );

    f.listen(1)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to listen");

    let acceptconn = f
        .getsockopt(crate::fd::sockopt::SO_ACCEPTCONN)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to getsockopt");
    assert_eq!(
        acceptconn, 1,
        "socket is not accepting connections after listen"
    );
}

#[test]
fn socket_setsockopt() {
    use crate::socket;
    use std::println;

    let f = File::socket(
        socket::ip::AF_INET,
        socket::sock_type::SOCK_STREAM,
        socket::ip::IPPROTO_TCP,
    )
    .map_err(|e| e.into_std_io_error())
    .expect("failed to create socket");

    let addr = socket::ip::SockAddrIpv4::new(socket::ip::Ipv4Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");

    let dontroute = f
        .getsockopt(crate::fd::sockopt::SO_DONTROUTE)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to getsockopt");
    assert_eq!(dontroute, 0, "SO_DONTROUTE is set before we set it");

    f.setsockopt(crate::fd::sockopt::SO_DONTROUTE, 1)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to setsockopt");

    let dontroute = f
        .getsockopt(crate::fd::sockopt::SO_DONTROUTE)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to getsockopt");
    assert_eq!(dontroute, 1, "SO_DONTROUTE is not set after we set it");
}

#[test]
fn futex_mutex() {
    use crate::sync::Mutex;
    use std::sync::Arc;
    use std::thread;

    let m1 = Arc::new(Mutex::new(0_usize));
    let m2 = Arc::clone(&m1);
    let m3 = Arc::clone(&m1);

    let join1 = thread::spawn(move || {
        let mut g = m1.lock();
        *g += 1;
    });
    let join2 = thread::spawn(move || {
        let mut g = m2.lock();
        *g += 1;
    });
    join1.join().unwrap();
    join2.join().unwrap();

    let g = m3.lock();
    std::assert_eq!(*g, 2, "wrong final value");
}
