use crate::include::core::SkSamplingOptions::{SkFilterMode, SkMipmapMode, SkSamplingOptions};
// Source: src/core/SkSamplingPriv.h:18-18
pub const kBicubicFilterTexelPad: i32 = 2;
// Source: src/core/SkSamplingPriv.h:22-29
#[repr(i32)]
#[derive(Clone, Copy)]
pub enum SkLegacyFQ {
    kNone_SkLegacyFQ = 0,
    kLow_SkLegacyFQ = 1,
    kMedium_SkLegacyFQ = 2,
    kHigh_SkLegacyFQ = 3,
}
// Source: src/core/SkSamplingPriv.h:28-28
pub const kLast_SkLegacyFQ: SkLegacyFQ = SkLegacyFQ::kHigh_SkLegacyFQ;
// Source: src/core/SkSamplingPriv.h:32-35
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkMediumAs {
    kNearest_SkMediumAs,
    kLinear_SkMediumAs,
}
// Source: src/core/SkSamplingPriv.h:37-38
pub struct SkSamplingPriv;
impl SkSamplingPriv {
    // Source: src/core/SkSamplingPriv.h:39-45
    pub fn FlatSize(options: &SkSamplingOptions) -> usize {
        let mut size = core::mem::size_of::<u32>();
        if !options.isAniso() {
            size += 3 * core::mem::size_of::<u32>();
        }
        size
    }
    // Source: src/core/SkSamplingPriv.h:48-48
    // Source: src/core/SkSamplingPriv.h:52-53
    pub fn NoChangeWithIdentityMatrix(sampling: &SkSamplingOptions) -> bool {
        !sampling.useCubic() || sampling.cubic().B == 0.0
    }
    // Source: src/core/SkSamplingPriv.h:59-62
    pub fn AnisoFallback(imageIsMipped: bool) -> SkSamplingOptions {
        let mm = if imageIsMipped {
            SkMipmapMode::kLinear
        } else {
            SkMipmapMode::kNone
        };
        SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, mm)
    }
}
// Source: src/core/SkSamplingPriv.h:64-78
impl SkSamplingPriv {
    pub fn FromFQ(fq: SkLegacyFQ, behavior: SkMediumAs) -> SkSamplingOptions {
        match fq {
            SkLegacyFQ::kHigh_SkLegacyFQ => SkSamplingOptions::with_cubic(
                crate::include::core::SkSamplingOptions::SkCubicResampler {
                    B: 1.0 / 3.0,
                    C: 1.0 / 3.0,
                },
            ),
            SkLegacyFQ::kMedium_SkLegacyFQ => SkSamplingOptions::with_filter_mipmap(
                SkFilterMode::kLinear,
                if behavior == SkMediumAs::kNearest_SkMediumAs {
                    SkMipmapMode::kNearest
                } else {
                    SkMipmapMode::kLinear
                },
            ),
            SkLegacyFQ::kLow_SkLegacyFQ => {
                SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kNone)
            }
            SkLegacyFQ::kNone_SkLegacyFQ => {
                SkSamplingOptions::with_filter_mipmap(SkFilterMode::kNearest, SkMipmapMode::kNone)
            }
        }
    }
}
