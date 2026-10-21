// Copyright 2018 Google Inc. BSD-style license.
use crate::modules::skcms::src::skcms_public::{
    skcms_AlphaFormat, skcms_Curve, skcms_ICCProfile, skcms_Matrix3x3, skcms_PixelFormat,
    skcms_TFType, skcms_TransferFunction,
};
use skcms_TFType::*;
// Source: modules/skcms/skcms.cc:42-42
const SKCMS_MAX_GRID_POINTS: u64 = 350_000_000;
// Source: modules/skcms/skcms.cc:44-48
// Atomic storage is the Rust data-race adaptation of the original global flag;
// on the selected ARM64 configuration cpu_type takes its native baseline arm.
static sAllowRuntimeCPUDetection: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(true);
pub fn skcms_DisableRuntimeCPUDetection() {
    sAllowRuntimeCPUDetection.store(false, core::sync::atomic::Ordering::Relaxed);
}
// Source: modules/skcms/skcms.cc:50-50
// Source: modules/skcms/skcms.cc:52-53
// Source: modules/skcms/skcms.cc:55-55
// Source: modules/skcms/skcms.cc:58-60
// Source: modules/skcms/skcms.cc:62-64
fn log2f_(x: f32) -> f32 {
    let bits = x.to_bits() as i32;
    let e = bits as f32 * (1.0 / ((1 << 23) as f32));
    let m = f32::from_bits(((bits & 0x007fffff) | 0x3f000000) as u32);
    e - 124.225514990_f32 - 1.498030302_f32 * m - 1.725879990_f32 / (0.3520887068_f32 + m)
}
// Source: modules/skcms/skcms.cc:65-68
fn logf_(x: f32) -> f32 {
    let ln2 = 0.69314718_f32;
    ln2 * log2f_(x)
}
// Source: modules/skcms/skcms.cc:71-77
// Source: modules/skcms/skcms.cc:79-81
// Source: modules/skcms/skcms.cc:87-91
// Source: modules/skcms/skcms.cc:93-96
fn exp2f_(mut x: f32) -> f32 {
    // INFINITY_ is the native IEEE positive infinity constant.
    if x > 128.0 {
        return f32::INFINITY;
    } else if x < -127.0 {
        return 0.0;
    }
    let fract = x - crate::modules::skcms::src::skcms_internals::floorf_(x);
    let fbits = (1.0 * ((1 << 23) as f32))
        * (x + 121.274057500_f32 - 1.490129070_f32 * fract
            + 27.728023300_f32 / (4.84252568_f32 - fract));
    if fbits >= i32::MAX as f32 {
        return f32::INFINITY;
    } else if fbits < 0.0 {
        return 0.0;
    }
    let bits = fbits as i32;
    x = f32::from_bits(bits as u32);
    x
}
// Source: modules/skcms/skcms.cc:99-107
pub fn powf_(x: f32, y: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    exp2f_(log2f_(x) * y)
}
// Source: modules/skcms/skcms.cc:109-112
fn expf_(x: f32) -> f32 {
    let log2_e = 1.4426950408889634074_f32;
    exp2f_(log2_e * x)
}
// Source: modules/skcms/skcms.cc:114-115
fn fmaxf_(x: f32, y: f32) -> f32 {
    if x > y {
        x
    } else {
        y
    }
}
fn fminf_(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}
// Source: modules/skcms/skcms.cc:117-117
fn isfinitef_(x: f32) -> bool {
    0.0 == x * 0.0
}
// Source: modules/skcms/skcms.cc:119-126
fn minus_1_ulp(x: f32) -> f32 {
    f32::from_bits(((x.to_bits() as i32) - 1) as u32)
}
// Source: modules/skcms/skcms.cc:130-131
struct TF_PQish {
    A: f32,
    B: f32,
    C: f32,
    D: f32,
    E: f32,
    F: f32,
}
struct TF_HLGish {
    R: f32,
    G: f32,
    a: f32,
    b: f32,
    c: f32,
    K_minus_1: f32,
}
// Source: modules/skcms/skcms.cc:136-136
// Source: modules/skcms/skcms.cc:138-139
// Source: modules/skcms/skcms.cc:141-143
// Source: modules/skcms/skcms.cc:145-148
// Source: modules/skcms/skcms.cc:150-152
// Source: modules/skcms/skcms.cc:154-182
// Source: modules/skcms/skcms.cc:185-185
// Source: modules/skcms/skcms.cc:187-190
// Source: modules/skcms/skcms.cc:192-194
// Source: modules/skcms/skcms.cc:196-197
fn TFKind_marker(kind: skcms_TFType) -> f32 {
    -(kind as i32 as f32)
}
fn classify(
    tf: &skcms_TransferFunction,
    pq: Option<&mut TF_PQish>,
    hlg: Option<&mut TF_HLGish>,
) -> skcms_TFType {
    if tf.g < 0.0 {
        if tf.g < -128.0 {
            return skcms_TFType_Invalid;
        }
        let enum_g = -(tf.g as i32);
        if (-enum_g) as f32 != tf.g {
            return skcms_TFType_Invalid;
        }
        match enum_g {
            2 => {
                if let Some(pq) = pq {
                    *pq = TF_PQish {
                        A: tf.a,
                        B: tf.b,
                        C: tf.c,
                        D: tf.d,
                        E: tf.e,
                        F: tf.f,
                    };
                }
                return skcms_TFType_PQish;
            }
            3 => {
                if let Some(hlg) = hlg {
                    *hlg = TF_HLGish {
                        R: tf.a,
                        G: tf.b,
                        a: tf.c,
                        b: tf.d,
                        c: tf.e,
                        K_minus_1: tf.f,
                    };
                }
                return skcms_TFType_HLGish;
            }
            4 => {
                if let Some(hlg) = hlg {
                    *hlg = TF_HLGish {
                        R: tf.a,
                        G: tf.b,
                        a: tf.c,
                        b: tf.d,
                        c: tf.e,
                        K_minus_1: tf.f,
                    };
                }
                return skcms_TFType_HLGinvish;
            }
            5 => {
                if tf.b != 0.0 || tf.c != 0.0 || tf.d != 0.0 || tf.e != 0.0 || tf.f != 0.0 {
                    return skcms_TFType_Invalid;
                }
                return skcms_TFType_PQ;
            }
            6 => {
                if tf.d != 0.0 || tf.e != 0.0 || tf.f != 0.0 {
                    return skcms_TFType_Invalid;
                }
                return skcms_TFType_HLG;
            }
            _ => {}
        }
        return skcms_TFType_Invalid;
    }
    if isfinitef_(tf.a + tf.b + tf.c + tf.d + tf.e + tf.f + tf.g)
        && tf.a >= 0.0
        && tf.c >= 0.0
        && tf.d >= 0.0
        && tf.g >= 0.0
        && tf.a * tf.d + tf.b >= 0.0
    {
        return skcms_TFType_sRGBish;
    }
    skcms_TFType_Invalid
}
// Source: modules/skcms/skcms.cc:199-216
pub fn skcms_TransferFunction_getType(tf: &skcms_TransferFunction) -> skcms_TFType {
    classify(tf, None, None)
}
pub fn skcms_TransferFunction_isSRGBish(tf: &skcms_TransferFunction) -> bool {
    classify(tf, None, None) == skcms_TFType_sRGBish
}
pub fn skcms_TransferFunction_isPQish(tf: &skcms_TransferFunction) -> bool {
    classify(tf, None, None) == skcms_TFType_PQish
}
pub fn skcms_TransferFunction_isHLGish(tf: &skcms_TransferFunction) -> bool {
    classify(tf, None, None) == skcms_TFType_HLGish
}
pub fn skcms_TransferFunction_isPQ(tf: &skcms_TransferFunction) -> bool {
    classify(tf, None, None) == skcms_TFType_PQ
}
pub fn skcms_TransferFunction_isHLG(tf: &skcms_TransferFunction) -> bool {
    classify(tf, None, None) == skcms_TFType_HLG
}
// Source: modules/skcms/skcms.cc:218-224
// Source: modules/skcms/skcms.cc:226-232
// Source: modules/skcms/skcms.cc:234-241
// Source: modules/skcms/skcms.cc:243-254
pub fn skcms_TransferFunction_makePQish(
    tf: &mut skcms_TransferFunction,
    A: f32,
    B: f32,
    C: f32,
    D: f32,
    E: f32,
    F: f32,
) -> bool {
    *tf = skcms_TransferFunction {
        g: TFKind_marker(skcms_TFType_PQish),
        a: A,
        b: B,
        c: C,
        d: D,
        e: E,
        f: F,
    };
    debug_assert!(skcms_TransferFunction_isPQish(tf));
    true
}
pub fn skcms_TransferFunction_makeScaledHLGish(
    tf: &mut skcms_TransferFunction,
    K: f32,
    R: f32,
    G: f32,
    a: f32,
    b: f32,
    c: f32,
) -> bool {
    *tf = skcms_TransferFunction {
        g: TFKind_marker(skcms_TFType_HLGish),
        a: R,
        b: G,
        c: a,
        d: b,
        e: c,
        f: K - 1.0,
    };
    debug_assert!(skcms_TransferFunction_isHLGish(tf));
    true
}
pub fn skcms_TransferFunction_makePQ(
    tf: &mut skcms_TransferFunction,
    hdr_reference_white_luminance: f32,
) {
    *tf = skcms_TransferFunction {
        g: TFKind_marker(skcms_TFType_PQ),
        a: hdr_reference_white_luminance,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    debug_assert!(skcms_TransferFunction_isPQ(tf));
}
pub fn skcms_TransferFunction_makeHLG(
    tf: &mut skcms_TransferFunction,
    hdr_reference_white_luminance: f32,
    peak_luminance: f32,
    system_gamma: f32,
) {
    *tf = skcms_TransferFunction {
        g: TFKind_marker(skcms_TFType_HLG),
        a: hdr_reference_white_luminance,
        b: peak_luminance,
        c: system_gamma,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    debug_assert!(skcms_TransferFunction_isHLG(tf));
}
// Source: modules/skcms/skcms.cc:256-258
// Source: modules/skcms/skcms.cc:260-263
// Source: modules/skcms/skcms.cc:265-270
// Source: modules/skcms/skcms.cc:272-276
// Source: modules/skcms/skcms.cc:279-284
// Source: modules/skcms/skcms.cc:286-288
// Source: modules/skcms/skcms.cc:290-298
// Source: modules/skcms/skcms.cc:300-305
pub fn skcms_TransferFunction_eval(tf: &skcms_TransferFunction, mut x: f32) -> f32 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    x *= sign;
    let mut pq = TF_PQish {
        A: 0.0,
        B: 0.0,
        C: 0.0,
        D: 0.0,
        E: 0.0,
        F: 0.0,
    };
    let mut hlg = TF_HLGish {
        R: 0.0,
        G: 0.0,
        a: 0.0,
        b: 0.0,
        c: 0.0,
        K_minus_1: 0.0,
    };
    match classify(tf, Some(&mut pq), Some(&mut hlg)) {
        skcms_TFType_Invalid => {}
        skcms_TFType_HLG => {
            let a = 0.17883277;
            let b = 0.28466892;
            let c = 0.55991073;
            return sign
                * if x <= 0.5 {
                    x * x / 3.0
                } else {
                    (expf_((x - c) / a) + b) / 12.0
                };
        }
        skcms_TFType_HLGish => {
            let K = hlg.K_minus_1 + 1.0;
            return K
                * sign
                * if x * hlg.R <= 1.0 {
                    powf_(x * hlg.R, hlg.G)
                } else {
                    expf_((x - hlg.c) * hlg.a) + hlg.b
                };
        }
        skcms_TFType_HLGinvish => {
            let K = hlg.K_minus_1 + 1.0;
            x /= K;
            return sign
                * if x <= 1.0 {
                    hlg.R * powf_(x, hlg.G)
                } else {
                    hlg.a * logf_(x - hlg.b) + hlg.c
                };
        }
        skcms_TFType_sRGBish => {
            return sign
                * if x < tf.d {
                    tf.c * x + tf.f
                } else {
                    powf_(tf.a * x + tf.b, tf.g) + tf.e
                }
        }
        skcms_TFType_PQ => {
            let c1 = 107.0 / 128.0;
            let c2 = 2413.0 / 128.0;
            let c3 = 2392.0 / 128.0;
            let m1 = 1305.0 / 8192.0;
            let m2 = 2523.0 / 32.0;
            let p = powf_(x, 1.0 / m2);
            return powf_((p - c1) / (c2 - c3 * p), 1.0 / m1);
        }
        skcms_TFType_PQish => {
            return sign
                * powf_(
                    (pq.A + pq.B * powf_(x, pq.C)) / (pq.D + pq.E * powf_(x, pq.C)),
                    pq.F,
                )
        }
    }
    0.0
}
// Source: modules/skcms/skcms.cc:308-332
// Borrowed ICC table storage must remain readable through table_entries for this call.
unsafe fn eval_curve(curve: &crate::modules::skcms::src::skcms_public::skcms_Curve, x: f32) -> f32 {
    let entries = unsafe { curve.table.table_entries };
    if entries == 0 {
        return skcms_TransferFunction_eval(unsafe { &curve.parametric.parametric }, x);
    }
    let ix = fmaxf_(0.0, fminf_(x, 1.0)) * (entries - 1) as f32;
    let lo = ix as usize;
    let hi = minus_1_ulp(ix + 1.0) as usize;
    let t = ix - lo as f32;
    let table = unsafe { curve.table };
    let (l, h) = if !table.table_8.is_null() {
        (
            unsafe { *table.table_8.add(lo) } as f32 * (1.0 / 255.0),
            unsafe { *table.table_8.add(hi) } as f32 * (1.0 / 255.0),
        )
    } else {
        let be_l = unsafe { table.table_16.add(2 * lo).cast::<u16>().read_unaligned() };
        let be_h = unsafe { table.table_16.add(2 * hi).cast::<u16>().read_unaligned() };
        let le_l = be_l.swap_bytes();
        let le_h = be_h.swap_bytes();
        (le_l as f32 * (1.0 / 65535.0), le_h as f32 * (1.0 / 65535.0))
    };
    l + (h - l) * t
}
// Source: modules/skcms/skcms.cc:334-344
pub unsafe fn skcms_MaxRoundtripError(
    curve: &crate::modules::skcms::src::skcms_public::skcms_Curve,
    inv_tf: &skcms_TransferFunction,
) -> f32 {
    let entries = unsafe { curve.table.table_entries };
    let n = if entries > 256 { entries } else { 256 };
    let dx = 1.0 / (n - 1) as f32;
    let mut err = 0.0;
    for i in 0..n {
        let x = i as f32 * dx;
        let y = unsafe { eval_curve(curve, x) };
        err = fmaxf_(
            err,
            crate::modules::skcms::src::skcms_internals::fabsf_(
                x - skcms_TransferFunction_eval(inv_tf, y),
            ),
        );
    }
    err
}
// Source: modules/skcms/skcms.cc:346-348
pub unsafe fn skcms_AreApproximateInverses(
    curve: &crate::modules::skcms::src::skcms_public::skcms_Curve,
    inv_tf: &skcms_TransferFunction,
) -> bool {
    (unsafe { skcms_MaxRoundtripError(curve, inv_tf) }) < (1.0 / 512.0)
}
use crate::modules::skcms::src::skcms_internals::skcms_ICCTag;
use crate::modules::skcms::src::skcms_public::{skcms_A2B, skcms_B2A, skcms_CICP};
// Source: modules/skcms/skcms.cc:351-383
const skcms_Signature_acsp: u32 = 0x61637370;
const skcms_Signature_rTRC: u32 = 0x72545243;
const skcms_Signature_gTRC: u32 = 0x67545243;
const skcms_Signature_bTRC: u32 = 0x62545243;
const skcms_Signature_kTRC: u32 = 0x6B545243;
const skcms_Signature_rXYZ: u32 = 0x7258595A;
const skcms_Signature_gXYZ: u32 = 0x6758595A;
const skcms_Signature_bXYZ: u32 = 0x6258595A;
const skcms_Signature_A2B0: u32 = 0x41324230;
const skcms_Signature_B2A0: u32 = 0x42324130;
const skcms_Signature_CHAD: u32 = 0x63686164;
const skcms_Signature_WTPT: u32 = 0x77747074;
const skcms_Signature_CICP: u32 = 0x63696370;
const skcms_Signature_curv: u32 = 0x63757276;
const skcms_Signature_mft1: u32 = 0x6D667431;
const skcms_Signature_mft2: u32 = 0x6D667432;
const skcms_Signature_mAB: u32 = 0x6D414220;
const skcms_Signature_mBA: u32 = 0x6D424120;
const skcms_Signature_para: u32 = 0x70617261;
const skcms_Signature_sf32: u32 = 0x73663332;
// Source: modules/skcms/skcms.cc:385-393
unsafe fn read_big_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(unsafe { [*ptr, *ptr.add(1)] })
}
// Source: modules/skcms/skcms.cc:395-403
unsafe fn read_big_u32(ptr: *const u8) -> u32 {
    u32::from_be_bytes(unsafe { [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)] })
}
// Source: modules/skcms/skcms.cc:405-407
unsafe fn read_big_i32(ptr: *const u8) -> i32 {
    unsafe { read_big_u32(ptr) as i32 }
}
// Source: modules/skcms/skcms.cc:409-411
unsafe fn read_big_fixed(ptr: *const u8) -> f32 {
    unsafe { read_big_i32(ptr) as f32 * (1.0 / 65536.0) }
}
// Source: modules/skcms/skcms.cc:415-437
#[repr(C)]
struct header_Layout {
    size: [u8; 4],
    cmm_type: [u8; 4],
    version: [u8; 4],
    profile_class: [u8; 4],
    data_color_space: [u8; 4],
    pcs: [u8; 4],
    creation_date_time: [u8; 12],
    signature: [u8; 4],
    platform: [u8; 4],
    flags: [u8; 4],
    device_manufacturer: [u8; 4],
    device_model: [u8; 4],
    device_attributes: [u8; 8],
    rendering_intent: [u8; 4],
    illuminant_X: [u8; 4],
    illuminant_Y: [u8; 4],
    illuminant_Z: [u8; 4],
    creator: [u8; 4],
    profile_id: [u8; 16],
    reserved: [u8; 28],
    tag_count: [u8; 4],
}
// Source: modules/skcms/skcms.cc:439-443
#[repr(C)]
struct tag_Layout {
    signature: [u8; 4],
    offset: [u8; 4],
    size: [u8; 4],
}
// Source: modules/skcms/skcms.cc:445-447
unsafe fn get_tag_table(profile: &skcms_ICCProfile) -> *const tag_Layout {
    unsafe {
        profile
            .buffer
            .add(std::mem::size_of::<header_Layout>())
            .cast()
    }
}
// Source: modules/skcms/skcms.cc:451-455
#[repr(C)]
struct sf32_Layout {
    r#type: [u8; 4],
    reserved: [u8; 4],
    values: [u8; 36],
}
// Source: modules/skcms/skcms.cc:457-474
pub fn skcms_GetCHAD(profile: &skcms_ICCProfile, m: &mut skcms_Matrix3x3) -> bool {
    let mut tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
    unsafe {
        if !skcms_GetTagBySignature(profile, skcms_Signature_CHAD, tag.as_mut_ptr()) {
            return false;
        }
        let tag = tag.assume_init();
        if tag.r#type != skcms_Signature_sf32
            || tag.size < (std::mem::size_of::<sf32_Layout>() as u32)
        {
            return false;
        }
        let sf32Tag = &*tag.buf.cast::<sf32_Layout>();
        let mut values = sf32Tag.values.as_ptr();
        for r in 0..3 {
            for c in 0..3 {
                m.vals[r][c] = read_big_fixed(values);
                values = values.add(4);
            }
        }
    }
    true
}
// Source: modules/skcms/skcms.cc:478-484
#[repr(C)]
struct XYZ_Layout {
    r#type: [u8; 4],
    reserved: [u8; 4],
    X: [u8; 4],
    Y: [u8; 4],
    Z: [u8; 4],
}
// Source: modules/skcms/skcms.cc:486-497
unsafe fn read_tag_xyz(tag: &skcms_ICCTag, x: *mut f32, y: *mut f32, z: *mut f32) -> bool {
    unsafe {
        if tag.r#type != skcms_Signature::skcms_Signature_XYZ as u32
            || tag.size < (std::mem::size_of::<XYZ_Layout>() as u32)
        {
            return false;
        }
        let xyzTag = &*tag.buf.cast::<XYZ_Layout>();
        *x = read_big_fixed(xyzTag.X.as_ptr());
        *y = read_big_fixed(xyzTag.Y.as_ptr());
        *z = read_big_fixed(xyzTag.Z.as_ptr());
        true
    }
}
// Source: modules/skcms/skcms.cc:499-503
pub fn skcms_GetWTPT(profile: &skcms_ICCProfile, xyz: &mut [f32; 3]) -> bool {
    unsafe {
        let mut tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
        skcms_GetTagBySignature(profile, skcms_Signature_WTPT, tag.as_mut_ptr())
            && read_tag_xyz(
                &tag.assume_init(),
                xyz.as_mut_ptr(),
                xyz.as_mut_ptr().add(1),
                xyz.as_mut_ptr().add(2),
            )
    }
}
// Source: modules/skcms/skcms.cc:505-534
fn data_color_space_channel_count(data_color_space: u32) -> i32 {
    match data_color_space {
        x if x == skcms_Signature::skcms_Signature_CMYK as u32 => 4,
        x if x == skcms_Signature::skcms_Signature_Gray as u32 => 1,
        x if x == skcms_Signature::skcms_Signature_RGB as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_Lab as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_XYZ as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_CIELUV as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_YCbCr as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_CIEYxy as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_HSV as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_HLS as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_CMY as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_2CLR as u32 => 2,
        x if x == skcms_Signature::skcms_Signature_3CLR as u32 => 3,
        x if x == skcms_Signature::skcms_Signature_4CLR as u32 => 4,
        x if x == skcms_Signature::skcms_Signature_5CLR as u32 => 5,
        x if x == skcms_Signature::skcms_Signature_6CLR as u32 => 6,
        x if x == skcms_Signature::skcms_Signature_7CLR as u32 => 7,
        x if x == skcms_Signature::skcms_Signature_8CLR as u32 => 8,
        x if x == skcms_Signature::skcms_Signature_9CLR as u32 => 9,
        x if x == skcms_Signature::skcms_Signature_10CLR as u32 => 10,
        x if x == skcms_Signature::skcms_Signature_11CLR as u32 => 11,
        x if x == skcms_Signature::skcms_Signature_12CLR as u32 => 12,
        x if x == skcms_Signature::skcms_Signature_13CLR as u32 => 13,
        x if x == skcms_Signature::skcms_Signature_14CLR as u32 => 14,
        x if x == skcms_Signature::skcms_Signature_15CLR as u32 => 15,
        _ => -1,
    }
}
// Source: modules/skcms/skcms.cc:536-566
pub fn skcms_GetInputChannelCount(profile: &skcms_ICCProfile) -> i32 {
    let mut a2b_count = 0;
    if profile.has_A2B {
        a2b_count = if profile.A2B.input_channels != 0 {
            profile.A2B.input_channels as i32
        } else {
            3
        };
    }
    let mut tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
    let mut trc_count = 0;
    if unsafe { skcms_GetTagBySignature(profile, skcms_Signature_kTRC, tag.as_mut_ptr()) } {
        trc_count = 1;
    } else if profile.has_trc {
        trc_count = 3;
    }
    let dcs_count = data_color_space_channel_count(profile.data_color_space);
    if dcs_count < 0 {
        return -1;
    }
    if a2b_count > 0 && a2b_count != dcs_count {
        return -1;
    }
    if trc_count > 0 && trc_count != dcs_count {
        return -1;
    }
    dcs_count
}
// Source: modules/skcms/skcms.cc:568-573
unsafe fn read_to_XYZD50(
    rXYZ: &skcms_ICCTag,
    gXYZ: &skcms_ICCTag,
    bXYZ: &skcms_ICCTag,
    toXYZ: &mut skcms_Matrix3x3,
) -> bool {
    unsafe {
        read_tag_xyz(
            rXYZ,
            &raw mut toXYZ.vals[0][0],
            &raw mut toXYZ.vals[1][0],
            &raw mut toXYZ.vals[2][0],
        ) && read_tag_xyz(
            gXYZ,
            &raw mut toXYZ.vals[0][1],
            &raw mut toXYZ.vals[1][1],
            &raw mut toXYZ.vals[2][1],
        ) && read_tag_xyz(
            bXYZ,
            &raw mut toXYZ.vals[0][2],
            &raw mut toXYZ.vals[1][2],
            &raw mut toXYZ.vals[2][2],
        )
    }
}
// Source: modules/skcms/skcms.cc:575-581
#[repr(C)]
struct para_Layout {
    r#type: [u8; 4],
    reserved_a: [u8; 4],
    function_type: [u8; 2],
    reserved_b: [u8; 2],
    variable: [u8; 1],
}
// Source: modules/skcms/skcms.cc:583-650
unsafe fn read_curve_para(
    buf: *const u8,
    size: u32,
    curve: &mut skcms_Curve,
    curve_size: *mut u32,
) -> bool {
    unsafe {
        const FIXED: u32 = std::mem::offset_of!(para_Layout, variable) as u32;
        if size < FIXED {
            return false;
        }
        let paraTag = &*buf.cast::<para_Layout>();
        let function_type = read_big_u16(paraTag.function_type.as_ptr());
        if function_type > 4 {
            return false;
        }
        static CURVE_BYTES: [u32; 5] = [4, 12, 16, 20, 28];
        if size < FIXED + CURVE_BYTES[function_type as usize] {
            return false;
        }
        if !curve_size.is_null() {
            *curve_size = FIXED + CURVE_BYTES[function_type as usize];
        }
        curve.parametric.alias_of_table_entries = 0;
        curve.parametric.parametric = skcms_TransferFunction {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
            g: read_big_fixed(paraTag.variable.as_ptr()),
        };
        let tf = &mut curve.parametric.parametric;
        let variable = paraTag.variable.as_ptr();
        match function_type {
            1 => {
                tf.a = read_big_fixed(variable.add(4));
                tf.b = read_big_fixed(variable.add(8));
                if tf.a == 0.0 {
                    return false;
                }
                tf.d = -tf.b / tf.a;
            }
            2 => {
                tf.a = read_big_fixed(variable.add(4));
                tf.b = read_big_fixed(variable.add(8));
                tf.e = read_big_fixed(variable.add(12));
                if tf.a == 0.0 {
                    return false;
                }
                tf.d = -tf.b / tf.a;
                tf.f = tf.e;
            }
            3 => {
                tf.a = read_big_fixed(variable.add(4));
                tf.b = read_big_fixed(variable.add(8));
                tf.c = read_big_fixed(variable.add(12));
                tf.d = read_big_fixed(variable.add(16));
            }
            4 => {
                tf.a = read_big_fixed(variable.add(4));
                tf.b = read_big_fixed(variable.add(8));
                tf.c = read_big_fixed(variable.add(12));
                tf.d = read_big_fixed(variable.add(16));
                tf.e = read_big_fixed(variable.add(20));
                tf.f = read_big_fixed(variable.add(24));
            }
            _ => {}
        }
        skcms_TransferFunction_isSRGBish(tf)
    }
}
// Source: modules/skcms/skcms.cc:652-657
#[repr(C)]
struct curv_Layout {
    r#type: [u8; 4],
    reserved: [u8; 4],
    value_count: [u8; 4],
    variable: [u8; 1],
}
// Source: modules/skcms/skcms.cc:660-660
const kMaxTableEntries: u32 = 1 << 24;
// Source: modules/skcms/skcms.cc:662-703
unsafe fn read_curve_curv(
    buf: *const u8,
    size: u32,
    curve: &mut skcms_Curve,
    curve_size: *mut u32,
) -> bool {
    unsafe {
        const FIXED: u32 = std::mem::offset_of!(curv_Layout, variable) as u32;
        if size < FIXED {
            return false;
        }
        let curvTag = &*buf.cast::<curv_Layout>();
        let value_count = read_big_u32(curvTag.value_count.as_ptr());
        if (size as u64) < FIXED as u64 + value_count as u64 * 2 {
            return false;
        }
        if !curve_size.is_null() {
            *curve_size = FIXED + value_count * 2;
        }
        if value_count < 2 {
            curve.parametric.alias_of_table_entries = 0;
            curve.parametric.parametric = skcms_TransferFunction {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
                g: if value_count == 0 {
                    1.0
                } else {
                    read_big_u16(curvTag.variable.as_ptr()) as f32 * (1.0 / 256.0)
                },
            };
            return true;
        }
        if value_count > kMaxTableEntries {
            return false;
        }
        curve.table.table_8 = std::ptr::null();
        curve.table.table_16 = curvTag.variable.as_ptr();
        curve.table.table_entries = value_count;
        true
    }
}
// Source: modules/skcms/skcms.cc:707-721
unsafe fn read_curve(
    buf: *const u8,
    size: u32,
    curve: *mut skcms_Curve,
    curve_size: *mut u32,
) -> bool {
    unsafe {
        if buf.is_null() || size < 4 || curve.is_null() {
            return false;
        }
        let ty = read_big_u32(buf);
        if ty == skcms_Signature_para {
            return read_curve_para(buf, size, &mut *curve, curve_size);
        } else if ty == skcms_Signature_curv {
            return read_curve_curv(buf, size, &mut *curve, curve_size);
        }
        false
    }
}
// Source: modules/skcms/skcms.cc:724-732
#[repr(C)]
struct mft_CommonLayout {
    r#type: [u8; 4],
    reserved_a: [u8; 4],
    input_channels: [u8; 1],
    output_channels: [u8; 1],
    grid_points: [u8; 1],
    reserved_b: [u8; 1],
    matrix: [u8; 36],
}
// Source: modules/skcms/skcms.cc:734-738
#[repr(C)]
struct mft1_Layout {
    common: [mft_CommonLayout; 1],
    variable: [u8; 1],
}
// Source: modules/skcms/skcms.cc:740-746
#[repr(C)]
struct mft2_Layout {
    common: [mft_CommonLayout; 1],
    input_table_entries: [u8; 2],
    output_table_entries: [u8; 2],
    variable: [u8; 1],
}
// Source: modules/skcms/skcms.cc:748-776
fn read_mft_common_a2b(mftTag: &mft_CommonLayout, a2b: &mut skcms_A2B) -> bool {
    a2b.matrix_channels = 0;
    a2b.input_channels = mftTag.input_channels[0] as u32;
    a2b.output_channels = mftTag.output_channels[0] as u32;
    if a2b.output_channels != a2b.output_curves.len() as u32 {
        return false;
    }
    if a2b.input_channels < 1 || a2b.input_channels > a2b.input_curves.len() as u32 {
        return false;
    }
    let mut total_grid_points = 1u64;
    for i in 0..a2b.input_channels as usize {
        a2b.grid_points[i] = mftTag.grid_points[0];
        total_grid_points *= a2b.grid_points[i] as u64;
    }
    if a2b.grid_points[0] < 2 || total_grid_points > SKCMS_MAX_GRID_POINTS {
        return false;
    }
    true
}
// Source: modules/skcms/skcms.cc:779-804
fn read_mft_common_b2a(mftTag: &mft_CommonLayout, b2a: &mut skcms_B2A) -> bool {
    b2a.matrix_channels = 0;
    b2a.input_channels = mftTag.input_channels[0] as u32;
    b2a.output_channels = mftTag.output_channels[0] as u32;
    if b2a.input_channels != b2a.input_curves.len() as u32 {
        return false;
    }
    if b2a.output_channels < 3 || b2a.output_channels > b2a.output_curves.len() as u32 {
        return false;
    }
    let mut total_grid_points = 1u64;
    for i in 0..b2a.input_channels as usize {
        b2a.grid_points[i] = mftTag.grid_points[0];
        total_grid_points *= b2a.grid_points[i] as u64;
    }
    if b2a.grid_points[0] < 2 || total_grid_points > SKCMS_MAX_GRID_POINTS {
        return false;
    }
    true
}
// Source: modules/skcms/skcms.cc:806-859
// Macro monomorphization preserves the native A2B_or_B2A template field operations.
macro_rules! init_tables {
    ($table_base:expr,$max_tables_len:expr,$byte_width:expr,$input_table_entries:expr,$output_table_entries:expr,$out:expr) => {{
        let table_base = $table_base;
        let max_tables_len = $max_tables_len;
        let byte_width = $byte_width;
        let input_table_entries = $input_table_entries;
        let output_table_entries = $output_table_entries;
        let out = $out;
        (|| unsafe {
            let byte_len_per_input_table = input_table_entries * byte_width;
            let byte_len_per_output_table = output_table_entries * byte_width;
            let byte_len_all_input_tables = out.input_channels * byte_len_per_input_table;
            let byte_len_all_output_tables = out.output_channels * byte_len_per_output_table;
            let mut grid_size = (out.output_channels * byte_width) as u64;
            for axis in 0..out.input_channels as usize {
                grid_size *= out.grid_points[axis] as u64;
            }
            if max_tables_len
                < (byte_len_all_input_tables as u64 + grid_size + byte_len_all_output_tables as u64)
            {
                return false;
            }
            for i in 0..out.input_channels as usize {
                out.input_curves[i].table.table_entries = input_table_entries;
                if byte_width == 1 {
                    out.input_curves[i].table.table_8 =
                        table_base.add(i * byte_len_per_input_table as usize);
                    out.input_curves[i].table.table_16 = std::ptr::null();
                } else {
                    out.input_curves[i].table.table_8 = std::ptr::null();
                    out.input_curves[i].table.table_16 =
                        table_base.add(i * byte_len_per_input_table as usize);
                }
            }
            if byte_width == 1 {
                out.grid_8 = table_base.add(byte_len_all_input_tables as usize);
                out.grid_16 = std::ptr::null();
            } else {
                out.grid_8 = std::ptr::null();
                out.grid_16 = table_base.add(byte_len_all_input_tables as usize);
            }
            let output_table_base =
                table_base.add(byte_len_all_input_tables as usize + grid_size as usize);
            for i in 0..out.output_channels as usize {
                out.output_curves[i].table.table_entries = output_table_entries;
                if byte_width == 1 {
                    out.output_curves[i].table.table_8 =
                        output_table_base.add(i * byte_len_per_output_table as usize);
                    out.output_curves[i].table.table_16 = std::ptr::null();
                } else {
                    out.output_curves[i].table.table_8 = std::ptr::null();
                    out.output_curves[i].table.table_16 =
                        output_table_base.add(i * byte_len_per_output_table as usize);
                }
            }
            true
        })()
    }};
}
// Source: modules/skcms/skcms.cc:861-877
macro_rules! read_tag_mft1 {
    ($tag:expr,$out:expr,$common:ident) => {{
        let tag = $tag;
        let out = $out;
        (|| unsafe {
            const FIXED: u32 = std::mem::offset_of!(mft1_Layout, variable) as u32;
            if tag.size < FIXED {
                return false;
            }
            let mftTag = &*tag.buf.cast::<mft1_Layout>();
            if !$common(&mftTag.common[0], out) {
                return false;
            }
            let input_table_entries = 256u32;
            let output_table_entries = 256u32;
            init_tables!(
                mftTag.variable.as_ptr(),
                (tag.size - FIXED) as u64,
                1u32,
                input_table_entries,
                output_table_entries,
                out
            )
        })()
    }};
}
// Source: modules/skcms/skcms.cc:879-901
macro_rules! read_tag_mft2 {
    ($tag:expr,$out:expr,$common:ident) => {{
        let tag = $tag;
        let out = $out;
        (|| unsafe {
            const FIXED: u32 = std::mem::offset_of!(mft2_Layout, variable) as u32;
            if tag.size < FIXED {
                return false;
            }
            let mftTag = &*tag.buf.cast::<mft2_Layout>();
            if !$common(&mftTag.common[0], out) {
                return false;
            }
            let input_table_entries = read_big_u16(mftTag.input_table_entries.as_ptr()) as u32;
            let output_table_entries = read_big_u16(mftTag.output_table_entries.as_ptr()) as u32;
            if input_table_entries < 2
                || input_table_entries > 4096
                || output_table_entries < 2
                || output_table_entries > 4096
            {
                return false;
            }
            init_tables!(
                mftTag.variable.as_ptr(),
                (tag.size - FIXED) as u64,
                2u32,
                input_table_entries,
                output_table_entries,
                out
            )
        })()
    }};
}
// Source: modules/skcms/skcms.cc:903-928
unsafe fn read_curves(
    buf: *const u8,
    size: u32,
    mut curve_offset: u32,
    num_curves: u32,
    curves: *mut skcms_Curve,
) -> bool {
    unsafe {
        for i in 0..num_curves as usize {
            if curve_offset > size {
                return false;
            }
            let mut curve_bytes = 0;
            if !read_curve(
                buf.add(curve_offset as usize),
                size - curve_offset,
                curves.add(i),
                &mut curve_bytes,
            ) {
                return false;
            }
            if curve_bytes > u32::MAX - 3 {
                return false;
            }
            curve_bytes = (curve_bytes + 3) & !3;
            let new_offset_64 = curve_offset as u64 + curve_bytes as u64;
            curve_offset = new_offset_64 as u32;
            if new_offset_64 != curve_offset as u64 {
                return false;
            }
        }
        true
    }
}
// Source: modules/skcms/skcms.cc:931-942
#[repr(C)]
struct mAB_or_mBA_Layout {
    r#type: [u8; 4],
    reserved_a: [u8; 4],
    input_channels: [u8; 1],
    output_channels: [u8; 1],
    reserved_b: [u8; 2],
    b_curve_offset: [u8; 4],
    matrix_offset: [u8; 4],
    m_curve_offset: [u8; 4],
    clut_offset: [u8; 4],
    a_curve_offset: [u8; 4],
}
// Source: modules/skcms/skcms.cc:944-949
#[repr(C)]
struct CLUT_Layout {
    grid_points: [u8; 16],
    grid_byte_width: [u8; 1],
    reserved: [u8; 3],
    variable: [u8; 1],
}
// Source: modules/skcms/skcms.cc:951-1093
unsafe fn read_tag_mab(
    tag: &skcms_ICCTag,
    out: &mut skcms_A2B,
    pcs_is_xyz: bool,
    endOfBuffer: *const u8,
) -> bool {
    unsafe {
        if (tag.size as usize) < std::mem::size_of::<mAB_or_mBA_Layout>() {
            return false;
        }
        let mTag = &*tag.buf.cast::<mAB_or_mBA_Layout>();
        out.input_channels = mTag.input_channels[0] as u32;
        out.output_channels = mTag.output_channels[0] as u32;
        if out.output_channels != out.output_curves.len() as u32 {
            return false;
        }
        if out.input_channels > out.input_curves.len() as u32 {
            return false;
        }
        let b_curve_offset = read_big_u32(mTag.b_curve_offset.as_ptr());
        let matrix_offset = read_big_u32(mTag.matrix_offset.as_ptr());
        let m_curve_offset = read_big_u32(mTag.m_curve_offset.as_ptr());
        let clut_offset = read_big_u32(mTag.clut_offset.as_ptr());
        let a_curve_offset = read_big_u32(mTag.a_curve_offset.as_ptr());
        if b_curve_offset == 0 {
            return false;
        }
        if !read_curves(
            tag.buf,
            tag.size,
            b_curve_offset,
            out.output_channels,
            out.output_curves.as_mut_ptr(),
        ) {
            return false;
        }
        if m_curve_offset != 0 {
            if matrix_offset == 0 {
                return false;
            }
            out.matrix_channels = out.output_channels;
            if !read_curves(
                tag.buf,
                tag.size,
                m_curve_offset,
                out.matrix_channels,
                out.matrix_curves.as_mut_ptr(),
            ) {
                return false;
            }
            if (tag.size as u64) < matrix_offset as u64 + 48 {
                return false;
            }
            let encoding_factor = if pcs_is_xyz { 65535.0 / 32768.0 } else { 1.0 };
            let mtx_buf = tag.buf.add(matrix_offset as usize);
            out.matrix.vals[0][0] = encoding_factor * read_big_fixed(mtx_buf.add(0));
            out.matrix.vals[0][1] = encoding_factor * read_big_fixed(mtx_buf.add(4));
            out.matrix.vals[0][2] = encoding_factor * read_big_fixed(mtx_buf.add(8));
            out.matrix.vals[1][0] = encoding_factor * read_big_fixed(mtx_buf.add(12));
            out.matrix.vals[1][1] = encoding_factor * read_big_fixed(mtx_buf.add(16));
            out.matrix.vals[1][2] = encoding_factor * read_big_fixed(mtx_buf.add(20));
            out.matrix.vals[2][0] = encoding_factor * read_big_fixed(mtx_buf.add(24));
            out.matrix.vals[2][1] = encoding_factor * read_big_fixed(mtx_buf.add(28));
            out.matrix.vals[2][2] = encoding_factor * read_big_fixed(mtx_buf.add(32));
            out.matrix.vals[0][3] = encoding_factor * read_big_fixed(mtx_buf.add(36));
            out.matrix.vals[1][3] = encoding_factor * read_big_fixed(mtx_buf.add(40));
            out.matrix.vals[2][3] = encoding_factor * read_big_fixed(mtx_buf.add(44));
        } else {
            if matrix_offset != 0 {
                return false;
            }
            out.matrix_channels = 0;
        }
        if a_curve_offset != 0 {
            if clut_offset == 0 {
                return false;
            }
            if !read_curves(
                tag.buf,
                tag.size,
                a_curve_offset,
                out.input_channels,
                out.input_curves.as_mut_ptr(),
            ) {
                return false;
            }
            const FIXED: u64 = std::mem::offset_of!(CLUT_Layout, variable) as u64;
            if (tag.size as u64) < clut_offset as u64 + FIXED {
                return false;
            }
            let clut = &*tag.buf.add(clut_offset as usize).cast::<CLUT_Layout>();
            if clut.grid_byte_width[0] == 1 {
                out.grid_8 = clut.variable.as_ptr();
                out.grid_16 = std::ptr::null();
            } else if clut.grid_byte_width[0] == 2 {
                out.grid_8 = std::ptr::null();
                out.grid_16 = clut.variable.as_ptr();
            } else {
                return false;
            }
            let mut grid_size = (out.output_channels * clut.grid_byte_width[0] as u32) as u64;
            let mut total_grid_points = 1u64;
            for i in 0..out.input_channels as usize {
                out.grid_points[i] = clut.grid_points[i];
                if out.grid_points[i] < 2 {
                    return false;
                }
                grid_size *= out.grid_points[i] as u64;
                total_grid_points *= out.grid_points[i] as u64;
            }
            if total_grid_points > SKCMS_MAX_GRID_POINTS {
                return false;
            }
            let table_size = clut_offset as u64 + FIXED + grid_size;
            if table_size > tag.size as u64 {
                return false;
            }
            let mut slack = 0;
            if out.output_channels == 3 {
                slack = if clut.grid_byte_width[0] == 1 { 1 } else { 2 };
            }
            if tag.buf.wrapping_add((table_size + slack) as usize) > endOfBuffer {
                return false;
            }
        } else {
            if clut_offset != 0 {
                return false;
            }
            if out.input_channels != out.output_channels {
                return false;
            }
            out.input_channels = 0;
        }
        true
    }
}
// Source: modules/skcms/skcms.cc:1097-1235
unsafe fn read_tag_mba(
    tag: &skcms_ICCTag,
    out: &mut skcms_B2A,
    pcs_is_xyz: bool,
    endOfBuffer: *const u8,
) -> bool {
    unsafe {
        if (tag.size as usize) < std::mem::size_of::<mAB_or_mBA_Layout>() {
            return false;
        }
        let mTag = &*tag.buf.cast::<mAB_or_mBA_Layout>();
        out.input_channels = mTag.input_channels[0] as u32;
        out.output_channels = mTag.output_channels[0] as u32;
        if out.input_channels != out.input_curves.len() as u32 {
            return false;
        }
        if out.output_channels < 3 || out.output_channels > out.output_curves.len() as u32 {
            return false;
        }
        let b_curve_offset = read_big_u32(mTag.b_curve_offset.as_ptr());
        let matrix_offset = read_big_u32(mTag.matrix_offset.as_ptr());
        let m_curve_offset = read_big_u32(mTag.m_curve_offset.as_ptr());
        let clut_offset = read_big_u32(mTag.clut_offset.as_ptr());
        let a_curve_offset = read_big_u32(mTag.a_curve_offset.as_ptr());
        if b_curve_offset == 0 {
            return false;
        }
        if !read_curves(
            tag.buf,
            tag.size,
            b_curve_offset,
            out.input_channels,
            out.input_curves.as_mut_ptr(),
        ) {
            return false;
        }
        if m_curve_offset != 0 {
            if matrix_offset == 0 {
                return false;
            }
            out.matrix_channels = out.input_channels;
            if !read_curves(
                tag.buf,
                tag.size,
                m_curve_offset,
                out.matrix_channels,
                out.matrix_curves.as_mut_ptr(),
            ) {
                return false;
            }
            if (tag.size as u64) < matrix_offset as u64 + 48 {
                return false;
            }
            let encoding_factor = if pcs_is_xyz { 32768.0 / 65535.0 } else { 1.0 };
            let mtx_buf = tag.buf.add(matrix_offset as usize);
            out.matrix.vals[0][0] = encoding_factor * read_big_fixed(mtx_buf.add(0));
            out.matrix.vals[0][1] = encoding_factor * read_big_fixed(mtx_buf.add(4));
            out.matrix.vals[0][2] = encoding_factor * read_big_fixed(mtx_buf.add(8));
            out.matrix.vals[1][0] = encoding_factor * read_big_fixed(mtx_buf.add(12));
            out.matrix.vals[1][1] = encoding_factor * read_big_fixed(mtx_buf.add(16));
            out.matrix.vals[1][2] = encoding_factor * read_big_fixed(mtx_buf.add(20));
            out.matrix.vals[2][0] = encoding_factor * read_big_fixed(mtx_buf.add(24));
            out.matrix.vals[2][1] = encoding_factor * read_big_fixed(mtx_buf.add(28));
            out.matrix.vals[2][2] = encoding_factor * read_big_fixed(mtx_buf.add(32));
            out.matrix.vals[0][3] = encoding_factor * read_big_fixed(mtx_buf.add(36));
            out.matrix.vals[1][3] = encoding_factor * read_big_fixed(mtx_buf.add(40));
            out.matrix.vals[2][3] = encoding_factor * read_big_fixed(mtx_buf.add(44));
        } else {
            if matrix_offset != 0 {
                return false;
            }
            out.matrix_channels = 0;
        }
        if a_curve_offset != 0 {
            if clut_offset == 0 {
                return false;
            }
            if !read_curves(
                tag.buf,
                tag.size,
                a_curve_offset,
                out.output_channels,
                out.output_curves.as_mut_ptr(),
            ) {
                return false;
            }
            const FIXED: u64 = std::mem::offset_of!(CLUT_Layout, variable) as u64;
            if (tag.size as u64) < clut_offset as u64 + FIXED {
                return false;
            }
            let clut = &*tag.buf.add(clut_offset as usize).cast::<CLUT_Layout>();
            if clut.grid_byte_width[0] == 1 {
                out.grid_8 = clut.variable.as_ptr();
                out.grid_16 = std::ptr::null();
            } else if clut.grid_byte_width[0] == 2 {
                out.grid_8 = std::ptr::null();
                out.grid_16 = clut.variable.as_ptr();
            } else {
                return false;
            }
            let mut grid_size = (out.output_channels * clut.grid_byte_width[0] as u32) as u64;
            let mut total_grid_points = 1u64;
            for i in 0..out.input_channels as usize {
                out.grid_points[i] = clut.grid_points[i];
                if out.grid_points[i] < 2 {
                    return false;
                }
                grid_size *= out.grid_points[i] as u64;
                total_grid_points *= out.grid_points[i] as u64;
            }
            if total_grid_points > SKCMS_MAX_GRID_POINTS {
                return false;
            }
            let table_size = clut_offset as u64 + FIXED + grid_size;
            if table_size > tag.size as u64 {
                return false;
            }
            let mut slack = 0;
            if out.output_channels == 3 {
                slack = if clut.grid_byte_width[0] == 1 { 1 } else { 2 };
            }
            if tag.buf.wrapping_add((table_size + slack) as usize) > endOfBuffer {
                return false;
            }
        } else {
            if clut_offset != 0 {
                return false;
            }
            if out.input_channels != out.output_channels {
                return false;
            }
            out.output_channels = 0;
        }
        true
    }
}
// Source: modules/skcms/skcms.cc:1239-1289
unsafe fn fit_linear(
    curve: &skcms_Curve,
    n: i32,
    tol: f32,
    c: &mut f32,
    d: &mut f32,
    f: Option<&mut f32>,
) -> i32 {
    debug_assert!(n > 1);
    let dx = 1.0 / (n - 1) as f32;
    let mut lin_points = 1;
    let mut f_zero = 0.0;
    let f = if let Some(f) = f {
        *f = unsafe { eval_curve(curve, 0.0) };
        f
    } else {
        &mut f_zero
    };
    let mut slope_min = f32::NEG_INFINITY;
    let mut slope_max = f32::INFINITY;
    for i in 1..n {
        let x = i as f32 * dx;
        let y = unsafe { eval_curve(curve, x) };
        let slope_max_i = (y + tol - *f) / x;
        let slope_min_i = (y - tol - *f) / x;
        if slope_max_i < slope_min || slope_max < slope_min_i {
            break;
        }
        slope_max = fminf_(slope_max, slope_max_i);
        slope_min = fmaxf_(slope_min, slope_min_i);
        let cur_slope = (y - *f) / x;
        if slope_min <= cur_slope && cur_slope <= slope_max {
            lin_points = i + 1;
            *c = cur_slope;
        }
    }
    *d = (lin_points - 1) as f32 * dx;
    lin_points
}
// Source: modules/skcms/skcms.cc:1292-1306
fn canonicalize_identity(curve: &mut skcms_Curve) {
    unsafe {
        if curve.table.table_entries != 0 && curve.table.table_entries <= i32::MAX as u32 {
            let n = curve.table.table_entries as i32;
            let (mut c, mut d, mut f) = (0.0, 0.0, 0.0);
            if n == fit_linear(curve, n, 1.0 / (2 * n) as f32, &mut c, &mut d, Some(&mut f))
                && c == 1.0
                && f == 0.0
            {
                curve.table.table_entries = 0;
                curve.table.table_8 = std::ptr::null();
                curve.table.table_16 = std::ptr::null();
                curve.parametric.parametric = skcms_TransferFunction {
                    g: 1.0,
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 0.0,
                    e: 0.0,
                    f: 0.0,
                };
            }
        }
    }
}
// Source: modules/skcms/skcms.cc:1308-1331
unsafe fn read_a2b(
    tag: &skcms_ICCTag,
    a2b: &mut skcms_A2B,
    pcs_is_xyz: bool,
    eob: *const u8,
) -> bool {
    unsafe {
        let mut ok = false;
        if tag.r#type == skcms_Signature_mft1 {
            ok = read_tag_mft1!(tag, &mut *a2b, read_mft_common_a2b);
        }
        if tag.r#type == skcms_Signature_mft2 {
            ok = read_tag_mft2!(tag, &mut *a2b, read_mft_common_a2b);
        }
        if tag.r#type == skcms_Signature_mAB {
            ok = read_tag_mab(tag, a2b, pcs_is_xyz, eob);
        }
        if !ok {
            return false;
        }
        if a2b.input_channels > 0 {
            canonicalize_identity(&mut a2b.input_curves[0]);
        }
        if a2b.input_channels > 1 {
            canonicalize_identity(&mut a2b.input_curves[1]);
        }
        if a2b.input_channels > 2 {
            canonicalize_identity(&mut a2b.input_curves[2]);
        }
        if a2b.input_channels > 3 {
            canonicalize_identity(&mut a2b.input_curves[3]);
        }
        if a2b.matrix_channels > 0 {
            canonicalize_identity(&mut a2b.matrix_curves[0]);
        }
        if a2b.matrix_channels > 1 {
            canonicalize_identity(&mut a2b.matrix_curves[1]);
        }
        if a2b.matrix_channels > 2 {
            canonicalize_identity(&mut a2b.matrix_curves[2]);
        }
        if a2b.output_channels > 0 {
            canonicalize_identity(&mut a2b.output_curves[0]);
        }
        if a2b.output_channels > 1 {
            canonicalize_identity(&mut a2b.output_curves[1]);
        }
        if a2b.output_channels > 2 {
            canonicalize_identity(&mut a2b.output_curves[2]);
        }
        true
    }
}
// Source: modules/skcms/skcms.cc:1333-1356
unsafe fn read_b2a(
    tag: &skcms_ICCTag,
    b2a: &mut skcms_B2A,
    pcs_is_xyz: bool,
    eob: *const u8,
) -> bool {
    unsafe {
        let mut ok = false;
        if tag.r#type == skcms_Signature_mft1 {
            ok = read_tag_mft1!(tag, &mut *b2a, read_mft_common_b2a);
        }
        if tag.r#type == skcms_Signature_mft2 {
            ok = read_tag_mft2!(tag, &mut *b2a, read_mft_common_b2a);
        }
        if tag.r#type == skcms_Signature_mBA {
            ok = read_tag_mba(tag, b2a, pcs_is_xyz, eob);
        }
        if !ok {
            return false;
        }
        if b2a.input_channels > 0 {
            canonicalize_identity(&mut b2a.input_curves[0]);
        }
        if b2a.input_channels > 1 {
            canonicalize_identity(&mut b2a.input_curves[1]);
        }
        if b2a.input_channels > 2 {
            canonicalize_identity(&mut b2a.input_curves[2]);
        }
        if b2a.matrix_channels > 0 {
            canonicalize_identity(&mut b2a.matrix_curves[0]);
        }
        if b2a.matrix_channels > 1 {
            canonicalize_identity(&mut b2a.matrix_curves[1]);
        }
        if b2a.matrix_channels > 2 {
            canonicalize_identity(&mut b2a.matrix_curves[2]);
        }
        if b2a.output_channels > 0 {
            canonicalize_identity(&mut b2a.output_curves[0]);
        }
        if b2a.output_channels > 1 {
            canonicalize_identity(&mut b2a.output_curves[1]);
        }
        if b2a.output_channels > 2 {
            canonicalize_identity(&mut b2a.output_curves[2]);
        }
        if b2a.output_channels > 3 {
            canonicalize_identity(&mut b2a.output_curves[3]);
        }
        true
    }
}
// Source: modules/skcms/skcms.cc:1358-1365
#[repr(C)]
struct CICP_Layout {
    r#type: [u8; 4],
    reserved: [u8; 4],
    color_primaries: [u8; 1],
    transfer_characteristics: [u8; 1],
    matrix_coefficients: [u8; 1],
    video_full_range_flag: [u8; 1],
}
// Source: modules/skcms/skcms.cc:1367-1379
unsafe fn read_cicp(tag: &skcms_ICCTag, cicp: &mut skcms_CICP) -> bool {
    unsafe {
        if tag.r#type != skcms_Signature_CICP
            || (tag.size as usize) < std::mem::size_of::<CICP_Layout>()
        {
            return false;
        }
        let cicpTag = &*tag.buf.cast::<CICP_Layout>();
        cicp.color_primaries = cicpTag.color_primaries[0];
        cicp.transfer_characteristics = cicpTag.transfer_characteristics[0];
        cicp.matrix_coefficients = cicpTag.matrix_coefficients[0];
        cicp.video_full_range_flag = cicpTag.video_full_range_flag[0];
        true
    }
}
// Source: modules/skcms/skcms.cc:1381-1389
pub unsafe fn skcms_GetTagByIndex(
    profile: *const skcms_ICCProfile,
    idx: u32,
    tag: *mut skcms_ICCTag,
) {
    unsafe {
        if profile.is_null() || (*profile).buffer.is_null() || tag.is_null() {
            return;
        }
        let profile = &*profile;
        if idx > profile.tag_count {
            return;
        }
        let tags = get_tag_table(profile);
        let entry = &*tags.add(idx as usize);
        (*tag).signature = read_big_u32(entry.signature.as_ptr());
        (*tag).size = read_big_u32(entry.size.as_ptr());
        (*tag).buf = profile
            .buffer
            .add(read_big_u32(entry.offset.as_ptr()) as usize);
        (*tag).r#type = read_big_u32((*tag).buf);
    }
}
// Source: modules/skcms/skcms.cc:1391-1404
pub unsafe fn skcms_GetTagBySignature(
    profile: *const skcms_ICCProfile,
    sig: u32,
    tag: *mut skcms_ICCTag,
) -> bool {
    unsafe {
        if profile.is_null() || (*profile).buffer.is_null() || tag.is_null() {
            return false;
        }
        let profile = &*profile;
        let tags = get_tag_table(profile);
        for i in 0..profile.tag_count as usize {
            let entry = &*tags.add(i);
            if read_big_u32(entry.signature.as_ptr()) == sig {
                (*tag).signature = sig;
                (*tag).size = read_big_u32(entry.size.as_ptr());
                (*tag).buf = profile
                    .buffer
                    .add(read_big_u32(entry.offset.as_ptr()) as usize);
                (*tag).r#type = read_big_u32((*tag).buf);
                return true;
            }
        }
        false
    }
}
// Source: modules/skcms/skcms.cc:1406-1409
fn usable_as_src(profile: &skcms_ICCProfile) -> bool {
    profile.has_A2B || (profile.has_trc && profile.has_toXYZD50)
}
// Source: modules/skcms/skcms.cc:1411-1561
pub unsafe fn skcms_ParseWithA2BPriority(
    buf: *const std::ffi::c_void,
    len: usize,
    priority: *const i32,
    priorities: i32,
    profile: *mut skcms_ICCProfile,
) -> bool {
    unsafe {
        const _: () = assert!(std::mem::size_of::<header_Layout>() == 132);
        if profile.is_null() {
            return false;
        }
        std::ptr::write_bytes(profile, 0, 1);
        if len < std::mem::size_of::<header_Layout>() {
            return false;
        }
        let header = &*buf.cast::<header_Layout>();
        let profile = &mut *profile;
        profile.buffer = buf.cast();
        profile.size = read_big_u32(header.size.as_ptr());
        let version = read_big_u32(header.version.as_ptr());
        profile.data_color_space = read_big_u32(header.data_color_space.as_ptr());
        profile.pcs = read_big_u32(header.pcs.as_ptr());
        let signature = read_big_u32(header.signature.as_ptr());
        let illuminant_X = read_big_fixed(header.illuminant_X.as_ptr());
        let illuminant_Y = read_big_fixed(header.illuminant_Y.as_ptr());
        let illuminant_Z = read_big_fixed(header.illuminant_Z.as_ptr());
        profile.tag_count = read_big_u32(header.tag_count.as_ptr());
        let tag_table_size = profile.tag_count as u64 * std::mem::size_of::<tag_Layout>() as u64;
        if signature != skcms_Signature_acsp
            || profile.size as usize > len
            || (profile.size as u64) < std::mem::size_of::<header_Layout>() as u64 + tag_table_size
            || (version >> 24) > 4
        {
            return false;
        }
        if crate::modules::skcms::src::skcms_internals::fabsf_(illuminant_X - 0.9642) > 0.0100
            || crate::modules::skcms::src::skcms_internals::fabsf_(illuminant_Y - 1.0) > 0.0100
            || crate::modules::skcms::src::skcms_internals::fabsf_(illuminant_Z - 0.8249) > 0.0100
        {
            return false;
        }
        let tags = get_tag_table(profile);
        for i in 0..profile.tag_count as usize {
            let entry = &*tags.add(i);
            let tag_offset = read_big_u32(entry.offset.as_ptr());
            let tag_size = read_big_u32(entry.size.as_ptr());
            let tag_end = tag_offset as u64 + tag_size as u64;
            if tag_size < 4 || tag_end > profile.size as u64 {
                return false;
            }
        }
        if profile.pcs != skcms_Signature::skcms_Signature_XYZ as u32
            && profile.pcs != skcms_Signature::skcms_Signature_Lab as u32
        {
            return false;
        }
        let pcs_is_xyz = profile.pcs == skcms_Signature::skcms_Signature_XYZ as u32;
        let mut kTRC = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
        if profile.data_color_space == skcms_Signature::skcms_Signature_Gray as u32
            && skcms_GetTagBySignature(profile, skcms_Signature_kTRC, kTRC.as_mut_ptr())
        {
            let kTRC = kTRC.assume_init();
            if !read_curve(
                kTRC.buf,
                kTRC.size,
                &mut profile.trc[0],
                std::ptr::null_mut(),
            ) {
                return false;
            }
            profile.trc[1] = profile.trc[0];
            profile.trc[2] = profile.trc[0];
            profile.has_trc = true;
            if pcs_is_xyz {
                profile.toXYZD50.vals[0][0] = illuminant_X;
                profile.toXYZD50.vals[1][1] = illuminant_Y;
                profile.toXYZD50.vals[2][2] = illuminant_Z;
                profile.has_toXYZD50 = true;
            }
        } else {
            let mut rTRC = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            let mut gTRC = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            let mut bTRC = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            if skcms_GetTagBySignature(profile, skcms_Signature_rTRC, rTRC.as_mut_ptr())
                && skcms_GetTagBySignature(profile, skcms_Signature_gTRC, gTRC.as_mut_ptr())
                && skcms_GetTagBySignature(profile, skcms_Signature_bTRC, bTRC.as_mut_ptr())
            {
                let (rTRC, gTRC, bTRC) =
                    (rTRC.assume_init(), gTRC.assume_init(), bTRC.assume_init());
                if !read_curve(
                    rTRC.buf,
                    rTRC.size,
                    &mut profile.trc[0],
                    std::ptr::null_mut(),
                ) || !read_curve(
                    gTRC.buf,
                    gTRC.size,
                    &mut profile.trc[1],
                    std::ptr::null_mut(),
                ) || !read_curve(
                    bTRC.buf,
                    bTRC.size,
                    &mut profile.trc[2],
                    std::ptr::null_mut(),
                ) {
                    return false;
                }
                profile.has_trc = true;
            }
            let mut rXYZ = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            let mut gXYZ = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            let mut bXYZ = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            if skcms_GetTagBySignature(profile, skcms_Signature_rXYZ, rXYZ.as_mut_ptr())
                && skcms_GetTagBySignature(profile, skcms_Signature_gXYZ, gXYZ.as_mut_ptr())
                && skcms_GetTagBySignature(profile, skcms_Signature_bXYZ, bXYZ.as_mut_ptr())
            {
                if !read_to_XYZD50(
                    &rXYZ.assume_init(),
                    &gXYZ.assume_init(),
                    &bXYZ.assume_init(),
                    &mut profile.toXYZD50,
                ) {
                    return false;
                }
                profile.has_toXYZD50 = true;
            }
        }
        let endOfBuffer = buf.cast::<u8>().add(len);
        for i in 0..priorities {
            let p = *priority.add(i as usize);
            if p < 0 || p > 2 {
                return false;
            }
            let sig = skcms_Signature_A2B0 + p as u32;
            let mut tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            if skcms_GetTagBySignature(profile, sig, tag.as_mut_ptr()) {
                if !read_a2b(
                    &tag.assume_init(),
                    &mut profile.A2B,
                    pcs_is_xyz,
                    endOfBuffer,
                ) {
                    return false;
                }
                profile.has_A2B = true;
                break;
            }
        }
        for i in 0..priorities {
            let p = *priority.add(i as usize);
            if p < 0 || p > 2 {
                return false;
            }
            let sig = skcms_Signature_B2A0 + p as u32;
            let mut tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
            if skcms_GetTagBySignature(profile, sig, tag.as_mut_ptr()) {
                if !read_b2a(
                    &tag.assume_init(),
                    &mut profile.B2A,
                    pcs_is_xyz,
                    endOfBuffer,
                ) {
                    return false;
                }
                profile.has_B2A = true;
                break;
            }
        }
        let mut cicp_tag = std::mem::MaybeUninit::<skcms_ICCTag>::uninit();
        if skcms_GetTagBySignature(profile, skcms_Signature_CICP, cicp_tag.as_mut_ptr()) {
            if !read_cicp(&cicp_tag.assume_init(), &mut profile.CICP) {
                return false;
            }
            profile.has_CICP = true;
        }
        usable_as_src(profile)
    }
}

