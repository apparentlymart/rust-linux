#![allow(non_camel_case_types)]

/// A set of signals for use with signal blocking functions.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct sigset_t {
    // For now we only support 32-bit and 64-bit architectures, and there are
    // only 32 signals defined, so it's safe to assume that they all fit in
    // one element. We'll need to be more clever about this if we ever support
    // an architecture where sigset_t has a different layout.
    sig: [crate::ulong; 1],
}

impl sigset_t {
    const ELEMS: usize = 1;
    const ELEM_BITS: usize = core::mem::size_of::<crate::ulong>() * 8;
    const FILLED: crate::ulong = !0;

    #[inline]
    pub const fn new_empty() -> Self {
        Self {
            sig: [0; Self::ELEMS],
        }
    }

    #[inline]
    pub const fn new_filled() -> Self {
        Self {
            sig: [Self::FILLED; Self::ELEMS],
        }
    }

    #[inline]
    pub fn sigemptyset(&mut self) {
        for elem in &mut self.sig {
            *elem = 0;
        }
    }

    #[inline]
    pub fn sigfillset(&mut self) {
        for elem in &mut self.sig {
            *elem = Self::FILLED;
        }
    }

    #[inline]
    pub fn sigaddset(&mut self, signum: crate::int) -> crate::result::Result<()> {
        let (elem, bit) = Self::sigpos(signum)?;
        self.sig[elem] |= (1 << bit) as crate::ulong;
        Ok(())
    }

    #[inline]
    pub fn sigdelset(&mut self, signum: crate::int) -> crate::result::Result<()> {
        let (elem, bit) = Self::sigpos(signum)?;
        self.sig[elem] &= !(1 << bit) as crate::ulong;
        Ok(())
    }

    #[inline]
    pub fn sigismember(&mut self, signum: crate::int) -> crate::result::Result<bool> {
        let (elem, bit) = Self::sigpos(signum)?;
        Ok((self.sig[elem] & (1 << bit)) != 0)
    }

    pub fn as_ptr(&self) -> *const Self {
        self as *const Self
    }

    pub fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    fn sigpos(signum: crate::int) -> crate::result::Result<(usize, usize)> {
        let total_bit = (signum - 1) as usize;
        let elem = total_bit / Self::ELEM_BITS;
        if elem >= Self::ELEMS {
            return Err(crate::result::Error::new(22 /* EINVAL */));
        }
        let bit = total_bit % Self::ELEM_BITS;
        Ok((elem, bit))
    }
}
