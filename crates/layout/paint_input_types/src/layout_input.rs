//! Paint-facing value declarations of `layoutng/internal/layout_input.h`.
//! Layout-only input declarations stay in Layout's `layout_input` module,
//! which re-exports these from their original path.

#![allow(non_camel_case_types)]

pub use foundation::LayoutPaintWritingMode as WritingMode;
pub use foundation::{
    BorderLineStyle, LayoutEdges as Edges, LayoutOffset as Offset, LayoutSize as Size,
    TransformMatrix,
};
pub use font_engine::resolver::FontSmoothing;
pub use font_engine::resolver::{FontFace, FontUnicodeRange};
pub use font_engine::{FontFallbackRole, FontVariation};
pub use image_resource::PaintImage;

use crate::layout_input_types::Color;

// cpp: layoutng/internal/layout_input.h:54-57
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    kLtr,
    kRtl,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectFit {
    kFill,
    kContain,
    kCover,
    kNone,
    kScaleDown,
}

// cpp: layoutng/internal/layout_input.h:655-677
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintShadow {
    pub offset: Offset,
    pub blur_radius: f64,
    pub spread: f64,
    pub color: Color,
    pub inset: bool,
}
impl Default for PaintShadow {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            blur_radius: 0.0,
            spread: 0.0,
            color: Color::default(),
            inset: false,
        }
    }
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecorationStyle {
    kSolid,
    kDouble,
    kDotted,
    kDashed,
    kWavy,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TextDecorationPaint {
    pub underline: bool,
    pub overline: bool,
    pub line_through: bool,
    pub spelling_error: bool,
    pub grammar_error: bool,
    pub color: Option<Color>,
    pub style: TextDecorationStyle,
    pub thickness: foundation::TextDecorationThickness,
    pub underline_offset: f64,
    pub underline_offset_auto: bool,
    pub skip_ink: foundation::ETextDecorationSkipInk,
}
impl Default for TextDecorationPaint {
    fn default() -> Self {
        Self {
            underline: false,
            overline: false,
            line_through: false,
            spelling_error: false,
            grammar_error: false,
            color: None,
            style: TextDecorationStyle::kSolid,
            thickness: foundation::TextDecorationThickness::default(),
            underline_offset: 0.0,
            underline_offset_auto: true,
            skip_ink: foundation::ETextDecorationSkipInk::kAuto,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:687-702
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintPathVerb {
    kMoveTo,
    kLineTo,
    kQuadraticTo,
    kConicTo,
    kCubicTo,
    kClose,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintPathCommand {
    pub verb: PaintPathVerb,
    pub control1: Offset,
    pub control2: Offset,
    pub point: Offset,
    pub conic_weight: f64,
}
impl Default for PaintPathCommand {
    fn default() -> Self {
        Self {
            verb: PaintPathVerb::kMoveTo,
            control1: Offset::default(),
            control2: Offset::default(),
            point: Offset::default(),
            conic_weight: 1.0,
        }
    }
}

/// One independently filled path in a synthesized clip.  SVG clipPath child
/// graphics are combined by coverage union, while the clipPath as a whole is
/// intersected with its ancestor clip chain.  Keeping the fill rule per child
/// avoids the lossy "concatenate every contour and use one rule" shortcut.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintClipPathChild {
    pub commands: Vec<PaintPathCommand>,
    pub even_odd: bool,
}

// cpp: layoutng/internal/layout_input.h:704-718
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgShapeGeometry {
    kPath,
    kRectangle,
    kEllipse,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgShapeData {
    pub geometry: SvgShapeGeometry,
    pub bounds_offset: Offset,
    pub bounds_width: f64,
    pub bounds_height: f64,
    pub rectangle_radius_x: f64,
    pub rectangle_radius_y: f64,
    pub path: Vec<PaintPathCommand>,
    pub local_transform: Option<TransformMatrix>,
}
impl Default for SvgShapeData {
    fn default() -> Self {
        Self {
            geometry: SvgShapeGeometry::kPath,
            bounds_offset: Offset::default(),
            bounds_width: 0.0,
            bounds_height: 0.0,
            rectangle_radius_x: 0.0,
            rectangle_radius_y: 0.0,
            path: Vec::new(),
            local_transform: None,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:720-732
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SvgViewBoxData {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub preserve_none: bool,
    pub slice: bool,
    pub align_x: i8,
    pub align_y: i8,
    // cpp: SVGViewSpec::Transform / SVGSVGElement::ViewBoxToViewTransform.
    // This is distinct from the root element's authored CSS/SVG transform and
    // is concatenated after the viewBox-to-viewport mapping.
    pub view_transform: Option<TransformMatrix>,
}

/// Sticky-position constraint inputs Layout publishes for PrePaint's
/// StickyTranslation (Blink cc::StickyPositionConstraint, filled by
/// PaintPropertyTreeBuilder::UpdateStickyTranslation).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyAxisPaintData {
    pub min_inset: Option<f64>,
    pub max_inset: Option<f64>,
    pub scroll_container_relative_containing_block_offset: f64,
    pub scroll_container_relative_containing_block_size: f64,
    pub scroll_container_relative_sticky_box_offset: f64,
    pub scroll_container_relative_sticky_box_size: f64,
    pub constraining_offset: f64,
    pub constraining_size: f64,
    pub scroll_container_client_id: u64,
    pub nearest_sticky_box_client_id: u64,
    pub nearest_containing_block_client_id: u64,
    pub current_sticky_offset: f64,
    pub current_total_sticky_box_offset: f64,
    pub current_total_containing_block_offset: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyPositionPaintData {
    pub owner_client_id: u64,
    pub horizontal: Option<StickyAxisPaintData>,
    pub vertical: Option<StickyAxisPaintData>,
}

impl StickyPositionPaintData {
    /// Compare retained layout geometry while ignoring resolved scroll offsets.
    pub fn same_geometry(&self, other: &Self) -> bool {
        let (mut before, mut after) = (*self, *other);
        for data in [&mut before, &mut after] {
            for axis in [&mut data.horizontal, &mut data.vertical]
                .into_iter()
                .flatten()
            {
                if ![
                    axis.current_sticky_offset,
                    axis.current_total_sticky_box_offset,
                    axis.current_total_containing_block_offset,
                ]
                .into_iter()
                .all(f64::is_finite)
                {
                    return false;
                }
                axis.current_sticky_offset = 0.0;
                axis.current_total_sticky_box_offset = 0.0;
                axis.current_total_containing_block_offset = 0.0;
            }
        }
        before == after
    }
}

/// CSS overscroll-behavior (Blink EOverscrollBehavior; cc::OverscrollBehavior
/// on the ScrollPaintPropertyNode).
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverscrollBehavior {
    #[default]
    kAuto,
    kContain,
    kNone,
    kChain,
}
impl OverscrollBehavior {
    pub fn AllowsScrollChaining(self) -> bool {
        matches!(self, Self::kAuto | Self::kChain)
    }
}
