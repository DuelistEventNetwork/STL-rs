use std::{
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

use cstl_sys::{CSTL_CopyType, CSTL_DropType, CSTL_MoveType, CSTL_Type};

pub trait BaseType: Sized {
    const TYPE: CSTL_Type = if Self::SIZE & Self::ALIGN == 0 {
        !Self::SIZE | Self::ALIGN + 1
    } else {
        Self::SIZE
    } as CSTL_Type;

    const SIZE: usize = if mem::size_of::<Self>() != 0 {
        mem::size_of::<Self>()
    } else {
        1
    };

    const ALIGN: usize = if mem::align_of::<Self>() != 0 {
        mem::align_of::<Self>()
    } else {
        1
    };

    const DROP: CSTL_DropType = CSTL_DropType {
        drop: unsafe { Some(mem::transmute(Self::raw_drop as *const ())) },
    };

    unsafe extern "C" fn raw_drop(first: NonNull<Self>, last: NonNull<Self>) {
        unsafe {
            let len = last
                .offset_from(first)
                .try_into()
                .expect("`first` > `last`");

            ptr::slice_from_raw_parts_mut(first.as_ptr(), len).drop_in_place();
        }
    }
}

impl<T> BaseType for T {}

pub trait MoveType: Default + Sized {
    const MOVE: CSTL_MoveType = CSTL_MoveType {
        drop_type: <Self as BaseType>::DROP,
        move_: unsafe { Some(mem::transmute(Self::raw_move as *const ())) },
    };

    unsafe extern "C" fn raw_move(first: NonNull<Self>, last: NonNull<Self>, dest: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                dest.offset(i).write(mem::take(first.offset(i).as_mut()));
            }
        }
    }
}

impl<T: Default> MoveType for T {}

pub trait CopyMoveType: Clone + Default + Sized {
    const COPY: CSTL_CopyType = CSTL_CopyType {
        move_type: <Self as MoveType>::MOVE,
        copy: unsafe { Some(mem::transmute(Self::raw_copy as *const ())) },
        fill: unsafe { Some(mem::transmute(Self::raw_fill as *const ())) },
    };

    unsafe extern "C" fn raw_copy(first: NonNull<Self>, last: NonNull<Self>, dest: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                dest.offset(i).write(first.offset(i).as_ref().clone());
            }
        }
    }

    unsafe extern "C" fn raw_fill(first: NonNull<Self>, last: NonNull<Self>, value: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                first.offset(i).write(value.as_ref().clone());
            }
        }
    }
}

impl<T: Clone + Default> CopyMoveType for T {}

pub trait CopyOnlyType: Clone + Sized {
    const COPY: CSTL_CopyType = CSTL_CopyType {
        move_type: CSTL_MoveType {
            drop_type: <Self as BaseType>::DROP,
            move_: unsafe { Some(mem::transmute(Self::raw_move as *const ())) },
        },
        copy: unsafe { Some(mem::transmute(Self::raw_copy as *const ())) },
        fill: unsafe { Some(mem::transmute(Self::raw_fill as *const ())) },
    };

    unsafe extern "C" fn raw_move(first: NonNull<Self>, last: NonNull<Self>, dest: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                dest.offset(i).write(first.offset(i).as_ref().clone());
            }
        }
    }

    unsafe extern "C" fn raw_copy(first: NonNull<Self>, last: NonNull<Self>, dest: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                dest.offset(i).write(first.offset(i).as_ref().clone());
            }
        }
    }

    unsafe extern "C" fn raw_fill(first: NonNull<Self>, last: NonNull<Self>, value: NonNull<Self>) {
        unsafe {
            for i in 0..last.offset_from(first) {
                first.offset(i).write(value.as_ref().clone());
            }
        }
    }
}

impl<T: Clone> CopyOnlyType for T {}

pub(crate) struct DefaultUninit<T>(MaybeUninit<T>);

impl<T> DefaultUninit<T> {
    pub const fn new(val: T) -> Self {
        Self(MaybeUninit::new(val))
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    pub const unsafe fn assume_init(self) -> T {
        self.0.assume_init()
    }
}

impl<T: Clone> Clone for DefaultUninit<T> {
    fn clone(&self) -> Self {
        unsafe { Self((&self.0 as *const MaybeUninit<T>).read()) }
    }
}

impl<T> Default for DefaultUninit<T> {
    fn default() -> Self {
        Self(MaybeUninit::uninit())
    }
}
