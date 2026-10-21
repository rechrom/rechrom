//! Original font wire protocol, using the actual readTypeface/MemoryStream
//! callback owner. New translation is unverified.
use super::{
    SkFont::{Edging, SkFont},
    SkFontPriv::SkFontPriv,
    SkReadBuffer::SkReadBuffer,
    SkWriteBuffer::SkWriteBuffer,
};
use crate::include::core::SkFontTypes::SkFontHinting;

// Source: src/core/SkFont_serial.cpp:24-41
const kSize_Is_Byte_Bit: u32 = 1 << 31;
const kHas_ScaleX_Bit: u32 = 1 << 30;
const kHas_SkewX_Bit: u32 = 1 << 29;
const kHas_Typeface_Bit: u32 = 1 << 28;
const kShift_for_Size: u32 = 16;
const kMask_For_Size: u32 = 0xFF;
const kShift_For_Flags: u32 = 4;
const kMask_For_Flags: u32 = 0xFFF;
const kShift_For_Edging: u32 = 2;
const kMask_For_Edging: u32 = 0x3;
const kShift_For_Hinting: u32 = 0;
const kMask_For_Hinting: u32 = 0x3;

// Source: src/core/SkFont_serial.cpp:43-46
// Rust float-to-int saturates outside the native defined conversion domain.
fn scalar_is_byte(x: f32) -> bool {
    let ix = x as i32;
    ix as f32 == x && ix >= 0 && ix <= kMask_For_Size as i32
}

impl SkFontPriv {
    // Source: src/core/SkFont_serial.cpp:48-86
    pub fn Flatten(font: &SkFont, buffer: &mut dyn SkWriteBuffer) {
        crate::SkASSERT!(@source b"src/core/SkFont_serial.cpp\0".as_ptr(),49; @condition b"font.fFlags <= SkFont::kAllFlags\0".as_ptr(); font.fFlags <= SkFont::kAllFlags);
        debug_assert_eq!(font.fFlags as u32 & !kMask_For_Flags, 0);
        debug_assert_eq!(font.fEdging as u32 & !kMask_For_Edging, 0);
        debug_assert_eq!(font.fHinting as u32 & !kMask_For_Hinting, 0);
        let mut packed = 0;
        packed |= (font.fFlags as u32) << kShift_For_Flags;
        packed |= (font.fEdging as u32) << kShift_For_Edging;
        packed |= (font.fHinting as u32) << kShift_For_Hinting;
        if scalar_is_byte(font.fSize) {
            packed |= kSize_Is_Byte_Bit;
            packed |= (font.fSize as u32) << kShift_for_Size;
        }
        if font.fScaleX != 1.0 {
            packed |= kHas_ScaleX_Bit;
        }
        if font.fSkewX != 0.0 {
            packed |= kHas_SkewX_Bit;
        }
        if font.fTypeface.get().is_some() {
            packed |= kHas_Typeface_Bit;
        }
        buffer.write32(packed as i32);
        if packed & kSize_Is_Byte_Bit == 0 {
            buffer.writeScalar(font.fSize);
        }
        if packed & kHas_ScaleX_Bit != 0 {
            buffer.writeScalar(font.fScaleX);
        }
        if packed & kHas_SkewX_Bit != 0 {
            buffer.writeScalar(font.fSkewX);
        }
        if packed & kHas_Typeface_Bit != 0 {
            buffer.writeTypeface(Some(&font.fTypeface));
        }
    }

    // Source: src/core/SkFont_serial.cpp:88-123
    // Uses the actual ReadBuffer typeface playback and MemoryStream callback.
    pub fn Unflatten(font: &mut SkFont, buffer: &mut SkReadBuffer<'_>) -> bool {
        let packed = buffer.read32() as u32;
        if packed & kSize_Is_Byte_Bit != 0 {
            font.fSize = ((packed >> kShift_for_Size) & kMask_For_Size) as f32;
        } else {
            font.fSize = buffer.readScalar();
        }
        if packed & kHas_ScaleX_Bit != 0 {
            font.fScaleX = buffer.readScalar();
        }
        if packed & kHas_SkewX_Bit != 0 {
            font.fSkewX = buffer.readScalar();
        }
        if packed & kHas_Typeface_Bit != 0 {
            font.setTypeface(buffer.readTypeface());
        }
        debug_assert!(SkFont::kAllFlags as u32 <= kMask_For_Flags);
        font.fFlags = ((packed >> kShift_For_Flags) & SkFont::kAllFlags as u32) as u8;
        let mut edging = (packed >> kShift_For_Edging) & kMask_For_Edging;
        if edging > Edging::kSubpixelAntiAlias.0 as u32 {
            edging = 0;
        }
        font.fEdging = edging as u8;
        let mut hinting = (packed >> kShift_For_Hinting) & kMask_For_Hinting;
        if hinting > SkFontHinting::kFull.0 as u32 {
            hinting = 0;
        }
        font.fHinting = hinting as u8;
        buffer.isValid()
    }
}
