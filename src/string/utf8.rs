use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

use cstl_sys::{
    CSTL_UTF8StringVal, CSTL_u8string_append_char, CSTL_u8string_append_n, CSTL_u8string_assign_n,
    CSTL_u8string_c_str, CSTL_u8string_clear, CSTL_u8string_destroy, CSTL_u8string_reserve,
    CSTL_u8string_shrink_to_fit,
};

use crate::alloc::{CxxProxy, WithCxxProxy};

pub type CxxUtf8String<A = SysAlloc> = CxxUtf8StringLayout<A, Layout<A>>;

#[repr(C)]
pub struct Layout<A: CxxProxy> {
    alloc: A,
    val: CSTL_UTF8StringVal,
}

#[repr(C)]
pub struct CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
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

impl CxxUtf8String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            inner: Layout::new_in(SysAlloc),
        }
    }
}

impl<A: CxxProxy> CxxUtf8String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Layout::new_in(alloc),
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.inner.alloc
    }
}

impl<A, L> CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    pub fn from_bytes_in<T: AsRef<[u8]>>(s: T, alloc: A) -> Self {
        let mut new = Self::from_alloc(alloc);

        let slice = s.as_ref();

        new.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u8string_assign_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u8 {
        unsafe { CSTL_u8string_c_str(self.inner.value_as_ref()) as _ }
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
            CSTL_u8string_append_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn replace<T: AsRef<[u8]>>(&mut self, s: T) {
        self.clear();
        self.push(s);
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_u8string_clear(self.inner.value_as_mut());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u8string_reserve(val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u8string_shrink_to_fit(val, alloc);
        });
    }

    fn from_alloc(alloc: A) -> Self {
        Self {
            inner: L::new_in(alloc),
        }
    }
}

impl<A, L> fmt::Debug for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxUtf8String")
            .field("length", &self.inner.value_as_ref().size)
            .field("capacity", &self.inner.value_as_ref().res)
            .field("large_mode", &(self.inner.value_as_ref().res > 15))
            .finish()
    }
}

impl<A, L> AsRef<[u8]> for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A, L> Borrow<[u8]> for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn borrow(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A, L> Default for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy + Default,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn default() -> Self {
        Self::from_alloc(A::default())
    }
}

impl<A, L> Drop for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn drop(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u8string_destroy(val, alloc);
        });
    }
}

impl<A, L> Clone for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy + Clone,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.inner.alloc_as_ref().clone())
    }
}

impl<A, L> Extend<u8> for CxxUtf8StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<u8, Alloc = A, Value = CSTL_UTF8StringVal>,
{
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            for ch in iter {
                CSTL_u8string_append_char(val, 1, ch, alloc);
            }
        });
    }
}

const fn new_val() -> CSTL_UTF8StringVal {
    CSTL_UTF8StringVal {
        bx: cstl_sys::CSTL_UTF8StringUnion { buf: [0; 16] },
        size: 0,
        res: 15,
    }
}

impl<A: CxxProxy> WithCxxProxy<u8> for Layout<A> {
    type Value = CSTL_UTF8StringVal;
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
    use cstl_sys::CSTL_UTF8StringVal;

    use crate::alloc::{CxxProxy, WithCxxProxy};

    use super::{new_val, CxxUtf8StringLayout, SysAlloc};

    pub type CxxUtf8String<A = SysAlloc> = CxxUtf8StringLayout<A, Layout<A>>;

    #[repr(C)]
    pub struct Layout<A: CxxProxy> {
        val: CSTL_UTF8StringVal,
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

    impl CxxUtf8String<SysAlloc> {
        pub const fn new() -> Self {
            Self {
                inner: Layout::new_in(SysAlloc),
            }
        }
    }

    impl<A: CxxProxy> CxxUtf8String<A> {
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
        type Value = CSTL_UTF8StringVal;
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
