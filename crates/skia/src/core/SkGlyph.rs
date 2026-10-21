use crate::src::core::SkPoint::{SkIPoint, SkPoint};
// Source: src/core/SkGlyph.h:47-48
// Source: src/core/SkGlyph.h:50-51
// Source: src/core/SkGlyph.h:54-57
// Source: src/core/SkGlyph.h:60-62
// Source: src/core/SkGlyph.h:65-67
// Source: src/core/SkGlyph.h:69-70
// Source: src/core/SkGlyph.h:72-73
impl SkPackedGlyphID {
    pub const IMPOSSIBLE_ID: u32 = !0;
}
impl SkPackedGlyphID {
    pub const GLYPH_ID_LEN: u32 = 16;
}
impl SkPackedGlyphID {
    pub const SUB_PIXEL_POS_LEN: u32 = 2;
}
impl SkPackedGlyphID {
    pub const SUB_PIXEL_X: u32 = 0;
}
impl SkPackedGlyphID {
    pub const GLYPH_ID: u32 = Self::SUB_PIXEL_POS_LEN;
}
impl SkPackedGlyphID {
    pub const SUB_PIXEL_Y: u32 = Self::GLYPH_ID_LEN + Self::SUB_PIXEL_POS_LEN;
}
impl SkPackedGlyphID {
    pub const END_DATA: u32 = Self::GLYPH_ID_LEN + 2 * Self::SUB_PIXEL_POS_LEN;
}
impl SkPackedGlyphID {
    pub const GLYPH_ID_MASK: u32 = (1 << Self::GLYPH_ID_LEN) - 1;
}
impl SkPackedGlyphID {
    pub const SUB_PIXEL_POS_MASK: u32 = (1 << Self::SUB_PIXEL_POS_LEN) - 1;
}
impl SkPackedGlyphID {
    pub const MASK_ALL: u32 = (1 << Self::END_DATA) - 1;
}
impl SkPackedGlyphID {
    pub const FIXED_POINT_BINARY_POINT_POS: u32 = 16;
}
impl SkPackedGlyphID {
    pub const FIXED_POINT_SUB_PIXEL_POS_BITS: u32 =
        Self::FIXED_POINT_BINARY_POINT_POS - Self::SUB_PIXEL_POS_LEN;
}
impl SkPackedGlyphID {
    pub const SUBPIXEL_ROUND: f32 = 1. / (1 << (Self::SUB_PIXEL_POS_LEN + 1)) as f32;
}
impl SkPackedGlyphID {
    pub const XY_FIELD_MASK: SkIPoint = SkIPoint {
        fX: (Self::SUB_PIXEL_POS_MASK << Self::SUB_PIXEL_X) as i32,
        fY: (Self::SUB_PIXEL_POS_MASK << Self::SUB_PIXEL_Y) as i32,
    };
}
// Source: src/core/SkGlyph.h:75-79
pub struct PackedGlyphIDHash;
impl PackedGlyphIDHash {
    pub fn hash(&self, packed_id: SkPackedGlyphID) -> u32 {
        packed_id.hash()
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:81-82
    // Source: src/core/SkGlyph.h:84-85
    // Source: src/core/SkGlyph.h:87-88
    // Source: src/core/SkGlyph.h:90-91
    // Source: src/core/SkGlyph.h:93-94
    pub const fn from_glyph(glyph_id: u16) -> Self {
        Self {
            id: (glyph_id as u32) << Self::GLYPH_ID,
        }
    }
}
impl SkPackedGlyphID {
    pub const fn from_fixed(glyph_id: u16, x: i32, y: i32) -> Self {
        Self {
            id: Self::pack_id_xy(glyph_id, x, y),
        }
    }
}
impl SkPackedGlyphID {
    pub const fn from_subpixels(glyph_id: u16, x: u32, y: u32) -> Self {
        Self {
            id: Self::pack_id_sub_x_sub_y(glyph_id, x, y),
        }
    }
}
impl SkPackedGlyphID {
    pub fn from_point(glyph_id: u16, pt: SkPoint, mask: SkIPoint) -> Self {
        Self {
            id: Self::pack_id_point(glyph_id, pt, mask),
        }
    }
}
impl SkPackedGlyphID {
    pub const fn from_value(value: u32) -> Self {
        Self {
            id: value & Self::MASK_ALL,
        }
    }
}
impl SkPackedGlyphID {
    pub const fn impossible() -> Self {
        Self {
            id: Self::IMPOSSIBLE_ID,
        }
    }
}
// Source: src/core/SkGlyph.h:94
impl Default for SkPackedGlyphID {
    fn default() -> Self {
        Self::impossible()
    }
}
impl PartialEq for SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:96-98
    fn eq(&self, that: &Self) -> bool {
        self.id == that.id
    }
    // Source: src/core/SkGlyph.h:99-101
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}
impl Eq for SkPackedGlyphID {}
impl PartialOrd for SkPackedGlyphID {
    fn partial_cmp(&self, that: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&that.id))
    }
    // Source: src/core/SkGlyph.h:102-104
    fn lt(&self, that: &Self) -> bool {
        self.id < that.id
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:106-108
    // Source: src/core/SkGlyph.h:110-112
    // Source: src/core/SkGlyph.h:114-116
    // Source: src/core/SkGlyph.h:118-120
    pub const fn glyph_id(self) -> u16 {
        ((self.id >> Self::GLYPH_ID) & Self::GLYPH_ID_MASK) as u16
    }
}
impl SkPackedGlyphID {
    pub const fn value(self) -> u32 {
        self.id
    }
}
impl SkPackedGlyphID {
    pub const fn sub_x_fixed(self) -> i32 {
        self.sub_to_fixed(Self::SUB_PIXEL_X)
    }
}
impl SkPackedGlyphID {
    pub const fn sub_y_fixed(self) -> i32 {
        self.sub_to_fixed(Self::SUB_PIXEL_Y)
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:122-124
    pub fn hash(self) -> u32 {
        crate::src::core::SkChecksum::CheapMix(self.id)
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:126-130
    // Source: src/core/SkGlyph.h:132-138
    pub fn dump(self) -> String {
        let mut text = crate::src::core::SkString::SkString::new();
        unsafe {
            text.appendf(
                b"glyphID: %d, x: %d, y:%d\0".as_ptr(),
                self.glyph_id() as i32,
                self.sub_x_fixed(),
                self.sub_y_fixed(),
            );
        }
        // Retain the existing Rust return facade; this numeric-only C format
        // emits ASCII. No Unicode replacement or printf rule substitution.
        String::from_utf8(unsafe { core::slice::from_raw_parts(text.data(), text.size()) }.to_vec())
            .expect("native numeric dump is ASCII")
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:132-138
    pub fn short_dump(self) -> String {
        let mut text = crate::src::core::SkString::SkString::new();
        unsafe {
            text.appendf(
                b"0x%x|%1u|%1u\0".as_ptr(),
                self.glyph_id() as i32,
                self.sub_pixel_field(Self::SUB_PIXEL_X),
                self.sub_pixel_field(Self::SUB_PIXEL_Y),
            );
        }
        String::from_utf8(unsafe { core::slice::from_raw_parts(text.data(), text.size()) }.to_vec())
            .expect("native numeric dump is ASCII")
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:141-143
    // Source: src/core/SkGlyph.h:145-146
    const fn pack_id_sub_x_sub_y(glyph_id: u16, x: u32, y: u32) -> u32 {
        debug_assert!(x < (1 << Self::SUB_PIXEL_POS_LEN));
        debug_assert!(y < (1 << Self::SUB_PIXEL_POS_LEN));
        (x << Self::SUB_PIXEL_X) | (y << Self::SUB_PIXEL_Y) | ((glyph_id as u32) << Self::GLYPH_ID)
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:164-164
    fn pack_id_point(glyph_id: u16, pt: SkPoint, mask: SkIPoint) -> u32 {
        // Source: src/core/SkGlyph.h:177-178
        // Source: src/core/SkGlyph.h:180-187
        // Source: src/core/SkGlyph.h:190-193
        let magic_x = (1 << (Self::SUB_PIXEL_POS_LEN + Self::SUB_PIXEL_X)) as f32;
        let magic_y = (1 << (Self::SUB_PIXEL_POS_LEN + Self::SUB_PIXEL_Y)) as f32;
        let mut x = pt.x();
        let mut y = pt.y();
        x = (x - x.floor()) + 1.;
        y = (y - y.floor()) + 1.;
        let sub = [
            (x * magic_x) as i32 & mask.x(),
            (y * magic_y) as i32 & mask.y(),
        ];
        debug_assert!((sub[0] as u32) / (1 << Self::SUB_PIXEL_X) < (1 << Self::SUB_PIXEL_POS_LEN));
        debug_assert!((sub[1] as u32) / (1 << Self::SUB_PIXEL_Y) < (1 << Self::SUB_PIXEL_POS_LEN));
        ((glyph_id as u32) << Self::GLYPH_ID) | sub[0] as u32 | sub[1] as u32
    }
}
impl SkPackedGlyphID {
    // Source: src/core/SkGlyph.h:195-197
    // Source: src/core/SkGlyph.h:199-201
    // Source: src/core/SkGlyph.h:203-205
    // Source: src/core/SkGlyph.h:207-210
    const fn pack_id_xy(glyph_id: u16, x: i32, y: i32) -> u32 {
        Self::pack_id_sub_x_sub_y(glyph_id, Self::fixed_to_sub(x), Self::fixed_to_sub(y))
    }
}
impl SkPackedGlyphID {
    const fn fixed_to_sub(n: i32) -> u32 {
        ((n as u32) >> Self::FIXED_POINT_SUB_PIXEL_POS_BITS) & Self::SUB_PIXEL_POS_MASK
    }
}
impl SkPackedGlyphID {
    const fn sub_pixel_field(self, position: u32) -> u32 {
        (self.id >> position) & Self::SUB_PIXEL_POS_MASK
    }
}
impl SkPackedGlyphID {
    const fn sub_to_fixed(self, position: u32) -> i32 {
        (self.sub_pixel_field(position) << Self::FIXED_POINT_SUB_PIXEL_POS_BITS) as i32
    }
}
// Source: src/core/SkGlyph.h:212-213
#[derive(Clone, Copy, Debug, Ord)]
pub struct SkPackedGlyphID {
    id: u32,
}
// Source: src/core/SkGlyph.h:218-222
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkAxisAlignment(pub u32);
impl SkAxisAlignment {
    pub const None: Self = Self(0);
    pub const X: Self = Self(1);
    pub const Y: Self = Self(2);
}
// Source: src/core/SkGlyph.h:231-235
// Source: src/core/SkGlyph.h:237-241
pub struct SkGlyphPositionRoundingSpec {
    pub half_axis_sample_freq: SkPoint,
    pub ignore_position_mask: SkIPoint,
    pub ignore_position_field_mask: SkIPoint,
}
// Source: src/core/SkGlyph.h:251-252
// A four-element array replaces skvx storage; operations retain lane order and comparison semantics.
#[derive(Clone, Copy, Debug)]
pub struct SkGlyphRect {
    rect: [f32; 4],
}
// Source: src/core/SkGlyph.h:254-277
impl SkGlyphRect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            rect: [-left, -top, right, bottom],
        }
    }
}
impl SkGlyphRect {
    pub fn empty(&self) -> bool {
        -self.rect[0] >= self.rect[2] || -self.rect[1] >= self.rect[3]
    }
}
impl SkGlyphRect {
    pub fn rect(&self) -> crate::src::core::SkRect::SkRect {
        crate::src::core::SkRect::SkRect::MakeLTRB(
            -self.rect[0],
            -self.rect[1],
            self.rect[2],
            self.rect[3],
        )
    }
}
impl SkGlyphRect {
    pub fn offset(&self, x: f32, y: f32) -> Self {
        Self {
            rect: [
                self.rect[0] - x,
                self.rect[1] - y,
                self.rect[2] + x,
                self.rect[3] + y,
            ],
        }
    }
}
impl SkGlyphRect {
    pub fn offset_point(&self, pt: SkPoint) -> Self {
        self.offset(pt.x(), pt.y())
    }
}
impl SkGlyphRect {
    pub fn scale_and_offset(&self, scale: f32, offset: SkPoint) -> Self {
        Self {
            rect: [
                self.rect[0] * scale - offset.x(),
                self.rect[1] * scale - offset.y(),
                self.rect[2] * scale + offset.x(),
                self.rect[3] * scale + offset.y(),
            ],
        }
    }
}
impl SkGlyphRect {
    pub fn inset(&self, dx: f32, dy: f32) -> Self {
        Self {
            rect: [
                self.rect[0] - dx,
                self.rect[1] - dy,
                self.rect[2] - dx,
                self.rect[3] - dy,
            ],
        }
    }
}
impl SkGlyphRect {
    pub fn left_top(&self) -> SkPoint {
        let p = self.neg_left_top();
        SkPoint::make(-p.x(), -p.y())
    }
}
impl SkGlyphRect {
    pub fn right_bottom(&self) -> SkPoint {
        SkPoint::make(self.rect[2], self.rect[3])
    }
}
impl SkGlyphRect {
    pub fn width_height(&self) -> SkPoint {
        self.right_bottom() + self.neg_left_top()
    }
}
impl SkGlyphRect {
    // Source: src/core/SkGlyph.h:282-286
    fn neg_left_top(&self) -> SkPoint {
        SkPoint::make(self.rect[0], self.rect[1])
    }
}
// Source: src/core/SkGlyph.h:288-302
pub mod skglyph {
    use super::SkGlyphRect;
    pub fn empty_rect() -> SkGlyphRect {
        let max = f32::MAX;
        SkGlyphRect::new(max, max, -max, -max)
    }
    pub fn full_rect() -> SkGlyphRect {
        let max = f32::MAX;
        SkGlyphRect::new(-max, -max, max, max)
    }
    pub fn rect_union(a: SkGlyphRect, b: SkGlyphRect) -> SkGlyphRect {
        SkGlyphRect {
            rect: std::array::from_fn(|i| {
                if a.rect[i] < b.rect[i] {
                    b.rect[i]
                } else {
                    a.rect[i]
                }
            }),
        }
    }
    pub fn rect_intersection(a: SkGlyphRect, b: SkGlyphRect) -> SkGlyphRect {
        SkGlyphRect {
            rect: std::array::from_fn(|i| {
                if b.rect[i] < a.rect[i] {
                    b.rect[i]
                } else {
                    a.rect[i]
                }
            }),
        }
    }
}
// Source: src/core/SkGlyph.h:304-310
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphAction {
    kUnset,
    kAccept,
    kReject,
    kDrop,
    kSize,
}
// Source: src/core/SkGlyph.h:312-319
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionType {
    kDirectMask = 0,
    kDirectMaskCPU = 2,
    kMask = 4,
    kSDFT = 6,
    kPath = 8,
    kDrawable = 10,
}
// Source: src/core/SkGlyph.h:321-324
pub const kTotalBits: u32 = 12;
impl SkGlyphDigest {
    // Source: src/core/SkGlyph.h:333-333
    pub const kSkSideTooBigForAtlas: u16 = 256;
    // Source: src/core/SkGlyph.h:338-338
    pub fn index(&self) -> i32 {
        ((self.packed >> 20) & 0xfffff) as i32
    }
    // Source: src/core/SkGlyph.h:339-339
    pub fn isEmpty(&self) -> bool {
        self.packed & (1 << 40) != 0
    }
    // Source: src/core/SkGlyph.h:340-340
    pub fn isColor(&self) -> bool {
        self.maskFormat() == crate::src::core::SkMask::Format::kARGB32_Format
    }
    // Source: src/core/SkGlyph.h:341-341
    pub fn maskFormat(&self) -> crate::src::core::SkMask::Format {
        use crate::src::core::SkMask::Format::*;
        match (self.packed >> 41) & 7 {
            0 => kBW_Format,
            1 => kA8_Format,
            2 => k3D_Format,
            3 => kARGB32_Format,
            4 => kLCD16_Format,
            5 => kSDF_Format,
            _ => unreachable!("invalid private glyph format"),
        }
    }
    // Source: src/core/SkGlyph.h:343-345
    pub fn actionFor(&self, action_type: ActionType) -> GlyphAction {
        match (self.packed >> (44 + action_type as u32)) & 3 {
            0 => GlyphAction::kUnset,
            1 => GlyphAction::kAccept,
            2 => GlyphAction::kReject,
            _ => GlyphAction::kDrop,
        }
    }
    // Source: src/core/SkGlyph.h:349-351
    pub fn maxDimension(&self) -> u16 {
        self.fWidth.max(self.fHeight)
    }
    // Source: src/core/SkGlyph.h:353-355
    pub fn fitsInAtlasDirect(&self) -> bool {
        self.maxDimension() <= Self::kSkSideTooBigForAtlas
    }
    // Source: src/core/SkGlyph.h:357-360
    pub fn fitsInAtlasInterpolated(&self) -> bool {
        self.maxDimension() <= Self::kSkSideTooBigForAtlas - 2
    }
    // Source: src/core/SkGlyph.h:362-364
    pub fn bounds(&self) -> SkGlyphRect {
        SkGlyphRect::new(
            self.fLeft as f32,
            self.fTop as f32,
            self.fLeft as f32 + self.fWidth as f32,
            self.fTop as f32 + self.fHeight as f32,
        )
    }
    // Source: src/core/SkGlyph.h:369-371
    pub fn GetKey(digest: Self) -> SkPackedGlyphID {
        SkPackedGlyphID::from_value((digest.packed & 0xfffff) as u32)
    }
    // Source: src/core/SkGlyph.h:372-374
    pub fn Hash(packed_id: SkPackedGlyphID) -> u32 {
        packed_id.hash()
    }
    // Source: src/core/SkGlyph.h:375-379
    pub fn ShouldGrow(count: i32, capacity: i32) -> bool {
        2 * count >= capacity
    }
    // Source: src/core/SkGlyph.h:380-383
    pub fn ShouldShrink(count: i32, capacity: i32) -> bool {
        6 * count <= capacity
    }
    // Source: src/core/SkGlyph.h:386-393
    fn setAction(&mut self, action_type: ActionType, action: GlyphAction) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.h\0".as_ptr(),388; @condition b"action != GlyphAction::kUnset\0".as_ptr(); (action)!=(GlyphAction::kUnset));
        crate::SkASSERT!(@source b"src/core/SkGlyph.h\0".as_ptr(),389; @condition b"this->actionFor(actionType) == GlyphAction::kUnset\0".as_ptr(); (self.actionFor(action_type))==(GlyphAction::kUnset));
        let shift = 44 + action_type as u32;
        self.packed &= !(3u64 << shift);
        self.packed |= ((action as u64) & 3) << shift;
    }
}
// Source: src/core/SkGlyph.h:395-397
const _: () = {
    assert!(SkPackedGlyphID::END_DATA == 20);
    assert!(crate::src::core::SkMask::kCountMaskFormats <= 8);
    assert!(GlyphAction::kSize as u8 <= 4);
};
// Source: src/core/SkGlyph.h:399-407
// Adapted: explicit u64 shifts implement original 20/20/1/3/12-bit native fields.
// Private storage has no uninitialized Default; native hash-table slots must initialize
// a digest before reading it. This does not claim a platform bitfield ABI.
#[derive(Clone, Copy)]
pub struct SkGlyphDigest {
    packed: u64,
    fLeft: i16,
    fTop: i16,
    fWidth: u16,
    fHeight: u16,
}

// Source: src/core/SkGlyph.h:432-432
// Source: src/core/SkGlyph.h:426
impl Default for SkGlyph {
    fn default() -> Self {
        Self::new(SkPackedGlyphID::default())
    }
}
impl SkGlyph {
    pub const fn new(id: SkPackedGlyphID) -> Self {
        Self {
            fWidth: 0,
            fHeight: 0,
            fTop: 0,
            fLeft: 0,
            fImage: std::ptr::null_mut(),
            fPathData: std::ptr::null_mut(),
            fDrawableData: std::ptr::null_mut(),
            fAdvanceX: 0.,
            fAdvanceY: 0.,
            fMaskFormat: crate::src::core::SkMask::Format::kBW_Format,
            fScalerContextBits: 0,
            #[cfg(sk_debug)]
            fAdvancesBoundsFormatAndInitialPathDone: false,
            fID: id,
        }
    }
    // Source: src/core/SkGlyph.h:434-434
    pub fn advanceVector(&self) -> SkPoint {
        SkPoint::make(self.fAdvanceX, self.fAdvanceY)
    }
    // Source: src/core/SkGlyph.h:435-435
    pub fn advanceX(&self) -> f32 {
        self.fAdvanceX
    }
    // Source: src/core/SkGlyph.h:436-436
    pub fn advanceY(&self) -> f32 {
        self.fAdvanceY
    }
    // Source: src/core/SkGlyph.h:438-438
    pub fn getGlyphID(&self) -> u16 {
        self.fID.glyph_id()
    }
    // Source: src/core/SkGlyph.h:439-439
    pub fn getPackedID(&self) -> SkPackedGlyphID {
        self.fID
    }
    // Source: src/core/SkGlyph.h:440-440
    pub fn getSubXFixed(&self) -> i32 {
        self.fID.sub_x_fixed()
    }
    // Source: src/core/SkGlyph.h:441-441
    pub fn getSubYFixed(&self) -> i32 {
        self.fID.sub_y_fixed()
    }
    // Source: src/core/SkGlyph.h:468-471
    pub fn setImageHasBeenCalled(&self) -> bool {
        self.isEmpty() || !self.fImage.is_null() || self.imageTooLarge()
    }
    // Source: src/core/SkGlyph.h:474-474
    pub fn image(&self) -> *const u8 {
        crate::SkASSERT!(@source b"src/core/SkGlyph.h\0".as_ptr(),474; @condition b"this->setImageHasBeenCalled()\0".as_ptr(); self.setImageHasBeenCalled());
        self.fImage
    }
    // Source: src/core/SkGlyph.h:495-495
    pub fn setPathHasBeenCalled(&self) -> bool {
        !self.fPathData.is_null()
    }
    // Source: src/core/SkGlyph.h:505-505
    pub fn setDrawableHasBeenCalled(&self) -> bool {
        !self.fDrawableData.is_null()
    }
    // Source: src/core/SkGlyph.h:509-509
    pub fn isColor(&self) -> bool {
        self.fMaskFormat == crate::src::core::SkMask::Format::kARGB32_Format
    }
    // Source: src/core/SkGlyph.h:510-510
    pub fn maskFormat(&self) -> crate::src::core::SkMask::Format {
        self.fMaskFormat
    }
    // Source: src/core/SkGlyph.h:514-514
    pub fn maxDimension(&self) -> i32 {
        self.fWidth.max(self.fHeight) as i32
    }
    // Source: src/core/SkGlyph.h:515-515
    pub fn iRect(&self) -> crate::src::core::SkRect::SkIRect {
        crate::src::core::SkRect::SkIRect::make_xywh(
            self.fLeft as i32,
            self.fTop as i32,
            self.fWidth as i32,
            self.fHeight as i32,
        )
    }
    // Source: src/core/SkGlyph.h:516-516
    pub fn rect(&self) -> crate::src::core::SkRect::SkRect {
        crate::src::core::SkRect::SkRect::MakeXYWH(
            self.fLeft as f32,
            self.fTop as f32,
            self.fWidth as f32,
            self.fHeight as f32,
        )
    }
    // Source: src/core/SkGlyph.h:517-519
    pub fn glyphRect(&self) -> SkGlyphRect {
        SkGlyphRect::new(
            self.fLeft as f32,
            self.fTop as f32,
            (self.fLeft as i32 + self.fWidth as i32) as f32,
            (self.fTop as i32 + self.fHeight as i32) as f32,
        )
    }
    // Source: src/core/SkGlyph.h:520-520
    pub fn left(&self) -> i32 {
        self.fLeft as i32
    }
    // Source: src/core/SkGlyph.h:521-521
    pub fn top(&self) -> i32 {
        self.fTop as i32
    }
    // Source: src/core/SkGlyph.h:522-522
    pub fn width(&self) -> i32 {
        self.fWidth as i32
    }
    // Source: src/core/SkGlyph.h:523-523
    pub fn height(&self) -> i32 {
        self.fHeight as i32
    }
    // Source: src/core/SkGlyph.h:524-526
    pub fn isEmpty(&self) -> bool {
        self.fWidth == 0 || self.fHeight == 0
    }
    // Source: src/core/SkGlyph.h:527-527
    pub fn imageTooLarge(&self) -> bool {
        self.fWidth >= Self::MAX_GLYPH_WIDTH
    }
    // Source: src/core/SkGlyph.h:529-529
    pub fn extraBits(&self) -> u16 {
        self.fScalerContextBits
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.h:541
    /// The caller retains the backing allocation for every use of this non-owning glyph.
    pub unsafe fn setImageRaw(&mut self, image: *mut u8) {
        self.fImage = image;
    }
}
// Source: src/core/SkGlyph.h:573-573
impl SkGlyph {
    const MAX_GLYPH_WIDTH: u16 = 1 << 13;
}
// Source: src/core/SkGlyph.h:580-584
#[repr(C)]
pub(crate) struct Intercept {
    pub(crate) fNext: *mut Intercept,
    pub(crate) fBounds: [f32; 2],
    pub(crate) fInterval: [f32; 2],
}
// Source: src/core/SkGlyph.h:586-589
// Source: src/core/SkGlyph.h:594-594
// Source: src/core/SkGlyph.h:600-601
/// Arena owns this path sidecar and invokes its SkPath destructor. Glyphs only borrow
/// its stable address; neither a copied glyph nor this intercept link owns arena storage.
#[repr(C)]
pub(crate) struct PathData {
    pub(crate) fIntercept: *mut Intercept,
    pub(crate) fPath: crate::src::core::SkPath::SkPath,
    pub(crate) fHasPath: bool,
    pub(crate) fHairline: bool,
    pub(crate) fModified: bool,
}
impl Default for PathData {
    fn default() -> Self {
        Self {
            fIntercept: std::ptr::null_mut(),
            fPath: crate::src::core::SkPath::SkPath::default(),
            fHasPath: false,
            fHairline: false,
            fModified: false,
        }
    }
}
// Source: src/core/SkGlyph.h:603-607
/// Sole retained drawable handle in the native arena-owned glyph sidecar.
/// The glyph borrows this sidecar; the arena runs its sk_sp destructor once.
pub(crate) struct DrawableData {
    pub(crate) fIntercept: *mut Intercept,
    pub(crate) fDrawable:
        crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDrawable::SkDrawable>,
    pub(crate) fHasDrawable: bool,
}
impl Default for DrawableData {
    fn default() -> Self {
        Self {
            fIntercept: std::ptr::null_mut(),
            fDrawable: crate::include::core::SkRefCnt::sk_sp::default(),
            fHasDrawable: false,
        }
    }
}
// Source: src/core/SkGlyph.h:623-624
// Source: src/core/SkGlyph.h:627-628
// Source: src/core/SkGlyph.h:631-631
// Source: src/core/SkGlyph.h:636-637
// Source: src/core/SkGlyph.h:640-641
// Source: src/core/SkGlyph.h:643-643
// Source: src/core/SkGlyph.h:647-647
// Source: src/core/SkGlyph.h:651-651
// Source: src/core/SkGlyph.h:653-653
// Source: src/core/SkGlyph.h:427-431
// Existing Clone/Copy, Rust assignment/move and trivial drop express native
// default memberwise operations; arena pointers remain borrowed. UNVERIFIED.
// Rust flat storage and explicit native in-class initializers; arena pointers are nonowning.
#[derive(Clone, Copy)]
pub struct SkGlyph {
    pub(crate) fWidth: u16,
    pub(crate) fHeight: u16,
    pub(crate) fTop: i16,
    pub(crate) fLeft: i16,
    pub(crate) fImage: *mut u8,
    pub(crate) fPathData: *mut PathData,
    pub(crate) fDrawableData: *mut DrawableData,
    pub(crate) fAdvanceX: f32,
    pub(crate) fAdvanceY: f32,
    pub(crate) fMaskFormat: crate::src::core::SkMask::Format,
    pub(crate) fScalerContextBits: u16,
    #[cfg(sk_debug)]
    pub(crate) fAdvancesBoundsFormatAndInitialPathDone: bool,
    pub(crate) fID: SkPackedGlyphID,
}
impl SkGlyph {
    // Source: src/core/SkGlyph.h:611-614
    /// The caller retains the backing allocation for every use of this non-owning glyph.
    pub unsafe fn installImage(&mut self, image: *mut u8) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.h\0".as_ptr(),612; @condition b"!this->setImageHasBeenCalled()\0".as_ptr(); !self.setImageHasBeenCalled());
        self.fImage = image;
    }
}
// Source: src/core/SkGlyph.cpp:103-125
impl SkGlyph {
    pub fn MakeFromBuffer(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Option<Self> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),104; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let packed_id = SkPackedGlyphID::from_value(buffer.readUInt());
        let advance = buffer.readPoint();
        let dimensions = buffer.readUInt();
        let left_top = buffer.readUInt();
        let format_code = buffer.readUInt();
        debug_assert!(format_code <= u8::MAX as u32);
        if !buffer.validate(crate::src::core::SkMask::SkMask::IsValidFormat(
            format_code as u8,
        )) {
            return None;
        }
        Self::from_metrics_fields(packed_id, advance, dimensions, left_top, format_code)
    }
}
// Source: src/core/SkGlyph.cpp:109-125
// Decoded-field construction shared by the actual reader above.
impl SkGlyph {
    pub fn from_metrics_fields(
        packed_id: SkPackedGlyphID,
        advance: SkPoint,
        dimensions: u32,
        left_top: u32,
        format_code: u32,
    ) -> Option<Self> {
        debug_assert!(format_code <= u8::MAX as u32);
        use crate::src::core::SkMask::Format::*;
        let format = match format_code as u8 {
            0 => kBW_Format,
            1 => kA8_Format,
            2 => k3D_Format,
            3 => kARGB32_Format,
            4 => kLCD16_Format,
            5 => kSDF_Format,
            _ => return None,
        };
        let mut glyph = Self::new(packed_id);
        glyph.fAdvanceX = advance.fX;
        glyph.fAdvanceY = advance.fY;
        glyph.fWidth = (dimensions >> 16) as u16;
        glyph.fHeight = dimensions as u16;
        glyph.fLeft = (left_top >> 16) as i16;
        glyph.fTop = left_top as i16;
        glyph.fMaskFormat = format;
        #[cfg(sk_debug)]
        {
            glyph.fAdvancesBoundsFormatAndInitialPathDone = true;
        }
        Some(glyph)
    }
}

