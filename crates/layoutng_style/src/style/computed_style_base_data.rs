use std::cell::Cell;

use font_engine::Font;
use foundation::gfx;
use foundation::{
    g_null_atom, keywords, AtomicString, CSSValueID, Color, DynamicRangeLimit,
    DynamicRangeLimitKind, EAlignmentBaseline, EAspectRatioType, EBaselineSource, EBorderStyle,
    EBoxDecorationBreak, EBufferedRendering, EColorInterpolation, EColorRendering, ECursor,
    EDominantBaseline, EFrameSizing, EMaskType, EShapeRendering, ETextAnchor, ETextBoxTrim,
    ETextTransform, EVectorEffect, EmptyTransformOperations, Length, LengthBox, LengthPoint,
    LengthSize, LineCap, LineJoin, MakeGarbageCollected, Member, ScopedRefPtr, StyleAspectRatio,
    TabSize, TextDecorationThickness, TouchAction, Vector, Visitor, WindRule,
};
use foundation::{
    BlendMode, EBackfaceVisibility, EBoxAlignment, EBoxOrient, EBoxPack, EBreakBetween,
    EBreakInside, EColumnFill, EColumnSpan, EColumnWrap, EContinue, EFlexDirection, EGridLanesPack,
    EInternalOverscrollContainer, EInternalOverscrollPosition, EInternalUnbounded, EIsolation,
    EMaxContentSizing, EObjectFit, EOverlay, EOverscrollBehavior, EOverscrollContainerType,
    EPageMarginSafety, EPositionTryOrder, EReadingFlow, EResize, ERuleOverlap, EScrollAxisLock,
    EScrollInitialTarget, EScrollTargetGroup, EScrollbarWidth, ETextDecorationStyle,
    ETransformStyle3D, EUserDrag, EViewTransitionScope, RuleBreak, RuleVisibilityItems,
};
use foundation::{
    EBlockEllipsis, EDraggableRegionMode, EEmptyCells, EForcedColorAdjust, EImageRendering,
    EInteractivity, EInterpolateSize, EMathShift, EMathStyle, EOverflowWrap, ERubyAlign,
    ERubyOverhang, ESpeak, ETextAlignLast, ETextAutospace, ETextCombine, ETextDecorationSkipInk,
    ETextOrientation, ETextSecurity, EUserModify, EUserSelect, EWordBreak, HangingPunctuation,
    Hyphens, ImageAnimationEnum, LineBreak, RespectImageOrientationEnum, RubyPosition,
    TextDecorationSkipSpaces, TextEmphasisFill, TextEmphasisMark, TextJustify,
};
use foundation::{
    EBorderCollapse, EBoxDirection, EBoxSizing, ECaptionSide, ECaretAnimation, ECaretShape, EClear,
    EContentVisibility, EDisplay, EFloat, EInlineBlockBaselineEdge, EInsideLink,
    EListStylePosition, EOrder, EOriginTrialTestProperty, EOverflow, EOverflowAnchor,
    EPointerEvents, EPosition, EPrintColorAdjust, EScrollSnapStop, ETableLayout, ETextAlign,
    ETransformBox, EVisibility, TextDirection, TextWrapMode, TextWrapStyle, UnicodeBidi,
    WritingMode,
};

use super::text_size_adjust::TextSizeAdjust;
use crate::css::style_auto_color::StyleAutoColor;
use crate::css::style_caret_color::StyleCaretColor;
use crate::css::style_color::StyleColor;
use crate::css::white_space::WhiteSpaceCollapse;

use super::appearance::AppearanceValue;
use super::computed_style::ComputedStyle;
use super::computed_style_base::{
    BuilderAccessFlags, ComputedStyleBase, ComputedStyleBaseData, ComputedStyleBuilderBase,
    StyleBackgroundData, StyleBoxData, StyleFillData, StyleForcedColorsData, StyleGeometryData,
    StyleHighlightDataData, StyleInheritedData, StyleInheritedForcedColorsData,
    StyleInheritedVisitedData, StyleMathData, StyleMisc1Data, StyleMisc2Data, StyleMisc3Data,
    StyleMisc4Data, StyleMisc5Data, StyleMisc6Data, StyleMisc7Data, StyleMisc8Data, StyleMiscData,
    StyleMiscInherited1Data, StyleMiscInherited2Data, StyleMiscInheritedData, StyleResourcesData,
    StyleSVGData, StyleStopData, StyleStrokeData, StyleSurroundData, StyleSvginheritedData,
    StyleSvgmiscData, StyleTimelineData, StyleVisitedData, StyleVisualData,
};
use super::computed_style_constants::{
    Containment, ContentDistributionType, ContentPosition, EBaselineShiftType, EContainerType,
    EFillLayerType, EMarginTrim, EPaintOrder, EVerticalAlign, FlexWrapMode, GridAutoFlow,
    ItemPosition, OffsetRotationType, OverflowAlignment, PositionVisibility, PseudoId,
    ScrollbarGutter, TextUnderlinePosition,
};
use super::computed_style_initial_values::ComputedStyleInitialValues;
use super::fill_layer::FillLayer;
use super::filter_operations::FilterOperations;
use super::flow_tolerance::FlowTolerance;
use super::gap_data_list::GapDataList;
use super::grid_lanes_direction::GridLanesDirection;
use super::grid_position::GridPosition;
use super::grid_track_list::GridTrackList;
use super::grid_track_size::GridTrackSize;
use super::list_style_type_data::ListStyleTypeData;
use super::max_lines_data::MaxLinesData;
use super::member_copy::{MemberCopyContentData, MemberCopyPaintImages};
use super::nine_piece_image::NinePieceImage;
use super::page_orientation::PageOrientation;
use super::page_size_type::PageSizeType;
use super::position_area::PositionArea;
use super::scroll_enums::mojom;
use super::style_anchor_scope::StyleAnchorScope;
use super::style_content_alignment_data::StyleContentAlignmentData;
use super::style_flex_wrap_data::StyleFlexWrapData;
use super::style_highlight_data::StyleHighlightData;
use super::style_hyphenate_limit_chars::StyleHyphenateLimitChars;
use super::style_inherited_variables::StyleInheritedVariables;
use super::style_interest_delay::StyleInterestDelay;
use super::style_intrinsic_length::StyleIntrinsicLength;
use super::style_non_inherited_variables::StyleNonInheritedVariables;
use super::style_offset_rotation::StyleOffsetRotation;
use super::style_position_anchor::StylePositionAnchor;
use super::style_self_alignment_data::StyleSelfAlignmentData;
use super::style_timeline_scope::StyleTimelineScope;
use super::style_trigger_scope::StyleTriggerScope;
use super::style_view_transition_group::StyleViewTransitionGroup;
use super::superellipse::Superellipse;
use super::svg_paint::SVGPaint;
use super::text_box_edge::TextBoxEdge;
use super::text_decoration_inset::TextDecorationInset;
use super::text_fit::TextFit;
use super::text_indent_flags::TextIndentFlags;
use super::text_overflow_data::{TextOverflowData, TextOverflowType};
use super::theme_defaults::layout_style_defaults;
use super::transform_origin::TransformOrigin;
use super::unzoomed_length::UnzoomedLength;

