use std::{alloc::System as SysAlloc, borrow::Borrow, fmt, slice};

pub use cstl_sys::CSTL_UTF16StringVal as RawUtf16String;
use cstl_sys::{
    CSTL_u16string_append_char, CSTL_u16string_append_n, CSTL_u16string_assign_n,
    CSTL_u16string_c_str, CSTL_u16string_clear, CSTL_u16string_destroy, CSTL_u16string_reserve,
    CSTL_u16string_shrink_to_fit,
};

use crate::alloc::{CxxProxy, WithCxxProxy};

pub type CxxUtf16String<A = SysAlloc> = CxxUtf16StringLayout<A, Layout<A>>;

#[repr(C)]
pub struct Layout<A: CxxProxy> {
    alloc: A,
    val: RawUtf16String,
}

#[repr(C)]
pub struct CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
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

impl CxxUtf16String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            inner: Layout::new_in(SysAlloc),
        }
    }
}

impl<A: CxxProxy> CxxUtf16String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Layout::new_in(alloc),
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.inner.alloc
    }
}

impl<A, L> CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    pub fn from_bytes_in<T: AsRef<[u16]>>(s: T, alloc: A) -> Self {
        let mut new = Self::from_alloc(alloc);

        let slice = s.as_ref();

        new.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u16string_assign_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });

        new
    }

    pub fn as_ptr(&self) -> *const u16 {
        unsafe { CSTL_u16string_c_str(self.inner.value_as_ref()) as _ }
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
            CSTL_u16string_append_n(val, slice.as_ptr() as _, slice.len(), alloc);
        });
    }

    pub fn replace<T: AsRef<[u16]>>(&mut self, s: T) {
        self.clear();
        self.push(s);
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_u16string_clear(self.inner.value_as_mut());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u16string_reserve(val, capacity + additional, alloc);
        });
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u16string_shrink_to_fit(val, alloc);
        });
    }

    fn from_alloc(alloc: A) -> Self {
        Self {
            inner: L::new_in(alloc),
        }
    }
}

impl<A, L> fmt::Debug for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CxxUtf16String")
            .field("length", &self.inner.value_as_ref().size)
            .field("capacity", &self.inner.value_as_ref().res)
            .field("large_mode", &(self.inner.value_as_ref().res > 15))
            .finish()
    }
}

impl<A, L> AsRef<[u16]> for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn as_ref(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A, L> Borrow<[u16]> for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn borrow(&self) -> &[u16] {
        self.as_bytes()
    }
}

impl<A, L> Default for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy + Default,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn default() -> Self {
        Self::from_alloc(A::default())
    }
}

impl<A, L> Drop for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn drop(&mut self) {
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            CSTL_u16string_destroy(val, alloc);
        });
    }
}

impl<A, L> Clone for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy + Clone,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn clone(&self) -> Self {
        Self::from_bytes_in(self, self.inner.alloc_as_ref().clone())
    }
}

impl<A, L> Extend<u16> for CxxUtf16StringLayout<A, L>
where
    A: CxxProxy,
    L: WithCxxProxy<Alloc = A, Value = RawUtf16String>,
{
    fn extend<I: IntoIterator<Item = u16>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        self.inner.with_proxy_mut(|val, alloc| unsafe {
            for ch in iter {
                CSTL_u16string_append_char(val, 1, ch, alloc);
            }
        });
    }
}

const fn new_val() -> RawUtf16String {
    RawUtf16String {
        bx: cstl_sys::CSTL_UTF16StringUnion { buf: [0; 8] },
        size: 0,
        res: 7,
    }
}

impl<A: CxxProxy> WithCxxProxy for Layout<A> {
    type Value = RawUtf16String;
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
    use cstl_sys::CSTL_UTF16StringVal as RawUtf16String;

    use crate::alloc::{CxxProxy, WithCxxProxy};

    use super::{new_val, CxxUtf16StringLayout, SysAlloc};

    pub type CxxUtf16String<A = SysAlloc> = CxxUtf16StringLayout<A, Layout<A>>;

    #[repr(C)]
    pub struct Layout<A: CxxProxy> {
        val: RawUtf16String,
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

    impl CxxUtf16String<SysAlloc> {
        pub const fn new() -> Self {
            Self {
                inner: Layout::new_in(SysAlloc),
            }
        }
    }

    impl<A: CxxProxy> CxxUtf16String<A> {
        pub const fn new_in(alloc: A) -> Self {
            Self {
                inner: Layout::new_in(alloc),
            }
        }

        pub const fn allocator(&self) -> &A {
            &self.inner.alloc
        }
    }

    impl<A: CxxProxy> WithCxxProxy for Layout<A> {
        type Value = RawUtf16String;
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
