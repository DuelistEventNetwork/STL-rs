use std::alloc::System as SysAlloc;

use cstl_sys::CSTL_UTF32StringVal;

use crate::alloc::CxxProxy;

#[repr(C)]
pub struct CxxUtf32String<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_UTF32StringVal,
}

impl CxxUtf32String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_UTF32StringVal {
                bx: cstl_sys::CSTL_UTF32StringUnion { buf: [0; 4] },
                size: 0,
                res: 3,
            },
        }
    }
}

impl<A: CxxProxy> CxxUtf32String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_UTF32StringVal {
                bx: cstl_sys::CSTL_UTF32StringUnion { buf: [0; 4] },
                size: 0,
                res: 3,
            },
        }
    }
}
