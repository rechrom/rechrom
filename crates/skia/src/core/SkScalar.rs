use crate::src::core::SkFloatingPoint::*;

// Source: include/core/SkScalar.h:16-16
pub type SkScalar = f32;

// Source: include/core/SkScalar.h:18-18
pub const SK_Scalar1: f32 = 1.0;

// Source: include/core/SkScalar.h:19-19
pub const SK_ScalarHalf: f32 = 0.5;

// Source: include/core/SkScalar.h:20-20
pub const SK_ScalarSqrt2: f32 = SK_FloatSqrt2;

// Source: include/core/SkScalar.h:21-21
pub const SK_ScalarPI: f32 = SK_FloatPI;

// Source: include/core/SkScalar.h:22-22
pub const SK_ScalarTanPIOver8: f32 = 0.414213562;

// Source: include/core/SkScalar.h:23-23
pub const SK_ScalarRoot2Over2: f32 = 0.707106781;

// Source: include/core/SkScalar.h:24-24
pub const SK_ScalarMax: f32 = 3.402823466e38;

// Source: include/core/SkScalar.h:25-25
pub const SK_ScalarMin: f32 = -SK_ScalarMax;

// Source: include/core/SkScalar.h:26-26
pub const SK_ScalarInfinity: f32 = SK_FloatInfinity;

// Source: include/core/SkScalar.h:27-27
pub const SK_ScalarNegativeInfinity: f32 = SK_FloatNegativeInfinity;

// Source: include/core/SkScalar.h:28-28
pub const SK_ScalarNaN: f32 = SK_FloatNaN;

// Source: include/core/SkScalar.h:30-30
#[inline(always)]
pub fn SkScalarFloorToScalar(x: f32) -> f32 {
    x.floor()
}

// Source: include/core/SkScalar.h:31-31
#[inline(always)]
pub fn SkScalarCeilToScalar(x: f32) -> f32 {
    x.ceil()
}

// Source: include/core/SkScalar.h:32-32
#[inline(always)]
pub fn SkScalarRoundToScalar(x: f32) -> f32 {
    sk_float_round(x)
}

// Source: include/core/SkScalar.h:33-33
#[inline(always)]
pub fn SkScalarTruncToScalar(x: f32) -> f32 {
    x.trunc()
}

// Source: include/core/SkScalar.h:35-35
#[inline(always)]
pub fn SkScalarFloorToInt(x: f32) -> i32 {
    sk_float_floor2int(x)
}

// Source: include/core/SkScalar.h:36-36
#[inline(always)]
pub fn SkScalarCeilToInt(x: f32) -> i32 {
    sk_float_ceil2int(x)
}

// Source: include/core/SkScalar.h:37-37
#[inline(always)]
pub fn SkScalarRoundToInt(x: f32) -> i32 {
    sk_float_round2int(x)
}

// Source: include/core/SkScalar.h:39-39
#[inline(always)]
pub fn SkScalarAbs(x: f32) -> f32 {
    x.abs()
}

// Source: include/core/SkScalar.h:40-40
#[inline(always)]
pub fn SkScalarCopySign(x: f32, y: f32) -> f32 {
    x.copysign(y)
}

// Source: include/core/SkScalar.h:41-41
#[inline(always)]
pub fn SkScalarMod(x: f32, y: f32) -> f32 {
    x % y
}

// Source: include/core/SkScalar.h:42-42
#[inline(always)]
pub fn SkScalarSqrt(x: f32) -> f32 {
    x.sqrt()
}

// Source: include/core/SkScalar.h:43-43
#[inline(always)]
pub fn SkScalarPow(x: f32, y: f32) -> f32 {
    x.powf(y)
}

// Source: include/core/SkScalar.h:45-45
#[inline(always)]
pub fn SkScalarSin(x: f32) -> f32 {
    x.sin()
}

// Source: include/core/SkScalar.h:46-46
#[inline(always)]
pub fn SkScalarCos(x: f32) -> f32 {
    x.cos()
}

// Source: include/core/SkScalar.h:47-47
#[inline(always)]
pub fn SkScalarTan(x: f32) -> f32 {
    x.tan()
}

// Source: include/core/SkScalar.h:48-48
#[inline(always)]
pub fn SkScalarASin(x: f32) -> f32 {
    x.asin()
}

// Source: include/core/SkScalar.h:49-49
#[inline(always)]
pub fn SkScalarACos(x: f32) -> f32 {
    x.acos()
}

// Source: include/core/SkScalar.h:50-50
#[inline(always)]
pub fn SkScalarATan2(x: f32, y: f32) -> f32 {
    x.atan2(y)
}

// Source: include/core/SkScalar.h:51-51
#[inline(always)]
pub fn SkScalarExp(x: f32) -> f32 {
    x.exp()
}

// Source: include/core/SkScalar.h:52-52
#[inline(always)]
pub fn SkScalarLog(x: f32) -> f32 {
    x.ln()
}

// Source: include/core/SkScalar.h:53-53
#[inline(always)]
pub fn SkScalarLog2(x: f32) -> f32 {
    x.log2()
}

// Source: include/core/SkScalar.h:57-57
#[inline(always)]
pub fn SkIntToScalar(x: i32) -> f32 {
    x as f32
}

// Source: include/core/SkScalar.h:58-58
#[inline(always)]
pub fn SkIntToFloat(x: i32) -> f32 {
    x as f32
}

