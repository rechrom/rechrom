// Source: include/codec/SkCodecAnimation.h:19
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisposalMethod(pub core::ffi::c_int);
impl DisposalMethod {
    // Source: include/codec/SkCodecAnimation.h:25
    pub const kKeep: Self = Self(1);
    // Source: include/codec/SkCodecAnimation.h:32
    pub const kRestoreBGColor: Self = Self(2);
    // Source: include/codec/SkCodecAnimation.h:40
    pub const kRestorePrevious: Self = Self(3);
}
// Source: include/codec/SkCodecAnimation.h:46
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blend(pub core::ffi::c_int);
impl Blend {
    // Source: include/codec/SkCodecAnimation.h:50
    pub const kSrcOver: Self = Self(0);
    // Source: include/codec/SkCodecAnimation.h:57
    pub const kSrc: Self = Self(1);
}
