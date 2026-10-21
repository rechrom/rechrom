// cpp: layoutng_style/style/style_difference.h:112-116
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
enum PaintType {
    NoPaint = 0,
    SimplePaint = 1,
    NormalPaint = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
enum LayoutType {
    NoLayout = 0,
    PositionedLayout = 1,
    FullLayout = 2,
}

const PAINT_SHIFT: u32 = 21;
const LAYOUT_SHIFT: u32 = 23;
const TWO_BITS: u32 = 0b11;

// cpp: layoutng_style/style/style_difference.h:16-128
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleDifference {
    bits: u32,
}

macro_rules! flag_accessors {
    ($($get:ident, $set:ident, $bit:expr;)*) => {
        $(
            pub fn $get(&self) -> bool {
                self.bits & (1u32 << $bit) != 0
            }
            pub fn $set(&mut self, value: bool) {
                let mask = 1u32 << $bit;
                self.bits = (self.bits & !mask) | (u32::from(value) << $bit);
            }
        )*
    };
}

#[allow(non_snake_case)]
impl StyleDifference {
    // cpp: layoutng_style/style/style_difference.h:86-109
    flag_accessors! {
        needs_reshape, set_needs_reshape, 0;
        needs_recompute_visual_overflow, set_needs_recompute_visual_overflow, 1;
        disable_scroll_anchoring, set_disable_scroll_anchoring, 2;
        compositing_reasons_changed, set_compositing_reasons_changed, 3;
        background_color_changed, set_background_color_changed, 4;
        blend_mode_changed, set_blend_mode_changed, 5;
        border_radius_changed, set_border_radius_changed, 6;
        border_shape_changed, set_border_shape_changed, 7;
        clip_path_changed, set_clip_path_changed, 8;
        clip_property_changed, set_clip_property_changed, 9;
        filter_changed, set_filter_changed, 10;
        mask_changed, set_mask_changed, 11;
        opacity_changed, set_opacity_changed, 12;
        only_transform_property_changed, set_only_transform_property_changed, 13;
        text_decoration_or_color_changed, set_text_decoration_or_color_changed, 14;
        transform_changed, set_transform_changed, 15;
        transform_data_changed, set_transform_data_changed, 16;
        z_index_changed, set_z_index_changed, 17;
        needs_box_paint_property_update, set_needs_box_paint_property_update, 18;
        ax_visibility_or_inert_changed, set_ax_visibility_or_inert_changed, 19;
        ax_style_changed, set_ax_style_changed, 20;
    }

    fn paint_type(&self) -> PaintType {
        match (self.bits >> PAINT_SHIFT) & TWO_BITS {
            0 => PaintType::NoPaint,
            1 => PaintType::SimplePaint,
            2 => PaintType::NormalPaint,
            _ => unreachable!("private C++ paint type cannot have value 3"),
        }
    }

    fn set_paint_type(&mut self, value: PaintType) {
        self.bits = (self.bits & !(TWO_BITS << PAINT_SHIFT)) | ((value as u32) << PAINT_SHIFT);
    }

    fn layout_type(&self) -> LayoutType {
        match (self.bits >> LAYOUT_SHIFT) & TWO_BITS {
            0 => LayoutType::NoLayout,
            1 => LayoutType::PositionedLayout,
            2 => LayoutType::FullLayout,
            _ => unreachable!("private C++ layout type cannot have value 3"),
        }
    }

    fn set_layout_type(&mut self, value: LayoutType) {
        self.bits = (self.bits & !(TWO_BITS << LAYOUT_SHIFT)) | ((value as u32) << LAYOUT_SHIFT);
    }

