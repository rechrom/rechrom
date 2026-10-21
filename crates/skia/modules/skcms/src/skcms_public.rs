// Copyright 2018 Google Inc. BSD-style license.
// repr(C) retains field order for these translated data values; no ABI identity claim.
// Raw ICC buffer pointers retain the upstream borrowed-storage lifetime requirements.
// Source: modules/skcms/src/skcms_public.h:51-53
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct skcms_Matrix3x3 {
    pub vals: [[f32; 3]; 3],
}
// Source: modules/skcms/src/skcms_public.h:56-56
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_Matrix3x3_invert;

// Source: modules/skcms/src/skcms_public.h:57-57
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_Matrix3x3_concat;

// Source: modules/skcms/src/skcms_public.h:60-62
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct skcms_Matrix3x4 {
    pub vals: [[f32; 4]; 3],
}
// Source: modules/skcms/src/skcms_public.h:71-73
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct skcms_TransferFunction {
    pub g: f32,
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}
// Source: modules/skcms/src/skcms_public.h:75-75
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_eval;

// Source: modules/skcms/src/skcms_public.h:76-77
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_invert;

// Source: modules/skcms/src/skcms_public.h:79-87
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum skcms_TFType {
    skcms_TFType_Invalid,
    skcms_TFType_sRGBish,
    skcms_TFType_PQish,
    skcms_TFType_HLGish,
    skcms_TFType_HLGinvish,
    skcms_TFType_PQ,
    skcms_TFType_HLG,
}
// Source: modules/skcms/src/skcms_public.h:90-90
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_getType;

// Source: modules/skcms/src/skcms_public.h:99-101
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_makePQish;

// Source: modules/skcms/src/skcms_public.h:105-107
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_makeScaledHLGish;

// Source: modules/skcms/src/skcms_public.h:110-114
pub fn skcms_TransferFunction_makeHLGish(
    tf: &mut skcms_TransferFunction,
    r: f32,
    g: f32,
    a: f32,
    b: f32,
    c: f32,
) -> bool {
    crate::modules::skcms::skcms::skcms_TransferFunction_makeScaledHLGish(tf, 1.0, r, g, a, b, c)
}
// Source: modules/skcms/src/skcms_public.h:127-129
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_makePQ;

// Source: modules/skcms/src/skcms_public.h:146-150
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_makeHLG;

// Source: modules/skcms/src/skcms_public.h:153-153
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_isSRGBish;

// Source: modules/skcms/src/skcms_public.h:154-154
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_isPQish;

// Source: modules/skcms/src/skcms_public.h:155-155
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_isHLGish;

// Source: modules/skcms/src/skcms_public.h:156-156
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_isPQ;

// Source: modules/skcms/src/skcms_public.h:157-157
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TransferFunction_isHLG;