// Source: include/core/SkScalar.h:59-59
#[inline(always)]
pub fn SkScalarTruncToInt(x: f32) -> i32 {
    sk_float_saturate2int(x)
}

// Source: include/core/SkScalar.h:61-61
#[inline(always)]
pub fn SkScalarToFloat(x: f32) -> f32 {
    x
}

// Source: include/core/SkScalar.h:62-62
#[inline(always)]
pub fn SkFloatToScalar(x: f32) -> f32 {
    x
}

// Source: include/core/SkScalar.h:63-63
#[inline(always)]
pub fn SkScalarToDouble(x: f32) -> f64 {
    x as f64
}

// Source: include/core/SkScalar.h:64-64
#[inline(always)]
pub fn SkDoubleToScalar(x: f64) -> f32 {
    sk_double_to_float(x)
}

// Source: include/core/SkScalar.h:67-69
#[inline]
pub fn SkScalarFraction(x: f32) -> f32 {
    x - x.trunc()
}

// Source: include/core/SkScalar.h:71-71
#[inline]
pub fn SkScalarSquare(x: f32) -> f32 {
    x * x
}

// Source: include/core/SkScalar.h:73-73
#[inline(always)]
pub fn SkScalarInvert(x: f32) -> f32 {
    SK_Scalar1 / x
}

// Source: include/core/SkScalar.h:74-74
#[inline(always)]
pub fn SkScalarAve(x: f32, y: f32) -> f32 {
    sk_float_midpoint(x, y)
}

// Source: include/core/SkScalar.h:76-76
#[inline(always)]
pub fn SkDegreesToRadians(x: f32) -> f32 {
    x * (SK_ScalarPI / 180.0)
}

// Source: include/core/SkScalar.h:77-77
#[inline(always)]
pub fn SkRadiansToDegrees(x: f32) -> f32 {
    x * (180.0 / SK_ScalarPI)
}

// Source: include/core/SkScalar.h:79-81
#[inline]
pub fn SkScalarIsInt(x: f32) -> bool {
    x == x.floor()
}

// Source: include/core/SkScalar.h:89-91
#[inline]
pub fn SkScalarSignAsInt(x: f32) -> i32 {
    if x < 0.0 {
        -1
    } else {
        (x > 0.0) as i32
    }
}

// Source: include/core/SkScalar.h:94-96
#[inline]
pub fn SkScalarSignAsScalar(x: f32) -> f32 {
    if x < 0.0 {
        -SK_Scalar1
    } else if x > 0.0 {
        SK_Scalar1
    } else {
        0.0
    }
}

// Source: include/core/SkScalar.h:98-98
pub const SK_ScalarNearlyZero: f32 = 1.0 / (1 << 12) as f32;

// Source: include/core/SkScalar.h:100-104
#[inline]
pub fn SkScalarNearlyZero(x: f32, tolerance: f32) -> bool {
    debug_assert!(tolerance >= 0.0);
    x.abs() <= tolerance
}

// Source: include/core/SkScalar.h:106-110
#[inline]
pub fn SkScalarNearlyEqual(x: f32, y: f32, tolerance: f32) -> bool {
    debug_assert!(tolerance >= 0.0);
    (x - y).abs() <= tolerance
}

// Source: include/core/SkScalar.h:112-112
pub const SK_ScalarSinCosNearlyZero: f32 = 1.0 / (1 << 16) as f32;

// Source: include/core/SkScalar.h:114-117
#[inline]
pub fn SkScalarSinSnapToZero(radians: f32) -> f32 {
    let v = radians.sin();
    if SkScalarNearlyZero(v, SK_ScalarSinCosNearlyZero) {
        0.0
    } else {
        v
    }
}

// Source: include/core/SkScalar.h:119-122
#[inline]
pub fn SkScalarCosSnapToZero(radians: f32) -> f32 {
    let v = radians.cos();
    if SkScalarNearlyZero(v, SK_ScalarSinCosNearlyZero) {
        0.0
    } else {
        v
    }
}

// Source: include/core/SkScalar.h:130-133
#[inline]
pub fn SkScalarInterp(a: f32, b: f32, t: f32) -> f32 {
    debug_assert!(t >= 0.0 && t <= SK_Scalar1);
    a + (b - a) * t
}

// Source: include/core/SkScalar.h:138-146
#[inline]
pub fn SkScalarsEqual(a: &[f32], b: &[f32]) -> bool {
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

// Source: src/core/SkScalar.cpp:11-37
pub fn SkFloatInterpFunc(searchKey: f32, keys: &[f32], values: &[f32], length: i32) -> f32 {
    crate::SkASSERT!(@source b"src/core/SkScalar.cpp\0".as_ptr(),12; @condition b"length > 0\0".as_ptr(); length > 0);
    let length = length as usize;
    debug_assert!(keys.len() >= length && values.len() >= length);
    #[cfg(sk_debug)]
    for i in 1..length {
        crate::SkASSERT!(@source b"src/core/SkScalar.cpp\0".as_ptr(),17; @condition b"keys[i-1] <= keys[i]\0".as_ptr(); keys[i - 1] <= keys[i]);
    }
    let mut right = 0;
    while right < length && keys[right] < searchKey {
        right += 1;
    }
    if right == length {
        return values[length - 1];
    }
    if right == 0 {
        return values[0];
    }
    let leftKey = keys[right - 1];
    let rightKey = keys[right];
    let fract = (searchKey - leftKey) / (rightKey - leftKey);
    SkScalarInterp(values[right - 1], values[right], fract)
}
