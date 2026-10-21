use crate::src::core::SkFont::SkFont;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRSXform::SkRSXform;
// Source: src/core/SkTextBlobPriv.h:25-25
pub struct SkTextBlobPriv;
// Source: src/core/SkTextBlobPriv.h:64-64
#[cfg(sk_debug)]
pub const kRunRecordMagic: u32 = 0xb10bcafe;
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:68-73
    // Source: src/core/SkTextBlobPriv.h:75-80
    /// Safety: p names aligned uninitialized storage (never an already live record),
    /// with a successful StorageSize allocation for count/text_size/pos. The font source
    /// must be live and independent of the destination storage while it is cloned.
    /// All fields and the extended text-size slot are initialized without moving the record.
    pub(crate) unsafe fn init_at(
        p: *mut Self,
        count: u32,
        text_size: u32,
        offset: SkPoint,
        font: &SkFont,
        pos: GlyphPositioning,
    ) {
        p.write(Self {
            fFont: core::mem::ManuallyDrop::new(font.clone()),
            fCount: count,
            fOffset: offset,
            fFlags: pos as u32,
            #[cfg(sk_debug)]
            fMagic: kRunRecordMagic,
        });
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),73; @condition b"static_cast<unsigned>(pos) <= Flags::kPositioning_Mask\0".as_ptr(); (pos as u32) <= Self::POSITIONING_MASK);
        if text_size > 0 {
            (*p).fFlags |= Self::EXTENDED_FLAG;
            *(*p).textSizePtr() = text_size
        }
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:82-84
    pub fn glyphCount(&self) -> u32 {
        self.fCount
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:86-88
    pub fn offset(&self) -> &SkPoint {
        &self.fOffset
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:90-92
    pub fn font(&self) -> &SkFont {
        &self.fFont
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:94-96
    pub fn positioning(&self) -> GlyphPositioning {
        match self.fFlags & Self::POSITIONING_MASK {
            0 => GlyphPositioning::kDefault_Positioning,
            1 => GlyphPositioning::kHorizontal_Positioning,
            2 => GlyphPositioning::kFull_Positioning,
            _ => GlyphPositioning::kRSXform_Positioning,
        }
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:98-99
    // Source: src/core/SkTextBlobPriv.h:101-102
    pub unsafe fn glyphBuffer(&self) -> *mut u16 {
        debug_assert_eq!(
            std::mem::size_of::<Self>() % std::mem::size_of::<usize>(),
            0
        );
        (self as *const Self).add(1).cast_mut().cast()
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:105-105
    // Source: src/core/SkTextBlobPriv.h:107-109
    pub unsafe fn posBuffer(&self) -> *mut f32 {
        let bytes = (self.fCount as usize).wrapping_mul(std::mem::size_of::<u16>());
        self.glyphBuffer()
            .cast::<u8>()
            .add(bytes.wrapping_add(3) & !3)
            .cast()
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:112-115
    pub unsafe fn pointBuffer(&self) -> *mut SkPoint {
        debug_assert_eq!(self.positioning(), GlyphPositioning::kFull_Positioning);
        self.posBuffer().cast()
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:118-121
    pub unsafe fn xformBuffer(&self) -> *mut SkRSXform {
        debug_assert_eq!(self.positioning(), GlyphPositioning::kRSXform_Positioning);
        self.posBuffer().cast()
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:123-123
    pub unsafe fn textSize(&self) -> u32 {
        if self.isExtended() {
            *self.textSizePtr()
        } else {
            0
        }
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:125-125
    // Source: src/core/SkTextBlobPriv.h:127-128
    pub unsafe fn clusterBuffer(&self) -> *mut u32 {
        if self.isExtended() {
            self.textSizePtr().add(1)
        } else {
            std::ptr::null_mut()
        }
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:130-134
    pub unsafe fn textBuffer(&self) -> *mut u8 {
        if self.isExtended() {
            self.clusterBuffer().add(self.fCount as usize).cast()
        } else {
            std::ptr::null_mut()
        }
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:136-136
    pub fn isLastRun(&self) -> bool {
        self.fFlags & Self::LAST_FLAG != 0
    }
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:151-155
    pub(crate) const POSITIONING_MASK: u32 = 0x03;
}
impl RunRecord {
    pub(crate) const LAST_FLAG: u32 = 0x04;
}
impl RunRecord {
    pub(crate) const EXTENDED_FLAG: u32 = 0x08;
}
impl RunRecord {
    // Source: src/core/SkTextBlobPriv.h:167-169
    pub(crate) fn isExtended(&self) -> bool {
        self.fFlags & Self::EXTENDED_FLAG != 0
    }
}
// Adapted source: src/core/SkTextBlobPriv.h:66-67: flat Rust RunRecord storage declaration separated from native inline constructor/accessor scope
// Source: src/core/SkTextBlobPriv.h:171-174
// Source: src/core/SkTextBlobPriv.h:176-177
#[repr(C)]
pub struct RunRecord {
    pub(crate) fFont: core::mem::ManuallyDrop<SkFont>,
    pub(crate) fCount: u32,
    pub(crate) fOffset: SkPoint,
    pub(crate) fFlags: u32,
    #[cfg(sk_debug)]
    pub(crate) fMagic: u32,
}
// Source: src/core/SkTextBlobPriv.h:171-177 (implicit member destruction)
// The trailing integer/point/debug members have trivial destruction. The font
// is the final owning member destroyed, at its original inline record address.
impl Drop for RunRecord {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.fFont);
        }
    }
}
// Source: src/core/SkTextBlobPriv.h:189-194
// UNVERIFIED public type-surface adapter: iterator and blob share the already
// canonical repr(u8) values 0/1/2/3. No second enum or incompatible values created.
pub use crate::src::core::SkTextBlob::GlyphPositioning;

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:196-198
    pub fn done(&self) -> bool {
        self.fCurrentRun.is_null()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:201-204
    pub unsafe fn glyphCount(&self) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),202; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).glyphCount()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:205-208
    pub unsafe fn glyphs(&self) -> *const u16 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),206; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).glyphBuffer()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:209-212
    pub unsafe fn pos(&self) -> *const f32 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),210; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).posBuffer()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:214-216
    pub unsafe fn points(&self) -> *const SkPoint {
        (*self.fCurrentRun).pointBuffer()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:218-220
    pub unsafe fn xforms(&self) -> *const SkRSXform {
        (*self.fCurrentRun).xformBuffer()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:221-224
    pub unsafe fn offset(&self) -> &SkPoint {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),222; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).offset()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:225-228
    pub unsafe fn font(&self) -> &SkFont {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),226; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).font()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:231-234
    pub unsafe fn clusters(&self) -> *mut u32 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),232; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).clusterBuffer()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:235-238
    pub unsafe fn textSize(&self) -> u32 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),236; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).textSize()
    }
}
impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlobPriv.h:239-242
    pub unsafe fn text(&self) -> *mut u8 {
        crate::SkASSERT!(@source b"src/core/SkTextBlobPriv.h\0".as_ptr(),240; @condition b"!this->done()\0".as_ptr(); !self.done());
        (*self.fCurrentRun).textBuffer()
    }
}
// Adapted source: src/core/SkTextBlobPriv.h:185-186: flat Rust iterator storage declaration separated from native inline accessors
// Structural source: src/core/SkTextBlobPriv.h:187-187: bodyless constructor prototype; actual original cpp221-224 implementation exists
// Source: src/core/SkTextBlobPriv.h:247-247
// Source: src/core/SkTextBlobPriv.h:249-250
/// Nonowning iterator. The blob and its immutable contiguous run storage must outlive it.
pub struct SkTextBlobRunIterator {
    pub(crate) fCurrentRun: *const RunRecord,
    #[cfg(sk_debug)]
    pub(crate) fStorageTop: *const u8,
}

// Source: src/core/SkTextBlobPriv.h:41
// HasRSXForm(const SkTextBlob&) -> bool is an ordinary static declaration.
// Its previously known missing native definition remains pending. It is not a
// virtual interface and is not replaced by a provider trait or alternate scan.
