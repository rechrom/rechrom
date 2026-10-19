//! Selected-source PaintOpBuffer metadata, initialization, move and reset.
//! Byte storage/serialization/playback are outside this selected-source module.
#![allow(non_snake_case)]
// Source: cc/paint/paint_op_buffer.h:111.
pub const MIN_NUMBER_OF_SLOW_PATHS_FOR_MSAA: i32 = 6;
/// Op method projections have the same meaning as cc::PaintOp subtype methods;
/// they must inspect that actual op and its retained subrecord owners.
pub trait MetadataOp<C> {
    fn IsValid(&self) -> bool;
    fn CountSlowPathsFromFlags(&self) -> i32;
    fn CountSlowPaths(&self) -> i32;
    fn HasNonAAPaint(&self) -> bool;
    fn AdditionalBytesUsed(&self) -> usize;
    fn AdditionalOpCount(&self) -> usize;
    fn IsDrawOp(&self) -> bool;
    fn HasDrawTextOps(&self) -> bool;
    fn HasSaveLayerOps(&self) -> bool;
    fn HasSaveLayerAlphaOps(&self) -> bool;
    fn HasEffectsPreventingLCDTextForSaveLayerAlpha(&self) -> bool;
    fn HasDiscardableImages(&self, usage: &mut C) -> bool;
    fn HasDiscardableImagesFromFlags(&self, usage: &mut C) -> bool;
}
// Source: cc/paint/paint_op_buffer.h:379-400. data_ is owned by the storage
// adapter; its used/capacity sizes are supplied to the native getter formulas.
#[derive(Clone)]
pub struct PaintOpBufferMetadata<C> {
    used_: usize,
    op_count_: usize,
    subrecord_bytes_used_: usize,
    subrecord_op_count_: usize,
    num_slow_paths_up_to_min_for_MSAA_: i32,
    has_non_aa_paint_: bool,
    has_draw_ops_: bool,
    has_draw_text_ops_: bool,
    has_save_layer_ops_: bool,
    has_save_layer_alpha_ops_: bool,
    has_effects_preventing_lcd_text_for_save_layer_alpha_: bool,
    has_discardable_images_: bool,
    content_color_usage_: C,
}
impl<C: Copy> PaintOpBufferMetadata<C> {
    // Source: cc/paint/paint_op_buffer.cc:76-80; h:379-400.
    // `srgb` is the actual ContentColorUsage::kSRGB typed value, not Default.
    pub fn new(srgb: C) -> Self {
        Self {
            used_: 0,
            op_count_: 0,
            subrecord_bytes_used_: 0,
            subrecord_op_count_: 0,
            num_slow_paths_up_to_min_for_MSAA_: 0,
            has_non_aa_paint_: false,
            has_draw_ops_: false,
            has_draw_text_ops_: false,
            has_save_layer_ops_: false,
            has_save_layer_alpha_ops_: false,
            has_effects_preventing_lcd_text_for_save_layer_alpha_: false,
            has_discardable_images_: false,
            content_color_usage_: srgb,
        }
    }
    // Source: cc/paint/paint_op_buffer.h:370-371, selected counter writes.
    // Actual allocation/alignment remains the owning storage implementation.
    pub fn DidAllocatePaintOp(&mut self, aligned_size: usize) {
        self.used_ += aligned_size;
        self.op_count_ += 1;
    }
    // Source: cc/paint/paint_op_buffer.h:285-310.
    pub fn AnalyzeAddedOp(&mut self, op: &impl MetadataOp<C>) {
        debug_assert!(op.IsValid());
        if self.num_slow_paths_up_to_min_for_MSAA_ < MIN_NUMBER_OF_SLOW_PATHS_FOR_MSAA {
            self.num_slow_paths_up_to_min_for_MSAA_ += op.CountSlowPathsFromFlags();
            self.num_slow_paths_up_to_min_for_MSAA_ += op.CountSlowPaths();
        }
        self.has_non_aa_paint_ |= op.HasNonAAPaint();
        self.subrecord_bytes_used_ += op.AdditionalBytesUsed();
        self.subrecord_op_count_ += op.AdditionalOpCount();
        self.has_draw_ops_ |= op.IsDrawOp();
        self.has_draw_text_ops_ |= op.HasDrawTextOps();
        self.has_save_layer_ops_ |= op.HasSaveLayerOps();
        self.has_save_layer_alpha_ops_ |= op.HasSaveLayerAlphaOps();
        self.has_effects_preventing_lcd_text_for_save_layer_alpha_ |=
            op.HasEffectsPreventingLCDTextForSaveLayerAlpha();
        self.has_discardable_images_ |= op.HasDiscardableImages(&mut self.content_color_usage_);
        self.has_discardable_images_ |=
            op.HasDiscardableImagesFromFlags(&mut self.content_color_usage_);
    }
    // Source: cc/paint/paint_op_buffer.h:213-245.
    pub fn size(&self) -> usize {
        self.op_count_
    }
    pub fn empty(&self) -> bool {
        self.size() == 0
    }
    pub fn bytes_used(&self, owner_size: usize, data_capacity: usize) -> usize {
        owner_size + data_capacity + self.subrecord_bytes_used_
    }
    pub fn paint_ops_size(&self) -> usize {
        self.used_ + self.subrecord_bytes_used_
    }
    pub fn total_op_count(&self) -> usize {
        self.op_count_ + self.subrecord_op_count_
    }
    pub fn next_op_offset(&self) -> usize {
        self.used_
    }
    pub fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32 {
        self.num_slow_paths_up_to_min_for_MSAA_
    }
    pub fn has_non_aa_paint(&self) -> bool {
        self.has_non_aa_paint_
    }
    pub fn has_draw_ops(&self) -> bool {
        self.has_draw_ops_
    }
    pub fn has_draw_text_ops(&self) -> bool {
        self.has_draw_text_ops_
    }
    pub fn has_save_layer_ops(&self) -> bool {
        self.has_save_layer_ops_
    }
    pub fn has_save_layer_alpha_ops(&self) -> bool {
        self.has_save_layer_alpha_ops_
    }
    pub fn has_effects_preventing_lcd_text_for_save_layer_alpha(&self) -> bool {
        self.has_effects_preventing_lcd_text_for_save_layer_alpha_
    }
    pub fn has_discardable_images(&self) -> bool {
        self.has_discardable_images_
    }
    pub fn content_color_usage(&self) -> C {
        self.content_color_usage_
    }
    // Source: cc/paint/paint_op_buffer.cc:283-304. The buffer owner moves
    // data_ first, then moves this metadata and resets the source for reuse.
    pub fn MoveFrom(&mut self, other: &mut Self, srgb: C) {
        self.used_ = other.used_;
        self.op_count_ = other.op_count_;
        self.num_slow_paths_up_to_min_for_MSAA_ = other.num_slow_paths_up_to_min_for_MSAA_;
        self.subrecord_bytes_used_ = other.subrecord_bytes_used_;
        self.subrecord_op_count_ = other.subrecord_op_count_;
        self.has_non_aa_paint_ = other.has_non_aa_paint_;
        self.has_draw_ops_ = other.has_draw_ops_;
        self.has_draw_text_ops_ = other.has_draw_text_ops_;
        self.has_save_layer_ops_ = other.has_save_layer_ops_;
        self.has_save_layer_alpha_ops_ = other.has_save_layer_alpha_ops_;
        self.has_effects_preventing_lcd_text_for_save_layer_alpha_ =
            other.has_effects_preventing_lcd_text_for_save_layer_alpha_;
        self.has_discardable_images_ = other.has_discardable_images_;
        self.content_color_usage_ = other.content_color_usage_;
        other.ResetRetainingBuffer(srgb);
    }
    // Source: cc/paint/paint_op_buffer.cc:324-339.
    pub fn ResetRetainingBuffer(&mut self, srgb: C) {
        self.used_ = 0;
        self.op_count_ = 0;
        self.num_slow_paths_up_to_min_for_MSAA_ = 0;
        self.has_non_aa_paint_ = false;
        self.subrecord_bytes_used_ = 0;
        self.subrecord_op_count_ = 0;
        self.has_draw_ops_ = false;
        self.has_draw_text_ops_ = false;
        self.has_save_layer_ops_ = false;
        self.has_save_layer_alpha_ops_ = false;
        self.has_effects_preventing_lcd_text_for_save_layer_alpha_ = false;
        self.has_discardable_images_ = false;
        self.content_color_usage_ = srgb;
    }
    // Source: cc/paint/paint_op_buffer.cc:588-601.
    pub fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool {
        if !self.has_draw_text_ops() || !self.has_save_layer_alpha_ops() {
            return false;
        }
        if !old.has_draw_text_ops() || !old.has_save_layer_alpha_ops() {
            return false;
        }
        self.has_effects_preventing_lcd_text_for_save_layer_alpha()
            != old.has_effects_preventing_lcd_text_for_save_layer_alpha()
    }
}
// Selected-source translation only; no whole-stem completion claim.
