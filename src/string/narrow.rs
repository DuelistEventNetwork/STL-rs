use std::alloc::System as SysAlloc;

use cstl_sys::CSTL_StringVal;

use crate::alloc::CxxProxy;

#[repr(C)]
pub struct CxxNarrowString<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_StringVal,
}

impl CxxNarrowString<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_StringVal {
                bx: cstl_sys::CSTL_StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}

impl<A: CxxProxy> CxxNarrowString<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_StringVal {
                bx: cstl_sys::CSTL_StringUnion { buf: [0; 16] },
                size: 0,
                res: 15,
            },
        }
    }
}
