use crate::modules::skcms::src::skcms_public::{skcms_Matrix3x3, skcms_TransferFunction};
use std::sync::{Arc, OnceLock};
// Copyright 2016 Google Inc. BSD-style license.
// Source: include/core/SkColorSpace.h:25-33
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkColorSpacePrimaries {
    pub fRX: f32,
    pub fRY: f32,
    pub fGX: f32,
    pub fGY: f32,
    pub fBX: f32,
    pub fBY: f32,
    pub fWX: f32,
    pub fWY: f32,
}
// Source: include/core/SkColorSpace.h:41-45
impl PartialEq for SkColorSpacePrimaries {
    fn eq(&self, other: &Self) -> bool {
        self.fRX == other.fRX
            && self.fRY == other.fRY
            && self.fGX == other.fGX
            && self.fGY == other.fGY
            && self.fBX == other.fBX
            && self.fBY == other.fBY
            && self.fWX == other.fWX
            && self.fWY == other.fWY
    }
}
// Source: src/core/SkColorSpace.cpp:18-25
// Source: src/core/SkColorSpace.cpp:27-28
fn xyz_almost_equal(a: &skcms_Matrix3x3, b: &skcms_Matrix3x3) -> bool {
    for r in 0..3 {
        for c in 0..3 {
            if !super::SkColorSpacePriv::color_space_almost_equal(a.vals[r][c], b.vals[r][c]) {
                return false;
            }
        }
    }
    true
}
pub mod SkNamedPrimaries {
    use super::SkColorSpacePrimaries;
    // Source: include/core/SkColorSpace.h:58-59
    pub const kRec709: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.64_f32,
        fRY: 0.33_f32,
        fGX: 0.3_f32,
        fGY: 0.6_f32,
        fBX: 0.15_f32,
        fBY: 0.06_f32,
        fWX: 0.3127_f32,
        fWY: 0.329_f32,
    };
    // Source: include/core/SkColorSpace.h:62-63
    pub const kRec470SystemM: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.67_f32,
        fRY: 0.33_f32,
        fGX: 0.21_f32,
        fGY: 0.71_f32,
        fBX: 0.14_f32,
        fBY: 0.08_f32,
        fWX: 0.31_f32,
        fWY: 0.316_f32,
    };
    // Source: include/core/SkColorSpace.h:66-67
    pub const kRec470SystemBG: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.64_f32,
        fRY: 0.33_f32,
        fGX: 0.29_f32,
        fGY: 0.60_f32,
        fBX: 0.15_f32,
        fBY: 0.06_f32,
        fWX: 0.3127_f32,
        fWY: 0.3290_f32,
    };
    // Source: include/core/SkColorSpace.h:70-71
    pub const kRec601: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.630_f32,
        fRY: 0.340_f32,
        fGX: 0.310_f32,
        fGY: 0.595_f32,
        fBX: 0.155_f32,
        fBY: 0.070_f32,
        fWX: 0.3127_f32,
        fWY: 0.3290_f32,
    };
    // Source: include/core/SkColorSpace.h:74-74
    pub const kSMPTE_ST_240: SkColorSpacePrimaries = kRec601;
    // Source: include/core/SkColorSpace.h:77-78
    pub const kGenericFilm: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.681_f32,
        fRY: 0.319_f32,
        fGX: 0.243_f32,
        fGY: 0.692_f32,
        fBX: 0.145_f32,
        fBY: 0.049_f32,
        fWX: 0.310_f32,
        fWY: 0.316_f32,
    };
    // Source: include/core/SkColorSpace.h:81-82
    pub const kRec2020: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.708_f32,
        fRY: 0.292_f32,
        fGX: 0.170_f32,
        fGY: 0.797_f32,
        fBX: 0.131_f32,
        fBY: 0.046_f32,
        fWX: 0.3127_f32,
        fWY: 0.3290_f32,
    };
    // Source: include/core/SkColorSpace.h:85-86
    pub const kSMPTE_ST_428_1: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 1.0_f32,
        fRY: 0.0_f32,
        fGX: 0.0_f32,
        fGY: 1.0_f32,
        fBX: 0.0_f32,
        fBY: 0.0_f32,
        fWX: 1.0_f32 / 3.0_f32,
        fWY: 1.0_f32 / 3.0_f32,
    };
    // Source: include/core/SkColorSpace.h:89-90
    pub const kSMPTE_RP_431_2: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.680_f32,
        fRY: 0.320_f32,
        fGX: 0.265_f32,
        fGY: 0.690_f32,
        fBX: 0.150_f32,
        fBY: 0.060_f32,
        fWX: 0.314_f32,
        fWY: 0.351_f32,
    };
    // Source: include/core/SkColorSpace.h:93-94
    pub const kSMPTE_EG_432_1: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.680_f32,
        fRY: 0.320_f32,
        fGX: 0.265_f32,
        fGY: 0.690_f32,
        fBX: 0.150_f32,
        fBY: 0.060_f32,
        fWX: 0.3127_f32,
        fWY: 0.3290_f32,
    };
    // Source: include/core/SkColorSpace.h:99-100
    pub const kITU_T_H273_Value22: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.630_f32,
        fRY: 0.340_f32,
        fGX: 0.295_f32,
        fGY: 0.605_f32,
        fBX: 0.155_f32,
        fBY: 0.077_f32,
        fWX: 0.3127_f32,
        fWY: 0.3290_f32,
    };
    // Source: include/core/SkColorSpace.h:105-105
    // Source: include/core/SkColorSpace.h:107-107
    // Source: include/core/SkColorSpace.h:110-118
    // Source: include/core/SkColorSpace.h:120-120
    // Source: include/core/SkColorSpace.h:122-122
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CicpId {
        kRec709 = 1,
        kRec470SystemM = 4,
        kRec470SystemBG = 5,
        kRec601 = 6,
        kSMPTE_ST_240 = 7,
        kGenericFilm = 8,
        kRec2020 = 9,
        kSMPTE_ST_428_1 = 10,
        kSMPTE_RP_431_2 = 11,
        kSMPTE_EG_432_1 = 12,
        kITU_T_H273_Value22 = 22,
    }
    // Source: include/core/SkColorSpace.h:125-126
    pub const kProPhotoRGB: SkColorSpacePrimaries = SkColorSpacePrimaries {
        fRX: 0.7347,
        fRY: 0.2653,
        fGX: 0.1596,
        fGY: 0.8404,
        fBX: 0.0366,
        fBY: 0.0001,
        fWX: 0.34567,
        fWY: 0.35850,
    };
    // Source: src/core/SkColorSpace.cpp:33-50
    struct TableEntry {
        cicp_id: CicpId,
        sk_primaries: SkColorSpacePrimaries,
        to_xyzd50: Option<&'static super::skcms_Matrix3x3>,
    }
    const CICP_TABLE: &[TableEntry] = &[
        TableEntry {
            cicp_id: CicpId::kRec709,
            sk_primaries: kRec709,
            to_xyzd50: Some(&super::SkNamedGamut::kSRGB),
        },
        TableEntry {
            cicp_id: CicpId::kRec470SystemM,
            sk_primaries: kRec470SystemM,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kRec470SystemBG,
            sk_primaries: kRec470SystemBG,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kRec601,
            sk_primaries: kRec601,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_ST_240,
            sk_primaries: kSMPTE_ST_240,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kGenericFilm,
            sk_primaries: kGenericFilm,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kRec2020,
            sk_primaries: kRec2020,
            to_xyzd50: Some(&super::SkNamedGamut::kRec2020),
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_ST_428_1,
            sk_primaries: kSMPTE_ST_428_1,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_RP_431_2,
            sk_primaries: kSMPTE_RP_431_2,
            to_xyzd50: None,
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_EG_432_1,
            sk_primaries: kSMPTE_EG_432_1,
            to_xyzd50: Some(&super::SkNamedGamut::kDisplayP3),
        },
        TableEntry {
            cicp_id: CicpId::kITU_T_H273_Value22,
            sk_primaries: kITU_T_H273_Value22,
            to_xyzd50: None,
        },
    ];
    // Source: src/core/SkColorSpace.cpp:53-53
    pub const kCicpIdApplicationDefined: u8 = 2;
    // Source: src/core/SkColorSpace.cpp:55-69
    // Source: src/core/SkColorSpace.cpp:71-82
    pub fn GetCicp(primaries: CicpId, toXYZD50: &mut super::skcms_Matrix3x3) -> bool {
        GetCicpRaw(primaries as u8, toXYZD50)
    }
    // Rust enum values cannot represent unrecognized native CICP IDs.
    pub(crate) fn GetCicpRaw(primaries: u8, toXYZD50: &mut super::skcms_Matrix3x3) -> bool {
        for table_entry in CICP_TABLE {
            if primaries != table_entry.cicp_id as u8 {
                continue;
            }
            if let Some(m) = table_entry.to_xyzd50 {
                *toXYZD50 = *m;
                return true;
            }
            if table_entry.sk_primaries.toXYZD50(toXYZD50) {
                return true;
            }
        }
        false
    }
    pub fn GetCicpFromMatrix(m: &super::skcms_Matrix3x3, primaries: &mut CicpId) -> bool {
        for table_entry in CICP_TABLE {
            let mut matrix = super::skcms_Matrix3x3 {
                vals: [[0.0; 3]; 3],
            };
            if table_entry.sk_primaries.toXYZD50(&mut matrix) {
                if super::xyz_almost_equal(m, &matrix) {
                    *primaries = table_entry.cicp_id;
                    return true;
                }
            }
        }
        false
    }
}
pub mod SkNamedTransferFn {
    use super::skcms_TransferFunction;
    // Source: include/core/SkColorSpace.h:133-134
    pub const kSRGB: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.4_f32,
        a: (1.0_f64 / 1.055_f64) as f32,
        b: (0.055_f64 / 1.055_f64) as f32,
        c: (1.0_f64 / 12.92_f64) as f32,
        d: 0.04045_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:136-137
    pub const k2Dot2: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.2_f32,
        a: 1.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:139-140
    pub const kRec2020: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.22222_f32,
        a: 0.909672_f32,
        b: 0.0903276_f32,
        c: 0.222222_f32,
        d: 0.0812429_f32,
        e: 0.0,
        f: 0.0,
    };
    // Source: include/core/SkColorSpace.h:152-152
    pub const kRec709: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.4_f32,
        a: 1.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:155-155
    pub const kRec470SystemM: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.2_f32,
        a: 1.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:159-159
    pub const kRec470SystemBG: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.8_f32,
        a: 1.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:162-162
    pub const kRec601: skcms_TransferFunction = kRec709;
    // Source: include/core/SkColorSpace.h:165-166
    pub const kSMPTE_ST_240: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.222222222222_f32,
        a: 0.899626676224_f32,
        b: 0.100373323776_f32,
        c: 0.25_f32,
        d: 0.091286342118_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:169-170
    pub const kLinear: skcms_TransferFunction = skcms_TransferFunction {
        g: 1.0_f32,
        a: 1.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:173-173
    pub const kIEC61966_2_4: skcms_TransferFunction = kRec709;
    // Source: include/core/SkColorSpace.h:176-176
    pub const kIEC61966_2_1: skcms_TransferFunction = kSRGB;
    // Source: include/core/SkColorSpace.h:179-179
    pub const kRec2020_10bit: skcms_TransferFunction = kRec709;
    // Source: include/core/SkColorSpace.h:182-182
    pub const kRec2020_12bit: skcms_TransferFunction = kRec709;
    // Source: include/core/SkColorSpace.h:185-186
    pub const kPQ: skcms_TransferFunction = skcms_TransferFunction {
        g: -5.0_f32,
        a: 203.0_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:189-190
    pub const kSMPTE_ST_428_1: skcms_TransferFunction = skcms_TransferFunction {
        g: 2.6_f32,
        a: 1.034080527699_f32,
        b: 0.0_f32,
        c: 0.0_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:193-194
    pub const kHLG: skcms_TransferFunction = skcms_TransferFunction {
        g: -6.0_f32,
        a: 203.0_f32,
        b: 1000.0_f32,
        c: 1.2_f32,
        d: 0.0_f32,
        e: 0.0_f32,
        f: 0.0_f32,
    };
    // Source: include/core/SkColorSpace.h:199-199
    // Source: include/core/SkColorSpace.h:201-201
    // Source: include/core/SkColorSpace.h:204-208
    // Source: include/core/SkColorSpace.h:211-211
    // Source: include/core/SkColorSpace.h:213-219
    // Source: include/core/SkColorSpace.h:221-221
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CicpId {
        kRec709 = 1,
        kRec470SystemM = 4,
        kRec470SystemBG = 5,
        kRec601 = 6,
        kSMPTE_ST_240 = 7,
        kLinear = 8,
        kIEC61966_2_4 = 11,
        kIEC61966_2_1 = 13,
        kRec2020_10bit = 14,
        kRec2020_12bit = 15,
        kPQ = 16,
        kSMPTE_ST_428_1 = 17,
        kHLG = 18,
    }
    impl CicpId {
        pub const kSRGB: Self = Self::kIEC61966_2_1;
    }
    // Source: include/core/SkColorSpace.h:225-225
    pub const kProPhotoRGB: skcms_TransferFunction = skcms_TransferFunction {
        g: 1.8,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    // Source: include/core/SkColorSpace.h:228-228
    pub const kA98RGB: skcms_TransferFunction = k2Dot2;
    // Source: src/core/SkColorSpace.cpp:89-107
    struct TableEntry {
        cicp_id: CicpId,
        trfn: skcms_TransferFunction,
    }
    const CICP_TABLE: &[TableEntry] = &[
        TableEntry {
            cicp_id: CicpId::kRec709,
            trfn: kRec709,
        },
        TableEntry {
            cicp_id: CicpId::kRec470SystemM,
            trfn: kRec470SystemM,
        },
        TableEntry {
            cicp_id: CicpId::kRec470SystemBG,
            trfn: kRec470SystemBG,
        },
        TableEntry {
            cicp_id: CicpId::kRec601,
            trfn: kRec601,
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_ST_240,
            trfn: kSMPTE_ST_240,
        },
        TableEntry {
            cicp_id: CicpId::kLinear,
            trfn: kLinear,
        },
        TableEntry {
            cicp_id: CicpId::kIEC61966_2_4,
            trfn: kIEC61966_2_4,
        },
        TableEntry {
            cicp_id: CicpId::kIEC61966_2_1,
            trfn: kIEC61966_2_1,
        },
        TableEntry {
            cicp_id: CicpId::kRec2020_10bit,
            trfn: kRec2020_10bit,
        },
        TableEntry {
            cicp_id: CicpId::kRec2020_12bit,
            trfn: kRec2020_12bit,
        },
        TableEntry {
            cicp_id: CicpId::kPQ,
            trfn: kPQ,
        },
        TableEntry {
            cicp_id: CicpId::kSMPTE_ST_428_1,
            trfn: kSMPTE_ST_428_1,
        },
        TableEntry {
            cicp_id: CicpId::kHLG,
            trfn: kHLG,
        },
    ];
    // Source: src/core/SkColorSpace.cpp:110-110
    // Source: src/core/SkColorSpace.cpp:112-120
    pub const kCicpIdApplicationDefined: u8 = 2;
    pub fn GetCicp(transfer_characteristics: CicpId, trfn: &mut skcms_TransferFunction) -> bool {
        GetCicpRaw(transfer_characteristics as u8, trfn)
    }
    // Preserve unknown native CICP integer values without creating an invalid Rust enum.
    pub(crate) fn GetCicpRaw(
        transfer_characteristics: u8,
        trfn: &mut skcms_TransferFunction,
    ) -> bool {
        for table_entry in CICP_TABLE {
            if transfer_characteristics == table_entry.cicp_id as u8 {
                *trfn = table_entry.trfn;
                return true;
            }
        }
        false
    }
}
pub mod SkNamedGamut {
    use super::skcms_Matrix3x3;
    // Source: include/core/SkColorSpace.h:234-234
    // Source: include/core/SkColorSpace.h:239-242
    pub const kSRGB: skcms_Matrix3x3 = skcms_Matrix3x3 {
        vals: [
            [
                (0x6FA2 as f32 * (1.0 / 65536.0)),
                (0x6299 as f32 * (1.0 / 65536.0)),
                (0x24A0 as f32 * (1.0 / 65536.0)),
            ],
            [
                (0x38F5 as f32 * (1.0 / 65536.0)),
                (0xB785 as f32 * (1.0 / 65536.0)),
                (0x0F84 as f32 * (1.0 / 65536.0)),
            ],
            [
                (0x0390 as f32 * (1.0 / 65536.0)),
                (0x18DA as f32 * (1.0 / 65536.0)),
                (0xB6CF as f32 * (1.0 / 65536.0)),
            ],
        ],
    };
    // Source: include/core/SkColorSpace.h:244-244
    // Source: include/core/SkColorSpace.h:249-252
    pub const kAdobeRGB: skcms_Matrix3x3 = skcms_Matrix3x3 {
        vals: [
            [
                (0x9c18 as f32 * (1.0 / 65536.0)),
                (0x348d as f32 * (1.0 / 65536.0)),
                (0x2631 as f32 * (1.0 / 65536.0)),
            ],
            [
                (0x4fa5 as f32 * (1.0 / 65536.0)),
                (0xa02c as f32 * (1.0 / 65536.0)),
                (0x102_f32 as f32 * (1.0 / 65536.0)),
            ],
            [
                (0x04fc as f32 * (1.0 / 65536.0)),
                (0x0f95 as f32 * (1.0 / 65536.0)),
                (0xbe9c as f32 * (1.0 / 65536.0)),
            ],
        ],
    };
    // Source: include/core/SkColorSpace.h:254-258
    pub const kDisplayP3: skcms_Matrix3x3 = skcms_Matrix3x3 {
        vals: [
            [0.515102_f32, 0.291965_f32, 0.157153_f32],
            [0.241182_f32, 0.692236_f32, 0.0665819_f32],
            [-0.00104941_f32, 0.0418818_f32, 0.784378_f32],
        ],
    };
    // Source: include/core/SkColorSpace.h:260-264
    pub const kRec2020: skcms_Matrix3x3 = skcms_Matrix3x3 {
        vals: [
            [0.673459_f32, 0.165661_f32, 0.125100_f32],
            [0.279033_f32, 0.675338_f32, 0.0456288_f32],
            [-0.00193139_f32, 0.0299794_f32, 0.797162_f32],
        ],
    };
    // Source: include/core/SkColorSpace.h:266-270
    pub const kXYZ: skcms_Matrix3x3 = skcms_Matrix3x3 {
        vals: [
            [1.0_f32, 0.0_f32, 0.0_f32],
            [0.0_f32, 1.0_f32, 0.0_f32],
            [0.0_f32, 0.0_f32, 1.0_f32],
        ],
    };
}
// Source: include/core/SkColorSpace.h:411-412
impl SkColorSpace {
    pub fn transferFnHash(&self) -> u32 {
        self.fTransferFnHash
    }
}
impl SkColorSpace {
    pub fn hash(&self) -> u64 {
        (self.fTransferFnHash as u64) << 32 | self.fToXYZD50Hash as u64
    }
}
// Source: include/core/SkColorSpace.h:421-422
// Source: include/core/SkColorSpace.h:424-425
// Source: include/core/SkColorSpace.h:427-430
// OnceLock retains the original once-only lazy publication. Its payload owns both destination fields.
struct SkColorSpaceDstFields {
    fInvTransferFn: skcms_TransferFunction,
    fFromXYZD50: skcms_Matrix3x3,
}
pub struct SkColorSpace {
    fTransferFnHash: u32,
    fToXYZD50Hash: u32,
    fTransferFn: skcms_TransferFunction,
    fToXYZD50: skcms_Matrix3x3,
    fLazyDstFieldsOnce: OnceLock<SkColorSpaceDstFields>,
}
// Scalar byte projection of the contiguous g,a,b,c,d,e,f native float representation.
fn transfer_values(tf: &skcms_TransferFunction) -> [f32; 7] {
    [tf.g, tf.a, tf.b, tf.c, tf.d, tf.e, tf.f]
}
fn transfer_bytes(tf: &skcms_TransferFunction) -> [u8; 28] {
    let vals = transfer_values(tf);
    let mut bytes = [0; 28];
    for i in 0..7 {
        bytes[4 * i..4 * i + 4].copy_from_slice(&vals[i].to_ne_bytes());
    }
    bytes
}
// Source: src/core/SkColorSpace.cpp:124-126
impl SkColorSpacePrimaries {
    pub fn toXYZD50(&self, m: &mut skcms_Matrix3x3) -> bool {
        crate::modules::skcms::skcms::skcms_PrimariesToXYZD50(
            self.fRX,
            self.fRY,
            self.fGX,
            self.fGY,
            self.fBX,
            self.fBY,
            self.fWX,
            self.fWY,
            Some(m),
        )
    }
}
// Source: src/core/SkColorSpace.cpp:128-134
impl SkColorSpace {
    fn new(transferFn: skcms_TransferFunction, toXYZD50: skcms_Matrix3x3) -> Self {
        let tf_bytes = transfer_bytes(&transferFn);
        let mut gamut_bytes = [0u8; 36];
        for r in 0..3 {
            for c in 0..3 {
                let i = r * 3 + c;
                gamut_bytes[4 * i..4 * i + 4].copy_from_slice(&toXYZD50.vals[r][c].to_ne_bytes());
            }
        }
        Self {
            fTransferFnHash: crate::src::core::SkChecksum::Hash32(&tf_bytes, 0),
            fToXYZD50Hash: crate::src::core::SkChecksum::Hash32(&gamut_bytes, 0),
            fTransferFn: transferFn,
            fToXYZD50: toXYZD50,
            fLazyDstFieldsOnce: OnceLock::new(),
        }
    }
}
// Source: src/core/SkColorSpace.cpp:136-140
// Source: src/core/SkColorSpace.cpp:142-142
// Source: src/core/SkColorSpace.cpp:144-156
// Source: src/core/SkColorSpace.cpp:158-159
// Source: src/core/SkColorSpace.cpp:161-166
// Source: src/core/SkColorSpace.cpp:168-171
// Source: src/core/SkColorSpace.cpp:173-174
impl SkColorSpace {
    pub fn MakeRGB(
        transferFn: &skcms_TransferFunction,
        toXYZ: &skcms_Matrix3x3,
    ) -> Option<Arc<Self>> {
        if crate::modules::skcms::skcms::skcms_TransferFunction_getType(transferFn)
            == crate::modules::skcms::src::skcms_public::skcms_TFType::skcms_TFType_Invalid
        {
            return None;
        }
        let mut tf = transferFn;
        if super::SkColorSpacePriv::is_almost_srgb(transferFn) {
            if xyz_almost_equal(toXYZ, &SkNamedGamut::kSRGB) {
                return Some(Self::MakeSRGB());
            }
            tf = &SkNamedTransferFn::kSRGB;
        } else if super::SkColorSpacePriv::is_almost_2dot2(transferFn) {
            tf = &SkNamedTransferFn::k2Dot2;
        } else if super::SkColorSpacePriv::is_almost_linear(transferFn) {
            if xyz_almost_equal(toXYZ, &SkNamedGamut::kSRGB) {
                return Some(Self::MakeSRGBLinear());
            }
            tf = &SkNamedTransferFn::kLinear;
        }
        Some(Arc::new(Self::new(*tf, *toXYZ)))
    }
}
impl SkColorSpace {
    pub fn MakeCICP(
        color_primaries: SkNamedPrimaries::CicpId,
        transfer_characteristics: SkNamedTransferFn::CicpId,
    ) -> Option<Arc<Self>> {
        let mut trfn = SkNamedTransferFn::kLinear;
        if !SkNamedTransferFn::GetCicp(transfer_characteristics, &mut trfn) {
            return None;
        }
        let mut toXYZD50 = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        if !SkNamedPrimaries::GetCicp(color_primaries, &mut toXYZD50) {
            return None;
        }
        Self::MakeRGB(&trfn, &toXYZD50)
    }
}
// Source: src/core/SkColorSpace.cpp:176-182
// Source: src/core/SkColorSpace.cpp:184-188
// Source: src/core/SkColorSpace.cpp:190-194
// Source: src/core/SkColorSpace.cpp:196-198
// Source: src/core/SkColorSpace.cpp:200-202
// A process-resident Arc keeps singleton ownership alive; each caller receives a retained reference.
pub(crate) fn sk_srgb_singleton() -> &'static Arc<SkColorSpace> {
    static CS: OnceLock<Arc<SkColorSpace>> = OnceLock::new();
    CS.get_or_init(|| {
        Arc::new(SkColorSpace::new(
            SkNamedTransferFn::kSRGB,
            SkNamedGamut::kSRGB,
        ))
    })
}
pub(crate) fn sk_srgb_linear_singleton() -> &'static Arc<SkColorSpace> {
    static CS: OnceLock<Arc<SkColorSpace>> = OnceLock::new();
    CS.get_or_init(|| {
        Arc::new(SkColorSpace::new(
            SkNamedTransferFn::kLinear,
            SkNamedGamut::kSRGB,
        ))
    })
}
impl SkColorSpace {
    pub fn MakeSRGB() -> Arc<Self> {
        Arc::clone(sk_srgb_singleton())
    }
}
impl SkColorSpace {
    pub fn MakeSRGBLinear() -> Arc<Self> {
        Arc::clone(sk_srgb_linear_singleton())
    }
}
// Source: src/core/SkColorSpace.cpp:204-205
// Source: src/core/SkColorSpace.cpp:208-213
// Source: src/core/SkColorSpace.cpp:216-220
// Source: src/core/SkColorSpace.cpp:222-223
impl SkColorSpace {
    fn computeLazyDstFields(&self) -> &SkColorSpaceDstFields {
        self.fLazyDstFieldsOnce.get_or_init(|| {
            use crate::modules::skcms::skcms::*;
            let mut from = skcms_Matrix3x3 {
                vals: [[0.0; 3]; 3],
            };
            if !skcms_Matrix3x3_invert(&self.fToXYZD50, &mut from) {
                let success = skcms_Matrix3x3_invert(&SRGB_PROFILE_TO_XYZD50, &mut from);
                crate::SkAssertResult!(@source b"src/core/SkColorSpace.cpp\0".as_ptr(),210; @condition b"skcms_Matrix3x3_invert(&skcms_sRGB_profile()->toXYZD50, &fFromXYZD50)\0".as_ptr(); success);
            }
            let mut inv = self.fTransferFn;
            if !skcms_TransferFunction_invert(&self.fTransferFn, &mut inv) {
                inv = *skcms_sRGB_Inverse_TransferFunction();
            }
            SkColorSpaceDstFields {
                fInvTransferFn: inv,
                fFromXYZD50: from,
            }
        })
    }
}
impl SkColorSpace {
    // Source: include/core/SkColorSpace.h:340-340
    // Source: src/core/SkColorSpace.cpp:225-231
    pub fn isNumericalTransferFn(&self, coeffs: &mut skcms_TransferFunction) -> bool {
        self.transferFn(coeffs);
        crate::modules::skcms::skcms::skcms_TransferFunction_getType(coeffs)
            == crate::modules::skcms::src::skcms_public::skcms_TFType::skcms_TFType_sRGBish
    }
}
impl SkColorSpace {
    pub fn transferFn(&self, tf: &mut skcms_TransferFunction) {
        *tf = self.fTransferFn;
    }
}
// Source: src/core/SkColorSpace.cpp:233-235
// Source: src/core/SkColorSpace.cpp:237-239
impl SkColorSpace {
    pub fn transferFn_array(&self, gabcdef: &mut [f32; 7]) {
        *gabcdef = transfer_values(&self.fTransferFn);
    }
}
impl SkColorSpace {
    // Source: src/core/SkColorSpace.cpp:241-244
    pub fn invTransferFn(&self, tf: &mut skcms_TransferFunction) {
        *tf = self.computeLazyDstFields().fInvTransferFn;
    }
}
impl SkColorSpace {
    pub fn gammaCloseToSRGB(&self) -> bool {
        transfer_bytes(&self.fTransferFn) == transfer_bytes(&SkNamedTransferFn::kSRGB)
    }
}
impl SkColorSpace {
    pub fn gammaIsLinear(&self) -> bool {
        transfer_bytes(&self.fTransferFn) == transfer_bytes(&SkNamedTransferFn::kLinear)
    }
}
// Source: src/core/SkColorSpace.cpp:246-249
impl SkColorSpace {
    pub fn toXYZD50(&self, m: &mut skcms_Matrix3x3) -> bool {
        *m = self.fToXYZD50;
        true
    }
}
impl SkColorSpace {
    pub fn makeSRGBGamma(self: &Arc<Self>) -> Option<Arc<Self>> {
        if self.gammaCloseToSRGB() {
            return Some(Arc::clone(self));
        }
        Self::MakeRGB(&SkNamedTransferFn::kSRGB, &self.fToXYZD50)
    }
}
impl SkColorSpace {
    pub fn makeColorSpin(&self) -> Arc<Self> {
        let spin = skcms_Matrix3x3 {
            vals: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        };
        let spun = crate::modules::skcms::skcms::skcms_Matrix3x3_concat(&self.fToXYZD50, &spin);
        Arc::new(Self::new(self.fTransferFn, spun))
    }
}
impl SkColorSpace {
    // Source: src/core/SkColorSpace.cpp:251-254
    pub fn gamutTransformTo(&self, dst: &Self, m: &mut skcms_Matrix3x3) {
        *m = crate::modules::skcms::skcms::skcms_Matrix3x3_concat(
            &dst.computeLazyDstFields().fFromXYZD50,
            &self.fToXYZD50,
        );
    }
}
// Source: src/core/SkColorSpace.cpp:256-258
// Source: src/core/SkColorSpace.cpp:260-260
// Source: src/core/SkColorSpace.cpp:262-263
// Source: src/core/SkColorSpace.cpp:265-265
// Source: src/core/SkColorSpace.cpp:267-268
impl SkColorSpace {
    pub fn isSRGB(&self) -> bool {
        std::ptr::eq(Arc::as_ptr(sk_srgb_singleton()), self)
    }
}
// Source: src/core/SkColorSpace.cpp:270-275
// Source: src/core/SkColorSpace.cpp:277-282
// Source: src/core/SkColorSpace.cpp:284-289
// Source: src/core/SkColorSpace.cpp:291-291
// Source: src/core/SkColorSpace.cpp:293-294
impl SkColorSpace {
    pub fn makeLinearGamma(self: &Arc<Self>) -> Option<Arc<Self>> {
        if self.gammaIsLinear() {
            return Some(Arc::clone(self));
        }
        Self::MakeRGB(&SkNamedTransferFn::kLinear, &self.fToXYZD50)
    }
}
// Source: src/core/SkColorSpace.cpp:296-297
// Source: src/core/SkColorSpace.cpp:300-301
// Source: src/core/SkColorSpace.cpp:303-329
impl SkColorSpace {
    pub fn toProfile(
        &self,
        profile: &mut crate::modules::skcms::src::skcms_public::skcms_ICCProfile,
    ) {
        use crate::modules::skcms::skcms::skcms_TransferFunction_getType;
        use crate::modules::skcms::src::skcms_public::*;
        skcms_Init(profile);
        skcms_SetTransferFunction(profile, &self.fTransferFn);
        skcms_SetXYZD50(profile, &self.fToXYZD50);
        match skcms_TransferFunction_getType(&self.fTransferFn) {
            skcms_TFType::skcms_TFType_PQ | skcms_TFType::skcms_TFType_PQish => {
                profile.has_CICP = true;
                profile.CICP.transfer_characteristics = SkNamedTransferFn::CicpId::kPQ as u8;
            }
            skcms_TFType::skcms_TFType_HLG | skcms_TFType::skcms_TFType_HLGish => {
                profile.has_CICP = true;
                profile.CICP.transfer_characteristics = SkNamedTransferFn::CicpId::kHLG as u8;
            }
            _ => {}
        }
        if profile.has_CICP {
            profile.CICP.matrix_coefficients = 0;
            profile.CICP.video_full_range_flag = 1;
            let mut primaries_id = SkNamedPrimaries::CicpId::kRec709;
            if SkNamedPrimaries::GetCicpFromMatrix(&self.fToXYZD50, &mut primaries_id) {
                profile.CICP.color_primaries = primaries_id as u8;
            } else {
                profile.CICP.color_primaries = SkNamedPrimaries::kCicpIdApplicationDefined;
            }
        }
    }
}
// Source: src/core/SkColorSpace.cpp:331-407
impl SkColorSpace {
    /// All raw curve/table storage referenced by `profile` must remain valid for this call.
    pub unsafe fn Make(
        profile: &crate::modules::skcms::src::skcms_public::skcms_ICCProfile,
    ) -> Option<Arc<Self>> {
        use crate::modules::skcms::skcms::*;
        let use_cicp = profile.has_CICP
            && profile.CICP.matrix_coefficients == 0
            && profile.CICP.video_full_range_flag == 1;
        // Keep the integer representation: native enum casts allow unknown IDs.
        let cicp_color_primaries = profile.CICP.color_primaries;
        let cicp_transfer_characteristics = profile.CICP.transfer_characteristics;
        if use_cicp
            && cicp_color_primaries == SkNamedPrimaries::CicpId::kRec709 as u8
            && cicp_transfer_characteristics == SkNamedTransferFn::CicpId::kIEC61966_2_4 as u8
        {
            return Some(Self::MakeSRGB());
        } else if unsafe { skcms_ApproximatelyEqualProfiles(profile, skcms_sRGB_profile()) } {
            return Some(Self::MakeSRGB());
        }
        // Rust requires initialized POD for mutable references; these initial bytes are never
        // selected as a result unless a native producer sets the corresponding success flag.
        let mut toXYZD50 = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        let mut hasSetToXYZD50 = false;
        if use_cicp {
            if SkNamedPrimaries::GetCicpRaw(cicp_color_primaries, &mut toXYZD50) {
                hasSetToXYZD50 = true;
            } else if profile.CICP.color_primaries != SkNamedPrimaries::kCicpIdApplicationDefined {
                return None;
            }
        }
        if profile.has_toXYZD50 && !hasSetToXYZD50 {
            let mut inv = skcms_Matrix3x3 {
                vals: [[0.0; 3]; 3],
            };
            toXYZD50 = profile.toXYZD50;
            if skcms_Matrix3x3_invert(&toXYZD50, &mut inv) {
                hasSetToXYZD50 = true;
            }
        }
        if !hasSetToXYZD50 {
            return None;
        }
        let mut trfn = SkNamedTransferFn::kLinear;
        let mut hasSetTrfn = false;
        if use_cicp {
            if SkNamedTransferFn::GetCicpRaw(cicp_transfer_characteristics, &mut trfn) {
                hasSetTrfn = true;
            } else if profile.CICP.transfer_characteristics
                != SkNamedTransferFn::kCicpIdApplicationDefined
            {
                return None;
            }
        }
        if profile.has_trc && !hasSetTrfn {
            let trc = &profile.trc;
            if unsafe {
                trc[0].table.table_entries == 0
                    && trc[1].table.table_entries == 0
                    && trc[2].table.table_entries == 0
                    && transfer_bytes(&trc[0].parametric.parametric)
                        == transfer_bytes(&trc[1].parametric.parametric)
                    && transfer_bytes(&trc[0].parametric.parametric)
                        == transfer_bytes(&trc[2].parametric.parametric)
            } {
                trfn = unsafe { profile.trc[0].parametric.parametric };
                hasSetTrfn = true;
            } else if unsafe {
                skcms_TRCs_AreApproximateInverse(
                    Some(profile),
                    skcms_sRGB_Inverse_TransferFunction(),
                )
            } {
                trfn = SkNamedTransferFn::kSRGB;
                hasSetTrfn = true;
            }
        }
        if !hasSetTrfn {
            return None;
        }
        Self::MakeRGB(&trfn, &toXYZD50)
    }
}
// Source: src/core/SkColorSpace.cpp:411-416
// Rust constants retain the native alias kCurrent_Version == k1_Version.
const k0_Version: u8 = 0;
const k1_Version: u8 = 1;
const kCurrent_Version: u8 = k1_Version;

// Source: src/core/SkColorSpace.cpp:418-425
#[repr(C)]
struct ColorSpaceHeader {
    fVersion: u8,
    fReserved0: u8,
    fReserved1: u8,
    fReserved2: u8,
}
impl Default for ColorSpaceHeader {
    fn default() -> Self {
        Self {
            fVersion: kCurrent_Version,
            fReserved0: 0,
            fReserved1: 0,
            fReserved2: 0,
        }
    }
}
impl SkColorSpace {
    // Source: src/core/SkColorSpace.cpp:427-439
    /// Safety: nonnull memory names at least the returned byte count of writable
    /// storage, disjoint from this color space. Null queries the required size.
    /// Native-endian scalar bytes are preserved without assuming Rust skcms layout.
    pub unsafe fn writeToMemory(&self, mut memory: *mut u8) -> usize {
        if !memory.is_null() {
            let header = ColorSpaceHeader::default();
            let bytes = [
                header.fVersion,
                header.fReserved0,
                header.fReserved1,
                header.fReserved2,
            ];
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory, bytes.len());
            memory = memory.add(std::mem::size_of::<ColorSpaceHeader>());
            let tf = &self.fTransferFn;
            for value in [tf.g, tf.a, tf.b, tf.c, tf.d, tf.e, tf.f] {
                std::ptr::copy_nonoverlapping(value.to_ne_bytes().as_ptr(), memory, 4);
                memory = memory.add(4);
            }
            for row in &self.fToXYZD50.vals {
                for value in row {
                    std::ptr::copy_nonoverlapping(value.to_ne_bytes().as_ptr(), memory, 4);
                    memory = memory.add(4);
                }
            }
        }
        std::mem::size_of::<ColorSpaceHeader>() + 16 * std::mem::size_of::<f32>()
    }

    // Source: src/core/SkColorSpace.cpp:441-445
    pub fn serialize(&self) -> Arc<super::SkData::SkData> {
        unsafe {
            let data =
                super::SkData::SkData::MakeUninitialized(self.writeToMemory(std::ptr::null_mut()));
            self.writeToMemory(data.writable_data());
            data
        }
    }

    // Source: src/core/SkColorSpace.cpp:447-470
    // Adapted byte-slice access retains native unaligned memcpy behavior, version
    // rejection and ignored reserved/trailing bytes. The actual MakeRGB validates
    // and canonicalizes the decoded parameters.
    pub fn Deserialize(mut data: &[u8]) -> Option<Arc<Self>> {
        if data.len() < std::mem::size_of::<ColorSpaceHeader>() {
            return None;
        }
        let header = ColorSpaceHeader {
            fVersion: data[0],
            fReserved0: data[1],
            fReserved1: data[2],
            fReserved2: data[3],
        };
        data = &data[std::mem::size_of::<ColorSpaceHeader>()..];
        if header.fVersion != k1_Version {
            return None;
        }
        if data.len() < 16 * std::mem::size_of::<f32>() {
            return None;
        }
        let scalar = |bytes: &[u8]| f32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let transferFn = skcms_TransferFunction {
            g: scalar(&data[0..4]),
            a: scalar(&data[4..8]),
            b: scalar(&data[8..12]),
            c: scalar(&data[12..16]),
            d: scalar(&data[16..20]),
            e: scalar(&data[20..24]),
            f: scalar(&data[24..28]),
        };
        data = &data[7 * std::mem::size_of::<f32>()..];
        let mut toXYZ = skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        for row in 0..3 {
            for col in 0..3 {
                let offset = (row * 3 + col) * 4;
                toXYZ.vals[row][col] = scalar(&data[offset..offset + 4]);
            }
        }
        Self::MakeRGB(&transferFn, &toXYZ)
    }
}

