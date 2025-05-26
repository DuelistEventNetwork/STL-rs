use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

use cstl_sys::{
    CSTL_StringVal, CSTL_string_append_char, CSTL_string_append_n, CSTL_string_assign_n,
    CSTL_string_c_str, CSTL_string_clear, CSTL_string_destroy, CSTL_string_reserve,
    CSTL_string_shrink_to_fit,
};

use crate::alloc::{CxxProxy, WithCxxProxy};

pub type CxxNarrowString<A = SysAlloc> = CxxNarrowStringLayout<A, Layout<A>>;

#[repr(C)]
pub struct Layout<A: CxxProxy> {
    alloc: A,
    val: CSTL_StringVal,
}

#[repr(C)]
pub struct CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    inner: L,
}

impl<A: CxxProxy> Layout<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: new_val(),
        }
    }
}

impl CxxNarrowString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            inner: Layout::new_in(SysAlloc),
        }
    }
}

impl<A: CxxProxy> CxxNarrowString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Layout::new_in(alloc),
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.inner.alloc
    }
}

impl<A, L> CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    pub fn from_bytes_in<T: AsRef<[u8]>>(s: T, alloc: A) -> Self {
        let mut new = Self::from_alloc(alloc);

        let slice = s.as_ref();

        new.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_string_assign_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u8 {
        unsafe { CSTL_string_c_str(self.inner.value_as_ref()) as _ }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len() + 1) }
    }

    pub fn len(&self) -> usize {
        self.inner.value_as_ref().size
    }

    pub fn is_empty(&self) -> bool {
        self.inner.value_as_ref().size == 0
    }

    pub fn capacity(&self) -> usize {
        self.inner.value_as_ref().res
    }

    pub fn push<T: AsRef<[u8]>>(&mut self, s: T) {
        let slice = s.as_ref();

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_string_append_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn replace<T: AsRef<[u8]>>(&mut self, s: T) {
        self.clear();
        self.push(s);
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_string_clear(self.inner.value_as_mut());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_string_reserve(val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_string_shrink_to_fit(val, alloc);
        });
    }

    fn from_alloc(alloc: A) -> Self {
        Self {
            inner: L::new_in(alloc),
        }
    }
}

impl<A, L> fmt::Debug for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxNarrowString")
            .field("length", &self.inner.value_as_ref().size)
            .field("capacity", &self.inner.value_as_ref().res)
            .field("large_mode", &(self.inner.value_as_ref().res > 15))
            .finish()
    }
}

impl<A, L> AsRef<[u8]> for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A, L> Borrow<[u8]> for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn borrow(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A, L> Default for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy + Default,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn default() -> Self {
        Self::from_alloc(A::default())
    }
}

impl<A, L> Drop for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn drop(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_string_destroy(val, alloc);
        });
    }
}

impl<A, L> Clone for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy + Clone,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.inner.alloc_as_ref().clone())
    }
}

impl<A, L> Extend<u8> for CxxNarrowStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_StringVal>,
{
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            for ch in iter {
                CSTL_string_append_char(val, 1, ch as i8, alloc);
            }
        });
    }
}

const fn new_val() -> CSTL_StringVal {
    CSTL_StringVal {
        bx: cstl_sys::CSTL_StringUnion { buf: [0; 16] },
        size: 0,
        res: 15,
    }
}

impl<A: CxxProxy> WithCxxProxy<u8> for Layout<A> {
    type Value = CSTL_StringVal;
    type Alloc = A;

    fn value_as_ref(&self) -> &Self::Value {
        &self.val
    }

    fn value_as_mut(&mut self) -> &mut Self::Value {
        &mut self.val
    }

    fn alloc_as_ref(&self) -> &Self::Alloc {
        &self.alloc
    }

    fn new_in(alloc: Self::Alloc) -> Self {
        Self {
            alloc,
            val: new_val(),
        }
    }
}

#[cfg(feature = "msvc2012")]
pub mod msvc2012 {
    use cstl_sys::CSTL_StringVal;

    use crate::alloc::{CxxProxy, WithCxxProxy};

    use super::{new_val, CxxNarrowStringLayout, SysAlloc};

    pub type CxxNarrowString<A = SysAlloc> = CxxNarrowStringLayout<A, Layout<A>>;

    #[repr(C)]
    pub struct Layout<A: CxxProxy> {
        val: CSTL_StringVal,
        alloc: A,
    }

    impl<A: CxxProxy> Layout<A> {
        pub const fn new_in(alloc: A) -> Self {
            Self {
                alloc,
                val: new_val(),
            }
        }
    }

    impl CxxNarrowString<SysAlloc> {
        pub const fn new() -> Self {
            Self {
                inner: Layout::new_in(SysAlloc),
            }
        }
    }

    impl<A: CxxProxy> CxxNarrowString<A> {
        pub const fn new_in(alloc: A) -> Self {
            Self {
                inner: Layout::new_in(alloc),
            }
        }

        pub const fn allocator(&self) -> &A {
            &self.inner.alloc
        }
    }

    impl<A: CxxProxy> WithCxxProxy<u8> for Layout<A> {
        type Value = CSTL_StringVal;
        type Alloc = A;

        fn value_as_ref(&self) -> &Self::Value {
            &self.val
        }

        fn value_as_mut(&mut self) -> &mut Self::Value {
            &mut self.val
        }

        fn alloc_as_ref(&self) -> &Self::Alloc {
            &self.alloc
        }

        fn new_in(alloc: Self::Alloc) -> Self {
            Self {
                alloc,
                val: new_val(),
            }
        }
    }
}
