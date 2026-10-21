// UNVERIFIED: all validation prohibited by user.
use crate::src::{
    core::{
        SkArenaAlloc::SkArenaAlloc, SkOpts, SkRasterPipeline::SkRasterPipeline,
        SkRasterPipelineContextUtils::Pack, SkRasterPipelineOpContexts as C,
        SkRasterPipelineOpList::SkRasterPipelineOp,
    },
    sksl::tracing::{SkSLDebugTracePriv::DebugTracePriv, SkSLTraceHook::TraceHook},
};
use core::{ffi::c_void, ptr::NonNull};
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:27-27
pub type SkRPOffset = u32;
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:37-44
pub type Slot = i32;
pub const NA: Slot = -1;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SlotRange {
    pub index: Slot,
    pub count: i32,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:62-71
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramOp {
    move_src_dst = 0,
    move_dst_src = 1,
    swap_src_dst = 2,
    clamp_01 = 3,
    clamp_a_01 = 4,
    clamp_gamut = 5,
    premul = 6,
    premul_dst = 7,
    force_opaque = 8,
    force_opaque_dst = 9,
    set_rgb = 10,
    swap_rb = 11,
    swap_rb_dst = 12,
    black_color = 13,
    white_color = 14,
    uniform_color = 15,
    uniform_color_dst = 16,
    seed_shader = 17,
    load_a8 = 18,
    load_a8_dst = 19,
    store_a8 = 20,
    gather_a8 = 21,
    load_565 = 22,
    load_565_dst = 23,
    store_565 = 24,
    gather_565 = 25,
    load_4444 = 26,
    load_4444_dst = 27,
    store_4444 = 28,
    gather_4444 = 29,
    load_8888 = 30,
    load_8888_dst = 31,
    store_8888 = 32,
    gather_8888 = 33,
    load_rg88 = 34,
    load_rg88_dst = 35,
    store_rg88 = 36,
    gather_rg88 = 37,
    store_r8 = 38,
    alpha_to_gray = 39,
    alpha_to_gray_dst = 40,
    alpha_to_red = 41,
    alpha_to_red_dst = 42,
    bt709_luminance_or_luma_to_alpha = 43,
    bt709_luminance_or_luma_to_rgb = 44,
    bilerp_clamp_8888 = 45,
    load_src = 46,
    store_src = 47,
    store_src_a = 48,
    load_dst = 49,
    store_dst = 50,
    scale_u8 = 51,
    scale_565 = 52,
    scale_1_float = 53,
    scale_native = 54,
    lerp_u8 = 55,
    lerp_565 = 56,
    lerp_1_float = 57,
    lerp_native = 58,
    dstatop = 59,
    dstin = 60,
    dstout = 61,
    dstover = 62,
    srcatop = 63,
    srcin = 64,
    srcout = 65,
    srcover = 66,
    clear = 67,
    modulate = 68,
    multiply = 69,
    plus_ = 70,
    screen = 71,
    xor_ = 72,
    darken = 73,
    difference = 74,
    exclusion = 75,
    hardlight = 76,
    lighten = 77,
    overlay = 78,
    srcover_rgba_8888 = 79,
    matrix_translate = 80,
    matrix_scale_translate = 81,
    matrix_2x3 = 82,
    matrix_perspective = 83,
    decal_x = 84,
    decal_y = 85,
    decal_x_and_y = 86,
    check_decal_mask = 87,
    clamp_x_1 = 88,
    mirror_x_1 = 89,
    repeat_x_1 = 90,
    clamp_x_and_y = 91,
    evenly_spaced_gradient = 92,
    gradient = 93,
    evenly_spaced_2_stop_gradient = 94,
    xy_to_unit_angle = 95,
    xy_to_radius = 96,
    emboss = 97,
    swizzle = 98,
    debug_x = 99,
    debug_y = 100,
    debug_r = 101,
    debug_g = 102,
    debug_b = 103,
    debug_a = 104,
    debug_r_255 = 105,
    debug_g_255 = 106,
    debug_b_255 = 107,
    debug_a_255 = 108,
    init_lane_masks = 109,
    store_device_xy01 = 110,
    exchange_src = 111,
    load_condition_mask = 112,
    store_condition_mask = 113,
    merge_condition_mask = 114,
    merge_inv_condition_mask = 115,
    load_loop_mask = 116,
    store_loop_mask = 117,
    mask_off_loop_mask = 118,
    reenable_loop_mask = 119,
    merge_loop_mask = 120,
    case_op = 121,
    continue_op = 122,
    load_return_mask = 123,
    store_return_mask = 124,
    mask_off_return_mask = 125,
    branch_if_all_lanes_active = 126,
    branch_if_any_lanes_active = 127,
    branch_if_no_lanes_active = 128,
    branch_if_no_active_lanes_eq = 129,
    jump = 130,
    bitwise_and_imm_4_ints = 131,
    bitwise_and_imm_3_ints = 132,
    bitwise_and_imm_2_ints = 133,
    bitwise_and_imm_int = 134,
    bitwise_and_n_ints = 135,
    bitwise_and_int = 136,
    bitwise_and_2_ints = 137,
    bitwise_and_3_ints = 138,
    bitwise_and_4_ints = 139,
    bitwise_or_n_ints = 140,
    bitwise_or_int = 141,
    bitwise_or_2_ints = 142,
    bitwise_or_3_ints = 143,
    bitwise_or_4_ints = 144,
    bitwise_xor_imm_int = 145,
    bitwise_xor_n_ints = 146,
    bitwise_xor_int = 147,
    bitwise_xor_2_ints = 148,
    bitwise_xor_3_ints = 149,
    bitwise_xor_4_ints = 150,
    cast_to_float_from_int = 151,
    cast_to_float_from_2_ints = 152,
    cast_to_float_from_3_ints = 153,
    cast_to_float_from_4_ints = 154,
    cast_to_float_from_uint = 155,
    cast_to_float_from_2_uints = 156,
    cast_to_float_from_3_uints = 157,
    cast_to_float_from_4_uints = 158,
    cast_to_int_from_float = 159,
    cast_to_int_from_2_floats = 160,
    cast_to_int_from_3_floats = 161,
    cast_to_int_from_4_floats = 162,
    cast_to_uint_from_float = 163,
    cast_to_uint_from_2_floats = 164,
    cast_to_uint_from_3_floats = 165,
    cast_to_uint_from_4_floats = 166,
    abs_int = 167,
    abs_2_ints = 168,
    abs_3_ints = 169,
    abs_4_ints = 170,
    floor_float = 171,
    floor_2_floats = 172,
    floor_3_floats = 173,
    floor_4_floats = 174,
    ceil_float = 175,
    ceil_2_floats = 176,
    ceil_3_floats = 177,
    ceil_4_floats = 178,
    invsqrt_float = 179,
    invsqrt_2_floats = 180,
    invsqrt_3_floats = 181,
    invsqrt_4_floats = 182,
    inverse_mat2 = 183,
    inverse_mat3 = 184,
    inverse_mat4 = 185,
    sin_float = 186,
    cos_float = 187,
    tan_float = 188,
    asin_float = 189,
    acos_float = 190,
    atan_float = 191,
    atan2_n_floats = 192,
    sqrt_float = 193,
    pow_n_floats = 194,
    exp_float = 195,
    exp2_float = 196,
    log_float = 197,
    log2_float = 198,
    refract_4_floats = 199,
    copy_uniform = 200,
    copy_2_uniforms = 201,
    copy_3_uniforms = 202,
    copy_4_uniforms = 203,
    copy_constant = 204,
    splat_2_constants = 205,
    splat_3_constants = 206,
    splat_4_constants = 207,
    copy_slot_masked = 208,
    copy_2_slots_masked = 209,
    copy_3_slots_masked = 210,
    copy_4_slots_masked = 211,
    copy_from_indirect_unmasked = 212,
    copy_from_indirect_uniform_unmasked = 213,
    copy_to_indirect_masked = 214,
    swizzle_copy_to_indirect_masked = 215,
    copy_slot_unmasked = 216,
    copy_2_slots_unmasked = 217,
    copy_3_slots_unmasked = 218,
    copy_4_slots_unmasked = 219,
    copy_immutable_unmasked = 220,
    copy_2_immutables_unmasked = 221,
    copy_3_immutables_unmasked = 222,
    copy_4_immutables_unmasked = 223,
    swizzle_copy_slot_masked = 224,
    swizzle_copy_2_slots_masked = 225,
    swizzle_copy_3_slots_masked = 226,
    swizzle_copy_4_slots_masked = 227,
    swizzle_1 = 228,
    swizzle_2 = 229,
    swizzle_3 = 230,
    swizzle_4 = 231,
    shuffle = 232,
    matrix_multiply_2 = 233,
    matrix_multiply_3 = 234,
    matrix_multiply_4 = 235,
    smoothstep_n_floats = 236,
    dot_2_floats = 237,
    dot_3_floats = 238,
    dot_4_floats = 239,
    add_imm_float = 240,
    add_n_floats = 241,
    add_float = 242,
    add_2_floats = 243,
    add_3_floats = 244,
    add_4_floats = 245,
    add_imm_int = 246,
    add_n_ints = 247,
    add_int = 248,
    add_2_ints = 249,
    add_3_ints = 250,
    add_4_ints = 251,
    sub_n_floats = 252,
    sub_float = 253,
    sub_2_floats = 254,
    sub_3_floats = 255,
    sub_4_floats = 256,
    sub_n_ints = 257,
    sub_int = 258,
    sub_2_ints = 259,
    sub_3_ints = 260,
    sub_4_ints = 261,
    mul_imm_float = 262,
    mul_n_floats = 263,
    mul_float = 264,
    mul_2_floats = 265,
    mul_3_floats = 266,
    mul_4_floats = 267,
    mul_imm_int = 268,
    mul_n_ints = 269,
    mul_int = 270,
    mul_2_ints = 271,
    mul_3_ints = 272,
    mul_4_ints = 273,
    div_n_floats = 274,
    div_float = 275,
    div_2_floats = 276,
    div_3_floats = 277,
    div_4_floats = 278,
    div_n_ints = 279,
    div_int = 280,
    div_2_ints = 281,
    div_3_ints = 282,
    div_4_ints = 283,
    div_n_uints = 284,
    div_uint = 285,
    div_2_uints = 286,
    div_3_uints = 287,
    div_4_uints = 288,
    max_imm_float = 289,
    max_n_floats = 290,
    max_float = 291,
    max_2_floats = 292,
    max_3_floats = 293,
    max_4_floats = 294,
    max_n_ints = 295,
    max_int = 296,
    max_2_ints = 297,
    max_3_ints = 298,
    max_4_ints = 299,
    max_n_uints = 300,
    max_uint = 301,
    max_2_uints = 302,
    max_3_uints = 303,
    max_4_uints = 304,
    min_imm_float = 305,
    min_n_floats = 306,
    min_float = 307,
    min_2_floats = 308,
    min_3_floats = 309,
    min_4_floats = 310,
    min_n_ints = 311,
    min_int = 312,
    min_2_ints = 313,
    min_3_ints = 314,
    min_4_ints = 315,
    min_n_uints = 316,
    min_uint = 317,
    min_2_uints = 318,
    min_3_uints = 319,
    min_4_uints = 320,
    mod_n_floats = 321,
    mod_float = 322,
    mod_2_floats = 323,
    mod_3_floats = 324,
    mod_4_floats = 325,
    mix_n_floats = 326,
    mix_float = 327,
    mix_2_floats = 328,
    mix_3_floats = 329,
    mix_4_floats = 330,
    mix_n_ints = 331,
    mix_int = 332,
    mix_2_ints = 333,
    mix_3_ints = 334,
    mix_4_ints = 335,
    cmplt_imm_float = 336,
    cmplt_n_floats = 337,
    cmplt_float = 338,
    cmplt_2_floats = 339,
    cmplt_3_floats = 340,
    cmplt_4_floats = 341,
    cmplt_imm_int = 342,
    cmplt_n_ints = 343,
    cmplt_int = 344,
    cmplt_2_ints = 345,
    cmplt_3_ints = 346,
    cmplt_4_ints = 347,
    cmplt_imm_uint = 348,
    cmplt_n_uints = 349,
    cmplt_uint = 350,
    cmplt_2_uints = 351,
    cmplt_3_uints = 352,
    cmplt_4_uints = 353,
    cmple_imm_float = 354,
    cmple_n_floats = 355,
    cmple_float = 356,
    cmple_2_floats = 357,
    cmple_3_floats = 358,
    cmple_4_floats = 359,
    cmple_imm_int = 360,
    cmple_n_ints = 361,
    cmple_int = 362,
    cmple_2_ints = 363,
    cmple_3_ints = 364,
    cmple_4_ints = 365,
    cmple_imm_uint = 366,
    cmple_n_uints = 367,
    cmple_uint = 368,
    cmple_2_uints = 369,
    cmple_3_uints = 370,
    cmple_4_uints = 371,
    cmpeq_imm_float = 372,
    cmpeq_n_floats = 373,
    cmpeq_float = 374,
    cmpeq_2_floats = 375,
    cmpeq_3_floats = 376,
    cmpeq_4_floats = 377,
    cmpeq_imm_int = 378,
    cmpeq_n_ints = 379,
    cmpeq_int = 380,
    cmpeq_2_ints = 381,
    cmpeq_3_ints = 382,
    cmpeq_4_ints = 383,
    cmpne_imm_float = 384,
    cmpne_n_floats = 385,
    cmpne_float = 386,
    cmpne_2_floats = 387,
    cmpne_3_floats = 388,
    cmpne_4_floats = 389,
    cmpne_imm_int = 390,
    cmpne_n_ints = 391,
    cmpne_int = 392,
    cmpne_2_ints = 393,
    cmpne_3_ints = 394,
    cmpne_4_ints = 395,
    trace_line = 396,
    trace_var = 397,
    trace_enter = 398,
    trace_exit = 399,
    trace_scope = 400,
    callback = 401,
    stack_checkpoint = 402,
    stack_rewind = 403,
    unbounded_set_rgb = 404,
    unbounded_uniform_color = 405,
    unpremul = 406,
    unpremul_polar = 407,
    dither = 408,
    load_16161616 = 409,
    load_16161616_dst = 410,
    store_16161616 = 411,
    gather_16161616 = 412,
    load_a16 = 413,
    load_a16_dst = 414,
    store_a16 = 415,
    gather_a16 = 416,
    load_r16 = 417,
    load_r16_dst = 418,
    store_r16 = 419,
    gather_r16 = 420,
    load_rg1616 = 421,
    load_rg1616_dst = 422,
    store_rg1616 = 423,
    gather_rg1616 = 424,
    load_f16 = 425,
    load_f16_dst = 426,
    store_f16 = 427,
    gather_f16 = 428,
    load_af16 = 429,
    load_af16_dst = 430,
    store_af16 = 431,
    gather_af16 = 432,
    load_rf16 = 433,
    load_rf16_dst = 434,
    store_rf16 = 435,
    gather_rf16 = 436,
    load_rgf16 = 437,
    load_rgf16_dst = 438,
    store_rgf16 = 439,
    gather_rgf16 = 440,
    load_f32 = 441,
    load_f32_dst = 442,
    store_f32 = 443,
    gather_f32 = 444,
    load_1010102 = 445,
    load_1010102_dst = 446,
    store_1010102 = 447,
    gather_1010102 = 448,
    load_1010102_xr = 449,
    load_1010102_xr_dst = 450,
    store_1010102_xr = 451,
    gather_1010102_xr = 452,
    load_10x6 = 453,
    load_10x6_dst = 454,
    store_10x6 = 455,
    gather_10x6 = 456,
    gather_10101010_xr = 457,
    load_10101010_xr = 458,
    load_10101010_xr_dst = 459,
    store_10101010_xr = 460,
    store_src_rg = 461,
    load_src_rg = 462,
    byte_tables = 463,
    colorburn = 464,
    colordodge = 465,
    softlight = 466,
    hue = 467,
    saturation = 468,
    color = 469,
    luminosity = 470,
    matrix_3x3 = 471,
    matrix_3x4 = 472,
    matrix_4x5 = 473,
    matrix_4x3 = 474,
    parametric = 475,
    gamma_ = 476,
    PQish = 477,
    HLGish = 478,
    HLGinvish = 479,
    ootf = 480,
    rgb_to_hsl = 481,
    hsl_to_rgb = 482,
    css_lab_to_xyz = 483,
    css_oklab_to_linear_srgb = 484,
    css_oklab_gamut_map_to_linear_srgb = 485,
    css_hcl_to_lab = 486,
    css_hsl_to_srgb = 487,
    css_hwb_to_srgb = 488,
    gauss_a_to_rgba = 489,
    mirror_x = 490,
    repeat_x = 491,
    mirror_y = 492,
    repeat_y = 493,
    negate_x = 494,
    bilerp_clamp_8888_force_highp = 495,
    bicubic_clamp_8888 = 496,
    bilinear_setup = 497,
    bilinear_nx = 498,
    bilinear_px = 499,
    bilinear_ny = 500,
    bilinear_py = 501,
    bicubic_setup = 502,
    bicubic_n3x = 503,
    bicubic_n1x = 504,
    bicubic_p1x = 505,
    bicubic_p3x = 506,
    bicubic_n3y = 507,
    bicubic_n1y = 508,
    bicubic_p1y = 509,
    bicubic_p3y = 510,
    accumulate = 511,
    perlin_noise = 512,
    mipmap_linear_init = 513,
    mipmap_linear_update = 514,
    mipmap_linear_finish = 515,
    xy_to_2pt_conical_strip = 516,
    xy_to_2pt_conical_focal_on_circle = 517,
    xy_to_2pt_conical_well_behaved = 518,
    xy_to_2pt_conical_smaller = 519,
    xy_to_2pt_conical_greater = 520,
    alter_2pt_conical_compensate_focal = 521,
    alter_2pt_conical_unswap = 522,
    mask_2pt_conical_nan = 523,
    mask_2pt_conical_degenerates = 524,
    apply_vector_mask = 525,
    set_base_pointer = 526,
    label = 527,
    invoke_shader = 528,
    invoke_color_filter = 529,
    invoke_blender = 530,
    invoke_to_linear_srgb = 531,
    invoke_from_linear_srgb = 532,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:78-124
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuilderOp {
    move_src_dst = 0,
    move_dst_src = 1,
    swap_src_dst = 2,
    clamp_01 = 3,
    clamp_a_01 = 4,
    clamp_gamut = 5,
    premul = 6,
    premul_dst = 7,
    force_opaque = 8,
    force_opaque_dst = 9,
    set_rgb = 10,
    swap_rb = 11,
    swap_rb_dst = 12,
    black_color = 13,
    white_color = 14,
    uniform_color = 15,
    uniform_color_dst = 16,
    seed_shader = 17,
    load_a8 = 18,
    load_a8_dst = 19,
    store_a8 = 20,
    gather_a8 = 21,
    load_565 = 22,
    load_565_dst = 23,
    store_565 = 24,
    gather_565 = 25,
    load_4444 = 26,
    load_4444_dst = 27,
    store_4444 = 28,
    gather_4444 = 29,
    load_8888 = 30,
    load_8888_dst = 31,
    store_8888 = 32,
    gather_8888 = 33,
    load_rg88 = 34,
    load_rg88_dst = 35,
    store_rg88 = 36,
    gather_rg88 = 37,
    store_r8 = 38,
    alpha_to_gray = 39,
    alpha_to_gray_dst = 40,
    alpha_to_red = 41,
    alpha_to_red_dst = 42,
    bt709_luminance_or_luma_to_alpha = 43,
    bt709_luminance_or_luma_to_rgb = 44,
    bilerp_clamp_8888 = 45,
    load_src = 46,
    store_src = 47,
    store_src_a = 48,
    load_dst = 49,
    store_dst = 50,
    scale_u8 = 51,
    scale_565 = 52,
    scale_1_float = 53,
    scale_native = 54,
    lerp_u8 = 55,
    lerp_565 = 56,
    lerp_1_float = 57,
    lerp_native = 58,
    dstatop = 59,
    dstin = 60,
    dstout = 61,
    dstover = 62,
    srcatop = 63,
    srcin = 64,
    srcout = 65,
    srcover = 66,
    clear = 67,
    modulate = 68,
    multiply = 69,
    plus_ = 70,
    screen = 71,
    xor_ = 72,
    darken = 73,
    difference = 74,
    exclusion = 75,
    hardlight = 76,
    lighten = 77,
    overlay = 78,
    srcover_rgba_8888 = 79,
    matrix_translate = 80,
    matrix_scale_translate = 81,
    matrix_2x3 = 82,
    matrix_perspective = 83,
    decal_x = 84,
    decal_y = 85,
    decal_x_and_y = 86,
    check_decal_mask = 87,
    clamp_x_1 = 88,
    mirror_x_1 = 89,
    repeat_x_1 = 90,
    clamp_x_and_y = 91,
    evenly_spaced_gradient = 92,
    gradient = 93,
    evenly_spaced_2_stop_gradient = 94,
    xy_to_unit_angle = 95,
    xy_to_radius = 96,
    emboss = 97,
    swizzle = 98,
    debug_x = 99,
    debug_y = 100,
    debug_r = 101,
    debug_g = 102,
    debug_b = 103,
    debug_a = 104,
    debug_r_255 = 105,
    debug_g_255 = 106,
    debug_b_255 = 107,
    debug_a_255 = 108,
    init_lane_masks = 109,
    store_device_xy01 = 110,
    exchange_src = 111,
    load_condition_mask = 112,
    store_condition_mask = 113,
    merge_condition_mask = 114,
    merge_inv_condition_mask = 115,
    load_loop_mask = 116,
    store_loop_mask = 117,
    mask_off_loop_mask = 118,
    reenable_loop_mask = 119,
    merge_loop_mask = 120,
    case_op = 121,
    continue_op = 122,
    load_return_mask = 123,
    store_return_mask = 124,
    mask_off_return_mask = 125,
    branch_if_all_lanes_active = 126,
    branch_if_any_lanes_active = 127,
    branch_if_no_lanes_active = 128,
    branch_if_no_active_lanes_eq = 129,
    jump = 130,
    bitwise_and_imm_4_ints = 131,
    bitwise_and_imm_3_ints = 132,
    bitwise_and_imm_2_ints = 133,
    bitwise_and_imm_int = 134,
    bitwise_and_n_ints = 135,
    bitwise_and_int = 136,
    bitwise_and_2_ints = 137,
    bitwise_and_3_ints = 138,
    bitwise_and_4_ints = 139,
    bitwise_or_n_ints = 140,
    bitwise_or_int = 141,
    bitwise_or_2_ints = 142,
    bitwise_or_3_ints = 143,
    bitwise_or_4_ints = 144,
    bitwise_xor_imm_int = 145,
    bitwise_xor_n_ints = 146,
    bitwise_xor_int = 147,
    bitwise_xor_2_ints = 148,
    bitwise_xor_3_ints = 149,
    bitwise_xor_4_ints = 150,
    cast_to_float_from_int = 151,
    cast_to_float_from_2_ints = 152,
    cast_to_float_from_3_ints = 153,
    cast_to_float_from_4_ints = 154,
    cast_to_float_from_uint = 155,
    cast_to_float_from_2_uints = 156,
    cast_to_float_from_3_uints = 157,
    cast_to_float_from_4_uints = 158,
    cast_to_int_from_float = 159,
    cast_to_int_from_2_floats = 160,
    cast_to_int_from_3_floats = 161,
    cast_to_int_from_4_floats = 162,
    cast_to_uint_from_float = 163,
    cast_to_uint_from_2_floats = 164,
    cast_to_uint_from_3_floats = 165,
    cast_to_uint_from_4_floats = 166,
    abs_int = 167,
    abs_2_ints = 168,
    abs_3_ints = 169,
    abs_4_ints = 170,
    floor_float = 171,
    floor_2_floats = 172,
    floor_3_floats = 173,
    floor_4_floats = 174,
    ceil_float = 175,
    ceil_2_floats = 176,
    ceil_3_floats = 177,
    ceil_4_floats = 178,
    invsqrt_float = 179,
    invsqrt_2_floats = 180,
    invsqrt_3_floats = 181,
    invsqrt_4_floats = 182,
    inverse_mat2 = 183,
    inverse_mat3 = 184,
    inverse_mat4 = 185,
    sin_float = 186,
    cos_float = 187,
    tan_float = 188,
    asin_float = 189,
    acos_float = 190,
    atan_float = 191,
    atan2_n_floats = 192,
    sqrt_float = 193,
    pow_n_floats = 194,
    exp_float = 195,
    exp2_float = 196,
    log_float = 197,
    log2_float = 198,
    refract_4_floats = 199,
    copy_uniform = 200,
    copy_2_uniforms = 201,
    copy_3_uniforms = 202,
    copy_4_uniforms = 203,
    copy_constant = 204,
    splat_2_constants = 205,
    splat_3_constants = 206,
    splat_4_constants = 207,
    copy_slot_masked = 208,
    copy_2_slots_masked = 209,
    copy_3_slots_masked = 210,
    copy_4_slots_masked = 211,
    copy_from_indirect_unmasked = 212,
    copy_from_indirect_uniform_unmasked = 213,
    copy_to_indirect_masked = 214,
    swizzle_copy_to_indirect_masked = 215,
    copy_slot_unmasked = 216,
    copy_2_slots_unmasked = 217,
    copy_3_slots_unmasked = 218,
    copy_4_slots_unmasked = 219,
    copy_immutable_unmasked = 220,
    copy_2_immutables_unmasked = 221,
    copy_3_immutables_unmasked = 222,
    copy_4_immutables_unmasked = 223,
    swizzle_copy_slot_masked = 224,
    swizzle_copy_2_slots_masked = 225,
    swizzle_copy_3_slots_masked = 226,
    swizzle_copy_4_slots_masked = 227,
    swizzle_1 = 228,
    swizzle_2 = 229,
    swizzle_3 = 230,
    swizzle_4 = 231,
    shuffle = 232,
    matrix_multiply_2 = 233,
    matrix_multiply_3 = 234,
    matrix_multiply_4 = 235,
    smoothstep_n_floats = 236,
    dot_2_floats = 237,
    dot_3_floats = 238,
    dot_4_floats = 239,
    add_imm_float = 240,
    add_n_floats = 241,
    add_float = 242,
    add_2_floats = 243,
    add_3_floats = 244,
    add_4_floats = 245,
    add_imm_int = 246,
    add_n_ints = 247,
    add_int = 248,
    add_2_ints = 249,
    add_3_ints = 250,
    add_4_ints = 251,
    sub_n_floats = 252,
    sub_float = 253,
    sub_2_floats = 254,
    sub_3_floats = 255,
    sub_4_floats = 256,
    sub_n_ints = 257,
    sub_int = 258,
    sub_2_ints = 259,
    sub_3_ints = 260,
    sub_4_ints = 261,
    mul_imm_float = 262,
    mul_n_floats = 263,
    mul_float = 264,
    mul_2_floats = 265,
    mul_3_floats = 266,
    mul_4_floats = 267,
    mul_imm_int = 268,
    mul_n_ints = 269,
    mul_int = 270,
    mul_2_ints = 271,
    mul_3_ints = 272,
    mul_4_ints = 273,
    div_n_floats = 274,
    div_float = 275,
    div_2_floats = 276,
    div_3_floats = 277,
    div_4_floats = 278,
    div_n_ints = 279,
    div_int = 280,
    div_2_ints = 281,
    div_3_ints = 282,
    div_4_ints = 283,
    div_n_uints = 284,
    div_uint = 285,
    div_2_uints = 286,
    div_3_uints = 287,
    div_4_uints = 288,
    max_imm_float = 289,
    max_n_floats = 290,
    max_float = 291,
    max_2_floats = 292,
    max_3_floats = 293,
    max_4_floats = 294,
    max_n_ints = 295,
    max_int = 296,
    max_2_ints = 297,
    max_3_ints = 298,
    max_4_ints = 299,
    max_n_uints = 300,
    max_uint = 301,
    max_2_uints = 302,
    max_3_uints = 303,
    max_4_uints = 304,
    min_imm_float = 305,
    min_n_floats = 306,
    min_float = 307,
    min_2_floats = 308,
    min_3_floats = 309,
    min_4_floats = 310,
    min_n_ints = 311,
    min_int = 312,
    min_2_ints = 313,
    min_3_ints = 314,
    min_4_ints = 315,
    min_n_uints = 316,
    min_uint = 317,
    min_2_uints = 318,
    min_3_uints = 319,
    min_4_uints = 320,
    mod_n_floats = 321,
    mod_float = 322,
    mod_2_floats = 323,
    mod_3_floats = 324,
    mod_4_floats = 325,
    mix_n_floats = 326,
    mix_float = 327,
    mix_2_floats = 328,
    mix_3_floats = 329,
    mix_4_floats = 330,
    mix_n_ints = 331,
    mix_int = 332,
    mix_2_ints = 333,
    mix_3_ints = 334,
    mix_4_ints = 335,
    cmplt_imm_float = 336,
    cmplt_n_floats = 337,
    cmplt_float = 338,
    cmplt_2_floats = 339,
    cmplt_3_floats = 340,
    cmplt_4_floats = 341,
    cmplt_imm_int = 342,
    cmplt_n_ints = 343,
    cmplt_int = 344,
    cmplt_2_ints = 345,
    cmplt_3_ints = 346,
    cmplt_4_ints = 347,
    cmplt_imm_uint = 348,
    cmplt_n_uints = 349,
    cmplt_uint = 350,
    cmplt_2_uints = 351,
    cmplt_3_uints = 352,
    cmplt_4_uints = 353,
    cmple_imm_float = 354,
    cmple_n_floats = 355,
    cmple_float = 356,
    cmple_2_floats = 357,
    cmple_3_floats = 358,
    cmple_4_floats = 359,
    cmple_imm_int = 360,
    cmple_n_ints = 361,
    cmple_int = 362,
    cmple_2_ints = 363,
    cmple_3_ints = 364,
    cmple_4_ints = 365,
    cmple_imm_uint = 366,
    cmple_n_uints = 367,
    cmple_uint = 368,
    cmple_2_uints = 369,
    cmple_3_uints = 370,
    cmple_4_uints = 371,
    cmpeq_imm_float = 372,
    cmpeq_n_floats = 373,
    cmpeq_float = 374,
    cmpeq_2_floats = 375,
    cmpeq_3_floats = 376,
    cmpeq_4_floats = 377,
    cmpeq_imm_int = 378,
    cmpeq_n_ints = 379,
    cmpeq_int = 380,
    cmpeq_2_ints = 381,
    cmpeq_3_ints = 382,
    cmpeq_4_ints = 383,
    cmpne_imm_float = 384,
    cmpne_n_floats = 385,
    cmpne_float = 386,
    cmpne_2_floats = 387,
    cmpne_3_floats = 388,
    cmpne_4_floats = 389,
    cmpne_imm_int = 390,
    cmpne_n_ints = 391,
    cmpne_int = 392,
    cmpne_2_ints = 393,
    cmpne_3_ints = 394,
    cmpne_4_ints = 395,
    trace_line = 396,
    trace_var = 397,
    trace_enter = 398,
    trace_exit = 399,
    trace_scope = 400,
    callback = 401,
    stack_checkpoint = 402,
    stack_rewind = 403,
    unbounded_set_rgb = 404,
    unbounded_uniform_color = 405,
    unpremul = 406,
    unpremul_polar = 407,
    dither = 408,
    load_16161616 = 409,
    load_16161616_dst = 410,
    store_16161616 = 411,
    gather_16161616 = 412,
    load_a16 = 413,
    load_a16_dst = 414,
    store_a16 = 415,
    gather_a16 = 416,
    load_r16 = 417,
    load_r16_dst = 418,
    store_r16 = 419,
    gather_r16 = 420,
    load_rg1616 = 421,
    load_rg1616_dst = 422,
    store_rg1616 = 423,
    gather_rg1616 = 424,
    load_f16 = 425,
    load_f16_dst = 426,
    store_f16 = 427,
    gather_f16 = 428,
    load_af16 = 429,
    load_af16_dst = 430,
    store_af16 = 431,
    gather_af16 = 432,
    load_rf16 = 433,
    load_rf16_dst = 434,
    store_rf16 = 435,
    gather_rf16 = 436,
    load_rgf16 = 437,
    load_rgf16_dst = 438,
    store_rgf16 = 439,
    gather_rgf16 = 440,
    load_f32 = 441,
    load_f32_dst = 442,
    store_f32 = 443,
    gather_f32 = 444,
    load_1010102 = 445,
    load_1010102_dst = 446,
    store_1010102 = 447,
    gather_1010102 = 448,
    load_1010102_xr = 449,
    load_1010102_xr_dst = 450,
    store_1010102_xr = 451,
    gather_1010102_xr = 452,
    load_10x6 = 453,
    load_10x6_dst = 454,
    store_10x6 = 455,
    gather_10x6 = 456,
    gather_10101010_xr = 457,
    load_10101010_xr = 458,
    load_10101010_xr_dst = 459,
    store_10101010_xr = 460,
    store_src_rg = 461,
    load_src_rg = 462,
    byte_tables = 463,
    colorburn = 464,
    colordodge = 465,
    softlight = 466,
    hue = 467,
    saturation = 468,
    color = 469,
    luminosity = 470,
    matrix_3x3 = 471,
    matrix_3x4 = 472,
    matrix_4x5 = 473,
    matrix_4x3 = 474,
    parametric = 475,
    gamma_ = 476,
    PQish = 477,
    HLGish = 478,
    HLGinvish = 479,
    ootf = 480,
    rgb_to_hsl = 481,
    hsl_to_rgb = 482,
    css_lab_to_xyz = 483,
    css_oklab_to_linear_srgb = 484,
    css_oklab_gamut_map_to_linear_srgb = 485,
    css_hcl_to_lab = 486,
    css_hsl_to_srgb = 487,
    css_hwb_to_srgb = 488,
    gauss_a_to_rgba = 489,
    mirror_x = 490,
    repeat_x = 491,
    mirror_y = 492,
    repeat_y = 493,
    negate_x = 494,
    bilerp_clamp_8888_force_highp = 495,
    bicubic_clamp_8888 = 496,
    bilinear_setup = 497,
    bilinear_nx = 498,
    bilinear_px = 499,
    bilinear_ny = 500,
    bilinear_py = 501,
    bicubic_setup = 502,
    bicubic_n3x = 503,
    bicubic_n1x = 504,
    bicubic_p1x = 505,
    bicubic_p3x = 506,
    bicubic_n3y = 507,
    bicubic_n1y = 508,
    bicubic_p1y = 509,
    bicubic_p3y = 510,
    accumulate = 511,
    perlin_noise = 512,
    mipmap_linear_init = 513,
    mipmap_linear_update = 514,
    mipmap_linear_finish = 515,
    xy_to_2pt_conical_strip = 516,
    xy_to_2pt_conical_focal_on_circle = 517,
    xy_to_2pt_conical_well_behaved = 518,
    xy_to_2pt_conical_smaller = 519,
    xy_to_2pt_conical_greater = 520,
    alter_2pt_conical_compensate_focal = 521,
    alter_2pt_conical_unswap = 522,
    mask_2pt_conical_nan = 523,
    mask_2pt_conical_degenerates = 524,
    apply_vector_mask = 525,
    set_base_pointer = 526,
    label = 527,
    invoke_shader = 528,
    invoke_color_filter = 529,
    invoke_blender = 530,
    invoke_to_linear_srgb = 531,
    invoke_from_linear_srgb = 532,
    push_clone = 533,
    push_clone_from_stack = 534,
    push_clone_indirect_from_stack = 535,
    push_constant = 536,
    push_immutable = 537,
    push_immutable_indirect = 538,
    push_slots = 539,
    push_slots_indirect = 540,
    push_uniform = 541,
    push_uniform_indirect = 542,
    copy_stack_to_slots = 543,
    copy_stack_to_slots_unmasked = 544,
    copy_stack_to_slots_indirect = 545,
    copy_uniform_to_slots_unmasked = 546,
    store_immutable_value = 547,
    swizzle_copy_stack_to_slots = 548,
    swizzle_copy_stack_to_slots_indirect = 549,
    discard_stack = 550,
    pad_stack = 551,
    select = 552,
    push_condition_mask = 553,
    pop_condition_mask = 554,
    push_loop_mask = 555,
    pop_loop_mask = 556,
    pop_and_reenable_loop_mask = 557,
    push_return_mask = 558,
    pop_return_mask = 559,
    push_src_rgba = 560,
    push_dst_rgba = 561,
    push_device_xy01 = 562,
    pop_src_rgba = 563,
    pop_dst_rgba = 564,
    trace_var_indirect = 565,
    branch_if_no_active_lanes_on_stack_top_equal = 566,
    unsupported = 567,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:130-139
#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub fOp: BuilderOp,
    pub fSlotA: Slot,
    pub fSlotB: Slot,
    pub fImmA: i32,
    pub fImmB: i32,
    pub fImmC: i32,
    pub fImmD: i32,
    pub fStackID: i32,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:141-151
pub trait Callbacks {
    fn appendShader(&mut self, index: i32) -> Result<bool, crate::RasterError>;
    fn appendColorFilter(&mut self, index: i32) -> Result<bool, crate::RasterError>;
    fn appendBlender(&mut self, index: i32) -> Result<bool, crate::RasterError>;
    fn toLinearSrgb(&mut self, color: *const c_void) -> Result<(), crate::RasterError>;
    fn fromLinearSrgb(&mut self, color: *const c_void) -> Result<(), crate::RasterError>;
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:286-294
pub struct Program {
    fInstructions: Vec<Instruction>,
    fNumValueSlots: i32,
    fNumUniformSlots: i32,
    fNumImmutableSlots: i32,
    fNumTempStackSlots: i32,
    fNumLabels: i32,
    fTempStackMaxDepths: Vec<i32>,
    fDebugTrace: Option<NonNull<DebugTracePriv>>,
    fTraceHook: Option<core::cell::UnsafeCell<Box<dyn TraceHook>>>,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:175-179
struct SlotData {
    values: *mut f32,
    stack: *mut f32,
    immutable: *mut f32,
    stack_len: usize,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:182-185
#[derive(Clone, Copy)]
struct Stage {
    op: ProgramOp,
    ctx: *mut c_void,
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:727-730
#[derive(Default)]
pub struct Builder {
    fInstructions: Vec<Instruction>,
    fNumLabels: i32,
    fExecutionMaskWritesEnabled: i32,
    fCurrentStackID: i32,
}

// Header inline methods retain original header order.
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:309-311
impl Builder {
    pub fn nextLabelID(&mut self) -> i32 {
        let id = self.fNumLabels;
        self.fNumLabels += 1;
        id
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:318-320
impl Builder {
    pub fn enableExecutionMaskWrites(&mut self) {
        self.fExecutionMaskWritesEnabled += 1;
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:322-325
impl Builder {
    pub fn disableExecutionMaskWrites(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),323; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.fExecutionMaskWritesEnabled -= 1;
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:327-329
impl Builder {
    pub fn executionMaskWritesAreEnabled(&self) -> bool {
        self.fExecutionMaskWritesEnabled > 0
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:332-334
impl Builder {
    pub fn init_lane_masks(&mut self) {
        self.appendInstruction(BuilderOp::init_lane_masks, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:336-339
impl Builder {
    pub fn store_src_rg(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),337; @condition b"slots.count == 2\0".as_ptr(); slots.count == 2);
        self.appendInstruction(BuilderOp::store_src_rg, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:341-344
impl Builder {
    pub fn store_src(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),342; @condition b"slots.count == 4\0".as_ptr(); slots.count == 4);
        self.appendInstruction(BuilderOp::store_src, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:346-349
impl Builder {
    pub fn store_dst(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),347; @condition b"slots.count == 4\0".as_ptr(); slots.count == 4);
        self.appendInstruction(BuilderOp::store_dst, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:351-354
impl Builder {
    pub fn store_device_xy01(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),352; @condition b"slots.count == 4\0".as_ptr(); slots.count == 4);
        self.appendInstruction(BuilderOp::store_device_xy01, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:356-359
impl Builder {
    pub fn load_src(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),357; @condition b"slots.count == 4\0".as_ptr(); slots.count == 4);
        self.appendInstruction(BuilderOp::load_src, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:361-364
impl Builder {
    pub fn load_dst(&mut self, slots: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),362; @condition b"slots.count == 4\0".as_ptr(); slots.count == 4);
        self.appendInstruction(BuilderOp::load_dst, &[slots.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:366-368
impl Builder {
    pub fn set_current_stack(&mut self, id: i32) {
        self.fCurrentStackID = id;
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:391-393
impl Builder {
    pub fn push_zeros(&mut self, count: i32) {
        self.push_constant_i(0, count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:395-397
impl Builder {
    pub fn push_constant_f(&mut self, val: f32) {
        self.push_constant_i(val.to_bits() as i32, 1);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:399-401
impl Builder {
    pub fn push_constant_u(&mut self, val: u32, count: i32) {
        self.push_constant_i(val as i32, count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:408-410
impl Builder {
    pub fn store_immutable_value_i(&mut self, slot: Slot, val: i32) {
        self.appendInstruction(BuilderOp::store_immutable_value, &[slot], &[val]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:423-425
impl Builder {
    pub fn push_slots(&mut self, src: SlotRange) {
        self.push_slots_or_immutable(src, BuilderOp::push_slots);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:428-430
impl Builder {
    pub fn push_immutable(&mut self, src: SlotRange) {
        self.push_slots_or_immutable(src, BuilderOp::push_immutable);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:438-441
impl Builder {
    pub fn push_slots_indirect(
        &mut self,
        fixedRange: SlotRange,
        dynamicStack: i32,
        limitRange: SlotRange,
    ) {
        self.push_slots_or_immutable_indirect(
            fixedRange,
            dynamicStack,
            limitRange,
            BuilderOp::push_slots_indirect,
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:443-446
impl Builder {
    pub fn push_immutable_indirect(
        &mut self,
        fixedRange: SlotRange,
        dynamicStack: i32,
        limitRange: SlotRange,
    ) {
        self.push_slots_or_immutable_indirect(
            fixedRange,
            dynamicStack,
            limitRange,
            BuilderOp::push_immutable_indirect,
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:490-493
impl Builder {
    pub fn pop_slots_indirect(
        &mut self,
        fixedRange: SlotRange,
        dynamicStackID: i32,
        limitRange: SlotRange,
    ) {
        self.copy_stack_to_slots_indirect(fixedRange, dynamicStackID, limitRange);
        self.discard_stack(fixedRange.count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:550-552
impl Builder {
    pub fn case_op(&mut self, value: i32) {
        self.appendInstruction(BuilderOp::case_op, &[], &[value]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:555-557
impl Builder {
    pub fn continue_op(&mut self, continueMaskStackID: i32) {
        self.appendInstruction(BuilderOp::continue_op, &[], &[continueMaskStackID]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:559-564
impl Builder {
    pub fn select(&mut self, slots: i32) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),562; @condition b"slots > 0\0".as_ptr(); slots > 0);
        self.appendInstruction(BuilderOp::select, &[], &[slots]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:570-573
impl Builder {
    pub fn copy_slots_masked(&mut self, dst: SlotRange, src: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),571; @condition b"dst.count == src.count\0".as_ptr(); dst.count == src.count);
        self.appendInstruction(
            BuilderOp::copy_slot_masked,
            &[dst.index, src.index],
            &[dst.count],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:603-606
impl Builder {
    pub fn pop_condition_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),604; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::pop_condition_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:610-613
impl Builder {
    pub fn merge_inv_condition_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),611; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::merge_inv_condition_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:615-618
impl Builder {
    pub fn push_loop_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),616; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::push_loop_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:620-623
impl Builder {
    pub fn pop_loop_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),621; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::pop_loop_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:628-630
impl Builder {
    pub fn push_src_rgba(&mut self) {
        self.appendInstruction(BuilderOp::push_src_rgba, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:632-634
impl Builder {
    pub fn push_dst_rgba(&mut self) {
        self.appendInstruction(BuilderOp::push_dst_rgba, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:636-638
impl Builder {
    pub fn push_device_xy01(&mut self) {
        self.appendInstruction(BuilderOp::push_device_xy01, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:642-644
impl Builder {
    pub fn pop_dst_rgba(&mut self) {
        self.appendInstruction(BuilderOp::pop_dst_rgba, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:646-649
impl Builder {
    pub fn mask_off_loop_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),647; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::mask_off_loop_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:651-655
impl Builder {
    pub fn reenable_loop_mask(&mut self, src: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),652; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),653; @condition b"src.count == 1\0".as_ptr(); src.count == 1);
        self.appendInstruction(BuilderOp::reenable_loop_mask, &[src.index], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:657-660
impl Builder {
    pub fn pop_and_reenable_loop_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),658; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::pop_and_reenable_loop_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:662-665
impl Builder {
    pub fn merge_loop_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),663; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::merge_loop_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:667-670
impl Builder {
    pub fn push_return_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),668; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::push_return_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:674-677
impl Builder {
    pub fn mask_off_return_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.h\0".as_ptr(),675; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        self.appendInstruction(BuilderOp::mask_off_return_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:686-688
impl Builder {
    pub fn trace_line(&mut self, traceMaskStackID: i32, line: i32) {
        self.appendInstruction(BuilderOp::trace_line, &[], &[traceMaskStackID, line]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:691-693
impl Builder {
    pub fn trace_var(&mut self, traceMaskStackID: i32, r: SlotRange) {
        self.appendInstruction(
            BuilderOp::trace_var,
            &[r.index],
            &[traceMaskStackID, r.count],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:700-702
impl Builder {
    pub fn trace_enter(&mut self, traceMaskStackID: i32, funcID: i32) {
        self.appendInstruction(BuilderOp::trace_enter, &[], &[traceMaskStackID, funcID]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:705-707
impl Builder {
    pub fn trace_exit(&mut self, traceMaskStackID: i32, funcID: i32) {
        self.appendInstruction(BuilderOp::trace_exit, &[], &[traceMaskStackID, funcID]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:710-712
impl Builder {
    pub fn trace_scope(&mut self, traceMaskStackID: i32, delta: i32) {
        self.appendInstruction(BuilderOp::trace_scope, &[], &[traceMaskStackID, delta]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:727-730
impl Builder {
    pub fn new() -> Self {
        Self::default()
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:46-138
const ALL_SINGLE_SLOT_UNARY_OP_CASES: &[BuilderOp] = &[
    BuilderOp::acos_float,
    BuilderOp::asin_float,
    BuilderOp::atan_float,
    BuilderOp::cos_float,
    BuilderOp::exp_float,
    BuilderOp::exp2_float,
    BuilderOp::log_float,
    BuilderOp::log2_float,
    BuilderOp::sin_float,
    BuilderOp::sqrt_float,
    BuilderOp::tan_float,
];
const ALL_MULTI_SLOT_UNARY_OP_CASES: &[BuilderOp] = &[
    BuilderOp::abs_int,
    BuilderOp::cast_to_float_from_int,
    BuilderOp::cast_to_float_from_uint,
    BuilderOp::cast_to_int_from_float,
    BuilderOp::cast_to_uint_from_float,
    BuilderOp::ceil_float,
    BuilderOp::floor_float,
    BuilderOp::invsqrt_float,
];
const ALL_N_WAY_BINARY_OP_CASES: &[BuilderOp] =
    &[BuilderOp::atan2_n_floats, BuilderOp::pow_n_floats];
const ALL_MULTI_SLOT_BINARY_OP_CASES: &[BuilderOp] = &[
    BuilderOp::add_n_floats,
    BuilderOp::add_n_ints,
    BuilderOp::sub_n_floats,
    BuilderOp::sub_n_ints,
    BuilderOp::mul_n_floats,
    BuilderOp::mul_n_ints,
    BuilderOp::div_n_floats,
    BuilderOp::div_n_ints,
    BuilderOp::div_n_uints,
    BuilderOp::bitwise_and_n_ints,
    BuilderOp::bitwise_or_n_ints,
    BuilderOp::bitwise_xor_n_ints,
    BuilderOp::mod_n_floats,
    BuilderOp::min_n_floats,
    BuilderOp::min_n_ints,
    BuilderOp::min_n_uints,
    BuilderOp::max_n_floats,
    BuilderOp::max_n_ints,
    BuilderOp::max_n_uints,
    BuilderOp::cmple_n_floats,
    BuilderOp::cmple_n_ints,
    BuilderOp::cmple_n_uints,
    BuilderOp::cmplt_n_floats,
    BuilderOp::cmplt_n_ints,
    BuilderOp::cmplt_n_uints,
    BuilderOp::cmpeq_n_floats,
    BuilderOp::cmpeq_n_ints,
    BuilderOp::cmpne_n_floats,
    BuilderOp::cmpne_n_ints,
];
const ALL_IMMEDIATE_BINARY_OP_CASES: &[BuilderOp] = &[
    BuilderOp::add_imm_float,
    BuilderOp::add_imm_int,
    BuilderOp::mul_imm_float,
    BuilderOp::mul_imm_int,
    BuilderOp::bitwise_and_imm_int,
    BuilderOp::bitwise_xor_imm_int,
    BuilderOp::min_imm_float,
    BuilderOp::max_imm_float,
    BuilderOp::cmple_imm_float,
    BuilderOp::cmple_imm_int,
    BuilderOp::cmple_imm_uint,
    BuilderOp::cmplt_imm_float,
    BuilderOp::cmplt_imm_int,
    BuilderOp::cmplt_imm_uint,
    BuilderOp::cmpeq_imm_float,
    BuilderOp::cmpeq_imm_int,
    BuilderOp::cmpne_imm_float,
    BuilderOp::cmpne_imm_int,
];
const ALL_IMMEDIATE_MULTI_SLOT_BINARY_OP_CASES: &[BuilderOp] = &[BuilderOp::bitwise_and_imm_int];
const ALL_N_WAY_TERNARY_OP_CASES: &[BuilderOp] = &[BuilderOp::smoothstep_n_floats];
const ALL_MULTI_SLOT_TERNARY_OP_CASES: &[BuilderOp] =
    &[BuilderOp::mix_n_floats, BuilderOp::mix_n_ints];

// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:170-170
impl Program {
    pub fn numUniforms(&self) -> i32 {
        self.fNumUniformSlots
    }
}
// Uncredited Rust ABI/typed-enum projection helpers.
fn stride() -> usize {
    *SkOpts::raster_pipeline_highp_stride
}
unsafe fn program_op(op: BuilderOp) -> ProgramOp {
    core::mem::transmute(op as u32)
}
unsafe fn add_op(op: ProgramOp, n: i32) -> ProgramOp {
    core::mem::transmute((op as i32 + n) as u32)
}
fn stage(p: &mut Vec<Stage>, op: ProgramOp, ctx: *mut c_void) {
    p.push(Stage { op, ctx });
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:135-140
fn is_immediate_op(op: BuilderOp) -> bool {
    ALL_IMMEDIATE_BINARY_OP_CASES.contains(&op)
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:142-147
fn is_multi_slot_immediate_op(op: BuilderOp) -> bool {
    ALL_IMMEDIATE_MULTI_SLOT_BINARY_OP_CASES.contains(&op)
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:149-189
fn convert_n_way_op_to_immediate(op: BuilderOp, slots: i32, value: &mut i32) -> BuilderOp {
    let imm = unsafe { core::mem::transmute::<u32, BuilderOp>(op as u32 - 1) };
    if is_multi_slot_immediate_op(imm) {
        return imm;
    }
    if slots <= 2 {
        if is_immediate_op(imm) {
            return imm;
        }
        match op {
            BuilderOp::sub_n_ints => {
                if *value == i32::MIN {
                    return op;
                }
                *value = -*value;
                return BuilderOp::add_imm_int;
            }
            BuilderOp::sub_n_floats => {
                *value ^= i32::MIN;
                return BuilderOp::add_imm_float;
            }
            _ => {}
        }
    }
    op
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:191-195
impl Builder {
    fn appendInstruction(&mut self, op: BuilderOp, slots: &[i32], imm: &[i32]) {
        self.fInstructions.push(Instruction {
            fOp: op,
            fSlotA: slots.first().copied().unwrap_or(NA),
            fSlotB: slots.get(1).copied().unwrap_or(NA),
            fImmA: imm.first().copied().unwrap_or(0),
            fImmB: imm.get(1).copied().unwrap_or(0),
            fImmC: imm.get(2).copied().unwrap_or(0),
            fImmD: imm.get(3).copied().unwrap_or(0),
            fStackID: self.fCurrentStackID,
        });
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:197-206
impl Builder {
    fn lastInstruction(&mut self, back: usize) -> Option<&mut Instruction> {
        let n = self.fInstructions.len();
        if n <= back {
            return None;
        }
        let i = &mut self.fInstructions[n - 1 - back];
        if i.fStackID != self.fCurrentStackID {
            return None;
        }
        Some(i)
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:208-213
impl Builder {
    fn lastInstructionOnAnyStack(&mut self, back: usize) -> Option<&mut Instruction> {
        let n = self.fInstructions.len();
        if n <= back {
            None
        } else {
            Some(&mut self.fInstructions[n - 1 - back])
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:215-226
impl Builder {
    pub fn unary_op(&mut self, op: BuilderOp, slots: i32) {
        if ALL_SINGLE_SLOT_UNARY_OP_CASES.contains(&op)
            || ALL_MULTI_SLOT_UNARY_OP_CASES.contains(&op)
        {
            self.appendInstruction(op, &[], &[slots]);
        } else {
            crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),223; b"not a unary op\0".as_ptr());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:228-255
impl Builder {
    pub fn binary_op(&mut self, op: BuilderOp, slots: i32) {
        if let Some(last) = self.lastInstruction(0).copied() {
            if last.fOp == BuilderOp::push_constant && last.fImmA >= slots {
                let mut value = last.fImmB;
                let imm = convert_n_way_op_to_immediate(op, slots, &mut value);
                if imm != op {
                    self.discard_stack(slots);
                    self.appendInstruction(imm, &[], &[slots, value]);
                    return;
                }
            }
        }
        if ALL_N_WAY_BINARY_OP_CASES.contains(&op) || ALL_MULTI_SLOT_BINARY_OP_CASES.contains(&op) {
            self.appendInstruction(op, &[], &[slots]);
        } else {
            crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),252; b"not a binary op\0".as_ptr());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:257-268
impl Builder {
    pub fn ternary_op(&mut self, op: BuilderOp, slots: i32) {
        if ALL_N_WAY_TERNARY_OP_CASES.contains(&op) || ALL_MULTI_SLOT_TERNARY_OP_CASES.contains(&op)
        {
            self.appendInstruction(op, &[], &[slots]);
        } else {
            crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),265; b"not a ternary op\0".as_ptr());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:270-281
impl Builder {
    pub fn dot_floats(&mut self, slots: i32) {
        let op = match slots {
            1 => BuilderOp::mul_n_floats,
            2 => BuilderOp::dot_2_floats,
            3 => BuilderOp::dot_3_floats,
            4 => BuilderOp::dot_4_floats,
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),278; b"invalid number of slots\0".as_ptr());
                return;
            }
        };
        self.appendInstruction(op, &[], &[slots]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:283-285
impl Builder {
    pub fn refract_floats(&mut self) {
        self.appendInstruction(BuilderOp::refract_4_floats, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:287-294
impl Builder {
    pub fn inverse_matrix(&mut self, n: i32) {
        let (op, count) = match n {
            2 => (BuilderOp::inverse_mat2, 4),
            3 => (BuilderOp::inverse_mat3, 9),
            4 => (BuilderOp::inverse_mat4, 16),
            _ => {
                crate::SkUNREACHABLE!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),292;)
            }
        };
        self.appendInstruction(op, &[], &[count]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:296-300
impl Builder {
    pub fn pad_stack(&mut self, count: i32) {
        if count > 0 {
            self.appendInstruction(BuilderOp::pad_stack, &[], &[count]);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:302-344
impl Builder {
    fn simplifyImmediateUnmaskedOp(&mut self) -> bool {
        if self.fInstructions.len() < 3 {
            return false;
        }
        let (Some(pop), Some(imm), Some(push)) = (
            self.lastInstruction(0).copied(),
            self.lastInstruction(1).copied(),
            self.lastInstruction(2).copied(),
        ) else {
            return false;
        };
        if pop.fOp == BuilderOp::copy_stack_to_slots_unmasked
            && is_immediate_op(imm.fOp)
            && imm.fImmA == pop.fImmA
            && (imm.fImmA == 1 || is_multi_slot_immediate_op(imm.fOp))
            && matches!(push.fOp, BuilderOp::push_slots | BuilderOp::push_immutable)
            && push.fImmA >= pop.fImmA
        {
            let immslot = pop.fSlotA + pop.fImmA;
            let pushslot = push.fSlotA + push.fImmA;
            if immslot == pushslot {
                let n = self.fInstructions.len();
                self.fInstructions[n - 3].fImmA -= imm.fImmA;
                self.fInstructions[n - 2].fSlotA = immslot - imm.fImmA;
                self.fInstructions.pop();
                return true;
            }
        }
        false
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:346-433
impl Builder {
    pub fn discard_stack(&mut self, count: i32) {
        self.discard_stack_on(count, self.fCurrentStackID);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:346-433
impl Builder {
    pub fn discard_stack_on(&mut self, mut count: i32, stackID: i32) {
        while count > 0 {
            let Some(last) = self.lastInstructionOnAnyStack(0).copied() else {
                break;
            };
            if last.fStackID != stackID {
                break;
            }
            match last.fOp {
                BuilderOp::discard_stack => {
                    self.fInstructions.last_mut().unwrap().fImmA += count;
                    return;
                }
                BuilderOp::push_clone
                | BuilderOp::push_clone_from_stack
                | BuilderOp::push_clone_indirect_from_stack
                | BuilderOp::push_constant
                | BuilderOp::push_immutable
                | BuilderOp::push_immutable_indirect
                | BuilderOp::push_slots
                | BuilderOp::push_slots_indirect
                | BuilderOp::push_uniform
                | BuilderOp::push_uniform_indirect
                | BuilderOp::pad_stack => {
                    let cancel = count.min(last.fImmA);
                    count -= cancel;
                    let last = self.fInstructions.last_mut().unwrap();
                    last.fImmA -= cancel;
                    if last.fImmA == 0 {
                        self.fInstructions.pop();
                    }
                    continue;
                }
                BuilderOp::push_condition_mask
                | BuilderOp::push_loop_mask
                | BuilderOp::push_return_mask => {
                    count -= 1;
                    self.fInstructions.pop();
                    continue;
                }
                BuilderOp::copy_stack_to_slots_unmasked => {
                    if count == last.fImmA {
                        if self.simplifyImmediateUnmaskedOp() {
                            return;
                        }
                        let mut dst = SlotRange {
                            index: last.fSlotA,
                            count: last.fImmA,
                        };
                        self.fInstructions.pop();
                        self.simplifyPopSlotsUnmasked(&mut dst);
                        if dst.count == 0 {
                            return;
                        }
                        self.copy_stack_to_slots_unmasked(dst);
                        count = dst.count;
                    }
                }
                _ => {}
            }
            break;
        }
        if count > 0 {
            self.appendInstruction(BuilderOp::discard_stack, &[], &[count]);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:435-459
impl Builder {
    pub fn label(&mut self, id: i32) {
        debug_assert!(id >= 0 && id < self.fNumLabels);
        while let Some(last) = self.lastInstructionOnAnyStack(0).copied() {
            if matches!(
                last.fOp,
                BuilderOp::jump
                    | BuilderOp::branch_if_all_lanes_active
                    | BuilderOp::branch_if_any_lanes_active
                    | BuilderOp::branch_if_no_lanes_active
                    | BuilderOp::branch_if_no_active_lanes_on_stack_top_equal
            ) && last.fImmA == id
            {
                self.fInstructions.pop();
                continue;
            }
            break;
        }
        self.appendInstruction(BuilderOp::label, &[], &[id]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:461-470
impl Builder {
    pub fn jump(&mut self, id: i32) {
        debug_assert!(id >= 0 && id < self.fNumLabels);
        if self
            .lastInstructionOnAnyStack(0)
            .is_some_and(|i| i.fOp == BuilderOp::jump)
        {
            return;
        }
        self.appendInstruction(BuilderOp::jump, &[], &[id]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:472-488
impl Builder {
    pub fn branch_if_any_lanes_active(&mut self, id: i32) {
        self.branch_active(BuilderOp::branch_if_any_lanes_active, id, true);
    }
}
// Shared Rust control-flow helper for the three native branch methods; no duplicate credit.
impl Builder {
    fn branch_active(&mut self, op: BuilderOp, id: i32, unmaskedJump: bool) {
        if !self.executionMaskWritesAreEnabled() {
            if unmaskedJump {
                self.jump(id);
            }
            return;
        }
        debug_assert!(id >= 0 && id < self.fNumLabels);
        if self
            .lastInstructionOnAnyStack(0)
            .is_some_and(|i| i.fOp == op || i.fOp == BuilderOp::jump)
        {
            return;
        }
        self.appendInstruction(op, &[], &[id]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:490-506
impl Builder {
    pub fn branch_if_all_lanes_active(&mut self, id: i32) {
        self.branch_active(BuilderOp::branch_if_all_lanes_active, id, true);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:508-523
impl Builder {
    pub fn branch_if_no_lanes_active(&mut self, id: i32) {
        self.branch_active(BuilderOp::branch_if_no_lanes_active, id, false);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:525-538
impl Builder {
    pub fn branch_if_no_active_lanes_on_stack_top_equal(&mut self, value: i32, id: i32) {
        debug_assert!(id >= 0 && id < self.fNumLabels);
        if self.lastInstructionOnAnyStack(0).is_some_and(|i| {
            i.fOp == BuilderOp::jump
                || (i.fOp == BuilderOp::branch_if_no_active_lanes_on_stack_top_equal
                    && i.fImmB == value)
        }) {
            return;
        }
        self.appendInstruction(
            BuilderOp::branch_if_no_active_lanes_on_stack_top_equal,
            &[],
            &[id, value],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:540-582
impl Builder {
    pub fn push_slots_or_immutable(&mut self, mut src: SlotRange, op: BuilderOp) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),541; @condition b"src.count >= 0\0".as_ptr(); src.count>=0);
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == op && last.fSlotA + last.fImmA == src.index {
                last.fImmA += src.count;
                src.count = 0;
            }
        }
        if src.count > 0 {
            self.appendInstruction(op, &[src.index], &[src.count]);
        }
        if self.fInstructions.len() >= 3 {
            if let (Some(push), Some(discard), Some(copy)) = (
                self.lastInstruction(0).copied(),
                self.lastInstruction(1).copied(),
                self.lastInstruction(2).copied(),
            ) {
                if push.fOp == BuilderOp::push_slots
                    && discard.fOp == BuilderOp::discard_stack
                    && discard.fImmA == push.fImmA
                    && matches!(
                        copy.fOp,
                        BuilderOp::copy_stack_to_slots | BuilderOp::copy_stack_to_slots_unmasked
                    )
                    && copy.fSlotA == push.fSlotA
                    && copy.fImmA == push.fImmA
                {
                    self.fInstructions.pop();
                    self.fInstructions.pop();
                }
            }
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:584-596
impl Builder {
    pub fn push_slots_or_immutable_indirect(
        &mut self,
        fixed: SlotRange,
        dynamic: i32,
        limit: SlotRange,
        op: BuilderOp,
    ) {
        self.appendInstruction(
            op,
            &[fixed.index, limit.index + limit.count],
            &[fixed.count, dynamic],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:598-613
impl Builder {
    pub fn push_uniform(&mut self, src: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),599; @condition b"src.count >= 0\0".as_ptr(); src.count>=0);
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::push_uniform && last.fSlotA + last.fImmA == src.index {
                last.fImmA += src.count;
                return;
            }
        }
        if src.count > 0 {
            self.appendInstruction(BuilderOp::push_uniform, &[src.index], &[src.count]);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:615-626
impl Builder {
    pub fn push_uniform_indirect(&mut self, fixed: SlotRange, dynamic: i32, limit: SlotRange) {
        self.appendInstruction(
            BuilderOp::push_uniform_indirect,
            &[fixed.index, limit.index + limit.count],
            &[fixed.count, dynamic],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:628-642
impl Builder {
    pub fn trace_var_indirect(
        &mut self,
        mask: i32,
        fixed: SlotRange,
        dynamic: i32,
        limit: SlotRange,
    ) {
        self.appendInstruction(
            BuilderOp::trace_var_indirect,
            &[fixed.index, limit.index + limit.count],
            &[mask, fixed.count, dynamic],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:644-656
impl Builder {
    pub fn push_constant_i(&mut self, val: i32, count: i32) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),645; @condition b"count >= 0\0".as_ptr(); count>=0);
        if count > 0 {
            if let Some(last) = self.lastInstruction(0) {
                if last.fOp == BuilderOp::push_constant && last.fImmB == val {
                    last.fImmA += count;
                    return;
                }
            }
            self.appendInstruction(BuilderOp::push_constant, &[], &[count, val]);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:658-683
impl Builder {
    pub fn push_duplicates(&mut self, mut count: i32) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::push_constant {
                last.fImmA += count;
                return;
            }
        }
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),666; @condition b"count >= 0\0".as_ptr(); count>=0);
        if count >= 3 {
            self.swizzle(1, &[0, 0, 0, 0]);
            count -= 3;
        }
        while count >= 4 {
            self.push_clone(4, 0);
            count -= 4;
        }
        match count {
            3 => self.swizzle(1, &[0, 0, 0, 0]),
            2 => self.swizzle(1, &[0, 0, 0]),
            1 => self.push_clone(1, 0),
            _ => {}
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:685-698
impl Builder {
    pub fn push_clone(&mut self, num: i32, offset: i32) {
        if num == 1 && offset == 0 {
            if let Some(last) = self.lastInstruction(0) {
                if last.fOp == BuilderOp::push_constant {
                    last.fImmA += 1;
                    return;
                }
            }
        }
        self.appendInstruction(BuilderOp::push_clone, &[], &[num, num + offset]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:700-721
impl Builder {
    pub fn push_clone_from_stack(&mut self, range: SlotRange, other: i32, mut offset: i32) {
        offset -= range.index;
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::push_clone_from_stack
                && last.fImmB == other
                && last.fImmC - last.fImmA == offset
            {
                last.fImmA += range.count;
                return;
            }
        }
        self.appendInstruction(
            BuilderOp::push_clone_from_stack,
            &[],
            &[range.count, other, offset],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:723-735
impl Builder {
    pub fn push_clone_indirect_from_stack(
        &mut self,
        fixed: SlotRange,
        dynamic: i32,
        other: i32,
        offset: i32,
    ) {
        self.appendInstruction(
            BuilderOp::push_clone_indirect_from_stack,
            &[],
            &[fixed.count, other, offset - fixed.index, dynamic],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:737-745
impl Builder {
    pub fn pop_slots(&mut self, dst: SlotRange) {
        if !self.executionMaskWritesAreEnabled() {
            self.pop_slots_unmasked(dst);
            return;
        }
        self.copy_stack_to_slots(dst);
        self.discard_stack(dst.count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:747-832
impl Builder {
    fn simplifyPopSlotsUnmasked(&mut self, dst: &mut SlotRange) {
        if dst.count == 0 {
            return;
        }
        let Some(last) = self.lastInstruction(0).copied() else {
            return;
        };
        let op = last.fOp;
        if !matches!(
            op,
            BuilderOp::push_constant
                | BuilderOp::push_uniform
                | BuilderOp::push_slots
                | BuilderOp::push_immutable
        ) {
            return;
        }
        let src = last.fSlotA + last.fImmA - 1;
        let value = last.fImmB;
        self.fInstructions.last_mut().unwrap().fImmA -= 1;
        if self.fInstructions.last().unwrap().fImmA == 0 {
            self.fInstructions.pop();
        }
        dst.count -= 1;
        let d = SlotRange {
            index: dst.index + dst.count,
            count: 1,
        };
        self.simplifyPopSlotsUnmasked(dst);
        let s = SlotRange {
            index: src,
            count: 1,
        };
        match op {
            BuilderOp::push_constant => self.copy_constant(d.index, value),
            BuilderOp::push_uniform => self.copy_uniform_to_slots_unmasked(d, s),
            BuilderOp::push_slots => {
                if d.index != src {
                    self.copy_slots_unmasked(d, s);
                }
            }
            BuilderOp::push_immutable => self.copy_immutable_unmasked(d, s),
            _ => unreachable!(),
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:834-838
impl Builder {
    pub fn pop_slots_unmasked(&mut self, dst: SlotRange) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),835; @condition b"dst.count >= 0\0".as_ptr(); dst.count>=0);
        self.copy_stack_to_slots_unmasked(dst);
        self.discard_stack(dst.count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:840-851
impl Builder {
    pub fn exchange_src(&mut self) {
        if self
            .lastInstruction(0)
            .is_some_and(|i| i.fOp == BuilderOp::exchange_src)
        {
            self.fInstructions.pop();
            return;
        }
        self.appendInstruction(BuilderOp::exchange_src, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:853-865
impl Builder {
    pub fn pop_src_rgba(&mut self) {
        if self
            .lastInstruction(0)
            .is_some_and(|i| i.fOp == BuilderOp::exchange_src)
        {
            self.fInstructions.pop();
            self.discard_stack(4);
            return;
        }
        self.appendInstruction(BuilderOp::pop_src_rgba, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:867-890
impl Builder {
    pub fn copy_stack_to_slots(&mut self, dst: SlotRange) {
        self.copy_stack_to_slots_offset(dst, dst.count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:867-890
impl Builder {
    pub fn copy_stack_to_slots_offset(&mut self, dst: SlotRange, offset: i32) {
        if !self.executionMaskWritesAreEnabled() {
            self.copy_stack_to_slots_unmasked_offset(dst, offset);
            return;
        }
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_stack_to_slots
                && last.fSlotA + last.fImmA == dst.index
                && last.fImmB - last.fImmA == offset
            {
                last.fImmA += dst.count;
                return;
            }
        }
        self.appendInstruction(
            BuilderOp::copy_stack_to_slots,
            &[dst.index],
            &[dst.count, offset],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:892-903
impl Builder {
    pub fn copy_stack_to_slots_indirect(
        &mut self,
        fixed: SlotRange,
        dynamic: i32,
        limit: SlotRange,
    ) {
        self.appendInstruction(
            BuilderOp::copy_stack_to_slots_indirect,
            &[fixed.index, limit.index + limit.count],
            &[fixed.count, dynamic],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:905-908
fn slot_ranges_overlap(x: SlotRange, y: SlotRange) -> bool {
    x.index < y.index + y.count && y.index < x.index + x.count
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:910-926
impl Builder {
    pub fn copy_constant(&mut self, slot: Slot, value: i32) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_constant
                && last.fImmB == value
                && last.fSlotA + last.fImmA == slot
            {
                last.fImmA += 1;
                return;
            }
        }
        self.appendInstruction(BuilderOp::copy_constant, &[slot], &[1, value]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:928-948
impl Builder {
    pub fn copy_slots_unmasked(&mut self, dst: SlotRange, src: SlotRange) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_slot_unmasked
                && last.fSlotA + last.fImmA == dst.index
                && last.fSlotB + last.fImmA == src.index
                && !slot_ranges_overlap(
                    SlotRange {
                        index: last.fSlotB,
                        count: last.fImmA + dst.count,
                    },
                    SlotRange {
                        index: last.fSlotA,
                        count: last.fImmA + dst.count,
                    },
                )
            {
                last.fImmA += dst.count;
                return;
            }
        }
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),946; @condition b"dst.count == src.count\0".as_ptr(); dst.count==src.count);
        self.appendInstruction(
            BuilderOp::copy_slot_unmasked,
            &[dst.index, src.index],
            &[dst.count],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:950-967
impl Builder {
    pub fn copy_immutable_unmasked(&mut self, dst: SlotRange, src: SlotRange) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_immutable_unmasked
                && last.fSlotA + last.fImmA == dst.index
                && last.fSlotB + last.fImmA == src.index
            {
                last.fImmA += dst.count;
                return;
            }
        }
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),965; @condition b"dst.count == src.count\0".as_ptr(); dst.count==src.count);
        self.appendInstruction(
            BuilderOp::copy_immutable_unmasked,
            &[dst.index, src.index],
            &[dst.count],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:969-987
impl Builder {
    pub fn copy_uniform_to_slots_unmasked(&mut self, dst: SlotRange, src: SlotRange) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_uniform_to_slots_unmasked
                && last.fSlotB + last.fImmA == dst.index
                && last.fSlotA + last.fImmA == src.index
            {
                last.fImmA += dst.count;
                return;
            }
        }
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),984; @condition b"dst.count == src.count\0".as_ptr(); dst.count==src.count);
        self.appendInstruction(
            BuilderOp::copy_uniform_to_slots_unmasked,
            &[src.index, dst.index],
            &[dst.count],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:989-1006
impl Builder {
    pub fn copy_stack_to_slots_unmasked(&mut self, dst: SlotRange) {
        self.copy_stack_to_slots_unmasked_offset(dst, dst.count);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:989-1006
impl Builder {
    pub fn copy_stack_to_slots_unmasked_offset(&mut self, dst: SlotRange, offset: i32) {
        if let Some(last) = self.lastInstruction(0) {
            if last.fOp == BuilderOp::copy_stack_to_slots_unmasked
                && last.fSlotA + last.fImmA == dst.index
                && last.fImmB - last.fImmA == offset
            {
                last.fImmA += dst.count;
                return;
            }
        }
        self.appendInstruction(
            BuilderOp::copy_stack_to_slots_unmasked,
            &[dst.index],
            &[dst.count, offset],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1008-1020
impl Builder {
    pub fn pop_return_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),1009; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        if self
            .lastInstructionOnAnyStack(0)
            .is_some_and(|i| i.fOp == BuilderOp::mask_off_return_mask)
        {
            self.fInstructions.pop();
        }
        self.appendInstruction(BuilderOp::pop_return_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1022-1034
impl Builder {
    pub fn push_condition_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),1023; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        if self
            .lastInstruction(0)
            .is_some_and(|i| i.fOp == BuilderOp::pop_condition_mask)
        {
            self.pad_stack(1);
            return;
        }
        self.appendInstruction(BuilderOp::push_condition_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1036-1050
impl Builder {
    pub fn merge_condition_mask(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),1037; @condition b"this->executionMaskWritesAreEnabled()\0".as_ptr(); self.executionMaskWritesAreEnabled());
        if let Some(i) = self.lastInstructionOnAnyStack(0).copied() {
            if i.fOp == BuilderOp::pop_condition_mask {
                self.fInstructions.pop();
                self.discard_stack_on(1, i.fStackID);
            }
        }
        self.appendInstruction(BuilderOp::merge_condition_mask, &[], &[]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1052-1073
impl Builder {
    pub fn zero_slots_unmasked(&mut self, dst: SlotRange) {
        if let Some(i) = self.lastInstruction(0) {
            if i.fOp == BuilderOp::copy_constant && i.fImmB == 0 {
                if i.fSlotA + i.fImmA == dst.index {
                    i.fImmA += dst.count;
                    return;
                }
                if i.fSlotA == dst.index + dst.count {
                    i.fSlotA = dst.index;
                    i.fImmA += dst.count;
                    return;
                }
            }
        }
        self.appendInstruction(BuilderOp::copy_constant, &[dst.index], &[dst.count, 0]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1075-1084
fn pack_nybbles(components: &[i8]) -> i32 {
    let mut packed = 0u32;
    for c in components.iter().rev() {
        debug_assert!(*c >= 0 && *c <= 15);
        packed = (packed << 4) | (*c as u32);
    }
    packed as i32
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1087-1093
fn offsets_u8(mut bits: u32, offsets: &mut [u8]) {
    for x in offsets {
        *x = ((bits & 15) * stride() as u32 * 4) as u8;
        bits >>= 4;
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1087-1093
fn offsets_u16(mut bits: u32, offsets: &mut [u16]) {
    for x in offsets {
        *x = ((bits & 15) * stride() as u32 * 4) as u16;
        bits >>= 4;
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1095-1102
fn max_packed_nybble(mut components: u32, count: usize) -> i32 {
    let mut largest = 0;
    for _ in 0..count {
        largest = largest.max((components & 15) as i32);
        components >>= 4;
    }
    largest
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1104-1118
impl Builder {
    pub fn swizzle_copy_stack_to_slots(&mut self, dst: SlotRange, components: &[i8], offset: i32) {
        self.appendInstruction(
            BuilderOp::swizzle_copy_stack_to_slots,
            &[dst.index],
            &[components.len() as i32, pack_nybbles(components), offset],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1120-1140
impl Builder {
    pub fn swizzle_copy_stack_to_slots_indirect(
        &mut self,
        fixed: SlotRange,
        dynamic: i32,
        limit: SlotRange,
        components: &[i8],
        offset: i32,
    ) {
        self.appendInstruction(
            BuilderOp::swizzle_copy_stack_to_slots_indirect,
            &[fixed.index, limit.index + limit.count],
            &[
                components.len() as i32,
                pack_nybbles(components),
                offset,
                dynamic,
            ],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1142-1195
impl Builder {
    pub fn swizzle(&mut self, mut consumed: i32, components: &[i8]) {
        debug_assert!(consumed >= 0);
        let mut count = components.len();
        debug_assert!(count <= 16 && components.iter().all(|c| *c >= 0 && *c <= 15));
        let mut elements = [0i8; 16];
        elements[..count].copy_from_slice(components);
        while count > 0 {
            if elements[0] != 0 || elements[1..count].contains(&0) {
                break;
            }
            for i in 1..count {
                elements[i - 1] = elements[i] - 1;
            }
            elements[count - 1] = 0;
            consumed -= 1;
            count -= 1;
        }
        if count == 0 {
            self.discard_stack(consumed);
            return;
        }
        if consumed <= 4 && count <= 4 {
            let op = unsafe {
                core::mem::transmute::<u32, BuilderOp>(
                    BuilderOp::swizzle_1 as u32 + count as u32 - 1,
                )
            };
            self.appendInstruction(op, &[], &[consumed, pack_nybbles(&elements[..count])]);
            return;
        }
        self.appendInstruction(
            BuilderOp::shuffle,
            &[],
            &[
                consumed,
                count as i32,
                pack_nybbles(&elements[..8]),
                pack_nybbles(&elements[8..]),
            ],
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1197-1207
impl Builder {
    pub fn transpose(&mut self, columns: i32, rows: i32) {
        let mut e = [0i8; 16];
        let mut i = 0;
        for r in 0..rows {
            for c in 0..columns {
                e[i] = (c * rows + r) as i8;
                i += 1;
            }
        }
        self.swizzle(columns * rows, &e[..i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1209-1219
impl Builder {
    pub fn diagonal_matrix(&mut self, columns: i32, rows: i32) {
        let mut e = [0i8; 16];
        let mut i = 0;
        for c in 0..columns {
            for r in 0..rows {
                e[i] = (c == r) as i8;
                i += 1;
            }
        }
        self.swizzle(2, &e[..i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1221-1255
impl Builder {
    pub fn matrix_resize(&mut self, oc: i32, or: i32, nc: i32, nr: i32) {
        let mut e = [0i8; 16];
        let mut i = 0;
        let mut consumed = oc * or;
        let (mut zero, mut one) = (0, 0);
        for c in 0..nc {
            for r in 0..nr {
                if c < oc && r < or {
                    e[i] = (c * or + r) as i8;
                } else if c == r {
                    if one == 0 {
                        self.push_constant_f(1.0);
                        one = consumed;
                        consumed += 1;
                    }
                    e[i] = one as i8;
                } else {
                    if zero == 0 {
                        self.push_constant_f(0.0);
                        zero = consumed;
                        consumed += 1;
                    }
                    e[i] = zero as i8;
                }
                i += 1;
            }
        }
        self.swizzle(consumed, &e[..i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1257-1267
impl Builder {
    pub fn matrix_multiply(&mut self, lc: i32, lr: i32, rc: i32, rr: i32) {
        let op = match lc {
            2 => BuilderOp::matrix_multiply_2,
            3 => BuilderOp::matrix_multiply_3,
            4 => BuilderOp::matrix_multiply_4,
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),1263; b"unsupported matrix dimensions\0".as_ptr());
                return;
            }
        };
        self.appendInstruction(op, &[], &[lc, lr, rc, rr]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1269-1278
impl Builder {
    pub unsafe fn finish(
        &mut self,
        values: i32,
        uniforms: i32,
        immutable: i32,
        trace: Option<NonNull<DebugTracePriv>>,
    ) -> Box<Program> {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineBuilder.cpp\0".as_ptr(),1274; @condition b"fExecutionMaskWritesEnabled == 0\0".as_ptr(); self.fExecutionMaskWritesEnabled==0);
        Box::new(Program::new(
            core::mem::take(&mut self.fInstructions),
            values,
            uniforms,
            immutable,
            self.fNumLabels,
            trace,
        ))
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1280-1361
fn stack_usage(inst: &Instruction) -> i32 {
    use BuilderOp::*;
    match inst.fOp {
        push_condition_mask | push_loop_mask | push_return_mask => 1,
        push_src_rgba | push_dst_rgba | push_device_xy01 => 4,
        push_immutable
        | push_immutable_indirect
        | push_constant
        | push_slots
        | push_slots_indirect
        | push_uniform
        | push_uniform_indirect
        | push_clone
        | push_clone_from_stack
        | push_clone_indirect_from_stack
        | pad_stack => inst.fImmA,
        pop_condition_mask | pop_loop_mask | pop_and_reenable_loop_mask | pop_return_mask => -1,
        pop_src_rgba | pop_dst_rgba => -4,
        op if ALL_N_WAY_BINARY_OP_CASES.contains(&op)
            || ALL_MULTI_SLOT_BINARY_OP_CASES.contains(&op)
            || matches!(op, discard_stack | select) =>
        {
            -inst.fImmA
        }
        op if ALL_N_WAY_TERNARY_OP_CASES.contains(&op)
            || ALL_MULTI_SLOT_TERNARY_OP_CASES.contains(&op) =>
        {
            -2 * inst.fImmA
        }
        swizzle_1 => 1 - inst.fImmA,
        swizzle_2 => 2 - inst.fImmA,
        swizzle_3 => 3 - inst.fImmA,
        swizzle_4 => 4 - inst.fImmA,
        dot_2_floats => -3,
        dot_3_floats => -5,
        dot_4_floats => -7,
        refract_4_floats => -5,
        matrix_multiply_2 | matrix_multiply_3 | matrix_multiply_4 => {
            -(inst.fImmA * inst.fImmB + inst.fImmC * inst.fImmD)
        }
        shuffle => inst.fImmB - inst.fImmA,
        _ => 0,
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1363-1390
impl Program {
    fn tempStackMaxDepths(&self) -> Vec<i32> {
        let mut num = 1;
        for i in &self.fInstructions {
            num = num.max(i.fStackID + 1);
        }
        let mut largest = vec![0; num as usize];
        let mut current = vec![0; num as usize];
        for i in &self.fInstructions {
            let id = i.fStackID as usize;
            current[id] += stack_usage(i);
            largest[id] = current[id].max(largest[id]);
            assert!(current[id] >= 0, "unbalanced temp stack push/pop on stack");
        }
        debug_assert!(
            current.iter().all(|c| *c == 0),
            "unbalanced temp stack push/pop on stack"
        );
        largest
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1392-1414
/// Safety: external trace owner/Vec are stable, live, thread-movable; deferred traced append/run holds an exclusive trace lease.
impl Program {
    pub unsafe fn new(
        instrs: Vec<Instruction>,
        values: i32,
        uniforms: i32,
        immutable: i32,
        labels: i32,
        trace: Option<NonNull<DebugTracePriv>>,
    ) -> Self {
        let mut p = Self {
            fInstructions: instrs,
            fNumValueSlots: values,
            fNumUniformSlots: uniforms,
            fNumImmutableSlots: immutable,
            fNumLabels: labels,
            fNumTempStackSlots: 0,
            fTempStackMaxDepths: Vec::new(),
            fDebugTrace: trace,
            fTraceHook: None,
        };
        p.fTempStackMaxDepths = p.tempStackMaxDepths();
        for d in &p.fTempStackMaxDepths {
            p.fNumTempStackSlots += *d;
        }
        if let Some(mut trace) = trace {
            p.fTraceHook = Some(core::cell::UnsafeCell::new(
                crate::src::sksl::tracing::SkSLTraceHook::Tracer::Make(NonNull::from(
                    &mut trace.as_mut().fTraceInfo,
                )) as Box<dyn TraceHook>,
            ));
        }
        p
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1416-1416
impl Drop for Program {
    fn drop(&mut self) {
        drop(self.fTraceHook.take());
        drop(core::mem::take(&mut self.fTempStackMaxDepths));
        drop(core::mem::take(&mut self.fInstructions));
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1418-1427
unsafe fn immutable_data_is_splattable(p: *const i32, count: i32) -> bool {
    for i in 1..count {
        if *p != *p.add(i as usize) {
            return false;
        }
    }
    true
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1429-1474
impl Program {
    unsafe fn appendCopy(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        base: *mut u8,
        op: ProgramOp,
        mut dst: u32,
        ds: i32,
        mut src: u32,
        ss: i32,
        mut count: i32,
    ) {
        debug_assert!(count >= 0);
        while count > 4 {
            self.appendCopy(p, a, base, op, dst, ds, src, ss, 4);
            dst = dst.wrapping_add((4 * ds * 4) as u32);
            src = src.wrapping_add((4 * ss * 4) as u32);
            count -= 4;
        }
        debug_assert!(count <= 4);
        if count > 0 {
            if !base.is_null() {
                debug_assert!(ss == 1);
                let ptr = base.add(src as usize).cast::<i32>();
                if immutable_data_is_splattable(ptr, count) {
                    let ctx = C::ConstantCtx { dst, value: *ptr };
                    stage(
                        p,
                        add_op(ProgramOp::copy_constant, count - 1),
                        Pack(&ctx, a),
                    );
                    return;
                }
            }
            stage(
                p,
                add_op(op, count - 1),
                Pack(&C::BinaryOpCtx { dst, src }, a),
            );
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1476-1486
impl Program {
    unsafe fn appendCopySlotsUnmasked(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        dst: u32,
        src: u32,
        n: i32,
    ) {
        self.appendCopy(
            p,
            a,
            core::ptr::null_mut(),
            ProgramOp::copy_slot_unmasked,
            dst,
            stride() as i32,
            src,
            stride() as i32,
            n,
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1488-1499
impl Program {
    unsafe fn appendCopyImmutableUnmasked(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        base: *mut u8,
        dst: u32,
        src: u32,
        n: i32,
    ) {
        self.appendCopy(
            p,
            a,
            base,
            ProgramOp::copy_immutable_unmasked,
            dst,
            stride() as i32,
            src,
            1,
            n,
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1501-1511
impl Program {
    unsafe fn appendCopySlotsMasked(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        dst: u32,
        src: u32,
        n: i32,
    ) {
        self.appendCopy(
            p,
            a,
            core::ptr::null_mut(),
            ProgramOp::copy_slot_masked,
            dst,
            stride() as i32,
            src,
            stride() as i32,
            n,
        );
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1513-1520
impl Program {
    unsafe fn appendSingleSlotUnaryOp(
        &self,
        p: &mut Vec<Stage>,
        op: ProgramOp,
        mut dst: *mut f32,
        n: i32,
    ) {
        debug_assert!(n >= 0);
        for _ in 0..n {
            stage(p, op, dst.cast());
            dst = dst.add(stride());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1522-1533
impl Program {
    unsafe fn appendMultiSlotUnaryOp(
        &self,
        p: &mut Vec<Stage>,
        op: ProgramOp,
        mut dst: *mut f32,
        mut n: i32,
    ) {
        debug_assert!(n >= 0);
        while n > 0 {
            stage(p, add_op(op, n.min(4) - 1), dst.cast());
            dst = dst.add(4 * stride());
            n -= 4;
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1535-1554
impl Program {
    unsafe fn appendImmediateBinaryOp(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        op: ProgramOp,
        dst: u32,
        value: i32,
        mut n: i32,
    ) {
        debug_assert!(is_immediate_op(core::mem::transmute(op as u32)));
        let slots = if is_multi_slot_immediate_op(core::mem::transmute(op as u32)) {
            4
        } else {
            1
        };
        let mut ctx = C::ConstantCtx { dst, value };
        debug_assert!(n >= 0);
        while n > 0 {
            stage(p, add_op(op, -(n.min(slots) - 1)), Pack(&ctx, a));
            ctx.dst += (slots as usize * stride() * 4) as u32;
            n -= slots;
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1556-1569
impl Program {
    unsafe fn appendAdjacentNWayBinaryOp(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        op: ProgramOp,
        dst: u32,
        src: u32,
        n: i32,
    ) {
        debug_assert!(n >= 0 && dst + (stride() * n as usize * 4) as u32 == src);
        if n > 0 {
            stage(p, op, Pack(&C::BinaryOpCtx { dst, src }, a));
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1571-1586
impl Program {
    unsafe fn appendAdjacentMultiSlotBinaryOp(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        op: ProgramOp,
        base: *mut u8,
        dst: u32,
        src: u32,
        n: i32,
    ) {
        debug_assert!(n >= 0 && dst + (stride() * n as usize * 4) as u32 == src);
        if n > 4 {
            self.appendAdjacentNWayBinaryOp(p, a, op, dst, src, n);
        } else if n > 0 {
            stage(p, add_op(op, n), base.add(dst as usize).cast());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1588-1602
impl Program {
    unsafe fn appendAdjacentNWayTernaryOp(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        op: ProgramOp,
        _base: *mut u8,
        dst: u32,
        src0: u32,
        src1: u32,
        n: i32,
    ) {
        debug_assert!(
            n >= 0
                && dst + (stride() * n as usize * 4) as u32 == src0
                && src0 + (stride() * n as usize * 4) as u32 == src1
        );
        if n > 0 {
            stage(
                p,
                op,
                Pack(
                    &C::TernaryOpCtx {
                        dst,
                        delta: src0 - dst,
                    },
                    a,
                ),
            );
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1604-1622
impl Program {
    unsafe fn appendAdjacentMultiSlotTernaryOp(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        op: ProgramOp,
        base: *mut u8,
        dst: u32,
        src0: u32,
        src1: u32,
        n: i32,
    ) {
        debug_assert!(
            n >= 0
                && dst + (stride() * n as usize * 4) as u32 == src0
                && src0 + (stride() * n as usize * 4) as u32 == src1
        );
        if n > 4 {
            self.appendAdjacentNWayTernaryOp(p, a, op, base, dst, src0, src1, n);
        } else if n > 0 {
            stage(p, add_op(op, n), base.add(dst as usize).cast());
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1624-1630
// Rust stage ABI has no musttail guarantee: native !SK_HAS_MUSTTAIL applies.
impl Program {
    fn appendStackRewindForNonTailcallers(&self, p: &mut Vec<Stage>) {
        self.appendStackRewind(p);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1632-1634
impl Program {
    fn appendStackRewind(&self, p: &mut Vec<Stage>) {
        stage(p, ProgramOp::stack_rewind, core::ptr::null_mut());
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1636-1638
impl Builder {
    pub fn invoke_shader(&mut self, i: i32) {
        self.appendInstruction(BuilderOp::invoke_shader, &[], &[i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1640-1642
impl Builder {
    pub fn invoke_color_filter(&mut self, i: i32) {
        self.appendInstruction(BuilderOp::invoke_color_filter, &[], &[i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1644-1646
impl Builder {
    pub fn invoke_blender(&mut self, i: i32) {
        self.appendInstruction(BuilderOp::invoke_blender, &[], &[i]);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1648-1654
impl Builder {
    pub fn invoke_to_linear_srgb(&mut self) {
        self.pad_stack(1);
        self.appendInstruction(BuilderOp::invoke_to_linear_srgb, &[], &[]);
        self.discard_stack(1);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1656-1662
impl Builder {
    pub fn invoke_from_linear_srgb(&mut self) {
        self.pad_stack(1);
        self.appendInstruction(BuilderOp::invoke_from_linear_srgb, &[], &[]);
        self.discard_stack(1);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1668-1688
impl Program {
    unsafe fn allocateSlotData(&self, a: &mut SkArenaAlloc) -> Option<SlotData> {
        let n = stride();
        let count = (self.fNumValueSlots as usize).checked_add(self.fNumTempStackSlots as usize)?;
        let size = (n * 4)
            .checked_mul(count)?
            .checked_add(4usize.checked_mul(self.fNumImmutableSlots as usize)?)?;
        if size > i32::MAX as usize {
            return None;
        }
        let values = a.makeBytesAlignedTo(size, n * 4).cast::<f32>();
        core::ptr::write_bytes(values.cast::<u8>(), 0, size);
        let stack = values.add(n * self.fNumValueSlots as usize);
        let stack_len = n * self.fNumTempStackSlots as usize;
        let immutable = stack.add(stack_len);
        Some(SlotData {
            values,
            stack,
            immutable,
            stack_len,
        })
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1690-1812
/// Safety: pipeline is valid for each synchronous call. Callbacks may access
/// that same native pipeline/arena pointers; no Rust pipeline or arena borrow
/// crosses a callback. Arena reborrows end with synchronous slot/stage creation.
/// Arena/uniform/Program/trace/callback leases outlive execution; tracing is exclusive.
impl Program {
    pub unsafe fn appendStages(
        &self,
        pipeline: *mut SkRasterPipeline,
        alloc: *mut SkArenaAlloc,
        mut callbacks: Option<&mut dyn Callbacks>,
        uniforms: &[f32],
    ) -> Result<bool, crate::RasterError> {
        let Some(slots) = self.allocateSlotData(&mut *alloc) else {
            return Ok(false);
        };
        let mut stages = Vec::new();
        self.makeStages(
            &mut stages,
            &mut *alloc,
            UniformSpan::from_slice(uniforms),
            &slots,
        );
        let mut branches: Vec<*mut C::BranchCtx> = Vec::with_capacity(self.fNumLabels as usize);
        let mut labels = vec![-1; self.fNumLabels as usize];
        let mut targets = Vec::with_capacity(self.fNumLabels as usize);
        (&mut *pipeline).append(SkRasterPipelineOp::set_base_pointer, slots.values.cast());
        for s in stages {
            match s.op {
                ProgramOp::stack_rewind => (&mut *pipeline).appendStackRewind(),
                ProgramOp::invoke_shader
                | ProgramOp::invoke_color_filter
                | ProgramOp::invoke_blender => {
                    let Some(cb) = callbacks.as_deref_mut() else {
                        return Ok(false);
                    };
                    let index = s.ctx as isize as i32;
                    let success = match s.op {
                        ProgramOp::invoke_shader => cb.appendShader(index)?,
                        ProgramOp::invoke_color_filter => cb.appendColorFilter(index)?,
                        _ => cb.appendBlender(index)?,
                    };
                    if !success {
                        return Ok(false);
                    }
                    (&mut *pipeline)
                        .append(SkRasterPipelineOp::set_base_pointer, slots.values.cast());
                }
                ProgramOp::invoke_to_linear_srgb | ProgramOp::invoke_from_linear_srgb => {
                    let Some(cb) = callbacks.as_deref_mut() else {
                        return Ok(false);
                    };
                    if s.op == ProgramOp::invoke_to_linear_srgb {
                        cb.toLinearSrgb(s.ctx)?;
                    } else {
                        cb.fromLinearSrgb(s.ctx)?;
                    }
                }
                ProgramOp::label => {
                    let id = s.ctx as isize as i32;
                    debug_assert!(id >= 0 && id < self.fNumLabels);
                    labels[id as usize] = (&mut *pipeline).getNumStages();
                }
                ProgramOp::jump
                | ProgramOp::branch_if_all_lanes_active
                | ProgramOp::branch_if_any_lanes_active
                | ProgramOp::branch_if_no_lanes_active
                | ProgramOp::branch_if_no_active_lanes_eq => {
                    let b = s.ctx.cast::<C::BranchCtx>();
                    let id = (*b).offset;
                    debug_assert!(id >= 0 && id < self.fNumLabels);
                    (*b).offset = (&mut *pipeline).getNumStages();
                    debug_assert!(branches.len() == targets.len());
                    branches.push(b);
                    targets.push(id);
                    (&mut *pipeline).append(core::mem::transmute(s.op as u32), s.ctx);
                }
                _ => {
                    debug_assert!(
                        (s.op as u32)
                            < crate::src::core::SkRasterPipelineOpList::kNumRasterPipelineHighpOps
                                as u32
                    );
                    (&mut *pipeline).append(core::mem::transmute(s.op as u32), s.ctx);
                }
            }
        }
        debug_assert!(branches.len() == targets.len());
        for (i, b) in branches.iter().enumerate() {
            let from = (**b).offset;
            let to = labels[targets[i] as usize];
            (**b).offset = to - from;
        }
        Ok(true)
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:1814-2481
impl Program {
    unsafe fn makeStages(
        &self,
        p: &mut Vec<Stage>,
        a: &mut SkArenaAlloc,
        uniforms: UniformSpan,
        slots: &SlotData,
    ) {
        debug_assert!(self.fNumUniformSlots as usize == uniforms.len());
        let n = stride() as isize;
        let mut rewind = 0usize;
        let mut pos = 0;
        let mut stacks = Vec::with_capacity(self.fTempStackMaxDepths.len());
        for depth in &self.fTempStackMaxDepths {
            stacks.push(slots.stack.offset(pos * n));
            pos += *depth as isize;
        }
        let mut labels = vec![-1i32; self.fNumLabels as usize];
        let mut invocation = 0i32;
        let base = slots.values.cast::<u8>();
        let offset = |ptr: *const f32| -> u32 { (ptr as usize).wrapping_sub(base as usize) as u32 };
        for inst in &self.fInstructions {
            if inst.fOp == BuilderOp::store_immutable_value {
                *slots.immutable.offset(inst.fSlotA as isize) = f32::from_bits(inst.fImmA as u32);
            }
        }
        p.reserve(self.fInstructions.len());
        for (index, inst) in self.fInstructions.iter().enumerate() {
            let i = *inst;
            let id = i.fStackID as usize;
            let top = stacks[id];
            let sa = || slots.values.offset(n * i.fSlotA as isize);
            let sb = || slots.values.offset(n * i.fSlotB as isize);
            let ia = || slots.immutable.offset(i.fSlotA as isize);
            let ib = || slots.immutable.offset(i.fSlotB as isize);
            let ua = || uniforms.as_ptr().offset(i.fSlotA as isize);
            let at = |count: i32| top.offset(-(count as isize) * n);
            if matches!(
                i.fOp,
                BuilderOp::jump
                    | BuilderOp::branch_if_any_lanes_active
                    | BuilderOp::branch_if_no_lanes_active
                    | BuilderOp::branch_if_all_lanes_active
                    | BuilderOp::branch_if_no_active_lanes_on_stack_top_equal
            ) {
                debug_assert!(i.fImmA >= 0 && i.fImmA < self.fNumLabels);
                let target = labels[i.fImmA as usize];
                if target >= 0 {
                    if invocation > target {
                        self.appendStackRewind(p);
                    } else {
                        self.appendStackRewindForNonTailcallers(p);
                    }
                    rewind = p.len();
                }
            }
            match i.fOp {
                BuilderOp::label => {
                    debug_assert!(i.fImmA >= 0 && i.fImmA < self.fNumLabels);
                    debug_assert!(labels[i.fImmA as usize] == -1);
                    labels[i.fImmA as usize] = index as i32;
                    stage(p, ProgramOp::label, i.fImmA as isize as *mut c_void);
                }
                BuilderOp::jump
                | BuilderOp::branch_if_any_lanes_active
                | BuilderOp::branch_if_no_lanes_active => {
                    let ctx = a.make(C::BranchCtx { offset: i.fImmA });
                    stage(p, program_op(i.fOp), ctx.cast());
                }
                BuilderOp::branch_if_all_lanes_active => {
                    let ctx = a.make(C::BranchIfAllLanesActiveCtx::new(i.fImmA));
                    stage(p, ProgramOp::branch_if_all_lanes_active, ctx.cast());
                }
                BuilderOp::branch_if_no_active_lanes_on_stack_top_equal => {
                    let ctx = a.make(C::BranchIfEqualCtx {
                        branch: C::BranchCtx { offset: i.fImmA },
                        value: i.fImmB,
                        ptr: at(1).cast(),
                    });
                    stage(p, ProgramOp::branch_if_no_active_lanes_eq, ctx.cast());
                }
                BuilderOp::init_lane_masks => {
                    let ctx = a.make(C::InitLaneMasksCtx {
                        tail: core::ptr::null_mut(),
                    });
                    stage(p, ProgramOp::init_lane_masks, ctx.cast());
                }
                BuilderOp::store_src_rg
                | BuilderOp::store_src
                | BuilderOp::store_dst
                | BuilderOp::store_device_xy01
                | BuilderOp::load_src
                | BuilderOp::load_dst => stage(p, program_op(i.fOp), sa().cast()),
                BuilderOp::store_immutable_value => {}
                op if ALL_SINGLE_SLOT_UNARY_OP_CASES.contains(&op) => {
                    self.appendSingleSlotUnaryOp(p, program_op(op), at(i.fImmA), i.fImmA)
                }
                op if ALL_MULTI_SLOT_UNARY_OP_CASES.contains(&op) => {
                    self.appendMultiSlotUnaryOp(p, program_op(op), at(i.fImmA), i.fImmA)
                }
                op if ALL_IMMEDIATE_BINARY_OP_CASES.contains(&op) => {
                    let dst = if i.fSlotA == NA { at(i.fImmA) } else { sa() };
                    self.appendImmediateBinaryOp(
                        p,
                        a,
                        program_op(op),
                        offset(dst),
                        i.fImmB,
                        i.fImmA,
                    );
                }
                op if ALL_N_WAY_BINARY_OP_CASES.contains(&op) => self.appendAdjacentNWayBinaryOp(
                    p,
                    a,
                    program_op(op),
                    offset(at(i.fImmA * 2)),
                    offset(at(i.fImmA)),
                    i.fImmA,
                ),
                op if ALL_MULTI_SLOT_BINARY_OP_CASES.contains(&op) => self
                    .appendAdjacentMultiSlotBinaryOp(
                        p,
                        a,
                        program_op(op),
                        base,
                        offset(at(i.fImmA * 2)),
                        offset(at(i.fImmA)),
                        i.fImmA,
                    ),
                op if ALL_N_WAY_TERNARY_OP_CASES.contains(&op) => self.appendAdjacentNWayTernaryOp(
                    p,
                    a,
                    program_op(op),
                    base,
                    offset(at(i.fImmA * 3)),
                    offset(at(i.fImmA * 2)),
                    offset(at(i.fImmA)),
                    i.fImmA,
                ),
                op if ALL_MULTI_SLOT_TERNARY_OP_CASES.contains(&op) => self
                    .appendAdjacentMultiSlotTernaryOp(
                        p,
                        a,
                        program_op(op),
                        base,
                        offset(at(i.fImmA * 3)),
                        offset(at(i.fImmA * 2)),
                        offset(at(i.fImmA)),
                        i.fImmA,
                    ),
                BuilderOp::select => self.appendCopySlotsMasked(
                    p,
                    a,
                    offset(at(i.fImmA * 2)),
                    offset(at(i.fImmA)),
                    i.fImmA,
                ),
                BuilderOp::copy_slot_masked => {
                    self.appendCopySlotsMasked(p, a, offset(sa()), offset(sb()), i.fImmA)
                }
                BuilderOp::copy_slot_unmasked => {
                    self.appendCopySlotsUnmasked(p, a, offset(sa()), offset(sb()), i.fImmA)
                }
                BuilderOp::copy_immutable_unmasked => self.appendCopyImmutableUnmasked(
                    p,
                    a,
                    base,
                    offset(sa()),
                    offset(ib()),
                    i.fImmA,
                ),
                BuilderOp::refract_4_floats => stage(p, ProgramOp::refract_4_floats, at(9).cast()),
                BuilderOp::inverse_mat2 | BuilderOp::inverse_mat3 | BuilderOp::inverse_mat4 => {
                    stage(p, program_op(i.fOp), at(i.fImmA).cast())
                }
                BuilderOp::dot_2_floats | BuilderOp::dot_3_floats | BuilderOp::dot_4_floats => {
                    stage(p, program_op(i.fOp), at(i.fImmA * 2).cast())
                }
                BuilderOp::swizzle_1 => {
                    debug_assert!(i.fImmB >= 0 && i.fImmB <= 15);
                    let dst = at(i.fImmA);
                    let src = dst.offset(i.fImmB as isize * n);
                    if src != dst {
                        self.appendCopySlotsUnmasked(p, a, offset(dst), offset(src), 1);
                    }
                }
                BuilderOp::swizzle_2 | BuilderOp::swizzle_3 | BuilderOp::swizzle_4 => {
                    let mut ctx = C::SwizzleCtx {
                        dst: offset(at(i.fImmA)),
                        offsets: [0; 4],
                    };
                    offsets_u8(i.fImmB as u32, &mut ctx.offsets);
                    stage(p, program_op(i.fOp), Pack(&ctx, a));
                }
                BuilderOp::shuffle => {
                    let mut ctx = C::ShuffleCtx {
                        ptr: at(i.fImmA).cast(),
                        count: i.fImmB,
                        offsets: [0; 16],
                    };
                    offsets_u16(i.fImmC as u32, &mut ctx.offsets[..8]);
                    offsets_u16(i.fImmD as u32, &mut ctx.offsets[8..]);
                    stage(p, ProgramOp::shuffle, a.make(ctx).cast());
                }
                BuilderOp::matrix_multiply_2
                | BuilderOp::matrix_multiply_3
                | BuilderOp::matrix_multiply_4 => {
                    let count = i.fImmB * i.fImmC + i.fImmA * i.fImmB + i.fImmC * i.fImmD;
                    let ctx = C::MatrixMultiplyCtx {
                        dst: offset(at(count)),
                        leftColumns: i.fImmA as u8,
                        leftRows: i.fImmB as u8,
                        rightColumns: i.fImmC as u8,
                        rightRows: i.fImmD as u8,
                    };
                    stage(p, program_op(i.fOp), Pack(&ctx, a));
                }
                BuilderOp::exchange_src => stage(p, ProgramOp::exchange_src, at(4).cast()),
                BuilderOp::push_src_rgba => stage(p, ProgramOp::store_src, top.cast()),
                BuilderOp::push_dst_rgba => stage(p, ProgramOp::store_dst, top.cast()),
                BuilderOp::push_device_xy01 => stage(p, ProgramOp::store_device_xy01, top.cast()),
                BuilderOp::pop_src_rgba => stage(p, ProgramOp::load_src, at(4).cast()),
                BuilderOp::pop_dst_rgba => stage(p, ProgramOp::load_dst, at(4).cast()),
                BuilderOp::push_slots => {
                    self.appendCopySlotsUnmasked(p, a, offset(top), offset(sa()), i.fImmA)
                }
                BuilderOp::push_immutable => {
                    self.appendCopyImmutableUnmasked(p, a, base, offset(top), offset(ia()), i.fImmA)
                }
                BuilderOp::copy_stack_to_slots_indirect
                | BuilderOp::push_immutable_indirect
                | BuilderOp::push_slots_indirect
                | BuilderOp::push_uniform_indirect => {
                    let (op, src, dst): (ProgramOp, *const i32, *mut i32) = match i.fOp {
                        BuilderOp::push_slots_indirect => (
                            ProgramOp::copy_from_indirect_unmasked,
                            sa().cast(),
                            top.cast(),
                        ),
                        BuilderOp::push_immutable_indirect => (
                            ProgramOp::copy_from_indirect_uniform_unmasked,
                            ia().cast(),
                            top.cast(),
                        ),
                        BuilderOp::push_uniform_indirect => (
                            ProgramOp::copy_from_indirect_uniform_unmasked,
                            ua().cast(),
                            top.cast(),
                        ),
                        _ => (
                            ProgramOp::copy_to_indirect_masked,
                            at(i.fImmA).cast(),
                            sa().cast(),
                        ),
                    };
                    let ctx = C::CopyIndirectCtx {
                        src,
                        dst,
                        indirectOffset: stacks[i.fImmB as usize].offset(-n).cast(),
                        indirectLimit: (i.fSlotB - i.fSlotA - i.fImmA) as u32,
                        slots: i.fImmA as u32,
                    };
                    stage(p, op, a.make(ctx).cast());
                }
                BuilderOp::push_uniform | BuilderOp::copy_uniform_to_slots_unmasked => {
                    let mut src = ua();
                    let mut dst = if i.fOp == BuilderOp::push_uniform {
                        top
                    } else {
                        sb()
                    };
                    let mut count = i.fImmA;
                    while count > 0 {
                        let ctx = C::UniformCtx {
                            src: src.cast(),
                            dst: dst.cast(),
                        };
                        stage(
                            p,
                            add_op(ProgramOp::copy_uniform, count.min(4) - 1),
                            a.make(ctx).cast(),
                        );
                        dst = dst.offset(4 * n);
                        src = src.add(4);
                        count -= 4;
                    }
                }
                BuilderOp::push_condition_mask => {
                    stage(p, ProgramOp::store_condition_mask, top.cast())
                }
                BuilderOp::pop_condition_mask => {
                    stage(p, ProgramOp::load_condition_mask, at(1).cast())
                }
                BuilderOp::merge_condition_mask | BuilderOp::merge_inv_condition_mask => {
                    stage(p, program_op(i.fOp), at(2).cast())
                }
                BuilderOp::push_loop_mask => stage(p, ProgramOp::store_loop_mask, top.cast()),
                BuilderOp::pop_loop_mask => stage(p, ProgramOp::load_loop_mask, at(1).cast()),
                BuilderOp::pop_and_reenable_loop_mask => {
                    stage(p, ProgramOp::reenable_loop_mask, at(1).cast())
                }
                BuilderOp::reenable_loop_mask => {
                    stage(p, ProgramOp::reenable_loop_mask, sa().cast())
                }
                BuilderOp::mask_off_loop_mask => {
                    stage(p, ProgramOp::mask_off_loop_mask, core::ptr::null_mut())
                }
                BuilderOp::merge_loop_mask => stage(p, ProgramOp::merge_loop_mask, at(1).cast()),
                BuilderOp::push_return_mask => stage(p, ProgramOp::store_return_mask, top.cast()),
                BuilderOp::pop_return_mask => stage(p, ProgramOp::load_return_mask, at(1).cast()),
                BuilderOp::mask_off_return_mask => {
                    stage(p, ProgramOp::mask_off_return_mask, core::ptr::null_mut())
                }
                BuilderOp::copy_constant | BuilderOp::push_constant => {
                    let mut dst = if i.fOp == BuilderOp::copy_constant {
                        sa()
                    } else {
                        top
                    };
                    let mut count = i.fImmA;
                    while count > 0 {
                        stage(
                            p,
                            add_op(ProgramOp::copy_constant, count.min(4) - 1),
                            Pack(
                                &C::ConstantCtx {
                                    dst: offset(dst),
                                    value: i.fImmB,
                                },
                                a,
                            ),
                        );
                        dst = dst.offset(4 * n);
                        count -= 4;
                    }
                }
                BuilderOp::copy_stack_to_slots => {
                    self.appendCopySlotsMasked(p, a, offset(sa()), offset(at(i.fImmB)), i.fImmA)
                }
                BuilderOp::copy_stack_to_slots_unmasked => {
                    self.appendCopySlotsUnmasked(p, a, offset(sa()), offset(at(i.fImmB)), i.fImmA)
                }
                BuilderOp::swizzle_copy_stack_to_slots => {
                    let mut ctx = C::SwizzleCopyCtx {
                        src: at(i.fImmC).cast(),
                        dst: sa().cast(),
                        offsets: [0; 4],
                    };
                    offsets_u16(i.fImmB as u32, &mut ctx.offsets);
                    stage(
                        p,
                        add_op(ProgramOp::swizzle_copy_slot_masked, i.fImmA - 1),
                        a.make(ctx).cast(),
                    );
                }
                BuilderOp::push_clone => {
                    self.appendCopySlotsUnmasked(p, a, offset(top), offset(at(i.fImmB)), i.fImmA)
                }
                BuilderOp::push_clone_from_stack => {
                    let src = stacks[i.fImmB as usize].offset(-(i.fImmC as isize) * n);
                    self.appendCopySlotsUnmasked(p, a, offset(top), offset(src), i.fImmA);
                }
                BuilderOp::push_clone_indirect_from_stack => {
                    let ctx = C::CopyIndirectCtx {
                        dst: top.cast(),
                        src: stacks[i.fImmB as usize]
                            .offset(-(i.fImmC as isize) * n)
                            .cast(),
                        indirectOffset: stacks[i.fImmD as usize].offset(-n).cast(),
                        indirectLimit: (i.fImmC - i.fImmA) as u32,
                        slots: i.fImmA as u32,
                    };
                    stage(
                        p,
                        ProgramOp::copy_from_indirect_unmasked,
                        a.make(ctx).cast(),
                    );
                }
                BuilderOp::swizzle_copy_stack_to_slots_indirect => {
                    let mut ctx = C::SwizzleCopyIndirectCtx {
                        base: C::CopyIndirectCtx {
                            src: at(i.fImmC).cast(),
                            dst: sa().cast(),
                            indirectOffset: stacks[i.fImmD as usize].offset(-n).cast(),
                            indirectLimit: (i.fSlotB
                                - i.fSlotA
                                - (max_packed_nybble(i.fImmB as u32, i.fImmA as usize) + 1))
                                as u32,
                            slots: i.fImmA as u32,
                        },
                        offsets: [0; 4],
                    };
                    offsets_u16(i.fImmB as u32, &mut ctx.offsets);
                    stage(
                        p,
                        ProgramOp::swizzle_copy_to_indirect_masked,
                        a.make(ctx).cast(),
                    );
                }
                BuilderOp::case_op => stage(
                    p,
                    ProgramOp::case_op,
                    Pack(
                        &C::CaseOpCtx {
                            expectedValue: i.fImmA,
                            offset: offset(at(2)),
                        },
                        a,
                    ),
                ),
                BuilderOp::continue_op => stage(
                    p,
                    ProgramOp::continue_op,
                    stacks[i.fImmA as usize].offset(-n).cast(),
                ),
                BuilderOp::pad_stack | BuilderOp::discard_stack => {}
                BuilderOp::invoke_shader
                | BuilderOp::invoke_color_filter
                | BuilderOp::invoke_blender => {
                    stage(p, program_op(i.fOp), i.fImmA as isize as *mut c_void);
                    invocation = index as i32;
                }
                BuilderOp::invoke_to_linear_srgb | BuilderOp::invoke_from_linear_srgb => {
                    stage(
                        p,
                        program_op(i.fOp),
                        stacks[i.fImmA as usize].offset(-4 * n).cast(),
                    );
                    invocation = index as i32;
                }
                BuilderOp::trace_line
                | BuilderOp::trace_scope
                | BuilderOp::trace_enter
                | BuilderOp::trace_exit
                | BuilderOp::trace_var
                | BuilderOp::trace_var_indirect => {
                    let hook = self
                        .fTraceHook
                        .as_ref()
                        .expect("native trace instruction requires trace hook");
                    let hook = &mut **hook.get() as *mut dyn TraceHook;
                    let mask = stacks[i.fImmA as usize].offset(-n).cast();
                    match i.fOp {
                        BuilderOp::trace_line => stage(
                            p,
                            ProgramOp::trace_line,
                            a.make(C::TraceLineCtx {
                                traceHook: hook,
                                traceMask: mask,
                                lineNumber: i.fImmB,
                            })
                            .cast(),
                        ),
                        BuilderOp::trace_scope => stage(
                            p,
                            ProgramOp::trace_scope,
                            a.make(C::TraceScopeCtx {
                                traceHook: hook,
                                traceMask: mask,
                                delta: i.fImmB,
                            })
                            .cast(),
                        ),
                        BuilderOp::trace_enter | BuilderOp::trace_exit => stage(
                            p,
                            program_op(i.fOp),
                            a.make(C::TraceFuncCtx {
                                traceHook: hook,
                                traceMask: mask,
                                funcIdx: i.fImmB,
                            })
                            .cast(),
                        ),
                        _ => {
                            let (indirect, limit) = if i.fOp == BuilderOp::trace_var_indirect {
                                (
                                    stacks[i.fImmC as usize].offset(-n).cast::<u32>() as *const u32,
                                    (i.fSlotB - i.fSlotA - i.fImmB) as u32,
                                )
                            } else {
                                (core::ptr::null(), 0)
                            };
                            stage(
                                p,
                                ProgramOp::trace_var,
                                a.make(C::TraceVarCtx {
                                    traceHook: hook,
                                    traceMask: mask,
                                    slotIdx: i.fSlotA,
                                    numSlots: i.fImmB,
                                    data: sa().cast(),
                                    indirectOffset: indirect,
                                    indirectLimit: limit,
                                })
                                .cast(),
                            );
                        }
                    }
                }
                _ => debug_assert!(
                    false,
                    "Raster Pipeline: unsupported instruction {}",
                    i.fOp as u32
                ),
            }
            let usage = stack_usage(&i);
            if usage != 0 {
                stacks[id] = top.offset(usage as isize * n);
                debug_assert!(
                    stacks[id] >= slots.stack && stacks[id] <= slots.stack.add(slots.stack_len)
                );
            }
            let count = p.len();
            if count - rewind > 500 {
                self.appendStackRewindForNonTailcallers(p);
                rewind = count;
            }
        }
    }
}
// Program contains immutable owned instructions and an optional native mutable
// trace pointee. UnsafeCell models const native pointee mutation, not a lock.
// TraceHook is Send+Sync; unsafe construction and append/run require the retained
// stable external Vec lease and exclusive access across all traced executions.
unsafe impl Send for Program {}
unsafe impl Sync for Program {}
// Dumper translation follows in native helper/dispatch order below.

// Rust nonowning uniform-span representation. It permits the native dump-only
// uninitialized allocation without constructing an invalid &[f32]. Stage
// generation offsets these pointers but does not read uniform values.
#[derive(Clone, Copy)]
struct UniformSpan {
    ptr: *const f32,
    count: usize,
}
impl UniformSpan {
    fn from_slice(v: &[f32]) -> Self {
        Self {
            ptr: v.as_ptr(),
            count: v.len(),
        }
    }
    fn len(self) -> usize {
        self.count
    }
    fn as_ptr(self) -> *const f32 {
        self.ptr
    }
}
// Language glue for native byte-string/string_view ownership and special members.
#[derive(Clone, Copy, PartialEq)]
struct DumpNameKey<'a>(&'a [u8]);
#[derive(Default)]
struct DumpNameHash;
impl crate::src::core::SkTHash::NativeHash<DumpNameKey<'_>> for DumpNameHash {
    fn hash(&self, k: &DumpNameKey<'_>) -> u32 {
        crate::src::core::SkChecksum::SkGoodHash::default().hash_string_view(k.0)
    }
}
struct DumpNameBytes(Vec<u8>);
impl crate::src::core::SkTHash::NativeDefault for DumpNameBytes {
    fn default_construct() -> Self {
        Self(Vec::new())
    }
}
impl crate::src::core::SkTHash::NativeMove for DumpNameBytes {
    fn move_construct(v: &mut Self) -> Self {
        Self(core::mem::take(&mut v.0))
    }
    fn move_assign(dst: &mut Self, src: &mut Self) {
        *dst = Self::move_construct(src);
    }
}
struct DumpPositionNames(crate::src::core::SkTHash::THashMap<i32, DumpNameBytes>);
impl crate::src::core::SkTHash::NativeDefault for DumpPositionNames {
    fn default_construct() -> Self {
        Self(Default::default())
    }
}
impl crate::src::core::SkTHash::NativeMove for DumpPositionNames {
    fn move_construct(v: &mut Self) -> Self {
        Self(crate::src::core::SkTHash::THashMap::move_from(&mut v.0))
    }
    fn move_assign(dst: &mut Self, src: &mut Self) {
        dst.0.assign_move(&mut src.0);
    }
}
fn dump_concat(parts: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::new();
    for p in parts {
        v.extend_from_slice(p);
    }
    v
}
fn dump_c_string(v: &[u8]) -> &[u8] {
    &v[..v.iter().position(|&b| b == 0).unwrap_or(v.len())]
}
unsafe fn dump_write(
    out: &mut core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
    v: &[u8],
) {
    let v = dump_c_string(v);
    let _ = out.as_mut().write(v.as_ptr().cast(), v.len());
}
unsafe fn dump_unpack<T: Copy>(v: *const c_void) -> T {
    use crate::src::core::SkRasterPipelineContextUtils::{Unpack, UnpackedType};
    match Unpack(v.cast::<T>()) {
        UnpackedType::Value(x) => x,
        UnpackedType::Borrowed(x) => *x,
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:283-284
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2483-2483
// The native nested class is private in this Rust module as well.
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2894-2902
struct Dumper<'p> {
    fUniforms: Option<UniformSpan>,
    fSlots: Option<SlotData>,
    fLabelToStageMap: crate::src::core::SkTHash::THashMap<i32, i32>,
    fSlotNameList: Vec<Vec<u8>>,
    fStages: Vec<Stage>,
    fProgram: &'p Program,
    N: usize,
}
impl<'p> Dumper<'p> {
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2485-2485
    fn new(p: &'p Program) -> Self {
        Self {
            N: stride(),
            fProgram: p,
            fStages: Vec::new(),
            fSlotNameList: Vec::new(),
            fLabelToStageMap: Default::default(),
            fSlots: None,
            fUniforms: None,
        }
    }
    fn slots(&self) -> &SlotData {
        self.fSlots
            .as_ref()
            .expect("native slot spans assigned before dump helpers")
    }
    fn uniforms(&self) -> UniformSpan {
        self.fUniforms
            .expect("native uniform span assigned before dump helpers")
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2490-2498
    fn buildLabelToStageMap(&mut self) {
        for (index, stage) in self.fStages.iter().enumerate() {
            if stage.op == ProgramOp::label {
                let id = stage.ctx as isize as i32;
                debug_assert!(self.fLabelToStageMap.find(&id).is_none());
                *self.fLabelToStageMap.index_or_default(&id) = index as i32;
            }
        }
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2502-2535
    unsafe fn buildUniqueSlotNameList(&mut self) {
        if let Some(trace) = self.fProgram.fDebugTrace {
            let info = &trace.as_ref().fSlotInfo;
            self.fSlotNameList.reserve_exact(info.len());
            let mut names: crate::src::core::SkTHash::THashMap<
                DumpNameKey<'_>,
                DumpPositionNames,
                DumpNameHash,
            > = Default::default();
            for slot in info {
                let pos = if slot.pos.valid() {
                    slot.pos.startOffset()
                } else {
                    0
                };
                let positions = &mut names.index_or_default(&DumpNameKey(&slot.name)).0;
                let unique = positions.index_or_default(&pos) as *mut DumpNameBytes;
                if (*unique).0.is_empty() {
                    (*unique).0 = slot.name.clone();
                    let subscript = positions.count() - 1;
                    if subscript > 0 {
                        for digit in subscript.to_string().bytes() {
                            (*unique)
                                .0
                                .extend_from_slice(&[0xE2, 0x82, 0x80 + digit - b'0']);
                        }
                    }
                }
                self.fSlotNameList.push((*unique).0.clone());
            }
        }
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2538-2545
    unsafe fn branchOffset(&self, ctx: *const C::BranchCtx, index: i32) -> Vec<u8> {
        let label = (*ctx).offset;
        let target = *self
            .fLabelToStageMap
            .find(&label)
            .expect("native label exists");
        crate::src::sksl::SkSLString::printf(
            c"%+d (label %d at #%d)".as_ptr(),
            target - index,
            label,
            target + 1,
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2548-2565
    fn imm(&self, value: f32, showAsFloat: bool) -> Vec<u8> {
        if value.to_bits() == 0 {
            return b"0".to_vec();
        }
        let mut text =
            unsafe { crate::src::sksl::SkSLString::printf(c"0x%08X".as_ptr(), value.to_bits()) };
        if showAsFloat && value.is_finite() {
            text.extend_from_slice(b" (");
            text.extend(crate::src::sksl::SkSLString::to_string_float(value));
            text.push(b')');
        }
        text
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2568-2572
    fn immCtx(&self, ctx: *const c_void, show: bool) -> Vec<u8> {
        let bits = ctx as usize;
        let bytes = bits.to_ne_bytes();
        self.imm(f32::from_ne_bytes(bytes[..4].try_into().unwrap()), show)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2575-2581
    fn asRange(&self, first: i32, count: i32) -> Vec<u8> {
        let mut t = first.to_string().into_bytes();
        if count > 1 {
            t.extend_from_slice(b"..");
            t.extend((first + count - 1).to_string().bytes());
        }
        t
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2588-2613
    fn slotOrUniformName(
        &self,
        info: &[crate::src::sksl::tracing::SkSLDebugTracePriv::SlotDebugInfo],
        names: &[Vec<u8>],
        mut range: SlotRange,
    ) -> Vec<u8> {
        debug_assert!(range.index >= 0 && (range.index + range.count) as usize <= info.len());
        let mut text = Vec::new();
        let mut separator = crate::src::sksl::SkSLString::Separator();
        while range.count > 0 {
            let slot = &info[range.index as usize];
            text.extend_from_slice(separator());
            text.extend_from_slice(if names.is_empty() {
                &slot.name
            } else {
                &names[range.index as usize]
            });
            let entire = slot.columns as i32 * slot.rows as i32;
            let chomp = range.count.min(entire - slot.componentIndex as i32);
            if chomp != entire {
                text.push(b'(');
                text.extend(self.asRange(slot.componentIndex as i32, chomp));
                text.push(b')');
            }
            range.index += chomp;
            range.count -= chomp;
        }
        text
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2616-2618
    unsafe fn slotName(&self, r: SlotRange) -> Vec<u8> {
        self.slotOrUniformName(
            &self
                .fProgram
                .fDebugTrace
                .expect("native debug trace")
                .as_ref()
                .fSlotInfo,
            &self.fSlotNameList,
            r,
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2621-2623
    unsafe fn uniformName(&self, r: SlotRange) -> Vec<u8> {
        self.slotOrUniformName(
            &self
                .fProgram
                .fDebugTrace
                .expect("native debug trace")
                .as_ref()
                .fUniformInfo,
            &[],
            r,
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2626-2641
    unsafe fn uniformPtrCtx(&self, p: *const f32, count: i32) -> Vec<u8> {
        let u = self.uniforms();
        let end = (p as usize).wrapping_add(count as usize * 4);
        if p as usize >= u.ptr as usize && end <= (u.ptr as usize).wrapping_add(u.count * 4) {
            let index = ((p as usize - u.ptr as usize) / 4) as i32;
            if self.fProgram.fDebugTrace.is_some() {
                let name = self.uniformName(SlotRange { index, count });
                if !name.is_empty() {
                    return name;
                }
            }
            return dump_concat(&[b"u", &self.asRange(index, count)]);
        }
        Vec::new()
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2644-2661
    unsafe fn valuePtrCtx(&self, p: *const f32, count: i32) -> Vec<u8> {
        let base = self.slots().values as usize;
        let end = (p as usize).wrapping_add(self.N * count as usize * 4);
        if p as usize >= base && end <= base + self.N * self.fProgram.fNumValueSlots as usize * 4 {
            let mut index = (p as usize - base) / 4;
            debug_assert!(index % self.N == 0);
            index /= self.N;
            if self.fProgram.fDebugTrace.is_some() {
                let name = self.slotName(SlotRange {
                    index: index as i32,
                    count,
                });
                if !name.is_empty() {
                    return name;
                }
            }
            return dump_concat(&[b"v", &self.asRange(index as i32, count)]);
        }
        Vec::new()
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2664-2674
    unsafe fn immutablePtrCtx(&self, p: *const f32, count: i32) -> Vec<u8> {
        let base = self.slots().immutable as usize;
        let end = (p as usize).wrapping_add(count as usize * 4);
        if p as usize >= base && end <= base + self.fProgram.fNumImmutableSlots as usize * 4 {
            let index = ((p as usize - base) / 4) as i32;
            return dump_concat(&[
                b"i",
                &self.asRange(index, count),
                b" ",
                &self.multiImmCtx(p, count),
            ]);
        }
        Vec::new()
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2677-2694
    unsafe fn multiImmCtx(&self, mut p: *const f32, mut count: i32) -> Vec<u8> {
        let uniform = self.uniformPtrCtx(p, count);
        if !uniform.is_empty() {
            return uniform;
        }
        if count == 1 {
            return dump_concat(&[b"[", &self.imm(p.read(), true), b"]"]);
        }
        let mut text = b"[".to_vec();
        let mut separator = crate::src::sksl::SkSLString::Separator();
        while count > 0 {
            count -= 1;
            text.extend_from_slice(separator());
            text.extend(self.imm(p.read(), true));
            p = p.add(1);
        }
        text.push(b']');
        text
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2697-2719
    unsafe fn ptrCtx(&self, ctx: *const c_void, count: i32) -> Vec<u8> {
        let p = ctx.cast::<f32>();
        let uniform = self.uniformPtrCtx(p, count);
        if !uniform.is_empty() {
            return uniform;
        }
        let value = self.valuePtrCtx(p, count);
        if !value.is_empty() {
            return value;
        }
        let immutable = self.immutablePtrCtx(p, count);
        if !immutable.is_empty() {
            return immutable;
        }
        let base = self.slots().stack as usize;
        if p as usize >= base && (p as usize) < base + self.slots().stack_len * 4 {
            let index = (p as usize - base) / 4;
            debug_assert!(index % self.N == 0);
            return dump_concat(&[b"$", &self.asRange((index / self.N) as i32, count)]);
        }
        dump_concat(&[b"ExternalPtr(", &self.asRange(0, count), b")"])
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2722-2724
    unsafe fn offsetToPtr(&self, offset: SkRPOffset) -> *mut c_void {
        self.slots().values.cast::<u8>().add(offset as usize).cast()
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2727-2729
    unsafe fn offsetCtx(&self, offset: SkRPOffset, count: i32) -> Vec<u8> {
        self.ptrCtx(self.offsetToPtr(offset), count)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2732-2738
    unsafe fn constantCtx(&self, v: *const c_void, count: i32, show: bool) -> (Vec<u8>, Vec<u8>) {
        let ctx: C::ConstantCtx = dump_unpack(v);
        (
            self.offsetCtx(ctx.dst, count),
            self.imm(f32::from_bits(ctx.value as u32), show),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2742-2746
    unsafe fn binaryOpCtx(&self, v: *const c_void, count: i32) -> (Vec<u8>, Vec<u8>) {
        let ctx: C::BinaryOpCtx = dump_unpack(v);
        (
            self.offsetCtx(ctx.dst, count),
            self.offsetCtx(ctx.src, count),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2750-2754
    unsafe fn copyUniformCtx(&self, v: *const c_void, count: i32) -> (Vec<u8>, Vec<u8>) {
        let ctx = &*v.cast::<C::UniformCtx>();
        (
            self.ptrCtx(ctx.dst.cast(), count),
            self.multiImmCtx(ctx.src.cast(), count),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2757-2761
    unsafe fn adjacentPtrCtx(&self, v: *const c_void, count: i32) -> (Vec<u8>, Vec<u8>) {
        (
            self.ptrCtx(v, count),
            self.ptrCtx(v.cast::<f32>().add(self.N * count as usize).cast(), count),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2764-2766
    unsafe fn adjacentOffsetCtx(&self, offset: SkRPOffset, count: i32) -> (Vec<u8>, Vec<u8>) {
        self.adjacentPtrCtx(self.offsetToPtr(offset), count)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2770-2774
    unsafe fn adjacentBinaryOpCtx(&self, v: *const c_void) -> (Vec<u8>, Vec<u8>) {
        let ctx: C::BinaryOpCtx = dump_unpack(v);
        let count = ctx.src.wrapping_sub(ctx.dst) as usize / (self.N * 4);
        self.adjacentOffsetCtx(ctx.dst, count as i32)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2777-2783
    unsafe fn adjacent3PtrCtx(&self, v: *const c_void, count: i32) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        (
            self.ptrCtx(v, count),
            self.ptrCtx(v.cast::<f32>().add(self.N * count as usize).cast(), count),
            self.ptrCtx(
                v.cast::<f32>().add(2 * self.N * count as usize).cast(),
                count,
            ),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2786-2789
    unsafe fn adjacent3OffsetCtx(
        &self,
        offset: SkRPOffset,
        count: i32,
    ) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        self.adjacent3PtrCtx(self.offsetToPtr(offset), count)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2792-2796
    unsafe fn adjacentTernaryOpCtx(&self, v: *const c_void) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let ctx: C::TernaryOpCtx = dump_unpack(v);
        self.adjacent3OffsetCtx(ctx.dst, (ctx.delta as usize / (4 * self.N)) as i32)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2800-2816
    fn swizzleOffsetSpan<T: Copy + Into<usize>>(&self, offsets: &[T]) -> Vec<u8> {
        offsets
            .iter()
            .map(|&x| {
                let x = x.into() as u16 as usize;
                if x == 0 {
                    b'x'
                } else if x == self.N * 4 {
                    b'y'
                } else if x == 2 * self.N * 4 {
                    b'z'
                } else if x == 3 * self.N * 4 {
                    b'w'
                } else {
                    b'?'
                }
            })
            .collect()
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2823-2828
    fn swizzleWidth<T: Copy + Into<usize>>(&self, offsets: &[T]) -> usize {
        let highest = offsets
            .iter()
            .map(|&x| x.into())
            .max()
            .expect("native nonempty swizzle")
            / (self.N * 4);
        offsets.len().max(highest + 1)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2832-2835
    unsafe fn swizzlePtr<T: Copy + Into<usize>>(&self, p: *const c_void, offsets: &[T]) -> Vec<u8> {
        dump_concat(&[
            b"(",
            &self.ptrCtx(p, self.swizzleWidth(offsets) as i32),
            b").",
            &self.swizzleOffsetSpan(offsets),
        ])
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2838-2843
    unsafe fn swizzleCtx(&self, op: ProgramOp, v: *const c_void) -> (Vec<u8>, Vec<u8>) {
        let ctx: C::SwizzleCtx = dump_unpack(v);
        let count = op as i32 - BuilderOp::swizzle_1 as i32 + 1;
        (
            self.offsetCtx(ctx.dst, count),
            self.swizzlePtr(self.offsetToPtr(ctx.dst), &ctx.offsets[..count as usize]),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2846-2852
    unsafe fn swizzleCopyCtx(&self, op: ProgramOp, v: *const c_void) -> (Vec<u8>, Vec<u8>) {
        let ctx = &*v.cast::<C::SwizzleCopyCtx>();
        let count = op as i32 - BuilderOp::swizzle_copy_slot_masked as i32 + 1;
        (
            self.swizzlePtr(ctx.dst.cast(), &ctx.offsets[..count as usize]),
            self.ptrCtx(ctx.src.cast(), count),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2855-2870
    unsafe fn shuffleCtx(&self, v: *const c_void) -> (Vec<u8>, Vec<u8>) {
        let ctx = &*v.cast::<C::ShuffleCtx>();
        let dst = self.ptrCtx(ctx.ptr.cast(), ctx.count);
        let mut src = dump_concat(&[b"(", &dst, b")["]);
        for index in 0..ctx.count as usize {
            let offset = ctx.offsets[index] as usize;
            if offset % (self.N * 4) != 0 {
                src.push(b'?');
            } else {
                src.extend((offset / (self.N * 4)).to_string().bytes());
            }
            src.push(b' ');
        }
        *src.last_mut().expect("native count positive") = b']';
        (dst, src)
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2873-2892
    unsafe fn matrixMultiply(&self, v: *const c_void) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let c: C::MatrixMultiplyCtx = dump_unpack(v);
        let lc = c.leftColumns as i32;
        let lr = c.leftRows as i32;
        let rc = c.rightColumns as i32;
        let rr = c.rightRows as i32;
        let left = c.dst + (rc * lr) as u32 * 4 * self.N as u32;
        let right = left + (lc * lr) as u32 * 4 * self.N as u32;
        let mat = |cols: i32, rows: i32, offset: u32, count: i32| {
            let mut text = self.offsetCtx(offset, count);
            text.push(0);
            crate::src::sksl::SkSLString::printf(
                c"mat%dx%d(%s)".as_ptr(),
                cols,
                rows,
                text.as_ptr(),
            )
        };
        (
            mat(rc, lr, c.dst, rc * lr),
            mat(lc, lr, left, lc * lr),
            mat(rc, rr, right, rc * rr),
        )
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2904-3836
    unsafe fn dump(
        &mut self,
        mut output: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
        writeInstructionCount: bool,
    ) {
        let out = &mut output;
        let mut owner = core::pin::pin!(SkArenaAlloc::new(1000));
        let alloc = owner.as_mut().get_unchecked_mut();
        let slots = self
            .fProgram
            .allocateSlotData(alloc)
            .expect("native optional.value slot allocation");
        let ptr = alloc
            .makeBytesAlignedTo(
                self.fProgram.fNumUniformSlots as usize * 4,
                core::mem::align_of::<f32>(),
            )
            .cast::<f32>();
        let uniforms = UniformSpan {
            ptr,
            count: self.fProgram.fNumUniformSlots as usize,
        };
        self.fSlots = Some(slots);
        self.fUniforms = Some(uniforms);
        let slots_ref = self.fSlots.as_ref().expect("native slot spans initialized");
        self.fProgram
            .makeStages(&mut self.fStages, alloc, uniforms, slots_ref);
        self.buildLabelToStageMap();
        self.buildUniqueSlotNameList();
        if writeInstructionCount {
            let mut invocations = 0;
            let mut instructions = 0;
            for stage in &self.fStages {
                match stage.op {
                    ProgramOp::label => {}
                    ProgramOp::invoke_shader
                    | ProgramOp::invoke_color_filter
                    | ProgramOp::invoke_blender
                    | ProgramOp::invoke_to_linear_srgb
                    | ProgramOp::invoke_from_linear_srgb => invocations += 1,
                    _ => instructions += 1,
                }
            }
            dump_write(out, instructions.to_string().as_bytes());
            dump_write(out, b" instructions");
            if invocations > 0 {
                dump_write(out, b", ");
                dump_write(out, invocations.to_string().as_bytes());
                dump_write(out, b" invocations");
            }
            dump_write(out, b"\n\n");
        }
        let mut header: &[u8] = b"[immutable slots]\n";
        let mut footer: &[u8] = b"";
        for inst in &self.fProgram.fInstructions {
            if inst.fOp == BuilderOp::store_immutable_value {
                dump_write(out, header);
                dump_write(out, b"i");
                dump_write(out, inst.fSlotA.to_string().as_bytes());
                dump_write(out, b" = ");
                dump_write(out, &self.imm(f32::from_bits(inst.fImmA as u32), true));
                dump_write(out, b"\n");
                header = b"";
                footer = b"\n";
            }
        }
        dump_write(out, footer);
        for index in 0..self.fStages.len() {
            let stage = self.fStages[index];
            let op = stage.op;
            let (mut a1, mut a2, mut a3, mut swizzle) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            match op {
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2979-2984
                ProgramOp::label
                | ProgramOp::invoke_shader
                | ProgramOp::invoke_color_filter
                | ProgramOp::invoke_blender => {
                    a1 = self.immCtx(stage.ctx, false);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2986-2992
                ProgramOp::case_op => {
                    let c: C::CaseOpCtx = dump_unpack(stage.ctx);
                    a1 = self.offsetCtx(c.offset, 1);
                    a2 = self.offsetCtx(c.offset + (4 * self.N) as u32, 1);
                    a3 = self.imm(f32::from_bits(c.expectedValue as u32), false);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:2994-2999
                ProgramOp::swizzle_1
                | ProgramOp::swizzle_2
                | ProgramOp::swizzle_3
                | ProgramOp::swizzle_4 => {
                    (a1, a2) = self.swizzleCtx(op, stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3001-3006
                ProgramOp::swizzle_copy_slot_masked
                | ProgramOp::swizzle_copy_2_slots_masked
                | ProgramOp::swizzle_copy_3_slots_masked
                | ProgramOp::swizzle_copy_4_slots_masked => {
                    (a1, a2) = self.swizzleCopyCtx(op, stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3008-3011
                ProgramOp::refract_4_floats => {
                    (a1, a2) = self.adjacentPtrCtx(stage.ctx, 4);
                    a3 = self.ptrCtx(stage.ctx.cast::<f32>().add(8 * self.N).cast(), 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3013-3016
                ProgramOp::dot_2_floats => {
                    a1 = self.ptrCtx(stage.ctx, 1);
                    (a2, a3) = self.adjacentPtrCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3018-3021
                ProgramOp::dot_3_floats => {
                    a1 = self.ptrCtx(stage.ctx, 1);
                    (a2, a3) = self.adjacentPtrCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3023-3026
                ProgramOp::dot_4_floats => {
                    a1 = self.ptrCtx(stage.ctx, 1);
                    (a2, a3) = self.adjacentPtrCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3028-3030
                ProgramOp::shuffle => {
                    (a1, a2) = self.shuffleCtx(stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3032-3036
                ProgramOp::matrix_multiply_2
                | ProgramOp::matrix_multiply_3
                | ProgramOp::matrix_multiply_4 => {
                    (a1, a2, a3) = self.matrixMultiply(stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3038-3065
                ProgramOp::load_condition_mask
                | ProgramOp::store_condition_mask
                | ProgramOp::load_loop_mask
                | ProgramOp::store_loop_mask
                | ProgramOp::merge_loop_mask
                | ProgramOp::reenable_loop_mask
                | ProgramOp::load_return_mask
                | ProgramOp::store_return_mask
                | ProgramOp::continue_op
                | ProgramOp::cast_to_float_from_int
                | ProgramOp::cast_to_float_from_uint
                | ProgramOp::cast_to_int_from_float
                | ProgramOp::cast_to_uint_from_float
                | ProgramOp::abs_int
                | ProgramOp::acos_float
                | ProgramOp::asin_float
                | ProgramOp::atan_float
                | ProgramOp::ceil_float
                | ProgramOp::cos_float
                | ProgramOp::exp_float
                | ProgramOp::exp2_float
                | ProgramOp::log_float
                | ProgramOp::log2_float
                | ProgramOp::floor_float
                | ProgramOp::invsqrt_float
                | ProgramOp::sin_float
                | ProgramOp::sqrt_float
                | ProgramOp::tan_float => {
                    a1 = self.ptrCtx(stage.ctx, 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3067-3075
                ProgramOp::store_src_rg
                | ProgramOp::cast_to_float_from_2_ints
                | ProgramOp::cast_to_float_from_2_uints
                | ProgramOp::cast_to_int_from_2_floats
                | ProgramOp::cast_to_uint_from_2_floats
                | ProgramOp::abs_2_ints
                | ProgramOp::ceil_2_floats
                | ProgramOp::floor_2_floats
                | ProgramOp::invsqrt_2_floats => {
                    a1 = self.ptrCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3077-3084
                ProgramOp::cast_to_float_from_3_ints
                | ProgramOp::cast_to_float_from_3_uints
                | ProgramOp::cast_to_int_from_3_floats
                | ProgramOp::cast_to_uint_from_3_floats
                | ProgramOp::abs_3_ints
                | ProgramOp::ceil_3_floats
                | ProgramOp::floor_3_floats
                | ProgramOp::invsqrt_3_floats => {
                    a1 = self.ptrCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3086-3102
                ProgramOp::load_src
                | ProgramOp::load_dst
                | ProgramOp::exchange_src
                | ProgramOp::store_src
                | ProgramOp::store_dst
                | ProgramOp::store_device_xy01
                | ProgramOp::invoke_to_linear_srgb
                | ProgramOp::invoke_from_linear_srgb
                | ProgramOp::cast_to_float_from_4_ints
                | ProgramOp::cast_to_float_from_4_uints
                | ProgramOp::cast_to_int_from_4_floats
                | ProgramOp::cast_to_uint_from_4_floats
                | ProgramOp::abs_4_ints
                | ProgramOp::ceil_4_floats
                | ProgramOp::floor_4_floats
                | ProgramOp::invsqrt_4_floats
                | ProgramOp::inverse_mat2 => {
                    a1 = self.ptrCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3104-3106
                ProgramOp::inverse_mat3 => {
                    a1 = self.ptrCtx(stage.ctx, 9);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3108-3110
                ProgramOp::inverse_mat4 => {
                    a1 = self.ptrCtx(stage.ctx, 16);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3112-3122
                ProgramOp::copy_constant
                | ProgramOp::add_imm_float
                | ProgramOp::mul_imm_float
                | ProgramOp::cmple_imm_float
                | ProgramOp::cmplt_imm_float
                | ProgramOp::cmpeq_imm_float
                | ProgramOp::cmpne_imm_float
                | ProgramOp::min_imm_float
                | ProgramOp::max_imm_float => {
                    (a1, a2) = self.constantCtx(stage.ctx, 1, true);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3124-3135
                ProgramOp::add_imm_int
                | ProgramOp::mul_imm_int
                | ProgramOp::bitwise_and_imm_int
                | ProgramOp::bitwise_xor_imm_int
                | ProgramOp::cmple_imm_int
                | ProgramOp::cmple_imm_uint
                | ProgramOp::cmplt_imm_int
                | ProgramOp::cmplt_imm_uint
                | ProgramOp::cmpeq_imm_int
                | ProgramOp::cmpne_imm_int => {
                    (a1, a2) = self.constantCtx(stage.ctx, 1, false);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3137-3140
                ProgramOp::splat_2_constants | ProgramOp::bitwise_and_imm_2_ints => {
                    (a1, a2) = self.constantCtx(stage.ctx, 2, true);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3142-3145
                ProgramOp::splat_3_constants | ProgramOp::bitwise_and_imm_3_ints => {
                    (a1, a2) = self.constantCtx(stage.ctx, 3, true);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3147-3150
                ProgramOp::splat_4_constants | ProgramOp::bitwise_and_imm_4_ints => {
                    (a1, a2) = self.constantCtx(stage.ctx, 4, true);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3152-3154
                ProgramOp::copy_uniform => {
                    (a1, a2) = self.copyUniformCtx(stage.ctx, 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3156-3158
                ProgramOp::copy_2_uniforms => {
                    (a1, a2) = self.copyUniformCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3160-3162
                ProgramOp::copy_3_uniforms => {
                    (a1, a2) = self.copyUniformCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3164-3166
                ProgramOp::copy_4_uniforms => {
                    (a1, a2) = self.copyUniformCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3168-3172
                ProgramOp::copy_slot_masked
                | ProgramOp::copy_slot_unmasked
                | ProgramOp::copy_immutable_unmasked => {
                    (a1, a2) = self.binaryOpCtx(stage.ctx, 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3174-3178
                ProgramOp::copy_2_slots_masked
                | ProgramOp::copy_2_slots_unmasked
                | ProgramOp::copy_2_immutables_unmasked => {
                    (a1, a2) = self.binaryOpCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3180-3184
                ProgramOp::copy_3_slots_masked
                | ProgramOp::copy_3_slots_unmasked
                | ProgramOp::copy_3_immutables_unmasked => {
                    (a1, a2) = self.binaryOpCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3186-3190
                ProgramOp::copy_4_slots_masked
                | ProgramOp::copy_4_slots_unmasked
                | ProgramOp::copy_4_immutables_unmasked => {
                    (a1, a2) = self.binaryOpCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3192-3201
                ProgramOp::copy_from_indirect_uniform_unmasked
                | ProgramOp::copy_from_indirect_unmasked
                | ProgramOp::copy_to_indirect_masked => {
                    let c = &*stage.ctx.cast::<C::CopyIndirectCtx>();
                    a1 = self.ptrCtx(c.dst.cast(), c.slots as i32);
                    a2 = self.ptrCtx(c.src.cast(), c.slots as i32);
                    a3 = self.ptrCtx(c.indirectOffset.cast(), 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3203-3211
                ProgramOp::swizzle_copy_to_indirect_masked => {
                    let c = &*stage.ctx.cast::<C::SwizzleCopyIndirectCtx>();
                    a1 = self.ptrCtx(
                        c.base.dst.cast(),
                        self.swizzleWidth(&c.offsets[..c.base.slots as usize]) as i32,
                    );
                    a2 = self.ptrCtx(c.base.src.cast(), c.base.slots as i32);
                    a3 = self.ptrCtx(c.base.indirectOffset.cast(), 1);
                    swizzle = self.swizzleOffsetSpan(&c.offsets[..c.base.slots as usize]);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3213-3230
                ProgramOp::merge_condition_mask
                | ProgramOp::merge_inv_condition_mask
                | ProgramOp::add_float
                | ProgramOp::add_int
                | ProgramOp::sub_float
                | ProgramOp::sub_int
                | ProgramOp::mul_float
                | ProgramOp::mul_int
                | ProgramOp::div_float
                | ProgramOp::div_int
                | ProgramOp::div_uint
                | ProgramOp::bitwise_and_int
                | ProgramOp::bitwise_or_int
                | ProgramOp::bitwise_xor_int
                | ProgramOp::mod_float
                | ProgramOp::min_float
                | ProgramOp::min_int
                | ProgramOp::min_uint
                | ProgramOp::max_float
                | ProgramOp::max_int
                | ProgramOp::max_uint
                | ProgramOp::cmplt_float
                | ProgramOp::cmplt_int
                | ProgramOp::cmplt_uint
                | ProgramOp::cmple_float
                | ProgramOp::cmple_int
                | ProgramOp::cmple_uint
                | ProgramOp::cmpeq_float
                | ProgramOp::cmpeq_int
                | ProgramOp::cmpne_float
                | ProgramOp::cmpne_int => {
                    (a1, a2) = self.adjacentPtrCtx(stage.ctx, 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3232-3234
                ProgramOp::mix_float | ProgramOp::mix_int => {
                    (a1, a2, a3) = self.adjacent3PtrCtx(stage.ctx, 1);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3236-3251
                ProgramOp::add_2_floats
                | ProgramOp::add_2_ints
                | ProgramOp::sub_2_floats
                | ProgramOp::sub_2_ints
                | ProgramOp::mul_2_floats
                | ProgramOp::mul_2_ints
                | ProgramOp::div_2_floats
                | ProgramOp::div_2_ints
                | ProgramOp::div_2_uints
                | ProgramOp::bitwise_and_2_ints
                | ProgramOp::bitwise_or_2_ints
                | ProgramOp::bitwise_xor_2_ints
                | ProgramOp::mod_2_floats
                | ProgramOp::min_2_floats
                | ProgramOp::min_2_ints
                | ProgramOp::min_2_uints
                | ProgramOp::max_2_floats
                | ProgramOp::max_2_ints
                | ProgramOp::max_2_uints
                | ProgramOp::cmplt_2_floats
                | ProgramOp::cmplt_2_ints
                | ProgramOp::cmplt_2_uints
                | ProgramOp::cmple_2_floats
                | ProgramOp::cmple_2_ints
                | ProgramOp::cmple_2_uints
                | ProgramOp::cmpeq_2_floats
                | ProgramOp::cmpeq_2_ints
                | ProgramOp::cmpne_2_floats
                | ProgramOp::cmpne_2_ints => {
                    (a1, a2) = self.adjacentPtrCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3253-3255
                ProgramOp::mix_2_floats | ProgramOp::mix_2_ints => {
                    (a1, a2, a3) = self.adjacent3PtrCtx(stage.ctx, 2);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3257-3272
                ProgramOp::add_3_floats
                | ProgramOp::add_3_ints
                | ProgramOp::sub_3_floats
                | ProgramOp::sub_3_ints
                | ProgramOp::mul_3_floats
                | ProgramOp::mul_3_ints
                | ProgramOp::div_3_floats
                | ProgramOp::div_3_ints
                | ProgramOp::div_3_uints
                | ProgramOp::bitwise_and_3_ints
                | ProgramOp::bitwise_or_3_ints
                | ProgramOp::bitwise_xor_3_ints
                | ProgramOp::mod_3_floats
                | ProgramOp::min_3_floats
                | ProgramOp::min_3_ints
                | ProgramOp::min_3_uints
                | ProgramOp::max_3_floats
                | ProgramOp::max_3_ints
                | ProgramOp::max_3_uints
                | ProgramOp::cmplt_3_floats
                | ProgramOp::cmplt_3_ints
                | ProgramOp::cmplt_3_uints
                | ProgramOp::cmple_3_floats
                | ProgramOp::cmple_3_ints
                | ProgramOp::cmple_3_uints
                | ProgramOp::cmpeq_3_floats
                | ProgramOp::cmpeq_3_ints
                | ProgramOp::cmpne_3_floats
                | ProgramOp::cmpne_3_ints => {
                    (a1, a2) = self.adjacentPtrCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3274-3276
                ProgramOp::mix_3_floats | ProgramOp::mix_3_ints => {
                    (a1, a2, a3) = self.adjacent3PtrCtx(stage.ctx, 3);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3278-3293
                ProgramOp::add_4_floats
                | ProgramOp::add_4_ints
                | ProgramOp::sub_4_floats
                | ProgramOp::sub_4_ints
                | ProgramOp::mul_4_floats
                | ProgramOp::mul_4_ints
                | ProgramOp::div_4_floats
                | ProgramOp::div_4_ints
                | ProgramOp::div_4_uints
                | ProgramOp::bitwise_and_4_ints
                | ProgramOp::bitwise_or_4_ints
                | ProgramOp::bitwise_xor_4_ints
                | ProgramOp::mod_4_floats
                | ProgramOp::min_4_floats
                | ProgramOp::min_4_ints
                | ProgramOp::min_4_uints
                | ProgramOp::max_4_floats
                | ProgramOp::max_4_ints
                | ProgramOp::max_4_uints
                | ProgramOp::cmplt_4_floats
                | ProgramOp::cmplt_4_ints
                | ProgramOp::cmplt_4_uints
                | ProgramOp::cmple_4_floats
                | ProgramOp::cmple_4_ints
                | ProgramOp::cmple_4_uints
                | ProgramOp::cmpeq_4_floats
                | ProgramOp::cmpeq_4_ints
                | ProgramOp::cmpne_4_floats
                | ProgramOp::cmpne_4_ints => {
                    (a1, a2) = self.adjacentPtrCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3295-3297
                ProgramOp::mix_4_floats | ProgramOp::mix_4_ints => {
                    (a1, a2, a3) = self.adjacent3PtrCtx(stage.ctx, 4);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3299-3316
                ProgramOp::add_n_floats
                | ProgramOp::add_n_ints
                | ProgramOp::sub_n_floats
                | ProgramOp::sub_n_ints
                | ProgramOp::mul_n_floats
                | ProgramOp::mul_n_ints
                | ProgramOp::div_n_floats
                | ProgramOp::div_n_ints
                | ProgramOp::div_n_uints
                | ProgramOp::bitwise_and_n_ints
                | ProgramOp::bitwise_or_n_ints
                | ProgramOp::bitwise_xor_n_ints
                | ProgramOp::mod_n_floats
                | ProgramOp::min_n_floats
                | ProgramOp::min_n_ints
                | ProgramOp::min_n_uints
                | ProgramOp::max_n_floats
                | ProgramOp::max_n_ints
                | ProgramOp::max_n_uints
                | ProgramOp::cmplt_n_floats
                | ProgramOp::cmplt_n_ints
                | ProgramOp::cmplt_n_uints
                | ProgramOp::cmple_n_floats
                | ProgramOp::cmple_n_ints
                | ProgramOp::cmple_n_uints
                | ProgramOp::cmpeq_n_floats
                | ProgramOp::cmpeq_n_ints
                | ProgramOp::cmpne_n_floats
                | ProgramOp::cmpne_n_ints
                | ProgramOp::atan2_n_floats
                | ProgramOp::pow_n_floats => {
                    (a1, a2) = self.adjacentBinaryOpCtx(stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3318-3321
                ProgramOp::mix_n_floats
                | ProgramOp::mix_n_ints
                | ProgramOp::smoothstep_n_floats => {
                    (a1, a2, a3) = self.adjacentTernaryOpCtx(stage.ctx);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3323-3329
                ProgramOp::jump
                | ProgramOp::branch_if_all_lanes_active
                | ProgramOp::branch_if_any_lanes_active
                | ProgramOp::branch_if_no_lanes_active => {
                    a1 = self.branchOffset(stage.ctx.cast::<C::BranchCtx>(), index as i32);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3331-3337
                ProgramOp::branch_if_no_active_lanes_eq => {
                    let c = &*stage.ctx.cast::<C::BranchIfEqualCtx>();
                    a1 = self.branchOffset(core::ptr::addr_of!(c.branch), index as i32);
                    a2 = self.ptrCtx(c.ptr.cast(), 1);
                    a3 = self.imm(f32::from_bits(c.value as u32), true);
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3339-3346
                ProgramOp::trace_var => {
                    let c = &*stage.ctx.cast::<C::TraceVarCtx<'_>>();
                    a1 = self.ptrCtx(c.traceMask.cast(), 1);
                    a2 = self.ptrCtx(c.data.cast(), c.numSlots);
                    if !c.indirectOffset.is_null() {
                        a3 = dump_concat(&[b" + ", &self.ptrCtx(c.indirectOffset.cast(), 1)]);
                    }
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3348-3352
                ProgramOp::trace_line => {
                    let c = &*stage.ctx.cast::<C::TraceLineCtx<'_>>();
                    a1 = self.ptrCtx(c.traceMask.cast(), 1);
                    a2 = c.lineNumber.to_string().into_bytes();
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3354-3363
                ProgramOp::trace_enter | ProgramOp::trace_exit => {
                    let c = &*stage.ctx.cast::<C::TraceFuncCtx<'_>>();
                    a1 = self.ptrCtx(c.traceMask.cast(), 1);
                    a2 = if let Some(t) = self.fProgram.fDebugTrace {
                        if c.funcIdx >= 0 && (c.funcIdx as usize) < t.as_ref().fFuncInfo.len() {
                            t.as_ref().fFuncInfo[c.funcIdx as usize].name.clone()
                        } else {
                            b"???".to_vec()
                        }
                    } else {
                        b"???".to_vec()
                    };
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3365-3369
                ProgramOp::trace_scope => {
                    let c = &*stage.ctx.cast::<C::TraceScopeCtx<'_>>();
                    a1 = self.ptrCtx(c.traceMask.cast(), 1);
                    a2 = crate::src::sksl::SkSLString::printf(c"%+d".as_ptr(), c.delta);
                }
                _ => {}
            }
            // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3375-3382
            fn dump_op_name(op: ProgramOp) -> &'static [u8] {
                match op {
                    ProgramOp::move_src_dst => b"move_src_dst",
                    ProgramOp::move_dst_src => b"move_dst_src",
                    ProgramOp::swap_src_dst => b"swap_src_dst",
                    ProgramOp::clamp_01 => b"clamp_01",
                    ProgramOp::clamp_a_01 => b"clamp_a_01",
                    ProgramOp::clamp_gamut => b"clamp_gamut",
                    ProgramOp::premul => b"premul",
                    ProgramOp::premul_dst => b"premul_dst",
                    ProgramOp::force_opaque => b"force_opaque",
                    ProgramOp::force_opaque_dst => b"force_opaque_dst",
                    ProgramOp::set_rgb => b"set_rgb",
                    ProgramOp::swap_rb => b"swap_rb",
                    ProgramOp::swap_rb_dst => b"swap_rb_dst",
                    ProgramOp::black_color => b"black_color",
                    ProgramOp::white_color => b"white_color",
                    ProgramOp::uniform_color => b"uniform_color",
                    ProgramOp::uniform_color_dst => b"uniform_color_dst",
                    ProgramOp::seed_shader => b"seed_shader",
                    ProgramOp::load_a8 => b"load_a8",
                    ProgramOp::load_a8_dst => b"load_a8_dst",
                    ProgramOp::store_a8 => b"store_a8",
                    ProgramOp::gather_a8 => b"gather_a8",
                    ProgramOp::load_565 => b"load_565",
                    ProgramOp::load_565_dst => b"load_565_dst",
                    ProgramOp::store_565 => b"store_565",
                    ProgramOp::gather_565 => b"gather_565",
                    ProgramOp::load_4444 => b"load_4444",
                    ProgramOp::load_4444_dst => b"load_4444_dst",
                    ProgramOp::store_4444 => b"store_4444",
                    ProgramOp::gather_4444 => b"gather_4444",
                    ProgramOp::load_8888 => b"load_8888",
                    ProgramOp::load_8888_dst => b"load_8888_dst",
                    ProgramOp::store_8888 => b"store_8888",
                    ProgramOp::gather_8888 => b"gather_8888",
                    ProgramOp::load_rg88 => b"load_rg88",
                    ProgramOp::load_rg88_dst => b"load_rg88_dst",
                    ProgramOp::store_rg88 => b"store_rg88",
                    ProgramOp::gather_rg88 => b"gather_rg88",
                    ProgramOp::store_r8 => b"store_r8",
                    ProgramOp::alpha_to_gray => b"alpha_to_gray",
                    ProgramOp::alpha_to_gray_dst => b"alpha_to_gray_dst",
                    ProgramOp::alpha_to_red => b"alpha_to_red",
                    ProgramOp::alpha_to_red_dst => b"alpha_to_red_dst",
                    ProgramOp::bt709_luminance_or_luma_to_alpha => {
                        b"bt709_luminance_or_luma_to_alpha"
                    }
                    ProgramOp::bt709_luminance_or_luma_to_rgb => b"bt709_luminance_or_luma_to_rgb",
                    ProgramOp::bilerp_clamp_8888 => b"bilerp_clamp_8888",
                    ProgramOp::load_src => b"load_src",
                    ProgramOp::store_src => b"store_src",
                    ProgramOp::store_src_a => b"store_src_a",
                    ProgramOp::load_dst => b"load_dst",
                    ProgramOp::store_dst => b"store_dst",
                    ProgramOp::scale_u8 => b"scale_u8",
                    ProgramOp::scale_565 => b"scale_565",
                    ProgramOp::scale_1_float => b"scale_1_float",
                    ProgramOp::scale_native => b"scale_native",
                    ProgramOp::lerp_u8 => b"lerp_u8",
                    ProgramOp::lerp_565 => b"lerp_565",
                    ProgramOp::lerp_1_float => b"lerp_1_float",
                    ProgramOp::lerp_native => b"lerp_native",
                    ProgramOp::dstatop => b"dstatop",
                    ProgramOp::dstin => b"dstin",
                    ProgramOp::dstout => b"dstout",
                    ProgramOp::dstover => b"dstover",
                    ProgramOp::srcatop => b"srcatop",
                    ProgramOp::srcin => b"srcin",
                    ProgramOp::srcout => b"srcout",
                    ProgramOp::srcover => b"srcover",
                    ProgramOp::clear => b"clear",
                    ProgramOp::modulate => b"modulate",
                    ProgramOp::multiply => b"multiply",
                    ProgramOp::plus_ => b"plus_",
                    ProgramOp::screen => b"screen",
                    ProgramOp::xor_ => b"xor_",
                    ProgramOp::darken => b"darken",
                    ProgramOp::difference => b"difference",
                    ProgramOp::exclusion => b"exclusion",
                    ProgramOp::hardlight => b"hardlight",
                    ProgramOp::lighten => b"lighten",
                    ProgramOp::overlay => b"overlay",
                    ProgramOp::srcover_rgba_8888 => b"srcover_rgba_8888",
                    ProgramOp::matrix_translate => b"matrix_translate",
                    ProgramOp::matrix_scale_translate => b"matrix_scale_translate",
                    ProgramOp::matrix_2x3 => b"matrix_2x3",
                    ProgramOp::matrix_perspective => b"matrix_perspective",
                    ProgramOp::decal_x => b"decal_x",
                    ProgramOp::decal_y => b"decal_y",
                    ProgramOp::decal_x_and_y => b"decal_x_and_y",
                    ProgramOp::check_decal_mask => b"check_decal_mask",
                    ProgramOp::clamp_x_1 => b"clamp_x_1",
                    ProgramOp::mirror_x_1 => b"mirror_x_1",
                    ProgramOp::repeat_x_1 => b"repeat_x_1",
                    ProgramOp::clamp_x_and_y => b"clamp_x_and_y",
                    ProgramOp::evenly_spaced_gradient => b"evenly_spaced_gradient",
                    ProgramOp::gradient => b"gradient",
                    ProgramOp::evenly_spaced_2_stop_gradient => b"evenly_spaced_2_stop_gradient",
                    ProgramOp::xy_to_unit_angle => b"xy_to_unit_angle",
                    ProgramOp::xy_to_radius => b"xy_to_radius",
                    ProgramOp::emboss => b"emboss",
                    ProgramOp::swizzle => b"swizzle",
                    ProgramOp::debug_x => b"debug_x",
                    ProgramOp::debug_y => b"debug_y",
                    ProgramOp::debug_r => b"debug_r",
                    ProgramOp::debug_g => b"debug_g",
                    ProgramOp::debug_b => b"debug_b",
                    ProgramOp::debug_a => b"debug_a",
                    ProgramOp::debug_r_255 => b"debug_r_255",
                    ProgramOp::debug_g_255 => b"debug_g_255",
                    ProgramOp::debug_b_255 => b"debug_b_255",
                    ProgramOp::debug_a_255 => b"debug_a_255",
                    ProgramOp::init_lane_masks => b"init_lane_masks",
                    ProgramOp::store_device_xy01 => b"store_device_xy01",
                    ProgramOp::exchange_src => b"exchange_src",
                    ProgramOp::load_condition_mask => b"load_condition_mask",
                    ProgramOp::store_condition_mask => b"store_condition_mask",
                    ProgramOp::merge_condition_mask => b"merge_condition_mask",
                    ProgramOp::merge_inv_condition_mask => b"merge_inv_condition_mask",
                    ProgramOp::load_loop_mask => b"load_loop_mask",
                    ProgramOp::store_loop_mask => b"store_loop_mask",
                    ProgramOp::mask_off_loop_mask => b"mask_off_loop_mask",
                    ProgramOp::reenable_loop_mask => b"reenable_loop_mask",
                    ProgramOp::merge_loop_mask => b"merge_loop_mask",
                    ProgramOp::case_op => b"case_op",
                    ProgramOp::continue_op => b"continue_op",
                    ProgramOp::load_return_mask => b"load_return_mask",
                    ProgramOp::store_return_mask => b"store_return_mask",
                    ProgramOp::mask_off_return_mask => b"mask_off_return_mask",
                    ProgramOp::branch_if_all_lanes_active => b"branch_if_all_lanes_active",
                    ProgramOp::branch_if_any_lanes_active => b"branch_if_any_lanes_active",
                    ProgramOp::branch_if_no_lanes_active => b"branch_if_no_lanes_active",
                    ProgramOp::branch_if_no_active_lanes_eq => b"branch_if_no_active_lanes_eq",
                    ProgramOp::jump => b"jump",
                    ProgramOp::bitwise_and_imm_4_ints => b"bitwise_and_imm_4_ints",
                    ProgramOp::bitwise_and_imm_3_ints => b"bitwise_and_imm_3_ints",
                    ProgramOp::bitwise_and_imm_2_ints => b"bitwise_and_imm_2_ints",
                    ProgramOp::bitwise_and_imm_int => b"bitwise_and_imm_int",
                    ProgramOp::bitwise_and_n_ints => b"bitwise_and_n_ints",
                    ProgramOp::bitwise_and_int => b"bitwise_and_int",
                    ProgramOp::bitwise_and_2_ints => b"bitwise_and_2_ints",
                    ProgramOp::bitwise_and_3_ints => b"bitwise_and_3_ints",
                    ProgramOp::bitwise_and_4_ints => b"bitwise_and_4_ints",
                    ProgramOp::bitwise_or_n_ints => b"bitwise_or_n_ints",
                    ProgramOp::bitwise_or_int => b"bitwise_or_int",
                    ProgramOp::bitwise_or_2_ints => b"bitwise_or_2_ints",
                    ProgramOp::bitwise_or_3_ints => b"bitwise_or_3_ints",
                    ProgramOp::bitwise_or_4_ints => b"bitwise_or_4_ints",
                    ProgramOp::bitwise_xor_imm_int => b"bitwise_xor_imm_int",
                    ProgramOp::bitwise_xor_n_ints => b"bitwise_xor_n_ints",
                    ProgramOp::bitwise_xor_int => b"bitwise_xor_int",
                    ProgramOp::bitwise_xor_2_ints => b"bitwise_xor_2_ints",
                    ProgramOp::bitwise_xor_3_ints => b"bitwise_xor_3_ints",
                    ProgramOp::bitwise_xor_4_ints => b"bitwise_xor_4_ints",
                    ProgramOp::cast_to_float_from_int => b"cast_to_float_from_int",
                    ProgramOp::cast_to_float_from_2_ints => b"cast_to_float_from_2_ints",
                    ProgramOp::cast_to_float_from_3_ints => b"cast_to_float_from_3_ints",
                    ProgramOp::cast_to_float_from_4_ints => b"cast_to_float_from_4_ints",
                    ProgramOp::cast_to_float_from_uint => b"cast_to_float_from_uint",
                    ProgramOp::cast_to_float_from_2_uints => b"cast_to_float_from_2_uints",
                    ProgramOp::cast_to_float_from_3_uints => b"cast_to_float_from_3_uints",
                    ProgramOp::cast_to_float_from_4_uints => b"cast_to_float_from_4_uints",
                    ProgramOp::cast_to_int_from_float => b"cast_to_int_from_float",
                    ProgramOp::cast_to_int_from_2_floats => b"cast_to_int_from_2_floats",
                    ProgramOp::cast_to_int_from_3_floats => b"cast_to_int_from_3_floats",
                    ProgramOp::cast_to_int_from_4_floats => b"cast_to_int_from_4_floats",
                    ProgramOp::cast_to_uint_from_float => b"cast_to_uint_from_float",
                    ProgramOp::cast_to_uint_from_2_floats => b"cast_to_uint_from_2_floats",
                    ProgramOp::cast_to_uint_from_3_floats => b"cast_to_uint_from_3_floats",
                    ProgramOp::cast_to_uint_from_4_floats => b"cast_to_uint_from_4_floats",
                    ProgramOp::abs_int => b"abs_int",
                    ProgramOp::abs_2_ints => b"abs_2_ints",
                    ProgramOp::abs_3_ints => b"abs_3_ints",
                    ProgramOp::abs_4_ints => b"abs_4_ints",
                    ProgramOp::floor_float => b"floor_float",
                    ProgramOp::floor_2_floats => b"floor_2_floats",
                    ProgramOp::floor_3_floats => b"floor_3_floats",
                    ProgramOp::floor_4_floats => b"floor_4_floats",
                    ProgramOp::ceil_float => b"ceil_float",
                    ProgramOp::ceil_2_floats => b"ceil_2_floats",
                    ProgramOp::ceil_3_floats => b"ceil_3_floats",
                    ProgramOp::ceil_4_floats => b"ceil_4_floats",
                    ProgramOp::invsqrt_float => b"invsqrt_float",
                    ProgramOp::invsqrt_2_floats => b"invsqrt_2_floats",
                    ProgramOp::invsqrt_3_floats => b"invsqrt_3_floats",
                    ProgramOp::invsqrt_4_floats => b"invsqrt_4_floats",
                    ProgramOp::inverse_mat2 => b"inverse_mat2",
                    ProgramOp::inverse_mat3 => b"inverse_mat3",
                    ProgramOp::inverse_mat4 => b"inverse_mat4",
                    ProgramOp::sin_float => b"sin_float",
                    ProgramOp::cos_float => b"cos_float",
                    ProgramOp::tan_float => b"tan_float",
                    ProgramOp::asin_float => b"asin_float",
                    ProgramOp::acos_float => b"acos_float",
                    ProgramOp::atan_float => b"atan_float",
                    ProgramOp::atan2_n_floats => b"atan2_n_floats",
                    ProgramOp::sqrt_float => b"sqrt_float",
                    ProgramOp::pow_n_floats => b"pow_n_floats",
                    ProgramOp::exp_float => b"exp_float",
                    ProgramOp::exp2_float => b"exp2_float",
                    ProgramOp::log_float => b"log_float",
                    ProgramOp::log2_float => b"log2_float",
                    ProgramOp::refract_4_floats => b"refract_4_floats",
                    ProgramOp::copy_uniform => b"copy_uniform",
                    ProgramOp::copy_2_uniforms => b"copy_2_uniforms",
                    ProgramOp::copy_3_uniforms => b"copy_3_uniforms",
                    ProgramOp::copy_4_uniforms => b"copy_4_uniforms",
                    ProgramOp::copy_constant => b"copy_constant",
                    ProgramOp::splat_2_constants => b"splat_2_constants",
                    ProgramOp::splat_3_constants => b"splat_3_constants",
                    ProgramOp::splat_4_constants => b"splat_4_constants",
                    ProgramOp::copy_slot_masked => b"copy_slot_masked",
                    ProgramOp::copy_2_slots_masked => b"copy_2_slots_masked",
                    ProgramOp::copy_3_slots_masked => b"copy_3_slots_masked",
                    ProgramOp::copy_4_slots_masked => b"copy_4_slots_masked",
                    ProgramOp::copy_from_indirect_unmasked => b"copy_from_indirect_unmasked",
                    ProgramOp::copy_from_indirect_uniform_unmasked => {
                        b"copy_from_indirect_uniform_unmasked"
                    }
                    ProgramOp::copy_to_indirect_masked => b"copy_to_indirect_masked",
                    ProgramOp::swizzle_copy_to_indirect_masked => {
                        b"swizzle_copy_to_indirect_masked"
                    }
                    ProgramOp::copy_slot_unmasked => b"copy_slot_unmasked",
                    ProgramOp::copy_2_slots_unmasked => b"copy_2_slots_unmasked",
                    ProgramOp::copy_3_slots_unmasked => b"copy_3_slots_unmasked",
                    ProgramOp::copy_4_slots_unmasked => b"copy_4_slots_unmasked",
                    ProgramOp::copy_immutable_unmasked => b"copy_immutable_unmasked",
                    ProgramOp::copy_2_immutables_unmasked => b"copy_2_immutables_unmasked",
                    ProgramOp::copy_3_immutables_unmasked => b"copy_3_immutables_unmasked",
                    ProgramOp::copy_4_immutables_unmasked => b"copy_4_immutables_unmasked",
                    ProgramOp::swizzle_copy_slot_masked => b"swizzle_copy_slot_masked",
                    ProgramOp::swizzle_copy_2_slots_masked => b"swizzle_copy_2_slots_masked",
                    ProgramOp::swizzle_copy_3_slots_masked => b"swizzle_copy_3_slots_masked",
                    ProgramOp::swizzle_copy_4_slots_masked => b"swizzle_copy_4_slots_masked",
                    ProgramOp::swizzle_1 => b"swizzle_1",
                    ProgramOp::swizzle_2 => b"swizzle_2",
                    ProgramOp::swizzle_3 => b"swizzle_3",
                    ProgramOp::swizzle_4 => b"swizzle_4",
                    ProgramOp::shuffle => b"shuffle",
                    ProgramOp::matrix_multiply_2 => b"matrix_multiply_2",
                    ProgramOp::matrix_multiply_3 => b"matrix_multiply_3",
                    ProgramOp::matrix_multiply_4 => b"matrix_multiply_4",
                    ProgramOp::smoothstep_n_floats => b"smoothstep_n_floats",
                    ProgramOp::dot_2_floats => b"dot_2_floats",
                    ProgramOp::dot_3_floats => b"dot_3_floats",
                    ProgramOp::dot_4_floats => b"dot_4_floats",
                    ProgramOp::add_imm_float => b"add_imm_float",
                    ProgramOp::add_n_floats => b"add_n_floats",
                    ProgramOp::add_float => b"add_float",
                    ProgramOp::add_2_floats => b"add_2_floats",
                    ProgramOp::add_3_floats => b"add_3_floats",
                    ProgramOp::add_4_floats => b"add_4_floats",
                    ProgramOp::add_imm_int => b"add_imm_int",
                    ProgramOp::add_n_ints => b"add_n_ints",
                    ProgramOp::add_int => b"add_int",
                    ProgramOp::add_2_ints => b"add_2_ints",
                    ProgramOp::add_3_ints => b"add_3_ints",
                    ProgramOp::add_4_ints => b"add_4_ints",
                    ProgramOp::sub_n_floats => b"sub_n_floats",
                    ProgramOp::sub_float => b"sub_float",
                    ProgramOp::sub_2_floats => b"sub_2_floats",
                    ProgramOp::sub_3_floats => b"sub_3_floats",
                    ProgramOp::sub_4_floats => b"sub_4_floats",
                    ProgramOp::sub_n_ints => b"sub_n_ints",
                    ProgramOp::sub_int => b"sub_int",
                    ProgramOp::sub_2_ints => b"sub_2_ints",
                    ProgramOp::sub_3_ints => b"sub_3_ints",
                    ProgramOp::sub_4_ints => b"sub_4_ints",
                    ProgramOp::mul_imm_float => b"mul_imm_float",
                    ProgramOp::mul_n_floats => b"mul_n_floats",
                    ProgramOp::mul_float => b"mul_float",
                    ProgramOp::mul_2_floats => b"mul_2_floats",
                    ProgramOp::mul_3_floats => b"mul_3_floats",
                    ProgramOp::mul_4_floats => b"mul_4_floats",
                    ProgramOp::mul_imm_int => b"mul_imm_int",
                    ProgramOp::mul_n_ints => b"mul_n_ints",
                    ProgramOp::mul_int => b"mul_int",
                    ProgramOp::mul_2_ints => b"mul_2_ints",
                    ProgramOp::mul_3_ints => b"mul_3_ints",
                    ProgramOp::mul_4_ints => b"mul_4_ints",
                    ProgramOp::div_n_floats => b"div_n_floats",
                    ProgramOp::div_float => b"div_float",
                    ProgramOp::div_2_floats => b"div_2_floats",
                    ProgramOp::div_3_floats => b"div_3_floats",
                    ProgramOp::div_4_floats => b"div_4_floats",
                    ProgramOp::div_n_ints => b"div_n_ints",
                    ProgramOp::div_int => b"div_int",
                    ProgramOp::div_2_ints => b"div_2_ints",
                    ProgramOp::div_3_ints => b"div_3_ints",
                    ProgramOp::div_4_ints => b"div_4_ints",
                    ProgramOp::div_n_uints => b"div_n_uints",
                    ProgramOp::div_uint => b"div_uint",
                    ProgramOp::div_2_uints => b"div_2_uints",
                    ProgramOp::div_3_uints => b"div_3_uints",
                    ProgramOp::div_4_uints => b"div_4_uints",
                    ProgramOp::max_imm_float => b"max_imm_float",
                    ProgramOp::max_n_floats => b"max_n_floats",
                    ProgramOp::max_float => b"max_float",
                    ProgramOp::max_2_floats => b"max_2_floats",
                    ProgramOp::max_3_floats => b"max_3_floats",
                    ProgramOp::max_4_floats => b"max_4_floats",
                    ProgramOp::max_n_ints => b"max_n_ints",
                    ProgramOp::max_int => b"max_int",
                    ProgramOp::max_2_ints => b"max_2_ints",
                    ProgramOp::max_3_ints => b"max_3_ints",
                    ProgramOp::max_4_ints => b"max_4_ints",
                    ProgramOp::max_n_uints => b"max_n_uints",
                    ProgramOp::max_uint => b"max_uint",
                    ProgramOp::max_2_uints => b"max_2_uints",
                    ProgramOp::max_3_uints => b"max_3_uints",
                    ProgramOp::max_4_uints => b"max_4_uints",
                    ProgramOp::min_imm_float => b"min_imm_float",
                    ProgramOp::min_n_floats => b"min_n_floats",
                    ProgramOp::min_float => b"min_float",
                    ProgramOp::min_2_floats => b"min_2_floats",
                    ProgramOp::min_3_floats => b"min_3_floats",
                    ProgramOp::min_4_floats => b"min_4_floats",
                    ProgramOp::min_n_ints => b"min_n_ints",
                    ProgramOp::min_int => b"min_int",
                    ProgramOp::min_2_ints => b"min_2_ints",
                    ProgramOp::min_3_ints => b"min_3_ints",
                    ProgramOp::min_4_ints => b"min_4_ints",
                    ProgramOp::min_n_uints => b"min_n_uints",
                    ProgramOp::min_uint => b"min_uint",
                    ProgramOp::min_2_uints => b"min_2_uints",
                    ProgramOp::min_3_uints => b"min_3_uints",
                    ProgramOp::min_4_uints => b"min_4_uints",
                    ProgramOp::mod_n_floats => b"mod_n_floats",
                    ProgramOp::mod_float => b"mod_float",
                    ProgramOp::mod_2_floats => b"mod_2_floats",
                    ProgramOp::mod_3_floats => b"mod_3_floats",
                    ProgramOp::mod_4_floats => b"mod_4_floats",
                    ProgramOp::mix_n_floats => b"mix_n_floats",
                    ProgramOp::mix_float => b"mix_float",
                    ProgramOp::mix_2_floats => b"mix_2_floats",
                    ProgramOp::mix_3_floats => b"mix_3_floats",
                    ProgramOp::mix_4_floats => b"mix_4_floats",
                    ProgramOp::mix_n_ints => b"mix_n_ints",
                    ProgramOp::mix_int => b"mix_int",
                    ProgramOp::mix_2_ints => b"mix_2_ints",
                    ProgramOp::mix_3_ints => b"mix_3_ints",
                    ProgramOp::mix_4_ints => b"mix_4_ints",
                    ProgramOp::cmplt_imm_float => b"cmplt_imm_float",
                    ProgramOp::cmplt_n_floats => b"cmplt_n_floats",
                    ProgramOp::cmplt_float => b"cmplt_float",
                    ProgramOp::cmplt_2_floats => b"cmplt_2_floats",
                    ProgramOp::cmplt_3_floats => b"cmplt_3_floats",
                    ProgramOp::cmplt_4_floats => b"cmplt_4_floats",
                    ProgramOp::cmplt_imm_int => b"cmplt_imm_int",
                    ProgramOp::cmplt_n_ints => b"cmplt_n_ints",
                    ProgramOp::cmplt_int => b"cmplt_int",
                    ProgramOp::cmplt_2_ints => b"cmplt_2_ints",
                    ProgramOp::cmplt_3_ints => b"cmplt_3_ints",
                    ProgramOp::cmplt_4_ints => b"cmplt_4_ints",
                    ProgramOp::cmplt_imm_uint => b"cmplt_imm_uint",
                    ProgramOp::cmplt_n_uints => b"cmplt_n_uints",
                    ProgramOp::cmplt_uint => b"cmplt_uint",
                    ProgramOp::cmplt_2_uints => b"cmplt_2_uints",
                    ProgramOp::cmplt_3_uints => b"cmplt_3_uints",
                    ProgramOp::cmplt_4_uints => b"cmplt_4_uints",
                    ProgramOp::cmple_imm_float => b"cmple_imm_float",
                    ProgramOp::cmple_n_floats => b"cmple_n_floats",
                    ProgramOp::cmple_float => b"cmple_float",
                    ProgramOp::cmple_2_floats => b"cmple_2_floats",
                    ProgramOp::cmple_3_floats => b"cmple_3_floats",
                    ProgramOp::cmple_4_floats => b"cmple_4_floats",
                    ProgramOp::cmple_imm_int => b"cmple_imm_int",
                    ProgramOp::cmple_n_ints => b"cmple_n_ints",
                    ProgramOp::cmple_int => b"cmple_int",
                    ProgramOp::cmple_2_ints => b"cmple_2_ints",
                    ProgramOp::cmple_3_ints => b"cmple_3_ints",
                    ProgramOp::cmple_4_ints => b"cmple_4_ints",
                    ProgramOp::cmple_imm_uint => b"cmple_imm_uint",
                    ProgramOp::cmple_n_uints => b"cmple_n_uints",
                    ProgramOp::cmple_uint => b"cmple_uint",
                    ProgramOp::cmple_2_uints => b"cmple_2_uints",
                    ProgramOp::cmple_3_uints => b"cmple_3_uints",
                    ProgramOp::cmple_4_uints => b"cmple_4_uints",
                    ProgramOp::cmpeq_imm_float => b"cmpeq_imm_float",
                    ProgramOp::cmpeq_n_floats => b"cmpeq_n_floats",
                    ProgramOp::cmpeq_float => b"cmpeq_float",
                    ProgramOp::cmpeq_2_floats => b"cmpeq_2_floats",
                    ProgramOp::cmpeq_3_floats => b"cmpeq_3_floats",
                    ProgramOp::cmpeq_4_floats => b"cmpeq_4_floats",
                    ProgramOp::cmpeq_imm_int => b"cmpeq_imm_int",
                    ProgramOp::cmpeq_n_ints => b"cmpeq_n_ints",
                    ProgramOp::cmpeq_int => b"cmpeq_int",
                    ProgramOp::cmpeq_2_ints => b"cmpeq_2_ints",
                    ProgramOp::cmpeq_3_ints => b"cmpeq_3_ints",
                    ProgramOp::cmpeq_4_ints => b"cmpeq_4_ints",
                    ProgramOp::cmpne_imm_float => b"cmpne_imm_float",
                    ProgramOp::cmpne_n_floats => b"cmpne_n_floats",
                    ProgramOp::cmpne_float => b"cmpne_float",
                    ProgramOp::cmpne_2_floats => b"cmpne_2_floats",
                    ProgramOp::cmpne_3_floats => b"cmpne_3_floats",
                    ProgramOp::cmpne_4_floats => b"cmpne_4_floats",
                    ProgramOp::cmpne_imm_int => b"cmpne_imm_int",
                    ProgramOp::cmpne_n_ints => b"cmpne_n_ints",
                    ProgramOp::cmpne_int => b"cmpne_int",
                    ProgramOp::cmpne_2_ints => b"cmpne_2_ints",
                    ProgramOp::cmpne_3_ints => b"cmpne_3_ints",
                    ProgramOp::cmpne_4_ints => b"cmpne_4_ints",
                    ProgramOp::trace_line => b"trace_line",
                    ProgramOp::trace_var => b"trace_var",
                    ProgramOp::trace_enter => b"trace_enter",
                    ProgramOp::trace_exit => b"trace_exit",
                    ProgramOp::trace_scope => b"trace_scope",
                    ProgramOp::callback => b"callback",
                    ProgramOp::stack_checkpoint => b"stack_checkpoint",
                    ProgramOp::stack_rewind => b"stack_rewind",
                    ProgramOp::unbounded_set_rgb => b"unbounded_set_rgb",
                    ProgramOp::unbounded_uniform_color => b"unbounded_uniform_color",
                    ProgramOp::unpremul => b"unpremul",
                    ProgramOp::unpremul_polar => b"unpremul_polar",
                    ProgramOp::dither => b"dither",
                    ProgramOp::load_16161616 => b"load_16161616",
                    ProgramOp::load_16161616_dst => b"load_16161616_dst",
                    ProgramOp::store_16161616 => b"store_16161616",
                    ProgramOp::gather_16161616 => b"gather_16161616",
                    ProgramOp::load_a16 => b"load_a16",
                    ProgramOp::load_a16_dst => b"load_a16_dst",
                    ProgramOp::store_a16 => b"store_a16",
                    ProgramOp::gather_a16 => b"gather_a16",
                    ProgramOp::load_r16 => b"load_r16",
                    ProgramOp::load_r16_dst => b"load_r16_dst",
                    ProgramOp::store_r16 => b"store_r16",
                    ProgramOp::gather_r16 => b"gather_r16",
                    ProgramOp::load_rg1616 => b"load_rg1616",
                    ProgramOp::load_rg1616_dst => b"load_rg1616_dst",
                    ProgramOp::store_rg1616 => b"store_rg1616",
                    ProgramOp::gather_rg1616 => b"gather_rg1616",
                    ProgramOp::load_f16 => b"load_f16",
                    ProgramOp::load_f16_dst => b"load_f16_dst",
                    ProgramOp::store_f16 => b"store_f16",
                    ProgramOp::gather_f16 => b"gather_f16",
                    ProgramOp::load_af16 => b"load_af16",
                    ProgramOp::load_af16_dst => b"load_af16_dst",
                    ProgramOp::store_af16 => b"store_af16",
                    ProgramOp::gather_af16 => b"gather_af16",
                    ProgramOp::load_rf16 => b"load_rf16",
                    ProgramOp::load_rf16_dst => b"load_rf16_dst",
                    ProgramOp::store_rf16 => b"store_rf16",
                    ProgramOp::gather_rf16 => b"gather_rf16",
                    ProgramOp::load_rgf16 => b"load_rgf16",
                    ProgramOp::load_rgf16_dst => b"load_rgf16_dst",
                    ProgramOp::store_rgf16 => b"store_rgf16",
                    ProgramOp::gather_rgf16 => b"gather_rgf16",
                    ProgramOp::load_f32 => b"load_f32",
                    ProgramOp::load_f32_dst => b"load_f32_dst",
                    ProgramOp::store_f32 => b"store_f32",
                    ProgramOp::gather_f32 => b"gather_f32",
                    ProgramOp::load_1010102 => b"load_1010102",
                    ProgramOp::load_1010102_dst => b"load_1010102_dst",
                    ProgramOp::store_1010102 => b"store_1010102",
                    ProgramOp::gather_1010102 => b"gather_1010102",
                    ProgramOp::load_1010102_xr => b"load_1010102_xr",
                    ProgramOp::load_1010102_xr_dst => b"load_1010102_xr_dst",
                    ProgramOp::store_1010102_xr => b"store_1010102_xr",
                    ProgramOp::gather_1010102_xr => b"gather_1010102_xr",
                    ProgramOp::load_10x6 => b"load_10x6",
                    ProgramOp::load_10x6_dst => b"load_10x6_dst",
                    ProgramOp::store_10x6 => b"store_10x6",
                    ProgramOp::gather_10x6 => b"gather_10x6",
                    ProgramOp::gather_10101010_xr => b"gather_10101010_xr",
                    ProgramOp::load_10101010_xr => b"load_10101010_xr",
                    ProgramOp::load_10101010_xr_dst => b"load_10101010_xr_dst",
                    ProgramOp::store_10101010_xr => b"store_10101010_xr",
                    ProgramOp::store_src_rg => b"store_src_rg",
                    ProgramOp::load_src_rg => b"load_src_rg",
                    ProgramOp::byte_tables => b"byte_tables",
                    ProgramOp::colorburn => b"colorburn",
                    ProgramOp::colordodge => b"colordodge",
                    ProgramOp::softlight => b"softlight",
                    ProgramOp::hue => b"hue",
                    ProgramOp::saturation => b"saturation",
                    ProgramOp::color => b"color",
                    ProgramOp::luminosity => b"luminosity",
                    ProgramOp::matrix_3x3 => b"matrix_3x3",
                    ProgramOp::matrix_3x4 => b"matrix_3x4",
                    ProgramOp::matrix_4x5 => b"matrix_4x5",
                    ProgramOp::matrix_4x3 => b"matrix_4x3",
                    ProgramOp::parametric => b"parametric",
                    ProgramOp::gamma_ => b"gamma_",
                    ProgramOp::PQish => b"PQish",
                    ProgramOp::HLGish => b"HLGish",
                    ProgramOp::HLGinvish => b"HLGinvish",
                    ProgramOp::ootf => b"ootf",
                    ProgramOp::rgb_to_hsl => b"rgb_to_hsl",
                    ProgramOp::hsl_to_rgb => b"hsl_to_rgb",
                    ProgramOp::css_lab_to_xyz => b"css_lab_to_xyz",
                    ProgramOp::css_oklab_to_linear_srgb => b"css_oklab_to_linear_srgb",
                    ProgramOp::css_oklab_gamut_map_to_linear_srgb => {
                        b"css_oklab_gamut_map_to_linear_srgb"
                    }
                    ProgramOp::css_hcl_to_lab => b"css_hcl_to_lab",
                    ProgramOp::css_hsl_to_srgb => b"css_hsl_to_srgb",
                    ProgramOp::css_hwb_to_srgb => b"css_hwb_to_srgb",
                    ProgramOp::gauss_a_to_rgba => b"gauss_a_to_rgba",
                    ProgramOp::mirror_x => b"mirror_x",
                    ProgramOp::repeat_x => b"repeat_x",
                    ProgramOp::mirror_y => b"mirror_y",
                    ProgramOp::repeat_y => b"repeat_y",
                    ProgramOp::negate_x => b"negate_x",
                    ProgramOp::bilerp_clamp_8888_force_highp => b"bilerp_clamp_8888_force_highp",
                    ProgramOp::bicubic_clamp_8888 => b"bicubic_clamp_8888",
                    ProgramOp::bilinear_setup => b"bilinear_setup",
                    ProgramOp::bilinear_nx => b"bilinear_nx",
                    ProgramOp::bilinear_px => b"bilinear_px",
                    ProgramOp::bilinear_ny => b"bilinear_ny",
                    ProgramOp::bilinear_py => b"bilinear_py",
                    ProgramOp::bicubic_setup => b"bicubic_setup",
                    ProgramOp::bicubic_n3x => b"bicubic_n3x",
                    ProgramOp::bicubic_n1x => b"bicubic_n1x",
                    ProgramOp::bicubic_p1x => b"bicubic_p1x",
                    ProgramOp::bicubic_p3x => b"bicubic_p3x",
                    ProgramOp::bicubic_n3y => b"bicubic_n3y",
                    ProgramOp::bicubic_n1y => b"bicubic_n1y",
                    ProgramOp::bicubic_p1y => b"bicubic_p1y",
                    ProgramOp::bicubic_p3y => b"bicubic_p3y",
                    ProgramOp::accumulate => b"accumulate",
                    ProgramOp::perlin_noise => b"perlin_noise",
                    ProgramOp::mipmap_linear_init => b"mipmap_linear_init",
                    ProgramOp::mipmap_linear_update => b"mipmap_linear_update",
                    ProgramOp::mipmap_linear_finish => b"mipmap_linear_finish",
                    ProgramOp::xy_to_2pt_conical_strip => b"xy_to_2pt_conical_strip",
                    ProgramOp::xy_to_2pt_conical_focal_on_circle => {
                        b"xy_to_2pt_conical_focal_on_circle"
                    }
                    ProgramOp::xy_to_2pt_conical_well_behaved => b"xy_to_2pt_conical_well_behaved",
                    ProgramOp::xy_to_2pt_conical_smaller => b"xy_to_2pt_conical_smaller",
                    ProgramOp::xy_to_2pt_conical_greater => b"xy_to_2pt_conical_greater",
                    ProgramOp::alter_2pt_conical_compensate_focal => {
                        b"alter_2pt_conical_compensate_focal"
                    }
                    ProgramOp::alter_2pt_conical_unswap => b"alter_2pt_conical_unswap",
                    ProgramOp::mask_2pt_conical_nan => b"mask_2pt_conical_nan",
                    ProgramOp::mask_2pt_conical_degenerates => b"mask_2pt_conical_degenerates",
                    ProgramOp::apply_vector_mask => b"apply_vector_mask",
                    ProgramOp::set_base_pointer => b"set_base_pointer",
                    ProgramOp::label => b"label",
                    ProgramOp::invoke_shader => b"invoke_shader",
                    ProgramOp::invoke_color_filter => b"invoke_color_filter",
                    ProgramOp::invoke_blender => b"invoke_blender",
                    ProgramOp::invoke_to_linear_srgb => b"invoke_to_linear_srgb",
                    ProgramOp::invoke_from_linear_srgb => b"invoke_from_linear_srgb",
                }
            }
            let name = dump_op_name(op);
            let text = match op {
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3385-3387
                ProgramOp::trace_var => {
                    dump_concat(&[b"TraceVar(", &a2, &a3, b") when ", &a1, b" is true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3389-3391
                ProgramOp::trace_line => {
                    dump_concat(&[b"TraceLine(", &a2, b") when ", &a1, b" is true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3393-3395
                ProgramOp::trace_enter => {
                    dump_concat(&[b"TraceEnter(", &a2, b") when ", &a1, b" is true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3397-3399
                ProgramOp::trace_exit => {
                    dump_concat(&[b"TraceExit(", &a2, b") when ", &a1, b" is true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3401-3403
                ProgramOp::trace_scope => {
                    dump_concat(&[b"TraceScope(", &a2, b") when ", &a1, b" is true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3405-3407
                ProgramOp::init_lane_masks => {
                    dump_concat(&[b"CondMask = LoopMask = RetMask = true"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3409-3411
                ProgramOp::load_condition_mask => dump_concat(&[b"CondMask = ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3413-3415
                ProgramOp::store_condition_mask => dump_concat(&[&a1, b" = CondMask"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3417-3419
                ProgramOp::merge_condition_mask => dump_concat(&[b"CondMask = ", &a1, b" & ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3421-3423
                ProgramOp::merge_inv_condition_mask => {
                    dump_concat(&[b"CondMask = ", &a1, b" & ~", &a2])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3425-3427
                ProgramOp::load_loop_mask => dump_concat(&[b"LoopMask = ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3429-3431
                ProgramOp::store_loop_mask => dump_concat(&[&a1, b" = LoopMask"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3433-3435
                ProgramOp::mask_off_loop_mask => {
                    dump_concat(&[b"LoopMask &= ~(CondMask & LoopMask & RetMask)"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3437-3439
                ProgramOp::reenable_loop_mask => dump_concat(&[b"LoopMask |= ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3441-3443
                ProgramOp::merge_loop_mask => dump_concat(&[b"LoopMask &= ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3445-3447
                ProgramOp::load_return_mask => dump_concat(&[b"RetMask = ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3449-3451
                ProgramOp::store_return_mask => dump_concat(&[&a1, b" = RetMask"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3453-3455
                ProgramOp::mask_off_return_mask => {
                    dump_concat(&[b"RetMask &= ~(CondMask & LoopMask & RetMask)"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3457-3459
                ProgramOp::store_src_rg => dump_concat(&[&a1, b" = src.rg"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3461-3463
                ProgramOp::exchange_src => dump_concat(&[b"swap(src.rgba, ", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3465-3467
                ProgramOp::store_src => dump_concat(&[&a1, b" = src.rgba"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3469-3471
                ProgramOp::store_dst => dump_concat(&[&a1, b" = dst.rgba"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3473-3475
                ProgramOp::store_device_xy01 => dump_concat(&[&a1, b" = DeviceCoords.xy01"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3477-3479
                ProgramOp::load_src => dump_concat(&[b"src.rgba = ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3481-3483
                ProgramOp::load_dst => dump_concat(&[b"dst.rgba = ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3485-3495
                ProgramOp::bitwise_and_int
                | ProgramOp::bitwise_and_2_ints
                | ProgramOp::bitwise_and_3_ints
                | ProgramOp::bitwise_and_4_ints
                | ProgramOp::bitwise_and_n_ints
                | ProgramOp::bitwise_and_imm_int
                | ProgramOp::bitwise_and_imm_2_ints
                | ProgramOp::bitwise_and_imm_3_ints
                | ProgramOp::bitwise_and_imm_4_ints => dump_concat(&[&a1, b" &= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3497-3503
                ProgramOp::bitwise_or_int
                | ProgramOp::bitwise_or_2_ints
                | ProgramOp::bitwise_or_3_ints
                | ProgramOp::bitwise_or_4_ints
                | ProgramOp::bitwise_or_n_ints => dump_concat(&[&a1, b" |= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3505-3512
                ProgramOp::bitwise_xor_int
                | ProgramOp::bitwise_xor_2_ints
                | ProgramOp::bitwise_xor_3_ints
                | ProgramOp::bitwise_xor_4_ints
                | ProgramOp::bitwise_xor_n_ints
                | ProgramOp::bitwise_xor_imm_int => dump_concat(&[&a1, b" ^= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3514-3519
                ProgramOp::cast_to_float_from_int
                | ProgramOp::cast_to_float_from_2_ints
                | ProgramOp::cast_to_float_from_3_ints
                | ProgramOp::cast_to_float_from_4_ints => {
                    dump_concat(&[&a1, b" = IntToFloat(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3521-3526
                ProgramOp::cast_to_float_from_uint
                | ProgramOp::cast_to_float_from_2_uints
                | ProgramOp::cast_to_float_from_3_uints
                | ProgramOp::cast_to_float_from_4_uints => {
                    dump_concat(&[&a1, b" = UintToFloat(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3528-3533
                ProgramOp::cast_to_int_from_float
                | ProgramOp::cast_to_int_from_2_floats
                | ProgramOp::cast_to_int_from_3_floats
                | ProgramOp::cast_to_int_from_4_floats => {
                    dump_concat(&[&a1, b" = FloatToInt(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3535-3540
                ProgramOp::cast_to_uint_from_float
                | ProgramOp::cast_to_uint_from_2_floats
                | ProgramOp::cast_to_uint_from_3_floats
                | ProgramOp::cast_to_uint_from_4_floats => {
                    dump_concat(&[&a1, b" = FloatToUint(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3542-3547
                ProgramOp::copy_slot_masked
                | ProgramOp::copy_2_slots_masked
                | ProgramOp::copy_3_slots_masked
                | ProgramOp::copy_4_slots_masked
                | ProgramOp::swizzle_copy_slot_masked
                | ProgramOp::swizzle_copy_2_slots_masked
                | ProgramOp::swizzle_copy_3_slots_masked
                | ProgramOp::swizzle_copy_4_slots_masked => {
                    dump_concat(&[&a1, b" = Mask(", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3549-3561
                ProgramOp::copy_uniform
                | ProgramOp::copy_2_uniforms
                | ProgramOp::copy_3_uniforms
                | ProgramOp::copy_4_uniforms
                | ProgramOp::copy_slot_unmasked
                | ProgramOp::copy_2_slots_unmasked
                | ProgramOp::copy_3_slots_unmasked
                | ProgramOp::copy_4_slots_unmasked
                | ProgramOp::copy_immutable_unmasked
                | ProgramOp::copy_2_immutables_unmasked
                | ProgramOp::copy_3_immutables_unmasked
                | ProgramOp::copy_4_immutables_unmasked
                | ProgramOp::copy_constant
                | ProgramOp::splat_2_constants
                | ProgramOp::splat_3_constants
                | ProgramOp::splat_4_constants
                | ProgramOp::swizzle_1
                | ProgramOp::swizzle_2
                | ProgramOp::swizzle_3
                | ProgramOp::swizzle_4
                | ProgramOp::shuffle => dump_concat(&[&a1, b" = ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3563-3566
                ProgramOp::copy_from_indirect_unmasked
                | ProgramOp::copy_from_indirect_uniform_unmasked => {
                    dump_concat(&[&a1, b" = Indirect(", &a2, b" + ", &a3, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3568-3570
                ProgramOp::copy_to_indirect_masked => {
                    dump_concat(&[b"Indirect(", &a1, b" + ", &a3, b") = Mask(", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3572-3575
                ProgramOp::swizzle_copy_to_indirect_masked => dump_concat(&[
                    b"Indirect(",
                    &a1,
                    b" + ",
                    &a3,
                    b").",
                    &swizzle,
                    b" = Mask(",
                    &a2,
                    b")",
                ]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3577-3582
                ProgramOp::abs_int
                | ProgramOp::abs_2_ints
                | ProgramOp::abs_3_ints
                | ProgramOp::abs_4_ints => dump_concat(&[&a1, b" = abs(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3584-3586
                ProgramOp::acos_float => dump_concat(&[&a1, b" = acos(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3588-3590
                ProgramOp::asin_float => dump_concat(&[&a1, b" = asin(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3592-3594
                ProgramOp::atan_float => dump_concat(&[&a1, b" = atan(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3596-3598
                ProgramOp::atan2_n_floats => {
                    dump_concat(&[&a1, b" = atan2(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3600-3605
                ProgramOp::ceil_float
                | ProgramOp::ceil_2_floats
                | ProgramOp::ceil_3_floats
                | ProgramOp::ceil_4_floats => dump_concat(&[&a1, b" = ceil(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3607-3609
                ProgramOp::cos_float => dump_concat(&[&a1, b" = cos(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3611-3613
                ProgramOp::refract_4_floats => {
                    dump_concat(&[&a1, b" = refract(", &a1, b", ", &a2, b", ", &a3, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3615-3619
                ProgramOp::dot_2_floats | ProgramOp::dot_3_floats | ProgramOp::dot_4_floats => {
                    dump_concat(&[&a1, b" = dot(", &a2, b", ", &a3, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3621-3623
                ProgramOp::exp_float => dump_concat(&[&a1, b" = exp(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3625-3627
                ProgramOp::exp2_float => dump_concat(&[&a1, b" = exp2(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3629-3631
                ProgramOp::log_float => dump_concat(&[&a1, b" = log(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3633-3635
                ProgramOp::log2_float => dump_concat(&[&a1, b" = log2(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3637-3639
                ProgramOp::pow_n_floats => dump_concat(&[&a1, b" = pow(", &a1, b", ", &a2, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3641-3643
                ProgramOp::sin_float => dump_concat(&[&a1, b" = sin(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3645-3647
                ProgramOp::sqrt_float => dump_concat(&[&a1, b" = sqrt(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3649-3651
                ProgramOp::tan_float => dump_concat(&[&a1, b" = tan(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3653-3658
                ProgramOp::floor_float
                | ProgramOp::floor_2_floats
                | ProgramOp::floor_3_floats
                | ProgramOp::floor_4_floats => dump_concat(&[&a1, b" = floor(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3660-3665
                ProgramOp::invsqrt_float
                | ProgramOp::invsqrt_2_floats
                | ProgramOp::invsqrt_3_floats
                | ProgramOp::invsqrt_4_floats => dump_concat(&[&a1, b" = inversesqrt(", &a1, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3667-3671
                ProgramOp::inverse_mat2 | ProgramOp::inverse_mat3 | ProgramOp::inverse_mat4 => {
                    dump_concat(&[&a1, b" = inverse(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3673-3680
                ProgramOp::add_float
                | ProgramOp::add_int
                | ProgramOp::add_2_floats
                | ProgramOp::add_2_ints
                | ProgramOp::add_3_floats
                | ProgramOp::add_3_ints
                | ProgramOp::add_4_floats
                | ProgramOp::add_4_ints
                | ProgramOp::add_n_floats
                | ProgramOp::add_n_ints
                | ProgramOp::add_imm_float
                | ProgramOp::add_imm_int => dump_concat(&[&a1, b" += ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3682-3688
                ProgramOp::sub_float
                | ProgramOp::sub_int
                | ProgramOp::sub_2_floats
                | ProgramOp::sub_2_ints
                | ProgramOp::sub_3_floats
                | ProgramOp::sub_3_ints
                | ProgramOp::sub_4_floats
                | ProgramOp::sub_4_ints
                | ProgramOp::sub_n_floats
                | ProgramOp::sub_n_ints => dump_concat(&[&a1, b" -= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3690-3697
                ProgramOp::mul_float
                | ProgramOp::mul_int
                | ProgramOp::mul_2_floats
                | ProgramOp::mul_2_ints
                | ProgramOp::mul_3_floats
                | ProgramOp::mul_3_ints
                | ProgramOp::mul_4_floats
                | ProgramOp::mul_4_ints
                | ProgramOp::mul_n_floats
                | ProgramOp::mul_n_ints
                | ProgramOp::mul_imm_float
                | ProgramOp::mul_imm_int => dump_concat(&[&a1, b" *= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3699-3705
                ProgramOp::div_float
                | ProgramOp::div_int
                | ProgramOp::div_uint
                | ProgramOp::div_2_floats
                | ProgramOp::div_2_ints
                | ProgramOp::div_2_uints
                | ProgramOp::div_3_floats
                | ProgramOp::div_3_ints
                | ProgramOp::div_3_uints
                | ProgramOp::div_4_floats
                | ProgramOp::div_4_ints
                | ProgramOp::div_4_uints
                | ProgramOp::div_n_floats
                | ProgramOp::div_n_ints
                | ProgramOp::div_n_uints => dump_concat(&[&a1, b" /= ", &a2]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3707-3711
                ProgramOp::matrix_multiply_2
                | ProgramOp::matrix_multiply_3
                | ProgramOp::matrix_multiply_4 => dump_concat(&[&a1, b" = ", &a2, b" * ", &a3]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3713-3719
                ProgramOp::mod_float
                | ProgramOp::mod_2_floats
                | ProgramOp::mod_3_floats
                | ProgramOp::mod_4_floats
                | ProgramOp::mod_n_floats => dump_concat(&[&a1, b" = mod(", &a1, b", ", &a2, b")"]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3721-3728
                ProgramOp::min_float
                | ProgramOp::min_int
                | ProgramOp::min_uint
                | ProgramOp::min_2_floats
                | ProgramOp::min_2_ints
                | ProgramOp::min_2_uints
                | ProgramOp::min_3_floats
                | ProgramOp::min_3_ints
                | ProgramOp::min_3_uints
                | ProgramOp::min_4_floats
                | ProgramOp::min_4_ints
                | ProgramOp::min_4_uints
                | ProgramOp::min_n_floats
                | ProgramOp::min_n_ints
                | ProgramOp::min_n_uints
                | ProgramOp::min_imm_float => {
                    dump_concat(&[&a1, b" = min(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3730-3737
                ProgramOp::max_float
                | ProgramOp::max_int
                | ProgramOp::max_uint
                | ProgramOp::max_2_floats
                | ProgramOp::max_2_ints
                | ProgramOp::max_2_uints
                | ProgramOp::max_3_floats
                | ProgramOp::max_3_ints
                | ProgramOp::max_3_uints
                | ProgramOp::max_4_floats
                | ProgramOp::max_4_ints
                | ProgramOp::max_4_uints
                | ProgramOp::max_n_floats
                | ProgramOp::max_n_ints
                | ProgramOp::max_n_uints
                | ProgramOp::max_imm_float => {
                    dump_concat(&[&a1, b" = max(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3739-3746
                ProgramOp::cmplt_float
                | ProgramOp::cmplt_int
                | ProgramOp::cmplt_uint
                | ProgramOp::cmplt_2_floats
                | ProgramOp::cmplt_2_ints
                | ProgramOp::cmplt_2_uints
                | ProgramOp::cmplt_3_floats
                | ProgramOp::cmplt_3_ints
                | ProgramOp::cmplt_3_uints
                | ProgramOp::cmplt_4_floats
                | ProgramOp::cmplt_4_ints
                | ProgramOp::cmplt_4_uints
                | ProgramOp::cmplt_n_floats
                | ProgramOp::cmplt_n_ints
                | ProgramOp::cmplt_n_uints
                | ProgramOp::cmplt_imm_float
                | ProgramOp::cmplt_imm_int
                | ProgramOp::cmplt_imm_uint => {
                    dump_concat(&[&a1, b" = lessThan(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3748-3755
                ProgramOp::cmple_float
                | ProgramOp::cmple_int
                | ProgramOp::cmple_uint
                | ProgramOp::cmple_2_floats
                | ProgramOp::cmple_2_ints
                | ProgramOp::cmple_2_uints
                | ProgramOp::cmple_3_floats
                | ProgramOp::cmple_3_ints
                | ProgramOp::cmple_3_uints
                | ProgramOp::cmple_4_floats
                | ProgramOp::cmple_4_ints
                | ProgramOp::cmple_4_uints
                | ProgramOp::cmple_n_floats
                | ProgramOp::cmple_n_ints
                | ProgramOp::cmple_n_uints
                | ProgramOp::cmple_imm_float
                | ProgramOp::cmple_imm_int
                | ProgramOp::cmple_imm_uint => {
                    dump_concat(&[&a1, b" = lessThanEqual(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3757-3764
                ProgramOp::cmpeq_float
                | ProgramOp::cmpeq_int
                | ProgramOp::cmpeq_2_floats
                | ProgramOp::cmpeq_2_ints
                | ProgramOp::cmpeq_3_floats
                | ProgramOp::cmpeq_3_ints
                | ProgramOp::cmpeq_4_floats
                | ProgramOp::cmpeq_4_ints
                | ProgramOp::cmpeq_n_floats
                | ProgramOp::cmpeq_n_ints
                | ProgramOp::cmpeq_imm_float
                | ProgramOp::cmpeq_imm_int => {
                    dump_concat(&[&a1, b" = equal(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3766-3773
                ProgramOp::cmpne_float
                | ProgramOp::cmpne_int
                | ProgramOp::cmpne_2_floats
                | ProgramOp::cmpne_2_ints
                | ProgramOp::cmpne_3_floats
                | ProgramOp::cmpne_3_ints
                | ProgramOp::cmpne_4_floats
                | ProgramOp::cmpne_4_ints
                | ProgramOp::cmpne_n_floats
                | ProgramOp::cmpne_n_ints
                | ProgramOp::cmpne_imm_float
                | ProgramOp::cmpne_imm_int => {
                    dump_concat(&[&a1, b" = notEqual(", &a1, b", ", &a2, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3775-3781
                ProgramOp::mix_float
                | ProgramOp::mix_int
                | ProgramOp::mix_2_floats
                | ProgramOp::mix_2_ints
                | ProgramOp::mix_3_floats
                | ProgramOp::mix_3_ints
                | ProgramOp::mix_4_floats
                | ProgramOp::mix_4_ints
                | ProgramOp::mix_n_floats
                | ProgramOp::mix_n_ints => {
                    dump_concat(&[&a1, b" = mix(", &a2, b", ", &a3, b", ", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3783-3785
                ProgramOp::smoothstep_n_floats => {
                    dump_concat(&[&a1, b" = smoothstep(", &a1, b", ", &a2, b", ", &a3, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3787-3795
                ProgramOp::jump
                | ProgramOp::branch_if_all_lanes_active
                | ProgramOp::branch_if_any_lanes_active
                | ProgramOp::branch_if_no_lanes_active
                | ProgramOp::invoke_shader
                | ProgramOp::invoke_color_filter
                | ProgramOp::invoke_blender => dump_concat(&[b" ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3797-3799
                ProgramOp::invoke_to_linear_srgb => {
                    dump_concat(&[&a1, b" = toLinearSrgb(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3801-3803
                ProgramOp::invoke_from_linear_srgb => {
                    dump_concat(&[&a1, b" = fromLinearSrgb(", &a1, b")"])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3805-3807
                ProgramOp::branch_if_no_active_lanes_eq => {
                    dump_concat(&[b"branch ", &a1, b" if no lanes of ", &a2, b" == ", &a3])
                }
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3809-3811
                ProgramOp::label => dump_concat(&[b"label ", &a1]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3813-3816
                ProgramOp::case_op => dump_concat(&[
                    b"if (",
                    &a1,
                    b" == ",
                    &a3,
                    b") { LoopMask = true; ",
                    &a2,
                    b" = false; }",
                ]),
                // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3818-3821
                ProgramOp::continue_op => dump_concat(&[
                    &a1,
                    b" |= Mask(0xFFFFFFFF); LoopMask &= ~(CondMask & LoopMask & RetMask)",
                ]),
                _ => Vec::new(),
            };
            let name = dump_c_string(&name[..name.len().min(30)]);
            // Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3827-3835
            let name = &name[..name.len().min(30)];
            let line = if !text.is_empty() {
                let mut terminated = text;
                terminated.push(0);
                crate::src::sksl::SkSLString::printf(
                    c"%-30.*s %s\n".as_ptr(),
                    name.len() as i32,
                    name.as_ptr(),
                    terminated.as_ptr(),
                )
            } else {
                crate::src::sksl::SkSLString::printf(
                    c"%.*s\n".as_ptr(),
                    name.len() as i32,
                    name.as_ptr(),
                )
            };
            dump_write(out, &line);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.h:168-168
// Source: src/sksl/codegen/SkSLRasterPipelineBuilder.cpp:3839-3841
impl Program {
    /// Safety: the original DebugTrace pointee and every borrowed slot name must
    /// stay live and unmodified during this dump. Traced Program/context access is
    /// exclusive under the same existing trace lease as unsafe appendStages. The
    /// output stream/callback owner satisfies its native mutable-write contract.
    pub unsafe fn dump(
        &self,
        out: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
        writeInstructionCount: bool,
    ) {
        Dumper::new(self).dump(out, writeInstructionCount);
    }
    pub unsafe fn dump_default(
        &self,
        out: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
    ) {
        self.dump(out, false);
    }
}
