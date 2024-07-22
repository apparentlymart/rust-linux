/// A pointer to readable userspace memory with a defined in-memory representation.
///
/// This exists primarily for use in defining safe `ioctl`/etc request types,
/// to ensure that only valid addresses can be assigned to a field from
/// safe Rust.
///
/// If a struct type used with an ioctl request has a field containing an
/// address that the kernel only reads from then defining that field as
/// a `UserPtr` ensures that it can only have a valid address assigned
/// to it unless the caller uses the unsafe [`Self::from_ptr`].
///
/// The in-memory representation of this type is guaranteed to match
/// that of type parameter `R`, which defaults to being a pointer to `T`.
#[repr(transparent)]
pub struct UserPtr<'a, T: Sized, R: Copy + Repr<T> = *const T> {
    v: R,
    _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: Sized, R: Copy + Repr<T>> UserPtr<'a, T, R> {
    /// Create a representation of the given pointer.
    ///
    /// # Safety
    ///
    /// The pointer must remain valid for the lifetime of the result.
    #[inline(always)]
    pub unsafe fn from_ptr(ptr: *const T) -> Self {
        Self {
            v: R::from_ptr(ptr),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a representation of a pointer referring to the same object
    /// as the given reference.
    #[inline(always)]
    pub fn from_ref(r: &'a T) -> Self {
        let ptr = r as *const _;
        unsafe { Self::from_ptr(ptr) }
    }

    /// Get the in-memory representation of the pointer.
    #[inline(always)]
    pub fn repr(&self) -> R {
        self.v
    }
}

/// A pointer to mutable userspace memory with a defined in-memory representation.
///
/// This exists primarily for use in defining safe `ioctl`/etc request types,
/// to ensure that only valid addresses can be assigned to a field from
/// safe Rust. Using this is important for any pointer that the kernel might
/// write through, to ensure that safe Rust cannot assign an address that
/// might cause the kernel to corrupt memory.
///
/// If a struct type used with an ioctl request has a field containing an
/// address that the kernel might write to then defining that field as
/// a `UserPtr` ensures that it can only have a valid address assigned
/// to it unless the caller uses the unsafe [`Self::from_ptr`].
///
/// The in-memory representation of this type is guaranteed to match
/// that of type parameter `R`, which defaults to being a pointer to `T`.
#[repr(transparent)]
pub struct UserMut<'a, T: Sized, R: Copy + ReprMut<T> = *mut T> {
    v: R,
    _phantom: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T: Sized, R: Copy + ReprMut<T>> UserMut<'a, T, R> {
    /// Create a representation of the given pointer.
    ///
    /// # Safety
    ///
    /// The pointer must remain valid for the lifetime of the result.
    #[inline(always)]
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            v: R::from_mut_ptr(ptr),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a representation of a pointer referring to the same object
    /// as the given reference.
    #[inline(always)]
    pub fn from_ref(r: &'a mut T) -> Self {
        let ptr = r as *mut _;
        unsafe { Self::from_ptr(ptr) }
    }

    /// Get the in-memory representation of the pointer.
    #[inline(always)]
    pub fn repr(&self) -> R {
        self.v
    }
}

/// Trait implemented by types that can represent readable userspace addresses.
pub unsafe trait Repr<T> {
    /// Convert the given pointer to the implementing type, without
    /// losing any information that the kernel would need to treat
    /// the result as a userspace pointer to the same address.
    fn from_ptr(ptr: *const T) -> Self;
}

/// Trait implemented by types that can represent writable userspace addresses.
pub unsafe trait ReprMut<T> {
    /// Convert the given pointer to the implementing type, without
    /// losing any information that the kernel would need to treat
    /// the result as a userspace pointer to the same address.
    fn from_mut_ptr(ptr: *mut T) -> Self;
}

unsafe impl<T> Repr<T> for *const T {
    #[inline(always)]
    fn from_ptr(ptr: *const T) -> Self {
        ptr
    }
}

unsafe impl<T> ReprMut<T> for *mut T {
    #[inline(always)]
    fn from_mut_ptr(ptr: *mut T) -> Self {
        ptr as *mut T
    }
}

unsafe impl<T> Repr<T> for u64 {
    #[inline(always)]
    fn from_ptr(ptr: *const T) -> Self {
        ptr as u64
    }
}

unsafe impl<T> ReprMut<T> for u64 {
    #[inline(always)]
    fn from_mut_ptr(ptr: *mut T) -> Self {
        ptr as u64
    }
}

unsafe impl<T> Repr<T> for usize {
    #[inline(always)]
    fn from_ptr(ptr: *const T) -> Self {
        ptr as usize
    }
}

unsafe impl<T> ReprMut<T> for usize {
    #[inline(always)]
    fn from_mut_ptr(ptr: *mut T) -> Self {
        ptr as usize
    }
}

#[cfg(test)]
#[allow(dead_code, unused_variables)]
pub fn ensure_reprs_possible() {
    // This is here just as a set of static assertions to ensure that
    // some commonly-needed patterns remain possible.

    let ptr: UserPtr<u8> = UserPtr::from_ref(&0_u8);
    let ptr: UserPtr<u8, *const u8> = UserPtr::from_ref(&0_u8);
    let ptr: UserPtr<u8, u64> = UserPtr::from_ref(&0_u8);
    let ptr: UserPtr<u8, usize> = UserPtr::from_ref(&0_u8);

    let ptr: UserMut<u8> = UserMut::from_ref(&mut 0_u8);
    let ptr: UserMut<u8, *mut u8> = UserMut::from_ref(&mut 0_u8);
    let ptr: UserMut<u8, u64> = UserMut::from_ref(&mut 0_u8);
    let ptr: UserMut<u8, usize> = UserMut::from_ref(&mut 0_u8);
}
