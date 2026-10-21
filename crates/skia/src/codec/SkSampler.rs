// Source: src/codec/SkSampler.h:17-17
// The native forward declaration resolves to the existing complete value type.
use super::SkCodec::ZeroInitialized;
use crate::include::core::SkColorType::SkColorType::*;
use crate::src::core::SkImageInfo::SkImageInfo;
// Source: src/codec/SkSampler.h:78-84
pub struct SkSampler {
    fSampleY: i32,
}
impl SkSampler {
    pub const fn new() -> Self {
        Self { fSampleY: 1 }
    }
    // Source: src/codec/SkSampler.h:32-34
    pub fn setSampleY(&mut self, sampleY: i32) {
        self.fSampleY = sampleY;
    }
    // Source: src/codec/SkSampler.h:39-41
    pub fn sampleY(&self) -> i32 {
        self.fSampleY
    }
    // Source: src/codec/SkSampler.h:48-50
    pub fn rowNeeded(&self, row: i32) -> bool {
        (row - super::SkCodecPriv::SkCodecPriv::GetStartCoord(self.fSampleY)) % self.fSampleY == 0
    }
    // Source: src/codec/SkSampler.cpp:20-71
    /// Safety: dst is nonnull, correctly aligned for the selected pixel type,
    /// and supplies info.height writable rows of info.width pixels at rowBytes
    /// pitch. The original image/count and nonoverlapping row contracts apply.
    pub unsafe fn Fill(
        info: &SkImageInfo,
        dst: *mut u8,
        rowBytes: usize,
        zeroInit: ZeroInitialized,
    ) {
        crate::SkASSERT!(@source b"src/codec/SkSampler.cpp\0".as_ptr(),22; @condition b"dst != nullptr\0".as_ptr(); !dst.is_null());
        if zeroInit == ZeroInitialized::kYes_ZeroInitialized {
            return;
        }
        let width = info.width();
        let numRows = info.height();
        match info.colorType() {
            kRGBA_8888_SkColorType | kBGRA_8888_SkColorType => {
                let mut dstRow = dst.cast::<u32>();
                for _ in 0..numRows {
                    unsafe {
                        crate::src::core::SkMemset::memset32(dstRow, 0, width);
                        dstRow = dstRow.cast::<u8>().wrapping_add(rowBytes).cast();
                    }
                }
            }
            kRGB_565_SkColorType => {
                let mut dstRow = dst.cast::<u16>();
                for _ in 0..numRows {
                    unsafe {
                        crate::src::core::SkMemset::memset16(dstRow, 0, width);
                        dstRow = dstRow.cast::<u8>().wrapping_add(rowBytes).cast();
                    }
                }
            }
            kGray_8_SkColorType => {
                let mut dstRow = dst;
                for _ in 0..numRows {
                    unsafe {
                        core::ptr::write_bytes(dstRow, 0, width as usize);
                        dstRow = dstRow.wrapping_add(rowBytes);
                    }
                }
            }
            kRGBA_F16_SkColorType => {
                let mut dstRow = dst.cast::<u64>();
                for _ in 0..numRows {
                    unsafe {
                        crate::src::core::SkMemset::memset64(dstRow, 0, width);
                        dstRow = dstRow.cast::<u8>().wrapping_add(rowBytes).cast();
                    }
                }
            }
            _ => {
                #[cfg(SK_PRINT_CODEC_MESSAGES)]
                crate::SkDebugf!(@source b"src/codec/SkSampler.cpp\0".as_ptr(),67; b"Error: Unsupported dst color type for fill().  Doing nothing.\n\0".as_ptr());
                crate::SkASSERT!(@source b"src/codec/SkSampler.cpp\0".as_ptr(),68; @condition b"false\0".as_ptr(); false);
            }
        }
    }
}
// Source: src/codec/SkSampler.h:25-27
// Source: src/codec/SkSampler.h:76-86
// Real pure virtual methods belong to the actual sampler subclass; no default
// sampler algorithm or substitute swizzler/decoder is constructed here.
pub trait SkSamplerVirtual {
    fn sampler(&self) -> &SkSampler;
    fn sampler_mut(&mut self) -> &mut SkSampler;
    fn onSetSampleX(&mut self, sampleX: i32) -> i32;
    fn fillWidth(&self) -> i32;
    fn setSampleX(&mut self, sampleX: i32) -> i32 {
        self.onSetSampleX(sampleX)
    }
    // Source: src/codec/SkSampler.h:32-50
    fn setSampleY(&mut self, sampleY: i32) {
        self.sampler_mut().setSampleY(sampleY);
    }
    fn sampleY(&self) -> i32 {
        self.sampler().sampleY()
    }
    fn rowNeeded(&self, row: i32) -> bool {
        self.sampler().rowNeeded(row)
    }
}
