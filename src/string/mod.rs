mod narrow;
pub use narrow::CxxNarrowString;

mod wide;
pub use wide::CxxWideString;

mod utf8;
pub use utf8::CxxUtf8String;

mod utf16;
pub use utf16::CxxUtf16String;

mod utf32;
pub use utf32::CxxUtf32String;

#[cfg(feature = "msvc2012")]
pub mod msvc2012 {
    pub use super::narrow::msvc2012::CxxNarrowString;

    pub use super::wide::msvc2012::CxxWideString;

    pub use super::utf8::msvc2012::CxxUtf8String;

    pub use super::utf16::msvc2012::CxxUtf16String;

    pub use super::utf32::msvc2012::CxxUtf32String;
}
