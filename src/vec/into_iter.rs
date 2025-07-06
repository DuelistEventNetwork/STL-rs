use std::{
    alloc::System as SysAlloc,
    iter::FusedIterator,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
    slice,
};

use cstl_sys::CSTL_VectorVal as RawVec;

use crate::{
    alloc::CxxProxy,
    vec::{CxxVec, Layout},
};

pub struct IntoIter<T, A: CxxProxy = SysAlloc> {
    pub(super) alloc: ManuallyDrop<A>,
    pub(super) val: RawVec,
    pub(super) _marker: PhantomData<T>,
    pub(super) ptr: NonNull<T>,
    pub(super) end: NonNull<T>,
}

impl<T, A: CxxProxy> IntoIter<T, A> {
    pub fn allocator(&self) -> &A {
        &self.alloc
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()) }
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len())
    }
}

impl<T, A: CxxProxy> AsRef<[T]> for IntoIter<T, A> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, A: CxxProxy> AsMut<[T]> for IntoIter<T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, A: CxxProxy> Iterator for IntoIter<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                let old = self.ptr;
                self.ptr = old.add(1);
                Some(old.read())
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = unsafe { self.end.offset_from(self.ptr) as usize };
        (left, Some(left))
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl<T, A: CxxProxy> DoubleEndedIterator for IntoIter<T, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                self.end = self.end.sub(1);
                Some(self.end.read())
            }
        } else {
            None
        }
    }
}

impl<T, A: CxxProxy> ExactSizeIterator for IntoIter<T, A> {}

impl<T, A: CxxProxy> FusedIterator for IntoIter<T, A> {}

impl<T, A: CxxProxy + Default> Default for IntoIter<T, A> {
    fn default() -> Self {
        super::CxxVec::default().into_iter()
    }
}

impl<T: Clone, A: CxxProxy + Clone> Clone for IntoIter<T, A> {
    fn clone(&self) -> Self {
        super::CxxVec::from_slice_in(self.as_slice(), A::clone(&self.alloc)).into_iter()
    }
}

impl<T, A: CxxProxy> Drop for IntoIter<T, A> {
    fn drop(&mut self) {
        struct DropGuard<'a, T, A: CxxProxy>(&'a mut IntoIter<T, A>);

        impl<T, A: CxxProxy> Drop for DropGuard<'_, T, A> {
            fn drop(&mut self) {
                unsafe {
                    let _ = CxxVec::<T, _> {
                        inner: Layout {
                            alloc: ManuallyDrop::take(&mut self.0.alloc),
                            val: self.0.val,
                        },
                        _marker: PhantomData,
                    };
                }
            }
        }

        let guard = DropGuard(self);

        unsafe {
            ptr::drop_in_place(guard.0.as_raw_mut_slice());
        }
    }
}

unsafe impl<T: Send, A: CxxProxy + Send> Send for IntoIter<T, A> {}

unsafe impl<T: Sync, A: CxxProxy + Sync> Sync for IntoIter<T, A> {}