// Source: src/core/SkColorSpace.cpp:472-475
// Source: src/core/SkColorSpace.cpp:477-479
// Source: src/core/SkColorSpace.cpp:481-481
// Source: src/core/SkColorSpace.cpp:487-490
// Source: src/core/SkColorSpace.cpp:492-502
// Source: src/core/SkColorSpace.cpp:504-507
impl SkColorSpace {
    pub fn Equals(x: Option<&Self>, y: Option<&Self>) -> bool {
        match (x, y) {
            (None, None) => return true,
            (Some(x), Some(y)) if std::ptr::eq(x, y) => return true,
            _ => {}
        }
        let (Some(x), Some(y)) = (x, y) else {
            return false;
        };
        if x.hash() == y.hash() {
            #[cfg(sk_debug)]
            {
                let equiv = |x: f32, y: f32| x == y || x.is_nan() && y.is_nan();
                let xt = transfer_values(&x.fTransferFn);
                let yt = transfer_values(&y.fTransferFn);
                for i in 0..7 {
                    crate::SkASSERTF!(@source b"src/core/SkColorSpace.cpp\0".as_ptr(),495; @condition b"equiv(X,Y)\0".as_ptr(); equiv(xt[i],yt[i]),"Hash collision at tf[%d], !equiv(%g,%g)\n",i as core::ffi::c_int,xt[i] as f64,yt[i] as f64);
                }
                for r in 0..3 {
                    for c in 0..3 {
                        crate::SkASSERTF!(@source b"src/core/SkColorSpace.cpp\0".as_ptr(),501; @condition b"equiv(X,Y)\0".as_ptr();
                            equiv(x.fToXYZD50.vals[r][c],y.fToXYZD50.vals[r][c]),
                            "Hash collision at toXYZD50[%d][%d], !equiv(%g,%g)\n",r as core::ffi::c_int,c as core::ffi::c_int,x.fToXYZD50.vals[r][c] as f64,y.fToXYZD50.vals[r][c] as f64);
                    }
                }
            }
            return true;
        }
        false
    }
}
// Source: include/core/SkColorSpace.h:351-351
impl SkColorSpace {
    pub fn toXYZD50Hash(&self) -> u32 {
        self.fToXYZD50Hash
    }
}