// Source: modules/skcms/skcms.cc:1564-1580
// The native const singleton contains only immutable scalar values and null borrowed pointers.
// This private wrapper permits static storage without asserting Sync for arbitrary borrowed ICC profiles.
struct ImmutableBuiltinProfile(crate::modules::skcms::src::skcms_public::skcms_ICCProfile);
unsafe impl Sync for ImmutableBuiltinProfile {}
static SRGB_PROFILE: ImmutableBuiltinProfile = ImmutableBuiltinProfile({
    use crate::modules::skcms::src::skcms_public::*;
    skcms_ICCProfile {
        buffer: std::ptr::null(),
        size: 0,
        data_color_space: skcms_Signature::skcms_Signature_RGB as u32,
        pcs: skcms_Signature::skcms_Signature_XYZ as u32,
        tag_count: 0,
        trc: [
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 2.4,
                        a: (1.0_f64 / 1.055_f64) as f32,
                        b: (0.055_f64 / 1.055_f64) as f32,
                        c: (1.0_f64 / 12.92_f64) as f32,
                        d: 0.04045,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 2.4,
                        a: (1.0_f64 / 1.055_f64) as f32,
                        b: (0.055_f64 / 1.055_f64) as f32,
                        c: (1.0_f64 / 12.92_f64) as f32,
                        d: 0.04045,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 2.4,
                        a: (1.0_f64 / 1.055_f64) as f32,
                        b: (0.055_f64 / 1.055_f64) as f32,
                        c: (1.0_f64 / 12.92_f64) as f32,
                        d: 0.04045,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
        ],
        // Source: modules/skcms/skcms.cc:1581-1585
        toXYZD50: skcms_Matrix3x3 {
            vals: [
                [0.436065674, 0.385147095, 0.143066406],
                [0.222488403, 0.716873169, 0.060607910],
                [0.013916016, 0.097076416, 0.714096069],
            ],
        },
        // Source: modules/skcms/skcms.cc:1587-1660
        A2B: skcms_A2B {
            input_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            grid_8: std::ptr::null(),
            grid_16: std::ptr::null(),
            input_channels: 0,
            grid_points: [0, 0, 0, 0],
            matrix_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            matrix: skcms_Matrix3x4 {
                vals: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
            matrix_channels: 0,
            output_channels: 0,
            output_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
        },
        B2A: skcms_B2A {
            input_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            input_channels: 0,
            matrix_channels: 0,
            matrix_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            matrix: skcms_Matrix3x4 {
                vals: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
            output_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            grid_8: std::ptr::null(),
            grid_16: std::ptr::null(),
            grid_points: [0, 0, 0, 0],
            output_channels: 0,
        },
        CICP: skcms_CICP {
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            video_full_range_flag: 0,
        },
        has_trc: true,
        has_toXYZD50: true,
        has_A2B: false,
        has_B2A: false,
        has_CICP: false,
    }
});
pub fn skcms_sRGB_profile() -> &'static crate::modules::skcms::src::skcms_public::skcms_ICCProfile {
    &SRGB_PROFILE.0
}
// Existing numeric consumers retain their canonical matrix value without duplicating profile state.
pub const SRGB_PROFILE_TO_XYZD50: skcms_Matrix3x3 = SRGB_PROFILE.0.toXYZD50;
// Source: modules/skcms/skcms.cc:1662-1758
static XYZD50_PROFILE: ImmutableBuiltinProfile = ImmutableBuiltinProfile({
    use crate::modules::skcms::src::skcms_public::*;
    skcms_ICCProfile {
        buffer: std::ptr::null(),
        size: 0,
        data_color_space: skcms_Signature::skcms_Signature_RGB as u32,
        pcs: skcms_Signature::skcms_Signature_XYZ as u32,
        tag_count: 0,
        trc: [
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 1.0,
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 0.0,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 1.0,
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 0.0,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
            skcms_Curve {
                parametric: skcms_CurveParametric {
                    alias_of_table_entries: 0,
                    parametric: skcms_TransferFunction {
                        g: 1.0,
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 0.0,
                        e: 0.0,
                        f: 0.0,
                    },
                },
            },
        ],
        toXYZD50: skcms_Matrix3x3 {
            vals: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        },
        A2B: skcms_A2B {
            input_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            grid_8: std::ptr::null(),
            grid_16: std::ptr::null(),
            input_channels: 0,
            grid_points: [0, 0, 0, 0],
            matrix_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            matrix: skcms_Matrix3x4 {
                vals: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
            matrix_channels: 0,
            output_channels: 0,
            output_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
        },
        B2A: skcms_B2A {
            input_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            input_channels: 0,
            matrix_channels: 0,
            matrix_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            matrix: skcms_Matrix3x4 {
                vals: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
            output_curves: [
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
                skcms_Curve {
                    parametric: skcms_CurveParametric {
                        alias_of_table_entries: 0,
                        parametric: skcms_TransferFunction {
                            g: 0.0,
                            a: 0.0,
                            b: 0.0,
                            c: 0.0,
                            d: 0.0,
                            e: 0.0,
                            f: 0.0,
                        },
                    },
                },
            ],
            grid_8: std::ptr::null(),
            grid_16: std::ptr::null(),
            grid_points: [0, 0, 0, 0],
            output_channels: 0,
        },
        CICP: skcms_CICP {
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            video_full_range_flag: 0,
        },
        has_trc: true,
        has_toXYZD50: true,
        has_A2B: false,
        has_B2A: false,
        has_CICP: false,
    }
});
pub fn skcms_XYZD50_profile() -> &'static crate::modules::skcms::src::skcms_public::skcms_ICCProfile
{
    &XYZD50_PROFILE.0
}
// Source: modules/skcms/skcms.cc:1760-1762
pub fn skcms_sRGB_TransferFunction() -> &'static skcms_TransferFunction {
    unsafe { &skcms_sRGB_profile().trc[0].parametric.parametric }
}
// Source: modules/skcms/skcms.cc:1764-1768
pub fn skcms_sRGB_Inverse_TransferFunction() -> &'static skcms_TransferFunction {
    static SRGB_INV: skcms_TransferFunction = skcms_TransferFunction {
        g: 0.416666657,
        a: 1.137283325,
        b: -0.0,
        c: 12.920000076,
        d: 0.003130805,
        e: -0.054969788,
        f: -0.0,
    };
    &SRGB_INV
}
// Source: modules/skcms/skcms.cc:1770-1773
pub fn skcms_Identity_TransferFunction() -> &'static skcms_TransferFunction {
    static IDENTITY: skcms_TransferFunction = skcms_TransferFunction {
        g: 1.0,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    &IDENTITY
}
// Source: modules/skcms/skcms.cc:1775-1792
pub static skcms_252_random_bytes: [u8; 252] = [
    8, 179, 128, 204, 253, 38, 134, 184, 68, 102, 32, 138, 99, 39, 169, 215, 119, 26, 3, 223, 95,
    239, 52, 132, 114, 74, 81, 234, 97, 116, 244, 205, 30, 154, 173, 12, 51, 159, 122, 153, 61,
    226, 236, 178, 229, 55, 181, 220, 191, 194, 160, 126, 168, 82, 131, 18, 180, 245, 163, 22, 246,
    69, 235, 252, 57, 108, 14, 6, 152, 240, 255, 171, 242, 20, 227, 177, 238, 96, 85, 16, 211, 70,
    200, 149, 155, 146, 127, 145, 100, 151, 109, 19, 165, 208, 195, 164, 137, 254, 182, 248, 64,
    201, 45, 209, 5, 147, 207, 210, 113, 162, 83, 225, 9, 31, 15, 231, 115, 37, 58, 53, 24, 49,
    197, 56, 120, 172, 48, 21, 214, 129, 111, 11, 50, 187, 196, 34, 60, 103, 71, 144, 47, 203, 77,
    80, 232, 140, 222, 250, 206, 166, 247, 139, 249, 221, 72, 106, 27, 199, 117, 54, 219, 135, 118,
    40, 79, 41, 251, 46, 93, 212, 92, 233, 148, 28, 121, 63, 123, 158, 105, 59, 29, 42, 143, 23, 0,
    107, 176, 87, 104, 183, 156, 193, 189, 90, 188, 65, 190, 17, 198, 7, 186, 161, 1, 124, 78, 125,
    170, 133, 174, 218, 67, 157, 75, 101, 89, 217, 62, 33, 141, 228, 25, 35, 91, 230, 4, 2, 13, 73,
    86, 167, 237, 84, 243, 44, 185, 66, 130, 110, 150, 142, 216, 88, 112, 36, 224, 136, 202, 76,
    94, 98, 175, 213,
];
// Source: modules/skcms/skcms.cc:1794-1850
// C memcmp preserves native object-representation equality, including signed-zero/NaN bits.
unsafe fn same_bytes(a: *const c_void, b: *const c_void, n: usize) -> bool {
    unsafe extern "C" {
        fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    }
    unsafe { memcmp(a, b, n) == 0 }
}
/// Caller retains readable ICC object representations and all borrowed profile table storage.
pub unsafe fn skcms_ApproximatelyEqualProfiles(a: &skcms_ICCProfile, b: &skcms_ICCProfile) -> bool {
    if std::ptr::eq(a, b)
        || unsafe {
            same_bytes(
                a as *const _ as *const c_void,
                b as *const _ as *const c_void,
                std::mem::size_of::<skcms_ICCProfile>(),
            )
        }
    {
        return true;
    }
    let cmyk = skcms_Signature::skcms_Signature_CMYK as u32;
    if (a.data_color_space == cmyk) != (b.data_color_space == cmyk) {
        return false;
    }
    let (mut fmt, mut npixels) = (skcms_PixelFormat::skcms_PixelFormat_RGB_888, 84);
    if a.data_color_space == cmyk {
        fmt = skcms_PixelFormat::skcms_PixelFormat_RGBA_8888;
        npixels = 63;
    }
    // Native CMYK uses63pixels, producing189RGB bytes, yet compares all252bytes below.
    // Its remaining63bytes are uninitialized: that native final boolean has undefined behavior.
    // Rust deliberately zero-initializes both tails as a safety adaptation, retaining the CMYK path.
    // No native-equivalence claim is made for that CMYK final comparison.
    let mut dstA = [0u8; 252];
    let mut dstB = [0u8; 252];
    if !unsafe {
        skcms_Transform(
            skcms_252_random_bytes.as_ptr().cast(),
            fmt,
            skcms_AlphaFormat::skcms_AlphaFormat_Unpremul,
            Some(a),
            dstA.as_mut_ptr().cast(),
            skcms_PixelFormat::skcms_PixelFormat_RGB_888,
            skcms_AlphaFormat::skcms_AlphaFormat_Unpremul,
            Some(skcms_XYZD50_profile()),
            npixels,
        )
    } {
        return false;
    }
    if !unsafe {
        skcms_Transform(
            skcms_252_random_bytes.as_ptr().cast(),
            fmt,
            skcms_AlphaFormat::skcms_AlphaFormat_Unpremul,
            Some(b),
            dstB.as_mut_ptr().cast(),
            skcms_PixelFormat::skcms_PixelFormat_RGB_888,
            skcms_AlphaFormat::skcms_AlphaFormat_Unpremul,
            Some(skcms_XYZD50_profile()),
            npixels,
        )
    } {
        return false;
    }
    for i in 0..252 {
        if (dstA[i] as i32 - dstB[i] as i32).abs() > 1 {
            return false;
        }
    }
    true
}
// Source: modules/skcms/skcms.cc:1852-1861
pub unsafe fn skcms_TRCs_AreApproximateInverse(
    profile: Option<&crate::modules::skcms::src::skcms_public::skcms_ICCProfile>,
    inv_tf: &skcms_TransferFunction,
) -> bool {
    let Some(profile) = profile else {
        return false;
    };
    if !profile.has_trc {
        return false;
    }
    unsafe {
        skcms_AreApproximateInverses(&profile.trc[0], inv_tf)
            && skcms_AreApproximateInverses(&profile.trc[1], inv_tf)
            && skcms_AreApproximateInverses(&profile.trc[2], inv_tf)
    }
}
// Source: modules/skcms/skcms.cc:1863-1865
// Source: modules/skcms/skcms.cc:1867-1867
fn is_zero_to_one(x: f32) -> bool {
    0.0 <= x && x <= 1.0
}
struct skcms_Vector3 {
    vals: [f32; 3],
}
// Source: modules/skcms/skcms.cc:1869-1877
fn mv_mul(m: &skcms_Matrix3x3, v: &skcms_Vector3) -> skcms_Vector3 {
    let mut dst = skcms_Vector3 { vals: [0.0; 3] };
    for row in 0..3 {
        dst.vals[row] =
            m.vals[row][0] * v.vals[0] + m.vals[row][1] * v.vals[1] + m.vals[row][2] * v.vals[2];
    }
    dst
}
// Source: modules/skcms/skcms.cc:1879-1884
// Source: modules/skcms/skcms.cc:1887-1887
// Source: modules/skcms/skcms.cc:1890-1890
// Source: modules/skcms/skcms.cc:1895-1904
// Source: modules/skcms/skcms.cc:1906-1907
// Source: modules/skcms/skcms.cc:1909-1915
// Source: modules/skcms/skcms.cc:1917-1918
pub fn skcms_AdaptToXYZD50(wx: f32, wy: f32, toXYZD50: Option<&mut skcms_Matrix3x3>) -> bool {
    if !is_zero_to_one(wx) || !is_zero_to_one(wy) || toXYZD50.is_none() {
        return false;
    }
    let toXYZD50 = toXYZD50.unwrap();
    let wXYZ = skcms_Vector3 {
        vals: [wx / wy, 1.0, (1.0 - wx - wy) / wy],
    };
    let wXYZD50 = skcms_Vector3 {
        vals: [0.96422, 1.0, 0.82521],
    };
    let xyz_to_lms = skcms_Matrix3x3 {
        vals: [
            [0.8951, 0.2664, -0.1614],
            [-0.7502, 1.7135, 0.0367],
            [0.0389, -0.0685, 1.0296],
        ],
    };
    let lms_to_xyz = skcms_Matrix3x3 {
        vals: [
            [0.9869929, -0.1470543, 0.1599627],
            [0.4323053, 0.5183603, 0.0492912],
            [-0.0085287, 0.0400428, 0.9684867],
        ],
    };
    let srcCone = mv_mul(&xyz_to_lms, &wXYZ);
    let dstCone = mv_mul(&xyz_to_lms, &wXYZD50);
    *toXYZD50 = skcms_Matrix3x3 {
        vals: [
            [dstCone.vals[0] / srcCone.vals[0], 0.0, 0.0],
            [0.0, dstCone.vals[1] / srcCone.vals[1], 0.0],
            [0.0, 0.0, dstCone.vals[2] / srcCone.vals[2]],
        ],
    };
    *toXYZD50 = skcms_Matrix3x3_concat(toXYZD50, &xyz_to_lms);
    *toXYZD50 = skcms_Matrix3x3_concat(&lms_to_xyz, toXYZD50);
    true
}
// Source: modules/skcms/skcms.cc:1920-1931
// Source: modules/skcms/skcms.cc:1934-1942
// Source: modules/skcms/skcms.cc:1945-1946
// Source: modules/skcms/skcms.cc:1948-1953
// Source: modules/skcms/skcms.cc:1955-1958
// Source: modules/skcms/skcms.cc:1960-1962
pub fn skcms_PrimariesToXYZD50(
    rx: f32,
    ry: f32,
    gx: f32,
    gy: f32,
    bx: f32,
    by: f32,
    wx: f32,
    wy: f32,
    toXYZD50: Option<&mut skcms_Matrix3x3>,
) -> bool {
    if !is_zero_to_one(rx)
        || !is_zero_to_one(ry)
        || !is_zero_to_one(gx)
        || !is_zero_to_one(gy)
        || !is_zero_to_one(bx)
        || !is_zero_to_one(by)
        || !is_zero_to_one(wx)
        || !is_zero_to_one(wy)
        || toXYZD50.is_none()
    {
        return false;
    }
    let toXYZD50 = toXYZD50.unwrap();
    let primaries = skcms_Matrix3x3 {
        vals: [
            [rx, gx, bx],
            [ry, gy, by],
            [1.0 - rx - ry, 1.0 - gx - gy, 1.0 - bx - by],
        ],
    };
    let mut primaries_inv = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    if !skcms_Matrix3x3_invert(&primaries, &mut primaries_inv) {
        return false;
    }
    let wXYZ = skcms_Vector3 {
        vals: [wx / wy, 1.0, (1.0 - wx - wy) / wy],
    };
    let XYZ = mv_mul(&primaries_inv, &wXYZ);
    let mut toXYZ = skcms_Matrix3x3 {
        vals: [
            [XYZ.vals[0], 0.0, 0.0],
            [0.0, XYZ.vals[1], 0.0],
            [0.0, 0.0, XYZ.vals[2]],
        ],
    };
    toXYZ = skcms_Matrix3x3_concat(&primaries, &toXYZ);
    let mut DXtoD50 = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    if !skcms_AdaptToXYZD50(wx, wy, Some(&mut DXtoD50)) {
        return false;
    }
    *toXYZD50 = skcms_Matrix3x3_concat(&DXtoD50, &toXYZ);
    true
}
// Source: modules/skcms/skcms.cc:1965-1974
// Source: modules/skcms/skcms.cc:1976-1981
// Source: modules/skcms/skcms.cc:1983-1985
// Source: modules/skcms/skcms.cc:1987-1989
// Source: modules/skcms/skcms.cc:1991-1994
// Source: modules/skcms/skcms.cc:1996-2001
// Source: modules/skcms/skcms.cc:2003-2011
// Source: modules/skcms/skcms.cc:2013-2020
pub fn skcms_Matrix3x3_invert(src: &skcms_Matrix3x3, dst: &mut skcms_Matrix3x3) -> bool {
    unsafe { matrix_invert_raw(src, dst) }
}
// Native output slot may begin uninitialized. No typed reference to that
// destination is formed until all original matrix component writes complete.
unsafe fn matrix_invert_raw(src: &skcms_Matrix3x3, dst: *mut skcms_Matrix3x3) -> bool {
    let a00 = src.vals[0][0] as f64;
    let a01 = src.vals[1][0] as f64;
    let a02 = src.vals[2][0] as f64;
    let a10 = src.vals[0][1] as f64;
    let a11 = src.vals[1][1] as f64;
    let a12 = src.vals[2][1] as f64;
    let a20 = src.vals[0][2] as f64;
    let a21 = src.vals[1][2] as f64;
    let a22 = src.vals[2][2] as f64;
    let mut b0 = a00 * a11 - a01 * a10;
    let mut b1 = a00 * a12 - a02 * a10;
    let mut b2 = a01 * a12 - a02 * a11;
    let mut b3 = a20;
    let mut b4 = a21;
    let mut b5 = a22;
    let determinant = b0 * b5 - b1 * b4 + b2 * b3;
    if determinant == 0.0 {
        return false;
    }
    let invdet = 1.0 / determinant;
    if invdet > f32::MAX as f64 || invdet < -(f32::MAX as f64) || !isfinitef_(invdet as f32) {
        return false;
    }
    b0 *= invdet;
    b1 *= invdet;
    b2 *= invdet;
    b3 *= invdet;
    b4 *= invdet;
    b5 *= invdet;
    let values = core::ptr::addr_of_mut!((*dst).vals).cast::<f32>();
    values.add(0).write((a11 * b5 - a12 * b4) as f32);
    values.add(3).write((a02 * b4 - a01 * b5) as f32);
    values.add(6).write(b2 as f32);
    values.add(1).write((a12 * b3 - a10 * b5) as f32);
    values.add(4).write((a00 * b5 - a02 * b3) as f32);
    values.add(7).write((-b1) as f32);
    values.add(2).write((a10 * b4 - a11 * b3) as f32);
    values.add(5).write((a01 * b3 - a00 * b4) as f32);
    values.add(8).write(b0 as f32);
    for r in 0..3 {
        for c in 0..3 {
            if !isfinitef_(*values.add(r * 3 + c)) {
                return false;
            }
        }
    }
    true
}
// Source: modules/skcms/skcms.cc:2022-2031
pub fn skcms_Matrix3x3_concat(a: &skcms_Matrix3x3, b: &skcms_Matrix3x3) -> skcms_Matrix3x3 {
    let mut m = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    for r in 0..3 {
        for c in 0..3 {
            m.vals[r][c] = a.vals[r][0] * b.vals[0][c]
                + a.vals[r][1] * b.vals[1][c]
                + a.vals[r][2] * b.vals[2][c];
        }
    }
    m
}
// Source: modules/skcms/skcms.cc:2036-2043
// Source: modules/skcms/skcms.cc:2045-2048
// Source: modules/skcms/skcms.cc:2050-2054
// Source: modules/skcms/skcms.cc:2056-2061
// Source: modules/skcms/skcms.cc:2063-2063
// Source: modules/skcms/skcms.cc:2069-2069
// Source: modules/skcms/skcms.cc:2074-2079
// Source: modules/skcms/skcms.cc:2082-2082
// Source: modules/skcms/skcms.cc:2087-2089
// Source: modules/skcms/skcms.cc:2103-2107
// Source: modules/skcms/skcms.cc:2114-2116
// Source: modules/skcms/skcms.cc:2118-2120
// Source: modules/skcms/skcms.cc:2124-2126
// Source: modules/skcms/skcms.cc:2128-2129
// Source: modules/skcms/skcms.cc:2134-2137
// Source: modules/skcms/skcms.cc:2139-2145
// Source: modules/skcms/skcms.cc:2147-2149
pub fn skcms_TransferFunction_invert(
    src: &skcms_TransferFunction,
    dst: &mut skcms_TransferFunction,
) -> bool {
    unsafe { transfer_function_invert_raw(src, dst) }
}
// Same native inversion writes directly to its original output slot. Raw output
// permits genuine uninitialized temporaries without manufacturing &mut TF.
unsafe fn transfer_function_invert_raw(
    src: &skcms_TransferFunction,
    dst: *mut skcms_TransferFunction,
) -> bool {
    let mut pq = TF_PQish {
        A: 0.0,
        B: 0.0,
        C: 0.0,
        D: 0.0,
        E: 0.0,
        F: 0.0,
    };
    let mut hlg = TF_HLGish {
        R: 0.0,
        G: 0.0,
        a: 0.0,
        b: 0.0,
        c: 0.0,
        K_minus_1: 0.0,
    };
    match classify(src, Some(&mut pq), Some(&mut hlg)) {
        skcms_TFType_Invalid | skcms_TFType_PQ | skcms_TFType_HLG => return false,
        skcms_TFType_sRGBish => {}
        skcms_TFType_PQish => {
            *dst = skcms_TransferFunction {
                g: TFKind_marker(skcms_TFType_PQish),
                a: -pq.A,
                b: pq.D,
                c: 1.0 / pq.F,
                d: pq.B,
                e: -pq.E,
                f: 1.0 / pq.C,
            };
            return true;
        }
        kind @ (skcms_TFType_HLGish | skcms_TFType_HLGinvish) => {
            *dst = skcms_TransferFunction {
                g: TFKind_marker(if kind == skcms_TFType_HLGish {
                    skcms_TFType_HLGinvish
                } else {
                    skcms_TFType_HLGish
                }),
                a: 1.0 / hlg.R,
                b: 1.0 / hlg.G,
                c: 1.0 / hlg.a,
                d: hlg.b,
                e: hlg.c,
                f: hlg.K_minus_1,
            };
            return true;
        }
    }
    debug_assert!(classify(src, None, None) == skcms_TFType_sRGBish);
    let mut inv = skcms_TransferFunction {
        g: 0.0,
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    let d_l = src.c * src.d + src.f;
    let d_r = powf_(src.a * src.d + src.b, src.g) + src.e;
    if crate::modules::skcms::src::skcms_internals::fabsf_(d_l - d_r) > 1.0 / 512.0 {
        return false;
    }
    inv.d = d_l;
    if inv.d > 0.0 {
        inv.c = 1.0 / src.c;
        inv.f = -src.f / src.c;
    }
    let k = powf_(src.a, -src.g);
    inv.g = 1.0 / src.g;
    inv.a = k;
    inv.b = -k * src.e;
    inv.e = -src.b / src.a;
    if inv.a < 0.0 {
        return false;
    }
    if inv.a * inv.d + inv.b < 0.0 {
        inv.b = -inv.a * inv.d;
    }
    if classify(&inv, None, None) != skcms_TFType_sRGBish {
        return false;
    }
    debug_assert!(inv.a >= 0.0);
    debug_assert!(inv.a * inv.d + inv.b >= 0.0);
    let mut s = skcms_TransferFunction_eval(src, 1.0);
    if !isfinitef_(s) {
        return false;
    }
    let sign = if s < 0.0 { -1.0 } else { 1.0 };
    s *= sign;
    if s < inv.d {
        inv.f = 1.0 - sign * inv.c * s;
    } else {
        inv.e = 1.0 - sign * powf_(inv.a * s + inv.b, inv.g);
    }
    *dst = inv;
    classify(&*dst, None, None) == skcms_TFType_sRGBish
}

use crate::modules::skcms::src::skcms_Transform::{baseline, Op};
use crate::modules::skcms::src::skcms_public::{
    skcms_CurveParametric, skcms_Matrix3x4, skcms_SetXYZD50, skcms_Signature,
};
use std::ffi::c_void;
// Source: modules/skcms/skcms.cc:2190-2216
unsafe fn rg_nonlinear(
    x: f32,
    curve: &skcms_Curve,
    tf: &skcms_TransferFunction,
    dfdP: &mut [f32; 3],
) -> f32 {
    unsafe {
        let y = eval_curve(curve, x);
        let (g, a, b, c, d, f) = (tf.g, tf.a, tf.b, tf.c, tf.d, tf.f);
        let Y = fmaxf_(a * y + b, 0.0);
        let D = a * d + b;
        debug_assert!(D >= 0.0);
        dfdP[0] = logf_(Y) * powf_(Y, g) - logf_(D) * powf_(D, g);
        dfdP[1] = y * g * powf_(Y, g - 1.0) - d * g * powf_(D, g - 1.0);
        dfdP[2] = g * powf_(Y, g - 1.0) - g * powf_(D, g - 1.0);
        let f_inv = powf_(Y, g) - powf_(D, g) + c * d + f;
        x - f_inv
    }
}
// Source: modules/skcms/skcms.cc:2218-2297
unsafe fn gauss_newton_step(
    curve: &skcms_Curve,
    tf: &mut skcms_TransferFunction,
    x0: f32,
    dx: f32,
    n: i32,
) -> bool {
    unsafe {
        let mut lhs = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        let mut rhs = skcms_Vector3 { vals: [0.0; 3] };
        for i in 0..n {
            let x = x0 + i as f32 * dx;
            let mut dfdP = [0.0; 3];
            let resid = rg_nonlinear(x, curve, tf, &mut dfdP);
            for r in 0..3 {
                for c in 0..3 {
                    lhs.vals[r][c] += dfdP[r] * dfdP[c];
                }
                rhs.vals[r] += dfdP[r] * resid;
            }
        }
        for k in 0..3 {
            if lhs.vals[0][k] == 0.0
                && lhs.vals[1][k] == 0.0
                && lhs.vals[2][k] == 0.0
                && lhs.vals[k][0] == 0.0
                && lhs.vals[k][1] == 0.0
                && lhs.vals[k][2] == 0.0
            {
                lhs.vals[k][k] = 1.0;
            }
        }
        let mut lhs_inv = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        if !skcms_Matrix3x3_invert(&lhs, &mut lhs_inv) {
            return false;
        }
        let dP = mv_mul(&lhs_inv, &rhs);
        tf.g += dP.vals[0];
        tf.a += dP.vals[1];
        tf.b += dP.vals[2];
        isfinitef_(tf.g) && isfinitef_(tf.a) && isfinitef_(tf.b)
    }
}
// Source: modules/skcms/skcms.cc:2299-2312
unsafe fn max_roundtrip_error_checked(curve: &skcms_Curve, tf_inv: &skcms_TransferFunction) -> f32 {
    unsafe {
        let mut tf = skcms_TransferFunction {
            g: 0.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
        };
        if !skcms_TransferFunction_invert(tf_inv, &mut tf)
            || classify(&tf, None, None) != skcms_TFType_sRGBish
        {
            return f32::INFINITY;
        }
        let mut tf_inv_again = tf;
        if !skcms_TransferFunction_invert(&tf, &mut tf_inv_again) {
            return f32::INFINITY;
        }
        skcms_MaxRoundtripError(curve, &tf_inv_again)
    }
}
// Source: modules/skcms/skcms.cc:2315-2372
unsafe fn fit_nonlinear(
    curve: &skcms_Curve,
    l: i32,
    n: i32,
    tf: &mut skcms_TransferFunction,
) -> bool {
    unsafe {
        let fixup_tf = |tf: &mut skcms_TransferFunction| {
            if tf.a < 0.0 {
                return false;
            }
            if tf.a * tf.d + tf.b < 0.0 {
                tf.b = -tf.a * tf.d;
            }
            debug_assert!(tf.a >= 0.0 && tf.a * tf.d + tf.b >= 0.0);
            tf.e = tf.c * tf.d + tf.f - powf_(tf.a * tf.d + tf.b, tf.g);
            isfinitef_(tf.e)
        };
        if !fixup_tf(tf) {
            return false;
        }
        let dx = 1.0 / (n - 1) as f32;
        let mut best_tf = *tf;
        let mut best_max_error = f32::INFINITY;
        let init_error = max_roundtrip_error_checked(curve, tf);
        if init_error < best_max_error {
            best_max_error = init_error;
            best_tf = *tf;
        }
        for _j in 0..8 {
            if !gauss_newton_step(curve, tf, l as f32 * dx, dx, n - l) || !fixup_tf(tf) {
                *tf = best_tf;
                return isfinitef_(best_max_error);
            }
            let max_error = max_roundtrip_error_checked(curve, tf);
            if max_error < best_max_error {
                best_max_error = max_error;
                best_tf = *tf;
            }
        }
        *tf = best_tf;
        isfinitef_(best_max_error)
    }
}
// Source: modules/skcms/skcms.cc:2374-2473
pub unsafe fn skcms_ApproximateCurve(
    curve: *const skcms_Curve,
    approx: *mut skcms_TransferFunction,
    max_error: *mut f32,
) -> bool {
    unsafe {
        if curve.is_null() || approx.is_null() || max_error.is_null() {
            return false;
        }
        let curve = &*curve;
        if curve.table.table_entries == 0 {
            return false;
        }
        if curve.table.table_entries == 1 || curve.table.table_entries > kMaxTableEntries {
            return false;
        }
        let n = curve.table.table_entries as i32;
        let dx = 1.0 / (n - 1) as f32;
        *max_error = f32::INFINITY;
        let tolerances = [1.5 / 65535.0, 1.0 / 512.0];
        for tolerance in tolerances {
            // Uninitialized native coefficients are initialized here solely for Rust validity;
            // each coefficient used below is set by its native branch before evaluation.
            let mut tf = skcms_TransferFunction {
                g: 0.0,
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            };
            let mut tf_inv = tf;
            tf.f = 0.0;
            let l = fit_linear(curve, n, tolerance, &mut tf.c, &mut tf.d, None);
            if l == n {
                tf.g = 1.0;
                tf.a = tf.c;
                tf.b = tf.f;
                tf.c = 0.0;
                tf.d = 0.0;
                tf.e = 0.0;
                tf.f = 0.0;
            } else if l == n - 1 {
                tf.g = 1.0;
                tf.a = (eval_curve(curve, (n - 1) as f32 * dx)
                    - eval_curve(curve, (n - 2) as f32 * dx))
                    / dx;
                tf.b = eval_curve(curve, (n - 2) as f32 * dx) - tf.a * (n - 2) as f32 * dx;
                tf.e = 0.0;
            } else {
                let mid = (l + n) / 2;
                let mid_x = mid as f32 / (n - 1) as f32;
                let mid_y = eval_curve(curve, mid_x);
                tf.g = log2f_(mid_y) / log2f_(mid_x);
                tf.a = 1.0;
                tf.b = 0.0;
                tf.e = tf.c * tf.d + tf.f - powf_(tf.a * tf.d + tf.b, tf.g);
                if !skcms_TransferFunction_invert(&tf, &mut tf_inv)
                    || !fit_nonlinear(curve, l, n, &mut tf_inv)
                {
                    continue;
                }
                if !skcms_TransferFunction_invert(&tf_inv, &mut tf) {
                    debug_assert!(false);
                    continue;
                }
            }
            if classify(&tf, None, None) != skcms_TFType_sRGBish {
                continue;
            }
            if !skcms_TransferFunction_invert(&tf, &mut tf_inv) {
                continue;
            }
            let err = skcms_MaxRoundtripError(curve, &tf_inv);
            if *max_error > err {
                *max_error = err;
                *approx = tf;
            }
        }
        isfinitef_(*max_error)
    }
}

// Source: modules/skcms/skcms.cc:2475-2475
// Only the selected ARM64 CPU variant is represented; x86 runtime dispatch is outside this configuration.
#[derive(Clone, Copy)]
enum CpuType {
    Baseline,
}
// Source: modules/skcms/skcms.cc:2477-2480
#[cfg(target_arch = "aarch64")]
fn cpu_type() -> CpuType {
    CpuType::Baseline
}
// Source: modules/skcms/skcms.cc:2537-2540
fn tf_is_gamma(tf: &skcms_TransferFunction) -> bool {
    tf.g > 0.0
        && tf.a == 1.0
        && tf.b == 0.0
        && tf.c == 0.0
        && tf.d == 0.0
        && tf.e == 0.0
        && tf.f == 0.0
}
// Source: modules/skcms/skcms.cc:2542-2545
#[repr(C)]
#[derive(Clone, Copy)]
struct OpAndArg {
    op: Op,
    arg: *const c_void,
}
// Source: modules/skcms/skcms.cc:2547-2585
unsafe fn select_curve_op(curve: &skcms_Curve, channel: usize) -> OpAndArg {
    let kops = [
        [
            Op::gamma_r,
            Op::tf_r,
            Op::pq_r,
            Op::hlg_r,
            Op::hlginv_r,
            Op::table_r,
        ],
        [
            Op::gamma_g,
            Op::tf_g,
            Op::pq_g,
            Op::hlg_g,
            Op::hlginv_g,
            Op::table_g,
        ],
        [
            Op::gamma_b,
            Op::tf_b,
            Op::pq_b,
            Op::hlg_b,
            Op::hlginv_b,
            Op::table_b,
        ],
        [
            Op::gamma_a,
            Op::tf_a,
            Op::pq_a,
            Op::hlg_a,
            Op::hlginv_a,
            Op::table_a,
        ],
    ];
    let op = kops[channel];
    unsafe {
        if curve.table.table_entries == 0 {
            let noop = OpAndArg {
                op: Op::load_a8,
                arg: std::ptr::null(),
            };
            let tf = &curve.parametric.parametric;
            if tf_is_gamma(tf) {
                return if tf.g != 1.0 {
                    OpAndArg {
                        op: op[0],
                        arg: tf as *const _ as *const c_void,
                    }
                } else {
                    noop
                };
            }
            let index = match classify(tf, None, None) {
                skcms_TFType_Invalid | skcms_TFType_PQ | skcms_TFType_HLG => return noop,
                skcms_TFType_sRGBish => 1,
                skcms_TFType_PQish => 2,
                skcms_TFType_HLGish => 3,
                skcms_TFType_HLGinvish => 4,
            };
            return OpAndArg {
                op: op[index],
                arg: tf as *const _ as *const c_void,
            };
        }
        assert!(curve.table.table_entries <= kMaxTableEntries);
        OpAndArg {
            op: op[5],
            arg: curve as *const _ as *const c_void,
        }
    }
}
// Source: modules/skcms/skcms.cc:2588-2633
unsafe fn select_curve_ops(
    curves: *const skcms_Curve,
    numChannels: i32,
    ops: &mut [OpAndArg; 4],
) -> i32 {
    let mut cursor = 0;
    for index in (0..numChannels).rev() {
        let curve = unsafe { &*curves.add(index as usize) };
        if unsafe { curve.table.table_entries } > kMaxTableEntries {
            return -1;
        }
        ops[cursor] = unsafe { select_curve_op(curve, index as usize) };
        if !ops[cursor].arg.is_null() {
            cursor += 1;
        }
    }
    if cursor >= 3 {
        let fusable = [
            [Op::gamma_r, Op::gamma_g, Op::gamma_b, Op::gamma_rgb],
            [Op::tf_r, Op::tf_g, Op::tf_b, Op::tf_rgb],
            [Op::pq_r, Op::pq_g, Op::pq_b, Op::pq_rgb],
            [Op::hlg_r, Op::hlg_g, Op::hlg_b, Op::hlg_rgb],
            [Op::hlginv_r, Op::hlginv_g, Op::hlginv_b, Op::hlginv_rgb],
        ];
        let posR = cursor - 1;
        let posG = cursor - 2;
        let posB = cursor - 3;
        for f in fusable {
            if ops[posR].op == f[0]
                && ops[posG].op == f[1]
                && ops[posB].op == f[2]
                && unsafe {
                    same_bytes(
                        ops[posR].arg,
                        ops[posG].arg,
                        std::mem::size_of::<skcms_TransferFunction>(),
                    )
                }
                && unsafe {
                    same_bytes(
                        ops[posR].arg,
                        ops[posB].arg,
                        std::mem::size_of::<skcms_TransferFunction>(),
                    )
                }
            {
                ops[posB].op = f[3];
                cursor -= 2;
                break;
            }
        }
    }
    cursor as i32
}
// Source: modules/skcms/skcms.cc:2635-2661
fn bytes_per_pixel(fmt: skcms_PixelFormat) -> usize {
    match (fmt as i32) >> 1 {
        x if x == (skcms_PixelFormat::skcms_PixelFormat_A_8 as i32) >> 1 => 1,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_G_8 as i32) >> 1 => 1,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_GA_88 as i32) >> 1 => 2,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_ABGR_4444 as i32) >> 1 => 2,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_565 as i32) >> 1 => 2,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_888 as i32) >> 1 => 3,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_8888 as i32) >> 1 => 4,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_8888_sRGB as i32) >> 1 => 4,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_1010102 as i32) >> 1 => 4,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_101010x_XR as i32) >> 1 => 4,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_161616LE as i32) >> 1 => 6,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_10101010_XR as i32) >> 1 => 8,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_16161616LE as i32) >> 1 => 8,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_161616BE as i32) >> 1 => 6,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_16161616BE as i32) >> 1 => 8,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_hhh_Norm as i32) >> 1 => 6,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_hhhh_Norm as i32) >> 1 => 8,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_hhh as i32) >> 1 => 6,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_hhhh as i32) >> 1 => 8,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGB_fff as i32) >> 1 => 12,
        x if x == (skcms_PixelFormat::skcms_PixelFormat_RGBA_ffff as i32) >> 1 => 16,
        _ => {
            assert!(false);
            0
        }
    }
}
// Source: modules/skcms/skcms.cc:2664-2676
const kTransferCicpIdPQ: u8 = 16;
const kTransferCicpIdHLG: u8 = 18;
fn has_cicp_pq_trc(p: &skcms_ICCProfile) -> bool {
    p.has_CICP && p.CICP.transfer_characteristics == kTransferCicpIdPQ
}
fn has_cicp_hlg_trc(p: &skcms_ICCProfile) -> bool {
    p.has_CICP && p.CICP.transfer_characteristics == kTransferCicpIdHLG
}
// Source: modules/skcms/skcms.cc:2678-2689
fn set_reference_pq_ish_trc(tf: &mut skcms_TransferFunction) {
    skcms_TransferFunction_makePQish(
        tf,
        -107.0 / 128.0,
        1.0,
        32.0 / 2523.0,
        2413.0 / 128.0,
        -2392.0 / 128.0,
        8192.0 / 1305.0,
    );
    let w = 10000.0 / 203.0;
    let ws = powf_(w, 1.0 / tf.f);
    tf.a = ws * tf.a;
    tf.b = ws * tf.b;
}
// Source: modules/skcms/skcms.cc:2696-2700
fn set_sdr_hlg_ish_trc(tf: &mut skcms_TransferFunction) {
    crate::modules::skcms::src::skcms_public::skcms_TransferFunction_makeHLGish(
        tf,
        2.0,
        2.0,
        1.0 / 0.17883277,
        0.28466892,
        0.55991073,
    );
    tf.f = 1.0 / 12.0 - 1.0;
}
// Source: modules/skcms/skcms.cc:2702-2752
fn prep_for_destination(
    profile: &skcms_ICCProfile,
    fromXYZD50: &mut skcms_Matrix3x3,
    inv: &mut [skcms_Curve; 3],
    dst_using_B2A: &mut bool,
    dst_using_hlg_ootf: &mut bool,
) -> bool {
    let has_xyzd50 = profile.has_toXYZD50 && skcms_Matrix3x3_invert(&profile.toXYZD50, fromXYZD50);
    *dst_using_B2A = false;
    *dst_using_hlg_ootf = false;
    unsafe {
        if has_cicp_pq_trc(profile) && has_xyzd50 {
            let mut trc_pq = skcms_TransferFunction {
                g: 0.0,
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            };
            set_reference_pq_ish_trc(&mut trc_pq);
            for c in inv {
                skcms_TransferFunction_invert(&trc_pq, &mut c.parametric.parametric);
            }
            return true;
        }
        if has_cicp_hlg_trc(profile) && has_xyzd50 {
            let mut trc_hlg = skcms_TransferFunction {
                g: 0.0,
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            };
            set_sdr_hlg_ish_trc(&mut trc_hlg);
            for c in inv {
                skcms_TransferFunction_invert(&trc_hlg, &mut c.parametric.parametric);
            }
            *dst_using_hlg_ootf = true;
            return true;
        }
        if profile.has_B2A {
            *dst_using_B2A = true;
            return true;
        }
        has_xyzd50
            && profile.has_trc
            && profile.trc[0].table.table_entries == 0
            && profile.trc[1].table.table_entries == 0
            && profile.trc[2].table.table_entries == 0
            && skcms_TransferFunction_invert(
                &profile.trc[0].parametric.parametric,
                &mut inv[0].parametric.parametric,
            )
            && skcms_TransferFunction_invert(
                &profile.trc[1].parametric.parametric,
                &mut inv[1].parametric.parametric,
            )
            && skcms_TransferFunction_invert(
                &profile.trc[2].parametric.parametric,
                &mut inv[2].parametric.parametric,
            )
    }
}

