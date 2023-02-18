use crate::fd::ioctl::{ioctl_read, ioctl_write, IoctlReqRead, IoctlReqWrite};

/// `ioctl` request for retrieving the current window size of a tty.
// NOTE: This ioctl number isn't valid for all Linux architectures, but is valid
// for all of the ones linux-unsafe supoorts at the time of writing.
pub const TIOCGWINSZ: IoctlReqRead<TtyDevice, WindowSize> = unsafe { ioctl_read(0x5413) };

/// `ioctl` request for changing the window size of a tty.
// NOTE: This ioctl number isn't valid for all Linux architectures, but is valid
// for all of the ones linux-unsafe supoorts at the time of writing.
pub const TIOCSWINSZ: IoctlReqWrite<TtyDevice, WindowSize> = unsafe { ioctl_write(0x5414) };

/// Represents the size of the window (or equivalent) that a tty is presented
/// through.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WindowSize {
    /// The number of rows in the window.
    pub ws_row: linux_unsafe::ushort,

    /// The number of columns in the window.
    pub ws_col: linux_unsafe::ushort,

    /// Not actually used.
    pub ws_xpixel: linux_unsafe::ushort,

    /// Not actually used.
    pub ws_ypixel: linux_unsafe::ushort,
}

/// A marker type for [`super::File`] objects that represent tty devices.
pub struct TtyDevice;

impl super::fd::ioctl::IoDevice for TtyDevice {}
