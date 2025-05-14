use std::alloc::System as SysAlloc;

use cstl_sys::CSTL_UTF8StringVal;

use crate::alloc::CxxProxy;

#[repr(C)]
pub struct CxxUtf8String<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_UTF8StringVal,
}

impl CxxUtf8String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_UTF8StringVal {
                bx: cstl_sys::CSTL_UTF8StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}

impl<A: CxxProxy> CxxUtf8String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_UTF8StringVal {
                bx: cstl_sys::CSTL_UTF8StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}
