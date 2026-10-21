// Copyright 2023 Google LLC. BSD-style license.
// Macro expansion becomes an explicit typed enum. Explicit discriminants retain
// the original LOWP + HIGHP_ONLY + SKSL expansion order while source regions
// remain ordered by upstream file location. This declares op IDs, not kernels.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkRasterPipelineOp {
    // Source: src/core/SkRasterPipelineOpList.h:13-13
    move_src_dst = 0,
    move_dst_src = 1,
    swap_src_dst = 2,
    // Source: src/core/SkRasterPipelineOpList.h:14-14
    clamp_01 = 3,
    clamp_a_01 = 4,
    clamp_gamut = 5,
    // Source: src/core/SkRasterPipelineOpList.h:15-15
    premul = 6,
    premul_dst = 7,
    // Source: src/core/SkRasterPipelineOpList.h:16-16
    force_opaque = 8,
    force_opaque_dst = 9,
    // Source: src/core/SkRasterPipelineOpList.h:17-17
    set_rgb = 10,
    swap_rb = 11,
    swap_rb_dst = 12,
    // Source: src/core/SkRasterPipelineOpList.h:18-18
    black_color = 13,
    white_color = 14,
    // Source: src/core/SkRasterPipelineOpList.h:19-19
    uniform_color = 15,
    uniform_color_dst = 16,
    // Source: src/core/SkRasterPipelineOpList.h:20-20
    seed_shader = 17,
    // Source: src/core/SkRasterPipelineOpList.h:21-21
    load_a8 = 18,
    load_a8_dst = 19,
    store_a8 = 20,
    gather_a8 = 21,
    // Source: src/core/SkRasterPipelineOpList.h:22-22
    load_565 = 22,
    load_565_dst = 23,
    store_565 = 24,
    gather_565 = 25,
    // Source: src/core/SkRasterPipelineOpList.h:23-23
    load_4444 = 26,
    load_4444_dst = 27,
    store_4444 = 28,
    gather_4444 = 29,
    // Source: src/core/SkRasterPipelineOpList.h:24-24
    load_8888 = 30,
    load_8888_dst = 31,
    store_8888 = 32,
    gather_8888 = 33,
    // Source: src/core/SkRasterPipelineOpList.h:25-25
    load_rg88 = 34,
    load_rg88_dst = 35,
    store_rg88 = 36,
    gather_rg88 = 37,
    // Source: src/core/SkRasterPipelineOpList.h:26-26
    store_r8 = 38,
    // Source: src/core/SkRasterPipelineOpList.h:27-27
    alpha_to_gray = 39,
    alpha_to_gray_dst = 40,
    // Source: src/core/SkRasterPipelineOpList.h:28-28
    alpha_to_red = 41,
    alpha_to_red_dst = 42,
    // Source: src/core/SkRasterPipelineOpList.h:29-29
    bt709_luminance_or_luma_to_alpha = 43,
    // Source: src/core/SkRasterPipelineOpList.h:30-30
    bt709_luminance_or_luma_to_rgb = 44,
    // Source: src/core/SkRasterPipelineOpList.h:31-31
    bilerp_clamp_8888 = 45,
    // Source: src/core/SkRasterPipelineOpList.h:32-32
    load_src = 46,
    store_src = 47,
    store_src_a = 48,
    // Source: src/core/SkRasterPipelineOpList.h:33-33
    load_dst = 49,
    store_dst = 50,
    // Source: src/core/SkRasterPipelineOpList.h:34-34
    scale_u8 = 51,
    scale_565 = 52,
    scale_1_float = 53,
    scale_native = 54,
    // Source: src/core/SkRasterPipelineOpList.h:35-35
    lerp_u8 = 55,
    lerp_565 = 56,
    lerp_1_float = 57,
    lerp_native = 58,
    // Source: src/core/SkRasterPipelineOpList.h:36-36
    dstatop = 59,
    dstin = 60,
    dstout = 61,
    dstover = 62,
    // Source: src/core/SkRasterPipelineOpList.h:37-37
    srcatop = 63,
    srcin = 64,
    srcout = 65,
    srcover = 66,
    // Source: src/core/SkRasterPipelineOpList.h:38-38
    clear = 67,
    modulate = 68,
    multiply = 69,
    plus_ = 70,
    screen = 71,
    xor_ = 72,
    // Source: src/core/SkRasterPipelineOpList.h:39-39
    darken = 73,
    difference = 74,
    // Source: src/core/SkRasterPipelineOpList.h:40-40
    exclusion = 75,
    hardlight = 76,
    lighten = 77,
    overlay = 78,
    // Source: src/core/SkRasterPipelineOpList.h:41-41
    srcover_rgba_8888 = 79,
    // Source: src/core/SkRasterPipelineOpList.h:42-42
    matrix_translate = 80,
    matrix_scale_translate = 81,
    // Source: src/core/SkRasterPipelineOpList.h:43-43
    matrix_2x3 = 82,
    // Source: src/core/SkRasterPipelineOpList.h:44-44
    matrix_perspective = 83,
    // Source: src/core/SkRasterPipelineOpList.h:45-45
    decal_x = 84,
    decal_y = 85,
    decal_x_and_y = 86,
    // Source: src/core/SkRasterPipelineOpList.h:46-46
    check_decal_mask = 87,
    // Source: src/core/SkRasterPipelineOpList.h:47-47
    clamp_x_1 = 88,
    mirror_x_1 = 89,
    repeat_x_1 = 90,
    // Source: src/core/SkRasterPipelineOpList.h:48-48
    clamp_x_and_y = 91,
    // Source: src/core/SkRasterPipelineOpList.h:49-49
    evenly_spaced_gradient = 92,
    // Source: src/core/SkRasterPipelineOpList.h:50-50
    gradient = 93,
    // Source: src/core/SkRasterPipelineOpList.h:51-51
    evenly_spaced_2_stop_gradient = 94,
    // Source: src/core/SkRasterPipelineOpList.h:52-52
    xy_to_unit_angle = 95,
    // Source: src/core/SkRasterPipelineOpList.h:53-53
    xy_to_radius = 96,
    // Source: src/core/SkRasterPipelineOpList.h:54-54
    emboss = 97,
    // Source: src/core/SkRasterPipelineOpList.h:55-55
    swizzle = 98,
    // Source: src/core/SkRasterPipelineOpList.h:56-56
    debug_x = 99,
    debug_y = 100,
    debug_r = 101,
    debug_g = 102,
    debug_b = 103,
    debug_a = 104,
    // Source: src/core/SkRasterPipelineOpList.h:57-57
    debug_r_255 = 105,
    debug_g_255 = 106,
    debug_b_255 = 107,
    debug_a_255 = 108,
    // Source: src/core/SkRasterPipelineOpList.h:66-66
    init_lane_masks = 235,
    store_device_xy01 = 236,
    exchange_src = 237,
    // Source: src/core/SkRasterPipelineOpList.h:67-67
    load_condition_mask = 238,
    store_condition_mask = 239,
    // Source: src/core/SkRasterPipelineOpList.h:68-68
    merge_condition_mask = 240,
    merge_inv_condition_mask = 241,
    // Source: src/core/SkRasterPipelineOpList.h:69-69
    load_loop_mask = 242,
    store_loop_mask = 243,
    mask_off_loop_mask = 244,
    // Source: src/core/SkRasterPipelineOpList.h:70-70
    reenable_loop_mask = 245,
    merge_loop_mask = 246,
    case_op = 247,
    continue_op = 248,
    // Source: src/core/SkRasterPipelineOpList.h:71-71
    load_return_mask = 249,
    store_return_mask = 250,
    mask_off_return_mask = 251,
    // Source: src/core/SkRasterPipelineOpList.h:72-72
    branch_if_all_lanes_active = 252,
    branch_if_any_lanes_active = 253,
    branch_if_no_lanes_active = 254,
    // Source: src/core/SkRasterPipelineOpList.h:73-73
    branch_if_no_active_lanes_eq = 255,
    jump = 256,
    // Source: src/core/SkRasterPipelineOpList.h:74-74
    bitwise_and_imm_4_ints = 257,
    // Source: src/core/SkRasterPipelineOpList.h:75-75
    bitwise_and_imm_3_ints = 258,
    bitwise_and_imm_2_ints = 259,
    bitwise_and_imm_int = 260,
    // Source: src/core/SkRasterPipelineOpList.h:76-76
    bitwise_and_n_ints = 261,
    bitwise_and_int = 262,
    bitwise_and_2_ints = 263,
    // Source: src/core/SkRasterPipelineOpList.h:77-77
    bitwise_and_3_ints = 264,
    bitwise_and_4_ints = 265,
    // Source: src/core/SkRasterPipelineOpList.h:78-78
    bitwise_or_n_ints = 266,
    // Source: src/core/SkRasterPipelineOpList.h:79-79
    bitwise_or_int = 267,
    bitwise_or_2_ints = 268,
    bitwise_or_3_ints = 269,
    bitwise_or_4_ints = 270,
    // Source: src/core/SkRasterPipelineOpList.h:80-80
    bitwise_xor_imm_int = 271,
    // Source: src/core/SkRasterPipelineOpList.h:81-81
    bitwise_xor_n_ints = 272,
    bitwise_xor_int = 273,
    bitwise_xor_2_ints = 274,
    // Source: src/core/SkRasterPipelineOpList.h:82-82
    bitwise_xor_3_ints = 275,
    bitwise_xor_4_ints = 276,
    // Source: src/core/SkRasterPipelineOpList.h:83-83
    cast_to_float_from_int = 277,
    cast_to_float_from_2_ints = 278,
    // Source: src/core/SkRasterPipelineOpList.h:84-84
    cast_to_float_from_3_ints = 279,
    cast_to_float_from_4_ints = 280,
    // Source: src/core/SkRasterPipelineOpList.h:85-85
    cast_to_float_from_uint = 281,
    cast_to_float_from_2_uints = 282,
    // Source: src/core/SkRasterPipelineOpList.h:86-86
    cast_to_float_from_3_uints = 283,
    cast_to_float_from_4_uints = 284,
    // Source: src/core/SkRasterPipelineOpList.h:87-87
    cast_to_int_from_float = 285,
    cast_to_int_from_2_floats = 286,
    // Source: src/core/SkRasterPipelineOpList.h:88-88
    cast_to_int_from_3_floats = 287,
    cast_to_int_from_4_floats = 288,
    // Source: src/core/SkRasterPipelineOpList.h:89-89
    cast_to_uint_from_float = 289,
    cast_to_uint_from_2_floats = 290,
    // Source: src/core/SkRasterPipelineOpList.h:90-90
    cast_to_uint_from_3_floats = 291,
    cast_to_uint_from_4_floats = 292,
    // Source: src/core/SkRasterPipelineOpList.h:91-91
    abs_int = 293,
    abs_2_ints = 294,
    abs_3_ints = 295,
    abs_4_ints = 296,
    // Source: src/core/SkRasterPipelineOpList.h:92-92
    floor_float = 297,
    floor_2_floats = 298,
    floor_3_floats = 299,
    floor_4_floats = 300,
    // Source: src/core/SkRasterPipelineOpList.h:93-93
    ceil_float = 301,
    ceil_2_floats = 302,
    ceil_3_floats = 303,
    ceil_4_floats = 304,
    // Source: src/core/SkRasterPipelineOpList.h:94-94
    invsqrt_float = 305,
    invsqrt_2_floats = 306,
    invsqrt_3_floats = 307,
    invsqrt_4_floats = 308,
    // Source: src/core/SkRasterPipelineOpList.h:95-95
    inverse_mat2 = 309,
    inverse_mat3 = 310,
    inverse_mat4 = 311,
    // Source: src/core/SkRasterPipelineOpList.h:96-96
    sin_float = 312,
    cos_float = 313,
    tan_float = 314,
    // Source: src/core/SkRasterPipelineOpList.h:97-97
    asin_float = 315,
    acos_float = 316,
    atan_float = 317,
    atan2_n_floats = 318,
    // Source: src/core/SkRasterPipelineOpList.h:98-98
    sqrt_float = 319,
    pow_n_floats = 320,
    exp_float = 321,
    exp2_float = 322,
    // Source: src/core/SkRasterPipelineOpList.h:99-99
    log_float = 323,
    log2_float = 324,
    refract_4_floats = 325,
    // Source: src/core/SkRasterPipelineOpList.h:100-100
    copy_uniform = 326,
    copy_2_uniforms = 327,
    copy_3_uniforms = 328,
    copy_4_uniforms = 329,
    // Source: src/core/SkRasterPipelineOpList.h:101-101
    copy_constant = 330,
    splat_2_constants = 331,
    splat_3_constants = 332,
    splat_4_constants = 333,
    // Source: src/core/SkRasterPipelineOpList.h:102-102
    copy_slot_masked = 334,
    copy_2_slots_masked = 335,
    copy_3_slots_masked = 336,
    copy_4_slots_masked = 337,
    // Source: src/core/SkRasterPipelineOpList.h:103-103
    copy_from_indirect_unmasked = 338,
    copy_from_indirect_uniform_unmasked = 339,
    // Source: src/core/SkRasterPipelineOpList.h:104-104
    copy_to_indirect_masked = 340,
    swizzle_copy_to_indirect_masked = 341,
    // Source: src/core/SkRasterPipelineOpList.h:105-105
    copy_slot_unmasked = 342,
    copy_2_slots_unmasked = 343,
    // Source: src/core/SkRasterPipelineOpList.h:106-106
    copy_3_slots_unmasked = 344,
    copy_4_slots_unmasked = 345,
    // Source: src/core/SkRasterPipelineOpList.h:107-107
    copy_immutable_unmasked = 346,
    copy_2_immutables_unmasked = 347,
    // Source: src/core/SkRasterPipelineOpList.h:108-108
    copy_3_immutables_unmasked = 348,
    copy_4_immutables_unmasked = 349,
    // Source: src/core/SkRasterPipelineOpList.h:109-109
    swizzle_copy_slot_masked = 350,
    swizzle_copy_2_slots_masked = 351,
    // Source: src/core/SkRasterPipelineOpList.h:110-110
    swizzle_copy_3_slots_masked = 352,
    swizzle_copy_4_slots_masked = 353,
    // Source: src/core/SkRasterPipelineOpList.h:111-111
    swizzle_1 = 354,
    swizzle_2 = 355,
    swizzle_3 = 356,
    swizzle_4 = 357,
    shuffle = 358,
    // Source: src/core/SkRasterPipelineOpList.h:112-112
    matrix_multiply_2 = 359,
    matrix_multiply_3 = 360,
    matrix_multiply_4 = 361,
    // Source: src/core/SkRasterPipelineOpList.h:113-113
    smoothstep_n_floats = 362,
    dot_2_floats = 363,
    dot_3_floats = 364,
    dot_4_floats = 365,
    // Source: src/core/SkRasterPipelineOpList.h:114-114
    add_imm_float = 366,
    // Source: src/core/SkRasterPipelineOpList.h:115-115
    add_n_floats = 367,
    add_float = 368,
    add_2_floats = 369,
    add_3_floats = 370,
    add_4_floats = 371,
    // Source: src/core/SkRasterPipelineOpList.h:116-116
    add_imm_int = 372,
    // Source: src/core/SkRasterPipelineOpList.h:117-117
    add_n_ints = 373,
    add_int = 374,
    add_2_ints = 375,
    add_3_ints = 376,
    add_4_ints = 377,
    // Source: src/core/SkRasterPipelineOpList.h:118-118
    sub_n_floats = 378,
    sub_float = 379,
    sub_2_floats = 380,
    sub_3_floats = 381,
    sub_4_floats = 382,
    // Source: src/core/SkRasterPipelineOpList.h:119-119
    sub_n_ints = 383,
    sub_int = 384,
    sub_2_ints = 385,
    sub_3_ints = 386,
    sub_4_ints = 387,
    // Source: src/core/SkRasterPipelineOpList.h:120-120
    mul_imm_float = 388,
    // Source: src/core/SkRasterPipelineOpList.h:121-121
    mul_n_floats = 389,
    mul_float = 390,
    mul_2_floats = 391,
    mul_3_floats = 392,
    mul_4_floats = 393,
    // Source: src/core/SkRasterPipelineOpList.h:122-122
    mul_imm_int = 394,
    // Source: src/core/SkRasterPipelineOpList.h:123-123
    mul_n_ints = 395,
    mul_int = 396,
    mul_2_ints = 397,
    mul_3_ints = 398,
    mul_4_ints = 399,
    // Source: src/core/SkRasterPipelineOpList.h:124-124
    div_n_floats = 400,
    div_float = 401,
    div_2_floats = 402,
    div_3_floats = 403,
    div_4_floats = 404,
    // Source: src/core/SkRasterPipelineOpList.h:125-125
    div_n_ints = 405,
    div_int = 406,
    div_2_ints = 407,
    div_3_ints = 408,
    div_4_ints = 409,
    // Source: src/core/SkRasterPipelineOpList.h:126-126
    div_n_uints = 410,
    div_uint = 411,
    div_2_uints = 412,
    div_3_uints = 413,
    div_4_uints = 414,
    // Source: src/core/SkRasterPipelineOpList.h:127-127
    max_imm_float = 415,
    // Source: src/core/SkRasterPipelineOpList.h:128-128
    max_n_floats = 416,
    max_float = 417,
    max_2_floats = 418,
    max_3_floats = 419,
    max_4_floats = 420,
    // Source: src/core/SkRasterPipelineOpList.h:129-129
    max_n_ints = 421,
    max_int = 422,
    max_2_ints = 423,
    max_3_ints = 424,
    max_4_ints = 425,
    // Source: src/core/SkRasterPipelineOpList.h:130-130
    max_n_uints = 426,
    max_uint = 427,
    max_2_uints = 428,
    max_3_uints = 429,
    max_4_uints = 430,
    // Source: src/core/SkRasterPipelineOpList.h:131-131
    min_imm_float = 431,
    // Source: src/core/SkRasterPipelineOpList.h:132-132
    min_n_floats = 432,
    min_float = 433,
    min_2_floats = 434,
    min_3_floats = 435,
    min_4_floats = 436,
    // Source: src/core/SkRasterPipelineOpList.h:133-133
    min_n_ints = 437,
    min_int = 438,
    min_2_ints = 439,
    min_3_ints = 440,
    min_4_ints = 441,
    // Source: src/core/SkRasterPipelineOpList.h:134-134
    min_n_uints = 442,
    min_uint = 443,
    min_2_uints = 444,
    min_3_uints = 445,
    min_4_uints = 446,
    // Source: src/core/SkRasterPipelineOpList.h:135-135
    mod_n_floats = 447,
    mod_float = 448,
    mod_2_floats = 449,
    mod_3_floats = 450,
    mod_4_floats = 451,
    // Source: src/core/SkRasterPipelineOpList.h:136-136
    mix_n_floats = 452,
    mix_float = 453,
    mix_2_floats = 454,
    mix_3_floats = 455,
    mix_4_floats = 456,
    // Source: src/core/SkRasterPipelineOpList.h:137-137
    mix_n_ints = 457,
    mix_int = 458,
    mix_2_ints = 459,
    mix_3_ints = 460,
    mix_4_ints = 461,
    // Source: src/core/SkRasterPipelineOpList.h:138-138
    cmplt_imm_float = 462,
    // Source: src/core/SkRasterPipelineOpList.h:139-139
    cmplt_n_floats = 463,
    cmplt_float = 464,
    cmplt_2_floats = 465,
    cmplt_3_floats = 466,
    cmplt_4_floats = 467,
    // Source: src/core/SkRasterPipelineOpList.h:140-140
    cmplt_imm_int = 468,
    // Source: src/core/SkRasterPipelineOpList.h:141-141
    cmplt_n_ints = 469,
    cmplt_int = 470,
    cmplt_2_ints = 471,
    cmplt_3_ints = 472,
    cmplt_4_ints = 473,
    // Source: src/core/SkRasterPipelineOpList.h:142-142
    cmplt_imm_uint = 474,
    // Source: src/core/SkRasterPipelineOpList.h:143-143
    cmplt_n_uints = 475,
    cmplt_uint = 476,
    cmplt_2_uints = 477,
    cmplt_3_uints = 478,
    cmplt_4_uints = 479,
    // Source: src/core/SkRasterPipelineOpList.h:144-144
    cmple_imm_float = 480,
    // Source: src/core/SkRasterPipelineOpList.h:145-145
    cmple_n_floats = 481,
    cmple_float = 482,
    cmple_2_floats = 483,
    cmple_3_floats = 484,
    cmple_4_floats = 485,
    // Source: src/core/SkRasterPipelineOpList.h:146-146
    cmple_imm_int = 486,
    // Source: src/core/SkRasterPipelineOpList.h:147-147
    cmple_n_ints = 487,
    cmple_int = 488,
    cmple_2_ints = 489,
    cmple_3_ints = 490,
    cmple_4_ints = 491,
    // Source: src/core/SkRasterPipelineOpList.h:148-148
    cmple_imm_uint = 492,
    // Source: src/core/SkRasterPipelineOpList.h:149-149
    cmple_n_uints = 493,
    cmple_uint = 494,
    cmple_2_uints = 495,
    cmple_3_uints = 496,
    cmple_4_uints = 497,
    // Source: src/core/SkRasterPipelineOpList.h:150-150
    cmpeq_imm_float = 498,
    // Source: src/core/SkRasterPipelineOpList.h:151-151
    cmpeq_n_floats = 499,
    cmpeq_float = 500,
    cmpeq_2_floats = 501,
    cmpeq_3_floats = 502,
    cmpeq_4_floats = 503,
    // Source: src/core/SkRasterPipelineOpList.h:152-152
    cmpeq_imm_int = 504,
    // Source: src/core/SkRasterPipelineOpList.h:153-153
    cmpeq_n_ints = 505,
    cmpeq_int = 506,
    cmpeq_2_ints = 507,
    cmpeq_3_ints = 508,
    cmpeq_4_ints = 509,
    // Source: src/core/SkRasterPipelineOpList.h:154-154
    cmpne_imm_float = 510,
    // Source: src/core/SkRasterPipelineOpList.h:155-155
    cmpne_n_floats = 511,
    cmpne_float = 512,
    cmpne_2_floats = 513,
    cmpne_3_floats = 514,
    cmpne_4_floats = 515,
    // Source: src/core/SkRasterPipelineOpList.h:156-156
    cmpne_imm_int = 516,
    // Source: src/core/SkRasterPipelineOpList.h:157-157
    cmpne_n_ints = 517,
    cmpne_int = 518,
    cmpne_2_ints = 519,
    cmpne_3_ints = 520,
    cmpne_4_ints = 521,
    // Source: src/core/SkRasterPipelineOpList.h:158-158
    trace_line = 522,
    trace_var = 523,
    trace_enter = 524,
    trace_exit = 525,
    trace_scope = 526,
    // Source: src/core/SkRasterPipelineOpList.h:163-163
    callback = 109,
    // Source: src/core/SkRasterPipelineOpList.h:164-164
    stack_checkpoint = 110,
    stack_rewind = 111,
    // Source: src/core/SkRasterPipelineOpList.h:165-165
    unbounded_set_rgb = 112,
    unbounded_uniform_color = 113,
    // Source: src/core/SkRasterPipelineOpList.h:166-166
    unpremul = 114,
    unpremul_polar = 115,
    dither = 116,
    // Source: src/core/SkRasterPipelineOpList.h:167-167
    load_16161616 = 117,
    load_16161616_dst = 118,
    store_16161616 = 119,
    gather_16161616 = 120,
    // Source: src/core/SkRasterPipelineOpList.h:168-168
    load_a16 = 121,
    load_a16_dst = 122,
    store_a16 = 123,
    gather_a16 = 124,
    // Source: src/core/SkRasterPipelineOpList.h:169-169
    load_r16 = 125,
    load_r16_dst = 126,
    store_r16 = 127,
    gather_r16 = 128,
    // Source: src/core/SkRasterPipelineOpList.h:170-170
    load_rg1616 = 129,
    load_rg1616_dst = 130,
    store_rg1616 = 131,
    gather_rg1616 = 132,
    // Source: src/core/SkRasterPipelineOpList.h:171-171
    load_f16 = 133,
    load_f16_dst = 134,
    store_f16 = 135,
    gather_f16 = 136,
    // Source: src/core/SkRasterPipelineOpList.h:172-172
    load_af16 = 137,
    load_af16_dst = 138,
    store_af16 = 139,
    gather_af16 = 140,
    // Source: src/core/SkRasterPipelineOpList.h:173-173
    load_rf16 = 141,
    load_rf16_dst = 142,
    store_rf16 = 143,
    gather_rf16 = 144,
    // Source: src/core/SkRasterPipelineOpList.h:174-174
    load_rgf16 = 145,
    load_rgf16_dst = 146,
    store_rgf16 = 147,
    gather_rgf16 = 148,
    // Source: src/core/SkRasterPipelineOpList.h:175-175
    load_f32 = 149,
    load_f32_dst = 150,
    store_f32 = 151,
    gather_f32 = 152,
    // Source: src/core/SkRasterPipelineOpList.h:176-176
    load_1010102 = 153,
    load_1010102_dst = 154,
    store_1010102 = 155,
    gather_1010102 = 156,
    // Source: src/core/SkRasterPipelineOpList.h:177-177
    load_1010102_xr = 157,
    load_1010102_xr_dst = 158,
    store_1010102_xr = 159,
    gather_1010102_xr = 160,
    // Source: src/core/SkRasterPipelineOpList.h:178-178
    load_10x6 = 161,
    load_10x6_dst = 162,
    store_10x6 = 163,
    gather_10x6 = 164,
    // Source: src/core/SkRasterPipelineOpList.h:179-179
    gather_10101010_xr = 165,
    load_10101010_xr = 166,
    load_10101010_xr_dst = 167,
    store_10101010_xr = 168,
    // Source: src/core/SkRasterPipelineOpList.h:180-180
    store_src_rg = 169,
    load_src_rg = 170,
    // Source: src/core/SkRasterPipelineOpList.h:181-181
    byte_tables = 171,
    // Source: src/core/SkRasterPipelineOpList.h:182-182
    colorburn = 172,
    colordodge = 173,
    softlight = 174,
    // Source: src/core/SkRasterPipelineOpList.h:183-183
    hue = 175,
    saturation = 176,
    color = 177,
    luminosity = 178,
    // Source: src/core/SkRasterPipelineOpList.h:184-184
    matrix_3x3 = 179,
    matrix_3x4 = 180,
    matrix_4x5 = 181,
    matrix_4x3 = 182,
    // Source: src/core/SkRasterPipelineOpList.h:185-185
    parametric = 183,
    gamma_ = 184,
    PQish = 185,
    HLGish = 186,
    HLGinvish = 187,
    ootf = 188,
    // Source: src/core/SkRasterPipelineOpList.h:186-186
    rgb_to_hsl = 189,
    hsl_to_rgb = 190,
    // Source: src/core/SkRasterPipelineOpList.h:187-187
    css_lab_to_xyz = 191,
    css_oklab_to_linear_srgb = 192,
    // Source: src/core/SkRasterPipelineOpList.h:188-188
    css_oklab_gamut_map_to_linear_srgb = 193,
    // Source: src/core/SkRasterPipelineOpList.h:189-189
    css_hcl_to_lab = 194,
    // Source: src/core/SkRasterPipelineOpList.h:190-190
    css_hsl_to_srgb = 195,
    css_hwb_to_srgb = 196,
    // Source: src/core/SkRasterPipelineOpList.h:191-191
    gauss_a_to_rgba = 197,
    // Source: src/core/SkRasterPipelineOpList.h:192-192
    mirror_x = 198,
    repeat_x = 199,
    // Source: src/core/SkRasterPipelineOpList.h:193-193
    mirror_y = 200,
    repeat_y = 201,
    // Source: src/core/SkRasterPipelineOpList.h:194-194
    negate_x = 202,
    // Source: src/core/SkRasterPipelineOpList.h:195-195
    bilerp_clamp_8888_force_highp = 203,
    // Source: src/core/SkRasterPipelineOpList.h:196-196
    bicubic_clamp_8888 = 204,
    // Source: src/core/SkRasterPipelineOpList.h:197-197
    bilinear_setup = 205,
    // Source: src/core/SkRasterPipelineOpList.h:198-198
    bilinear_nx = 206,
    bilinear_px = 207,
    bilinear_ny = 208,
    bilinear_py = 209,
    // Source: src/core/SkRasterPipelineOpList.h:199-199
    bicubic_setup = 210,
    // Source: src/core/SkRasterPipelineOpList.h:200-200
    bicubic_n3x = 211,
    bicubic_n1x = 212,
    bicubic_p1x = 213,
    bicubic_p3x = 214,
    // Source: src/core/SkRasterPipelineOpList.h:201-201
    bicubic_n3y = 215,
    bicubic_n1y = 216,
    bicubic_p1y = 217,
    bicubic_p3y = 218,
    // Source: src/core/SkRasterPipelineOpList.h:202-202
    accumulate = 219,
    // Source: src/core/SkRasterPipelineOpList.h:203-203
    perlin_noise = 220,
    // Source: src/core/SkRasterPipelineOpList.h:204-204
    mipmap_linear_init = 221,
    mipmap_linear_update = 222,
    mipmap_linear_finish = 223,
    // Source: src/core/SkRasterPipelineOpList.h:205-205
    xy_to_2pt_conical_strip = 224,
    // Source: src/core/SkRasterPipelineOpList.h:206-206
    xy_to_2pt_conical_focal_on_circle = 225,
    // Source: src/core/SkRasterPipelineOpList.h:207-207
    xy_to_2pt_conical_well_behaved = 226,
    // Source: src/core/SkRasterPipelineOpList.h:208-208
    xy_to_2pt_conical_smaller = 227,
    // Source: src/core/SkRasterPipelineOpList.h:209-209
    xy_to_2pt_conical_greater = 228,
    // Source: src/core/SkRasterPipelineOpList.h:210-210
    alter_2pt_conical_compensate_focal = 229,
    // Source: src/core/SkRasterPipelineOpList.h:211-211
    alter_2pt_conical_unswap = 230,
    // Source: src/core/SkRasterPipelineOpList.h:212-212
    mask_2pt_conical_nan = 231,
    // Source: src/core/SkRasterPipelineOpList.h:213-213
    mask_2pt_conical_degenerates = 232,
    apply_vector_mask = 233,
    // Source: src/core/SkRasterPipelineOpList.h:214-214
    set_base_pointer = 234,
}
// Source: src/core/SkRasterPipelineOpList.h:231-232
pub const kNumRasterPipelineLowpOps: usize = 109;
pub const kNumRasterPipelineHighpOps: usize = 527;
impl SkRasterPipelineOp {
    pub fn name(self) -> &'static str {
        match self {
            Self::debug_r_255 => "debug_r_255",
            Self::debug_g_255 => "debug_g_255",
            Self::debug_b_255 => "debug_b_255",
            Self::debug_a_255 => "debug_a_255",

            Self::move_src_dst => "move_src_dst",
            Self::move_dst_src => "move_dst_src",
            Self::swap_src_dst => "swap_src_dst",
            Self::clamp_01 => "clamp_01",
            Self::clamp_a_01 => "clamp_a_01",
            Self::clamp_gamut => "clamp_gamut",
            Self::premul => "premul",
            Self::premul_dst => "premul_dst",
            Self::force_opaque => "force_opaque",
            Self::force_opaque_dst => "force_opaque_dst",
            Self::set_rgb => "set_rgb",
            Self::swap_rb => "swap_rb",
            Self::swap_rb_dst => "swap_rb_dst",
            Self::black_color => "black_color",
            Self::white_color => "white_color",
            Self::uniform_color => "uniform_color",
            Self::uniform_color_dst => "uniform_color_dst",
            Self::seed_shader => "seed_shader",
            Self::load_a8 => "load_a8",
            Self::load_a8_dst => "load_a8_dst",
            Self::store_a8 => "store_a8",
            Self::gather_a8 => "gather_a8",
            Self::load_565 => "load_565",
            Self::load_565_dst => "load_565_dst",
            Self::store_565 => "store_565",
            Self::gather_565 => "gather_565",
            Self::load_4444 => "load_4444",
            Self::load_4444_dst => "load_4444_dst",
            Self::store_4444 => "store_4444",
            Self::gather_4444 => "gather_4444",
            Self::load_8888 => "load_8888",
            Self::load_8888_dst => "load_8888_dst",
            Self::store_8888 => "store_8888",
            Self::gather_8888 => "gather_8888",
            Self::load_rg88 => "load_rg88",
            Self::load_rg88_dst => "load_rg88_dst",
            Self::store_rg88 => "store_rg88",
            Self::gather_rg88 => "gather_rg88",
            Self::store_r8 => "store_r8",
            Self::alpha_to_gray => "alpha_to_gray",
            Self::alpha_to_gray_dst => "alpha_to_gray_dst",
            Self::alpha_to_red => "alpha_to_red",
            Self::alpha_to_red_dst => "alpha_to_red_dst",
            Self::bt709_luminance_or_luma_to_alpha => "bt709_luminance_or_luma_to_alpha",
            Self::bt709_luminance_or_luma_to_rgb => "bt709_luminance_or_luma_to_rgb",
            Self::bilerp_clamp_8888 => "bilerp_clamp_8888",
            Self::load_src => "load_src",
            Self::store_src => "store_src",
            Self::store_src_a => "store_src_a",
            Self::load_dst => "load_dst",
            Self::store_dst => "store_dst",
            Self::scale_u8 => "scale_u8",
            Self::scale_565 => "scale_565",
            Self::scale_1_float => "scale_1_float",
            Self::scale_native => "scale_native",
            Self::lerp_u8 => "lerp_u8",
            Self::lerp_565 => "lerp_565",
            Self::lerp_1_float => "lerp_1_float",
            Self::lerp_native => "lerp_native",
            Self::dstatop => "dstatop",
            Self::dstin => "dstin",
            Self::dstout => "dstout",
            Self::dstover => "dstover",
            Self::srcatop => "srcatop",
            Self::srcin => "srcin",
            Self::srcout => "srcout",
            Self::srcover => "srcover",
            Self::clear => "clear",
            Self::modulate => "modulate",
            Self::multiply => "multiply",
            Self::plus_ => "plus_",
            Self::screen => "screen",
            Self::xor_ => "xor_",
            Self::darken => "darken",
            Self::difference => "difference",
            Self::exclusion => "exclusion",
            Self::hardlight => "hardlight",
            Self::lighten => "lighten",
            Self::overlay => "overlay",
            Self::srcover_rgba_8888 => "srcover_rgba_8888",
            Self::matrix_translate => "matrix_translate",
            Self::matrix_scale_translate => "matrix_scale_translate",
            Self::matrix_2x3 => "matrix_2x3",
            Self::matrix_perspective => "matrix_perspective",
            Self::decal_x => "decal_x",
            Self::decal_y => "decal_y",
            Self::decal_x_and_y => "decal_x_and_y",
            Self::check_decal_mask => "check_decal_mask",
            Self::clamp_x_1 => "clamp_x_1",
            Self::mirror_x_1 => "mirror_x_1",
            Self::repeat_x_1 => "repeat_x_1",
            Self::clamp_x_and_y => "clamp_x_and_y",
            Self::evenly_spaced_gradient => "evenly_spaced_gradient",
            Self::gradient => "gradient",
            Self::evenly_spaced_2_stop_gradient => "evenly_spaced_2_stop_gradient",
            Self::xy_to_unit_angle => "xy_to_unit_angle",
            Self::xy_to_radius => "xy_to_radius",
            Self::emboss => "emboss",
            Self::swizzle => "swizzle",
            Self::debug_x => "debug_x",
            Self::debug_y => "debug_y",
            Self::debug_r => "debug_r",
            Self::debug_g => "debug_g",
            Self::debug_b => "debug_b",
            Self::debug_a => "debug_a",
            Self::callback => "callback",
            Self::stack_checkpoint => "stack_checkpoint",
            Self::stack_rewind => "stack_rewind",
            Self::unbounded_set_rgb => "unbounded_set_rgb",
            Self::unbounded_uniform_color => "unbounded_uniform_color",
            Self::unpremul => "unpremul",
            Self::unpremul_polar => "unpremul_polar",
            Self::dither => "dither",
            Self::load_16161616 => "load_16161616",
            Self::load_16161616_dst => "load_16161616_dst",
            Self::store_16161616 => "store_16161616",
            Self::gather_16161616 => "gather_16161616",
            Self::load_a16 => "load_a16",
            Self::load_a16_dst => "load_a16_dst",
            Self::store_a16 => "store_a16",
            Self::gather_a16 => "gather_a16",
            Self::load_r16 => "load_r16",
            Self::load_r16_dst => "load_r16_dst",
            Self::store_r16 => "store_r16",
            Self::gather_r16 => "gather_r16",
            Self::load_rg1616 => "load_rg1616",
            Self::load_rg1616_dst => "load_rg1616_dst",
            Self::store_rg1616 => "store_rg1616",
            Self::gather_rg1616 => "gather_rg1616",
            Self::load_f16 => "load_f16",
            Self::load_f16_dst => "load_f16_dst",
            Self::store_f16 => "store_f16",
            Self::gather_f16 => "gather_f16",
            Self::load_af16 => "load_af16",
            Self::load_af16_dst => "load_af16_dst",
            Self::store_af16 => "store_af16",
            Self::gather_af16 => "gather_af16",
            Self::load_rf16 => "load_rf16",
            Self::load_rf16_dst => "load_rf16_dst",
            Self::store_rf16 => "store_rf16",
            Self::gather_rf16 => "gather_rf16",
            Self::load_rgf16 => "load_rgf16",
            Self::load_rgf16_dst => "load_rgf16_dst",
            Self::store_rgf16 => "store_rgf16",
            Self::gather_rgf16 => "gather_rgf16",
            Self::load_f32 => "load_f32",
            Self::load_f32_dst => "load_f32_dst",
            Self::store_f32 => "store_f32",
            Self::gather_f32 => "gather_f32",
            Self::load_1010102 => "load_1010102",
            Self::load_1010102_dst => "load_1010102_dst",
            Self::store_1010102 => "store_1010102",
            Self::gather_1010102 => "gather_1010102",
            Self::load_1010102_xr => "load_1010102_xr",
            Self::load_1010102_xr_dst => "load_1010102_xr_dst",
            Self::store_1010102_xr => "store_1010102_xr",
            Self::gather_1010102_xr => "gather_1010102_xr",
            Self::load_10x6 => "load_10x6",
            Self::load_10x6_dst => "load_10x6_dst",
            Self::store_10x6 => "store_10x6",
            Self::gather_10x6 => "gather_10x6",
            Self::gather_10101010_xr => "gather_10101010_xr",
            Self::load_10101010_xr => "load_10101010_xr",
            Self::load_10101010_xr_dst => "load_10101010_xr_dst",
            Self::store_10101010_xr => "store_10101010_xr",
            Self::store_src_rg => "store_src_rg",
            Self::load_src_rg => "load_src_rg",
            Self::byte_tables => "byte_tables",
            Self::colorburn => "colorburn",
            Self::colordodge => "colordodge",
            Self::softlight => "softlight",
            Self::hue => "hue",
            Self::saturation => "saturation",
            Self::color => "color",
            Self::luminosity => "luminosity",
            Self::matrix_3x3 => "matrix_3x3",
            Self::matrix_3x4 => "matrix_3x4",
            Self::matrix_4x5 => "matrix_4x5",
            Self::matrix_4x3 => "matrix_4x3",
            Self::parametric => "parametric",
            Self::gamma_ => "gamma_",
            Self::PQish => "PQish",
            Self::HLGish => "HLGish",
            Self::HLGinvish => "HLGinvish",
            Self::ootf => "ootf",
            Self::rgb_to_hsl => "rgb_to_hsl",
            Self::hsl_to_rgb => "hsl_to_rgb",
            Self::css_lab_to_xyz => "css_lab_to_xyz",
            Self::css_oklab_to_linear_srgb => "css_oklab_to_linear_srgb",
            Self::css_oklab_gamut_map_to_linear_srgb => "css_oklab_gamut_map_to_linear_srgb",
            Self::css_hcl_to_lab => "css_hcl_to_lab",
            Self::css_hsl_to_srgb => "css_hsl_to_srgb",
            Self::css_hwb_to_srgb => "css_hwb_to_srgb",
            Self::gauss_a_to_rgba => "gauss_a_to_rgba",
            Self::mirror_x => "mirror_x",
            Self::repeat_x => "repeat_x",
            Self::mirror_y => "mirror_y",
            Self::repeat_y => "repeat_y",
            Self::negate_x => "negate_x",
            Self::bilerp_clamp_8888_force_highp => "bilerp_clamp_8888_force_highp",
            Self::bicubic_clamp_8888 => "bicubic_clamp_8888",
            Self::bilinear_setup => "bilinear_setup",
            Self::bilinear_nx => "bilinear_nx",
            Self::bilinear_px => "bilinear_px",
            Self::bilinear_ny => "bilinear_ny",
            Self::bilinear_py => "bilinear_py",
            Self::bicubic_setup => "bicubic_setup",
            Self::bicubic_n3x => "bicubic_n3x",
            Self::bicubic_n1x => "bicubic_n1x",
            Self::bicubic_p1x => "bicubic_p1x",
            Self::bicubic_p3x => "bicubic_p3x",
            Self::bicubic_n3y => "bicubic_n3y",
            Self::bicubic_n1y => "bicubic_n1y",
            Self::bicubic_p1y => "bicubic_p1y",
            Self::bicubic_p3y => "bicubic_p3y",
            Self::accumulate => "accumulate",
            Self::perlin_noise => "perlin_noise",
            Self::mipmap_linear_init => "mipmap_linear_init",
            Self::mipmap_linear_update => "mipmap_linear_update",
            Self::mipmap_linear_finish => "mipmap_linear_finish",
            Self::xy_to_2pt_conical_strip => "xy_to_2pt_conical_strip",
            Self::xy_to_2pt_conical_focal_on_circle => "xy_to_2pt_conical_focal_on_circle",
            Self::xy_to_2pt_conical_well_behaved => "xy_to_2pt_conical_well_behaved",
            Self::xy_to_2pt_conical_smaller => "xy_to_2pt_conical_smaller",
            Self::xy_to_2pt_conical_greater => "xy_to_2pt_conical_greater",
            Self::alter_2pt_conical_compensate_focal => "alter_2pt_conical_compensate_focal",
            Self::alter_2pt_conical_unswap => "alter_2pt_conical_unswap",
            Self::mask_2pt_conical_nan => "mask_2pt_conical_nan",
            Self::mask_2pt_conical_degenerates => "mask_2pt_conical_degenerates",
            Self::apply_vector_mask => "apply_vector_mask",
            Self::set_base_pointer => "set_base_pointer",
            Self::init_lane_masks => "init_lane_masks",
            Self::store_device_xy01 => "store_device_xy01",
            Self::exchange_src => "exchange_src",
            Self::load_condition_mask => "load_condition_mask",
            Self::store_condition_mask => "store_condition_mask",
            Self::merge_condition_mask => "merge_condition_mask",
            Self::merge_inv_condition_mask => "merge_inv_condition_mask",
            Self::load_loop_mask => "load_loop_mask",
            Self::store_loop_mask => "store_loop_mask",
            Self::mask_off_loop_mask => "mask_off_loop_mask",
            Self::reenable_loop_mask => "reenable_loop_mask",
            Self::merge_loop_mask => "merge_loop_mask",
            Self::case_op => "case_op",
            Self::continue_op => "continue_op",
            Self::load_return_mask => "load_return_mask",
            Self::store_return_mask => "store_return_mask",
            Self::mask_off_return_mask => "mask_off_return_mask",
            Self::branch_if_all_lanes_active => "branch_if_all_lanes_active",
            Self::branch_if_any_lanes_active => "branch_if_any_lanes_active",
            Self::branch_if_no_lanes_active => "branch_if_no_lanes_active",
            Self::branch_if_no_active_lanes_eq => "branch_if_no_active_lanes_eq",
            Self::jump => "jump",
            Self::bitwise_and_imm_4_ints => "bitwise_and_imm_4_ints",
            Self::bitwise_and_imm_3_ints => "bitwise_and_imm_3_ints",
            Self::bitwise_and_imm_2_ints => "bitwise_and_imm_2_ints",
            Self::bitwise_and_imm_int => "bitwise_and_imm_int",
            Self::bitwise_and_n_ints => "bitwise_and_n_ints",
            Self::bitwise_and_int => "bitwise_and_int",
            Self::bitwise_and_2_ints => "bitwise_and_2_ints",
            Self::bitwise_and_3_ints => "bitwise_and_3_ints",
            Self::bitwise_and_4_ints => "bitwise_and_4_ints",
            Self::bitwise_or_n_ints => "bitwise_or_n_ints",
            Self::bitwise_or_int => "bitwise_or_int",
            Self::bitwise_or_2_ints => "bitwise_or_2_ints",
            Self::bitwise_or_3_ints => "bitwise_or_3_ints",
            Self::bitwise_or_4_ints => "bitwise_or_4_ints",
            Self::bitwise_xor_imm_int => "bitwise_xor_imm_int",
            Self::bitwise_xor_n_ints => "bitwise_xor_n_ints",
            Self::bitwise_xor_int => "bitwise_xor_int",
            Self::bitwise_xor_2_ints => "bitwise_xor_2_ints",
            Self::bitwise_xor_3_ints => "bitwise_xor_3_ints",
            Self::bitwise_xor_4_ints => "bitwise_xor_4_ints",
            Self::cast_to_float_from_int => "cast_to_float_from_int",
            Self::cast_to_float_from_2_ints => "cast_to_float_from_2_ints",
            Self::cast_to_float_from_3_ints => "cast_to_float_from_3_ints",
            Self::cast_to_float_from_4_ints => "cast_to_float_from_4_ints",
            Self::cast_to_float_from_uint => "cast_to_float_from_uint",
            Self::cast_to_float_from_2_uints => "cast_to_float_from_2_uints",
            Self::cast_to_float_from_3_uints => "cast_to_float_from_3_uints",
            Self::cast_to_float_from_4_uints => "cast_to_float_from_4_uints",
            Self::cast_to_int_from_float => "cast_to_int_from_float",
            Self::cast_to_int_from_2_floats => "cast_to_int_from_2_floats",
            Self::cast_to_int_from_3_floats => "cast_to_int_from_3_floats",
            Self::cast_to_int_from_4_floats => "cast_to_int_from_4_floats",
            Self::cast_to_uint_from_float => "cast_to_uint_from_float",
            Self::cast_to_uint_from_2_floats => "cast_to_uint_from_2_floats",
            Self::cast_to_uint_from_3_floats => "cast_to_uint_from_3_floats",
            Self::cast_to_uint_from_4_floats => "cast_to_uint_from_4_floats",
            Self::abs_int => "abs_int",
            Self::abs_2_ints => "abs_2_ints",
            Self::abs_3_ints => "abs_3_ints",
            Self::abs_4_ints => "abs_4_ints",
            Self::floor_float => "floor_float",
            Self::floor_2_floats => "floor_2_floats",
            Self::floor_3_floats => "floor_3_floats",
            Self::floor_4_floats => "floor_4_floats",
            Self::ceil_float => "ceil_float",
            Self::ceil_2_floats => "ceil_2_floats",
            Self::ceil_3_floats => "ceil_3_floats",
            Self::ceil_4_floats => "ceil_4_floats",
            Self::invsqrt_float => "invsqrt_float",
            Self::invsqrt_2_floats => "invsqrt_2_floats",
            Self::invsqrt_3_floats => "invsqrt_3_floats",
            Self::invsqrt_4_floats => "invsqrt_4_floats",
            Self::inverse_mat2 => "inverse_mat2",
            Self::inverse_mat3 => "inverse_mat3",
            Self::inverse_mat4 => "inverse_mat4",
            Self::sin_float => "sin_float",
            Self::cos_float => "cos_float",
            Self::tan_float => "tan_float",
            Self::asin_float => "asin_float",
            Self::acos_float => "acos_float",
            Self::atan_float => "atan_float",
            Self::atan2_n_floats => "atan2_n_floats",
            Self::sqrt_float => "sqrt_float",
            Self::pow_n_floats => "pow_n_floats",
            Self::exp_float => "exp_float",
            Self::exp2_float => "exp2_float",
            Self::log_float => "log_float",
            Self::log2_float => "log2_float",
            Self::refract_4_floats => "refract_4_floats",
            Self::copy_uniform => "copy_uniform",
            Self::copy_2_uniforms => "copy_2_uniforms",
            Self::copy_3_uniforms => "copy_3_uniforms",
            Self::copy_4_uniforms => "copy_4_uniforms",
            Self::copy_constant => "copy_constant",
            Self::splat_2_constants => "splat_2_constants",
            Self::splat_3_constants => "splat_3_constants",
            Self::splat_4_constants => "splat_4_constants",
            Self::copy_slot_masked => "copy_slot_masked",
            Self::copy_2_slots_masked => "copy_2_slots_masked",
            Self::copy_3_slots_masked => "copy_3_slots_masked",
            Self::copy_4_slots_masked => "copy_4_slots_masked",
            Self::copy_from_indirect_unmasked => "copy_from_indirect_unmasked",
            Self::copy_from_indirect_uniform_unmasked => "copy_from_indirect_uniform_unmasked",
            Self::copy_to_indirect_masked => "copy_to_indirect_masked",
            Self::swizzle_copy_to_indirect_masked => "swizzle_copy_to_indirect_masked",
            Self::copy_slot_unmasked => "copy_slot_unmasked",
            Self::copy_2_slots_unmasked => "copy_2_slots_unmasked",
            Self::copy_3_slots_unmasked => "copy_3_slots_unmasked",
            Self::copy_4_slots_unmasked => "copy_4_slots_unmasked",
            Self::copy_immutable_unmasked => "copy_immutable_unmasked",
            Self::copy_2_immutables_unmasked => "copy_2_immutables_unmasked",
            Self::copy_3_immutables_unmasked => "copy_3_immutables_unmasked",
            Self::copy_4_immutables_unmasked => "copy_4_immutables_unmasked",
            Self::swizzle_copy_slot_masked => "swizzle_copy_slot_masked",
            Self::swizzle_copy_2_slots_masked => "swizzle_copy_2_slots_masked",
            Self::swizzle_copy_3_slots_masked => "swizzle_copy_3_slots_masked",
            Self::swizzle_copy_4_slots_masked => "swizzle_copy_4_slots_masked",
            Self::swizzle_1 => "swizzle_1",
            Self::swizzle_2 => "swizzle_2",
            Self::swizzle_3 => "swizzle_3",
            Self::swizzle_4 => "swizzle_4",
            Self::shuffle => "shuffle",
            Self::matrix_multiply_2 => "matrix_multiply_2",
            Self::matrix_multiply_3 => "matrix_multiply_3",
            Self::matrix_multiply_4 => "matrix_multiply_4",
            Self::smoothstep_n_floats => "smoothstep_n_floats",
            Self::dot_2_floats => "dot_2_floats",
            Self::dot_3_floats => "dot_3_floats",
            Self::dot_4_floats => "dot_4_floats",
            Self::add_imm_float => "add_imm_float",
            Self::add_n_floats => "add_n_floats",
            Self::add_float => "add_float",
            Self::add_2_floats => "add_2_floats",
            Self::add_3_floats => "add_3_floats",
            Self::add_4_floats => "add_4_floats",
            Self::add_imm_int => "add_imm_int",
            Self::add_n_ints => "add_n_ints",
            Self::add_int => "add_int",
            Self::add_2_ints => "add_2_ints",
            Self::add_3_ints => "add_3_ints",
            Self::add_4_ints => "add_4_ints",
            Self::sub_n_floats => "sub_n_floats",
            Self::sub_float => "sub_float",
            Self::sub_2_floats => "sub_2_floats",
            Self::sub_3_floats => "sub_3_floats",
            Self::sub_4_floats => "sub_4_floats",
            Self::sub_n_ints => "sub_n_ints",
            Self::sub_int => "sub_int",
            Self::sub_2_ints => "sub_2_ints",
            Self::sub_3_ints => "sub_3_ints",
            Self::sub_4_ints => "sub_4_ints",
            Self::mul_imm_float => "mul_imm_float",
            Self::mul_n_floats => "mul_n_floats",
            Self::mul_float => "mul_float",
            Self::mul_2_floats => "mul_2_floats",
            Self::mul_3_floats => "mul_3_floats",
            Self::mul_4_floats => "mul_4_floats",
            Self::mul_imm_int => "mul_imm_int",
            Self::mul_n_ints => "mul_n_ints",
            Self::mul_int => "mul_int",
            Self::mul_2_ints => "mul_2_ints",
            Self::mul_3_ints => "mul_3_ints",
            Self::mul_4_ints => "mul_4_ints",
            Self::div_n_floats => "div_n_floats",
            Self::div_float => "div_float",
            Self::div_2_floats => "div_2_floats",
            Self::div_3_floats => "div_3_floats",
            Self::div_4_floats => "div_4_floats",
            Self::div_n_ints => "div_n_ints",
            Self::div_int => "div_int",
            Self::div_2_ints => "div_2_ints",
            Self::div_3_ints => "div_3_ints",
            Self::div_4_ints => "div_4_ints",
            Self::div_n_uints => "div_n_uints",
            Self::div_uint => "div_uint",
            Self::div_2_uints => "div_2_uints",
            Self::div_3_uints => "div_3_uints",
            Self::div_4_uints => "div_4_uints",
            Self::max_imm_float => "max_imm_float",
            Self::max_n_floats => "max_n_floats",
            Self::max_float => "max_float",
            Self::max_2_floats => "max_2_floats",
            Self::max_3_floats => "max_3_floats",
            Self::max_4_floats => "max_4_floats",
            Self::max_n_ints => "max_n_ints",
            Self::max_int => "max_int",
            Self::max_2_ints => "max_2_ints",
            Self::max_3_ints => "max_3_ints",
            Self::max_4_ints => "max_4_ints",
            Self::max_n_uints => "max_n_uints",
            Self::max_uint => "max_uint",
            Self::max_2_uints => "max_2_uints",
            Self::max_3_uints => "max_3_uints",
            Self::max_4_uints => "max_4_uints",
            Self::min_imm_float => "min_imm_float",
            Self::min_n_floats => "min_n_floats",
            Self::min_float => "min_float",
            Self::min_2_floats => "min_2_floats",
            Self::min_3_floats => "min_3_floats",
            Self::min_4_floats => "min_4_floats",
            Self::min_n_ints => "min_n_ints",
            Self::min_int => "min_int",
            Self::min_2_ints => "min_2_ints",
            Self::min_3_ints => "min_3_ints",
            Self::min_4_ints => "min_4_ints",
            Self::min_n_uints => "min_n_uints",
            Self::min_uint => "min_uint",
            Self::min_2_uints => "min_2_uints",
            Self::min_3_uints => "min_3_uints",
            Self::min_4_uints => "min_4_uints",
            Self::mod_n_floats => "mod_n_floats",
            Self::mod_float => "mod_float",
            Self::mod_2_floats => "mod_2_floats",
            Self::mod_3_floats => "mod_3_floats",
            Self::mod_4_floats => "mod_4_floats",
            Self::mix_n_floats => "mix_n_floats",
            Self::mix_float => "mix_float",
            Self::mix_2_floats => "mix_2_floats",
            Self::mix_3_floats => "mix_3_floats",
            Self::mix_4_floats => "mix_4_floats",
            Self::mix_n_ints => "mix_n_ints",
            Self::mix_int => "mix_int",
            Self::mix_2_ints => "mix_2_ints",
            Self::mix_3_ints => "mix_3_ints",
            Self::mix_4_ints => "mix_4_ints",
            Self::cmplt_imm_float => "cmplt_imm_float",
            Self::cmplt_n_floats => "cmplt_n_floats",
            Self::cmplt_float => "cmplt_float",
            Self::cmplt_2_floats => "cmplt_2_floats",
            Self::cmplt_3_floats => "cmplt_3_floats",
            Self::cmplt_4_floats => "cmplt_4_floats",
            Self::cmplt_imm_int => "cmplt_imm_int",
            Self::cmplt_n_ints => "cmplt_n_ints",
            Self::cmplt_int => "cmplt_int",
            Self::cmplt_2_ints => "cmplt_2_ints",
            Self::cmplt_3_ints => "cmplt_3_ints",
            Self::cmplt_4_ints => "cmplt_4_ints",
            Self::cmplt_imm_uint => "cmplt_imm_uint",
            Self::cmplt_n_uints => "cmplt_n_uints",
            Self::cmplt_uint => "cmplt_uint",
            Self::cmplt_2_uints => "cmplt_2_uints",
            Self::cmplt_3_uints => "cmplt_3_uints",
            Self::cmplt_4_uints => "cmplt_4_uints",
            Self::cmple_imm_float => "cmple_imm_float",
            Self::cmple_n_floats => "cmple_n_floats",
            Self::cmple_float => "cmple_float",
            Self::cmple_2_floats => "cmple_2_floats",
            Self::cmple_3_floats => "cmple_3_floats",
            Self::cmple_4_floats => "cmple_4_floats",
            Self::cmple_imm_int => "cmple_imm_int",
            Self::cmple_n_ints => "cmple_n_ints",
            Self::cmple_int => "cmple_int",
            Self::cmple_2_ints => "cmple_2_ints",
            Self::cmple_3_ints => "cmple_3_ints",
            Self::cmple_4_ints => "cmple_4_ints",
            Self::cmple_imm_uint => "cmple_imm_uint",
            Self::cmple_n_uints => "cmple_n_uints",
            Self::cmple_uint => "cmple_uint",
            Self::cmple_2_uints => "cmple_2_uints",
            Self::cmple_3_uints => "cmple_3_uints",
            Self::cmple_4_uints => "cmple_4_uints",
            Self::cmpeq_imm_float => "cmpeq_imm_float",
            Self::cmpeq_n_floats => "cmpeq_n_floats",
            Self::cmpeq_float => "cmpeq_float",
            Self::cmpeq_2_floats => "cmpeq_2_floats",
            Self::cmpeq_3_floats => "cmpeq_3_floats",
            Self::cmpeq_4_floats => "cmpeq_4_floats",
            Self::cmpeq_imm_int => "cmpeq_imm_int",
            Self::cmpeq_n_ints => "cmpeq_n_ints",
            Self::cmpeq_int => "cmpeq_int",
            Self::cmpeq_2_ints => "cmpeq_2_ints",
            Self::cmpeq_3_ints => "cmpeq_3_ints",
            Self::cmpeq_4_ints => "cmpeq_4_ints",
            Self::cmpne_imm_float => "cmpne_imm_float",
            Self::cmpne_n_floats => "cmpne_n_floats",
            Self::cmpne_float => "cmpne_float",
            Self::cmpne_2_floats => "cmpne_2_floats",
            Self::cmpne_3_floats => "cmpne_3_floats",
            Self::cmpne_4_floats => "cmpne_4_floats",
            Self::cmpne_imm_int => "cmpne_imm_int",
            Self::cmpne_n_ints => "cmpne_n_ints",
            Self::cmpne_int => "cmpne_int",
            Self::cmpne_2_ints => "cmpne_2_ints",
            Self::cmpne_3_ints => "cmpne_3_ints",
            Self::cmpne_4_ints => "cmpne_4_ints",
            Self::trace_line => "trace_line",
            Self::trace_var => "trace_var",
            Self::trace_enter => "trace_enter",
            Self::trace_exit => "trace_exit",
            Self::trace_scope => "trace_scope",
        }
    }
}