// cpp: layoutng_style/style/computed_style_base_data.cc:143-160
impl Default for StyleMiscInherited1Data {
    fn default() -> Self {
        Self {
            hyphenation_string_: AtomicString::default(),
            color_scheme_: Vector::default(),
            quotes_: ScopedRefPtr::default(),
            list_style_image_: Member::default(),
            list_style_type_: Member::from_ptr(ListStyleTypeData::CreateCounterStyle(
                &keywords::kDisc,
                std::ptr::null(),
            )),
            scrollbar_color_: Member::default(),
            dynamic_range_limit_: DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh),
            tab_size_: TabSize::spaces(8.0),
            text_fit_: TextFit::default(),
            text_size_adjust_: TextSizeAdjust::AdjustAuto(),
            accent_color_: StyleAutoColor::AutoColor(),
            caret_color_: StyleCaretColor::default(),
            text_emphasis_color_: StyleColor::CurrentColor(),
            math_depth_: 0,
            orphans_: 2,
            hyphenate_limit_chars_: StyleHyphenateLimitChars::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:162-179
impl Clone for StyleMiscInherited1Data {
    fn clone(&self) -> Self {
        Self {
            hyphenation_string_: self.hyphenation_string_.clone(),
            color_scheme_: self.color_scheme_.clone(),
            quotes_: self.quotes_.clone(),
            list_style_image_: self.list_style_image_.clone(),
            list_style_type_: self.list_style_type_.clone(),
            scrollbar_color_: self.scrollbar_color_.clone(),
            dynamic_range_limit_: self.dynamic_range_limit_.clone(),
            tab_size_: self.tab_size_.clone(),
            text_fit_: self.text_fit_.clone(),
            text_size_adjust_: self.text_size_adjust_.clone(),
            accent_color_: self.accent_color_.clone(),
            caret_color_: self.caret_color_.clone(),
            text_emphasis_color_: self.text_emphasis_color_.clone(),
            math_depth_: self.math_depth_,
            orphans_: self.orphans_,
            hyphenate_limit_chars_: self.hyphenate_limit_chars_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:181-196
impl Default for StyleMiscInherited2Data {
    fn default() -> Self {
        Self {
            text_emphasis_custom_mark_: AtomicString::default(),
            ua_shadow_host_data_: None,
            text_shadow_: Member::default(),
            cursor_data_: Member::default(),
            initial_data_: Member::default(),
            text_indent_: Length::Fixed(0),
            text_underline_offset_: Length::default(),
            text_stroke_width_: 0.0,
            effective_zoom_: 1.0,
            tap_highlight_color_: StyleColor::from_color(
                layout_style_defaults::kDefaultTapHighlightColor,
            ),
            text_fill_color_: StyleColor::CurrentColor(),
            text_stroke_color_: StyleColor::CurrentColor(),
            widows_: 2,
            effective_touch_action_bits_: TouchAction::kAuto.bits() as u32,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:198-213
impl Clone for StyleMiscInherited2Data {
    fn clone(&self) -> Self {
        Self {
            text_emphasis_custom_mark_: self.text_emphasis_custom_mark_.clone(),
            ua_shadow_host_data_: self.ua_shadow_host_data_.clone(),
            text_shadow_: self.text_shadow_.clone(),
            cursor_data_: self.cursor_data_.clone(),
            initial_data_: self.initial_data_.clone(),
            text_indent_: self.text_indent_.clone(),
            text_underline_offset_: self.text_underline_offset_.clone(),
            text_stroke_width_: self.text_stroke_width_,
            effective_zoom_: self.effective_zoom_,
            tap_highlight_color_: self.tap_highlight_color_.clone(),
            text_fill_color_: self.text_fill_color_.clone(),
            text_stroke_color_: self.text_stroke_color_.clone(),
            widows_: self.widows_,
            effective_touch_action_bits_: self.effective_touch_action_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:443-447
impl Default for StyleForcedColorsData {
    fn default() -> Self {
        Self {
            internal_forced_background_color_: StyleColor::from_keyword(CSSValueID::kCanvas),
            internal_forced_border_color_: StyleColor::CurrentColor(),
            internal_forced_outline_color_: StyleColor::CurrentColor(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:449-453
impl Clone for StyleForcedColorsData {
    fn clone(&self) -> Self {
        Self {
            internal_forced_background_color_: self.internal_forced_background_color_.clone(),
            internal_forced_border_color_: self.internal_forced_border_color_.clone(),
            internal_forced_outline_color_: self.internal_forced_outline_color_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:901-904
impl Default for StyleInheritedForcedColorsData {
    fn default() -> Self {
        Self {
            internal_forced_visited_color_: StyleColor::from_keyword(CSSValueID::kCanvastext),
            internal_forced_color_: StyleColor::from_keyword(CSSValueID::kCanvastext),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:906-909
impl Clone for StyleInheritedForcedColorsData {
    fn clone(&self) -> Self {
        Self {
            internal_forced_visited_color_: self.internal_forced_visited_color_.clone(),
            internal_forced_color_: self.internal_forced_color_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:911-916
impl Default for StyleInheritedVisitedData {
    fn default() -> Self {
        Self {
            internal_visited_caret_color_: StyleCaretColor::default(),
            internal_visited_text_emphasis_color_: StyleColor::CurrentColor(),
            internal_visited_text_fill_color_: StyleColor::CurrentColor(),
            internal_visited_text_stroke_color_: StyleColor::CurrentColor(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:918-923
impl Clone for StyleInheritedVisitedData {
    fn clone(&self) -> Self {
        Self {
            internal_visited_caret_color_: self.internal_visited_caret_color_.clone(),
            internal_visited_text_emphasis_color_: self
                .internal_visited_text_emphasis_color_
                .clone(),
            internal_visited_text_fill_color_: self.internal_visited_text_fill_color_.clone(),
            internal_visited_text_stroke_color_: self.internal_visited_text_stroke_color_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:925-927
impl Default for StyleHighlightDataData {
    fn default() -> Self {
        Self {
            highlight_data_: StyleHighlightData::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:929-931
impl Clone for StyleHighlightDataData {
    fn clone(&self) -> Self {
        Self {
            highlight_data_: self.highlight_data_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:569-576
impl Default for StyleTimelineData {
    fn default() -> Self {
        Self {
            timeline_scope_: StyleTimelineScope::default(),
            scroll_timeline_name_: Vector::default(),
            view_timeline_name_: Vector::default(),
            scroll_timeline_axis_: Vector::default(),
            view_timeline_axis_: Vector::default(),
            view_timeline_inset_: Vector::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:578-585
impl Clone for StyleTimelineData {
    fn clone(&self) -> Self {
        Self {
            timeline_scope_: self.timeline_scope_.clone(),
            scroll_timeline_name_: self.scroll_timeline_name_.clone(),
            view_timeline_name_: self.view_timeline_name_.clone(),
            scroll_timeline_axis_: self.scroll_timeline_axis_.clone(),
            view_timeline_axis_: self.view_timeline_axis_.clone(),
            view_timeline_inset_: self.view_timeline_inset_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:645-654
impl Default for StyleMathData {
    fn default() -> Self {
        Self {
            math_baseline_: Length::default(),
            math_fraction_bar_thickness_: Length::default(),
            math_l_space_: Length::default(),
            math_r_space_: Length::default(),
            math_padded_v_offset_: Length::default(),
            math_padded_depth_: Length::default(),
            math_min_size_: Length::default(),
            math_max_size_: Length::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:656-665
impl Clone for StyleMathData {
    fn clone(&self) -> Self {
        Self {
            math_baseline_: self.math_baseline_.clone(),
            math_fraction_bar_thickness_: self.math_fraction_bar_thickness_.clone(),
            math_l_space_: self.math_l_space_.clone(),
            math_r_space_: self.math_r_space_.clone(),
            math_padded_v_offset_: self.math_padded_v_offset_.clone(),
            math_padded_depth_: self.math_padded_depth_.clone(),
            math_min_size_: self.math_min_size_.clone(),
            math_max_size_: self.math_max_size_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:997-1001
impl Default for StyleFillData {
    fn default() -> Self {
        Self {
            internal_visited_fill_paint_: SVGPaint::from_color(Color::kBlack),
            fill_paint_: SVGPaint::CreateInitialBlack(),
            fill_opacity_: 1.0,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1003-1007
impl Clone for StyleFillData {
    fn clone(&self) -> Self {
        Self {
            internal_visited_fill_paint_: self.internal_visited_fill_paint_.clone(),
            fill_paint_: self.fill_paint_.clone(),
            fill_opacity_: self.fill_opacity_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1033-1037
impl Default for StyleResourcesData {
    fn default() -> Self {
        Self {
            marker_end_resource_: Member::default(),
            marker_mid_resource_: Member::default(),
            marker_start_resource_: Member::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1039-1043
impl Clone for StyleResourcesData {
    fn clone(&self) -> Self {
        Self {
            marker_end_resource_: self.marker_end_resource_.clone(),
            marker_mid_resource_: self.marker_mid_resource_.clone(),
            marker_start_resource_: self.marker_start_resource_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1185-1188
impl Default for StyleStopData {
    fn default() -> Self {
        Self {
            stop_opacity_: 1.0,
            stop_color_: StyleColor::from_color(Color::kBlack),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1190-1193
impl Clone for StyleStopData {
    fn clone(&self) -> Self {
        Self {
            stop_opacity_: self.stop_opacity_,
            stop_color_: self.stop_color_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1277-1280
impl Default for StyleBackgroundData {
    fn default() -> Self {
        Self {
            background_: FillLayer::new(EFillLayerType::kBackground, true),
            background_color_: StyleColor::from_color(Color::kTransparent),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1282-1285
impl Clone for StyleBackgroundData {
    fn clone(&self) -> Self {
        Self {
            background_: self.background_.clone(),
            background_color_: self.background_color_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:345-354
impl Default for StyleVisitedData {
    fn default() -> Self {
        Self {
            internal_visited_column_rule_color_: GapDataList::DefaultGapColorDataList(),
            internal_visited_background_color_: StyleColor::from_color(Color::kTransparent),
            internal_visited_border_bottom_color_: StyleColor::CurrentColor(),
            internal_visited_border_left_color_: StyleColor::CurrentColor(),
            internal_visited_border_right_color_: StyleColor::CurrentColor(),
            internal_visited_border_top_color_: StyleColor::CurrentColor(),
            internal_visited_outline_color_: StyleColor::CurrentColor(),
            internal_visited_text_decoration_color_: StyleColor::CurrentColor(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:356-365
impl Clone for StyleVisitedData {
    fn clone(&self) -> Self {
        Self {
            internal_visited_column_rule_color_: self.internal_visited_column_rule_color_.clone(),
            internal_visited_background_color_: self.internal_visited_background_color_.clone(),
            internal_visited_border_bottom_color_: self
                .internal_visited_border_bottom_color_
                .clone(),
            internal_visited_border_left_color_: self.internal_visited_border_left_color_.clone(),
            internal_visited_border_right_color_: self.internal_visited_border_right_color_.clone(),
            internal_visited_border_top_color_: self.internal_visited_border_top_color_.clone(),
            internal_visited_outline_color_: self.internal_visited_outline_color_.clone(),
            internal_visited_text_decoration_color_: self
                .internal_visited_text_decoration_color_
                .clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:625-633
impl Default for StyleMisc8Data {
    fn default() -> Self {
        Self {
            callback_selectors_: Vector::default(),
            document_rules_selectors_: Member::default(),
            paint_images_: Member::default(),
            non_inherited_variables_: StyleNonInheritedVariables::default(),
            anchor_center_offset_: None,
            position_area_offsets_: None,
            max_lines_: MaxLinesData::new(0, true),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:635-643
impl Clone for StyleMisc8Data {
    fn clone(&self) -> Self {
        Self {
            callback_selectors_: self.callback_selectors_.clone(),
            document_rules_selectors_: self.document_rules_selectors_.clone(),
            paint_images_: MemberCopyPaintImages(&self.paint_images_),
            non_inherited_variables_: self.non_inherited_variables_.clone(),
            anchor_center_offset_: self.anchor_center_offset_.clone(),
            position_area_offsets_: self.position_area_offsets_.clone(),
            max_lines_: self.max_lines_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1161-1171
impl Default for StyleGeometryData {
    fn default() -> Self {
        Self {
            d_: Member::default(),
            cx_: Length::Fixed(0),
            cy_: Length::Fixed(0),
            path_length_: Length::None(),
            r_: Length::Fixed(0),
            rx_: Length::Auto().clone(),
            ry_: Length::Auto().clone(),
            x_: Length::Fixed(0),
            y_: Length::Fixed(0),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1173-1183
impl Clone for StyleGeometryData {
    fn clone(&self) -> Self {
        Self {
            d_: self.d_.clone(),
            cx_: self.cx_.clone(),
            cy_: self.cy_.clone(),
            path_length_: self.path_length_.clone(),
            r_: self.r_.clone(),
            rx_: self.rx_.clone(),
            ry_: self.ry_.clone(),
            x_: self.x_.clone(),
            y_: self.y_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:455-472
impl Default for StyleMisc4Data {
    fn default() -> Self {
        Self {
            page_: AtomicString::default(),
            row_rule_color_: GapDataList::DefaultGapColorDataList(),
            offset_path_: Member::default(),
            position_try_fallbacks_: Member::default(),
            rotate_: Member::default(),
            offset_distance_: Length::Fixed(0),
            offset_position_: LengthPoint::new(&Length::None(), &Length::None()),
            perspective_origin_: LengthPoint::new(&Length::Percent(50.0), &Length::Percent(50.0)),
            offset_rotate_: StyleOffsetRotation::new(0.0, OffsetRotationType::kAuto),
            perspective_: -1.0,
            row_gap_: None,
            outline_color_: StyleColor::CurrentColor(),
            order_: 0,
            outline_offset_: 0,
            outline_width_: 3,
            reading_order_: 0,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:474-491
impl Clone for StyleMisc4Data {
    fn clone(&self) -> Self {
        Self {
            page_: self.page_.clone(),
            row_rule_color_: self.row_rule_color_.clone(),
            offset_path_: self.offset_path_.clone(),
            position_try_fallbacks_: self.position_try_fallbacks_.clone(),
            rotate_: self.rotate_.clone(),
            offset_distance_: self.offset_distance_.clone(),
            offset_position_: self.offset_position_.clone(),
            perspective_origin_: self.perspective_origin_.clone(),
            offset_rotate_: self.offset_rotate_,
            perspective_: self.perspective_,
            row_gap_: self.row_gap_.clone(),
            outline_color_: self.outline_color_.clone(),
            order_: self.order_,
            outline_offset_: self.outline_offset_,
            outline_width_: self.outline_width_,
            reading_order_: self.reading_order_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:493-510
impl Default for StyleMisc5Data {
    fn default() -> Self {
        Self {
            row_rule_style_: GapDataList::DefaultGapStyleDataList(),
            row_rule_width_: GapDataList::DefaultGapWidthDataList(),
            scale_: Member::default(),
            scroll_marker_group_: Member::default(),
            row_rule_inset_cap_end_: Length::Fixed(0),
            row_rule_inset_cap_start_: Length::Fixed(0),
            row_rule_inset_junction_end_: Length::Fixed(0),
            row_rule_inset_junction_start_: Length::Fixed(0),
            scroll_padding_bottom_: Length::default(),
            scroll_padding_left_: Length::default(),
            scroll_padding_right_: Length::default(),
            scroll_padding_top_: Length::default(),
            scroll_margin_bottom_: 0.0,
            scroll_margin_left_: 0.0,
            scroll_margin_right_: 0.0,
            scroll_margin_top_: 0.0,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:512-529
impl Clone for StyleMisc5Data {
    fn clone(&self) -> Self {
        Self {
            row_rule_style_: self.row_rule_style_.clone(),
            row_rule_width_: self.row_rule_width_.clone(),
            scale_: self.scale_.clone(),
            scroll_marker_group_: self.scroll_marker_group_.clone(),
            row_rule_inset_cap_end_: self.row_rule_inset_cap_end_.clone(),
            row_rule_inset_cap_start_: self.row_rule_inset_cap_start_.clone(),
            row_rule_inset_junction_end_: self.row_rule_inset_junction_end_.clone(),
            row_rule_inset_junction_start_: self.row_rule_inset_junction_start_.clone(),
            scroll_padding_bottom_: self.scroll_padding_bottom_.clone(),
            scroll_padding_left_: self.scroll_padding_left_.clone(),
            scroll_padding_right_: self.scroll_padding_right_.clone(),
            scroll_padding_top_: self.scroll_padding_top_.clone(),
            scroll_margin_bottom_: self.scroll_margin_bottom_,
            scroll_margin_left_: self.scroll_margin_left_,
            scroll_margin_right_: self.scroll_margin_right_,
            scroll_margin_top_: self.scroll_margin_top_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:307-324
impl Default for StyleMisc1Data {
    fn default() -> Self {
        Self {
            backdrop_filter_: FilterOperations::new(),
            column_rule_color_: GapDataList::DefaultGapColorDataList(),
            anchor_name_: Member::default(),
            border_shape_: Member::default(),
            box_shadow_: Member::default(),
            clip_path_: Member::default(),
            anchor_scope_: StyleAnchorScope::default(),
            column_rule_inset_cap_end_: Length::Fixed(0),
            column_rule_inset_cap_start_: Length::Fixed(0),
            position_anchor_: StylePositionAnchor::Initial(),
            column_height_: 0.0,
            column_gap_: None,
            align_content_: StyleContentAlignmentData::new(
                ContentPosition::kNormal,
                ContentDistributionType::kDefault,
                OverflowAlignment::kDefault,
            ),
            align_self_: StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kAuto,
                OverflowAlignment::kDefault,
            ),
            column_count_: 1,
            position_area_: PositionArea::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:326-343
impl Clone for StyleMisc1Data {
    fn clone(&self) -> Self {
        Self {
            backdrop_filter_: self.backdrop_filter_.clone(),
            column_rule_color_: self.column_rule_color_.clone(),
            anchor_name_: self.anchor_name_.clone(),
            border_shape_: self.border_shape_.clone(),
            box_shadow_: self.box_shadow_.clone(),
            clip_path_: self.clip_path_.clone(),
            anchor_scope_: self.anchor_scope_.clone(),
            column_rule_inset_cap_end_: self.column_rule_inset_cap_end_.clone(),
            column_rule_inset_cap_start_: self.column_rule_inset_cap_start_.clone(),
            position_anchor_: self.position_anchor_.clone(),
            column_height_: self.column_height_,
            column_gap_: self.column_gap_.clone(),
            align_content_: self.align_content_,
            align_self_: self.align_self_,
            column_count_: self.column_count_,
            position_area_: self.position_area_.clone(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:367-384
impl Default for StyleMisc2Data {
    fn default() -> Self {
        let auto_track = GridTrackSize::from_length(Length::Auto());
        Self {
            filter_: FilterOperations::new(),
            column_rule_style_: GapDataList::DefaultGapStyleDataList(),
            column_rule_width_: GapDataList::DefaultGapWidthDataList(),
            grid_auto_columns_: GridTrackList::from_default_track_size(&auto_track),
            grid_auto_rows_: GridTrackList::from_default_track_size(&auto_track),
            container_name_: Member::default(),
            content_: None,
            flow_tolerance_: FlowTolerance::from_keyword(CSSValueID::kNormal),
            column_rule_inset_junction_end_: Length::Fixed(0),
            column_rule_inset_junction_start_: Length::Fixed(0),
            flex_basis_: Length::Auto().clone(),
            column_width_: 0.0,
            flex_grow_: 0.0,
            flex_shrink_: 1.0,
            flex_line_count_: 1,
            flex_wrap_: StyleFlexWrapData::new(FlexWrapMode::kNowrap),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:386-403
impl Clone for StyleMisc2Data {
    fn clone(&self) -> Self {
        Self {
            filter_: self.filter_.clone(),
            column_rule_style_: self.column_rule_style_.clone(),
            column_rule_width_: self.column_rule_width_.clone(),
            grid_auto_columns_: self.grid_auto_columns_.clone(),
            grid_auto_rows_: self.grid_auto_rows_.clone(),
            container_name_: self.container_name_.clone(),
            content_: MemberCopyContentData(&self.content_),
            flow_tolerance_: self.flow_tolerance_.clone(),
            column_rule_inset_junction_end_: self.column_rule_inset_junction_end_.clone(),
            column_rule_inset_junction_start_: self.column_rule_inset_junction_start_.clone(),
            flex_basis_: self.flex_basis_.clone(),
            column_width_: self.column_width_,
            flex_grow_: self.flex_grow_,
            flex_shrink_: self.flex_shrink_,
            flex_line_count_: self.flex_line_count_,
            flex_wrap_: self.flex_wrap_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:405-422
impl Default for StyleMisc3Data {
    fn default() -> Self {
        Self {
            interest_delay_end_: StyleInterestDelay::default(),
            interest_delay_start_: StyleInterestDelay::default(),
            grid_column_end_: GridPosition::default(),
            grid_column_start_: GridPosition::default(),
            grid_row_end_: GridPosition::default(),
            grid_row_start_: GridPosition::default(),
            grid_template_areas_: Member::default(),
            grid_template_columns_: Member::default(),
            grid_template_rows_: Member::default(),
            object_view_box_: None,
            object_position_: LengthPoint::new(&Length::Percent(50.0), &Length::Percent(50.0)),
            offset_anchor_: LengthPoint::new(Length::Auto(), Length::Auto()),
            initial_letter_: Default::default(),
            grid_lanes_direction_: GridLanesDirection::default(),
            justify_items_: StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kLegacy,
                OverflowAlignment::kDefault,
            ),
            justify_self_: StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kAuto,
                OverflowAlignment::kDefault,
            ),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:424-441
impl Clone for StyleMisc3Data {
    fn clone(&self) -> Self {
        Self {
            interest_delay_end_: self.interest_delay_end_,
            interest_delay_start_: self.interest_delay_start_,
            grid_column_end_: self.grid_column_end_.clone(),
            grid_column_start_: self.grid_column_start_.clone(),
            grid_row_end_: self.grid_row_end_.clone(),
            grid_row_start_: self.grid_row_start_.clone(),
            grid_template_areas_: self.grid_template_areas_.clone(),
            grid_template_columns_: self.grid_template_columns_.clone(),
            grid_template_rows_: self.grid_template_rows_.clone(),
            object_view_box_: self.object_view_box_.clone(),
            object_position_: self.object_position_.clone(),
            offset_anchor_: self.offset_anchor_.clone(),
            initial_letter_: self.initial_letter_.clone(),
            grid_lanes_direction_: self.grid_lanes_direction_,
            justify_items_: self.justify_items_,
            justify_self_: self.justify_self_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:587-604
impl Default for StyleMisc7Data {
    fn default() -> Self {
        Self {
            display_layout_custom_name_: g_null_atom.clone(),
            display_layout_custom_parent_name_: g_null_atom.clone(),
            custom_highlight_names_: None,
            counter_directives_: None,
            counter_increment_list_: None,
            counter_reset_list_: None,
            counter_set_list_: None,
            mask_: FillLayer::new(EFillLayerType::kMask, true),
            box_reflect_: Member::default(),
            animations_: Member::default(),
            transitions_: Member::default(),
            mask_box_image_: NinePieceImage::MaskDefaults(),
            page_size_: gfx::SizeF::default(),
            unconditional_scrollbar_size_: gfx::Size::default(),
            webkit_line_clamp_: 0,
            box_ordinal_group_: 1,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:606-623
impl Clone for StyleMisc7Data {
    fn clone(&self) -> Self {
        Self {
            display_layout_custom_name_: self.display_layout_custom_name_.clone(),
            display_layout_custom_parent_name_: self.display_layout_custom_parent_name_.clone(),
            custom_highlight_names_: self.custom_highlight_names_.clone(),
            counter_directives_: self.counter_directives_.clone(),
            counter_increment_list_: self.counter_increment_list_.clone(),
            counter_reset_list_: self.counter_reset_list_.clone(),
            counter_set_list_: self.counter_set_list_.clone(),
            mask_: self.mask_.clone(),
            box_reflect_: self.box_reflect_.clone(),
            animations_: self.animations_.clone(),
            transitions_: self.transitions_.clone(),
            mask_box_image_: self.mask_box_image_.clone(),
            page_size_: self.page_size_,
            unconditional_scrollbar_size_: self.unconditional_scrollbar_size_,
            webkit_line_clamp_: self.webkit_line_clamp_,
            box_ordinal_group_: self.box_ordinal_group_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1009-1019
impl Default for StyleStrokeData {
    fn default() -> Self {
        Self {
            stroke_dash_array_: Member::default(),
            internal_visited_stroke_paint_: SVGPaint::default(),
            stroke_paint_: SVGPaint::CreateInitial(),
            stroke_dash_offset_: Length::Fixed(0),
            stroke_width_: UnzoomedLength::new(&Length::Fixed(1)),
            stroke_miter_limit_: 4.0,
            stroke_opacity_: 1.0,
            style_bits_: (LineCap::kButtCap as u32 & 0b11)
                | ((LineJoin::kMiterJoin as u32 & 0b11) << 2),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1021-1031
impl Clone for StyleStrokeData {
    fn clone(&self) -> Self {
        Self {
            stroke_dash_array_: self.stroke_dash_array_.clone(),
            internal_visited_stroke_paint_: self.internal_visited_stroke_paint_.clone(),
            stroke_paint_: self.stroke_paint_.clone(),
            stroke_dash_offset_: self.stroke_dash_offset_.clone(),
            stroke_width_: self.stroke_width_.clone(),
            stroke_miter_limit_: self.stroke_miter_limit_,
            stroke_opacity_: self.stroke_opacity_,
            style_bits_: self.style_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1145-1151
impl Default for StyleSvgmiscData {
    fn default() -> Self {
        Self {
            baseline_shift_: Length::Fixed(0),
            flood_opacity_: 1.0,
            flood_color_: StyleColor::from_color(Color::kBlack),
            lighting_color_: StyleColor::from_color(Color::kWhite),
            baseline_shift_type_bits_: EBaselineShiftType::kLength as u32,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1153-1159
impl Clone for StyleSvgmiscData {
    fn clone(&self) -> Self {
        Self {
            baseline_shift_: self.baseline_shift_.clone(),
            flood_opacity_: self.flood_opacity_,
            flood_color_: self.flood_color_.clone(),
            lighting_color_: self.lighting_color_.clone(),
            baseline_shift_type_bits_: self.baseline_shift_type_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:975-984
impl Default for StyleVisualData {
    fn default() -> Self {
        Self {
            base_text_decoration_data_: Member::default(),
            clip_: LengthBox::default(),
            zoom_: 1.0,
            // TextDecorationLine::kNone and EFieldSizing::kFixed are both zero.
            visual_bits_: 1 << 7,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:986-995
impl Clone for StyleVisualData {
    fn clone(&self) -> Self {
        Self {
            base_text_decoration_data_: self.base_text_decoration_data_.clone(),
            clip_: self.clip_.clone(),
            zoom_: self.zoom_,
            visual_bits_: self.visual_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:531-548
impl Default for StyleMisc6Data {
    fn default() -> Self {
        Self {
            view_transition_group_: StyleViewTransitionGroup::Normal(),
            text_overflow_: TextOverflowData::from_type(TextOverflowType::kClip),
            shape_outside_: Member::default(),
            translate_: Member::default(),
            view_transition_class_: Member::default(),
            view_transition_name_: Member::default(),
            trigger_scope_: StyleTriggerScope::default(),
            shape_margin_: Length::Fixed(0),
            text_decoration_inset_: TextDecorationInset::new(&Length::Fixed(0), &Length::Fixed(0)),
            text_decoration_thickness_: TextDecorationThickness::new(&Length::Auto()),
            shape_image_threshold_: 0.0,
            box_flex_: 0.0,
            text_decoration_color_: StyleColor::CurrentColor(),
            scroll_snap_align_: Default::default(),
            scroll_snap_type_: Default::default(),
            touch_action_bits_: TouchAction::kAuto.bits() as u32,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:550-567
impl Clone for StyleMisc6Data {
    fn clone(&self) -> Self {
        Self {
            view_transition_group_: self.view_transition_group_.clone(),
            text_overflow_: self.text_overflow_.clone(),
            shape_outside_: self.shape_outside_.clone(),
            translate_: self.translate_.clone(),
            view_transition_class_: self.view_transition_class_.clone(),
            view_transition_name_: self.view_transition_name_.clone(),
            trigger_scope_: self.trigger_scope_.clone(),
            shape_margin_: self.shape_margin_.clone(),
            text_decoration_inset_: self.text_decoration_inset_.clone(),
            text_decoration_thickness_: self.text_decoration_thickness_.clone(),
            shape_image_threshold_: self.shape_image_threshold_,
            box_flex_: self.box_flex_,
            text_decoration_color_: self.text_decoration_color_.clone(),
            scroll_snap_align_: self.scroll_snap_align_.clone(),
            scroll_snap_type_: self.scroll_snap_type_.clone(),
            touch_action_bits_: self.touch_action_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:933-952
impl Default for StyleInheritedData {
    fn default() -> Self {
        Self {
            inherited_forced_colors_data_: Member::from_ptr(
                StyleInheritedForcedColorsData::Create(),
            ),
            inherited_visited_data_: Member::from_ptr(StyleInheritedVisitedData::Create()),
            highlight_data_data_: Member::from_ptr(StyleHighlightDataData::Create()),
            font_: Member::from_ptr(MakeGarbageCollected(Font::default())),
            container_font_: Member::default(),
            inherited_variables_: StyleInheritedVariables::default(),
            letter_spacing_: Length::Fixed(0),
            line_height_: Length::Auto().clone(),
            word_spacing_: Length::Fixed(0),
            internal_visited_color_: StyleColor::from_color(Color::kBlack),
            color_: StyleColor::from_color(Color::kBlack),
            horizontal_border_spacing_: 0,
            vertical_border_spacing_: 0,
            inherited_bits_: (ECursor::kAuto as u32 & 0x3f)
                | ((ETextTransform::kNone.bits() & 0x3f) << 6)
                | (1 << 12)
                | (1 << 14),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:954-973
impl Clone for StyleInheritedData {
    fn clone(&self) -> Self {
        Self {
            inherited_forced_colors_data_: self.inherited_forced_colors_data_.clone(),
            inherited_visited_data_: self.inherited_visited_data_.clone(),
            highlight_data_data_: self.highlight_data_data_.clone(),
            font_: self.font_.clone(),
            container_font_: self.container_font_.clone(),
            inherited_variables_: self.inherited_variables_.clone(),
            letter_spacing_: self.letter_spacing_.clone(),
            line_height_: self.line_height_.clone(),
            word_spacing_: self.word_spacing_.clone(),
            internal_visited_color_: self.internal_visited_color_.clone(),
            color_: self.color_.clone(),
            horizontal_border_spacing_: self.horizontal_border_spacing_,
            vertical_border_spacing_: self.vertical_border_spacing_,
            inherited_bits_: self.inherited_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1045-1059
impl Default for StyleSvginheritedData {
    fn default() -> Self {
        Self {
            fill_data_: Member::from_ptr(StyleFillData::Create()),
            stroke_data_: Member::from_ptr(StyleStrokeData::Create()),
            resources_data_: Member::from_ptr(StyleResourcesData::Create()),
            svg_inherited_bits_: (EDominantBaseline::kAuto as u32 & 0xf)
                | ((EDominantBaseline::kAuto as u32 & 0xf) << 4)
                | ((EPaintOrder::kPaintOrderNormal as u32 & 0x7) << 8)
                | ((EColorInterpolation::kSRGB as u32 & 0x3) << 11)
                | ((EColorInterpolation::kLinearrgb as u32 & 0x3) << 13)
                | ((EColorRendering::kAuto as u32 & 0x3) << 15)
                | ((EShapeRendering::kAuto as u32 & 0x3) << 17)
                | ((ETextAnchor::kStart as u32 & 0x3) << 19)
                | ((WindRule::RULE_NONZERO as u32 & 1) << 21)
                | ((WindRule::RULE_NONZERO as u32 & 1) << 22),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1061-1075
impl Clone for StyleSvginheritedData {
    fn clone(&self) -> Self {
        Self {
            fill_data_: self.fill_data_.clone(),
            stroke_data_: self.stroke_data_.clone(),
            resources_data_: self.resources_data_.clone(),
            svg_inherited_bits_: self.svg_inherited_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1195-1206
impl Default for StyleSVGData {
    fn default() -> Self {
        Self {
            svgmisc_data_: Member::from_ptr(StyleSvgmiscData::Create()),
            geometry_data_: Member::from_ptr(StyleGeometryData::Create()),
            stop_data_: Member::from_ptr(StyleStopData::Create()),
            transform_: EmptyTransformOperations.into(),
            transform_origin_: TransformOrigin::new(
                &Length::Percent(50.0),
                &Length::Percent(50.0),
                0.0,
            ),
            opacity_: 1.0,
            svg_bits_: (EAlignmentBaseline::kAuto as u32 & 0xf)
                | ((EBufferedRendering::kAuto as u32 & 0x3) << 4)
                | ((EMaskType::kLuminance as u32 & 1) << 6)
                | ((EVectorEffect::kNone as u32 & 1) << 7),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1208-1219
impl Clone for StyleSVGData {
    fn clone(&self) -> Self {
        Self {
            svgmisc_data_: self.svgmisc_data_.clone(),
            geometry_data_: self.geometry_data_.clone(),
            stop_data_: self.stop_data_.clone(),
            transform_: self.transform_.clone(),
            transform_origin_: self.transform_origin_.clone(),
            opacity_: self.opacity_,
            svg_bits_: self.svg_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1221-1247
impl Default for StyleSurroundData {
    fn default() -> Self {
        let zero_radius = || LengthSize::new(&Length::Fixed(0), &Length::Fixed(0));
        Self {
            corner_bottom_left_shape_: Superellipse::Round(),
            corner_bottom_right_shape_: Superellipse::Round(),
            corner_top_left_shape_: Superellipse::Round(),
            corner_top_right_shape_: Superellipse::Round(),
            border_image_: NinePieceImage::default(),
            bottom_: Length::default(),
            left_: Length::default(),
            right_: Length::default(),
            top_: Length::default(),
            border_bottom_left_radius_: zero_radius(),
            border_bottom_right_radius_: zero_radius(),
            border_top_left_radius_: zero_radius(),
            border_top_right_radius_: zero_radius(),
            aspect_ratio_: StyleAspectRatio::new(EAspectRatioType::kAuto, gfx::SizeF::default()),
            contain_intrinsic_height_: StyleIntrinsicLength::default(),
            contain_intrinsic_width_: StyleIntrinsicLength::default(),
            border_bottom_color_: StyleColor::CurrentColor(),
            border_left_color_: StyleColor::CurrentColor(),
            border_right_color_: StyleColor::CurrentColor(),
            border_top_color_: StyleColor::CurrentColor(),
            surround_bits_: EFrameSizing::kAuto as u32 & 0x7,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1249-1275
impl Clone for StyleSurroundData {
    fn clone(&self) -> Self {
        Self {
            corner_bottom_left_shape_: self.corner_bottom_left_shape_,
            corner_bottom_right_shape_: self.corner_bottom_right_shape_,
            corner_top_left_shape_: self.corner_top_left_shape_,
            corner_top_right_shape_: self.corner_top_right_shape_,
            border_image_: self.border_image_.clone(),
            bottom_: self.bottom_.clone(),
            left_: self.left_.clone(),
            right_: self.right_.clone(),
            top_: self.top_.clone(),
            border_bottom_left_radius_: self.border_bottom_left_radius_.clone(),
            border_bottom_right_radius_: self.border_bottom_right_radius_.clone(),
            border_top_left_radius_: self.border_top_left_radius_.clone(),
            border_top_right_radius_: self.border_top_right_radius_.clone(),
            aspect_ratio_: self.aspect_ratio_.clone(),
            contain_intrinsic_height_: self.contain_intrinsic_height_.clone(),
            contain_intrinsic_width_: self.contain_intrinsic_width_.clone(),
            border_bottom_color_: self.border_bottom_color_.clone(),
            border_left_color_: self.border_left_color_.clone(),
            border_right_color_: self.border_right_color_.clone(),
            border_top_color_: self.border_top_color_.clone(),
            surround_bits_: self.surround_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1077-1109
impl Default for StyleBoxData {
    fn default() -> Self {
        Self {
            height_: Length::default(),
            margin_bottom_: Length::Fixed(0),
            margin_left_: Length::Fixed(0),
            margin_right_: Length::Fixed(0),
            margin_top_: Length::Fixed(0),
            max_height_: Length::None(),
            max_width_: Length::None(),
            min_height_: Length::default(),
            min_width_: Length::default(),
            padding_bottom_: Length::Fixed(0),
            padding_left_: Length::Fixed(0),
            padding_right_: Length::Fixed(0),
            padding_top_: Length::Fixed(0),
            width_: Length::default(),
            vertical_align_length_: Length::default(),
            justify_content_: StyleContentAlignmentData::new(
                ContentPosition::kNormal,
                ContentDistributionType::kDefault,
                OverflowAlignment::kDefault,
            ),
            align_items_: StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kNormal,
                OverflowAlignment::kDefault,
            ),
            border_bottom_width_: 3,
            border_left_width_: 3,
            border_right_width_: 3,
            border_top_width_: 3,
            z_index_: 0,
            overflow_clip_margin_: None,
            box_bits_: (EBorderStyle::kNone as u32 & 0xf)
                | ((EBorderStyle::kNone as u32 & 0xf) << 4)
                | ((EBorderStyle::kNone as u32 & 0xf) << 8)
                | ((EBorderStyle::kNone as u32 & 0xf) << 12)
                | ((EBaselineSource::kAuto as u32 & 0x3) << 16)
                | ((ETextBoxTrim::kNone as u32 & 0x3) << 18)
                | ((EBoxDecorationBreak::kSlice as u32 & 1) << 20)
                | (1 << 21),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:1111-1143
impl Clone for StyleBoxData {
    fn clone(&self) -> Self {
        Self {
            height_: self.height_.clone(),
            margin_bottom_: self.margin_bottom_.clone(),
            margin_left_: self.margin_left_.clone(),
            margin_right_: self.margin_right_.clone(),
            margin_top_: self.margin_top_.clone(),
            max_height_: self.max_height_.clone(),
            max_width_: self.max_width_.clone(),
            min_height_: self.min_height_.clone(),
            min_width_: self.min_width_.clone(),
            padding_bottom_: self.padding_bottom_.clone(),
            padding_left_: self.padding_left_.clone(),
            padding_right_: self.padding_right_.clone(),
            padding_top_: self.padding_top_.clone(),
            width_: self.width_.clone(),
            vertical_align_length_: self.vertical_align_length_.clone(),
            justify_content_: self.justify_content_,
            align_items_: self.align_items_,
            border_bottom_width_: self.border_bottom_width_,
            border_left_width_: self.border_left_width_,
            border_right_width_: self.border_right_width_,
            border_top_width_: self.border_top_width_,
            z_index_: self.z_index_,
            overflow_clip_margin_: self.overflow_clip_margin_.clone(),
            box_bits_: self.box_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:215-259
impl Default for StyleMiscInheritedData {
    fn default() -> Self {
        // C++ initializes the two member handles before the bitfield values.
        let misc_inherited_1_data_ = Member::from_ptr(StyleMiscInherited1Data::Create());
        let misc_inherited_2_data_ = Member::from_ptr(StyleMiscInherited2Data::Create());
        let mut bits = [0u32; 3];
        // cpp: layoutng_style/style/computed_style_base_data.cc:218
        bits[0] |= ((TextBoxEdge::default().to_bits()) & 63u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:219
        bits[0] |= ((TextUnderlinePosition::kAuto.value()) & 15u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:220
        bits[0] |= (HangingPunctuation::kNone.bits() & 7u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:221
        bits[0] |= ((LineBreak::kAuto as u32) & 7u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:222
        bits[0] |= ((ESpeak::kNormal as u32) & 7u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:223
        bits[0] |= ((ETextAlignLast::kAuto as u32) & 7u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:224
        bits[0] |= (TextDecorationSkipSpaces::kNone.bits() & 7u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:225
        bits[0] |= ((TextEmphasisMark::kNone as u32) & 7u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:226
        bits[0] |=
            ((ComputedStyleInitialValues::InitialTextEmphasisPosition() as u32) & 7u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:227
        bits[0] |= ((EBlockEllipsis::kNoEllipsis as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:228
        bits[1] |= ((EUserSelect::kAuto as u32) & 7u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:229
        bits[1] |= ((EWordBreak::kNormal as u32) & 7u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:230
        bits[1] |= ((EDraggableRegionMode::kNone as u32) & 3u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:231
        bits[1] |= ((EForcedColorAdjust::kAuto as u32) & 3u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:232
        bits[1] |= ((Hyphens::kManual as u32) & 3u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:233
        bits[1] |= ((ImageAnimationEnum::kNormal as u32) & 3u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:234
        bits[1] |= ((EImageRendering::kAuto as u32) & 3u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:235
        bits[1] |= ((EOverflowWrap::kNormal as u32) & 3u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:236
        bits[1] |= ((ERubyAlign::kSpaceAround as u32) & 3u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:237
        bits[1] |= ((ERubyOverhang::kAuto as u32) & 3u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:238
        bits[1] |= ((ETextDecorationSkipInk::kAuto as u32) & 3u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:239
        bits[1] |= ((TextIndentFlags::kDefault.bits() as u32) & 3u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:240
        bits[1] |= ((TextJustify::kAuto as u32) & 3u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:241
        bits[1] |= ((ETextOrientation::kMixed as u32) & 3u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:242
        bits[1] |= ((ETextSecurity::kNone as u32) & 3u32) << 30;
        // cpp: layoutng_style/style/computed_style_base_data.cc:243
        bits[2] |= ((EUserModify::kReadOnly as u32) & 3u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:244
        bits[2] |= ((EEmptyCells::kShow as u32) & 1u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:245
        bits[2] |= ((false as u32) & 1u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:246
        bits[2] |= ((RespectImageOrientationEnum::kRespectImageOrientation as u32) & 1u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:247
        bits[2] |= ((false as u32) & 1u32) << 5;
        // cpp: layoutng_style/style/computed_style_base_data.cc:248
        bits[2] |= ((EInteractivity::kAuto as u32) & 1u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:249
        bits[2] |= ((EInterpolateSize::kNumericOnly as u32) & 1u32) << 7;
        // cpp: layoutng_style/style/computed_style_base_data.cc:250
        bits[2] |= ((false as u32) & 1u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:251
        bits[2] |= ((EMathShift::kNormal as u32) & 1u32) << 9;
        // cpp: layoutng_style/style/computed_style_base_data.cc:252
        bits[2] |= ((EMathStyle::kNormal as u32) & 1u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:253
        bits[2] |= ((RubyPosition::kOver as u32) & 1u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:254
        bits[2] |= ((false as u32) & 1u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:255
        bits[2] |= ((false as u32) & 1u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:256
        bits[2] |= ((ETextAutospace::kNoAutospace as u32) & 1u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:257
        bits[2] |= ((ETextCombine::kNone as u32) & 1u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:258
        bits[2] |= ((TextEmphasisFill::kFilled as u32) & 1u32) << 16;
        Self {
            misc_inherited_1_data_,
            misc_inherited_2_data_,
            misc_inherited_bits_: bits,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:261-305
impl Clone for StyleMiscInheritedData {
    fn clone(&self) -> Self {
        Self {
            misc_inherited_1_data_: self.misc_inherited_1_data_.clone(),
            misc_inherited_2_data_: self.misc_inherited_2_data_.clone(),
            misc_inherited_bits_: self.misc_inherited_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:667-782
impl Default for StyleMiscData {
    fn default() -> Self {
        // Initialize GC members before evaluating bitfield defaults, as in C++.
        let misc_1_data_ = Member::from_ptr(StyleMisc1Data::Create());
        let visited_data_ = Member::from_ptr(StyleVisitedData::Create());
        let misc_2_data_ = Member::from_ptr(StyleMisc2Data::Create());
        let misc_3_data_ = Member::from_ptr(StyleMisc3Data::Create());
        let forced_colors_data_ = Member::from_ptr(StyleForcedColorsData::Create());
        let misc_4_data_ = Member::from_ptr(StyleMisc4Data::Create());
        let misc_5_data_ = Member::from_ptr(StyleMisc5Data::Create());
        let misc_6_data_ = Member::from_ptr(StyleMisc6Data::Create());
        let timeline_data_ = Member::from_ptr(StyleTimelineData::Create());
        let misc_7_data_ = Member::from_ptr(StyleMisc7Data::Create());
        let misc_8_data_ = Member::from_ptr(StyleMisc8Data::Create());
        let math_data_ = Member::from_ptr(StyleMathData::Create());
        let will_change_ = Member::default();
        let mut bits = [0u32; 6];
        // cpp: layoutng_style/style/computed_style_base_data.cc:681
        bits[0] |= ((AppearanceValue::kNone as u32) & 31u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:682
        bits[0] |= ((BlendMode::kNormal as u32) & 31u32) << 5;
        // cpp: layoutng_style/style/computed_style_base_data.cc:683
        bits[0] |= ((Containment::kContainsNone.value() as u32) & 31u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:684
        bits[0] |= ((AppearanceValue::kNone as u32) & 31u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:685
        bits[0] |= ((EBreakBetween::kAuto as u32) & 15u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:686
        bits[0] |= ((EBreakBetween::kAuto as u32) & 15u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:687
        bits[0] |= ((EContainerType::kContainerTypeNormal.value() as u32) & 15u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:688
        bits[1] |= ((GridAutoFlow::kAutoFlowRow as u32) & 15u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:689
        bits[1] |= ((EMarginTrim::kMarginTrimNone.value() as u32) & 15u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:690
        bits[1] |= ((EBorderStyle::kNone as u32) & 15u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:691
        bits[1] |= ((ScrollbarGutter::kScrollbarGutterAuto.value() as u32) & 15u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:692
        bits[1] |= ((EBoxAlignment::kStretch as u32) & 7u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:693
        bits[1] |= ((EObjectFit::kFill as u32) & 7u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:694
        bits[1] |= ((EPositionTryOrder::kNormal as u32) & 7u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:695
        bits[1] |= ((PositionVisibility::kAnchorsVisible.value() as u32) & 7u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:696
        bits[1] |= ((EReadingFlow::kNormal as u32) & 7u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:697
        bits[1] |= ((false as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:698
        bits[2] |= ((EResize::kNone as u32) & 7u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:699
        bits[2] |= ((ETextDecorationStyle::kSolid as u32) & 7u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:700
        bits[2] |= ((EBoxPack::kStart as u32) & 3u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:701
        bits[2] |= ((EBreakInside::kAuto as u32) & 3u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:702
        bits[2] |= ((RuleBreak::kNormal as u32) & 3u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:703
        bits[2] |= ((RuleVisibilityItems::kNormal as u32) & 3u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:704
        bits[2] |= ((EColumnWrap::kAuto as u32) & 3u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:705
        bits[2] |= ((EContinue::kNormal as u32) & 3u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:706
        bits[2] |= ((EFlexDirection::kRow as u32) & 3u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:707
        bits[2] |= ((EOverscrollBehavior::kAuto as u32) & 3u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:708
        bits[2] |= ((EOverscrollBehavior::kAuto as u32) & 3u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:709
        bits[2] |= ((EOverscrollContainerType::kAuto as u32) & 3u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:710
        bits[2] |= ((EPageMarginSafety::kNone as u32) & 3u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:711
        bits[2] |= ((PageOrientation::kUpright as u32) & 3u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:712
        bits[2] |= ((PageSizeType::kAuto as u32) & 3u32) << 30;
        // cpp: layoutng_style/style/computed_style_base_data.cc:713
        bits[3] |= ((RuleBreak::kNormal as u32) & 3u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:714
        bits[3] |= ((RuleVisibilityItems::kNormal as u32) & 3u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:715
        bits[3] |= ((mojom::blink::ScrollBehavior::kAuto.value() as u32) & 3u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:716
        bits[3] |= ((EScrollbarWidth::kAuto as u32) & 3u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:717
        bits[3] |= ((EUserDrag::kAuto as u32) & 3u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:718
        bits[3] |= ((false as u32) & 1u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:719
        bits[3] |= ((false as u32) & 1u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:720
        bits[3] |= ((false as u32) & 1u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:721
        bits[3] |= ((EBackfaceVisibility::kVisible as u32) & 1u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:722
        bits[3] |= ((EBoxOrient::kHorizontal as u32) & 1u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:723
        bits[3] |= ((false as u32) & 1u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:724
        bits[3] |= ((EColumnFill::kBalance as u32) & 1u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:725
        bits[3] |= ((EColumnSpan::kNone as u32) & 1u32) << 17;
        // cpp: layoutng_style/style/computed_style_base_data.cc:726
        bits[3] |= ((false as u32) & 1u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:727
        bits[3] |= ((false as u32) & 1u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:728
        bits[3] |= ((false as u32) & 1u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:729
        bits[3] |= ((false as u32) & 1u32) << 21;
        // cpp: layoutng_style/style/computed_style_base_data.cc:730
        bits[3] |= ((false as u32) & 1u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:731
        bits[3] |= ((false as u32) & 1u32) << 23;
        // cpp: layoutng_style/style/computed_style_base_data.cc:732
        bits[3] |= ((false as u32) & 1u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:733
        bits[3] |= ((EGridLanesPack::kNormal as u32) & 1u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:734
        bits[3] |= ((true as u32) & 1u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:735
        bits[3] |= ((true as u32) & 1u32) << 27;
        // cpp: layoutng_style/style/computed_style_base_data.cc:736
        bits[3] |= ((true as u32) & 1u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:737
        bits[3] |= ((false as u32) & 1u32) << 29;
        // cpp: layoutng_style/style/computed_style_base_data.cc:738
        bits[3] |= ((false as u32) & 1u32) << 30;
        // cpp: layoutng_style/style/computed_style_base_data.cc:739
        bits[3] |= ((false as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:740
        bits[4] |= ((false as u32) & 1u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:741
        bits[4] |= ((false as u32) & 1u32) << 1;
        // cpp: layoutng_style/style/computed_style_base_data.cc:742
        bits[4] |= ((false as u32) & 1u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:743
        bits[4] |= ((false as u32) & 1u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:744
        bits[4] |= ((false as u32) & 1u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:745
        bits[4] |= ((false as u32) & 1u32) << 5;
        // cpp: layoutng_style/style/computed_style_base_data.cc:746
        bits[4] |= ((false as u32) & 1u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:747
        bits[4] |= ((false as u32) & 1u32) << 7;
        // cpp: layoutng_style/style/computed_style_base_data.cc:748
        bits[4] |= ((false as u32) & 1u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:749
        bits[4] |= ((false as u32) & 1u32) << 9;
        // cpp: layoutng_style/style/computed_style_base_data.cc:750
        bits[4] |= ((false as u32) & 1u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:751
        bits[4] |= ((false as u32) & 1u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:752
        bits[4] |= ((false as u32) & 1u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:753
        bits[4] |= ((false as u32) & 1u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:754
        bits[4] |= ((false as u32) & 1u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:755
        bits[4] |= ((false as u32) & 1u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:756
        bits[4] |= ((EInternalOverscrollContainer::kNone as u32) & 1u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:757
        bits[4] |= ((EInternalOverscrollPosition::kNone as u32) & 1u32) << 17;
        // cpp: layoutng_style/style/computed_style_base_data.cc:758
        bits[4] |= ((EInternalUnbounded::kNone as u32) & 1u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:759
        bits[4] |= ((false as u32) & 1u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:760
        bits[4] |= ((false as u32) & 1u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:761
        bits[4] |= ((false as u32) & 1u32) << 21;
        // cpp: layoutng_style/style/computed_style_base_data.cc:762
        bits[4] |= ((false as u32) & 1u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:763
        bits[4] |= ((false as u32) & 1u32) << 23;
        // cpp: layoutng_style/style/computed_style_base_data.cc:764
        bits[4] |= ((false as u32) & 1u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:765
        bits[4] |= ((false as u32) & 1u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:766
        bits[4] |= ((false as u32) & 1u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:767
        bits[4] |= ((false as u32) & 1u32) << 27;
        // cpp: layoutng_style/style/computed_style_base_data.cc:768
        bits[4] |= ((false as u32) & 1u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:769
        bits[4] |= ((false as u32) & 1u32) << 29;
        // cpp: layoutng_style/style/computed_style_base_data.cc:770
        bits[4] |= ((EIsolation::kAuto as u32) & 1u32) << 30;
        // cpp: layoutng_style/style/computed_style_base_data.cc:771
        bits[4] |= ((EBlockEllipsis::kNoEllipsis as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:772
        bits[5] |= ((EMaxContentSizing::kAuto as u32) & 1u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:773
        bits[5] |= ((false as u32) & 1u32) << 1;
        // cpp: layoutng_style/style/computed_style_base_data.cc:774
        bits[5] |= ((EOverlay::kNone as u32) & 1u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:775
        bits[5] |= ((false as u32) & 1u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:776
        bits[5] |= ((ERuleOverlap::kRowOverColumn as u32) & 1u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:777
        bits[5] |= ((EScrollAxisLock::kAuto as u32) & 1u32) << 5;
        // cpp: layoutng_style/style/computed_style_base_data.cc:778
        bits[5] |= ((EScrollInitialTarget::kNone as u32) & 1u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:779
        bits[5] |= ((EScrollTargetGroup::kNone as u32) & 1u32) << 7;
        // cpp: layoutng_style/style/computed_style_base_data.cc:780
        bits[5] |= ((ETransformStyle3D::kFlat as u32) & 1u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:781
        bits[5] |= ((EViewTransitionScope::kNone as u32) & 1u32) << 9;
        Self {
            misc_1_data_,
            visited_data_,
            misc_2_data_,
            misc_3_data_,
            forced_colors_data_,
            misc_4_data_,
            misc_5_data_,
            misc_6_data_,
            timeline_data_,
            misc_7_data_,
            misc_8_data_,
            math_data_,
            will_change_,
            misc_bits_: bits,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:784-899
impl Clone for StyleMiscData {
    fn clone(&self) -> Self {
        Self {
            misc_1_data_: self.misc_1_data_.clone(),
            visited_data_: self.visited_data_.clone(),
            misc_2_data_: self.misc_2_data_.clone(),
            misc_3_data_: self.misc_3_data_.clone(),
            forced_colors_data_: self.forced_colors_data_.clone(),
            misc_4_data_: self.misc_4_data_.clone(),
            misc_5_data_: self.misc_5_data_.clone(),
            misc_6_data_: self.misc_6_data_.clone(),
            timeline_data_: self.timeline_data_.clone(),
            misc_7_data_: self.misc_7_data_.clone(),
            misc_8_data_: self.misc_8_data_.clone(),
            math_data_: self.math_data_.clone(),
            will_change_: self.will_change_.clone(),
            misc_bits_: self.misc_bits_,
        }
    }
}

// cpp: layoutng_style/style/computed_style_base_data.cc:30-120
impl Default for ComputedStyleBaseData {
    fn default() -> Self {
        let mut bits = [0u32; 5];
        // cpp: layoutng_style/style/computed_style_base_data.cc:31
        bits[0] |= ((PseudoId::kPseudoIdNone.value() as u32) & 67108863u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:32
        bits[0] |= ((EDisplay::kInline as u32) & 63u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:33
        bits[1] |= ((PseudoId::kPseudoIdNone.value() as u32) & 63u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:34
        bits[1] |= ((EVerticalAlign::kBaseline as u32) & 15u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:35
        bits[1] |= ((EClear::kNone as u32) & 7u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:36
        bits[1] |= ((EFloat::kNone as u32) & 7u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:37
        bits[1] |= ((EOverflow::kVisible as u32) & 7u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:38
        bits[1] |= ((EOverflow::kVisible as u32) & 7u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:39
        bits[1] |= ((EPosition::kStatic as u32) & 7u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:40
        bits[1] |= ((ETransformBox::kViewBox as u32) & 7u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:41
        bits[1] |= ((UnicodeBidi::kNormal as u32) & 7u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:42
        bits[1] |= ((false as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:43
        bits[2] |= ((EContentVisibility::kVisible as u32) & 3u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:44
        bits[2] |= ((EInlineBlockBaselineEdge::kMarginBox as u32) & 3u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:45
        bits[2] |= ((0 as u32) & 3u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:46
        bits[2] |= ((EOverflowAnchor::kAuto as u32) & 3u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:47
        bits[2] |= ((0 as u32) & 3u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:48
        bits[2] |= ((false as u32) & 1u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:49
        bits[2] |= ((false as u32) & 1u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:50
        bits[2] |= ((false as u32) & 1u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:51
        bits[2] |= ((false as u32) & 1u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:52
        bits[2] |= ((true as u32) & 1u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:53
        bits[2] |= ((EBoxDirection::kNormal as u32) & 1u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:54
        bits[2] |= ((EBoxSizing::kContentBox as u32) & 1u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:55
        bits[2] |= ((true as u32) & 1u32) << 17;
        // cpp: layoutng_style/style/computed_style_base_data.cc:56
        bits[2] |= ((false as u32) & 1u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:57
        bits[2] |= ((true as u32) & 1u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:58
        bits[2] |= ((true as u32) & 1u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:59
        bits[2] |= ((false as u32) & 1u32) << 21;
        // cpp: layoutng_style/style/computed_style_base_data.cc:60
        bits[2] |= ((true as u32) & 1u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:61
        bits[2] |= ((false as u32) & 1u32) << 23;
        // cpp: layoutng_style/style/computed_style_base_data.cc:62
        bits[2] |= ((false as u32) & 1u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:63
        bits[2] |= ((false as u32) & 1u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:64
        bits[2] |= ((false as u32) & 1u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:65
        bits[2] |= ((false as u32) & 1u32) << 27;
        // cpp: layoutng_style/style/computed_style_base_data.cc:66
        bits[2] |= ((false as u32) & 1u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:67
        bits[2] |= ((false as u32) & 1u32) << 29;
        // cpp: layoutng_style/style/computed_style_base_data.cc:68
        bits[2] |= ((false as u32) & 1u32) << 30;
        // cpp: layoutng_style/style/computed_style_base_data.cc:69
        bits[2] |= ((false as u32) & 1u32) << 31;
        // cpp: layoutng_style/style/computed_style_base_data.cc:70
        bits[3] |= ((false as u32) & 1u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:71
        bits[3] |= ((false as u32) & 1u32) << 1;
        // cpp: layoutng_style/style/computed_style_base_data.cc:72
        bits[3] |= ((false as u32) & 1u32) << 2;
        // cpp: layoutng_style/style/computed_style_base_data.cc:73
        bits[3] |= ((false as u32) & 1u32) << 3;
        // cpp: layoutng_style/style/computed_style_base_data.cc:74
        bits[3] |= ((false as u32) & 1u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:75
        bits[3] |= ((false as u32) & 1u32) << 5;
        // cpp: layoutng_style/style/computed_style_base_data.cc:76
        bits[3] |= ((false as u32) & 1u32) << 6;
        // cpp: layoutng_style/style/computed_style_base_data.cc:77
        bits[3] |= ((true as u32) & 1u32) << 7;
        // cpp: layoutng_style/style/computed_style_base_data.cc:78
        bits[3] |= ((true as u32) & 1u32) << 8;
        // cpp: layoutng_style/style/computed_style_base_data.cc:79
        bits[3] |= ((false as u32) & 1u32) << 9;
        // cpp: layoutng_style/style/computed_style_base_data.cc:80
        bits[3] |= ((false as u32) & 1u32) << 10;
        // cpp: layoutng_style/style/computed_style_base_data.cc:81
        bits[3] |= ((true as u32) & 1u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:82
        bits[3] |= ((false as u32) & 1u32) << 12;
        // cpp: layoutng_style/style/computed_style_base_data.cc:83
        bits[3] |= ((false as u32) & 1u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:84
        bits[3] |= ((false as u32) & 1u32) << 14;
        // cpp: layoutng_style/style/computed_style_base_data.cc:85
        bits[3] |= ((false as u32) & 1u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:86
        bits[3] |= ((true as u32) & 1u32) << 16;
        // cpp: layoutng_style/style/computed_style_base_data.cc:87
        bits[3] |= ((false as u32) & 1u32) << 17;
        // cpp: layoutng_style/style/computed_style_base_data.cc:88
        bits[3] |= ((true as u32) & 1u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:89
        bits[3] |= ((EOriginTrialTestProperty::kNormal as u32) & 1u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:90
        bits[3] |= ((true as u32) & 1u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:91
        bits[3] |= ((true as u32) & 1u32) << 21;
        // cpp: layoutng_style/style/computed_style_base_data.cc:92
        bits[3] |= ((EScrollSnapStop::kNormal as u32) & 1u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:93
        bits[3] |= ((false as u32) & 1u32) << 23;
        // cpp: layoutng_style/style/computed_style_base_data.cc:94
        bits[3] |= ((false as u32) & 1u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:95
        bits[3] |= ((ETableLayout::kAuto as u32) & 1u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:96
        bits[3] |= ((true as u32) & 1u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:97
        bits[3] |= ((true as u32) & 1u32) << 27;
        // cpp: layoutng_style/style/computed_style_base_data.cc:98
        bits[3] |= ((EPointerEvents::kAuto as u32) & 15u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:99
        bits[4] |= ((ETextAlign::kStart as u32) & 15u32) << 0;
        // cpp: layoutng_style/style/computed_style_base_data.cc:100
        bits[4] |= ((WritingMode::kHorizontalTb as u32) & 7u32) << 4;
        // cpp: layoutng_style/style/computed_style_base_data.cc:101
        bits[4] |= ((ECaretShape::kAuto as u32) & 3u32) << 7;
        // cpp: layoutng_style/style/computed_style_base_data.cc:102
        bits[4] |= ((EInsideLink::kNotInsideLink as u32) & 3u32) << 9;
        // cpp: layoutng_style/style/computed_style_base_data.cc:103
        bits[4] |= ((TextWrapStyle::kAuto as u32) & 3u32) << 11;
        // cpp: layoutng_style/style/computed_style_base_data.cc:104
        bits[4] |= ((EVisibility::kVisible as u32) & 3u32) << 13;
        // cpp: layoutng_style/style/computed_style_base_data.cc:105
        bits[4] |= ((WhiteSpaceCollapse::kCollapse.bits() as u32) & 3u32) << 15;
        // cpp: layoutng_style/style/computed_style_base_data.cc:106
        bits[4] |= ((EBorderCollapse::kSeparate as u32) & 1u32) << 17;
        // cpp: layoutng_style/style/computed_style_base_data.cc:107
        bits[4] |= ((ECaptionSide::kTop as u32) & 1u32) << 18;
        // cpp: layoutng_style/style/computed_style_base_data.cc:108
        bits[4] |= ((ECaretAnimation::kAuto as u32) & 1u32) << 19;
        // cpp: layoutng_style/style/computed_style_base_data.cc:109
        bits[4] |= ((false as u32) & 1u32) << 20;
        // cpp: layoutng_style/style/computed_style_base_data.cc:110
        bits[4] |= ((false as u32) & 1u32) << 21;
        // cpp: layoutng_style/style/computed_style_base_data.cc:111
        bits[4] |= ((false as u32) & 1u32) << 22;
        // cpp: layoutng_style/style/computed_style_base_data.cc:112
        bits[4] |= ((TextDirection::kLtr as u32) & 1u32) << 23;
        // cpp: layoutng_style/style/computed_style_base_data.cc:113
        bits[4] |= ((false as u32) & 1u32) << 24;
        // cpp: layoutng_style/style/computed_style_base_data.cc:114
        bits[4] |= ((false as u32) & 1u32) << 25;
        // cpp: layoutng_style/style/computed_style_base_data.cc:115
        bits[4] |= ((EListStylePosition::kOutside as u32) & 1u32) << 26;
        // cpp: layoutng_style/style/computed_style_base_data.cc:116
        bits[4] |= ((false as u32) & 1u32) << 27;
        // cpp: layoutng_style/style/computed_style_base_data.cc:117
        bits[4] |= ((EPrintColorAdjust::kEconomy as u32) & 1u32) << 28;
        // cpp: layoutng_style/style/computed_style_base_data.cc:118
        bits[4] |= ((EOrder::kLogical as u32) & 1u32) << 29;
        // cpp: layoutng_style/style/computed_style_base_data.cc:119
        bits[4] |= ((TextWrapMode::kWrap as u32) & 1u32) << 30;
        Self {
            bits_: bits.map(Cell::new),
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBase {
    // C++ makes this constructor protected; only this crate constructs the base.
    // cpp: layoutng_style/style/computed_style_base_data.cc:19-122
    pub(crate) fn new() -> Self {
        Self {
            misc_inherited_data_: Member::from_ptr(StyleMiscInheritedData::Create()),
            misc_data_: Member::from_ptr(StyleMiscData::Create()),
            inherited_data_: Member::from_ptr(StyleInheritedData::Create()),
            visual_data_: Member::from_ptr(StyleVisualData::Create()),
            svginherited_data_: Member::from_ptr(StyleSvginheritedData::Create()),
            box_data_: Member::from_ptr(StyleBoxData::Create()),
            svg_data_: Member::from_ptr(StyleSVGData::Create()),
            surround_data_: Member::from_ptr(StyleSurroundData::Create()),
            background_data_: Member::from_ptr(StyleBackgroundData::Create()),
            base_data_: Member::default(),
            data_: ComputedStyleBaseData::default(),
        }
    }
}

// cpp: layoutng_style/style/computed_style_base.h:6923
impl Default for ComputedStyleBase {
    fn default() -> Self {
        Self::new()
    }
}

// The C++ defaulted copy constructor copies the GC handles and packed data.
// cpp: layoutng_style/style/computed_style_base.h:6924
impl Clone for ComputedStyleBase {
    fn clone(&self) -> Self {
        Self {
            misc_inherited_data_: Member::from_ptr(self.misc_inherited_data_.Get()),
            misc_data_: Member::from_ptr(self.misc_data_.Get()),
            inherited_data_: Member::from_ptr(self.inherited_data_.Get()),
            visual_data_: Member::from_ptr(self.visual_data_.Get()),
            svginherited_data_: Member::from_ptr(self.svginherited_data_.Get()),
            box_data_: Member::from_ptr(self.box_data_.Get()),
            svg_data_: Member::from_ptr(self.svg_data_.Get()),
            surround_data_: Member::from_ptr(self.surround_data_.Get()),
            background_data_: Member::from_ptr(self.background_data_.Get()),
            base_data_: Member::from_ptr(self.base_data_.Get()),
            data_: self.data_.clone(),
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBase {
    // cpp: layoutng_style/style/computed_style_base.h:6925
    // cpp: layoutng_style/style/computed_style_base_data.cc:124-140
    pub(crate) fn from_builder(builder: &ComputedStyleBuilderBase) -> Self {
        let result = Self {
            misc_inherited_data_: Member::from_ptr(builder.misc_inherited_data_),
            misc_data_: Member::from_ptr(builder.misc_data_),
            inherited_data_: Member::from_ptr(builder.inherited_data_),
            visual_data_: Member::from_ptr(builder.visual_data_),
            svginherited_data_: Member::from_ptr(builder.svginherited_data_),
            box_data_: Member::from_ptr(builder.box_data_),
            svg_data_: Member::from_ptr(builder.svg_data_),
            surround_data_: Member::from_ptr(builder.surround_data_),
            background_data_: Member::from_ptr(builder.background_data_),
            base_data_: Member::from_ptr(builder.base_data_),
            data_: builder.data_.clone(),
        };
        result
            .data_
            .set_is_stacking_context_without_containment_bits(0);
        result
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBuilderBase {
    // cpp: layoutng_style/style/computed_style_base.h:18932
    // cpp: layoutng_style/style/computed_style_base_data.cc:1288-1300
    pub(crate) fn from_style(style: &ComputedStyleBase) -> Self {
        Self {
            misc_inherited_data_: style.misc_inherited_data_.Get(),
            misc_data_: style.misc_data_.Get(),
            inherited_data_: style.inherited_data_.Get(),
            visual_data_: style.visual_data_.Get(),
            svginherited_data_: style.svginherited_data_.Get(),
            box_data_: style.box_data_.Get(),
            svg_data_: style.svg_data_.Get(),
            surround_data_: style.surround_data_.Get(),
            background_data_: style.background_data_.Get(),
            base_data_: style.base_data_.Get(),
            data_: style.data_.clone(),
            access_: BuilderAccessFlags::default(),
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBuilderBase {
    // cpp: layoutng_style/style/computed_style_base.h:18934-18935
    // cpp: layoutng_style/style/computed_style_base_data.cc:1302-1425
    pub(crate) fn from_parent_and_noninherited(
        source_for_noninherited: &ComputedStyleBase,
        parent_style: &ComputedStyleBase,
    ) -> Self {
        // The C++ aggregate copies each source bitfield then overwrites the
        // reset and inherited fields. Cloning the packed words has the same
        // values and avoids reading padding through an aggregate memcpy.
        let mut data = source_for_noninherited.data_.clone();
        // cpp: layoutng_style/style/computed_style_base_data.cc:1318
        data.set_pseudo_element_styles_bits(0);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1320
        data.set_style_type_bits(0);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1329
        data.set_affected_by_active_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1332
        data.set_is_stacking_context_without_containment_bits(0);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1335
        data.set_affected_by_drag_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1336
        data.set_affected_by_focus_within_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1337
        data.set_affected_by_hover_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1343
        data.set_child_has_explicit_inheritance_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1349
        data.set_has_attr_function_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1372
        data.set_is_link_bit(false);
        // cpp: layoutng_style/style/computed_style_base_data.cc:1385
        data.set_pointer_events_bits(parent_style.data_.pointer_events_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1386
        data.set_text_align_bits(parent_style.data_.text_align_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1387
        data.set_writing_mode_bits(parent_style.data_.writing_mode_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1388
        data.set_caret_shape_bits(parent_style.data_.caret_shape_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1389
        data.set_inside_link_bits(parent_style.data_.inside_link_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1390
        data.set_text_wrap_style_bits(parent_style.data_.text_wrap_style_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1391
        data.set_visibility_bits(parent_style.data_.visibility_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1392
        data.set_white_space_collapse_bits(parent_style.data_.white_space_collapse_bits());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1393
        data.set_border_collapse_bit(parent_style.data_.border_collapse_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1394
        data.set_caption_side_bit(parent_style.data_.caption_side_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1395
        data.set_caret_animation_bit(parent_style.data_.caret_animation_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1396
        data.set_color_scheme_flags_is_normal_bit(
            parent_style.data_.color_scheme_flags_is_normal_bit(),
        );
        // cpp: layoutng_style/style/computed_style_base_data.cc:1397
        data.set_color_scheme_forced_bit(parent_style.data_.color_scheme_forced_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1398
        data.set_dark_color_scheme_bit(parent_style.data_.dark_color_scheme_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1399
        data.set_direction_bit(parent_style.data_.direction_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1400
        data.set_is_css_inert_bit(parent_style.data_.is_css_inert_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1401
        data.set_is_html_inert_bit(parent_style.data_.is_html_inert_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1402
        data.set_list_style_position_bit(parent_style.data_.list_style_position_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1403
        data.set_prefers_default_scrollbar_styles_bit(
            parent_style.data_.prefers_default_scrollbar_styles_bit(),
        );
        // cpp: layoutng_style/style/computed_style_base_data.cc:1404
        data.set_print_color_adjust_bit(parent_style.data_.print_color_adjust_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1405
        data.set_rtl_ordering_bit(parent_style.data_.rtl_ordering_bit());
        // cpp: layoutng_style/style/computed_style_base_data.cc:1406
        data.set_text_wrap_mode_bit(parent_style.data_.text_wrap_mode_bit());
        let mut result = Self {
            misc_inherited_data_: parent_style.misc_inherited_data_.Get(),
            misc_data_: source_for_noninherited.misc_data_.Get(),
            inherited_data_: parent_style.inherited_data_.Get(),
            visual_data_: source_for_noninherited.visual_data_.Get(),
            svginherited_data_: parent_style.svginherited_data_.Get(),
            box_data_: source_for_noninherited.box_data_.Get(),
            svg_data_: source_for_noninherited.svg_data_.Get(),
            surround_data_: source_for_noninherited.surround_data_.Get(),
            background_data_: source_for_noninherited.background_data_.Get(),
            base_data_: std::ptr::null_mut(),
            data_: data,
            access_: BuilderAccessFlags::default(),
        };

        // cpp: layoutng_style/style/computed_style_base_data.cc:1411-1424
        let misc = unsafe { &*result.misc_data_ };
        let misc_7 = unsafe { &*misc.misc_7_data_.Get() };
        if misc_7.custom_highlight_names_.is_some() {
            let misc = Self::AccessPtr(&mut result.misc_data_, &result.access_.misc_data_);
            let misc_7 = Self::AccessMember(
                unsafe { &mut (*misc).misc_7_data_ },
                &result.access_.misc_7_data_,
            );
            unsafe { &mut *misc_7 }.custom_highlight_names_ = None;
        }
        if unsafe { &*result.misc_data_ }.has_non_ua_highlight_pseudo_styles_bit() {
            let misc = Self::AccessPtr(&mut result.misc_data_, &result.access_.misc_data_);
            unsafe { &mut *misc }.set_has_non_ua_highlight_pseudo_styles_bit(false);
        }
        if unsafe { &*result.misc_data_ }.has_non_universal_highlight_pseudo_styles_bit() {
            let misc = Self::AccessPtr(&mut result.misc_data_, &result.access_.misc_data_);
            unsafe { &mut *misc }.set_has_non_universal_highlight_pseudo_styles_bit(false);
        }
        if unsafe { &*result.misc_data_ }.highlights_depend_on_size_container_queries_bit() {
            let misc = Self::AccessPtr(&mut result.misc_data_, &result.access_.misc_data_);
            unsafe { &mut *misc }.set_highlights_depend_on_size_container_queries_bit(false);
        }
        result
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBuilderBase {
    // cpp: layoutng_style/style/computed_style.h:2949-2950
    // cpp: layoutng_style/style/computed_style_base.h:18937
    // cpp: layoutng_style/style/computed_style_base_data.cc:1427-1468
    pub(crate) fn PropagateIndependentInheritedProperties(
        &mut self,
        parent_style: &ComputedStyleBase,
    ) {
        // cpp: layoutng_style/style/computed_style_base_data.cc:1429-1431
        if self.data_.empty_cells_is_inherited_bit() {
            let value = unsafe { &*parent_style.misc_inherited_data_.Get() }.empty_cells_bit();
            if unsafe { &*self.misc_inherited_data_ }.empty_cells_bit() != value {
                let data = Self::AccessPtr(
                    &mut self.misc_inherited_data_,
                    &self.access_.misc_inherited_data_,
                );
                unsafe { &mut *data }.set_empty_cells_bit(value);
            }
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1432-1434
        if self.data_.interactivity_is_inherited_bit() {
            let value = unsafe { &*parent_style.misc_inherited_data_.Get() }.interactivity_bit();
            if unsafe { &*self.misc_inherited_data_ }.interactivity_bit() != value {
                let data = Self::AccessPtr(
                    &mut self.misc_inherited_data_,
                    &self.access_.misc_inherited_data_,
                );
                unsafe { &mut *data }.set_interactivity_bit(value);
            }
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1435-1437
        if self.data_.color_is_inherited_bit() {
            let value = &unsafe { &*parent_style.inherited_data_.Get() }.color_;
            if &unsafe { &*self.inherited_data_ }.color_ != value {
                let data =
                    Self::AccessPtr(&mut self.inherited_data_, &self.access_.inherited_data_);
                unsafe { &mut *data }.color_ = value.clone();
            }
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1438-1440
        if self.data_.cursor_is_inherited_bit() {
            let value = unsafe { &*parent_style.inherited_data_.Get() }.cursor_bits();
            if unsafe { &*self.inherited_data_ }.cursor_bits() != value {
                let data =
                    Self::AccessPtr(&mut self.inherited_data_, &self.access_.inherited_data_);
                unsafe { &mut *data }.set_cursor_bits(value);
            }
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1441-1443
        if self.data_.text_transform_is_inherited_bit() {
            let value = unsafe { &*parent_style.inherited_data_.Get() }.text_transform_bits();
            if unsafe { &*self.inherited_data_ }.text_transform_bits() != value {
                let data =
                    Self::AccessPtr(&mut self.inherited_data_, &self.access_.inherited_data_);
                unsafe { &mut *data }.set_text_transform_bits(value);
            }
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1444-1446
        if self.data_.pointer_events_is_inherited_bit() {
            self.data_
                .set_pointer_events_bits(parent_style.data_.pointer_events_bits());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1447-1449
        if self.data_.visibility_is_inherited_bit() {
            self.data_
                .set_visibility_bits(parent_style.data_.visibility_bits());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1450-1452
        if self.data_.border_collapse_is_inherited_bit() {
            self.data_
                .set_border_collapse_bit(parent_style.data_.border_collapse_bit());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1453-1455
        if self.data_.caption_side_is_inherited_bit() {
            self.data_
                .set_caption_side_bit(parent_style.data_.caption_side_bit());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1456-1458
        if self.data_.is_css_inert_is_inherited_bit() {
            self.data_
                .set_is_css_inert_bit(parent_style.data_.is_css_inert_bit());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1459-1461
        if self.data_.is_html_inert_is_inherited_bit() {
            self.data_
                .set_is_html_inert_bit(parent_style.data_.is_html_inert_bit());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1462-1464
        if self.data_.list_style_position_is_inherited_bit() {
            self.data_
                .set_list_style_position_bit(parent_style.data_.list_style_position_bit());
        }
        // cpp: layoutng_style/style/computed_style_base_data.cc:1465-1467
        if self.data_.rtl_ordering_is_inherited_bit() {
            self.data_
                .set_rtl_ordering_bit(parent_style.data_.rtl_ordering_bit());
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBase {
    // cpp: layoutng_style/style/computed_style_base.h:5471
    // cpp: layoutng_style/style/computed_style_base_data.cc:1473-1475
    pub fn Trace(&self, visitor: &mut Visitor) {
        let style = unsafe { &*(self as *const ComputedStyleBase as *const ComputedStyle) };
        style.TraceAfterDispatch(visitor);
    }
}