// Source: modules/skcms/skcms.cc:2755-3139
/// Caller retains borrowed ICC tables/CLUTs and valid source/destination pixel storage for this call.
/// Profiles use native object-representation equality; no Rust owner or buffer lifetime is erased.
#[cfg(target_arch = "aarch64")]
pub unsafe fn skcms_Transform(
    src: *const c_void,
    srcFmt: skcms_PixelFormat,
    mut srcAlpha: skcms_AlphaFormat,
    srcProfile: Option<&skcms_ICCProfile>,
    dst: *mut c_void,
    dstFmt: skcms_PixelFormat,
    mut dstAlpha: skcms_AlphaFormat,
    dstProfile: Option<&skcms_ICCProfile>,
    nz: usize,
) -> bool {
    use skcms_AlphaFormat::*;
    use skcms_PixelFormat::*;
    let dst_bpp = bytes_per_pixel(dstFmt);
    let src_bpp = bytes_per_pixel(srcFmt);
    if nz.wrapping_mul(dst_bpp) > i32::MAX as usize || nz.wrapping_mul(src_bpp) > i32::MAX as usize
    {
        return false;
    }
    let n = nz as i32;
    let srcProfile = match srcProfile {
        Some(p) => p,
        None => skcms_sRGB_profile(),
    };
    let mut dstProfile = (match dstProfile {
        Some(p) => p,
        None => skcms_sRGB_profile(),
    }) as *const skcms_ICCProfile;
    if dst.cast_const() == src && dst_bpp != src_bpp {
        return false;
    }
    // Rust initializes only otherwise-unused enum/context slots; only the actual emitted prefix executes.
    let mut program = [Op::load_a8; 32];
    let mut context = [std::ptr::null::<c_void>(); 32];
    let mut cursor = 0usize;
    macro_rules! add_op_ctx {
        ($o:expr,$c:expr) => {{
            program[cursor] = $o;
            context[cursor] = $c as *const _ as *const c_void;
            cursor += 1;
        }};
    }
    macro_rules! add_op {
        ($o:expr) => {{
            program[cursor] = $o;
            context[cursor] = std::ptr::null();
            cursor += 1;
        }};
    }
    macro_rules! add_curve_ops {
        ($curves:expr,$num:expr) => {{
            let num = $num;
            assert!(num <= 4);
            let mut oa = [OpAndArg {
                op: Op::load_a8,
                arg: std::ptr::null(),
            }; 4];
            let numOps = unsafe { select_curve_ops($curves, num, &mut oa) };
            if numOps < 0 {
                false
            } else {
                for item in &oa[..numOps as usize] {
                    add_op_ctx!(item.op, item.arg);
                }
                true
            }
        }};
    }
    let zero_tf = skcms_TransferFunction {
        g: 0.0,
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    let mut src_cicp_trc = zero_tf;
    let mut dst_curves = [skcms_Curve {
        parametric: skcms_CurveParametric {
            alias_of_table_entries: 0,
            parametric: zero_tf,
        },
    }; 3];
    let mut dst_from_xyz = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    let mut dst_from_src = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    match (srcFmt as i32) >> 1 {
        x if x == (skcms_PixelFormat_A_8 as i32) >> 1 => add_op!(Op::load_a8),
        x if x == (skcms_PixelFormat_G_8 as i32) >> 1 => add_op!(Op::load_g8),
        x if x == (skcms_PixelFormat_GA_88 as i32) >> 1 => add_op!(Op::load_ga88),
        x if x == (skcms_PixelFormat_ABGR_4444 as i32) >> 1 => add_op!(Op::load_4444),
        x if x == (skcms_PixelFormat_RGB_565 as i32) >> 1 => add_op!(Op::load_565),
        x if x == (skcms_PixelFormat_RGB_888 as i32) >> 1 => add_op!(Op::load_888),
        x if x == (skcms_PixelFormat_RGBA_8888 as i32) >> 1 => add_op!(Op::load_8888),
        x if x == (skcms_PixelFormat_RGBA_1010102 as i32) >> 1 => add_op!(Op::load_1010102),
        x if x == (skcms_PixelFormat_RGB_101010x_XR as i32) >> 1 => add_op!(Op::load_101010x_XR),
        x if x == (skcms_PixelFormat_RGBA_10101010_XR as i32) >> 1 => add_op!(Op::load_10101010_XR),
        x if x == (skcms_PixelFormat_RGB_161616LE as i32) >> 1 => add_op!(Op::load_161616LE),
        x if x == (skcms_PixelFormat_RGBA_16161616LE as i32) >> 1 => add_op!(Op::load_16161616LE),
        x if x == (skcms_PixelFormat_RGB_161616BE as i32) >> 1 => add_op!(Op::load_161616BE),
        x if x == (skcms_PixelFormat_RGBA_16161616BE as i32) >> 1 => add_op!(Op::load_16161616BE),
        x if x == (skcms_PixelFormat_RGB_hhh_Norm as i32) >> 1 => add_op!(Op::load_hhh),
        x if x == (skcms_PixelFormat_RGBA_hhhh_Norm as i32) >> 1 => add_op!(Op::load_hhhh),
        x if x == (skcms_PixelFormat_RGB_hhh as i32) >> 1 => add_op!(Op::load_hhh),
        x if x == (skcms_PixelFormat_RGBA_hhhh as i32) >> 1 => add_op!(Op::load_hhhh),
        x if x == (skcms_PixelFormat_RGB_fff as i32) >> 1 => add_op!(Op::load_fff),
        x if x == (skcms_PixelFormat_RGBA_ffff as i32) >> 1 => add_op!(Op::load_ffff),
        x if x == (skcms_PixelFormat_RGBA_8888_sRGB as i32) >> 1 => {
            add_op!(Op::load_8888);
            add_op_ctx!(Op::tf_rgb, skcms_sRGB_TransferFunction());
        }
        _ => return false,
    }
    if srcFmt == skcms_PixelFormat_RGB_hhh_Norm || srcFmt == skcms_PixelFormat_RGBA_hhhh_Norm {
        add_op!(Op::clamp);
    }
    if srcFmt as i32 & 1 != 0 {
        add_op!(Op::swap_rb);
    }
    let mut gray_dst_profile = std::mem::MaybeUninit::<skcms_ICCProfile>::uninit();
    // Preserve the exact native (unshifted) gray case labels in this shifted switch.
    match (dstFmt as i32) >> 1 {
        x if x == skcms_PixelFormat_G_8 as i32 || x == skcms_PixelFormat_GA_88 as i32 => unsafe {
            gray_dst_profile.write(*dstProfile);
            let gray = gray_dst_profile.assume_init_mut();
            skcms_SetXYZD50(gray, &skcms_XYZD50_profile().toXYZD50);
            dstProfile = gray as *const _;
        },
        _ => {}
    }
    if srcProfile.data_color_space == skcms_Signature::skcms_Signature_CMYK as u32 {
        add_op!(Op::invert);
        srcAlpha = skcms_AlphaFormat_Unpremul;
    }
    if srcAlpha == skcms_AlphaFormat_Opaque {
        add_op!(Op::force_opaque);
    } else if srcAlpha == skcms_AlphaFormat_PremulAsEncoded {
        add_op!(Op::unpremul);
    }
    if !std::ptr::eq(unsafe { &*dstProfile }, srcProfile) {
        let (mut src_using_A2B, mut src_using_hlg_ootf, mut dst_using_B2A, mut dst_using_hlg_ootf) =
            (false, false, false, false);
        if !prep_for_destination(
            unsafe { &*dstProfile },
            &mut dst_from_xyz,
            &mut dst_curves,
            &mut dst_using_B2A,
            &mut dst_using_hlg_ootf,
        ) {
            return false;
        }
        if has_cicp_pq_trc(srcProfile) && srcProfile.has_toXYZD50 {
            set_reference_pq_ish_trc(&mut src_cicp_trc);
            add_op_ctx!(Op::pq_rgb, &src_cicp_trc);
        } else if has_cicp_hlg_trc(srcProfile) && srcProfile.has_toXYZD50 {
            src_using_hlg_ootf = true;
            set_sdr_hlg_ish_trc(&mut src_cicp_trc);
            add_op_ctx!(Op::hlg_rgb, &src_cicp_trc);
        } else if srcProfile.has_A2B {
            src_using_A2B = true;
            let a2b = &srcProfile.A2B;
            if a2b.input_channels != 0 {
                if !add_curve_ops!(a2b.input_curves.as_ptr(), a2b.input_channels as i32) {
                    return false;
                }
                add_op!(Op::clamp);
                add_op_ctx!(Op::clut_A2B, a2b);
            }
            if a2b.matrix_channels == 3 {
                if !add_curve_ops!(a2b.matrix_curves.as_ptr(), 3) {
                    return false;
                }
                static I: skcms_Matrix3x4 = skcms_Matrix3x4 {
                    vals: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                    ],
                };
                if !unsafe {
                    same_bytes(
                        &I as *const _ as *const c_void,
                        &a2b.matrix as *const _ as *const c_void,
                        std::mem::size_of_val(&I),
                    )
                } {
                    add_op_ctx!(Op::matrix_3x4, &a2b.matrix);
                }
            }
            if a2b.output_channels == 3 && !add_curve_ops!(a2b.output_curves.as_ptr(), 3) {
                return false;
            }
            if srcProfile.pcs == skcms_Signature::skcms_Signature_Lab as u32 {
                add_op!(Op::lab_to_xyz);
            }
        } else if srcProfile.has_trc && srcProfile.has_toXYZD50 {
            if !add_curve_ops!(srcProfile.trc.as_ptr(), 3) {
                return false;
            }
        } else {
            return false;
        }
        assert!(srcProfile.has_A2B || srcProfile.has_toXYZD50);
        if dst_using_B2A {
            if !src_using_A2B {
                add_op_ctx!(Op::matrix_3x3, &srcProfile.toXYZD50);
                if src_using_hlg_ootf {
                    add_op!(Op::hlg_ootf_scale);
                }
            }
            if unsafe { (*dstProfile).pcs } == skcms_Signature::skcms_Signature_Lab as u32 {
                add_op!(Op::xyz_to_lab);
            }
            let b2a = unsafe { &(*dstProfile).B2A };
            if b2a.input_channels == 3 && !add_curve_ops!(b2a.input_curves.as_ptr(), 3) {
                return false;
            }
            if b2a.matrix_channels == 3 {
                static I: skcms_Matrix3x4 = skcms_Matrix3x4 {
                    vals: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                    ],
                };
                if !unsafe {
                    same_bytes(
                        &I as *const _ as *const c_void,
                        &b2a.matrix as *const _ as *const c_void,
                        std::mem::size_of_val(&I),
                    )
                } {
                    add_op_ctx!(Op::matrix_3x4, &b2a.matrix);
                }
                if !add_curve_ops!(b2a.matrix_curves.as_ptr(), 3) {
                    return false;
                }
            }
            if b2a.output_channels != 0 {
                add_op!(Op::clamp);
                add_op_ctx!(Op::clut_B2A, b2a);
                if !add_curve_ops!(b2a.output_curves.as_ptr(), b2a.output_channels as i32) {
                    return false;
                }
            }
        } else {
            if src_using_hlg_ootf != dst_using_hlg_ootf {
                if !src_using_A2B {
                    add_op_ctx!(Op::matrix_3x3, &srcProfile.toXYZD50);
                }
                if src_using_hlg_ootf {
                    add_op!(Op::hlg_ootf_scale);
                }
                if dst_using_hlg_ootf {
                    add_op!(Op::hlginv_ootf_scale);
                }
                add_op_ctx!(Op::matrix_3x3, &dst_from_xyz);
            } else if src_using_A2B {
                add_op_ctx!(Op::matrix_3x3, &dst_from_xyz);
            } else {
                let to_xyz = &srcProfile.toXYZD50;
                if !unsafe {
                    same_bytes(
                        &(*dstProfile).toXYZD50 as *const _ as *const c_void,
                        to_xyz as *const _ as *const c_void,
                        std::mem::size_of::<skcms_Matrix3x3>(),
                    )
                } {
                    dst_from_src = skcms_Matrix3x3_concat(&dst_from_xyz, to_xyz);
                    add_op_ctx!(Op::matrix_3x3, &dst_from_src);
                }
            }
            let mut oa = [OpAndArg {
                op: Op::load_a8,
                arg: std::ptr::null(),
            }; 4];
            let numOps = unsafe { select_curve_ops(dst_curves.as_ptr(), 3, &mut oa) };
            assert!(numOps >= 0);
            for item in &oa[..numOps as usize] {
                assert!(!matches!(
                    item.op,
                    Op::table_r | Op::table_g | Op::table_b | Op::table_a
                ));
                add_op_ctx!(item.op, item.arg);
            }
        }
    }
    if (dstFmt as i32) < skcms_PixelFormat_RGB_hhh as i32 {
        add_op!(Op::clamp);
    }
    if unsafe { (*dstProfile).data_color_space } == skcms_Signature::skcms_Signature_CMYK as u32 {
        add_op!(Op::invert);
        dstAlpha = skcms_AlphaFormat_Unpremul;
    }
    if dstAlpha == skcms_AlphaFormat_Opaque {
        add_op!(Op::force_opaque);
    } else if dstAlpha == skcms_AlphaFormat_PremulAsEncoded {
        add_op!(Op::premul);
    }
    if dstFmt as i32 & 1 != 0 {
        add_op!(Op::swap_rb);
    }
    match (dstFmt as i32) >> 1 {
        x if x == (skcms_PixelFormat_A_8 as i32) >> 1 => add_op!(Op::store_a8),
        x if x == (skcms_PixelFormat_G_8 as i32) >> 1 => add_op!(Op::store_g8),
        x if x == (skcms_PixelFormat_GA_88 as i32) >> 1 => add_op!(Op::store_ga88),
        x if x == (skcms_PixelFormat_ABGR_4444 as i32) >> 1 => add_op!(Op::store_4444),
        x if x == (skcms_PixelFormat_RGB_565 as i32) >> 1 => add_op!(Op::store_565),
        x if x == (skcms_PixelFormat_RGB_888 as i32) >> 1 => add_op!(Op::store_888),
        x if x == (skcms_PixelFormat_RGBA_8888 as i32) >> 1 => add_op!(Op::store_8888),
        x if x == (skcms_PixelFormat_RGBA_1010102 as i32) >> 1 => add_op!(Op::store_1010102),
        x if x == (skcms_PixelFormat_RGB_161616LE as i32) >> 1 => add_op!(Op::store_161616LE),
        x if x == (skcms_PixelFormat_RGBA_16161616LE as i32) >> 1 => add_op!(Op::store_16161616LE),
        x if x == (skcms_PixelFormat_RGB_161616BE as i32) >> 1 => add_op!(Op::store_161616BE),
        x if x == (skcms_PixelFormat_RGBA_16161616BE as i32) >> 1 => add_op!(Op::store_16161616BE),
        x if x == (skcms_PixelFormat_RGB_hhh_Norm as i32) >> 1 => add_op!(Op::store_hhh),
        x if x == (skcms_PixelFormat_RGBA_hhhh_Norm as i32) >> 1 => add_op!(Op::store_hhhh),
        x if x == (skcms_PixelFormat_RGB_101010x_XR as i32) >> 1 => add_op!(Op::store_101010x_XR),
        x if x == (skcms_PixelFormat_RGBA_10101010_XR as i32) >> 1 => {
            add_op!(Op::store_10101010_XR)
        }
        x if x == (skcms_PixelFormat_RGB_hhh as i32) >> 1 => add_op!(Op::store_hhh),
        x if x == (skcms_PixelFormat_RGBA_hhhh as i32) >> 1 => add_op!(Op::store_hhhh),
        x if x == (skcms_PixelFormat_RGB_fff as i32) >> 1 => add_op!(Op::store_fff),
        x if x == (skcms_PixelFormat_RGBA_ffff as i32) >> 1 => add_op!(Op::store_ffff),
        x if x == (skcms_PixelFormat_RGBA_8888_sRGB as i32) >> 1 => {
            add_op_ctx!(Op::tf_rgb, skcms_sRGB_Inverse_TransferFunction());
            add_op!(Op::store_8888);
        }
        _ => return false,
    }
    assert!(cursor <= program.len());
    assert!(cursor <= context.len());
    let run = baseline::run_program;
    match cpu_type() {
        CpuType::Baseline => {}
    }
    unsafe {
        run(
            program.as_ptr(),
            context.as_ptr(),
            cursor as isize,
            src.cast(),
            dst.cast(),
            n,
            src_bpp,
            dst_bpp,
        )
    };
    true
}
// Source: modules/skcms/skcms.cc:3141-3151
fn assert_usable_as_destination(profile: &skcms_ICCProfile) {
    #[cfg(sk_native_ndebug)]
    let _ = profile;
    #[cfg(not(sk_native_ndebug))]
    {
        let mut fromXYZD50 = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        let zero = skcms_Curve {
            parametric: skcms_CurveParametric {
                alias_of_table_entries: 0,
                parametric: skcms_TransferFunction {
                    g: 0.0,
                    a: 0.0,
                    b: 0.0,
                    c: 0.0,
                    d: 0.0,
                    e: 0.0,
                    f: 0.0,
                },
            },
        };
        let mut inv = [zero; 3];
        let mut useB2A = false;
        let mut useHlgOotf = false;
        assert!(prep_for_destination(
            profile,
            &mut fromXYZD50,
            &mut inv,
            &mut useB2A,
            &mut useHlgOotf
        ));
    }
}
// Source: modules/skcms/skcms.cc:3153-3184
pub fn skcms_MakeUsableAsDestination(profile: &mut skcms_ICCProfile) -> bool {
    unsafe {
        if !profile.has_B2A {
            let mut fromXYZD50 = skcms_Matrix3x3 {
                vals: [[0.0; 3]; 3],
            };
            if !profile.has_trc
                || !profile.has_toXYZD50
                || !skcms_Matrix3x3_invert(&profile.toXYZD50, &mut fromXYZD50)
            {
                return false;
            }
            let mut tf = [skcms_TransferFunction {
                g: 0.0,
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            }; 3];
            for i in 0..3 {
                let mut inv = tf[i];
                if profile.trc[i].table.table_entries == 0
                    && skcms_TransferFunction_invert(
                        &profile.trc[i].parametric.parametric,
                        &mut inv,
                    )
                {
                    tf[i] = profile.trc[i].parametric.parametric;
                    continue;
                }
                let mut max_error = 0.0;
                if !skcms_ApproximateCurve(&profile.trc[i], &mut tf[i], &mut max_error) {
                    return false;
                }
            }
            for i in 0..3 {
                profile.trc[i].parametric.alias_of_table_entries = 0;
                profile.trc[i].parametric.parametric = tf[i];
            }
        }
        assert_usable_as_destination(profile);
        true
    }
}
// Source: modules/skcms/skcms.cc:3186-3221
pub fn skcms_MakeUsableAsDestinationWithSingleCurve(profile: &mut skcms_ICCProfile) -> bool {
    unsafe {
        let mut result = *profile;
        result.has_B2A = false;
        if !skcms_MakeUsableAsDestination(&mut result) {
            return false;
        }
        let mut best_tf = 0;
        let mut min_max_error = f32::INFINITY;
        for i in 0..3 {
            let mut inv = skcms_TransferFunction {
                g: 0.0,
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            };
            if !skcms_TransferFunction_invert(&result.trc[i].parametric.parametric, &mut inv) {
                return false;
            }
            let mut err = 0.0;
            for j in 0..3 {
                err = fmaxf_(err, skcms_MaxRoundtripError(&profile.trc[j], &inv));
            }
            if min_max_error > err {
                min_max_error = err;
                best_tf = i;
            }
        }
        for i in 0..3 {
            result.trc[i].parametric.parametric = result.trc[best_tf].parametric.parametric;
        }
        *profile = result;
        assert_usable_as_destination(profile);
        true
    }
}
