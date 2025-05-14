use std::{
    alloc::System as SysAlloc,
    borrow::{Borrow, BorrowMut},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut, Index, IndexMut, Range},
    ptr::{self, NonNull},
    slice::{self, SliceIndex},
};

use cstl_sys::{
    CSTL_VectorVal, CSTL_vector_begin, CSTL_vector_clear, CSTL_vector_copy_assign,
    CSTL_vector_copy_assign_range, CSTL_vector_destroy, CSTL_vector_end, CSTL_vector_erase,
    CSTL_vector_iterator_add, CSTL_vector_iterator_eq, CSTL_vector_move_assign,
    CSTL_vector_move_assign_range, CSTL_vector_move_insert, CSTL_vector_move_push_back,
    CSTL_vector_pop_back, CSTL_vector_reserve, CSTL_vector_resize, CSTL_vector_shrink_to_fit,
    CSTL_vector_truncate,
};
use into_iter::IntoIter;

use crate::{
    alloc::{with_proxy, CxxProxy},
    semantics::{BaseType, CopyMoveType, CopyOnlyType, DefaultUninit, MoveType},
};

pub mod into_iter;

#[repr(C)]
pub struct CxxVec<T, A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_VectorVal,
    _marker: PhantomData<T>,
}

impl<T> CxxVec<T, SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: Self::new_val(),
            _marker: PhantomData,
        }
    }
}

impl<T, A: CxxProxy> CxxVec<T, A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: Self::new_val(),
            _marker: PhantomData,
        }
    }

    pub const fn allocator(&self) -> &A {
        &self.alloc
    }

    pub fn from_vec_in<A2: CxxProxy>(vec: CxxVec<T, A2>, alloc: A) -> Self {
        let mut new = CxxVec::new_in(alloc);
        let mut drained = vec;

        with_proxy(&drained.alloc, |old_alloc| {
            with_proxy(&new.alloc, |new_alloc| unsafe {
                let moved = CSTL_vector_move_assign(
                    &mut new.val,
                    <T as BaseType>::TYPE,
                    &<DefaultUninit<T> as MoveType>::MOVE,
                    &mut drained.val,
                    old_alloc,
                    new_alloc,
                    false,
                );

                if moved {
                    drained.val.last = drained.val.first;
                }
            });
        });

        new
    }

    pub fn into_vec_in<A2: CxxProxy>(self, alloc: A2) -> CxxVec<T, A2> {
        CxxVec::from_vec_in(self, alloc)
    }

    pub fn from_rust_vec_in(vec: Vec<T>, alloc: A) -> Self {
        let mut new = CxxVec::new_in(alloc);
        let mut drained = vec;

        with_proxy(&new.alloc, |alloc| unsafe {
            let Range { start, end } = drained.as_mut_ptr_range();

            let moved = CSTL_vector_move_assign_range(
                &mut new.val,
                <T as BaseType>::TYPE,
                &<DefaultUninit<T> as MoveType>::MOVE,
                start as _,
                end as _,
                alloc,
            );

            if moved {
                drained.set_len(0);
            }
        });

        new
    }

    pub fn into_rust_vec(self) -> Vec<T> {
        let mut new = Vec::new();
        let mut drained = self;

        unsafe {
            let left_uninit = slice::from_raw_parts_mut(
                drained.first_ptr_mut() as *mut DefaultUninit<T>,
                drained.len(),
            );

            new.extend(left_uninit.iter_mut().map(|v| mem::take(v).assume_init()));

            drained.val.last = drained.val.first;
        };

        new
    }

    pub fn from_slice_in(slice: &[T], alloc: A) -> Self
    where
        T: Clone,
    {
        let mut new = Self::new_in(alloc);

        new.reserve(slice.len());

        with_proxy(&new.alloc, |alloc| unsafe {
            let Range { start, end } = slice.as_ptr_range();

            CSTL_vector_copy_assign_range(
                &mut new.val,
                <T as BaseType>::TYPE,
                &<T as CopyOnlyType>::COPY,
                start as _,
                end as _,
                alloc,
            );
        });

        new
    }

    pub fn as_ptr(&self) -> *const T {
        if !self.first_ptr().is_null() {
            self.first_ptr()
        } else {
            ptr::dangling()
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        if !self.first_ptr_mut().is_null() {
            self.first_ptr_mut()
        } else {
            ptr::dangling_mut()
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }

    pub fn len(&self) -> usize {
        unsafe { self.last_ptr().offset_from(self.first_ptr()) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.first_ptr() == self.end_ptr()
    }

    pub fn capacity(&self) -> usize {
        unsafe { self.end_ptr().offset_from(self.first_ptr()) as usize }
    }

    pub fn push(&mut self, value: T) {
        with_proxy(&self.alloc, |alloc| unsafe {
            let mut value = DefaultUninit::new(value);

            let pushed = CSTL_vector_move_push_back(
                &mut self.val,
                <T as BaseType>::TYPE,
                &<DefaultUninit<T> as MoveType>::MOVE,
                value.as_mut_ptr() as _,
                alloc,
            );

            if !pushed {
                drop(value.assume_init());
            }
        });
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            unsafe {
                let last = self.last_ptr().offset(-1).read();

                CSTL_vector_pop_back(
                    &mut self.val,
                    <T as BaseType>::TYPE,
                    &<DefaultUninit<T> as BaseType>::DROP,
                );

                Some(last)
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let len = self.len();

        if index > len {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            let pos = CSTL_vector_iterator_add(
                CSTL_vector_begin(&self.val, <T as BaseType>::TYPE),
                index as isize,
            );

            let mut value = DefaultUninit::new(value);

            let inserted = CSTL_vector_move_insert(
                &mut self.val,
                &<DefaultUninit<T> as MoveType>::MOVE,
                pos,
                value.as_mut_ptr() as _,
                alloc,
            );

            if CSTL_vector_iterator_eq(inserted, CSTL_vector_end(&self.val, <T as BaseType>::TYPE))
            {
                drop(value.assume_init());
            }
        });
    }

    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();

        if index > len {
            panic!("removal index (is {index}) should be <= len (is {len})");
        }

        unsafe {
            let removed = self.first_ptr().offset(index as isize).read();

            let pos = CSTL_vector_iterator_add(
                CSTL_vector_begin(&self.val, <T as BaseType>::TYPE),
                index as isize,
            );

            CSTL_vector_erase(&mut self.val, &<DefaultUninit<T> as MoveType>::MOVE, pos);

            removed
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            CSTL_vector_clear(&mut self.val, &<T as BaseType>::DROP);
        }
    }

    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        if new_len > isize::MAX as usize {
            panic!("requested length ({new_len}) exceeded `isize::MAX`");
        }

        if new_len > self.len() {
            with_proxy(&self.alloc, |alloc| unsafe {
                CSTL_vector_resize(
                    &mut self.val,
                    <T as BaseType>::TYPE,
                    &<DefaultUninit<T> as CopyMoveType>::COPY,
                    new_len,
                    &value as *const T as _,
                    alloc,
                );
            });
        } else {
            self.truncate(new_len);
        }
    }

    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
        let len = self.len();

        if new_len > len {
            let additional = new_len - len;

            self.reserve(additional);

            for _ in 0..additional {
                self.push(f());
            }
        } else {
            self.truncate(new_len);
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len() {
            unsafe {
                CSTL_vector_truncate(
                    &mut self.val,
                    <T as BaseType>::TYPE,
                    &<T as BaseType>::DROP,
                    new_len,
                );
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();

        if isize::MAX as usize - capacity < additional {
            panic!("requested capacity ({capacity} + {additional}) overflowed `isize::MAX`");
        }

        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_vector_reserve(
                &mut self.val,
                <T as BaseType>::TYPE,
                &<DefaultUninit<T> as MoveType>::MOVE,
                capacity + additional,
                alloc,
            );
        });
    }

    pub fn shrink_to_fit(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_vector_shrink_to_fit(
                &mut self.val,
                <T as BaseType>::TYPE,
                &<DefaultUninit<T> as MoveType>::MOVE,
                alloc,
            );
        });
    }
}

