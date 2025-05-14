use core::fmt;
use std::{alloc::System as SysAlloc, borrow::Borrow, slice};

use cstl_sys::{
    CSTL_UTF32StringVal, CSTL_u32string_append_char, CSTL_u32string_append_n,
    CSTL_u32string_assign_n, CSTL_u32string_c_str, CSTL_u32string_clear, CSTL_u32string_destroy,
    CSTL_u32string_reserve, CSTL_u32string_shrink_to_fit,
};

use crate::alloc::{with_proxy, CxxProxy};

#[repr(C)]
pub struct CxxUtf32String<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_UTF32StringVal,
}

impl CxxUtf32String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_UTF32StringVal {
                bx: cstl_sys::CSTL_UTF32StringUnion { buf: [0; 4] },
                size: 0,
                res: 3,
            },
        }
    }
}

impl<A: CxxProxy> CxxUtf32String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_UTF32StringVal {
                bx: cstl_sys::CSTL_UTF32StringUnion { buf: [0; 4] },
                size: 0,
                res: 3,
            },
        }
    }

    pub fn from_bytes_in<T: AsRef<[u32]>>(s: T, alloc: A) -> Self {
        let mut new = Self::new_in(alloc);

        let slice = s.as_ref();

        with_proxy(&new.alloc, |alloc| unsafe {
            CSTL_u32string_assign_n(&mut new.val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u32 {
        unsafe { CSTL_u32string_c_str(&self.val) as _ }
    }

    pub fn as_bytes(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(CSTL_u32string_c_str(&self.val) as _, self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(CSTL_u32string_c_str(&self.val) as _, self.len() + 1) }
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

    pub fn push<T: AsRef<[u32]>>(&mut self, s: T) {
        let slice = s.as_ref();

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u32string_append_n(&mut self.val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_u32string_clear(&mut self.val);
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u32string_reserve(&mut self.val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u32string_shrink_to_fit(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy> fmt::Debug for CxxUtf32String<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxUtf32String")
            .field("length", &self.val.size)
            .field("capacity", &self.val.res)
            .field("large_mode", &(self.val.res > 3))
            .finish()
    }
}

impl<A: CxxProxy> AsRef<[u32]> for CxxUtf32String<A> {
    fn as_ref(&self) -> &[u32] {
        self.as_bytes()
    }
}

impl<A: CxxProxy> Borrow<[u32]> for CxxUtf32String<A> {
    fn borrow(&self) -> &[u32] {
        self.as_bytes()
    }
}

impl<A> Default for CxxUtf32String<A>
where
    A: CxxProxy + Default,
{
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<A: CxxProxy> Drop for CxxUtf32String<A> {
    fn drop(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_u32string_destroy(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy + Clone> Clone for CxxUtf32String<A> {
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.alloc.clone())
    }
}

impl<A: CxxProxy> Extend<u32> for CxxUtf32String<A> {
    fn extend<I: IntoIterator<Item = u32>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        with_proxy(&self.alloc, |alloc| unsafe {
            for ch in iter {
                CSTL_u32string_append_char(&mut self.val, 1, ch, alloc);
            }
        });
    }
}
