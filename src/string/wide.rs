use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

pub use cstl_sys::CSTL_WideStringVal as RawWideString;
use cstl_sys::{
    CSTL_wstring_append_char, CSTL_wstring_append_n, CSTL_wstring_assign_n,
    CSTL_wstring_c_str, CSTL_wstring_clear, CSTL_wstring_destroy, CSTL_wstring_reserve,
    CSTL_wstring_shrink_to_fit,
};

use crate::alloc::{CxxProxy, WithCxxProxy};

pub type CxxWideString<A = SysAlloc> = CxxWideStringLayout<A, Layout<A>>;

#[repr(C)]
pub struct Layout<A: CxxProxy> {
    alloc: A,
    val: RawWideString,
}

#[repr(C)]
pub struct CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
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

impl CxxWideString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            inner: Layout::new_in(SysAlloc),
        }
    }
}

impl<A: CxxProxy> CxxWideString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Layout::new_in(alloc),
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.inner.alloc
    }
}

impl<A, L> CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    pub fn from_bytes_in<T: AsRef<[u16]>>(s: T, alloc: A) -> Self {
        let mut new = Self::from_alloc(alloc);

        let slice = s.as_ref();

        new.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_wstring_assign_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u16 {
        unsafe { CSTL_wstring_c_str(self.inner.value_as_ref()) as _ }
    }

    pub fn as_bytes(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    pub fn as_bytes_with_nul(&self) -> &[u16] {
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

    pub fn push<T: AsRef<[u16]>>(&mut self, s: T) {
        let slice = s.as_ref();

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_wstring_append_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn replace<T: AsRef<[u16]>>(&mut self, s: T) {
        self.clear();
        self.push(s);
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_wstring_clear(self.inner.value_as_mut());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_wstring_reserve(val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_wstring_shrink_to_fit(val, alloc);
        });
    }

    fn from_alloc(alloc: A) -> Self {
        Self {
            inner: L::new_in(alloc),
        }
    }
}

impl<A, L> fmt::Debug for CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxWideString")
            .field("length", &self.inner.value_as_ref().size)
            .field("capacity", &self.inner.value_as_ref().res)
            .field("large_mode", &(self.inner.value_as_ref().res > 15))
            .finish()
    }
}

impl<A, L> AsRef<[u16]> for CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn as_ref(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A, L> Borrow<[u16]> for CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn borrow(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A, L> Default for CxxWideStringLayout<A, L>
where
    A: CxxProxy + Default,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn default() -> Self {
        Self::from_alloc(A::default())
    }
}

impl<A, L> Drop for CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn drop(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_wstring_destroy(val, alloc);
        });
    }
}

impl<A, L> Clone for CxxWideStringLayout<A, L>
where
    A: CxxProxy + Clone,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.inner.alloc_as_ref().clone())
    }
}

impl<A, L> Extend<u16> for CxxWideStringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawWideString>,
{
    fn extend<I: IntoIterator<Item = u16>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            for ch in iter {
                CSTL_wstring_append_char(val, 1, ch, alloc);
            }
        });
    }
}

const fn new_val() -> RawWideString {
    RawWideString {
        bx: cstl_sys::CSTL_WideStringUnion { buf: [0; 8] },
        size: 0,
        res: 7,
    }
}

impl<A: CxxProxy> WithCxxProxy for Layout<A> {
    type Value = RawWideString;
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
    use cstl_sys::CSTL_WideStringVal as RawWideString;

    use crate::alloc::{CxxProxy, WithCxxProxy};

    use super::{new_val, CxxWideStringLayout, SysAlloc};

    pub type CxxWideString<A = SysAlloc> = CxxWideStringLayout<A, Layout<A>>;

    #[repr(C)]
    pub struct Layout<A: CxxProxy> {
        val: RawWideString,
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

    impl CxxWideString<SysAlloc> {
        pub const fn new() -> Self {
            Self {
                inner: Layout::new_in(SysAlloc),
            }
        }
    }

    impl<A: CxxProxy> CxxWideString<A> {
        pub const fn new_in(alloc: A) -> Self {
            Self {
                inner: Layout::new_in(alloc),
            }
        }

        pub const fn allocator(&self) -> &A {
            &self.inner.alloc
        }
    }

    impl<A: CxxProxy> WithCxxProxy<u16> for Layout<A> {
        type Value = RawWideString;
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
