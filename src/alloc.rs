use std::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

use cstl_sys::CSTL_Alloc;

pub trait CxxProxy {
    fn proxy<'a>(&'a self) -> impl GlobalAlloc + 'a;
}

impl<A: GlobalAlloc + Clone> CxxProxy for A {
    fn proxy<'a>(&'a self) -> impl GlobalAlloc + 'a {
        self.clone()
    }
}

pub(crate) fn with_proxy<'a, T, R, F>(alloc: &'a T, f: F) -> R
where
    T: CxxProxy,
    F: FnOnce(&mut CSTL_Alloc) -> R
{
    let mut proxy_alloc = alloc.proxy();
    let mut raw_alloc = RawAlloc::from_ref_mut(&mut proxy_alloc);
    f(&mut raw_alloc.base)
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
