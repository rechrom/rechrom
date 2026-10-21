// Source: src/codec/SkHdrAgtmParse.cpp
use crate::src::codec::SkHdrAgtm::{PopulateSlopeFromPCHIP, PopulateUsingRwtmo, Validate};
use crate::src::codec::SkHdrMetadata::*;
use crate::src::core::{
    SkColorSpace::{SkColorSpacePrimaries, SkNamedPrimaries},
    SkData::SkData,
    SkStream::{SkDynamicMemoryWStream, SkMemoryStream, SkStream, SkWStream},
    SkStreamPriv,
};
use core::pin::Pin;
use std::sync::Arc;
unsafe extern "C" {
    fn lroundf(x: f32) -> core::ffi::c_long;
    fn tanf(x: f32) -> f32;
    fn atanf(x: f32) -> f32;
}
// Source: src/codec/SkHdrAgtmParse.cpp:16-19
fn clamp<T: PartialOrd + Copy>(x: T, a: T, b: T) -> T {
    let lower = if x < a { a } else { x };
    if b < lower {
        b
    } else {
        lower
    }
}
// Source: src/codec/SkHdrAgtmParse.cpp:23-28
unsafe fn float_to_uint16(f: f32, clamp_min: u16, clamp_max: u16, offset: u16, scale: f32) -> u16 {
    let v = unsafe { lroundf(f * scale) } as i32 + offset as i32;
    clamp(v, clamp_min as i32, clamp_max as i32) as u16
}
// Source: src/codec/SkHdrAgtmParse.cpp:31-35
fn uint16_to_float(v: u16, clamp_min: u16, clamp_max: u16, offset: u16, scale: f32) -> f32 {
    (clamp(v, clamp_min, clamp_max) as i32 - offset as i32) as f32 / scale
}
// Source: src/codec/SkHdrAgtmParse.cpp:38-53
#[derive(Default)]
// Source: src/codec/SkHdrAgtmParse.cpp:51-52
struct BitfieldReader {
    fBits: u8,
    fBitsRead: u8,
}
impl BitfieldReader {
    // Source: src/codec/SkHdrAgtmParse.cpp:40-42
    fn readFromStream(&mut self, s: Pin<&mut SkMemoryStream>) -> bool {
        s.readU8(&mut self.fBits)
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:43-49
    fn readBits(&mut self, bits: u8) -> u8 {
        let result = self.fBits >> (8 - bits);
        self.fBits = ((self.fBits as u32) << bits) as u8;
        self.fBitsRead += bits;
        result
    }
}
// Source: src/codec/SkHdrAgtmParse.cpp:56-75
#[derive(Default)]
// Source: src/codec/SkHdrAgtmParse.cpp:73-74
struct BitfieldWriter {
    fBits: u8,
    fBitsWritten: u8,
}
impl BitfieldWriter {
    // Source: src/codec/SkHdrAgtmParse.cpp:58-64
    fn writeBits(&mut self, value: u8, bits: u8) {
        crate::SkASSERT!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),60;@condition b"value <= (1 << bits) - 1\0".as_ptr();value as u32<=(1u32<<bits)-1);
        self.fBits = ((self.fBits as u32) << bits) as u8;
        self.fBits |= value;
        self.fBitsWritten += bits;
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:65-71
    fn padAndWriteToStream(&mut self, s: Pin<&mut SkDynamicMemoryWStream>) {
        crate::SkASSERT!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),67;@condition b"fBitsWritten <= 8\0".as_ptr();self.fBitsWritten<=8);
        self.fBits = ((self.fBits as u32) << (8 - self.fBitsWritten)) as u8;
        self.fBitsWritten = 8;
        let _ = s.write8(self.fBits as u32);
    }
}
// Source: src/codec/SkHdrAgtmParse.cpp:79-85
macro_rules! RETURN_ON_FALSE{($line:literal,$native:expr,$value:expr)=>{if !$value{
 crate::SkDebugf!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),$line;b"AGTM parsing failed %s at %d\n\0".as_ptr(),$native.as_ptr(),$line as core::ffi::c_int);return false;
}};}
// Source: src/codec/SkHdrAgtmParse.cpp:91-147
// Syntax is native memset-zeroed scratch. Bitfield widths are represented by
// actual packing reader/writer operations, not claimed as a C bitfield ABI.
struct AgtmSyntax {
    // Source: src/codec/SkHdrAgtmParse.cpp:118-119
    application_version: u8,
    minimum_application_version: u8,
    // Source: src/codec/SkHdrAgtmParse.cpp:122-124
    has_custom_hdr_reference_white_flag: u8,
    has_adaptive_tone_map_flag: u8,
    hdr_reference_white: u16,
    // Source: src/codec/SkHdrAgtmParse.cpp:127-129
    baseline_hdr_headroom: u16,
    use_reference_white_tone_mapping_flag: u8,
    num_alternate_images: u8,
    // Source: src/codec/SkHdrAgtmParse.cpp:130-132
    gain_application_space_chromaticities_flag: u8,
    has_common_component_mix_params_flag: u8,
    has_common_curve_params_flag: u8,
    // Source: src/codec/SkHdrAgtmParse.cpp:133-134
    gain_application_space_chromaticities: [u16; 8],
    alternate_hdr_headrooms: [u16; 4],
    // Source: src/codec/SkHdrAgtmParse.cpp:137-139
    component_mixing_type: [u8; 4],
    has_component_mixing_coefficient_flag: [[u8; 6]; 4],
    component_mixing_coefficient: [[u16; 6]; 4],
    // Source: src/codec/SkHdrAgtmParse.cpp:142-143
    gain_curve_num_control_points_minus_1: [u8; 4],
    gain_curve_use_pchip_slope_flag: [u8; 4],
    // Source: src/codec/SkHdrAgtmParse.cpp:144-146
    gain_curve_control_points_x: [[u16; 32]; 4],
    gain_curve_control_points_y: [[u16; 32]; 4],
    gain_curve_control_points_theta: [[u16; 32]; 4],
}
// Source: src/codec/SkHdrAgtmParse.cpp:149-151
const _: () = assert!(4 == HeadroomAdaptiveToneMap::kMaxNumAlternateImages);
// Source: src/codec/SkHdrAgtmParse.cpp:152-154
const _: () = assert!(32 == GainCurve::kMaxNumControlPoints);
impl AgtmSyntax {
    // Source: src/codec/SkHdrAgtmParse.cpp:375-376
    // Source: src/codec/SkHdrAgtmParse.cpp:526-527
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:157-169
    fn parse_application_info(&mut self, mut s: Pin<&mut SkMemoryStream>) -> bool {
        let mut flags = BitfieldReader::default();
        RETURN_ON_FALSE!(
            159,
            b"flags.readFromStream(s)\0",
            flags.readFromStream(s.as_mut())
        );
        self.application_version = flags.readBits(3);
        self.minimum_application_version = flags.readBits(3);
        if self.minimum_application_version > 0 {
            return false;
        }
        let reserved_zero = flags.readBits(2);
        RETURN_ON_FALSE!(166, b"reserved_zero == 0\0", reserved_zero == 0);
        RETURN_ON_FALSE!(
            167,
            b"parse_color_volume_transform(s)\0",
            self.parse_color_volume_transform(s)
        );
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:171-178
    fn write_application_info(&self, mut s: Pin<&mut SkDynamicMemoryWStream>) {
        let mut flags = BitfieldWriter::default();
        flags.writeBits(self.application_version, 3);
        flags.writeBits(self.minimum_application_version, 3);
        flags.padAndWriteToStream(s.as_mut());
        self.write_color_volume_transform(s);
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:181-195
    fn parse_color_volume_transform(&mut self, mut s: Pin<&mut SkMemoryStream>) -> bool {
        let mut flags = BitfieldReader::default();
        RETURN_ON_FALSE!(
            183,
            b"flags.readFromStream(s)\0",
            flags.readFromStream(s.as_mut())
        );
        self.has_custom_hdr_reference_white_flag = flags.readBits(1);
        self.has_adaptive_tone_map_flag = flags.readBits(1);
        let reserved_zero = flags.readBits(6);
        RETURN_ON_FALSE!(187, b"reserved_zero == 0\0", reserved_zero == 0);
        if self.has_custom_hdr_reference_white_flag == 1 {
            RETURN_ON_FALSE!(
                189,
                b"SkStreamPriv::ReadU16BE(&s, &hdr_reference_white)\0",
                SkStreamPriv::ReadU16BE(s.as_mut(), &mut self.hdr_reference_white)
            );
        }
        if self.has_adaptive_tone_map_flag == 1 {
            RETURN_ON_FALSE!(
                192,
                b"parse_adaptive_tone_map(s)\0",
                self.parse_adaptive_tone_map(s)
            );
        }
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:197-208
    fn write_color_volume_transform(&self, mut s: Pin<&mut SkDynamicMemoryWStream>) {
        let mut flags = BitfieldWriter::default();
        flags.writeBits(self.has_custom_hdr_reference_white_flag, 1);
        flags.writeBits(self.has_adaptive_tone_map_flag, 1);
        flags.padAndWriteToStream(s.as_mut());
        if self.has_custom_hdr_reference_white_flag != 0 {
            let _ = SkStreamPriv::WriteU16BE(s.as_mut(), self.hdr_reference_white);
        }
        if self.has_adaptive_tone_map_flag == 1 {
            self.write_adaptive_tone_map(s);
        }
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:210-236
    fn parse_adaptive_tone_map(&mut self, mut s: Pin<&mut SkMemoryStream>) -> bool {
        RETURN_ON_FALSE!(
            211,
            b"SkStreamPriv::ReadU16BE(&s, &baseline_hdr_headroom)\0",
            SkStreamPriv::ReadU16BE(s.as_mut(), &mut self.baseline_hdr_headroom)
        );
        let mut flags = BitfieldReader::default();
        RETURN_ON_FALSE!(
            213,
            b"flags.readFromStream(s)\0",
            flags.readFromStream(s.as_mut())
        );
        self.use_reference_white_tone_mapping_flag = flags.readBits(1);
        if self.use_reference_white_tone_mapping_flag == 0 {
            self.num_alternate_images = flags.readBits(3);
            self.gain_application_space_chromaticities_flag = flags.readBits(2);
            self.has_common_component_mix_params_flag = flags.readBits(1);
            self.has_common_curve_params_flag = flags.readBits(1);
            if self.gain_application_space_chromaticities_flag == 3 {
                for r in 0..8 {
                    RETURN_ON_FALSE!(
                        222,
                        b"SkStreamPriv::ReadU16BE(&s, &gain_application_space_chromaticities[r])\0",
                        SkStreamPriv::ReadU16BE(
                            s.as_mut(),
                            &mut self.gain_application_space_chromaticities[r]
                        )
                    );
                }
            }
            for a in 0..clamp(self.num_alternate_images, 0, 4) as usize {
                RETURN_ON_FALSE!(
                    227,
                    b"SkStreamPriv::ReadU16BE(&s, &alternate_hdr_headrooms[a])\0",
                    SkStreamPriv::ReadU16BE(s.as_mut(), &mut self.alternate_hdr_headrooms[a])
                );
                RETURN_ON_FALSE!(
                    228,
                    b"parse_component_mixing(a, s)\0",
                    self.parse_component_mixing(a, s.as_mut())
                );
                RETURN_ON_FALSE!(
                    229,
                    b"parse_gain_curve(a, s)\0",
                    self.parse_gain_curve(a, s.as_mut())
                );
            }
        } else {
            let reserved_zero = flags.readBits(7);
            RETURN_ON_FALSE!(233, b"reserved_zero == 0\0", reserved_zero == 0);
        }
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:238-261
    fn write_adaptive_tone_map(&self, mut s: Pin<&mut SkDynamicMemoryWStream>) {
        let _ = SkStreamPriv::WriteU16BE(s.as_mut(), self.baseline_hdr_headroom);
        let mut flags = BitfieldWriter::default();
        flags.writeBits(self.use_reference_white_tone_mapping_flag, 1);
        if self.use_reference_white_tone_mapping_flag == 0 {
            flags.writeBits(self.num_alternate_images, 3);
            flags.writeBits(self.gain_application_space_chromaticities_flag, 2);
            flags.writeBits(self.has_common_component_mix_params_flag, 1);
            flags.writeBits(self.has_common_curve_params_flag, 1);
            flags.padAndWriteToStream(s.as_mut());
            if self.gain_application_space_chromaticities_flag == 3 {
                for r in 0..8 {
                    let _ = SkStreamPriv::WriteU16BE(
                        s.as_mut(),
                        self.gain_application_space_chromaticities[r],
                    );
                }
            }
            for a in 0..self.num_alternate_images as usize {
                let _ = SkStreamPriv::WriteU16BE(s.as_mut(), self.alternate_hdr_headrooms[a]);
                self.write_component_mixing(a, s.as_mut());
                self.write_gain_curve(a, s.as_mut());
            }
        } else {
            flags.padAndWriteToStream(s);
        }
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:263-293
    fn parse_component_mixing(&mut self, a: usize, mut s: Pin<&mut SkMemoryStream>) -> bool {
        if a == 0 || self.has_common_component_mix_params_flag == 0 {
            let mut flags = BitfieldReader::default();
            RETURN_ON_FALSE!(
                266,
                b"flags.readFromStream(s)\0",
                flags.readFromStream(s.as_mut())
            );
            self.component_mixing_type[a] = flags.readBits(2);
            if self.component_mixing_type[a] != 3 {
                let reserved_zero = flags.readBits(6);
                RETURN_ON_FALSE!(270, b"reserved_zero == 0\0", reserved_zero == 0);
            } else {
                for k in 0..6 {
                    self.has_component_mixing_coefficient_flag[a][k] = flags.readBits(1);
                }
                for k in 0..6 {
                    if self.has_component_mixing_coefficient_flag[a][k] == 1 {
                        RETURN_ON_FALSE!(
                            277,
                            b"SkStreamPriv::ReadU16BE(&s, &component_mixing_coefficient[a][k])\0",
                            SkStreamPriv::ReadU16BE(
                                s.as_mut(),
                                &mut self.component_mixing_coefficient[a][k]
                            )
                        );
                    } else {
                        self.component_mixing_coefficient[a][k] = 0;
                    }
                }
            }
        } else {
            self.component_mixing_type[a] = self.component_mixing_type[0];
            if self.component_mixing_type[a] == 3 {
                for k in 0..6 {
                    self.component_mixing_coefficient[a][k] =
                        self.component_mixing_coefficient[0][k];
                }
            }
        }
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:295-313
    fn write_component_mixing(&self, a: usize, mut s: Pin<&mut SkDynamicMemoryWStream>) {
        if a == 0 || self.has_common_component_mix_params_flag == 0 {
            let mut flags = BitfieldWriter::default();
            flags.writeBits(self.component_mixing_type[a], 2);
            if self.component_mixing_type[a] == 3 {
                for k in 0..6 {
                    flags.writeBits(self.has_component_mixing_coefficient_flag[a][k], 1);
                }
                flags.padAndWriteToStream(s.as_mut());
                for k in 0..6 {
                    if self.has_component_mixing_coefficient_flag[a][k] == 1 {
                        let _ = SkStreamPriv::WriteU16BE(
                            s.as_mut(),
                            self.component_mixing_coefficient[a][k],
                        );
                    }
                }
            } else {
                flags.padAndWriteToStream(s);
            }
        }
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:315-342
    fn parse_gain_curve(&mut self, a: usize, mut s: Pin<&mut SkMemoryStream>) -> bool {
        if a == 0 || self.has_common_curve_params_flag == 0 {
            let mut flags = BitfieldReader::default();
            RETURN_ON_FALSE!(
                318,
                b"flags.readFromStream(s)\0",
                flags.readFromStream(s.as_mut())
            );
            self.gain_curve_num_control_points_minus_1[a] = flags.readBits(5);
            self.gain_curve_use_pchip_slope_flag[a] = flags.readBits(1);
            let reserved_zero = flags.readBits(2);
            RETURN_ON_FALSE!(322, b"reserved_zero == 0\0", reserved_zero == 0);
            for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
                RETURN_ON_FALSE!(
                    324,
                    b"SkStreamPriv::ReadU16BE(&s, &gain_curve_control_points_x[a][c])\0",
                    SkStreamPriv::ReadU16BE(
                        s.as_mut(),
                        &mut self.gain_curve_control_points_x[a][c]
                    )
                );
            }
        } else {
            self.gain_curve_num_control_points_minus_1[a] =
                self.gain_curve_num_control_points_minus_1[0];
            self.gain_curve_use_pchip_slope_flag[a] = self.gain_curve_use_pchip_slope_flag[0];
            for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
                self.gain_curve_control_points_x[a][c] = self.gain_curve_control_points_x[0][c];
            }
        }
        for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
            RETURN_ON_FALSE!(
                334,
                b"SkStreamPriv::ReadU16BE(&s, &gain_curve_control_points_y[a][c])\0",
                SkStreamPriv::ReadU16BE(s.as_mut(), &mut self.gain_curve_control_points_y[a][c])
            );
        }
        if self.gain_curve_use_pchip_slope_flag[a] == 0 {
            for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
                RETURN_ON_FALSE!(
                    338,
                    b"SkStreamPriv::ReadU16BE(&s, &gain_curve_control_points_theta[a][c])\0",
                    SkStreamPriv::ReadU16BE(
                        s.as_mut(),
                        &mut self.gain_curve_control_points_theta[a][c]
                    )
                );
            }
        }
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:344-362
    fn write_gain_curve(&self, a: usize, mut s: Pin<&mut SkDynamicMemoryWStream>) {
        // Native intentionally tests common_component_mix_params, not common_curve_params.
        if a == 0 || self.has_common_component_mix_params_flag == 0 {
            let mut flags = BitfieldWriter::default();
            flags.writeBits(self.gain_curve_num_control_points_minus_1[a], 5);
            flags.writeBits(self.gain_curve_use_pchip_slope_flag[a], 1);
            flags.padAndWriteToStream(s.as_mut());
            for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
                let _ =
                    SkStreamPriv::WriteU16BE(s.as_mut(), self.gain_curve_control_points_x[a][c]);
            }
        }
        for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
            let _ = SkStreamPriv::WriteU16BE(s.as_mut(), self.gain_curve_control_points_y[a][c]);
        }
        if self.gain_curve_use_pchip_slope_flag[a] == 0 {
            for c in 0..self.gain_curve_num_control_points_minus_1[a] as usize + 1 {
                let _ = SkStreamPriv::WriteU16BE(
                    s.as_mut(),
                    self.gain_curve_control_points_theta[a][c],
                );
            }
        }
    }
}
impl AdaptiveGlobalToneMap {
    // Source: src/codec/SkHdrAgtmParse.cpp:368-518
    pub unsafe fn parse(&mut self, data: *const SkData) -> bool {
        if data.is_null() {
            return false;
        }
        let data = unsafe { &*data };
        let mut s = core::pin::pin!(unsafe {
            SkMemoryStream::from_memory(data.data(), data.size(), false)
        });
        let mut syntax = AgtmSyntax::zeroed();
        if !syntax.parse_application_info(s.as_mut()) {
            return false;
        }
        self.fHdrReferenceWhite = if syntax.has_custom_hdr_reference_white_flag == 1 {
            uint16_to_float(syntax.hdr_reference_white, 1, 50000, 0, 5.0)
        } else {
            Self::kDefaultHdrReferenceWhite
        };
        if syntax.has_adaptive_tone_map_flag == 0 {
            return true;
        }
        self.fHeadroomAdaptiveToneMap = Some(HeadroomAdaptiveToneMap::default());
        let hatm = self.fHeadroomAdaptiveToneMap.as_mut().unwrap();
        hatm.fBaselineHdrHeadroom =
            uint16_to_float(syntax.baseline_hdr_headroom, 0, 60000, 0, 10000.0);
        if syntax.use_reference_white_tone_mapping_flag == 1 {
            PopulateUsingRwtmo(hatm);
            return true;
        }
        let numAlternateImages = clamp(
            syntax.num_alternate_images,
            0,
            HeadroomAdaptiveToneMap::kMaxNumAlternateImages as u8,
        ) as usize;
        hatm.fAlternateImages
            .resize_with(numAlternateImages, Default::default);
        for a in 0..numAlternateImages {
            hatm.fAlternateImages[a].fHdrHeadroom =
                uint16_to_float(syntax.alternate_hdr_headrooms[a], 0, 60000, 0, 10000.0);
        }
        hatm.fGainApplicationSpacePrimaries =
            match syntax.gain_application_space_chromaticities_flag {
                0 => SkNamedPrimaries::kRec709,
                1 => SkNamedPrimaries::kSMPTE_EG_432_1,
                2 => SkNamedPrimaries::kRec2020,
                3 => {
                    let c = &syntax.gain_application_space_chromaticities;
                    SkColorSpacePrimaries {
                        fRX: uint16_to_float(c[0], 0, 50000, 0, 50000.0),
                        fRY: uint16_to_float(c[1], 0, 50000, 0, 50000.0),
                        fGX: uint16_to_float(c[2], 0, 50000, 0, 50000.0),
                        fGY: uint16_to_float(c[3], 0, 50000, 0, 50000.0),
                        fBX: uint16_to_float(c[4], 0, 50000, 0, 50000.0),
                        fBY: uint16_to_float(c[5], 0, 50000, 0, 50000.0),
                        fWX: uint16_to_float(c[6], 0, 50000, 0, 50000.0),
                        fWY: uint16_to_float(c[7], 0, 50000, 0, 50000.0),
                    }
                }
                _ => crate::SkUNREACHABLE!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),443;),
            };
        for a in 0..numAlternateImages {
            let mix = &mut hatm.fAlternateImages[a].fColorGainFunction.fComponentMixing;
            match syntax.component_mixing_type[a] {
                0 => {
                    *mix = ComponentMixingFunction {
                        fMax: 1.0,
                        ..Default::default()
                    }
                }
                1 => {
                    *mix = ComponentMixingFunction {
                        fComponent: 1.0,
                        ..Default::default()
                    }
                }
                2 => {
                    *mix = ComponentMixingFunction {
                        fRed: 1.0 / 6.0,
                        fGreen: 1.0 / 6.0,
                        fBlue: 1.0 / 6.0,
                        fMax: 1.0 / 2.0,
                        ..Default::default()
                    }
                }
                3 => {
                    let mut p = [0.0; 6];
                    let mut p_sum = 0.0;
                    for k in 0..6 {
                        p[k] = uint16_to_float(
                            syntax.component_mixing_coefficient[a][k],
                            0,
                            50000,
                            0,
                            50000.0,
                        );
                        p_sum += p[k];
                    }
                    if p_sum == 0.0 {
                        return false;
                    }
                    mix.fRed = p[0] / p_sum;
                    mix.fGreen = p[1] / p_sum;
                    mix.fBlue = p[2] / p_sum;
                    mix.fMax = p[3] / p_sum;
                    mix.fMin = p[4] / p_sum;
                    mix.fComponent = p[5] / p_sum;
                }
                _ => {}
            }
        }
        for a in 0..numAlternateImages {
            let cubic = &mut hatm.fAlternateImages[a].fColorGainFunction.fGainCurve;
            let numControlPoints = syntax.gain_curve_num_control_points_minus_1[a] as usize + 1;
            cubic
                .fControlPoints
                .resize(numControlPoints, ControlPoint::default());
            let sign = if syntax.baseline_hdr_headroom < syntax.alternate_hdr_headrooms[a] {
                1.0
            } else {
                -1.0
            };
            for c in 0..numControlPoints {
                cubic.fControlPoints[c].fX = uint16_to_float(
                    syntax.gain_curve_control_points_x[a][c],
                    0,
                    64000,
                    0,
                    1000.0,
                );
                cubic.fControlPoints[c].fY = sign
                    * uint16_to_float(
                        syntax.gain_curve_control_points_y[a][c],
                        0,
                        60000,
                        0,
                        10000.0,
                    );
            }
            if syntax.gain_curve_use_pchip_slope_flag[a] == 0 {
                for c in 0..numControlPoints {
                    let theta = uint16_to_float(
                        syntax.gain_curve_control_points_theta[a][c],
                        1,
                        35999,
                        18000,
                        36000.0 / core::f32::consts::PI,
                    );
                    cubic.fControlPoints[c].fM = unsafe { tanf(theta) };
                }
            } else {
                PopulateSlopeFromPCHIP(cubic);
            }
        }
        if !Validate(self) {
            return false;
        }
        true
    }
    // Source: src/codec/SkHdrAgtmParse.cpp:520-655
    /// Safety: native float-to-integer conversions require finite representable
    /// values as in the original validated metadata serialization domain.
    pub unsafe fn serialize(&self) -> Option<Arc<SkData>> {
        if !Validate(self) {
            return None;
        }
        let mut syntax = AgtmSyntax::zeroed();
        syntax.application_version = 0;
        syntax.minimum_application_version = 0;
        syntax.has_custom_hdr_reference_white_flag =
            (self.fHdrReferenceWhite != Self::kDefaultHdrReferenceWhite) as u8;
        if syntax.has_custom_hdr_reference_white_flag != 0 {
            syntax.hdr_reference_white =
                unsafe { float_to_uint16(self.fHdrReferenceWhite, 1, 50000, 0, 5.0) };
        }
        if let Some(hatm) = &self.fHeadroomAdaptiveToneMap {
            syntax.has_adaptive_tone_map_flag = 1;
            syntax.baseline_hdr_headroom =
                unsafe { float_to_uint16(hatm.fBaselineHdrHeadroom, 0, 60000, 0, 10000.0) };
            syntax.use_reference_white_tone_mapping_flag = 0;
            {
                let mut hatm_rwtmo = hatm.clone();
                PopulateUsingRwtmo(&mut hatm_rwtmo);
                syntax.use_reference_white_tone_mapping_flag = (hatm == &hatm_rwtmo) as u8;
            }
            crate::SkASSERT!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),552;@condition b"hatm.fAlternateImages.size() <= HeadroomAdaptiveToneMap::kMaxNumAlternateImages\0".as_ptr();hatm.fAlternateImages.len()<=HeadroomAdaptiveToneMap::kMaxNumAlternateImages);
            syntax.num_alternate_images = hatm.fAlternateImages.len() as u8;
            for a in 0..hatm.fAlternateImages.len() {
                syntax.alternate_hdr_headrooms[a] = unsafe {
                    float_to_uint16(hatm.fAlternateImages[a].fHdrHeadroom, 0, 60000, 0, 10000.0)
                };
            }
            syntax.gain_application_space_chromaticities_flag =
                if hatm.fGainApplicationSpacePrimaries == SkNamedPrimaries::kRec709 {
                    0
                } else if hatm.fGainApplicationSpacePrimaries == SkNamedPrimaries::kSMPTE_EG_432_1 {
                    1
                } else if hatm.fGainApplicationSpacePrimaries == SkNamedPrimaries::kRec2020 {
                    2
                } else {
                    3
                };
            if syntax.gain_application_space_chromaticities_flag == 3 {
                let p = &hatm.fGainApplicationSpacePrimaries;
                syntax.gain_application_space_chromaticities[0] =
                    unsafe { float_to_uint16(p.fRX, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[1] =
                    unsafe { float_to_uint16(p.fRY, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[2] =
                    unsafe { float_to_uint16(p.fGX, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[3] =
                    unsafe { float_to_uint16(p.fGY, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[4] =
                    unsafe { float_to_uint16(p.fBX, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[5] =
                    unsafe { float_to_uint16(p.fBY, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[6] =
                    unsafe { float_to_uint16(p.fWX, 0, 50000, 0, 50000.0) };
                syntax.gain_application_space_chromaticities[7] =
                    unsafe { float_to_uint16(p.fWY, 0, 50000, 0, 50000.0) };
            }
            syntax.has_common_component_mix_params_flag = 0;
            for a in 0..hatm.fAlternateImages.len() {
                let mix = &hatm.fAlternateImages[a].fColorGainFunction.fComponentMixing;
                if mix.fRed == 0.0
                    && mix.fGreen == 0.0
                    && mix.fBlue == 0.0
                    && mix.fMax == 1.0
                    && mix.fMin == 0.0
                    && mix.fComponent == 0.0
                {
                    syntax.component_mixing_type[a] = 0;
                } else if mix.fRed == 0.0
                    && mix.fGreen == 0.0
                    && mix.fBlue == 0.0
                    && mix.fMax == 0.0
                    && mix.fMin == 0.0
                    && mix.fComponent == 1.0
                {
                    syntax.component_mixing_type[a] = 1;
                } else if mix.fRed == 1.0 / 6.0
                    && mix.fGreen == 1.0 / 6.0
                    && mix.fBlue == 1.0 / 6.0
                    && mix.fMax == 1.0 / 2.0
                    && mix.fMin == 0.0
                    && mix.fComponent == 0.0
                {
                    syntax.component_mixing_type[a] = 2;
                } else {
                    syntax.component_mixing_type[a] = 3;
                    syntax.component_mixing_coefficient[a][0] =
                        unsafe { float_to_uint16(mix.fRed, 0, 50000, 0, 50000.0) };
                    syntax.component_mixing_coefficient[a][1] =
                        unsafe { float_to_uint16(mix.fGreen, 0, 50000, 0, 50000.0) };
                    syntax.component_mixing_coefficient[a][2] =
                        unsafe { float_to_uint16(mix.fBlue, 0, 50000, 0, 50000.0) };
                    syntax.component_mixing_coefficient[a][3] =
                        unsafe { float_to_uint16(mix.fMax, 0, 50000, 0, 50000.0) };
                    syntax.component_mixing_coefficient[a][4] =
                        unsafe { float_to_uint16(mix.fMin, 0, 50000, 0, 50000.0) };
                    syntax.component_mixing_coefficient[a][5] =
                        unsafe { float_to_uint16(mix.fComponent, 0, 50000, 0, 50000.0) };
                    for k in 0..6 {
                        syntax.has_component_mixing_coefficient_flag[a][k] =
                            (syntax.component_mixing_coefficient[a][k] != 0) as u8;
                    }
                }
            }
            syntax.has_common_curve_params_flag = 0;
            for a in 0..hatm.fAlternateImages.len() {
                let alt = &hatm.fAlternateImages[a];
                let sign = if hatm.fBaselineHdrHeadroom < alt.fHdrHeadroom {
                    1.0
                } else {
                    -1.0
                };
                let cubic = &alt.fColorGainFunction.fGainCurve;
                crate::SkASSERT!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),629;@condition b"GainCurve::kMinNumControlPoints <= cubic.fControlPoints.size()\0".as_ptr();GainCurve::kMinNumControlPoints<=cubic.fControlPoints.len());
                crate::SkASSERT!(@source b"src/codec/SkHdrAgtmParse.cpp\0".as_ptr(),630;@condition b"cubic.fControlPoints.size() <= GainCurve::kMaxNumControlPoints\0".as_ptr();cubic.fControlPoints.len()<=GainCurve::kMaxNumControlPoints);
                syntax.gain_curve_num_control_points_minus_1[a] =
                    (cubic.fControlPoints.len() - 1) as u8;
                syntax.gain_curve_use_pchip_slope_flag[a] = 0;
                for c in 0..cubic.fControlPoints.len() {
                    syntax.gain_curve_control_points_x[a][c] =
                        unsafe { float_to_uint16(cubic.fControlPoints[c].fX, 0, 64000, 0, 1000.0) };
                }
                for c in 0..cubic.fControlPoints.len() {
                    syntax.gain_curve_control_points_y[a][c] = unsafe {
                        float_to_uint16(sign * cubic.fControlPoints[c].fY, 0, 60000, 0, 10000.0)
                    };
                }
                for c in 0..cubic.fControlPoints.len() {
                    let theta = unsafe { atanf(cubic.fControlPoints[c].fM) };
                    syntax.gain_curve_control_points_theta[a][c] = unsafe {
                        float_to_uint16(theta, 1, 35999, 18000, 36000.0 / core::f32::consts::PI)
                    };
                }
            }
        }
        let mut s = core::pin::pin!(SkDynamicMemoryWStream::new());
        syntax.write_application_info(s.as_mut());
        Some(s.as_mut().detachAsData())
    }
}