    // cpp: layoutng_style/style/style_difference.h:20-44
    pub fn Merge(&mut self, other: StyleDifference) {
        self.set_needs_reshape(self.needs_reshape() | other.needs_reshape());
        self.set_needs_recompute_visual_overflow(
            self.needs_recompute_visual_overflow() | other.needs_recompute_visual_overflow(),
        );
        self.set_disable_scroll_anchoring(
            self.disable_scroll_anchoring() | other.disable_scroll_anchoring(),
        );
        self.set_compositing_reasons_changed(
            self.compositing_reasons_changed() | other.compositing_reasons_changed(),
        );
        self.set_background_color_changed(
            self.background_color_changed() | other.background_color_changed(),
        );
        self.set_blend_mode_changed(self.blend_mode_changed() | other.blend_mode_changed());
        self.set_border_radius_changed(
            self.border_radius_changed() | other.border_radius_changed(),
        );
        self.set_border_shape_changed(self.border_shape_changed() | other.border_shape_changed());
        self.set_clip_path_changed(self.clip_path_changed() | other.clip_path_changed());
        self.set_clip_property_changed(
            self.clip_property_changed() | other.clip_property_changed(),
        );
        self.set_filter_changed(self.filter_changed() | other.filter_changed());
        self.set_mask_changed(self.mask_changed() | other.mask_changed());
        self.set_opacity_changed(self.opacity_changed() | other.opacity_changed());
        self.set_only_transform_property_changed(
            self.only_transform_property_changed() | other.only_transform_property_changed(),
        );
        self.set_text_decoration_or_color_changed(
            self.text_decoration_or_color_changed() | other.text_decoration_or_color_changed(),
        );
        self.set_transform_changed(self.transform_changed() | other.transform_changed());
        self.set_transform_data_changed(
            self.transform_data_changed() | other.transform_data_changed(),
        );
        self.set_z_index_changed(self.z_index_changed() | other.z_index_changed());
        self.set_paint_type(self.paint_type().max(other.paint_type()));
        self.set_layout_type(self.layout_type().max(other.layout_type()));
        self.set_needs_box_paint_property_update(
            self.needs_box_paint_property_update() | other.needs_box_paint_property_update(),
        );
        self.set_ax_visibility_or_inert_changed(
            self.ax_visibility_or_inert_changed() | other.ax_visibility_or_inert_changed(),
        );
        self.set_ax_style_changed(self.ax_style_changed() | other.ax_style_changed());
    }

    // cpp: layoutng_style/style/style_difference.h:46-57
    pub fn HasDifference(&self) -> bool {
        self.needs_reshape()
            || self.needs_recompute_visual_overflow()
            || self.disable_scroll_anchoring()
            || self.compositing_reasons_changed()
            || self.background_color_changed()
            || self.blend_mode_changed()
            || self.border_radius_changed()
            || self.border_shape_changed()
            || self.clip_path_changed()
            || self.clip_property_changed()
            || self.filter_changed()
            || self.mask_changed()
            || self.opacity_changed()
            || self.only_transform_property_changed()
            || self.text_decoration_or_color_changed()
            || self.transform_changed()
            || self.transform_data_changed()
            || self.z_index_changed()
            || self.paint_type() != PaintType::NoPaint
            || self.layout_type() != LayoutType::NoLayout
            || self.needs_box_paint_property_update()
            || self.ax_visibility_or_inert_changed()
            || self.ax_style_changed()
    }

    // cpp: layoutng_style/style/style_difference.h:62-67
    pub fn NeedsSimplePaintInvalidation(&self) -> bool {
        self.paint_type() == PaintType::SimplePaint
    }
    pub fn NeedsNormalPaintInvalidation(&self) -> bool {
        self.paint_type() == PaintType::NormalPaint
    }

    // cpp: layoutng_style/style/style_difference.h:69-73
    pub fn SetNeedsNormalPaintInvalidation(&mut self) {
        self.set_paint_type(PaintType::NormalPaint);
    }
    pub fn SetNeedsSimplePaintInvalidation(&mut self) {
        debug_assert!(!self.NeedsNormalPaintInvalidation());
        self.set_paint_type(PaintType::SimplePaint);
    }

    // cpp: layoutng_style/style/style_difference.h:75-78
    pub fn NeedsFullLayout(&self) -> bool {
        self.layout_type() == LayoutType::FullLayout
    }
    pub fn NeedsPositionedLayout(&self) -> bool {
        self.layout_type() == LayoutType::PositionedLayout
    }

    // cpp: layoutng_style/style/style_difference.h:80-84
    pub fn SetNeedsFullLayout(&mut self) {
        self.set_layout_type(LayoutType::FullLayout);
    }
    pub fn SetNeedsPositionedLayout(&mut self) {
        debug_assert!(!self.NeedsFullLayout());
        self.set_layout_type(LayoutType::PositionedLayout);
    }
}

// cpp: layoutng_style/style/style_difference.h:125-130
// The ostream insertion declaration has no definition in the supplied C++ tree.
