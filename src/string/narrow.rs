use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

use cstl_sys::{
    CSTL_StringVal, CSTL_string_append_char, CSTL_string_append_n, CSTL_string_assign_n,
    CSTL_string_c_str, CSTL_string_clear, CSTL_string_destroy, CSTL_string_reserve,
    CSTL_string_shrink_to_fit,
};

use crate::alloc::{with_proxy, CxxProxy};

#[repr(C)]
pub struct CxxNarrowString<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_StringVal,
}

impl CxxNarrowString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_StringVal {
                bx: cstl_sys::CSTL_StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}

impl<A: CxxProxy> CxxNarrowString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_StringVal {
                bx: cstl_sys::CSTL_StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }

    pub fn from_bytes_in<T: AsRef<[u8]>>(s: T, alloc: A) -> Self {
        let mut new = Self::new_in(alloc);

        let slice = s.as_ref();

        with_proxy(&new.alloc, |alloc| unsafe {
            CSTL_string_assign_n(&mut new.val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u8 {
        unsafe { CSTL_string_c_str(&self.val) as _ }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(CSTL_string_c_str(&self.val) as _, self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(CSTL_string_c_str(&self.val) as _, self.len() + 1) }
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
            CSTL_string_append_n(&mut self.val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_string_clear(&mut self.val);
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_string_reserve(&mut self.val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_string_shrink_to_fit(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy> fmt::Debug for CxxNarrowString<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxNarrowString")
            .field("length", &self.val.size)
            .field("capacity", &self.val.res)
            .field("large_mode", &(self.val.res > 15))
            .finish()
    }
}

impl<A: CxxProxy> AsRef<[u8]> for CxxNarrowString<A> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A: CxxProxy> Borrow<[u8]> for CxxNarrowString<A> {
    fn borrow(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A> Default for CxxNarrowString<A>
where
    A: CxxProxy + Default,
{
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<A: CxxProxy> Drop for CxxNarrowString<A> {
    fn drop(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_string_destroy(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy + Clone> Clone for CxxNarrowString<A> {
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.alloc.clone())
    }
}

impl<A: CxxProxy> Extend<u8> for CxxNarrowString<A> {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        with_proxy(&self.alloc, |alloc| unsafe {
            for ch in iter {
                CSTL_string_append_char(&mut self.val, 1, ch as i8, alloc);
            }
        });
    }
}
