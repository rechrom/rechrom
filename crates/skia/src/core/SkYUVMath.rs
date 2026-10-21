// Copyright 2019 Google LLC. BSD-style license.
// Precompiled native tables are retained verbatim by numeric value, not regenerated.
// The non-default Cargo feature mirrors SK_YUV_COLOR_SPACE_HIGH_PRECISION.
// Rust has no hex-float token; from_bits below is an exact literal codec.
use crate::src::core::SkImageInfo::SkYUVColorSpace;
use crate::src::core::SkImageInfo::SkYUVColorSpace::*;
// Source: src/core/SkYUVMath.cpp:19-19
const JPEG_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:20-20
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:21-24
        [
            f32::from_bits(0x3e991687),
            f32::from_bits(0x3f1645a1),
            f32::from_bits(0x3de978d5),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe2cc91a),
            f32::from_bits(0xbea99b72),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbed65e38),
            f32::from_bits(0xbda6871f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:25-25
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:26-29
        [
            0.299000_f32,
            0.587000_f32,
            0.114000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.168736_f32,
            -0.331264_f32,
            0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.418688_f32,
            -0.081312_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:30-31
};
// Source: src/core/SkYUVMath.cpp:32-32
const JPEG_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:33-33
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:34-37
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb26add5d),
            f32::from_bits(0x3fb374bc),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf3428e6),
            f32::from_bits(0x3f800001),
            f32::from_bits(0xbeb032a2),
            f32::from_bits(0xbf36d1a3),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f07fd78),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3fe2d0e5),
            f32::from_bits(0x32390020),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf63b49b),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:38-38
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:39-42
        [
            1.000000_f32,
            -0.000000_f32,
            1.402000_f32,
            0.000000_f32,
            -0.703749_f32,
            1.000000_f32,
            -0.344136_f32,
            -0.714136_f32,
            0.000000_f32,
            0.531211_f32,
            1.000000_f32,
            1.772000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.889475_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:43-44
};
// Source: src/core/SkYUVMath.cpp:45-45
const Rec601_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:46-46
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:47-50
        [
            f32::from_bits(0x3e8379bf),
            f32::from_bits(0x3f010e9f),
            f32::from_bits(0x3dc882e1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbe17c7bf),
            f32::from_bits(0xbe94fd01),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0xbebc4ebf),
            f32::from_bits(0xbd924884),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:51-51
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:52-55
        [
            0.256788_f32,
            0.504129_f32,
            0.097906_f32,
            0.000000_f32,
            0.062745_f32,
            -0.148223_f32,
            -0.290993_f32,
            0.439216_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.367788_f32,
            -0.071427_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:56-57
};
// Source: src/core/SkYUVMath.cpp:58-58
const Rec601_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:59-59
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:60-63
        [
            f32::from_bits(0x3f950a86),
            f32::from_bits(0xb077764c),
            f32::from_bits(0x3fcc4a9b),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf5fcbb8),
            f32::from_bits(0x3f950a86),
            f32::from_bits(0xbec89512),
            f32::from_bits(0xbf501ea7),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f081b63),
            f32::from_bits(0x3f950a86),
            f32::from_bits(0x40011a55),
            f32::from_bits(0x3296f05b),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf8af5f4),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:64-64
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:65-68
        [
            1.164384_f32,
            -0.000000_f32,
            1.596027_f32,
            0.000000_f32,
            -0.874202_f32,
            1.164384_f32,
            -0.391762_f32,
            -0.812968_f32,
            0.000000_f32,
            0.531668_f32,
            1.164384_f32,
            2.017232_f32,
            0.000000_f32,
            0.000000_f32,
            -1.085631_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:69-70
};
// Source: src/core/SkYUVMath.cpp:71-71
const Rec709_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:72-72
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:73-76
        [
            f32::from_bits(0x3e59b3d0),
            f32::from_bits(0x3f371759),
            f32::from_bits(0x3d93dd98),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbdeaa4c7),
            f32::from_bits(0xbec556ce),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3effffff),
            f32::from_bits(0xbee886ba),
            f32::from_bits(0xbd3bca28),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:77-77
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:78-81
        [
            0.212600_f32,
            0.715200_f32,
            0.072200_f32,
            0.000000_f32,
            0.000000_f32,
            -0.114572_f32,
            -0.385428_f32,
            0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.454153_f32,
            -0.045847_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:82-83
};
// Source: src/core/SkYUVMath.cpp:84-84
const Rec709_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:85-85
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:86-89
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb109a688),
            f32::from_bits(0x3fc9930d),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf4a5d6b),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbe3fd1ef),
            f32::from_bits(0xbeefadfc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ea873ef),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3fed844d),
            f32::from_bits(0xb00fec45),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6e72c1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:90-90
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:91-94
        [
            1.000000_f32,
            -0.000000_f32,
            1.574800_f32,
            0.000000_f32,
            -0.790488_f32,
            1.000000_f32,
            -0.187324_f32,
            -0.468124_f32,
            0.000000_f32,
            0.329010_f32,
            1.000000_f32,
            1.855600_f32,
            -0.000000_f32,
            0.000000_f32,
            -0.931439_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:95-96
};
// Source: src/core/SkYUVMath.cpp:97-97
const Rec709_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:98-98
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:99-102
        [
            f32::from_bits(0x3e3af7cb),
            f32::from_bits(0x3f1d3e37),
            f32::from_bits(0x3d7dfb1d),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbdce1e4d),
            f32::from_bits(0xbead594e),
            f32::from_bits(0x3ee0e0e2),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e0),
            f32::from_bits(0xbecc4225),
            f32::from_bits(0xbd24f5d9),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:103-103
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:104-107
        [
            0.182586_f32,
            0.614231_f32,
            0.062007_f32,
            0.000000_f32,
            0.062745_f32,
            -0.100644_f32,
            -0.338572_f32,
            0.439216_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.398942_f32,
            -0.040274_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:108-109
};
// Source: src/core/SkYUVMath.cpp:110-110
const Rec709_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:111-111
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:112-115
        [
            f32::from_bits(0x3f950a86),
            f32::from_bits(0xb1a995ba),
            f32::from_bits(0x3fe5788b),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf7912ef),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xbe5a5dd7),
            f32::from_bits(0xbf086cbf),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3e9a5bf0),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x40073197),
            f32::from_bits(0xb2b2aaa5),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf911353),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:116-116
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:117-120
        [
            1.164384_f32,
            -0.000000_f32,
            1.792741_f32,
            0.000000_f32,
            -0.972945_f32,
            1.164384_f32,
            -0.213249_f32,
            -0.532909_f32,
            0.000000_f32,
            0.301483_f32,
            1.164384_f32,
            2.112402_f32,
            -0.000000_f32,
            0.000000_f32,
            -1.133402_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:121-122
};
// Source: src/core/SkYUVMath.cpp:123-123
const BT2020_8bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:124-124
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:125-128
        [
            f32::from_bits(0x3e86809d),
            f32::from_bits(0x3f2d9169),
            f32::from_bits(0x3d72e48f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe0efb2e),
            f32::from_bits(0xbeb88269),
            f32::from_bits(0x3effffff),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbeeb6909),
            f32::from_bits(0xbd24b7bf),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:129-129
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:130-133
        [
            0.262700_f32,
            0.678000_f32,
            0.059300_f32,
            0.000000_f32,
            0.000000_f32,
            -0.139630_f32,
            -0.360370_f32,
            0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.459786_f32,
            -0.040214_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:134-135
};
// Source: src/core/SkYUVMath.cpp:136-136
const BT2020_8bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:137-137
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:138-141
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb08d8b41),
            f32::from_bits(0x3fbcbfb1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf3d7d2f),
            f32::from_bits(0x3f7fffff),
            f32::from_bits(0xbe28809e),
            f32::from_bits(0xbf124432),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ebd217c),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3ff0d1b8),
            f32::from_bits(0xb2a8966a),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf71c37c),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:142-142
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:143-146
        [
            1.000000_f32,
            -0.000000_f32,
            1.474600_f32,
            0.000000_f32,
            -0.740191_f32,
            1.000000_f32,
            -0.164553_f32,
            -0.571353_f32,
            0.000000_f32,
            0.369396_f32,
            1.000000_f32,
            1.881400_f32,
            -0.000000_f32,
            0.000000_f32,
            -0.944389_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:147-148
};
// Source: src/core/SkYUVMath.cpp:149-149
const BT2020_8bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:150-150
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:151-154
        [
            f32::from_bits(0x3e670714),
            f32::from_bits(0x3f151075),
            f32::from_bits(0x3d509a21),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbdfb32c4),
            f32::from_bits(0xbea21430),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e2),
            f32::from_bits(0xbececab3),
            f32::from_bits(0xbd10b179),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:155-155
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:156-159
        [
            0.225613_f32,
            0.582282_f32,
            0.050928_f32,
            0.000000_f32,
            0.062745_f32,
            -0.122655_f32,
            -0.316560_f32,
            0.439216_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.403890_f32,
            -0.035326_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:160-161
};
// Source: src/core/SkYUVMath.cpp:162-162
const BT2020_8bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:163-163
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:164-167
        [
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xb2117598),
            f32::from_bits(0x3fd6deca),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6a6a86),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xbe3fd26a),
            f32::from_bits(0xbf268234),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3eb1e614),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x400912cc),
            f32::from_bits(0xb2b9e9c6),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf92f66b),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:168-168
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:169-172
        [
            1.164384_f32,
            -0.000000_f32,
            1.678674_f32,
            0.000000_f32,
            -0.915688_f32,
            1.164384_f32,
            -0.187326_f32,
            -0.650424_f32,
            0.000000_f32,
            0.347458_f32,
            1.164384_f32,
            2.141772_f32,
            -0.000000_f32,
            0.000000_f32,
            -1.148145_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:173-174
};
// Source: src/core/SkYUVMath.cpp:175-175
const BT2020_10bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:176-176
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:177-180
        [
            f32::from_bits(0x3e86809d),
            f32::from_bits(0x3f2d9169),
            f32::from_bits(0x3d72e48f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe0efb2e),
            f32::from_bits(0xbeb88269),
            f32::from_bits(0x3effffff),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbeeb6909),
            f32::from_bits(0xbd24b7bf),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:181-181
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:182-185
        [
            0.262700_f32,
            0.678000_f32,
            0.059300_f32,
            0.000000_f32,
            0.000000_f32,
            -0.139630_f32,
            -0.360370_f32,
            0.500000_f32,
            0.000000_f32,
            0.500489_f32,
            0.500000_f32,
            -0.459786_f32,
            -0.040214_f32,
            0.000000_f32,
            0.500489_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:186-187
};
// Source: src/core/SkYUVMath.cpp:188-188
const BT2020_10bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:189-189
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:190-193
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb08d8b41),
            f32::from_bits(0x3fbcbfb1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf3ceeed),
            f32::from_bits(0x3f7fffff),
            f32::from_bits(0xbe28809e),
            f32::from_bits(0xbf124432),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ebc937e),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3ff0d1b8),
            f32::from_bits(0xb2a8966a),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf710dfb),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:194-194
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:195-198
        [
            1.000000_f32,
            -0.000000_f32,
            1.474600_f32,
            0.000000_f32,
            -0.738021_f32,
            1.000000_f32,
            -0.164553_f32,
            -0.571353_f32,
            0.000000_f32,
            0.368313_f32,
            1.000000_f32,
            1.881400_f32,
            -0.000000_f32,
            0.000000_f32,
            -0.941620_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:199-200
};
// Source: src/core/SkYUVMath.cpp:201-201
const BT2020_10bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:202-202
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:203-206
        [
            f32::from_bits(0x3e6659a3),
            f32::from_bits(0x3f14a08d),
            f32::from_bits(0x3d4ffd86),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d802008),
            f32::from_bits(0xbdfa762e),
            f32::from_bits(0xbea19a82),
            f32::from_bits(0x3ee0380d),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x3ee0380f),
            f32::from_bits(0xbece2f74),
            f32::from_bits(0xbd1044d8),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:207-207
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:208-211
        [
            0.224951_f32,
            0.580575_f32,
            0.050779_f32,
            0.000000_f32,
            0.062561_f32,
            -0.122296_f32,
            -0.315632_f32,
            0.437928_f32,
            0.000000_f32,
            0.500489_f32,
            0.437928_f32,
            -0.402706_f32,
            -0.035222_f32,
            0.000000_f32,
            0.500489_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:212-213
};
// Source: src/core/SkYUVMath.cpp:214-214
const BT2020_10bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:215-215
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:216-219
        [
            f32::from_bits(0x3f957abd),
            f32::from_bits(0xb2320891),
            f32::from_bits(0x3fd78093),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6a6a85),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0xbe4062d9),
            f32::from_bits(0xbf26ff93),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3eb1e613),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0x40097a02),
            f32::from_bits(0xaf11f2c1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf92f66c),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:220-220
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:221-224
        [
            1.167808_f32,
            -0.000000_f32,
            1.683611_f32,
            0.000000_f32,
            -0.915688_f32,
            1.167808_f32,
            -0.187877_f32,
            -0.652337_f32,
            0.000000_f32,
            0.347458_f32,
            1.167808_f32,
            2.148072_f32,
            -0.000000_f32,
            0.000000_f32,
            -1.148145_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:225-226
};
// Source: src/core/SkYUVMath.cpp:227-227
const BT2020_12bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:228-228
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:229-232
        [
            f32::from_bits(0x3e86809d),
            f32::from_bits(0x3f2d9169),
            f32::from_bits(0x3d72e48f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe0efb2e),
            f32::from_bits(0xbeb88269),
            f32::from_bits(0x3effffff),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbeeb6909),
            f32::from_bits(0xbd24b7bf),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:233-233
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:234-237
        [
            0.262700_f32,
            0.678000_f32,
            0.059300_f32,
            0.000000_f32,
            0.000000_f32,
            -0.139630_f32,
            -0.360370_f32,
            0.500000_f32,
            0.000000_f32,
            0.500122_f32,
            0.500000_f32,
            -0.459786_f32,
            -0.040214_f32,
            0.000000_f32,
            0.500122_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:238-239
};
// Source: src/core/SkYUVMath.cpp:240-240
const BT2020_12bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:241-241
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:242-245
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb08d8b41),
            f32::from_bits(0x3fbcbfb1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf3ccb7f),
            f32::from_bits(0x3f7fffff),
            f32::from_bits(0xbe28809e),
            f32::from_bits(0xbf124432),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ebc7021),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3ff0d1b8),
            f32::from_bits(0xb2a8966a),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf70e0c7),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:246-246
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:247-250
        [
            1.000000_f32,
            -0.000000_f32,
            1.474600_f32,
            0.000000_f32,
            -0.737480_f32,
            1.000000_f32,
            -0.164553_f32,
            -0.571353_f32,
            0.000000_f32,
            0.368043_f32,
            1.000000_f32,
            1.881400_f32,
            -0.000000_f32,
            0.000000_f32,
            -0.940930_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:251-252
};
// Source: src/core/SkYUVMath.cpp:253-253
const BT2020_12bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:254-254
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:255-258
        [
            f32::from_bits(0x3e662e70),
            f32::from_bits(0x3f1484ad),
            f32::from_bits(0x3d4fd684),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d800801),
            f32::from_bits(0xbdfa4735),
            f32::from_bits(0xbea17c33),
            f32::from_bits(0x3ee00e00),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x3ee00e02),
            f32::from_bits(0xbece08c9),
            f32::from_bits(0xbd1029ca),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:259-259
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:260-263
        [
            0.224787_f32,
            0.580149_f32,
            0.050742_f32,
            0.000000_f32,
            0.062515_f32,
            -0.122206_f32,
            -0.315401_f32,
            0.437607_f32,
            0.000000_f32,
            0.500122_f32,
            0.437607_f32,
            -0.402411_f32,
            -0.035196_f32,
            0.000000_f32,
            0.500122_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:264-265
};
// Source: src/core/SkYUVMath.cpp:266-266
const BT2020_12bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:267-267
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:268-271
        [
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0xb1f189e8),
            f32::from_bits(0x3fd7a905),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6a6a86),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0xbe4086f5),
            f32::from_bits(0xbf271eeb),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3eb1e614),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0x400993cf),
            f32::from_bits(0xb26d652f),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf92f66c),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:272-272
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:273-276
        [
            1.168664_f32,
            -0.000000_f32,
            1.684846_f32,
            0.000000_f32,
            -0.915688_f32,
            1.168664_f32,
            -0.188015_f32,
            -0.652816_f32,
            0.000000_f32,
            0.347458_f32,
            1.168664_f32,
            2.149647_f32,
            -0.000000_f32,
            0.000000_f32,
            -1.148145_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:277-278
};
// Source: src/core/SkYUVMath.cpp:279-279
const BT2020_16bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:280-280
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:281-284
        [
            f32::from_bits(0x3e86809d),
            f32::from_bits(0x3f2d9169),
            f32::from_bits(0x3d72e48f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe0efb2e),
            f32::from_bits(0xbeb88269),
            f32::from_bits(0x3effffff),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbeeb6909),
            f32::from_bits(0xbd24b7bf),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:285-285
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:286-289
        [
            0.262700_f32,
            0.678000_f32,
            0.059300_f32,
            0.000000_f32,
            0.000000_f32,
            -0.139630_f32,
            -0.360370_f32,
            0.500000_f32,
            0.000000_f32,
            0.500008_f32,
            0.500000_f32,
            -0.459786_f32,
            -0.040214_f32,
            0.000000_f32,
            0.500008_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:290-291
};
// Source: src/core/SkYUVMath.cpp:292-292
const BT2020_16bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:293-293
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:294-297
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xb08d8b41),
            f32::from_bits(0x3fbcbfb1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf3cc06e),
            f32::from_bits(0x3f7fffff),
            f32::from_bits(0xbe28809e),
            f32::from_bits(0xbf124432),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ebc6516),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3ff0d1b8),
            f32::from_bits(0xb2a8966a),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf70d2a9),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:298-298
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:299-302
        [
            1.000000_f32,
            -0.000000_f32,
            1.474600_f32,
            0.000000_f32,
            -0.737311_f32,
            1.000000_f32,
            -0.164553_f32,
            -0.571353_f32,
            0.000000_f32,
            0.367959_f32,
            1.000000_f32,
            1.881400_f32,
            -0.000000_f32,
            0.000000_f32,
            -0.940714_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:303-304
};
// Source: src/core/SkYUVMath.cpp:305-305
const BT2020_16bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:306-306
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:307-310
        [
            f32::from_bits(0x3e6620f3),
            f32::from_bits(0x3f147bf9),
            f32::from_bits(0x3d4fca56),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d800080),
            f32::from_bits(0xbdfa388b),
            f32::from_bits(0xbea172bd),
            f32::from_bits(0x3ee000e0),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x3ee000e0),
            f32::from_bits(0xbecdfcb5),
            f32::from_bits(0xbd102157),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:311-311
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:312-315
        [
            0.224735_f32,
            0.580017_f32,
            0.050730_f32,
            0.000000_f32,
            0.062501_f32,
            -0.122178_f32,
            -0.315329_f32,
            0.437507_f32,
            0.000000_f32,
            0.500008_f32,
            0.437507_f32,
            -0.402319_f32,
            -0.035188_f32,
            0.000000_f32,
            0.500008_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:316-317
};
// Source: src/core/SkYUVMath.cpp:318-318
const BT2020_16bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:319-319
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:320-323
        [
            f32::from_bits(0x3f959f90),
            f32::from_bits(0x302bb8c7),
            f32::from_bits(0x3fd7b5aa),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6a6a86),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0xbe40923d),
            f32::from_bits(0xbf2728b7),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3eb1e614),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0x40099bdf),
            f32::from_bits(0xb2c4675c),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf92f66b),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:324-324
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:325-328
        [
            1.168932_f32,
            0.000000_f32,
            1.685231_f32,
            0.000000_f32,
            -0.915688_f32,
            1.168932_f32,
            -0.188058_f32,
            -0.652965_f32,
            0.000000_f32,
            0.347458_f32,
            1.168932_f32,
            2.150139_f32,
            -0.000000_f32,
            0.000000_f32,
            -1.148145_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:329-330
};
// Source: src/core/SkYUVMath.cpp:331-331
const FCC_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:332-332
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:333-336
        [
            f32::from_bits(0x3e99999a),
            f32::from_bits(0x3f170a3d),
            f32::from_bits(0x3de147ae),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe2c9593),
            f32::from_bits(0xbea9b536),
            f32::from_bits(0x3effffff),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbed7c57c),
            f32::from_bits(0xbda0ea0f),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:337-337
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:338-341
        [
            0.300000_f32,
            0.590000_f32,
            0.110000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.168539_f32,
            -0.331461_f32,
            0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.421429_f32,
            -0.078571_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:342-343
};
// Source: src/core/SkYUVMath.cpp:344-344
const FCC_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:345-345
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:346-349
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0x318979f1),
            f32::from_bits(0x3fb33333),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf33e71b),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbea9ea23),
            f32::from_bits(0xbf363cc0),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f061f08),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3fe3d70b),
            f32::from_bits(0x300de6f9),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf64bbc8),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:350-350
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:351-354
        [
            1.000000_f32,
            0.000000_f32,
            1.400000_f32,
            0.000000_f32,
            -0.702745_f32,
            1.000000_f32,
            -0.331864_f32,
            -0.711864_f32,
            0.000000_f32,
            0.523911_f32,
            1.000000_f32,
            1.780000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.893490_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:355-356
};
// Source: src/core/SkYUVMath.cpp:357-357
const FCC_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:358-358
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:359-362
        [
            f32::from_bits(0x3e83ea51),
            f32::from_bits(0x3f01b77a),
            f32::from_bits(0x3dc179cc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbe179a7b),
            f32::from_bits(0xbe9513a3),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0xbebd8a57),
            f32::from_bits(0xbd8d5a27),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:363-363
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:364-367
        [
            0.257647_f32,
            0.506706_f32,
            0.094471_f32,
            0.000000_f32,
            0.062745_f32,
            -0.148050_f32,
            -0.291165_f32,
            0.439216_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.370196_f32,
            -0.069020_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:368-369
};
// Source: src/core/SkYUVMath.cpp:370-370
const FCC_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:371-371
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:372-375
        [
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xb11d4640),
            f32::from_bits(0x3fcc0000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf5f80d2),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xbec16df7),
            f32::from_bits(0xbf4f7527),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f05fabc),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x4001af8b),
            f32::from_bits(0x32547de6),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf8b8bc0),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:376-376
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:377-380
        [
            1.164384_f32,
            -0.000000_f32,
            1.593750_f32,
            0.000000_f32,
            -0.873059_f32,
            1.164384_f32,
            -0.377792_f32,
            -0.810381_f32,
            0.000000_f32,
            0.523357_f32,
            1.164384_f32,
            2.026339_f32,
            0.000000_f32,
            0.000000_f32,
            -1.090202_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:381-382
};
// Source: src/core/SkYUVMath.cpp:383-383
const SMPTE240_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:384-384
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:385-388
        [
            f32::from_bits(0x3e591687),
            f32::from_bits(0x3f3374bc),
            f32::from_bits(0x3db22d0e),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbdedc63e),
            f32::from_bits(0xbec48e71),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbee3bc6d),
            f32::from_bits(0xbd621c97),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:389-389
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:390-393
        [
            0.212000_f32,
            0.701000_f32,
            0.087000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.116101_f32,
            -0.383899_f32,
            0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.444797_f32,
            -0.055203_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:394-395
};
// Source: src/core/SkYUVMath.cpp:396-396
const SMPTE240_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:397-397
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:398-401
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3169ce2b),
            f32::from_bits(0x3fc9ba5e),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf4a84e4),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbe680f97),
            f32::from_bits(0xbef407cc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3eb4bc89),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3fe9ba5e),
            f32::from_bits(0x31751812),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf6aa504),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:402-402
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:403-406
        [
            1.000000_f32,
            0.000000_f32,
            1.576000_f32,
            0.000000_f32,
            -0.791090_f32,
            1.000000_f32,
            -0.226622_f32,
            -0.476622_f32,
            0.000000_f32,
            0.353001_f32,
            1.000000_f32,
            1.826000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.916580_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:407-408
};
// Source: src/core/SkYUVMath.cpp:409-409
const SMPTE240_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:410-410
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:411-414
        [
            f32::from_bits(0x3e3a70b6),
            f32::from_bits(0x3f1a1efc),
            f32::from_bits(0x3d990591),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbdd0de55),
            f32::from_bits(0xbeaca94c),
            f32::from_bits(0x3ee0e0e2),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0xbec80cec),
            f32::from_bits(0xbd469fa3),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:415-415
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:416-419
        [
            0.182071_f32,
            0.602035_f32,
            0.074718_f32,
            0.000000_f32,
            0.062745_f32,
            -0.101987_f32,
            -0.337229_f32,
            0.439216_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.390724_f32,
            -0.048492_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:420-421
};
// Source: src/core/SkYUVMath.cpp:422-422
const SMPTE240_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:423-423
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:424-427
        [
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xb2019988),
            f32::from_bits(0x3fe5a54e),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf793fdf),
            f32::from_bits(0x3f950a86),
            f32::from_bits(0xbe841696),
            f32::from_bits(0xbf0ae6b9),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3ea857bb),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x40050982),
            f32::from_bits(0x33159beb),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf8ee914),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:428-428
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:429-432
        [
            1.164384_f32,
            -0.000000_f32,
            1.794107_f32,
            0.000000_f32,
            -0.973631_f32,
            1.164384_f32,
            -0.257985_f32,
            -0.542583_f32,
            0.000000_f32,
            0.328794_f32,
            1.164384_f32,
            2.078705_f32,
            0.000000_f32,
            0.000000_f32,
            -1.116488_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:433-434
};
// Source: src/core/SkYUVMath.cpp:435-435
const YDZDX_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:436-436
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:437-440
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000000),
            f32::from_bits(0x3efc8f97),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbefded4a),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:441-441
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:442-445
        [
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.500000_f32,
            0.493283_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            -0.495951_f32,
            0.000000_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:446-447
};
// Source: src/core/SkYUVMath.cpp:448-448
const YDZDX_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:449-449
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:450-453
        [
            f32::from_bits(0x3f7ded4a),
            f32::from_bits(0x80000000),
            f32::from_bits(0x40000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf808081),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f81be33),
            f32::from_bits(0x4001be33),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf824074),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:454-454
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:455-458
        [
            0.991902_f32,
            -0.000000_f32,
            2.000000_f32,
            0.000000_f32,
            -1.003922_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.013617_f32,
            2.027234_f32,
            0.000000_f32,
            0.000000_f32,
            -1.017592_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:459-460
};
// Source: src/core/SkYUVMath.cpp:461-461
const YDZDX_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:462-462
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:463-466
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbee0e0e1),
            f32::from_bits(0x3edddb80),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3ee0e0e1),
            f32::from_bits(0xbedf0eb0),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:467-467
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:468-471
        [
            0.000000_f32,
            0.858824_f32,
            0.000000_f32,
            0.000000_f32,
            0.062745_f32,
            0.000000_f32,
            -0.439216_f32,
            0.433315_f32,
            0.000000_f32,
            0.501961_f32,
            0.439216_f32,
            -0.435659_f32,
            0.000000_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:472-473
};
// Source: src/core/SkYUVMath.cpp:474-474
const YDZDX_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:475-475
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:476-479
        [
            f32::from_bits(0x3f93d58c),
            f32::from_bits(0x80000000),
            f32::from_bits(0x4011b6db),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf9b8fc4),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x3f971211),
            f32::from_bits(0x4013b2ce),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf9dc1b2),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:480-480
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:481-484
        [
            1.154954_f32,
            -0.000000_f32,
            2.276786_f32,
            0.000000_f32,
            -1.215325_f32,
            1.164384_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.073059_f32,
            1.180239_f32,
            2.307788_f32,
            0.000000_f32,
            0.000000_f32,
            -1.232474_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:485-486
};
// Source: src/core/SkYUVMath.cpp:487-487
const GBR_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:488-488
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:489-492
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:493-493
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:494-497
        [
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:498-499
};
// Source: src/core/SkYUVMath.cpp:500-500
const GBR_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:501-501
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:502-505
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x80000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:506-506
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:507-510
        [
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:511-512
};
// Source: src/core/SkYUVMath.cpp:513-513
const GBR_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:514-514
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:515-518
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0x3f5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:519-519
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:520-523
        [
            0.000000_f32,
            0.858824_f32,
            0.000000_f32,
            0.000000_f32,
            0.062745_f32,
            0.000000_f32,
            0.000000_f32,
            0.858824_f32,
            0.000000_f32,
            0.062745_f32,
            0.858824_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            0.062745_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:524-525
};
// Source: src/core/SkYUVMath.cpp:526-526
const GBR_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:527-527
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:528-531
        [
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:532-532
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:533-536
        [
            0.000000_f32,
            0.000000_f32,
            1.164384_f32,
            0.000000_f32,
            -0.073059_f32,
            1.164384_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.073059_f32,
            0.000000_f32,
            1.164384_f32,
            0.000000_f32,
            0.000000_f32,
            -0.073059_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:537-538
};
// Source: src/core/SkYUVMath.cpp:539-539
const YCgCo_8bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:540-540
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:541-544
        [
            f32::from_bits(0x3e800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x3e800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:545-545
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:546-549
        [
            0.250000_f32,
            0.500000_f32,
            0.250000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.250000_f32,
            0.500000_f32,
            -0.250000_f32,
            0.000000_f32,
            0.501961_f32,
            0.500000_f32,
            0.000000_f32,
            -0.500000_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:550-551
};
// Source: src/core/SkYUVMath.cpp:552-552
const YCgCo_8bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:553-553
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:554-557
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf008081),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f808081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:558-558
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:559-562
        [
            1.000000_f32,
            -1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.501961_f32,
            1.000000_f32,
            -1.000000_f32,
            -1.000000_f32,
            0.000000_f32,
            1.003922_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:563-564
};
// Source: src/core/SkYUVMath.cpp:565-565
const YCgCo_8bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:566-566
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:567-570
        [
            f32::from_bits(0x3e5bdbdc),
            f32::from_bits(0x3edbdbdc),
            f32::from_bits(0x3e5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d808081),
            f32::from_bits(0xbe5bdbdc),
            f32::from_bits(0x3edbdbdc),
            f32::from_bits(0xbe5bdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x3edbdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbedbdbdc),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f008081),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:571-571
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:572-575
        [
            0.214706_f32,
            0.429412_f32,
            0.214706_f32,
            0.000000_f32,
            0.062745_f32,
            -0.214706_f32,
            0.429412_f32,
            -0.214706_f32,
            0.000000_f32,
            0.501961_f32,
            0.429412_f32,
            0.000000_f32,
            -0.429412_f32,
            0.000000_f32,
            0.501961_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:576-577
};
// Source: src/core/SkYUVMath.cpp:578-578
const YCgCo_8bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:579-579
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:580-583
        [
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xbf950a85),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf28542b),
            f32::from_bits(0x3f950a85),
            f32::from_bits(0xbf950a85),
            f32::from_bits(0xbf950a85),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f8c4624),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:584-584
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:585-588
        [
            1.164384_f32,
            -1.164384_f32,
            1.164384_f32,
            0.000000_f32,
            -0.073059_f32,
            1.164384_f32,
            1.164384_f32,
            0.000000_f32,
            0.000000_f32,
            -0.657534_f32,
            1.164384_f32,
            -1.164384_f32,
            -1.164384_f32,
            0.000000_f32,
            1.095891_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:589-590
};
// Source: src/core/SkYUVMath.cpp:591-591
const YCgCo_10bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:592-592
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:593-596
        [
            f32::from_bits(0x3e800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x3e800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:597-597
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:598-601
        [
            0.250000_f32,
            0.500000_f32,
            0.250000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.250000_f32,
            0.500000_f32,
            -0.250000_f32,
            0.000000_f32,
            0.500489_f32,
            0.500000_f32,
            0.000000_f32,
            -0.500000_f32,
            0.000000_f32,
            0.500489_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:602-603
};
// Source: src/core/SkYUVMath.cpp:604-604
const YCgCo_10bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:605-605
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:606-609
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf002008),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f802008),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:610-610
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:611-614
        [
            1.000000_f32,
            -1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.500489_f32,
            1.000000_f32,
            -1.000000_f32,
            -1.000000_f32,
            0.000000_f32,
            1.000978_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:615-616
};
// Source: src/core/SkYUVMath.cpp:617-617
const YCgCo_10bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:618-618
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:619-622
        [
            f32::from_bits(0x3e5b36ce),
            f32::from_bits(0x3edb36ce),
            f32::from_bits(0x3e5b36ce),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d802008),
            f32::from_bits(0xbe5b36ce),
            f32::from_bits(0x3edb36ce),
            f32::from_bits(0xbe5b36ce),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x3edb36ce),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbedb36ce),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f002008),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:623-623
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:624-627
        [
            0.214076_f32,
            0.428153_f32,
            0.214076_f32,
            0.000000_f32,
            0.062561_f32,
            -0.214076_f32,
            0.428153_f32,
            -0.214076_f32,
            0.000000_f32,
            0.500489_f32,
            0.428153_f32,
            0.000000_f32,
            -0.428153_f32,
            0.000000_f32,
            0.500489_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:628-629
};
// Source: src/core/SkYUVMath.cpp:630-630
const YCgCo_10bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:631-631
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:632-635
        [
            f32::from_bits(0x3f957abd),
            f32::from_bits(0xbf957abd),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a025),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf28542a),
            f32::from_bits(0x3f957abd),
            f32::from_bits(0xbf957abd),
            f32::from_bits(0xbf957abd),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f8c4623),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:636-636
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:637-640
        [
            1.167808_f32,
            -1.167808_f32,
            1.167808_f32,
            0.000000_f32,
            -0.073059_f32,
            1.167808_f32,
            1.167808_f32,
            0.000000_f32,
            0.000000_f32,
            -0.657534_f32,
            1.167808_f32,
            -1.167808_f32,
            -1.167808_f32,
            0.000000_f32,
            1.095890_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:641-642
};
// Source: src/core/SkYUVMath.cpp:643-643
const YCgCo_12bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:644-644
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:645-648
        [
            f32::from_bits(0x3e800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x3e800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:649-649
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:650-653
        [
            0.250000_f32,
            0.500000_f32,
            0.250000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.250000_f32,
            0.500000_f32,
            -0.250000_f32,
            0.000000_f32,
            0.500122_f32,
            0.500000_f32,
            0.000000_f32,
            -0.500000_f32,
            0.000000_f32,
            0.500122_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:654-655
};
// Source: src/core/SkYUVMath.cpp:656-656
const YCgCo_12bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:657-657
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:658-661
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000801),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800801),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:662-662
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:663-666
        [
            1.000000_f32,
            -1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.500122_f32,
            1.000000_f32,
            -1.000000_f32,
            -1.000000_f32,
            0.000000_f32,
            1.000244_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:667-668
};
// Source: src/core/SkYUVMath.cpp:669-669
const YCgCo_12bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:670-670
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:671-674
        [
            f32::from_bits(0x3e5b0db1),
            f32::from_bits(0x3edb0db1),
            f32::from_bits(0x3e5b0db1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d800801),
            f32::from_bits(0xbe5b0db1),
            f32::from_bits(0x3edb0db1),
            f32::from_bits(0xbe5b0db1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x3edb0db1),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbedb0db1),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000801),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:675-675
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:676-679
        [
            0.213919_f32,
            0.427839_f32,
            0.213919_f32,
            0.000000_f32,
            0.062515_f32,
            -0.213919_f32,
            0.427839_f32,
            -0.213919_f32,
            0.000000_f32,
            0.500122_f32,
            0.427839_f32,
            0.000000_f32,
            -0.427839_f32,
            0.000000_f32,
            0.500122_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:680-681
};
// Source: src/core/SkYUVMath.cpp:682-682
const YCgCo_12bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:683-683
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:684-687
        [
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0xbf9596cb),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a026),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf28542b),
            f32::from_bits(0x3f9596cb),
            f32::from_bits(0xbf9596cb),
            f32::from_bits(0xbf9596cb),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f8c4624),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:688-688
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:689-692
        [
            1.168664_f32,
            -1.168664_f32,
            1.168664_f32,
            0.000000_f32,
            -0.073059_f32,
            1.168664_f32,
            1.168664_f32,
            0.000000_f32,
            0.000000_f32,
            -0.657534_f32,
            1.168664_f32,
            -1.168664_f32,
            -1.168664_f32,
            0.000000_f32,
            1.095891_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:693-694
};
// Source: src/core/SkYUVMath.cpp:695-695
const YCgCo_16bit_full_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:696-696
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:697-700
        [
            f32::from_bits(0x3e800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x3e800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x3f000000),
            f32::from_bits(0xbe800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x3f000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:701-701
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:702-705
        [
            0.250000_f32,
            0.500000_f32,
            0.250000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.250000_f32,
            0.500000_f32,
            -0.250000_f32,
            0.000000_f32,
            0.500008_f32,
            0.500000_f32,
            0.000000_f32,
            -0.500000_f32,
            0.000000_f32,
            0.500008_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:706-707
};
// Source: src/core/SkYUVMath.cpp:708-708
const YCgCo_16bit_full_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:709-709
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:710-713
        [
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf000080),
            f32::from_bits(0x3f800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0xbf800000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800080),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:714-714
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:715-718
        [
            1.000000_f32,
            -1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            1.000000_f32,
            0.000000_f32,
            0.000000_f32,
            -0.500008_f32,
            1.000000_f32,
            -1.000000_f32,
            -1.000000_f32,
            0.000000_f32,
            1.000015_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:719-720
};
// Source: src/core/SkYUVMath.cpp:721-721
const YCgCo_16bit_limited_rgb_to_yuv: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:722-722
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:723-726
        [
            f32::from_bits(0x3e5b00db),
            f32::from_bits(0x3edb00db),
            f32::from_bits(0x3e5b00db),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3d800080),
            f32::from_bits(0xbe5b00db),
            f32::from_bits(0x3edb00db),
            f32::from_bits(0xbe5b00db),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x3edb00db),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbedb00db),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f000080),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:727-727
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:728-731
        [
            0.213870_f32,
            0.427741_f32,
            0.213870_f32,
            0.000000_f32,
            0.062501_f32,
            -0.213870_f32,
            0.427741_f32,
            -0.213870_f32,
            0.000000_f32,
            0.500008_f32,
            0.427741_f32,
            0.000000_f32,
            -0.427741_f32,
            0.000000_f32,
            0.500008_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:732-733
};
// Source: src/core/SkYUVMath.cpp:734-734
const YCgCo_16bit_limited_yuv_to_rgb: [f32; 20] = {
    // Source: src/core/SkYUVMath.cpp:735-735
    #[cfg(feature = "sk_yuv_color_space_high_precision")]
    {
        // Source: src/core/SkYUVMath.cpp:736-739
        [
            f32::from_bits(0x3f959f90),
            f32::from_bits(0xbf959f90),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbd95a025),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0xbf28542a),
            f32::from_bits(0x3f959f90),
            f32::from_bits(0xbf959f90),
            f32::from_bits(0xbf959f90),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f8c4623),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x00000000),
            f32::from_bits(0x3f800000),
            f32::from_bits(0x00000000),
        ]
    }
    // Source: src/core/SkYUVMath.cpp:740-740
    #[cfg(not(feature = "sk_yuv_color_space_high_precision"))]
    {
        // Source: src/core/SkYUVMath.cpp:741-744
        [
            1.168932_f32,
            -1.168932_f32,
            1.168932_f32,
            0.000000_f32,
            -0.073059_f32,
            1.168932_f32,
            1.168932_f32,
            0.000000_f32,
            0.000000_f32,
            -0.657534_f32,
            1.168932_f32,
            -1.168932_f32,
            -1.168932_f32,
            0.000000_f32,
            1.095890_f32,
            0.000000_f32,
            0.000000_f32,
            0.000000_f32,
            1.000000_f32,
            0.000000_f32,
        ]
    }
    // Source: src/core/SkYUVMath.cpp:745-746
};
// Source: src/core/SkYUVMath.cpp:748-748
const _: () = assert!(kJPEG_Full_SkYUVColorSpace as i32 == 0);
// Source: src/core/SkYUVMath.cpp:749-749
const _: () = assert!(kRec601_Limited_SkYUVColorSpace as i32 == 1);
// Source: src/core/SkYUVMath.cpp:750-750
const _: () = assert!(kRec709_Full_SkYUVColorSpace as i32 == 2);
// Source: src/core/SkYUVMath.cpp:751-751
const _: () = assert!(kRec709_Limited_SkYUVColorSpace as i32 == 3);
// Source: src/core/SkYUVMath.cpp:752-752
const _: () = assert!(kBT2020_8bit_Full_SkYUVColorSpace as i32 == 4);
// Source: src/core/SkYUVMath.cpp:753-753
const _: () = assert!(kBT2020_8bit_Limited_SkYUVColorSpace as i32 == 5);
// Source: src/core/SkYUVMath.cpp:754-754
const _: () = assert!(kBT2020_10bit_Full_SkYUVColorSpace as i32 == 6);
// Source: src/core/SkYUVMath.cpp:755-755
const _: () = assert!(kBT2020_10bit_Limited_SkYUVColorSpace as i32 == 7);
// Source: src/core/SkYUVMath.cpp:756-756
const _: () = assert!(kBT2020_12bit_Full_SkYUVColorSpace as i32 == 8);
// Source: src/core/SkYUVMath.cpp:757-757
const _: () = assert!(kBT2020_12bit_Limited_SkYUVColorSpace as i32 == 9);
// Source: src/core/SkYUVMath.cpp:758-758
const _: () = assert!(kBT2020_16bit_Full_SkYUVColorSpace as i32 == 10);
// Source: src/core/SkYUVMath.cpp:759-759
const _: () = assert!(kBT2020_16bit_Limited_SkYUVColorSpace as i32 == 11);
// Source: src/core/SkYUVMath.cpp:760-760
const _: () = assert!(kFCC_Full_SkYUVColorSpace as i32 == 12);
// Source: src/core/SkYUVMath.cpp:761-761
const _: () = assert!(kFCC_Limited_SkYUVColorSpace as i32 == 13);
// Source: src/core/SkYUVMath.cpp:762-762
const _: () = assert!(kSMPTE240_Full_SkYUVColorSpace as i32 == 14);
// Source: src/core/SkYUVMath.cpp:763-763
const _: () = assert!(kSMPTE240_Limited_SkYUVColorSpace as i32 == 15);
// Source: src/core/SkYUVMath.cpp:764-764
const _: () = assert!(kYDZDX_Full_SkYUVColorSpace as i32 == 16);
// Source: src/core/SkYUVMath.cpp:765-765
const _: () = assert!(kYDZDX_Limited_SkYUVColorSpace as i32 == 17);
// Source: src/core/SkYUVMath.cpp:766-766
const _: () = assert!(kGBR_Full_SkYUVColorSpace as i32 == 18);
// Source: src/core/SkYUVMath.cpp:767-767
const _: () = assert!(kGBR_Limited_SkYUVColorSpace as i32 == 19);
// Source: src/core/SkYUVMath.cpp:768-768
const _: () = assert!(kYCgCo_8bit_Full_SkYUVColorSpace as i32 == 20);
// Source: src/core/SkYUVMath.cpp:769-769
const _: () = assert!(kYCgCo_8bit_Limited_SkYUVColorSpace as i32 == 21);
// Source: src/core/SkYUVMath.cpp:770-770
const _: () = assert!(kYCgCo_10bit_Full_SkYUVColorSpace as i32 == 22);
// Source: src/core/SkYUVMath.cpp:771-771
const _: () = assert!(kYCgCo_10bit_Limited_SkYUVColorSpace as i32 == 23);
// Source: src/core/SkYUVMath.cpp:772-772
const _: () = assert!(kYCgCo_12bit_Full_SkYUVColorSpace as i32 == 24);
// Source: src/core/SkYUVMath.cpp:773-773
const _: () = assert!(kYCgCo_12bit_Limited_SkYUVColorSpace as i32 == 25);
// Source: src/core/SkYUVMath.cpp:774-774
const _: () = assert!(kYCgCo_16bit_Full_SkYUVColorSpace as i32 == 26);
// Source: src/core/SkYUVMath.cpp:775-775
const _: () = assert!(kYCgCo_16bit_Limited_SkYUVColorSpace as i32 == 27);
// Source: src/core/SkYUVMath.cpp:777-806
const yuv_to_rgb_array: [&[f32; 20]; 28] = [
    &JPEG_full_yuv_to_rgb,
    &Rec601_limited_yuv_to_rgb,
    &Rec709_full_yuv_to_rgb,
    &Rec709_limited_yuv_to_rgb,
    &BT2020_8bit_full_yuv_to_rgb,
    &BT2020_8bit_limited_yuv_to_rgb,
    &BT2020_10bit_full_yuv_to_rgb,
    &BT2020_10bit_limited_yuv_to_rgb,
    &BT2020_12bit_full_yuv_to_rgb,
    &BT2020_12bit_limited_yuv_to_rgb,
    &BT2020_16bit_full_yuv_to_rgb,
    &BT2020_16bit_limited_yuv_to_rgb,
    &FCC_full_yuv_to_rgb,
    &FCC_limited_yuv_to_rgb,
    &SMPTE240_full_yuv_to_rgb,
    &SMPTE240_limited_yuv_to_rgb,
    &YDZDX_full_yuv_to_rgb,
    &YDZDX_limited_yuv_to_rgb,
    &GBR_full_yuv_to_rgb,
    &GBR_limited_yuv_to_rgb,
    &YCgCo_8bit_full_yuv_to_rgb,
    &YCgCo_8bit_limited_yuv_to_rgb,
    &YCgCo_10bit_full_yuv_to_rgb,
    &YCgCo_10bit_limited_yuv_to_rgb,
    &YCgCo_12bit_full_yuv_to_rgb,
    &YCgCo_12bit_limited_yuv_to_rgb,
    &YCgCo_16bit_full_yuv_to_rgb,
    &YCgCo_16bit_limited_yuv_to_rgb,
];
// Source: src/core/SkYUVMath.cpp:808-837
const rgb_to_yuv_array: [&[f32; 20]; 28] = [
    &JPEG_full_rgb_to_yuv,
    &Rec601_limited_rgb_to_yuv,
    &Rec709_full_rgb_to_yuv,
    &Rec709_limited_rgb_to_yuv,
    &BT2020_8bit_full_rgb_to_yuv,
    &BT2020_8bit_limited_rgb_to_yuv,
    &BT2020_10bit_full_rgb_to_yuv,
    &BT2020_10bit_limited_rgb_to_yuv,
    &BT2020_12bit_full_rgb_to_yuv,
    &BT2020_12bit_limited_rgb_to_yuv,
    &BT2020_16bit_full_rgb_to_yuv,
    &BT2020_16bit_limited_rgb_to_yuv,
    &FCC_full_rgb_to_yuv,
    &FCC_limited_rgb_to_yuv,
    &SMPTE240_full_rgb_to_yuv,
    &SMPTE240_limited_rgb_to_yuv,
    &YDZDX_full_rgb_to_yuv,
    &YDZDX_limited_rgb_to_yuv,
    &GBR_full_rgb_to_yuv,
    &GBR_limited_rgb_to_yuv,
    &YCgCo_8bit_full_rgb_to_yuv,
    &YCgCo_8bit_limited_rgb_to_yuv,
    &YCgCo_10bit_full_rgb_to_yuv,
    &YCgCo_10bit_limited_rgb_to_yuv,
    &YCgCo_12bit_full_rgb_to_yuv,
    &YCgCo_12bit_limited_rgb_to_yuv,
    &YCgCo_16bit_full_rgb_to_yuv,
    &YCgCo_16bit_limited_rgb_to_yuv,
];
// Source: src/core/SkYUVMath.cpp:839-839
const kSizeOfColorMatrix: usize = 20 * core::mem::size_of::<f32>();
// Source: src/core/SkYUVMath.h:13-13
// Source: src/core/SkYUVMath.cpp:841-848
pub fn SkColorMatrix_RGB2YUV(cs: SkYUVColorSpace, m: &mut [f32; 20]) {
    if (cs as u32) < kIdentity_SkYUVColorSpace as u32 {
        m.copy_from_slice(rgb_to_yuv_array[cs as u32 as usize]);
    } else {
        m.fill(0.);
        m[18] = 1.;
        m[12] = 1.;
        m[6] = 1.;
        m[0] = 1.;
    }
}
// Source: src/core/SkYUVMath.h:14-14
// Source: src/core/SkYUVMath.cpp:850-857
pub fn SkColorMatrix_YUV2RGB(cs: SkYUVColorSpace, m: &mut [f32; 20]) {
    if (cs as u32) < kIdentity_SkYUVColorSpace as u32 {
        m.copy_from_slice(yuv_to_rgb_array[cs as u32 as usize]);
    } else {
        m.fill(0.);
        m[18] = 1.;
        m[12] = 1.;
        m[6] = 1.;
        m[0] = 1.;
    }
}
// cpp860-1165 and header17 are offline table regeneration only; excluded by
// the current production CPU scope. No M44/logging substitute is introduced.
