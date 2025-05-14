use std::alloc::System as SysAlloc;

use cstl_sys::CSTL_WideStringVal;

use crate::alloc::CxxProxy;

#[repr(C)]
pub struct CxxWideString<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_WideStringVal,
}

impl CxxWideString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_WideStringVal {
                bx: cstl_sys::CSTL_WideStringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }
}

impl<A: CxxProxy> CxxWideString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_WideStringVal {
                bx: cstl_sys::CSTL_WideStringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }
}