// Source: src/core/SkGlyph.cpp:127-130
impl SkGlyph {
    pub fn mask(&self) -> crate::src::core::SkMask::SkMask {
        let bounds = crate::src::core::SkRect::SkIRect::make_xywh(
            self.fLeft as i32,
            self.fTop as i32,
            self.fWidth as i32,
            self.fHeight as i32,
        );
        crate::src::core::SkMask::SkMask::new(
            self.fImage,
            bounds,
            self.rowBytes() as u32,
            self.fMaskFormat,
        )
    }
}
// Source: src/core/SkGlyph.cpp:132-137
impl SkGlyph {
    pub fn maskAt(&self, position: SkPoint) -> crate::src::core::SkMask::SkMask {
        debug_assert!(
            crate::src::core::SkScalar::SkScalarIsInt(position.fX)
                && crate::src::core::SkScalar::SkScalarIsInt(position.fY)
        );
        let mut bounds = crate::src::core::SkRect::SkIRect::make_xywh(
            self.fLeft as i32,
            self.fTop as i32,
            self.fWidth as i32,
            self.fHeight as i32,
        );
        bounds.offset(
            crate::src::core::SkFloatingPoint::sk_float_floor2int(position.fX),
            crate::src::core::SkFloatingPoint::sk_float_floor2int(position.fY),
        );
        crate::src::core::SkMask::SkMask::new(
            self.fImage,
            bounds,
            self.rowBytes() as u32,
            self.fMaskFormat,
        )
    }
}
// Source: src/core/SkGlyph.cpp:139-146
impl SkGlyph {
    pub fn zeroMetrics(&mut self) {
        self.fAdvanceX = 0.;
        self.fAdvanceY = 0.;
        self.fWidth = 0;
        self.fHeight = 0;
        self.fTop = 0;
        self.fLeft = 0;
    }
}
// Source: src/core/SkGlyph.cpp:148-150
pub fn bits_to_bytes(bits: usize) -> usize {
    bits.wrapping_add(7) >> 3
}
// Source: src/core/SkGlyph.cpp:152-165
pub fn format_alignment(format: crate::src::core::SkMask::Format) -> usize {
    use crate::src::core::SkMask::Format::*;
    match format {
        kBW_Format | kA8_Format | k3D_Format | kSDF_Format => std::mem::align_of::<u8>(),
        kARGB32_Format => std::mem::align_of::<u32>(),
        kLCD16_Format => std::mem::align_of::<u16>(),
    }
}
// Source: src/core/SkGlyph.cpp:167-170
pub fn format_rowbytes(width: i32, format: crate::src::core::SkMask::Format) -> usize {
    if format == crate::src::core::SkMask::Format::kBW_Format {
        bits_to_bytes(width as usize)
    } else {
        (width as usize).wrapping_mul(format_alignment(format))
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:172-174
    pub fn formatAlignment(&self) -> usize {
        format_alignment(self.maskFormat())
    }
    // Source: src/core/SkGlyph.cpp:176-182
    fn allocImage(&mut self, alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc) -> usize {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),177; @condition b"!this->isEmpty()\0".as_ptr(); !self.isEmpty());
        let size = self.imageSize();
        self.fImage = alloc.makeBytesAlignedTo(size, self.formatAlignment());
        size
    }
    // Source: src/core/SkGlyph.cpp:184-195
    /// Safety: actual scaler, arena and destination pixels retain exclusive native
    /// operation leases. A failed CPU stage rolls back the unpublished image pointer.
    pub unsafe fn setImageFromScaler(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        context: &mut dyn crate::src::core::SkScalerContext::SkScalerContextVirtual,
    ) -> Result<bool, crate::RasterError> {
        self.setImageFromScalerWithAllocation(alloc, context, |_| {})
    }

    // Same native cpp184-195 algorithm, with a Rust-only allocation checkpoint.
    // The arena keeps every allocated byte even if CPU execution fails or panics;
    // the cache caller accounts that actual allocation before invoking the backend.
    pub(crate) unsafe fn setImageFromScalerWithAllocation(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        context: &mut dyn crate::src::core::SkScalerContext::SkScalerContextVirtual,
        allocated: impl FnOnce(usize),
    ) -> Result<bool, crate::RasterError> {
        if !self.setImageHasBeenCalled() {
            #[cfg(sk_debug)]
            let old_format = self.maskFormat();
            let bytes = self.allocImage(alloc);
            // UNVERIFIED ownership correction: never publish incomplete pixels
            // as a cached image, on either RasterError or Rust backend unwind.
            struct UnpublishedImage(*mut *mut u8);
            impl Drop for UnpublishedImage {
                fn drop(&mut self) {
                    if !self.0.is_null() {
                        unsafe {
                            *self.0 = core::ptr::null_mut();
                        }
                    }
                }
            }
            let mut unpublished = UnpublishedImage(core::ptr::addr_of_mut!(self.fImage));
            allocated(bytes);
            context.getImage(self)?;
            #[cfg(sk_debug)]
            crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),191; @condition b"oldFormat == this->maskFormat()\0".as_ptr(); (old_format)==(self.maskFormat()));
            unpublished.0 = core::ptr::null_mut();
            return Ok(true);
        }
        Ok(false)
    }

    /// Safety: image points to at least imageSize readable bytes, disjoint from the new arena allocation.
    // Source: src/core/SkGlyph.cpp:197-204
    pub unsafe fn setImage(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        image: *const u8,
    ) -> bool {
        if !self.setImageHasBeenCalled() {
            self.allocImage(alloc);
            unsafe {
                std::ptr::copy_nonoverlapping(image, self.fImage, self.imageSize());
            }
            return true;
        }
        false
    }
    /// Safety: from's optional image remains readable and disjoint from new arena storage for the call.
    // Source: src/core/SkGlyph.cpp:206-230
    pub unsafe fn setMetricsAndImage(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        from: &Self,
    ) -> usize {
        debug_assert!(self.fImage.is_null() || from.fImage.is_null());
        if self.fImage.is_null() {
            self.fAdvanceX = from.fAdvanceX;
            self.fAdvanceY = from.fAdvanceY;
            self.fWidth = from.fWidth;
            self.fHeight = from.fHeight;
            self.fTop = from.fTop;
            self.fLeft = from.fLeft;
            self.fScalerContextBits = from.fScalerContextBits;
            self.fMaskFormat = from.fMaskFormat;
            if !from.fImage.is_null() && unsafe { self.setImage(alloc, from.image()) } {
                return self.imageSize();
            }
            #[cfg(sk_debug)]
            {
                self.fAdvancesBoundsFormatAndInitialPathDone =
                    from.fAdvancesBoundsFormatAndInitialPathDone;
            }
        }
        0
    }
    // Source: src/core/SkGlyph.cpp:232-234
    pub fn rowBytes(&self) -> usize {
        format_rowbytes(self.fWidth as i32, self.fMaskFormat)
    }
    // Source: src/core/SkGlyph.cpp:236-238
    pub fn rowBytesUsingFormat(&self, format: crate::src::core::SkMask::Format) -> usize {
        format_rowbytes(self.fWidth as i32, format)
    }
    // Source: src/core/SkGlyph.cpp:240-250
    pub fn imageSize(&self) -> usize {
        if self.isEmpty() || self.imageTooLarge() {
            return 0;
        }
        let mut size = self.rowBytes() * self.fHeight as usize;
        if self.fMaskFormat == crate::src::core::SkMask::Format::k3D_Format {
            size *= 3;
        }
        size
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:252-262
    /// Safety: this glyph's arena sidecars remain live and exclusively accessible for the call.
    pub unsafe fn installPath(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        path: Option<&crate::src::core::SkPath::SkPath>,
        hairline: bool,
        modified: bool,
    ) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),253; @condition b"fPathData == nullptr\0".as_ptr(); self.fPathData.is_null());
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),254; @condition b"!this->setPathHasBeenCalled()\0".as_ptr(); !self.setPathHasBeenCalled());
        self.fPathData = alloc.make(PathData::default());
        if let Some(path) = path {
            let data = &mut *self.fPathData;
            data.fPath = path.clone();
            data.fHasPath = true;
            data.fHairline = hairline;
            data.fModified = modified;
        }
    }
    // Source: src/core/SkGlyph.cpp:264-272
    /// Safety: the actual scaler and arena retain their native synchronous backing leases.
    pub unsafe fn setPathFromScaler(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        context: &mut dyn crate::src::core::SkScalerContext::SkScalerContextVirtual,
    ) -> bool {
        if !self.setPathHasBeenCalled() {
            unsafe {
                context.getPath(self, alloc);
            }
            crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),267; @condition b"this->setPathHasBeenCalled()\0".as_ptr(); self.setPathHasBeenCalled());
            return unsafe { self.path().is_some() };
        }
        false
    }

    // Source: src/core/SkGlyph.cpp:274-280
    /// Safety: the arena owning this glyph's sidecars is live and no other glyph copy mutates them.
    pub unsafe fn setPath(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        path: Option<&crate::src::core::SkPath::SkPath>,
        hairline: bool,
        modified: bool,
    ) -> bool {
        if !self.setPathHasBeenCalled() {
            self.installPath(alloc, path, hairline, modified);
            return self.path().is_some();
        }
        false
    }
    // Source: src/core/SkGlyph.cpp:282-289
    /// Safety: the sidecar arena outlives the returned borrow; no copied glyph mutates its path.
    pub unsafe fn path(&self) -> Option<&crate::src::core::SkPath::SkPath> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),284; @condition b"this->setPathHasBeenCalled()\0".as_ptr(); self.setPathHasBeenCalled());
        let data = &*self.fPathData;
        if data.fHasPath {
            Some(&data.fPath)
        } else {
            None
        }
    }
    // Source: src/core/SkGlyph.cpp:291-295
    /// Safety: the sidecar arena is live and not concurrently mutated.
    pub unsafe fn pathIsHairline(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),293; @condition b"this->setPathHasBeenCalled()\0".as_ptr(); self.setPathHasBeenCalled());
        (*self.fPathData).fHairline
    }
    // Source: src/core/SkGlyph.cpp:297-301
    /// Safety: the sidecar arena is live and not concurrently mutated.
    pub unsafe fn pathIsModified(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),299; @condition b"this->setPathHasBeenCalled()\0".as_ptr(); self.setPathHasBeenCalled());
        (*self.fPathData).fModified
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:303-312
    /// Safety: this glyph's sidecar arena remains live and exclusively accessible for the call.
    pub unsafe fn installDrawable(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        drawable: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDrawable::SkDrawable,
        >,
    ) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),304; @condition b"fDrawableData == nullptr\0".as_ptr(); self.fDrawableData.is_null());
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),305; @condition b"!this->setDrawableHasBeenCalled()\0".as_ptr(); !self.setDrawableHasBeenCalled());
        self.fDrawableData = alloc.make(DrawableData::default());
        if drawable.get().is_some() {
            let data = &mut *self.fDrawableData;
            data.fDrawable = drawable;
            data.fDrawable.getGenerationID();
            data.fHasDrawable = true;
        }
    }
    // Source: src/core/SkGlyph.cpp:314-321
    /// Safety: the actual scaler, drawable backing and arena retain their native leases.
    pub unsafe fn setDrawableFromScaler(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        context: &mut dyn crate::src::core::SkScalerContext::SkScalerContextVirtual,
    ) -> bool {
        if !self.setDrawableHasBeenCalled() {
            let drawable = unsafe { context.getDrawable(self) };
            unsafe {
                self.installDrawable(alloc, drawable);
            }
            return unsafe { self.drawable().is_some() };
        }
        false
    }

    // Source: src/core/SkGlyph.cpp:323-329
    /// Safety: sidecar arena is live, with no mutation through another copy of this glyph.
    pub unsafe fn setDrawable(
        &mut self,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        drawable: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDrawable::SkDrawable,
        >,
    ) -> bool {
        if !self.setDrawableHasBeenCalled() {
            self.installDrawable(alloc, drawable);
            return self.drawable().is_some();
        }
        false
    }
    // Source: src/core/SkGlyph.cpp:331-338
    /// Safety: sidecar arena outlives the returned borrow, with no aliased sidecar mutation.
    pub unsafe fn drawable(&self) -> Option<&dyn crate::src::core::SkDrawable::SkDrawable> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),333; @condition b"this->setDrawableHasBeenCalled()\0".as_ptr(); self.setDrawableHasBeenCalled());
        let data = &*self.fDrawableData;
        if data.fHasDrawable {
            data.fDrawable.get()
        } else {
            None
        }
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:340-351
    pub fn flattenMetrics(&self, buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer) {
        buffer.writeUInt(self.fID.value());
        buffer.writePoint(&SkPoint::make(self.fAdvanceX, self.fAdvanceY));
        buffer.writeUInt((self.fWidth as u32) << 16 | self.fHeight as u32);
        let left = self.fLeft as u16 as u32;
        let top = self.fTop as u16 as u32;
        buffer.writeUInt(left << 16 | top);
        buffer.writeUInt(self.fMaskFormat as u32);
    }
    // Source: src/core/SkGlyph.cpp:353-360
    /// Safety: prepared image bytes retain the glyph/arena lease through writing.
    pub unsafe fn flattenImage(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),354; @condition b"this->setImageHasBeenCalled()\0".as_ptr(); self.setImageHasBeenCalled());
        if !self.isEmpty() && SkGlyphDigest::FitsInAtlas(self) {
            unsafe { buffer.writeByteArray(self.image(), self.imageSize()) };
        }
    }
    // Source: src/core/SkGlyph.cpp:362-380
    /// Safety: glyph and arena are exclusively leased. Output is published only
    /// after the original reader-validity gate, with no initialized output slice.
    pub unsafe fn addImageFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> usize {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),363; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        if self.isEmpty() || !SkGlyphDigest::FitsInAtlas(self) {
            return 0;
        }
        let mut memory_increase = 0;
        let image_data = alloc.makeBytesAlignedTo(self.imageSize(), self.formatAlignment());
        unsafe { buffer.readByteArray_raw(image_data, self.imageSize()) };
        if buffer.isValid() {
            unsafe { self.installImage(image_data) };
            memory_increase += self.imageSize();
        }
        memory_increase
    }
    // Source: src/core/SkGlyph.cpp:382-392
    /// Safety: path sidecar remains alive and is not concurrently mutated.
    pub unsafe fn flattenPath(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),383; @condition b"this->setPathHasBeenCalled()\0".as_ptr(); self.setPathHasBeenCalled());
        let path = unsafe { self.path() };
        buffer.writeBool(path.is_some());
        if let Some(path) = path {
            buffer.writeBool(unsafe { self.pathIsHairline() });
            buffer.writeBool(unsafe { self.pathIsModified() });
            buffer.writePath(path);
        }
    }
    // Source: src/core/SkGlyph.cpp:394-422
    /// Safety: this glyph's sidecars and arena have their native exclusive lease.
    pub unsafe fn addPathFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> usize {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),395; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let mut memory_increase = 0;
        let has_path = buffer.readBool();
        if !buffer.isValid() {
            return 0;
        }
        if has_path {
            let hairline = buffer.readBool();
            let modified = buffer.readBool();
            if let Some(path) = buffer.readPath() {
                use crate::src::core::SkMask::Format::*;
                if self.fMaskFormat != kBW_Format
                    && self.fMaskFormat != kA8_Format
                    && self.fMaskFormat != kLCD16_Format
                {
                    buffer.validate(false);
                    return 0;
                }
                if unsafe { self.setPath(alloc, Some(&path), hairline, modified) } {
                    memory_increase += path.approximateBytesUsed();
                }
            }
        } else {
            unsafe { self.setPath(alloc, None, false, false) };
        }
        memory_increase
    }
}
// Source: src/core/SkGlyph.cpp:450-451
// Source: src/core/SkGlyph.cpp:454-455
// Source: src/core/SkGlyph.cpp:457-460
// Source: src/core/SkGlyph.cpp:463-468
// Source: src/core/SkGlyph.cpp:470-473
// Source: src/core/SkGlyph.cpp:475-478
// Source: src/core/SkGlyph.cpp:480-483
// Source: src/core/SkGlyph.cpp:485-488
// Source: src/core/SkGlyph.cpp:491-497
// Source: src/core/SkGlyph.cpp:499-507
// Source: src/core/SkGlyph.cpp:510-518
// Source: src/core/SkGlyph.cpp:521-534
// Source: src/core/SkGlyph.cpp:537-548
// Source: src/core/SkGlyph.cpp:550-551
pub(crate) fn calculate_path_gap(
    top_offset: f32,
    bottom_offset: f32,
    path: &crate::src::core::SkPath::SkPath,
) -> (f32, f32) {
    use crate::include::core::SkPathTypes::SkPathVerb;
    use crate::include::core::SkScalar::{SK_ScalarMax, SK_ScalarMin};
    use crate::src::core::SkBezierCurves::{SkBezierCubic, SkBezierQuad};
    let mut left = SK_ScalarMax;
    let mut right = SK_ScalarMin;
    let mut expand_gap = |v: f32| {
        if v < left {
            left = v
        }
        if right < v {
            right = v
        }
    };
    let add_line = |pts: &[SkPoint], offset: f32, expand: &mut dyn FnMut(f32)| {
        let t = (offset - pts[0].fY) / (pts[1].fY - pts[0].fY);
        if 0. <= t && t < 1. {
            expand(pts[0].fX + t * (pts[1].fX - pts[0].fX))
        }
    };
    let add_quad = |pts: &[SkPoint], offset: f32, expand: &mut dyn FnMut(f32)| {
        let mut storage = [0.; 2];
        let intersections = SkBezierQuad::IntersectWithHorizontalLine(pts, offset, &mut storage);
        for &intersection in intersections {
            expand(intersection)
        }
    };
    let add_cubic = |pts: &[SkPoint], offset: f32, expand: &mut dyn FnMut(f32)| {
        let mut storage = [0.; 3];
        let intersections = SkBezierCubic::IntersectWithHorizontalLine(pts, offset, &mut storage);
        for &intersection in intersections {
            expand(intersection)
        }
    };
    let add_pts = |pts: &[SkPoint], expand: &mut dyn FnMut(f32)| {
        for p in pts {
            if top_offset < p.fY && p.fY < bottom_offset {
                expand(p.fX)
            }
        }
    };
    // std::minmax's first minimum/last maximum tie choices are retained.
    let minmax = |pts: &[SkPoint]| {
        let (mut lo, mut hi) = if pts[1].fY < pts[0].fY {
            (pts[1].fY, pts[0].fY)
        } else {
            (pts[0].fY, pts[1].fY)
        };
        if pts.len() == 3 {
            let y = pts[2].fY;
            if y < lo {
                lo = y
            } else if !(y < hi) {
                hi = y
            }
        } else if pts.len() == 4 {
            let (pair_lo, pair_hi) = if pts[3].fY < pts[2].fY {
                (pts[3].fY, pts[2].fY)
            } else {
                (pts[2].fY, pts[3].fY)
            };
            if pair_lo < lo {
                lo = pair_lo
            }
            if !(pair_hi < hi) {
                hi = pair_hi
            }
        }
        (lo, hi)
    };
    let mut iter = crate::src::core::SkPath::Iter::new(path, false);
    while let Some(rec) = iter.next() {
        let pts = rec.fPoints;
        match rec.fVerb {
            SkPathVerb::kMove => {}
            SkPathVerb::kLine => {
                let (line_top, line_bottom) = minmax(pts);
                if top_offset <= line_bottom && line_top <= bottom_offset {
                    add_line(pts, top_offset, &mut expand_gap);
                    add_line(pts, bottom_offset, &mut expand_gap);
                    add_pts(pts, &mut expand_gap)
                }
            }
            SkPathVerb::kQuad => {
                let (quad_top, quad_bottom) = minmax(pts);
                if top_offset <= quad_bottom && quad_top <= bottom_offset {
                    add_quad(pts, top_offset, &mut expand_gap);
                    add_quad(pts, bottom_offset, &mut expand_gap);
                    add_pts(pts, &mut expand_gap)
                }
            }
            SkPathVerb::kConic => {
                debug_assert!(
                    false,
                    "There should be no conic primitives in glyph outlines."
                )
            }
            SkPathVerb::kCubic => {
                let (cubic_top, cubic_bottom) = minmax(pts);
                if top_offset <= cubic_bottom && cubic_top <= bottom_offset {
                    add_cubic(pts, top_offset, &mut expand_gap);
                    add_cubic(pts, bottom_offset, &mut expand_gap);
                    add_pts(pts, &mut expand_gap)
                }
            }
            SkPathVerb::kClose => {}
        }
    }
    (left, right)
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:553-611
    /// Safety: a PathData sidecar has been installed in the live arena. All linked
    /// intercepts remain live and exclusively accessible, including through glyph copies.
    /// When an array is provided, count is nonnegative and leaves two output elements.
    pub unsafe fn ensureIntercepts(
        &mut self,
        bounds: &[f32; 2],
        scale: f32,
        xPos: f32,
        mut array: Option<&mut [f32]>,
        count: &mut i32,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) {
        use crate::src::core::SkScalar::{SK_ScalarMax, SK_ScalarMin};
        let mut offset_results = |intercept: &Intercept| {
            if let Some(array) = array.as_deref_mut() {
                debug_assert!(*count >= 0);
                let offset = *count as usize;
                for index in 0..2 {
                    array[offset + index] = intercept.fInterval[index] * scale + xPos;
                }
            }
            *count += 2;
        };
        let mut matched = if self.fPathData.is_null() {
            core::ptr::null_mut()
        } else {
            (*self.fPathData).fIntercept
        };
        while !matched.is_null() {
            if bounds[0] == (*matched).fBounds[0] && bounds[1] == (*matched).fBounds[1] {
                break;
            }
            matched = (*matched).fNext;
        }
        if !matched.is_null() {
            if (*matched).fInterval[0] < (*matched).fInterval[1] {
                offset_results(&*matched);
            }
            return;
        }
        let intercept = alloc.make(Intercept {
            fNext: (*self.fPathData).fIntercept,
            fBounds: *bounds,
            fInterval: [SK_ScalarMax, SK_ScalarMin],
        });
        (*self.fPathData).fIntercept = intercept;
        let path = &(*self.fPathData).fPath;
        let path_bounds = path.getBounds();
        if path_bounds.fBottom < bounds[0] || bounds[1] < path_bounds.fTop {
            return;
        }
        let (left, right) = calculate_path_gap(bounds[0], bounds[1], path);
        (*intercept).fInterval = [left, right];
        if (*intercept).fInterval[0] >= (*intercept).fInterval[1] {
            (*intercept).fInterval = [SK_ScalarMax, SK_ScalarMin];
            return;
        }
        offset_results(&*intercept);
    }
}
// Source: src/core/SkGlyph.cpp:613-627
fn init_actions(glyph: &SkGlyph) -> u32 {
    let drop_action = GlyphAction::kDrop as u32;
    let all_drop = drop_action << ActionType::kDirectMask as u32
        | drop_action << ActionType::kDirectMaskCPU as u32
        | drop_action << ActionType::kMask as u32
        | drop_action << ActionType::kSDFT as u32
        | drop_action << ActionType::kPath as u32
        | drop_action << ActionType::kDrawable as u32;
    if glyph.isEmpty() {
        all_drop
    } else {
        0
    }
}
// Source: src/core/SkGlyph.cpp:629-638
impl SkGlyphDigest {
    pub fn new(index: usize, glyph: &SkGlyph) -> Self {
        Self {
            packed: (glyph.getPackedID().value() as u64 & 0xfffff)
                | ((index as u64 & 0xfffff) << 20)
                | ((glyph.isEmpty() as u64) << 40)
                | ((glyph.maskFormat() as u64) << 41)
                | ((init_actions(glyph) as u64) << 44),
            fLeft: glyph.left() as i16,
            fTop: glyph.top() as i16,
            fWidth: glyph.width() as u16,
            fHeight: glyph.height() as u16,
        }
    }
}
// The full StrikeForGPU dispatch remains pending. These three exact native
// cases do not require an image/path/drawable producer and do not provide fallback actions.
impl SkGlyphDigest {
    // Source: src/core/SkGlyph.cpp:644-645
    // Source: src/core/SkGlyph.cpp:647-652
    // Source: src/core/SkGlyph.cpp:686-688
    pub fn setDirectMaskAction(&mut self) {
        if self.actionFor(ActionType::kDirectMask) == GlyphAction::kUnset {
            let mut action = GlyphAction::kReject;
            if self.fitsInAtlasDirect() {
                action = GlyphAction::kAccept;
            }
            self.setAction(ActionType::kDirectMask, action);
        }
    }
    // Source: src/core/SkGlyph.cpp:644-645
    // Source: src/core/SkGlyph.cpp:660-665
    // Source: src/core/SkGlyph.cpp:686-688
    pub fn setMaskAction(&mut self) {
        if self.actionFor(ActionType::kMask) == GlyphAction::kUnset {
            let mut action = GlyphAction::kReject;
            if self.fitsInAtlasInterpolated() {
                action = GlyphAction::kAccept;
            }
            self.setAction(ActionType::kMask, action);
        }
    }
    // Source: src/core/SkGlyph.cpp:644-645
    // Source: src/core/SkGlyph.cpp:666-672
    // Source: src/core/SkGlyph.cpp:686-688
    pub fn setSDFTAction(&mut self) {
        if self.actionFor(ActionType::kSDFT) == GlyphAction::kUnset {
            let mut action = GlyphAction::kReject;
            if self.fitsInAtlasDirect()
                && self.maskFormat() == crate::src::core::SkMask::Format::kSDF_Format
            {
                action = GlyphAction::kAccept;
            }
            self.setAction(ActionType::kSDFT, action);
        }
    }
}