impl<T, A: CxxProxy> CxxVec<T, A> {
    const fn new_val() -> CSTL_VectorVal {
        // Could be `CSTL_vector_construct`, but this is an easy optimization.
        CSTL_VectorVal {
            first: ptr::null_mut(),
            last: ptr::null_mut(),
            end: ptr::null_mut(),
        }
    }

    fn first_ptr(&self) -> *const T {
        self.val.first as _
    }

    fn last_ptr(&self) -> *const T {
        self.val.last as _
    }

    fn end_ptr(&self) -> *const T {
        self.val.end as _
    }

    fn first_ptr_mut(&mut self) -> *mut T {
        self.val.first as _
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
        Self {
            alloc: A::default(),
            val: Self::new_val(),
            _marker: PhantomData,
        }
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

impl<T, A> Clone for CxxVec<T, A>
where
    T: Clone + Sized,
    A: CxxProxy + Clone,
{
    fn clone(&self) -> Self {
        let mut new = Self::new_in(self.alloc.clone());

        with_proxy(&self.alloc, |old_alloc| {
            with_proxy(&new.alloc, |new_alloc| unsafe {
                CSTL_vector_copy_assign(
                    &mut new.val,
                    <T as BaseType>::TYPE,
                    &<T as CopyOnlyType>::COPY,
                    &self.val,
                    new_alloc,
                    old_alloc,
                    false,
                );
            });
        });

        new
    }
}

impl<T, A: CxxProxy> Drop for CxxVec<T, A> {
    fn drop(&mut self) {
        with_proxy(&self.alloc, |alloc| unsafe {
            CSTL_vector_destroy(
                &mut self.val,
                <T as BaseType>::TYPE,
                &<T as BaseType>::DROP,
                alloc,
            );
        });
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
    type IntoIter = IntoIter<T, A>;

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

            IntoIter {
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
