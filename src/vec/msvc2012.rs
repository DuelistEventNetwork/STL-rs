use std::marker::PhantomData;

use cstl_sys::CSTL_VectorVal as RawVec;

use crate::alloc::{CxxProxy, WithCxxProxy};

use super::{new_val, CxxVecLayout, SysAlloc};

pub type CxxVec<T, A = SysAlloc> = CxxVecLayout<T, A, Layout<A>>;

#[repr(C)]
pub struct Layout<A: CxxProxy> {
    val: RawVec,
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

impl<T> CxxVec<T, SysAlloc> {
    pub const fn new() -> Self {
        Self {
            inner: Layout::new_in(SysAlloc),
            _marker: PhantomData,
        }
    }
}

impl<T, A: CxxProxy> CxxVec<T, A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Layout::new_in(alloc),
            _marker: PhantomData,
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.inner.alloc
    }
}

impl<A: CxxProxy> WithCxxProxy for Layout<A> {
    type Value = RawVec;
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
