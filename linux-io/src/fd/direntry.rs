use core::{ffi::CStr, slice};

/// A single directory entry extracted from a buffer populated by `getdents64`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DirEntry<'a> {
    pub ino: linux_unsafe::ino64_t,
    pub off: linux_unsafe::off64_t,
    pub entry_type: DirEntryType,
    pub name: &'a CStr,
}

/// Directory entry type.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum DirEntryType {
    Unknown = 0,
    Fifo = 1,
    Chr = 2,
    Dir = 4,
    Blk = 6,
    Reg = 8,
    Lnk = 10,
    Sock = 12,
    Wht = 14,
}

impl From<linux_unsafe::uchar> for DirEntryType {
    fn from(value: linux_unsafe::uchar) -> Self {
        match value {
            linux_unsafe::DT_FIFO => Self::Fifo,
            linux_unsafe::DT_CHR => Self::Chr,
            linux_unsafe::DT_DIR => Self::Dir,
            linux_unsafe::DT_BLK => Self::Blk,
            linux_unsafe::DT_REG => Self::Reg,
            linux_unsafe::DT_LNK => Self::Lnk,
            linux_unsafe::DT_SOCK => Self::Sock,
            linux_unsafe::DT_WHT => Self::Wht,
            _ => Self::Unknown,
        }
    }
}

/// An iterator over directory entries in an already-populated `getdents64`
/// result buffer.
pub struct DirEntries<'a> {
    remain: &'a [u8],
}

impl<'a> DirEntries<'a> {
    pub fn from_getdents64_buffer(buf: &'a [u8]) -> Self {
        Self { remain: buf }
    }

    /// Consume the iterator object and obtain the remaining bytes that it
    /// hasn't yet transformed into `DirEntry` values.
    ///
    /// The result could be passed back to [`Self::from_getdents64_buffer`]
    /// to continue iterating.
    pub fn to_remaining_bytes(self) -> &'a [u8] {
        self.remain
    }
}

impl<'a> Iterator for DirEntries<'a> {
    type Item = DirEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (ret, remain) = dir_entry_from_buf(self.remain);
        self.remain = remain;
        ret
    }
}

fn dir_entry_from_buf<'a>(buf: &'a [u8]) -> (Option<DirEntry<'a>>, &'a [u8]) {
    #[derive(Debug)]
    #[repr(C)]
    struct DirEntryHeader {
        // These fields must match the fixed part of linux_unsafe::linux_dirent64
        d_ino: linux_unsafe::ino64_t,
        d_off: linux_unsafe::off64_t,
        d_reclen: linux_unsafe::ushort,
        d_type: linux_unsafe::uchar,
        d_name: (),
    }
    // NOTE: Because DirEntryHeader has 8-byte alignment, HEADER_SIZE is
    // 24 (3*8) even though the name begins at offset 19 (NAME_OFFSET).
    // The kernel leaves padding bytes between the entries to maintain
    // the needed alignment.
    const HEADER_SIZE: usize = core::mem::size_of::<DirEntryHeader>();
    const NAME_OFFSET: usize = core::mem::offset_of!(DirEntryHeader, d_name);

    if buf.len() < HEADER_SIZE {
        // Not enough bytes left for an entry.
        return (None, buf);
    }

    let (raw_len, ino, off, entry_type) = {
        let hdr_ptr = buf.as_ptr() as *const DirEntryHeader;
        let hdr = unsafe { &*hdr_ptr };
        let claimed_len = hdr.d_reclen as usize;
        if buf.len() < claimed_len || claimed_len < HEADER_SIZE {
            // Not enough room for the claimed length, or claimed length
            // shorter than the header. Neither is valid.
            return (None, buf);
        }
        (claimed_len, hdr.d_ino, hdr.d_off, hdr.d_type)
    };

    let name_len = raw_len - NAME_OFFSET;
    let name_start = unsafe { buf.as_ptr().add(NAME_OFFSET) } as *const u8;
    let name = unsafe { slice::from_raw_parts::<'a, _>(name_start, name_len) };
    let name = CStr::from_bytes_until_nul(name).unwrap();

    let remain = &buf[raw_len..];
    let ret = DirEntry {
        ino,
        off,
        entry_type: entry_type.into(),
        name,
    };
    (Some(ret), remain)
}

pub struct AllDirEntries<'file, 'buf, TF, R, D>
where
    TF: FnMut(DirEntry<'buf>) -> R,
{
    f: Option<&'file crate::File<D>>,
    buf: &'buf mut [u8],
    rng: Range,
    transform: TF,
}

impl<'file, 'buf, TF, R, D> AllDirEntries<'file, 'buf, TF, R, D>
where
    TF: for<'tmp> FnMut(DirEntry<'tmp>) -> R,
{
    pub(crate) fn new(f: &'file crate::File<D>, buf: &'buf mut [u8], transform: TF) -> Self {
        Self {
            f: Some(f),
            buf,
            rng: Range::new(0, 0),
            transform,
        }
    }
}

fn try_read_entry<'a>(buf: &'a [u8]) -> (Option<DirEntry<'a>>, usize) {
    if buf.len() == 0 {
        return (None, 0);
    }
    let (ret, remain) = dir_entry_from_buf(buf);
    if let Some(entry) = ret {
        let advance = remain.as_ptr() as usize - buf.as_ptr() as usize;
        (Some(entry), advance)
    } else {
        (None, buf.len())
    }
}

impl<'file, 'buf, TF, R, D> Iterator for AllDirEntries<'file, 'buf, TF, R, D>
where
    TF: for<'tmp> FnMut(DirEntry<'tmp>) -> R,
{
    type Item = Result<R, crate::result::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(f) = self.f {
            {
                let buf = &self.buf[self.rng.start..self.rng.end];
                let (maybe_entry, advance) = try_read_entry(buf);
                self.rng.start += advance;
                if let Some(entry) = maybe_entry {
                    return Some(Ok((self.transform)(entry)));
                }
            }
            let result = unsafe {
                f.getdents_raw(
                    self.buf.as_mut_ptr() as *mut _,
                    self.buf.len() as linux_unsafe::int,
                )
            };
            match result {
                Ok(result_len) => {
                    self.rng = Range::new(0, result_len);
                    let buf = &self.buf[self.rng.start..self.rng.end];
                    let (maybe_entry, advance) = try_read_entry(buf);
                    self.rng.start += advance;
                    maybe_entry.map(|entry| Ok((self.transform)(entry)))
                }
                Err(e) => {
                    self.f = None; // no longer usable
                    Some(Err(e))
                }
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    #[inline(always)]
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
