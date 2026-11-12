// Copyright 2006-2008 the V8 project authors. All rights reserved.
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials provided
//       with the distribution.
//     * Neither the name of Google Inc. nor the names of its
//       contributors may be used to endorse or promote products derived
//       from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: cached-powers.cc:40-173. Exact cached significands from Chromium.
use super::diy_fp::DiyFp;
const kCachedPowers: &[(u64, i32, i32)] = &[
    (0xfa8fd5a0_081c0288, -1220, -348),
    (0xbaaee17f_a23ebf76, -1193, -340),
    (0x8b16fb20_3055ac76, -1166, -332),
    (0xcf42894a_5dce35ea, -1140, -324),
    (0x9a6bb0aa_55653b2d, -1113, -316),
    (0xe61acf03_3d1a45df, -1087, -308),
    (0xab70fe17_c79ac6ca, -1060, -300),
    (0xff77b1fc_bebcdc4f, -1034, -292),
    (0xbe5691ef_416bd60c, -1007, -284),
    (0x8dd01fad_907ffc3c, -980, -276),
    (0xd3515c28_31559a83, -954, -268),
    (0x9d71ac8f_ada6c9b5, -927, -260),
    (0xea9c2277_23ee8bcb, -901, -252),
    (0xaecc4991_4078536d, -874, -244),
    (0x823c1279_5db6ce57, -847, -236),
    (0xc2109436_4dfb5637, -821, -228),
    (0x9096ea6f_3848984f, -794, -220),
    (0xd77485cb_25823ac7, -768, -212),
    (0xa086cfcd_97bf97f4, -741, -204),
    (0xef340a98_172aace5, -715, -196),
    (0xb23867fb_2a35b28e, -688, -188),
    (0x84c8d4df_d2c63f3b, -661, -180),
    (0xc5dd4427_1ad3cdba, -635, -172),
    (0x936b9fce_bb25c996, -608, -164),
    (0xdbac6c24_7d62a584, -582, -156),
    (0xa3ab6658_0d5fdaf6, -555, -148),
    (0xf3e2f893_dec3f126, -529, -140),
    (0xb5b5ada8_aaff80b8, -502, -132),
    (0x87625f05_6c7c4a8b, -475, -124),
    (0xc9bcff60_34c13053, -449, -116),
    (0x964e858c_91ba2655, -422, -108),
    (0xdff97724_70297ebd, -396, -100),
    (0xa6dfbd9f_b8e5b88f, -369, -92),
    (0xf8a95fcf_88747d94, -343, -84),
    (0xb9447093_8fa89bcf, -316, -76),
    (0x8a08f0f8_bf0f156b, -289, -68),
    (0xcdb02555_653131b6, -263, -60),
    (0x993fe2c6_d07b7fac, -236, -52),
    (0xe45c10c4_2a2b3b06, -210, -44),
    (0xaa242499_697392d3, -183, -36),
    (0xfd87b5f2_8300ca0e, -157, -28),
    (0xbce50864_92111aeb, -130, -20),
    (0x8cbccc09_6f5088cc, -103, -12),
    (0xd1b71758_e219652c, -77, -4),
    (0x9c400000_00000000, -50, 4),
    (0xe8d4a510_00000000, -24, 12),
    (0xad78ebc5_ac620000, 3, 20),
    (0x813f3978_f8940984, 30, 28),
    (0xc097ce7b_c90715b3, 56, 36),
    (0x8f7e32ce_7bea5c70, 83, 44),
    (0xd5d238a4_abe98068, 109, 52),
    (0x9f4f2726_179a2245, 136, 60),
    (0xed63a231_d4c4fb27, 162, 68),
    (0xb0de6538_8cc8ada8, 189, 76),
    (0x83c7088e_1aab65db, 216, 84),
    (0xc45d1df9_42711d9a, 242, 92),
    (0x924d692c_a61be758, 269, 100),
    (0xda01ee64_1a708dea, 295, 108),
    (0xa26da399_9aef774a, 322, 116),
    (0xf209787b_b47d6b85, 348, 124),
    (0xb454e4a1_79dd1877, 375, 132),
    (0x865b8692_5b9bc5c2, 402, 140),
    (0xc83553c5_c8965d3d, 428, 148),
    (0x952ab45c_fa97a0b3, 455, 156),
    (0xde469fbd_99a05fe3, 481, 164),
    (0xa59bc234_db398c25, 508, 172),
    (0xf6c69a72_a3989f5c, 534, 180),
    (0xb7dcbf53_54e9bece, 561, 188),
    (0x88fcf317_f22241e2, 588, 196),
    (0xcc20ce9b_d35c78a5, 614, 204),
    (0x98165af3_7b2153df, 641, 212),
    (0xe2a0b5dc_971f303a, 667, 220),
    (0xa8d9d153_5ce3b396, 694, 228),
    (0xfb9b7cd9_a4a7443c, 720, 236),
    (0xbb764c4c_a7a44410, 747, 244),
    (0x8bab8eef_b6409c1a, 774, 252),
    (0xd01fef10_a657842c, 800, 260),
    (0x9b10a4e5_e9913129, 827, 268),
    (0xe7109bfb_a19c0c9d, 853, 276),
    (0xac2820d9_623bf429, 880, 284),
    (0x80444b5e_7aa7cf85, 907, 292),
    (0xbf21e440_03acdd2d, 933, 300),
    (0x8e679c2f_5e44ff8f, 960, 308),
    (0xd433179d_9c8cb841, 986, 316),
    (0x9e19db92_b4e31ba9, 1013, 324),
    (0xeb96bf6e_badf77d9, 1039, 332),
    (0xaf87023b_9bf0ee6b, 1066, 340),
];
pub(super) fn GetCachedPowerForBinaryExponentRange(min: i32, max: i32) -> (DiyFp, i32) {
    let k = ((min + 64 - 1) as f64 * 0.30102999566398114).ceil() as i32;
    let index = ((348 + k - 1) / 8 + 1) as usize;
    let (f, e, decimal) = kCachedPowers[index];
    debug_assert!(min <= e && e <= max);
    (DiyFp::new(f, e), decimal)
}
pub(super) fn GetCachedPowerForDecimalExponent(requested: i32) -> (DiyFp, i32) {
    debug_assert!((-348..348).contains(&requested));
    let index = ((requested + 348) / 8) as usize;
    let (f, e, decimal) = kCachedPowers[index];
    debug_assert!(decimal <= requested && requested < decimal + 8);
    (DiyFp::new(f, e), decimal)
}
