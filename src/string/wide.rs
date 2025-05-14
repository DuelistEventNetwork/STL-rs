use core::fmt;
use std::{alloc::System as SysAlloc, borrow::Borrow, slice};

use cstl_sys::{
    CSTL_WideStringVal, CSTL_wstring_append_n, CSTL_wstring_assign_n, CSTL_wstring_c_str, CSTL_wstring_clear, CSTL_wstring_destroy, CSTL_wstring_reserve, CSTL_wstring_shrink_to_fit
};

use crate::alloc::{with_proxy, CxxProxy};

#[repr(C)]
pub struct CxxWideString<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_WideStringVal,
}

impl CxxWideString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_WideStringVal {
                bx: cstl_sys::CSTL_WideStringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }
}

impl<A: CxxProxy> CxxWideString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_WideStringVal {
                bx: cstl_sys::CSTL_WideStringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }

    pub fn from_bytes_in<T: AsRef<[u16]>>(s: T, alloc: A) -> Self {
        let mut new = Self::new_in(alloc);

        let slice = s.as_ref();

        with_proxy(&new.alloc, |alloc| unsafe {
            CSTL_wstring_assign_n(&mut new.val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u16 {
        unsafe { CSTL_wstring_c_str(&self.val) as _ }
    }

    pub fn as_bytes(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(CSTL_wstring_c_str(&self.val) as _, self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(CSTL_wstring_c_str(&self.val) as _, self.len() + 1) }
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

    pub fn push<T: AsRef<[u16]>>(&mut self, s: T) {
        let slice = s.as_ref();

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_wstring_append_n(&mut self.val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_wstring_clear(&mut self.val);
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_wstring_reserve(&mut self.val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_wstring_shrink_to_fit(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy> fmt::Debug for CxxWideString<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxWideString")
            .field("length", &self.val.size)
            .field("capacity", &self.val.res)
            .field("large_mode", &(self.val.res > 7))
            .finish()
    }
}

impl<A: CxxProxy> AsRef<[u16]> for CxxWideString<A> {
    fn as_ref(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A: CxxProxy> Borrow<[u16]> for CxxWideString<A> {
    fn borrow(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A> Default for CxxWideString<A>
where
    A: CxxProxy + Default,
{
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<A: CxxProxy> Drop for CxxWideString<A> {
    fn drop(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_wstring_destroy(&mut self.val, alloc);
        });
    }
}

impl<A: CxxProxy + Clone> Clone for CxxWideString<A> {
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.alloc.clone())
    }
}
