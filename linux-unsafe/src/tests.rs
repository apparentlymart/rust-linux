extern crate std;

use crate::*;

#[test]
fn test_raw_syscall_getpid() {
    let want = std::process::id();
    let got = unsafe { raw::syscall0(raw::GETPID) } as u32;
    assert_eq!(
        got, want,
        "result {} does not match actual pid {}",
        got, want,
    );
}

#[test]
fn test_getpid() {
    let want = std::process::id() as pid_t;
    let got = unsafe { getpid() };
    assert_eq!(
        got, want,
        "result {} does not match actual pid {}",
        got, want,
    );
}
