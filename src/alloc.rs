//! C++ allocation interface.
//!
//! Types that implement either [`CxxProxy`] or [`GlobalAlloc`] + [`Clone`]
//! can be used as C++ compatible allocators.

use std::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

use cstl_sys::CSTL_Alloc;

/// Trait for types that can spawn an opaque allocator instance from itself
/// via [`CxxProxy::proxy`].
///
/// Types that implement [`CxxProxy`] can be used as C++ compatible allocators.
pub trait CxxProxy {
    fn proxy<'a>(&self) -> impl GlobalAlloc + 'a
    where
        Self: 'a;
}

impl<A: GlobalAlloc + Clone> CxxProxy for A {
    fn proxy<'a>(&self) -> impl GlobalAlloc + 'a
    where
        Self: 'a,
    {
        self.clone()
    }
}

#[doc(hidden)]
pub trait WithCxxProxy: Sized {
    type Value;
    type Alloc: CxxProxy;

    fn value_as_ref(&self) -> &Self::Value;

    fn value_as_mut(&mut self) -> &mut Self::Value;

    fn alloc_as_ref(&self) -> &Self::Alloc;

    fn new_in(alloc: Self::Alloc) -> Self;

    #[inline]
    fn with_proxy<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Value, &mut CSTL_Alloc) -> R,
    {
        let mut proxy_alloc = self.alloc_as_ref().proxy();
        let mut raw_alloc = RawAlloc::from_ref_mut(&mut proxy_alloc);
        f(self.value_as_ref(), &mut raw_alloc.base)
    }

    #[inline]
    fn with_proxy_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self::Value, &mut CSTL_Alloc) -> R,
    {
        let mut proxy_alloc = self.alloc_as_ref().proxy();
        let mut raw_alloc = RawAlloc::from_ref_mut(&mut proxy_alloc);
        f(self.value_as_mut(), &mut raw_alloc.base)
    }
}

struct RawAlloc<'a, A>
where
    A: GlobalAlloc + 'a,
{
    base: CSTL_Alloc,
    _marker: PhantomData<&'a mut A>,
}

impl<'a, A> RawAlloc<'a, A>
where
    A: GlobalAlloc + 'a,
{
    #[inline]
    fn from_ref_mut(value: &'a mut A) -> Self {
        Self {
            base: CSTL_Alloc {
                opaque: value as *mut A as _,
                aligned_alloc: Some(Self::RAW_ALLOC_PTR),
                aligned_free: Some(Self::RAW_FREE_PTR),
            },
            _marker: PhantomData,
        }
    }

    const RAW_ALLOC_PTR: unsafe extern "C" fn(*mut c_void, usize, usize) -> *mut c_void =
        unsafe { mem::transmute(Self::raw_alloc as *const ()) };

    const RAW_FREE_PTR: unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize) =
        unsafe { mem::transmute(Self::raw_free as *const ()) };

    unsafe extern "C" fn raw_alloc(
        opaque: NonNull<A>,
        size: usize,
        alignment: usize,
    ) -> *mut c_void {
        unsafe {
            let alloc = opaque.as_ref();
            let layout =
                Layout::from_size_align(size, alignment).expect("bad layout passed from CSTL");
            alloc.alloc(layout) as _
        }
    }

    unsafe extern "C" fn raw_free(opaque: NonNull<A>, ptr: *mut u8, size: usize, alignment: usize) {
        unsafe {
            if !ptr.is_null() {
                let alloc = opaque.as_ref();
                let layout =
                    Layout::from_size_align(size, alignment).expect("bad layout passed from CSTL");
                alloc.dealloc(ptr, layout);
            }
        }
    }
}
