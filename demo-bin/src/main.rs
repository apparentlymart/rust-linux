#![no_std]

use core::fmt::Write;

use linux_io::File;

fn main() {
    // Wrap the stdout handle -- fd 1 -- in a File object.
    // (Safety: we are not using std and so nothing else thinks it owns this fd)
    let mut stdout = unsafe { File::from_raw_fd(1) };
    let stdin = unsafe { File::from_raw_fd(0) };

    if writeln!(stdout, "hello world").is_err() {
        unsafe { linux_unsafe::exit(1) };
    }

    let stdin_poll = linux_io::poll::PollRequest::new(&stdin);
    let mut poll_reqs = [stdin_poll];
    // FIXME: Can't currently loop because of overly-strict lifetime bounds
    // on PollRequest.
    // loop {
    let n = linux_io::poll::poll(&mut poll_reqs[..], 0).unwrap();
    if n > 0 {
        if poll_reqs[0].response().readable() {
            writeln!(stdout, "stdin is readable").unwrap();
            let mut buf = [0_u8; 8];
            let n = stdin.read(&mut buf[..]).unwrap();
            writeln!(stdout, "read {:?}\n", &buf[..n]).unwrap();
        } else {
            writeln!(stdout, "stdin did not become readable").unwrap();
        }
    } else {
        writeln!(stdout, "no poll events").unwrap();
    }
    //}
}
