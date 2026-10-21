// Copyright 2018 Google LLC. BSD-style license.
// Source: modules/skcms/src/skcms_Transform.h:20-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    // Source: modules/skcms/src/skcms_Transform.h:21-21
    load_a8,
    // Source: modules/skcms/src/skcms_Transform.h:22-22
    load_g8,
    // Source: modules/skcms/src/skcms_Transform.h:23-23
    load_ga88,
    // Source: modules/skcms/src/skcms_Transform.h:24-24
    load_4444,
    // Source: modules/skcms/src/skcms_Transform.h:25-25
    load_565,
    // Source: modules/skcms/src/skcms_Transform.h:26-26
    load_888,
    // Source: modules/skcms/src/skcms_Transform.h:27-27
    load_8888,
    // Source: modules/skcms/src/skcms_Transform.h:28-28
    load_1010102,
    // Source: modules/skcms/src/skcms_Transform.h:29-29
    load_101010x_XR,
    // Source: modules/skcms/src/skcms_Transform.h:30-30
    load_10101010_XR,
    // Source: modules/skcms/src/skcms_Transform.h:31-31
    load_161616LE,
    // Source: modules/skcms/src/skcms_Transform.h:32-32
    load_16161616LE,
    // Source: modules/skcms/src/skcms_Transform.h:33-33
    load_161616BE,
    // Source: modules/skcms/src/skcms_Transform.h:34-34
    load_16161616BE,
    // Source: modules/skcms/src/skcms_Transform.h:35-35
    load_hhh,
    // Source: modules/skcms/src/skcms_Transform.h:36-36
    load_hhhh,
    // Source: modules/skcms/src/skcms_Transform.h:37-37
    load_fff,
    // Source: modules/skcms/src/skcms_Transform.h:38-38
    load_ffff,
    // Source: modules/skcms/src/skcms_Transform.h:40-40
    swap_rb,
    // Source: modules/skcms/src/skcms_Transform.h:41-41
    clamp,
    // Source: modules/skcms/src/skcms_Transform.h:42-42
    invert,
    // Source: modules/skcms/src/skcms_Transform.h:43-43
    force_opaque,
    // Source: modules/skcms/src/skcms_Transform.h:44-44
    premul,
    // Source: modules/skcms/src/skcms_Transform.h:45-45
    unpremul,
    // Source: modules/skcms/src/skcms_Transform.h:46-46
    matrix_3x3,
    // Source: modules/skcms/src/skcms_Transform.h:47-47
    matrix_3x4,
    // Source: modules/skcms/src/skcms_Transform.h:49-49
    lab_to_xyz,
    // Source: modules/skcms/src/skcms_Transform.h:50-50
    xyz_to_lab,
    // Source: modules/skcms/src/skcms_Transform.h:52-52
    gamma_r,
    // Source: modules/skcms/src/skcms_Transform.h:53-53
    gamma_g,
    // Source: modules/skcms/src/skcms_Transform.h:54-54
    gamma_b,
    // Source: modules/skcms/src/skcms_Transform.h:55-55
    gamma_a,
    // Source: modules/skcms/src/skcms_Transform.h:56-56
    gamma_rgb,
    // Source: modules/skcms/src/skcms_Transform.h:58-58
    tf_r,
    // Source: modules/skcms/src/skcms_Transform.h:59-59
    tf_g,
    // Source: modules/skcms/src/skcms_Transform.h:60-60
    tf_b,
    // Source: modules/skcms/src/skcms_Transform.h:61-61
    tf_a,
    // Source: modules/skcms/src/skcms_Transform.h:62-62
    tf_rgb,
    // Source: modules/skcms/src/skcms_Transform.h:64-64
    pq_r,
    // Source: modules/skcms/src/skcms_Transform.h:65-65
    pq_g,
    // Source: modules/skcms/src/skcms_Transform.h:66-66
    pq_b,
    // Source: modules/skcms/src/skcms_Transform.h:67-67
    pq_a,
    // Source: modules/skcms/src/skcms_Transform.h:68-68
    pq_rgb,
    // Source: modules/skcms/src/skcms_Transform.h:70-70
    hlg_r,
    // Source: modules/skcms/src/skcms_Transform.h:71-71
    hlg_g,
    // Source: modules/skcms/src/skcms_Transform.h:72-72
    hlg_b,
    // Source: modules/skcms/src/skcms_Transform.h:73-73
    hlg_a,
    // Source: modules/skcms/src/skcms_Transform.h:74-74
    hlg_rgb,
    // Source: modules/skcms/src/skcms_Transform.h:75-75
    hlg_ootf_scale,
    // Source: modules/skcms/src/skcms_Transform.h:77-77
    hlginv_r,
    // Source: modules/skcms/src/skcms_Transform.h:78-78
    hlginv_g,
    // Source: modules/skcms/src/skcms_Transform.h:79-79
    hlginv_b,
    // Source: modules/skcms/src/skcms_Transform.h:80-80
    hlginv_a,
    // Source: modules/skcms/src/skcms_Transform.h:81-81
    hlginv_rgb,
    // Source: modules/skcms/src/skcms_Transform.h:82-82
    hlginv_ootf_scale,
    // Source: modules/skcms/src/skcms_Transform.h:84-84
    table_r,
    // Source: modules/skcms/src/skcms_Transform.h:85-85
    table_g,
    // Source: modules/skcms/src/skcms_Transform.h:86-86
    table_b,
    // Source: modules/skcms/src/skcms_Transform.h:87-87
    table_a,
    // Source: modules/skcms/src/skcms_Transform.h:89-89
    clut_A2B,
    // Source: modules/skcms/src/skcms_Transform.h:90-90
    clut_B2A,
    // Source: modules/skcms/src/skcms_Transform.h:93-93
    store_a8,
    // Source: modules/skcms/src/skcms_Transform.h:94-94
    store_g8,
    // Source: modules/skcms/src/skcms_Transform.h:95-95
    store_ga88,
    // Source: modules/skcms/src/skcms_Transform.h:96-96
    store_4444,
    // Source: modules/skcms/src/skcms_Transform.h:97-97
    store_565,
    // Source: modules/skcms/src/skcms_Transform.h:98-98
    store_888,
    // Source: modules/skcms/src/skcms_Transform.h:99-99
    store_8888,
    // Source: modules/skcms/src/skcms_Transform.h:100-100
    store_1010102,
    // Source: modules/skcms/src/skcms_Transform.h:101-101
    store_161616LE,
    // Source: modules/skcms/src/skcms_Transform.h:102-102
    store_16161616LE,
    // Source: modules/skcms/src/skcms_Transform.h:103-103
    store_161616BE,
    // Source: modules/skcms/src/skcms_Transform.h:104-104
    store_16161616BE,
    // Source: modules/skcms/src/skcms_Transform.h:105-105
    store_101010x_XR,
    // Source: modules/skcms/src/skcms_Transform.h:106-106
    store_10101010_XR,
    // Source: modules/skcms/src/skcms_Transform.h:107-107
    store_hhh,
    // Source: modules/skcms/src/skcms_Transform.h:108-108
    store_hhhh,
    // Source: modules/skcms/src/skcms_Transform.h:109-109
    store_fff,
    // Source: modules/skcms/src/skcms_Transform.h:110-110
    store_ffff,
}
// Source: modules/skcms/src/skcms_Transform.h:112-117
// Native macro-expanded enum is represented by the explicit variants above.
// Source: modules/skcms/src/skcms_Transform.h:122-122
pub const INFINITY_: f32 = f32::INFINITY;
// Source: modules/skcms/src/skcms_Transform.h:133-134
// Selected ARM64 baseline instantiates N=4 for these six genuine lane types.
// This is the existing Rust register-lane representation, not a Clang ABI claim.
pub trait BaselineVectorLane {
    type Vector;
}
macro_rules! baseline_vector_lane {($($t:ty),*)=>{$(
    impl BaselineVectorLane for $t { type Vector=[$t;4]; }
)*};}
baseline_vector_lane!(f32, i32, u64, u32, u16, u8);
// Source: modules/skcms/src/skcms_Transform.h:146-152
// Rust re-exports the actual baseline implementation in the native namespace; no duplicate owner.
pub use super::skcms_TransformBaseline as baseline;

// Source: modules/skcms/src/skcms_Transform.h:153-159
// The selected ARM64 native disabled-ISA namespace reaches the actual baseline
// interpreter through its original HSW wrapper, not an enabled-AVX substitute.
#[cfg(target_arch = "aarch64")]
pub use super::skcms_TransformHsw as hsw;
// Source: modules/skcms/src/skcms_Transform.h:160-166
#[cfg(target_arch = "aarch64")]
pub use super::skcms_TransformSkx as skx;