// Source: src/core/SkRasterPipelineOpList.h:12-57
// Actual native macro expansion order; each element is a real function address.
macro_rules! native_lowp_function_table {
    ($ns:ident,$erased:ty) => {
        [
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::move_src_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::move_dst_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_src_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_01 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_a_01 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_gamut as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::premul as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::premul_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::force_opaque as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::force_opaque_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::set_rgb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_rb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_rb_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::black_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::white_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::uniform_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::uniform_color_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::seed_shader as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a8_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_565_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_4444_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_8888_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg88_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_r8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_gray as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::alpha_to_gray_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_red as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_red_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bt709_luminance_or_luma_to_alpha as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bt709_luminance_or_luma_to_rgb as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bilerp_clamp_8888 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_src_a as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_u8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_1_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_native as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_u8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_1_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_native as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstatop as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstin as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstout as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstover as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcatop as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcin as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcout as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcover as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clear as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::modulate as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::multiply as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::plus_ as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::screen as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xor_ as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::darken as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::difference as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::exclusion as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::hardlight as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lighten as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::overlay as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::srcover_rgba_8888 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_translate as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_scale_translate as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_2x3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_perspective as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_x_and_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::check_decal_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mirror_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::repeat_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_x_and_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::evenly_spaced_gradient as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gradient as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::evenly_spaced_2_stop_gradient as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xy_to_unit_angle as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xy_to_radius as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::emboss as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_r as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_g as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_b as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_a as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_r_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_g_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_b_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_a_255 as $ns::StageFn)
            }),
        ]
    };
}
pub(crate) use native_lowp_function_table;

