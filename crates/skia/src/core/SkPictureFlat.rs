// Copyright 2011 Google Inc. BSD-style license.
use super::{SkFlattenable::Factory, SkReadBuffer::SkReadBuffer, SkTypeface::SkTypefaceRef};
// Source: src/core/SkPictureFlat.h:32-128
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawType {
    UNUSED,
    CLIP_PATH,
    CLIP_REGION,
    CLIP_RECT,
    CLIP_RRECT,
    CONCAT,
    DRAW_BITMAP_RETIRED_2016_REMOVED_2018,
    DRAW_BITMAP_MATRIX_RETIRED_2016_REMOVED_2018,
    DRAW_BITMAP_NINE_RETIRED_2016_REMOVED_2018,
    DRAW_BITMAP_RECT_RETIRED_2016_REMOVED_2018,
    DRAW_CLEAR,
    DRAW_DATA,
    DRAW_OVAL,
    DRAW_PAINT,
    DRAW_PATH,
    DRAW_PICTURE,
    DRAW_POINTS,
    DRAW_POS_TEXT_REMOVED_1_2019,
    DRAW_POS_TEXT_TOP_BOTTOM_REMOVED_1_2019,
    DRAW_POS_TEXT_H_REMOVED_1_2019,
    DRAW_POS_TEXT_H_TOP_BOTTOM_REMOVED_1_2019,
    DRAW_RECT,
    DRAW_RRECT,
    DRAW_SPRITE_RETIRED_2015_REMOVED_2018,
    DRAW_TEXT_REMOVED_1_2019,
    DRAW_TEXT_ON_PATH_RETIRED_08_2018_REMOVED_10_2018,
    DRAW_TEXT_TOP_BOTTOM_REMOVED_1_2019,
    DRAW_VERTICES_RETIRED_03_2017_REMOVED_01_2018,
    RESTORE,
    ROTATE,
    SAVE,
    SAVE_LAYER_SAVEFLAGS_DEPRECATED_2015_REMOVED_12_2020,
    SCALE,
    SET_MATRIX,
    SKEW,
    TRANSLATE,
    NOOP,
    BEGIN_COMMENT_GROUP_obsolete,
    COMMENT_obsolete,
    END_COMMENT_GROUP_obsolete,
    DRAW_DRRECT,
    PUSH_CULL,
    POP_CULL,
    DRAW_PATCH,
    DRAW_PICTURE_MATRIX_PAINT,
    DRAW_TEXT_BLOB,
    DRAW_IMAGE,
    DRAW_IMAGE_RECT_STRICT_obsolete,
    DRAW_ATLAS,
    DRAW_IMAGE_NINE,
    DRAW_IMAGE_RECT,
    SAVE_LAYER_SAVELAYERFLAGS_DEPRECATED_JAN_2016_REMOVED_01_2018,
    SAVE_LAYER_SAVELAYERREC,
    DRAW_ANNOTATION,
    DRAW_DRAWABLE,
    DRAW_DRAWABLE_MATRIX,
    DRAW_TEXT_RSXFORM_DEPRECATED_DEC_2018,
    TRANSLATE_Z,
    DRAW_SHADOW_REC,
    DRAW_IMAGE_LATTICE,
    DRAW_ARC,
    DRAW_REGION,
    DRAW_VERTICES_OBJECT,
    FLUSH,
    DRAW_EDGEAA_IMAGE_SET,
    SAVE_BEHIND,
    DRAW_EDGEAA_QUAD,
    DRAW_BEHIND_PAINT,
    CONCAT44,
    CLIP_SHADER_IN_PAINT,
    MARK_CTM,
    SET_M44,
    DRAW_IMAGE2,
    DRAW_IMAGE_RECT2,
    DRAW_IMAGE_LATTICE2,
    DRAW_EDGEAA_IMAGE_SET2,
    RESET_CLIP,
    DRAW_SLUG,
}
pub use DrawType::*;
pub const LAST_DRAWTYPE_ENUM: u32 = DrawType::DRAW_SLUG as u32;
// Source: src/core/SkPictureFlat.h:130-135
pub const DRAW_VERTICES_HAS_TEXS: u32 = 0x01;
pub const DRAW_VERTICES_HAS_COLORS: u32 = 0x02;
pub const DRAW_VERTICES_HAS_INDICES: u32 = 0x04;
pub const DRAW_VERTICES_HAS_XFER: u32 = 0x08;
// Source: src/core/SkPictureFlat.h:137-141
pub const DRAW_ATLAS_HAS_COLORS: u32 = 1 << 0;
pub const DRAW_ATLAS_HAS_CULL: u32 = 1 << 1;
pub const DRAW_ATLAS_HAS_SAMPLING: u32 = 1 << 2;
// Source: src/core/SkPictureFlat.h:143-145
pub const DRAW_TEXT_RSXFORM_HAS_CULL: u32 = 1 << 0;
// Source: src/core/SkPictureFlat.h:147-157
pub const SAVELAYERREC_HAS_BOUNDS: u32 = 1 << 0;
pub const SAVELAYERREC_HAS_PAINT: u32 = 1 << 1;
pub const SAVELAYERREC_HAS_BACKDROP: u32 = 1 << 2;
pub const SAVELAYERREC_HAS_FLAGS: u32 = 1 << 3;
pub const SAVELAYERREC_HAS_CLIPMASK_OBSOLETE: u32 = 1 << 4;
pub const SAVELAYERREC_HAS_CLIPMATRIX_OBSOLETE: u32 = 1 << 5;
pub const SAVELAYERREC_HAS_BACKDROP_SCALE: u32 = 1 << 6;
pub const SAVELAYERREC_HAS_MULTIPLE_FILTERS: u32 = 1 << 7;
pub const SAVELAYERREC_HAS_BACKDROP_TILEMODE: u32 = 1 << 8;
// Source: src/core/SkPictureFlat.h:159-161
pub const SAVEBEHIND_HAS_SUBSET: u32 = 1 << 0;
// Source: src/core/SkPictureFlat.h:168-171
pub fn ClipParams_pack(op: crate::include::core::SkClipOp::SkClipOp, doAA: bool) -> u32 {
    ((doAA as u32) << 4) | (op as u32)
}
// Source: src/core/SkPictureFlat.h:175-183
pub fn ClipParams_unpackRegionOp(
    buffer: &mut SkReadBuffer<'_>,
    packed: u32,
) -> super::SkRegion::Op {
    use super::SkRegion::Op;
    let unpacked = packed & 0xf;
    if buffer.validate(
        unpacked <= Op::Intersect as u32
            || (unpacked <= Op::Replace as u32
                && buffer.isVersionLT(super::SkPicturePriv::kNoExpandingClipOps)),
    ) {
        match unpacked {
            0 => Op::Difference,
            1 => Op::Intersect,
            2 => Op::Union,
            3 => Op::Xor,
            4 => Op::ReverseDifference,
            5 => Op::Replace,
            _ => unreachable!(),
        }
    } else {
        Op::Intersect
    }
}
// Source: src/core/SkPictureFlat.h:185-187
pub fn ClipParams_unpackDoAA(packed: u32) -> bool {
    ((packed >> 4) & 1) != 0
}
// Source: src/core/SkPictureFlat.h:191-212
pub struct SkTypefacePlayback {
    fCount: usize,
    fArray: Box<[Option<SkTypefaceRef>]>,
}
impl SkTypefacePlayback {
    pub fn new() -> Self {
        Self {
            fCount: 0,
            fArray: Box::new([]),
        }
    }
    pub fn count(&self) -> usize {
        self.fCount
    }
    pub fn get_mut(&mut self, index: usize) -> &mut Option<SkTypefaceRef> {
        crate::SkASSERT!(@source b"src/core/SkPictureFlat.h\0".as_ptr(),201; @condition b"index < fCount\0".as_ptr(); index<self.fCount);
        &mut self.fArray[index]
    }
    pub fn setupBuffer<'a>(&'a self, buffer: &mut SkReadBuffer<'a>) {
        buffer.setTypefaceArray(&self.fArray);
    }
    // Source: src/core/SkPictureFlat.cpp:16-19
    pub fn setCount(&mut self, count: usize) {
        self.fCount = count;
        let mut replacement = vec![None; count].into_boxed_slice();
        core::mem::swap(&mut self.fArray, &mut replacement);
        // unique_ptr<sk_sp<Typeface>[]> destroys its old array last-to-first.
        for slot in replacement.iter_mut().rev() {
            drop(slot.take());
        }
        drop(replacement);
    }
}
// Source: src/core/SkPictureFlat.h:214-229
pub struct SkFactoryPlayback {
    fCount: i32,
    fArray: Box<[Option<Factory>]>,
}
impl SkFactoryPlayback {
    pub fn new(count: i32) -> Self {
        Self {
            fCount: count,
            fArray: vec![None; count as usize].into_boxed_slice(),
        }
    }
    pub fn base(&mut self) -> &mut [Option<Factory>] {
        &mut self.fArray
    }
    pub fn setupBuffer<'a>(&'a self, buffer: &mut SkReadBuffer<'a>) {
        buffer.setFactoryPlayback(&self.fArray);
    }
}

// Native unique_ptr array destruction releases the last slot first, then the
// allocation. Empty slots remain for Rust's automatic final array deallocation.
impl Drop for SkTypefacePlayback {
    fn drop(&mut self) {
        for slot in self.fArray.iter_mut().rev() {
            drop(slot.take());
        }
    }
}