// Source: modules/skcms/src/skcms_public.h:160-161
// Source: modules/skcms/src/skcms_public.h:165-173
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_CurveParametric {
    pub alias_of_table_entries: u32,
    pub parametric: skcms_TransferFunction,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_CurveTable {
    pub table_entries: u32,
    pub table_8: *const u8,
    pub table_16: *const u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union skcms_Curve {
    pub parametric: skcms_CurveParametric,
    pub table: skcms_CurveTable,
}
// Source: modules/skcms/src/skcms_public.h:179-179
// Source: modules/skcms/src/skcms_public.h:183-187
// Source: modules/skcms/src/skcms_public.h:192-194
// Source: modules/skcms/src/skcms_public.h:197-199
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_A2B {
    pub input_curves: [skcms_Curve; 4],
    pub grid_8: *const u8,
    pub grid_16: *const u8,
    pub input_channels: u32,
    pub grid_points: [u8; 4],
    pub matrix_curves: [skcms_Curve; 3],
    pub matrix: skcms_Matrix3x4,
    pub matrix_channels: u32,
    pub output_channels: u32,
    pub output_curves: [skcms_Curve; 3],
}
// Source: modules/skcms/src/skcms_public.h:201-201
// Source: modules/skcms/src/skcms_public.h:203-204
// Source: modules/skcms/src/skcms_public.h:209-211
// Source: modules/skcms/src/skcms_public.h:216-221
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_B2A {
    pub input_curves: [skcms_Curve; 3],
    pub input_channels: u32,
    pub matrix_channels: u32,
    pub matrix_curves: [skcms_Curve; 3],
    pub matrix: skcms_Matrix3x4,
    pub output_curves: [skcms_Curve; 4],
    pub grid_8: *const u8,
    pub grid_16: *const u8,
    pub grid_points: [u8; 4],
    pub output_channels: u32,
}
// Source: modules/skcms/src/skcms_public.h:223-228
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_CICP {
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub video_full_range_flag: u8,
}
// Source: modules/skcms/src/skcms_public.h:230-231
// Source: modules/skcms/src/skcms_public.h:233-236
// Source: modules/skcms/src/skcms_public.h:242-242
// Source: modules/skcms/src/skcms_public.h:246-246
// Source: modules/skcms/src/skcms_public.h:251-251
// Source: modules/skcms/src/skcms_public.h:256-256
// Source: modules/skcms/src/skcms_public.h:260-260
// Source: modules/skcms/src/skcms_public.h:262-267
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_ICCProfile {
    pub buffer: *const u8,
    pub size: u32,
    pub data_color_space: u32,
    pub pcs: u32,
    pub tag_count: u32,
    pub trc: [skcms_Curve; 3],
    pub toXYZD50: skcms_Matrix3x3,
    pub A2B: skcms_A2B,
    pub B2A: skcms_B2A,
    pub CICP: skcms_CICP,
    pub has_trc: bool,
    pub has_toXYZD50: bool,
    pub has_A2B: bool,
    pub has_B2A: bool,
    pub has_CICP: bool,
}
// Source: modules/skcms/src/skcms_public.h:270-270
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_sRGB_profile;

// Source: modules/skcms/src/skcms_public.h:272-272
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_XYZD50_profile;

// Source: modules/skcms/src/skcms_public.h:274-274
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_sRGB_TransferFunction;

// Source: modules/skcms/src/skcms_public.h:275-275
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_sRGB_Inverse_TransferFunction;

// Source: modules/skcms/src/skcms_public.h:276-276
pub use crate::modules::skcms::skcms::skcms_Identity_TransferFunction;
// Source: modules/skcms/src/skcms_public.h:281-282
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_ApproximatelyEqualProfiles;

// Source: modules/skcms/src/skcms_public.h:287-288
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_AreApproximateInverses;

// Source: modules/skcms/src/skcms_public.h:293-294
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_TRCs_AreApproximateInverse;

// Source: modules/skcms/src/skcms_public.h:301-303
pub use crate::modules::skcms::skcms::skcms_ParseWithA2BPriority;
// Source: modules/skcms/src/skcms_public.h:305-312
/// Safety: original immutable ICC buffer, including the required read padding,
/// remains valid for every use of the resulting borrowed profile and its tables.
pub unsafe fn skcms_Parse(
    buf: *const core::ffi::c_void,
    len: usize,
    profile: *mut skcms_ICCProfile,
) -> bool {
    let priority = [0i32, 1];
    skcms_ParseWithA2BPriority(buf, len, priority.as_ptr(), priority.len() as i32, profile)
}
// Source: modules/skcms/src/skcms_public.h:314-319
pub use crate::modules::skcms::skcms::{skcms_ApproximateCurve, skcms_GetCHAD, skcms_GetWTPT};
// Source: modules/skcms/src/skcms_public.h:325-325
pub use crate::modules::skcms::skcms::skcms_GetInputChannelCount;
// Source: modules/skcms/src/skcms_public.h:328-328
// Source: modules/skcms/src/skcms_public.h:330-332
// Source: modules/skcms/src/skcms_public.h:335-336
// Source: modules/skcms/src/skcms_public.h:339-359
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum skcms_Signature {
    skcms_Signature_CMYK = 0x434D594B,
    skcms_Signature_Gray = 0x47524159,
    skcms_Signature_RGB = 0x52474220,

    skcms_Signature_Lab = 0x4C616220,
    skcms_Signature_XYZ = 0x58595A20,

    skcms_Signature_CIELUV = 0x4C757620,
    skcms_Signature_YCbCr = 0x59436272,
    skcms_Signature_CIEYxy = 0x59787920,
    skcms_Signature_HSV = 0x48535620,
    skcms_Signature_HLS = 0x484C5320,
    skcms_Signature_CMY = 0x434D5920,
    skcms_Signature_2CLR = 0x32434C52,
    skcms_Signature_3CLR = 0x33434C52,
    skcms_Signature_4CLR = 0x34434C52,
    skcms_Signature_5CLR = 0x35434C52,
    skcms_Signature_6CLR = 0x36434C52,
    skcms_Signature_7CLR = 0x37434C52,
    skcms_Signature_8CLR = 0x38434C52,
    skcms_Signature_9CLR = 0x39434C52,
    skcms_Signature_10CLR = 0x41434C52,
    skcms_Signature_11CLR = 0x42434C52,
    skcms_Signature_12CLR = 0x43434C52,
    skcms_Signature_13CLR = 0x44434C52,
    skcms_Signature_14CLR = 0x45434C52,
    skcms_Signature_15CLR = 0x46434C52,
}
// Source: modules/skcms/src/skcms_public.h:361-367
// Source: modules/skcms/src/skcms_public.h:369-370
// Source: modules/skcms/src/skcms_public.h:372-373
// Source: modules/skcms/src/skcms_public.h:375-380
// Source: modules/skcms/src/skcms_public.h:382-383
// Source: modules/skcms/src/skcms_public.h:385-388
// Source: modules/skcms/src/skcms_public.h:390-393
// Source: modules/skcms/src/skcms_public.h:395-398
// Source: modules/skcms/src/skcms_public.h:400-403
// Source: modules/skcms/src/skcms_public.h:405-408
// Source: modules/skcms/src/skcms_public.h:410-414
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum skcms_PixelFormat {
    skcms_PixelFormat_A_8,
    skcms_PixelFormat_A_8_,
    skcms_PixelFormat_G_8,
    skcms_PixelFormat_G_8_,
    skcms_PixelFormat_GA_88,
    skcms_PixelFormat_GA_88_,

    skcms_PixelFormat_RGB_565,
    skcms_PixelFormat_BGR_565,

    skcms_PixelFormat_ABGR_4444,
    skcms_PixelFormat_ARGB_4444,

    skcms_PixelFormat_RGB_888,
    skcms_PixelFormat_BGR_888,
    skcms_PixelFormat_RGBA_8888,
    skcms_PixelFormat_BGRA_8888,
    skcms_PixelFormat_RGBA_8888_sRGB,
    skcms_PixelFormat_BGRA_8888_sRGB,

    skcms_PixelFormat_RGBA_1010102,
    skcms_PixelFormat_BGRA_1010102,

    skcms_PixelFormat_RGB_161616LE,
    skcms_PixelFormat_BGR_161616LE,
    skcms_PixelFormat_RGBA_16161616LE,
    skcms_PixelFormat_BGRA_16161616LE,

    skcms_PixelFormat_RGB_161616BE,
    skcms_PixelFormat_BGR_161616BE,
    skcms_PixelFormat_RGBA_16161616BE,
    skcms_PixelFormat_BGRA_16161616BE,

    skcms_PixelFormat_RGB_hhh_Norm,
    skcms_PixelFormat_BGR_hhh_Norm,
    skcms_PixelFormat_RGBA_hhhh_Norm,
    skcms_PixelFormat_BGRA_hhhh_Norm,

    skcms_PixelFormat_RGB_hhh,
    skcms_PixelFormat_BGR_hhh,
    skcms_PixelFormat_RGBA_hhhh,
    skcms_PixelFormat_BGRA_hhhh,

    skcms_PixelFormat_RGB_fff,
    skcms_PixelFormat_BGR_fff,
    skcms_PixelFormat_RGBA_ffff,
    skcms_PixelFormat_BGRA_ffff,

    skcms_PixelFormat_RGB_101010x_XR,
    skcms_PixelFormat_BGR_101010x_XR,
    skcms_PixelFormat_RGBA_10101010_XR,
    skcms_PixelFormat_BGRA_10101010_XR,
}
// Source: modules/skcms/src/skcms_public.h:426-427
// Source: modules/skcms/src/skcms_public.h:429-429
// Source: modules/skcms/src/skcms_public.h:431-431
// Source: modules/skcms/src/skcms_public.h:433-433
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum skcms_AlphaFormat {
    skcms_AlphaFormat_Opaque,

    skcms_AlphaFormat_Unpremul,

    skcms_AlphaFormat_PremulAsEncoded,
}
// Source: modules/skcms/src/skcms_public.h:437-445
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_Transform;

// Source: modules/skcms/src/skcms_public.h:450-450
pub use crate::modules::skcms::skcms::skcms_MakeUsableAsDestination;
// Source: modules/skcms/src/skcms_public.h:456-456
pub use crate::modules::skcms::skcms::skcms_MakeUsableAsDestinationWithSingleCurve;
// Source: modules/skcms/src/skcms_public.h:459-460
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_AdaptToXYZD50;

// Source: modules/skcms/src/skcms_public.h:464-468
// Public declaration exposes the existing canonical numeric implementation;
// original pointer/buffer leases are those of that function, with no copy engine.
pub use crate::modules::skcms::skcms::skcms_PrimariesToXYZD50;

// Source: modules/skcms/src/skcms_public.h:471-471
pub use crate::modules::skcms::skcms::skcms_DisableRuntimeCPUDetection;

// Source: modules/skcms/src/skcms_public.h:474-478
pub fn skcms_Init(p: &mut skcms_ICCProfile) {
    // Upstream explicitly zeroes this POD object; null raw pointers, zero floats, and false booleans are valid here.
    unsafe { std::ptr::write_bytes(p as *mut skcms_ICCProfile, 0, 1) };
    p.data_color_space = skcms_Signature::skcms_Signature_RGB as u32;
    p.pcs = skcms_Signature::skcms_Signature_XYZ as u32;
}
// Source: modules/skcms/src/skcms_public.h:480-487
pub fn skcms_SetTransferFunction(p: &mut skcms_ICCProfile, tf: &skcms_TransferFunction) {
    p.has_trc = true;
    for i in 0..3 {
        p.trc[i].parametric.alias_of_table_entries = 0;
        p.trc[i].parametric.parametric = *tf;
    }
}
// Source: modules/skcms/src/skcms_public.h:489-492
pub fn skcms_SetXYZD50(p: &mut skcms_ICCProfile, m: &skcms_Matrix3x3) {
    p.has_toXYZD50 = true;
    p.toXYZD50 = *m;
}
