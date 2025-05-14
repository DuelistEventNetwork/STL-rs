use std::alloc::System as SysAlloc;

use cstl_sys::CSTL_UTF16StringVal;

use crate::alloc::CxxProxy;

#[repr(C)]
pub struct CxxUtf16String<A: CxxProxy = SysAlloc> {
    alloc: A,
    val: CSTL_UTF16StringVal,
}

impl CxxUtf16String<SysAlloc> {
    pub const fn new() -> Self {
        Self {
            alloc: SysAlloc,
            val: CSTL_UTF16StringVal {
                bx: cstl_sys::CSTL_UTF16StringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }
}

impl<A: CxxProxy> CxxUtf16String<A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            val: CSTL_UTF16StringVal {
                bx: cstl_sys::CSTL_UTF16StringUnion { buf: [0; 8] },
                size: 0,
                res: 7,
            },
        }
    }
}
