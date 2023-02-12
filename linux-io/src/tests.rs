extern crate std;
use std::vec::Vec;
use std::{os::unix::prelude::OsStrExt, path::PathBuf};

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

    let mut f = File::create_raw(&filename_raw, 0o666)
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
fn socket_ipv4_bind_tcp() {
    use crate::sockaddr;
    use std::println;

    // AF_INET + SOCK_STREAM is implicitly TCP, without explicitly naming it
    let mut f = File::socket(sockaddr::ip::AF_INET, sockaddr::sock_type::SOCK_STREAM, 0)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create socket");

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    let addr = sockaddr::ip::SockAddrIpv4::new(sockaddr::ip::Ipv4Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");
}

#[test]
fn socket_ipv6_bind_tcp() {
    use crate::sockaddr;
    use std::println;

    // AF_INET6 + SOCK_STREAM is implicitly TCP, without explicitly naming it
    let mut f = File::socket(sockaddr::ip::AF_INET6, sockaddr::sock_type::SOCK_STREAM, 0)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to create socket");

    // Using a dynamically-assigned loopback port to minimize the risk of
    // collisions when running these tests on systems that probably have
    // other network software running.
    let addr = sockaddr::ip::SockAddrIpv6::new(sockaddr::ip::Ipv6Addr::LOOPBACK, 0);
    println!("binding to {:?}", addr);
    f.bind(addr)
        .map_err(|e| e.into_std_io_error())
        .expect("failed to bind socket");
}
