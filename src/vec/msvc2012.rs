use std::{
    borrow::{Borrow, BorrowMut},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::{self, NonNull},
    slice::{self, SliceIndex},
};

use cstl_sys::{CSTL_VectorVal, CSTL_vector_copy_assign, CSTL_vector_destroy};

use crate::{
    alloc::{CxxProxy, WithCxxProxy},
    semantics::{BaseType, CopyOnlyType},
};

use super::{new_val, CxxVecWithProxy, SysAlloc};

#[repr(C)]
pub struct CxxVec<T, A: CxxProxy = SysAlloc> {
    pub(super) val: CSTL_VectorVal,
    pub(super) alloc: A,
    _marker: PhantomData<T>,
}

impl<T> CxxVec<T, SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: new_val(),
            _marker: PhantomData,
        }
    }
}

impl<T, A: CxxProxy> CxxVec<T, A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: new_val(),
            _marker: PhantomData,
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.alloc
    }
}

impl<T, A: CxxProxy> AsRef<CxxVec<T, A>> for CxxVec<T, A> {
    fn as_ref(&self) -> &CxxVec<T, A> {
        self
    }
}

impl<T, A: CxxProxy> AsRef<[T]> for CxxVec<T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, A: CxxProxy> AsMut<CxxVec<T, A>> for CxxVec<T, A> {
    fn as_mut(&mut self) -> &mut CxxVec<T, A> {
        self
    }
}

impl<T, A: CxxProxy> AsMut<[T]> for CxxVec<T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, A: CxxProxy> Borrow<[T]> for CxxVec<T, A> {
    fn borrow(&self) -> &[T] {
        &self[..]
    }
}

impl<T, A: CxxProxy> BorrowMut<[T]> for CxxVec<T, A> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self[..]
    }
}

impl<T, A> fmt::Debug for CxxVec<T, A>
where
    T: fmt::Debug,
    A: CxxProxy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, A> Default for CxxVec<T, A>
where
    A: CxxProxy + Default,
{
    fn default() -> Self {
        Self::new_in(A::default())
    }
}

impl<T, A: CxxProxy> Deref for CxxVec<T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, A: CxxProxy> DerefMut for CxxVec<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, A: CxxProxy> Drop for CxxVec<T, A> {
    fn drop(&mut self) {
        self.with_proxy_mut(|val, alloc| unsafe {
            CSTL_vector_destroy(val, <T as BaseType>::TYPE, &<T as BaseType>::DROP, alloc);
        });
    }
}

impl<T, A> Clone for CxxVec<T, A>
where
    T: Clone + Sized,
    A: CxxProxy + Clone,
{
    fn clone(&self) -> Self {
        let mut new = Self::new_in(self.alloc.clone());

        self.with_proxy(|old_val, old_alloc| {
            new.with_proxy_mut(|new_val, new_alloc| unsafe {
                CSTL_vector_copy_assign(
                    new_val,
                    <T as BaseType>::TYPE,
                    &<T as CopyOnlyType>::COPY,
                    old_val,
                    new_alloc,
                    old_alloc,
                    false,
                );
            });
        });

        new
    }
}

impl<T, I: SliceIndex<[T]>, A: CxxProxy> Index<I> for CxxVec<T, A> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, A: CxxProxy> IndexMut<I> for CxxVec<T, A> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T, A: CxxProxy> Extend<T> for CxxVec<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        iter.for_each(|e| self.push(e));
    }
}

impl<'a, T: Copy + 'a, A: CxxProxy> Extend<&'a T> for CxxVec<T, A> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied())
    }
}

impl<T: PartialEq, A1: CxxProxy, A2: CxxProxy> PartialEq<CxxVec<T, A2>> for CxxVec<T, A1> {
    fn eq(&self, other: &CxxVec<T, A2>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: PartialOrd, A1: CxxProxy, A2: CxxProxy> PartialOrd<CxxVec<T, A2>> for CxxVec<T, A1> {
    fn partial_cmp(&self, other: &CxxVec<T, A2>) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Eq, A: CxxProxy> Eq for CxxVec<T, A> {}

impl<T: Ord, A: CxxProxy> Ord for CxxVec<T, A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Hash, A: CxxProxy> Hash for CxxVec<T, A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, A: CxxProxy> IntoIterator for CxxVec<T, A> {
    type Item = T;
    type IntoIter = super::IntoIter<T, A>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            // Dropped by IntoIter:
            let mut vec = ManuallyDrop::new(self);
            let alloc = ManuallyDrop::new(ptr::read(vec.allocator()));

            let ptr = NonNull::new_unchecked(vec.as_mut_ptr());
            let end = ptr.add(vec.len());

            let val = CSTL_VectorVal {
                first: vec.val.first,
                last: vec.val.first,
                end: vec.val.end,
            };

            super::IntoIter {
                alloc,
                val,
                _marker: vec._marker,
                ptr,
                end,
            }
        }
    }
}

impl<'a, T, A: CxxProxy> IntoIterator for &'a CxxVec<T, A> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, A: CxxProxy> IntoIterator for &'a mut CxxVec<T, A> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

unsafe impl<T: Send, A: CxxProxy + Send> Send for CxxVec<T, A> {}

unsafe impl<T: Sync, A: CxxProxy + Sync> Sync for CxxVec<T, A> {}

impl<T, A: CxxProxy> WithCxxProxy<T> for CxxVec<T, A> {
    type Value = CSTL_VectorVal;
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
        Self::new_in(alloc)
    }
}
