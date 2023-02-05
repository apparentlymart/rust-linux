extern crate std;

use crate::raw;

#[test]
fn test_syscall_getpid() {
    let want = std::process::id();
    let got = unsafe { raw::syscall0(raw::GETPID) } as u32;
    assert_eq!(
        got, want,
        "result {} does not match actual pid {}",
        got, want,
    );
}