// Source: src/core/SkRasterPipelineOpList.h:12-57
// Source: src/core/SkRasterPipelineOpList.h:162-215
// Source: src/core/SkRasterPipelineOpList.h:65-158
// Actual native macro expansion order; each element is a real function address.
macro_rules! native_highp_function_table {
    ($ns:ident,$erased:ty) => {
        [
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::move_src_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::move_dst_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_src_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_01 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_a_01 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_gamut as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::premul as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::premul_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::force_opaque as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::force_opaque_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::set_rgb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_rb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swap_rb_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::black_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::white_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::uniform_color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::uniform_color_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::seed_shader as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a8_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_a8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_565_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_4444_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_4444 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_8888_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_8888 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg88_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_rg88 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_r8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_gray as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::alpha_to_gray_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_red as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::alpha_to_red_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bt709_luminance_or_luma_to_alpha as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bt709_luminance_or_luma_to_rgb as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bilerp_clamp_8888 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_src_a as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_u8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_1_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::scale_native as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_u8 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_565 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_1_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lerp_native as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstatop as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstin as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstout as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dstover as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcatop as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcin as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcout as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::srcover as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clear as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::modulate as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::multiply as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::plus_ as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::screen as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xor_ as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::darken as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::difference as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::exclusion as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::hardlight as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::lighten as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::overlay as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::srcover_rgba_8888 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_translate as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_scale_translate as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_2x3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_perspective as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::decal_x_and_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::check_decal_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mirror_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::repeat_x_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::clamp_x_and_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::evenly_spaced_gradient as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gradient as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::evenly_spaced_2_stop_gradient as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xy_to_unit_angle as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::xy_to_radius as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::emboss as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_r as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_g as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_b as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_a as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_r_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_g_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_b_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::debug_a_255 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::callback as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::stack_checkpoint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::stack_rewind as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::unbounded_set_rgb as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::unbounded_uniform_color as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::unpremul as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::unpremul_polar as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dither as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_16161616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::load_16161616_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_16161616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_16161616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_a16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_a16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_a16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_r16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_r16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_r16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_r16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg1616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rg1616_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_rg1616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_rg1616 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_f16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_f16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_f16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_f16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_af16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_af16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_af16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_af16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rf16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_rf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_rf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rgf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_rgf16_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_rgf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_rgf16 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_f32 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_f32_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_f32 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_f32 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_1010102 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_1010102_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_1010102 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_1010102 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_1010102_xr as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::load_1010102_xr_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_1010102_xr as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::gather_1010102_xr as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_10x6 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_10x6_dst as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_10x6 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gather_10x6 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::gather_10101010_xr as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_10101010_xr as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::load_10101010_xr_dst as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::store_10101010_xr as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_src_rg as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_src_rg as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::byte_tables as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::colorburn as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::colordodge as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::softlight as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::hue as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::saturation as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::color as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::luminosity as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_3x3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_3x4 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_4x5 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::matrix_4x3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::parametric as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gamma_ as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::PQish as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::HLGish as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::HLGinvish as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::ootf as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::rgb_to_hsl as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::hsl_to_rgb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::css_lab_to_xyz as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::css_oklab_to_linear_srgb as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::css_oklab_gamut_map_to_linear_srgb as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::css_hcl_to_lab as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::css_hsl_to_srgb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::css_hwb_to_srgb as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::gauss_a_to_rgba as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mirror_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::repeat_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mirror_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::repeat_y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::negate_x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bilerp_clamp_8888_force_highp as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bicubic_clamp_8888 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bilinear_setup as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bilinear_nx as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bilinear_px as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bilinear_ny as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bilinear_py as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_setup as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_n3x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_n1x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_p1x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_p3x as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_n3y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_n1y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_p1y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bicubic_p3y as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::accumulate as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::perlin_noise as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mipmap_linear_init as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mipmap_linear_update as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mipmap_linear_finish as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::xy_to_2pt_conical_strip as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::xy_to_2pt_conical_focal_on_circle as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::xy_to_2pt_conical_well_behaved as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::xy_to_2pt_conical_smaller as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::xy_to_2pt_conical_greater as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::alter_2pt_conical_compensate_focal as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::alter_2pt_conical_unswap as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mask_2pt_conical_nan as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mask_2pt_conical_degenerates as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::apply_vector_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::set_base_pointer as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::init_lane_masks as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::store_device_xy01 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::exchange_src as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::load_condition_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::store_condition_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::merge_condition_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::merge_inv_condition_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_loop_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::store_loop_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mask_off_loop_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::reenable_loop_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::merge_loop_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::case_op as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::continue_op as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::load_return_mask as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::store_return_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::mask_off_return_mask as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::branch_if_all_lanes_active as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::branch_if_any_lanes_active as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::branch_if_no_lanes_active as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::branch_if_no_active_lanes_eq as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::jump as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_imm_4_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_imm_3_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_imm_2_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_imm_int as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_n_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bitwise_and_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_2_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_3_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_and_4_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_or_n_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bitwise_or_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_or_2_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_or_3_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_or_4_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_xor_imm_int as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_xor_n_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::bitwise_xor_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_xor_2_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_xor_3_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::bitwise_xor_4_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_int as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_2_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_3_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_4_ints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_uint as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_2_uints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_3_uints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_float_from_4_uints as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_int_from_float as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_int_from_2_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_int_from_3_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_int_from_4_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_uint_from_float as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_uint_from_2_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_uint_from_3_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::cast_to_uint_from_4_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::abs_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::abs_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::abs_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::abs_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::floor_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::floor_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::floor_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::floor_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::ceil_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::ceil_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::ceil_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::ceil_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::invsqrt_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::invsqrt_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::invsqrt_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::invsqrt_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::inverse_mat2 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::inverse_mat3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::inverse_mat4 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sin_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cos_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::tan_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::asin_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::acos_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::atan_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::atan2_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sqrt_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::pow_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::exp_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::exp2_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::log_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::log2_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::refract_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_uniform as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_2_uniforms as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_3_uniforms as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_4_uniforms as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_constant as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::splat_2_constants as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::splat_3_constants as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::splat_4_constants as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::copy_slot_masked as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_2_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_3_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_4_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_from_indirect_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_from_indirect_uniform_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_to_indirect_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::swizzle_copy_to_indirect_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_slot_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_2_slots_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_3_slots_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_4_slots_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_immutable_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_2_immutables_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_3_immutables_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::copy_4_immutables_unmasked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::swizzle_copy_slot_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::swizzle_copy_2_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::swizzle_copy_3_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::swizzle_copy_4_slots_masked as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle_1 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle_2 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle_3 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::swizzle_4 as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::shuffle as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_multiply_2 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_multiply_3 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::matrix_multiply_4 as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>(
                    $ns::smoothstep_n_floats as $ns::StageFn,
                )
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dot_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dot_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::dot_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::add_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::sub_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mul_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_n_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_2_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_3_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::div_4_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_n_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_2_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_3_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::max_4_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_n_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_2_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_3_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::min_4_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mod_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mod_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mod_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mod_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mod_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::mix_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_imm_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_n_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_2_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_3_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmplt_4_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_imm_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_n_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_uint as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_2_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_3_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmple_4_uints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpeq_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_imm_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_n_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_float as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_2_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_3_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_4_floats as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_imm_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_n_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_int as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_2_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_3_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::cmpne_4_ints as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::trace_line as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::trace_var as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::trace_enter as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::trace_exit as $ns::StageFn)
            }),
            Some(unsafe {
                core::mem::transmute::<$ns::StageFn, $erased>($ns::trace_scope as $ns::StageFn)
            }),
        ]
    };
}
pub(crate) use native_highp_function_table;