// Source: src/core/SkGlyph.cpp:640-688
impl SkGlyphDigest {
    // Native StrikeForGPU virtual preparation is borrowed under the actual CPU
    // strike monitor. The callback prepares this exact glyph, never a copy.
    pub(crate) unsafe fn setActionForCPU(
        &mut self,
        action_type: ActionType,
        glyph: &mut SkGlyph,
        mut prepare: impl FnMut(ActionType, &mut SkGlyph) -> Result<bool, crate::RasterError>,
    ) -> Result<(), crate::RasterError> {
        if self.actionFor(action_type) == GlyphAction::kUnset {
            let mut action = GlyphAction::kReject;
            let accepted = match action_type {
                ActionType::kDirectMask => self.fitsInAtlasDirect(),
                ActionType::kDirectMaskCPU => {
                    let ready = prepare(action_type, glyph)?;
                    if ready {
                        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),655; @condition b"!glyph->isEmpty()\0".as_ptr(); !glyph.isEmpty());
                    }
                    ready
                }
                ActionType::kMask => self.fitsInAtlasInterpolated(),
                ActionType::kSDFT => {
                    self.fitsInAtlasDirect()
                        && self.maskFormat() == crate::src::core::SkMask::Format::kSDF_Format
                }
                ActionType::kPath | ActionType::kDrawable => prepare(action_type, glyph)?,
            };
            if accepted {
                action = GlyphAction::kAccept;
            }
            self.setAction(action_type, action);
        }
        Ok(())
    }
}

