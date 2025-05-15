use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

use cstl_sys::{
    CSTL_UTF8StringVal, CSTL_u8string_append_char, CSTL_u8string_append_n, CSTL_u8string_assign_n,
    CSTL_u8string_c_str, CSTL_u8string_clear, CSTL_u8string_destroy, CSTL_u8string_reserve,
    CSTL_u8string_shrink_to_fit,
};

use crate::alloc::{with_proxy, CxxProxy};

#[repr(C)]
pub struct CxxUtf8String<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_UTF8StringVal,
}

impl CxxUtf8String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_UTF8StringVal {
                bx: cstl_sys::CSTL_UTF8StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}

impl<A: CxxProxy> CxxUtf8String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_UTF8StringVal {
                bx: cstl_sys::CSTL_UTF8StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.alloc
    }

    pub fn from_bytes_in<T: AsRef<[u8]>>(s: T, alloc: A) -> Self {
        let mut new = Self::new_in(alloc);

        let slice = s.as_ref();

        with_proxy(&new.alloc, |alloc| unsafe {
            CSTL_u8string_assign_n(&mut new.val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u8 {
        unsafe { CSTL_u8string_c_str(&self.val) as _ }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(CSTL_u8string_c_str(&self.val) as _, self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(CSTL_u8string_c_str(&self.val) as _, self.len() + 1) }
    }

    pub fn len(&self) -> usize {
        self.val.size
    }

    pub fn is_empty(&self) -> bool {
        self.val.size == 0
    }

    pub fn capacity(&self) -> usize {
        self.val.res
    }

    pub fn push<T: AsRef<[u8]>>(&mut self, s: T) {
        let slice = s.as_ref();

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u8string_append_n(&mut self.val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_u8string_clear(&mut self.val);
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u8string_reserve(&mut self.val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u8string_shrink_to_fit(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy> fmt::Debug for CxxUtf8String<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxUtf8String")
            .field("length", &self.val.size)
            .field("capacity", &self.val.res)
            .field("large_mode", &(self.val.res > 15))
            .finish()
    }
}

impl<A: CxxProxy> AsRef<[u8]> for CxxUtf8String<A> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A: CxxProxy> Borrow<[u8]> for CxxUtf8String<A> {
    fn borrow(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A> Default for CxxUtf8String<A>
where
    A: CxxProxy + Default,
{
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<A: CxxProxy> Drop for CxxUtf8String<A> {
    fn drop(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u8string_destroy(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy + Clone> Clone for CxxUtf8String<A> {
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.alloc.clone())
    }
}

impl<A: CxxProxy> Extend<u8> for CxxUtf8String<A> {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        with_proxy(&self.alloc, |alloc| unsafe {
            for ch in iter {
                CSTL_u8string_append_char(&mut self.val, 1, ch, alloc);
            }
        });
    }
}
