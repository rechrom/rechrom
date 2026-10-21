use crate::include::core::SkPathTypes::SkPathDirection;

// Source: src/core/SkPathEnums.h:17-25
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkPathConvexity {
    kConvex_CW = 0,
    kConvex_CCW = 1,
    kConvex_Degenerate = 2,
    kConcave = 3,
    kUnknown = 4,
}

// Source: src/core/SkPathEnums.h:27-30
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkResolveConvexity {
    kNo,
    kYes,
}

// Source: src/core/SkPathEnums.h:32-36
#[inline]
pub fn SkPathConvexity_IsConvex(cv: SkPathConvexity) -> bool {
    cv == SkPathConvexity::kConvex_CW
        || cv == SkPathConvexity::kConvex_CCW
        || cv == SkPathConvexity::kConvex_Degenerate
}

// Source: src/core/SkPathEnums.h:38-46
#[inline]
pub fn SkPathConvexity_OppositeConvexDirection(cv: SkPathConvexity) -> SkPathConvexity {
    crate::SkASSERT!(@source b"src/core/SkPathEnums.h\0".as_ptr(),39; @condition b"SkPathConvexity_IsConvex(cv)\0".as_ptr(); SkPathConvexity_IsConvex(cv));
    match cv {
        SkPathConvexity::kConvex_CW => SkPathConvexity::kConvex_CCW,
        SkPathConvexity::kConvex_CCW => SkPathConvexity::kConvex_CW,
        _ => cv,
    }
}

// Source: src/core/SkPathEnums.h:48-52
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathFirstDirection {
    kCW,
    kCCW,
    kUnknown,
}

// Source: src/core/SkPathEnums.h:54-60
#[inline]
pub fn SkPathDirection_ToConvexity(dir: SkPathDirection) -> SkPathConvexity {
    match dir {
        SkPathDirection::kCW => SkPathConvexity::kConvex_CW,
        SkPathDirection::kCCW => SkPathConvexity::kConvex_CCW,
    }
}

// Source: src/core/SkPathEnums.h:62-69
#[inline]
pub fn SkPathFirstDirection_ToConvexity(dir: SkPathFirstDirection) -> SkPathConvexity {
    match dir {
        SkPathFirstDirection::kCW => SkPathConvexity::kConvex_CW,
        SkPathFirstDirection::kCCW => SkPathConvexity::kConvex_CCW,
        SkPathFirstDirection::kUnknown => SkPathConvexity::kConvex_Degenerate,
    }
}

// Source: src/core/SkPathEnums.h:71-79
#[inline]
pub fn SkPathConvexity_ToDirection(cv: SkPathConvexity) -> Option<SkPathDirection> {
    if cv == SkPathConvexity::kConvex_CW {
        return Some(SkPathDirection::kCW);
    }
    if cv == SkPathConvexity::kConvex_CCW {
        return Some(SkPathDirection::kCCW);
    }
    None
}

// Source: src/core/SkPathEnums.h:81-89
#[inline]
pub fn SkPathConvexity_ToFirstDirection(cv: SkPathConvexity) -> SkPathFirstDirection {
    if cv == SkPathConvexity::kConvex_CW {
        return SkPathFirstDirection::kCW;
    }
    if cv == SkPathConvexity::kConvex_CCW {
        return SkPathFirstDirection::kCCW;
    }
    SkPathFirstDirection::kUnknown
}

// Source: src/core/SkPathEnums.h:91-94
#[inline]
pub fn SkPathDirectionToFirst(dir: SkPathDirection) -> SkPathFirstDirection {
    if dir == SkPathDirection::kCW {
        SkPathFirstDirection::kCW
    } else {
        SkPathFirstDirection::kCCW
    }
}