// Source: src/core/SkGlyph.cpp:690-692
impl SkGlyphDigest {
    pub fn FitsInAtlas(glyph: &SkGlyph) -> bool {
        glyph.maxDimension() <= Self::kSkSideTooBigForAtlas as i32
    }
}

impl SkGlyphPositionRoundingSpec {
    // Source: src/core/SkGlyph.cpp:695-712
    fn half_axis_sample_freq(subpixel: bool, alignment: SkAxisAlignment) -> SkPoint {
        if !subpixel {
            SkPoint::make(0.5, 0.5)
        } else {
            let round = SkPackedGlyphID::SUBPIXEL_ROUND;
            match alignment {
                SkAxisAlignment::X => SkPoint::make(round, 0.5),
                SkAxisAlignment::Y => SkPoint::make(0.5, round),
                SkAxisAlignment::None => SkPoint::make(round, round),
                // Source: src/core/SkGlyph.cpp:710-711
                _ => SkPoint::make(0.0, 0.0),
            }
        }
    }
}
impl SkGlyphPositionRoundingSpec {
    // Source: src/core/SkGlyph.cpp:714-718
    fn ignore_position_mask(subpixel: bool, alignment: SkAxisAlignment) -> SkIPoint {
        SkIPoint::make(
            if !subpixel || alignment == SkAxisAlignment::Y {
                0
            } else {
                !0
            },
            if !subpixel || alignment == SkAxisAlignment::X {
                0
            } else {
                !0
            },
        )
    }
}
impl SkGlyphPositionRoundingSpec {
    // Source: src/core/SkGlyph.cpp:720-726
    fn ignore_position_field_mask(subpixel: bool, alignment: SkAxisAlignment) -> SkIPoint {
        let mask = Self::ignore_position_mask(subpixel, alignment);
        SkIPoint::make(
            mask.x() & SkPackedGlyphID::XY_FIELD_MASK.x(),
            mask.y() & SkPackedGlyphID::XY_FIELD_MASK.y(),
        )
    }
}
impl SkGlyphPositionRoundingSpec {
    // Source: src/core/SkGlyph.cpp:728-732
    pub fn new(subpixel: bool, alignment: SkAxisAlignment) -> Self {
        Self {
            half_axis_sample_freq: Self::half_axis_sample_freq(subpixel, alignment),
            ignore_position_mask: Self::ignore_position_mask(subpixel, alignment),
            ignore_position_field_mask: Self::ignore_position_field_mask(subpixel, alignment),
        }
    }
}
// Source: src/core/SkGlyph.h:409-420
pub struct SkPictureBackedGlyphDrawable {
    fPicture: core::mem::ManuallyDrop<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
    >,
    inherited: crate::src::core::SkDrawable::SkDrawableState,
    fRefCnt: crate::include::core::SkRefCnt::SkRefCnt,
}
// Source: src/core/SkGlyph.h:409-420 (SkDrawable's actual SkRefCnt parent)
// The selected parent owns the same one Base count and native adoption state;
// retain must preserve that selected parent's hook, not erase it to Base.
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for SkPictureBackedGlyphDrawable {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
impl SkPictureBackedGlyphDrawable {
    // Source: src/core/SkGlyph.cpp:37-65
    pub fn MakeFromBuffer(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Result<Option<crate::include::core::SkRefCnt::sk_sp<Self>>, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),39; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let data = buffer.readByteArrayAsData();
        if !buffer.isValid() || data.as_ref().is_none_or(|data| data.empty()) {
            return Ok(None);
        }
        let mut procs = crate::include::core::SkSerialProcs::SkDeserialProcs::default();
        procs.fAllowSkSL = false;
        unsafe fn allow_tag(tag: u32, _: *mut core::ffi::c_void) -> bool {
            use crate::src::core::SkPictureData::*;
            tag == SK_PICT_BUFFER_SIZE_TAG
                || tag == SK_PICT_FACTORY_TAG
                || tag == SK_PICT_PAINT_BUFFER_TAG
                || tag == SK_PICT_PATH_BUFFER_TAG
                || tag == SK_PICT_READER_TAG
        }
        procs.fAllowTagsProc = Some(allow_tag);
        let picture =
            crate::src::core::SkPicture::MakeFromDataObject(data.as_deref(), Some(&procs))?;
        if !buffer.validate(picture.is_some()) {
            return Ok(None);
        }
        Ok(Some(crate::include::core::SkRefCnt::sk_sp::new(Self::new(
            picture.expect("native validated picture"),
        ))))
    }
    // Source: src/core/SkGlyph.cpp:67-86
    pub unsafe fn FlattenDrawable(
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
        drawable: Option<&dyn crate::src::core::SkDrawable::SkDrawable>,
    ) -> Result<(), crate::RasterError> {
        let Some(drawable) = drawable else {
            unsafe { buffer.writeByteArray(core::ptr::null(), 0) };
            return Ok(());
        };
        let picture = unsafe { drawable.makePictureSnapshot() }?;
        let data = crate::src::core::SkPicture::serialize_data(
            picture.get().expect("native snapshot picture"),
            None,
        )?;
        if data.size() > u32::MAX as usize || data.empty() {
            unsafe { buffer.writeByteArray(core::ptr::null(), 0) };
            return Ok(());
        }
        unsafe { buffer.writeByteArray(data.data(), data.size()) };
        Ok(())
    }
    // Source: src/core/SkGlyph.cpp:88-89
    pub fn new(
        picture: crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
    ) -> Self {
        // Native parent construction precedes the derived picture member. All
        // potentially failing base work precedes the final manual-drop commit.
        let parent = crate::include::core::SkRefCnt::SkRefCnt::new();
        let inherited = crate::src::core::SkDrawable::SkDrawableState::new();
        Self {
            fPicture: core::mem::ManuallyDrop::new(picture),
            inherited,
            fRefCnt: parent,
        }
    }
}
impl crate::src::core::SkDrawable::SkDrawable for SkPictureBackedGlyphDrawable {
    fn drawable_state(&self) -> &crate::src::core::SkDrawable::SkDrawableState {
        &self.inherited
    }
    // Source: src/core/SkGlyph.cpp:91-93
    fn onGetBounds(&self) -> crate::src::core::SkRect::SkRect {
        self.fPicture.cullRect()
    }
    // Source: src/core/SkGlyph.cpp:95-97
    fn onApproximateBytesUsed(&self) -> usize {
        core::mem::size_of::<Self>().wrapping_add(self.fPicture.approximateBytesUsed())
    }
    // Source: src/core/SkGlyph.cpp:99-101
    unsafe fn onDraw(
        &self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::drawPicture_raw(
                canvas,
                self.fPicture.get(),
                None,
                None,
            )
        }
    }
}
impl Drop for SkPictureBackedGlyphDrawable {
    fn drop(&mut self) {
        unsafe { core::mem::ManuallyDrop::drop(&mut self.fPicture) }
    }
}
impl SkGlyph {
    // Source: src/core/SkGlyph.cpp:424-433
    pub unsafe fn flattenDrawable(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),425; @condition b"this->setDrawableHasBeenCalled()\0".as_ptr(); self.setDrawableHasBeenCalled());
        if self.isEmpty() || unsafe { self.drawable() }.is_none() {
            return unsafe { SkPictureBackedGlyphDrawable::FlattenDrawable(buffer, None) };
        }
        unsafe { SkPictureBackedGlyphDrawable::FlattenDrawable(buffer, self.drawable()) }
    }
    // Source: src/core/SkGlyph.cpp:435-448
    pub unsafe fn addDrawableFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<usize, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkGlyph.cpp\0".as_ptr(),436; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let drawable = SkPictureBackedGlyphDrawable::MakeFromBuffer(buffer)?;
        if !buffer.isValid() {
            return Ok(0);
        }
        let drawable = match drawable {
            None => crate::include::core::SkRefCnt::sk_sp::default(),
            Some(mut owner) => {
                let raw = owner.release().expect("native nonnull picture drawable");
                let raw: *mut dyn crate::src::core::SkDrawable::SkDrawable = raw.as_ptr();
                unsafe { crate::include::core::SkRefCnt::sk_sp::from_raw(raw) }
            }
        };
        if unsafe { self.setDrawable(alloc, drawable) } {
            return Ok(unsafe { self.drawable() }
                .expect("native installed drawable")
                .approximateBytesUsed());
        }
        Ok(0)
    }
}
// Source: src/core/SkGlyph.h:253-253
impl SkGlyphRect {
    pub fn new_uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
}
// Source: src/core/SkGlyph.h:336-336
impl SkGlyphDigest {
    pub fn new_uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
}

impl SkGlyphDigest {
    // Source: src/core/SkGlyph.h:347-347
    /// Safety: strike is locked by the current native mutation lease; glyph
    /// belongs to that strike and has no conflicting references during prepare.
    pub unsafe fn setActionFor(
        &mut self,
        action: ActionType,
        glyph: &mut SkGlyph,
        strike: &dyn crate::src::text::StrikeForGPU::StrikeForGPU,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.setActionForCPU(action, glyph, |kind, value| match kind {
                ActionType::kDirectMaskCPU => strike.prepareForImage(value as *mut SkGlyph),
                ActionType::kPath => strike.prepareForPath(value as *mut SkGlyph),
                ActionType::kDrawable => strike.prepareForDrawable(value as *mut SkGlyph),
                _ => unreachable!("native non-producing action does not invoke preparation"),
            })
        }
    }
}
