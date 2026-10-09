// cpp: foundation/blink_base/heap/trace_traits.h:20-28
// C++ selects tracing with IsTraceableV<T>. Rust has no stable specialization,
// so these concrete style field types state that policy explicitly. The
// source-mapped Trace bodies remain in their original Rust modules.
use foundation::{Traceable, Visitor};

use crate::css::style_auto_color::StyleAutoColor;
use crate::css::style_caret_color::StyleCaretColor;

use super::computed_grid_template_areas::ComputedGridTemplateAreas;
use super::computed_style_constants::TimelineAxis;
use super::flow_tolerance::FlowTolerance;
use super::gap_data::GapValue;
use super::gap_data_list::GapDataList;
use super::grid_lanes_direction::GridLanesDirection;
use super::grid_position::GridPosition;
use super::grid_track_list::GridTrackList;
use super::max_lines_data::MaxLinesData;
use super::position_area::{PositionArea, PositionAreaOffsets};
use super::scroll_marker_group::ScrollMarkerGroup;
use super::scroll_snap_data::cc::{ScrollSnapAlign, ScrollSnapType};
use super::shape_value::ShapeValue;
use super::style_border_shape::StyleBorderShape;
use super::style_content_alignment_data::StyleContentAlignmentData;
use super::style_flex_wrap_data::StyleFlexWrapData;
use super::style_highlight_data::StyleHighlightData;
use super::style_hyphenate_limit_chars::StyleHyphenateLimitChars;
use super::style_inherited_variables::StyleInheritedVariables;
use super::style_interest_delay::StyleInterestDelay;
use super::style_intrinsic_length::StyleIntrinsicLength;
use super::style_non_inherited_variables::StyleNonInheritedVariables;
use super::style_offset_rotation::StyleOffsetRotation;
use super::style_overflow_clip_margin::StyleOverflowClipMargin;
use super::style_position_anchor::StylePositionAnchor;
use super::style_self_alignment_data::StyleSelfAlignmentData;
use super::style_timeline_scope::StyleTimelineScope;
use super::style_view_transition_group::StyleViewTransitionGroup;
use super::superellipse::Superellipse;
use super::svg_paint::SVGPaint;
use super::text_decoration_inset::TextDecorationInset;
use super::text_fit::TextFit;
use super::text_overflow_data::TextOverflowData;
use super::text_size_adjust::TextSizeAdjust;
use super::timeline_inset::TimelineInset;
use super::transform_origin::TransformOrigin;
use super::unzoomed_length::UnzoomedLength;

macro_rules! trace_source_method {
    ($($type:ty),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                <$type>::Trace(self, visitor);
            }
        })+
    };
}

trace_source_method!(
    StyleAutoColor,
    StyleCaretColor,
    StyleHighlightData,
    StyleInheritedVariables,
    StyleNonInheritedVariables,
    StylePositionAnchor,
    SVGPaint,
    ShapeValue,
    StyleBorderShape,
);

// This source Trace method has no managed fields and takes an optional
// visitor; bind its actual body without adding a second policy.
// cpp: layoutng_style/style/computed_grid_template_areas.h:24-24
impl Traceable for ComputedGridTemplateAreas {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ComputedGridTemplateAreas::Trace(self, Some(visitor));
    }
}
// cpp: layoutng_style/style/scroll_marker_group.h:41. Bind the source body.
impl Traceable for ScrollMarkerGroup {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ScrollMarkerGroup::Trace(self, Some(visitor));
    }
}

impl<T: GapValue> Traceable for GapDataList<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        GapDataList::Trace(self, visitor);
    }
}

// These values contain only scalars, interned strings, ordinary vectors, or
// Length handles. None owns a layout-heap Member edge.
macro_rules! trace_unmanaged_value {
    ($($type:ty),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, _visitor: &mut Visitor<'_>) {}
        })+
    };
}

trace_unmanaged_value!(
    FlowTolerance,
    GridLanesDirection,
    GridPosition,
    GridTrackList,
    MaxLinesData,
    PositionArea,
    PositionAreaOffsets,
    ScrollSnapAlign,
    ScrollSnapType,
    StyleContentAlignmentData,
    StyleFlexWrapData,
    StyleHyphenateLimitChars,
    StyleInterestDelay,
    StyleIntrinsicLength,
    StyleOffsetRotation,
    StyleOverflowClipMargin,
    StyleSelfAlignmentData,
    StyleTimelineScope,
    StyleViewTransitionGroup,
    Superellipse,
    TextDecorationInset,
    TextFit,
    TextOverflowData,
    TextSizeAdjust,
    TimelineAxis,
    TimelineInset,
    TransformOrigin,
    UnzoomedLength,
);
