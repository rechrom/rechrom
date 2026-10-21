use font_engine::Font;
use foundation::gfx;

use super::anonymous_style::*;
use super::appearance::*;
use super::applied_text_decoration::*;
use super::basic_shapes::*;
use super::border_edge::*;
use super::border_image_length::*;
use super::border_image_length_box::*;
use super::clip_path_operation::*;
use super::color_scheme::*;
use super::computed_grid_template_areas::*;
use super::computed_grid_track_list::*;
use super::computed_style_alignment::*;
use super::computed_style_constants::*;
use super::computed_style_font_metrics::*;
use super::computed_style_layout::*;
use super::computed_style_text_align::*;
use super::content_data::*;
use super::counter_directives::*;
use super::cursor_data::*;
use super::cursor_list::*;
use super::default_anchor_data::*;
use super::display_adjustment::*;
use super::display_style::*;
use super::fill_layer::*;
use super::fill_layer_data::*;
use super::filter_operations::*;
use super::filter_operations_data::*;
use super::flow_tolerance::*;
use super::font_size_style::*;
use super::forward::*;
use super::gap_data::*;
use super::gap_data_list::*;
use super::grid_area::*;
use super::grid_enums::*;
use super::grid_lanes_direction::*;
use super::grid_position::*;
use super::grid_track_list::*;
use super::grid_track_size::*;
use super::list_style_type_data::*;
use super::max_lines_data::*;
use super::member_copy::*;
use super::named_grid_lines_map::*;
use super::natural_sizing_info::*;
use super::nine_piece_image::*;
use super::offset_path_operation::*;
use super::ordered_named_grid_lines::*;
use super::outline_type::*;
use super::page_orientation::*;
use super::page_size_type::*;
use super::paint_images::*;
use super::position_area::*;
use super::position_try_fallbacks::*;
use super::scroll_enums::*;
use super::scroll_marker_group::*;
use super::scroll_snap_data::*;
use super::scrollbar_style_data::*;
use super::shadow_data::*;
use super::shadow_list::*;
use super::shape_value::*;
use super::style_anchor_scope::*;
use super::style_base_data::*;
use super::style_border_shape::*;
use super::style_cached_data::*;
use super::style_content_alignment_data::*;
use super::style_difference::*;
use super::style_flex_wrap_data::*;
use super::style_highlight_data::*;
use super::style_hyphenate_limit_chars::*;
use super::style_image::*;
use super::style_inherited_variables::*;
use super::style_initial_data::*;
use super::style_interest_delay::*;
use super::style_intrinsic_length::*;
use super::style_non_inherited_variables::*;
use super::style_offset_rotation::*;
use super::style_overflow_clip_margin::*;
use super::style_position_anchor::*;
use super::style_reflection::*;
use super::style_scrollbar_color::*;
use super::style_self_alignment_data::*;
use super::style_svg_resource::*;
use super::style_timeline_scope::*;
use super::style_transform::*;
use super::style_trigger_scope::*;
use super::style_ua_shadow_host_data::*;
use super::style_variables::*;
use super::style_view_transition_group::*;
use super::style_view_transition_name::*;
use super::style_will_change_data::*;
use super::superellipse::*;
use super::svg_dash_array::*;
use super::svg_paint::*;
use super::text_box_edge::*;
use super::text_decoration_inset::*;
use super::text_fit::*;
use super::text_indent_flags::*;
use super::text_overflow_data::*;
use super::text_size_adjust::*;
use super::theme_defaults::*;
use super::theme_types::*;
use super::timeline_inset::*;
use super::transform_origin::*;
use super::unzoomed_length::*;
use crate::css::style_auto_color::*;
use crate::css::style_caret_color::*;
use crate::css::style_color::StyleColor;
use crate::css::white_space::*;
use foundation::keywords;
use foundation::*;
use std::cell::OnceCell;

// cpp: layoutng_style/style/computed_style_initial_values.h:43-48
pub struct ComputedStyleInitialValues;

#[allow(non_snake_case)]
impl ComputedStyleInitialValues {
    // cpp: layoutng_style/style/computed_style_initial_values.h:57-61
    pub fn InitialContentAlignment() -> StyleContentAlignmentData {
        StyleContentAlignmentData::new(
            ContentPosition::kNormal,
            ContentDistributionType::kDefault,
            OverflowAlignment::kDefault,
        )
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:62-65
    pub fn InitialDefaultAlignment() -> StyleSelfAlignmentData {
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:66-66
    pub fn InitialBorderImageSource() -> Option<*mut StyleImage> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:67-67
    pub fn InitialBorderWidth() -> f32 {
        3.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:68-70
    pub fn InitialColumnRuleWidth() -> GapDataList<i32> {
        GapDataList::<i32>::DefaultGapWidthDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:71-73
    pub fn InitialRowRuleWidth() -> GapDataList<i32> {
        GapDataList::<i32>::DefaultGapWidthDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:77-77
    pub fn InitialGridAutoRepeatInsertionPoint() -> WtfSizeT {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:78-80
    pub fn InitialGridAutoRepeatType() -> AutoRepeatType {
        AutoRepeatType::kNoAutoRepeat
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:81-83
    pub fn InitialGridAxisType() -> GridAxisType {
        GridAxisType::kStandaloneAxis
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:85-87
    pub fn InitialVerticalAlign() -> EVerticalAlign {
        EVerticalAlign::kBaseline
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:90-90
    pub fn InitialPerspectiveOriginX() -> Length {
        Length::Percent(50.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:93-93
    pub fn InitialPerspectiveOriginY() -> Length {
        Length::Percent(50.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:96-96
    pub fn InitialTransformOriginX() -> Length {
        Length::Percent(50.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:98-98
    pub fn InitialTransformOriginY() -> Length {
        Length::Percent(50.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:100-100
    pub fn InitialTransformOriginZ() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:103-103
    pub fn InitialMaskBoxImageSource() -> Option<*mut StyleImage> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:105-107
    pub fn InitialFilter() -> &'static FilterOperations {
        Self::InitialFilterInternal()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:108-110
    pub fn InitialBackdropFilter() -> &'static FilterOperations {
        Self::InitialFilterInternal()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:112-117
    pub fn InitialTextEmphasisPosition() -> TextEmphasisPosition {
        if RuntimeEnabledFeatures::TextEmphasisPositionAutoEnabled() {
            TextEmphasisPosition::kAuto
        } else {
            TextEmphasisPosition::kOverRight
        }
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:121-123
    pub fn InitialColorScheme() -> Vector<AtomicString> {
        Vector::<AtomicString>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:125-127
    pub fn InitialForcedColorAdjust() -> EForcedColorAdjust {
        EForcedColorAdjust::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:129-131
    pub fn InitialMathDepth() -> i16 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:133-135
    pub fn InitialPosition() -> EPosition {
        EPosition::kStatic
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:137-139
    pub fn InitialPositionAnchor() -> StylePositionAnchor {
        StylePositionAnchor::Initial()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:141-143
    pub fn InitialTextSizeAdjust() -> TextSizeAdjust {
        TextSizeAdjust::AdjustAuto()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:145-147
    pub fn InitialInternalVisitedColor() -> StyleColor {
        StyleColor::from_color(Color::kBlack)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:149-151
    pub fn InitialAppearance() -> AppearanceValue {
        AppearanceValue::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:153-155
    pub fn InitialColor() -> StyleColor {
        StyleColor::from_color(Color::kBlack)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:157-159
    pub fn InitialDirection() -> TextDirection {
        TextDirection::kLtr
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:161-163
    pub fn InitialPositionArea() -> PositionArea {
        PositionArea::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:165-167
    pub fn InitialTextOrientation() -> ETextOrientation {
        ETextOrientation::kMixed
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:169-171
    pub fn InitialWritingMode() -> WritingMode {
        WritingMode::kHorizontalTb
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:173-175
    pub fn InitialZoom() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:177-179
    pub fn InitialInternalForcedVisitedColor() -> StyleColor {
        StyleColor::from_keyword(CSSValueID::kCanvastext)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:181-183
    pub fn InitialInternalVisitedBackgroundColor() -> StyleColor {
        StyleColor::from_color(Color::kTransparent)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:185-187
    pub fn InitialInternalVisitedBorderBottomColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:189-191
    pub fn InitialInternalVisitedBorderLeftColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:193-195
    pub fn InitialInternalVisitedBorderRightColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:197-199
    pub fn InitialInternalVisitedBorderTopColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:201-203
    pub fn InitialInternalVisitedCaretColor() -> StyleCaretColor {
        StyleCaretColor::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:205-207
    pub fn InitialInternalVisitedColumnRuleColor() -> GapDataList<StyleColor> {
        GapDataList::<StyleColor>::DefaultGapColorDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:209-211
    pub fn InitialInternalVisitedFillPaint() -> SVGPaint {
        SVGPaint::from_color(Color::kBlack)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:213-215
    pub fn InitialInternalVisitedOutlineColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:217-219
    pub fn InitialInternalVisitedStrokePaint() -> SVGPaint {
        SVGPaint::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:221-223
    pub fn InitialInternalVisitedTextDecorationColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:225-227
    pub fn InitialInternalVisitedTextEmphasisColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:229-231
    pub fn InitialInternalVisitedTextFillColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:233-235
    pub fn InitialInternalVisitedTextStrokeColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:237-239
    pub fn InitialAccentColor() -> StyleAutoColor {
        StyleAutoColor::AutoColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:241-243
    pub fn InitialAlignContent() -> StyleContentAlignmentData {
        StyleContentAlignmentData::new(
            ContentPosition::kNormal,
            ContentDistributionType::kDefault,
            OverflowAlignment::kDefault,
        )
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:245-247
    pub fn InitialAlignItems() -> StyleSelfAlignmentData {
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:249-251
    pub fn InitialAlignSelf() -> StyleSelfAlignmentData {
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kAuto, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:253-255
    pub fn InitialAlignmentBaseline() -> EAlignmentBaseline {
        EAlignmentBaseline::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:257-259
    pub fn InitialAnchorName() -> Option<*mut ScopedCSSNameList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:261-263
    pub fn InitialAnchorScope() -> StyleAnchorScope {
        StyleAnchorScope::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:265-267
    pub fn InitialAspectRatio() -> StyleAspectRatio {
        StyleAspectRatio::new(EAspectRatioType::kAuto, gfx::SizeF::default())
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:269-271
    pub fn InitialBackfaceVisibility() -> EBackfaceVisibility {
        EBackfaceVisibility::kVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:273-275
    pub fn InitialBackgroundColor() -> StyleColor {
        StyleColor::from_color(Color::kTransparent)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:277-279
    pub fn InitialBaselineShift() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:281-283
    pub fn InitialBaselineSource() -> EBaselineSource {
        EBaselineSource::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:285-287
    pub fn InitialBlockEllipsis() -> EBlockEllipsis {
        EBlockEllipsis::kNoEllipsis
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:289-291
    pub fn InitialBorderBottomColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:293-295
    pub fn InitialBorderBottomLeftRadius() -> LengthSize {
        LengthSize::new(&Length::Fixed(0), &Length::Fixed(0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:297-299
    pub fn InitialBorderBottomRightRadius() -> LengthSize {
        LengthSize::new(&Length::Fixed(0), &Length::Fixed(0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:301-303
    pub fn InitialBorderBottomStyle() -> EBorderStyle {
        EBorderStyle::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:305-307
    pub fn InitialBorderBottomWidth() -> i32 {
        3
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:309-311
    pub fn InitialBorderCollapse() -> EBorderCollapse {
        EBorderCollapse::kSeparate
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:313-315
    pub fn InitialBorderLeftColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:317-319
    pub fn InitialBorderLeftStyle() -> EBorderStyle {
        EBorderStyle::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:321-323
    pub fn InitialBorderLeftWidth() -> i32 {
        3
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:325-327
    pub fn InitialBorderRightColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:329-331
    pub fn InitialBorderRightStyle() -> EBorderStyle {
        EBorderStyle::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:333-335
    pub fn InitialBorderRightWidth() -> i32 {
        3
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:337-339
    pub fn InitialBorderShape() -> Option<*mut StyleBorderShape> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:341-343
    pub fn InitialBorderTopColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:345-347
    pub fn InitialBorderTopLeftRadius() -> LengthSize {
        LengthSize::new(&Length::Fixed(0), &Length::Fixed(0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:349-351
    pub fn InitialBorderTopRightRadius() -> LengthSize {
        LengthSize::new(&Length::Fixed(0), &Length::Fixed(0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:353-355
    pub fn InitialBorderTopStyle() -> EBorderStyle {
        EBorderStyle::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:357-359
    pub fn InitialBorderTopWidth() -> i32 {
        3
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:361-363
    pub fn InitialBottom() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:365-367
    pub fn InitialBoxDecorationBreak() -> EBoxDecorationBreak {
        EBoxDecorationBreak::kSlice
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:369-371
    pub fn InitialBoxShadow() -> Option<*mut ShadowList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:373-375
    pub fn InitialBoxSizing() -> EBoxSizing {
        EBoxSizing::kContentBox
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:377-379
    pub fn InitialBreakAfter() -> EBreakBetween {
        EBreakBetween::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:381-383
    pub fn InitialBreakBefore() -> EBreakBetween {
        EBreakBetween::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:385-387
    pub fn InitialBreakInside() -> EBreakInside {
        EBreakInside::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:389-391
    pub fn InitialBufferedRendering() -> EBufferedRendering {
        EBufferedRendering::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:393-395
    pub fn InitialCaptionSide() -> ECaptionSide {
        ECaptionSide::kTop
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:397-399
    pub fn InitialCaretAnimation() -> ECaretAnimation {
        ECaretAnimation::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:401-403
    pub fn InitialCaretColor() -> StyleCaretColor {
        StyleCaretColor::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:405-407
    pub fn InitialCaretShape() -> ECaretShape {
        ECaretShape::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:409-411
    pub fn InitialClear() -> EClear {
        EClear::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:413-415
    pub fn InitialClip() -> LengthBox {
        LengthBox::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:417-419
    pub fn InitialClipPath() -> Option<*mut dyn ClipPathOperation> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:421-423
    pub fn InitialClipRule() -> WindRule {
        WindRule::RULE_NONZERO
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:425-427
    pub fn InitialColorInterpolation() -> EColorInterpolation {
        EColorInterpolation::kSRGB
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:429-431
    pub fn InitialColorInterpolationFilters() -> EColorInterpolation {
        EColorInterpolation::kLinearrgb
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:433-435
    pub fn InitialColorRendering() -> EColorRendering {
        EColorRendering::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:437-439
    pub fn InitialColumnCount() -> u16 {
        1
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:441-443
    pub fn InitialColumnFill() -> EColumnFill {
        EColumnFill::kBalance
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:445-447
    pub fn InitialColumnGap() -> Option<Length> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:449-451
    pub fn InitialColumnHeight() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:453-455
    pub fn InitialColumnRuleBreak() -> RuleBreak {
        RuleBreak::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:457-459
    pub fn InitialColumnRuleColor() -> GapDataList<StyleColor> {
        GapDataList::<StyleColor>::DefaultGapColorDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:461-463
    pub fn InitialColumnRuleInsetCapEnd() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:465-467
    pub fn InitialColumnRuleInsetCapStart() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:469-471
    pub fn InitialColumnRuleInsetJunctionEnd() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:473-475
    pub fn InitialColumnRuleInsetJunctionStart() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:477-479
    pub fn InitialColumnRuleStyle() -> GapDataList<EBorderStyle> {
        GapDataList::<EBorderStyle>::DefaultGapStyleDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:481-483
    pub fn InitialColumnRuleVisibilityItems() -> RuleVisibilityItems {
        RuleVisibilityItems::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:485-487
    pub fn InitialColumnSpan() -> EColumnSpan {
        EColumnSpan::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:489-491
    pub fn InitialColumnWidth() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:493-495
    pub fn InitialColumnWrap() -> EColumnWrap {
        EColumnWrap::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:497-499
    pub fn InitialContain() -> u32 {
        Containment::kContainsNone.value() as u32
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:501-503
    pub fn InitialContainIntrinsicHeight() -> StyleIntrinsicLength {
        StyleIntrinsicLength::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:505-507
    pub fn InitialContainIntrinsicWidth() -> StyleIntrinsicLength {
        StyleIntrinsicLength::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:509-511
    pub fn InitialContainerName() -> Option<*mut ScopedCSSNameList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:513-515
    pub fn InitialContainerType() -> u32 {
        EContainerType::kContainerTypeNormal.value() as u32
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:517-519
    pub fn InitialContent() -> Option<*mut dyn ContentData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:521-523
    pub fn InitialContentVisibility() -> EContentVisibility {
        EContentVisibility::kVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:525-527
    pub fn InitialCornerBottomLeftShape() -> Superellipse {
        Superellipse::Round()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:529-531
    pub fn InitialCornerBottomRightShape() -> Superellipse {
        Superellipse::Round()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:533-535
    pub fn InitialCornerTopLeftShape() -> Superellipse {
        Superellipse::Round()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:537-539
    pub fn InitialCornerTopRightShape() -> Superellipse {
        Superellipse::Round()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:541-543
    pub fn InitialCursor() -> ECursor {
        ECursor::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:545-547
    pub fn InitialCx() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:549-551
    pub fn InitialCy() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:553-555
    pub fn InitialD() -> Option<*mut StylePath> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:557-559
    pub fn InitialDominantBaseline() -> EDominantBaseline {
        EDominantBaseline::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:561-563
    pub fn InitialDynamicRangeLimit() -> DynamicRangeLimit {
        DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:565-567
    pub fn InitialEmptyCells() -> EEmptyCells {
        EEmptyCells::kShow
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:569-571
    pub fn InitialFieldSizing() -> EFieldSizing {
        EFieldSizing::kFixed
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:573-575
    pub fn InitialFillPaint() -> SVGPaint {
        SVGPaint::CreateInitialBlack()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:577-579
    pub fn InitialFillOpacity() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:581-583
    pub fn InitialFillRule() -> WindRule {
        WindRule::RULE_NONZERO
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:585-587
    pub fn InitialFlexBasis() -> Length {
        Length::Auto().clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:589-591
    pub fn InitialFlexDirection() -> EFlexDirection {
        EFlexDirection::kRow
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:593-595
    pub fn InitialFlexGrow() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:597-599
    pub fn InitialFlexLineCount() -> u16 {
        1
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:601-603
    pub fn InitialFlexShrink() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:605-607
    pub fn InitialFlexWrap() -> StyleFlexWrapData {
        StyleFlexWrapData::new(FlexWrapMode::kNowrap)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:609-611
    pub fn InitialFloating() -> EFloat {
        EFloat::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:613-615
    pub fn InitialFloodColor() -> StyleColor {
        StyleColor::from_color(Color::kBlack)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:617-619
    pub fn InitialFloodOpacity() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:621-623
    pub fn InitialFlowTolerance() -> FlowTolerance {
        FlowTolerance::from_keyword(CSSValueID::kNormal)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:625-627
    pub fn InitialFrameSizing() -> EFrameSizing {
        EFrameSizing::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:629-631
    pub fn InitialGridAutoColumns() -> GridTrackList {
        GridTrackList::from_default_track_size(&GridTrackSize::from_length(Length::Auto()))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:633-635
    pub fn InitialGridAutoFlow() -> GridAutoFlow {
        GridAutoFlow::kAutoFlowRow
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:637-639
    pub fn InitialGridAutoRows() -> GridTrackList {
        GridTrackList::from_default_track_size(&GridTrackSize::from_length(Length::Auto()))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:641-643
    pub fn InitialGridColumnEnd() -> GridPosition {
        GridPosition::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:645-647
    pub fn InitialGridColumnStart() -> GridPosition {
        GridPosition::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:649-651
    pub fn InitialGridLanesDirection() -> GridLanesDirection {
        GridLanesDirection::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:653-655
    pub fn InitialGridLanesPack() -> EGridLanesPack {
        EGridLanesPack::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:657-659
    pub fn InitialGridRowEnd() -> GridPosition {
        GridPosition::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:661-663
    pub fn InitialGridRowStart() -> GridPosition {
        GridPosition::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:665-667
    pub fn InitialGridTemplateAreas() -> Option<*mut ComputedGridTemplateAreas> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:669-671
    pub fn InitialGridTemplateColumns() -> Option<*mut ComputedGridTrackList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:673-675
    pub fn InitialGridTemplateRows() -> Option<*mut ComputedGridTrackList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:677-679
    pub fn InitialHangingPunctuation() -> HangingPunctuation {
        HangingPunctuation::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:681-683
    pub fn InitialHeight() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:685-687
    pub fn InitialHyphenationString() -> AtomicString {
        AtomicString::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:689-691
    pub fn InitialHyphenateLimitChars() -> StyleHyphenateLimitChars {
        StyleHyphenateLimitChars::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:693-695
    pub fn InitialHyphens() -> Hyphens {
        Hyphens::kManual
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:697-699
    pub fn InitialImageAnimation() -> ImageAnimationEnum {
        ImageAnimationEnum::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:701-703
    pub fn InitialImageOrientation() -> RespectImageOrientationEnum {
        RespectImageOrientationEnum::kRespectImageOrientation
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:705-707
    pub fn InitialImageRendering() -> EImageRendering {
        EImageRendering::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:709-711
    pub fn InitialInitialLetter() -> StyleInitialLetter {
        StyleInitialLetter::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:713-715
    pub fn InitialInteractivity() -> EInteractivity {
        EInteractivity::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:717-719
    pub fn InitialInterestDelayEnd() -> StyleInterestDelay {
        StyleInterestDelay::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:721-723
    pub fn InitialInterestDelayStart() -> StyleInterestDelay {
        StyleInterestDelay::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:725-727
    pub fn InitialAlignContentBlockCenter() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:729-731
    pub fn InitialHasLineIfEmpty() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:733-735
    pub fn InitialInternalForcedBackgroundColor() -> StyleColor {
        StyleColor::from_keyword(CSSValueID::kCanvas)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:737-739
    pub fn InitialInternalForcedBorderColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:741-743
    pub fn InitialInternalForcedColor() -> StyleColor {
        StyleColor::from_keyword(CSSValueID::kCanvastext)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:745-747
    pub fn InitialInternalForcedOutlineColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:749-751
    pub fn InitialInternalOverscrollContainer() -> EInternalOverscrollContainer {
        EInternalOverscrollContainer::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:753-755
    pub fn InitialInternalOverscrollPosition() -> EInternalOverscrollPosition {
        EInternalOverscrollPosition::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:757-759
    pub fn InitialInternalUnbounded() -> EInternalUnbounded {
        EInternalUnbounded::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:761-763
    pub fn InitialInterpolateSize() -> EInterpolateSize {
        EInterpolateSize::kNumericOnly
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:765-767
    pub fn InitialIsolation() -> EIsolation {
        EIsolation::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:769-771
    pub fn InitialJustifyContent() -> StyleContentAlignmentData {
        StyleContentAlignmentData::new(
            ContentPosition::kNormal,
            ContentDistributionType::kDefault,
            OverflowAlignment::kDefault,
        )
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:773-775
    pub fn InitialJustifyItems() -> StyleSelfAlignmentData {
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kLegacy, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:777-779
    pub fn InitialJustifySelf() -> StyleSelfAlignmentData {
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kAuto, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:781-783
    pub fn InitialLeft() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:785-787
    pub fn InitialLetterSpacing() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:789-791
    pub fn InitialLightingColor() -> StyleColor {
        StyleColor::from_color(Color::kWhite)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:793-795
    pub fn InitialLineBreak() -> LineBreak {
        LineBreak::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:797-799
    pub fn InitialLineHeight() -> Length {
        Length::Auto().clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:801-803
    pub fn InitialListStyleImage() -> Option<*mut StyleImage> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:805-807
    pub fn InitialListStylePosition() -> EListStylePosition {
        EListStylePosition::kOutside
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:809-811
    pub fn InitialListStyleType() -> Option<*mut ListStyleTypeData> {
        Some(ListStyleTypeData::CreateCounterStyle(
            &keywords::kDisc,
            std::ptr::null(),
        ))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:813-815
    pub fn InitialMarginBottom() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:817-819
    pub fn InitialMarginLeft() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:821-823
    pub fn InitialMarginRight() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:825-827
    pub fn InitialMarginTop() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:829-831
    pub fn InitialMarginTrim() -> u32 {
        EMarginTrim::kMarginTrimNone.value() as u32
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:833-835
    pub fn InitialMarkerEndResource() -> Option<*mut StyleSVGResource> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:837-839
    pub fn InitialMarkerMidResource() -> Option<*mut StyleSVGResource> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:841-843
    pub fn InitialMarkerStartResource() -> Option<*mut StyleSVGResource> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:845-847
    pub fn InitialMaskType() -> EMaskType {
        EMaskType::kLuminance
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:849-851
    pub fn InitialMathShift() -> EMathShift {
        EMathShift::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:853-855
    pub fn InitialMathStyle() -> EMathStyle {
        EMathStyle::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:857-859
    pub fn InitialMaxContentSizing() -> EMaxContentSizing {
        EMaxContentSizing::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:861-863
    pub fn InitialMaxHeight() -> Length {
        Length::None()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:865-867
    pub fn InitialMaxWidth() -> Length {
        Length::None()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:869-871
    pub fn InitialMinHeight() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:873-875
    pub fn InitialMinWidth() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:877-879
    pub fn InitialBlendMode() -> BlendMode {
        BlendMode::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:881-883
    pub fn InitialObjectFit() -> EObjectFit {
        EObjectFit::kFill
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:885-887
    pub fn InitialObjectPosition() -> LengthPoint {
        LengthPoint::new(&Length::Percent(50.0), &Length::Percent(50.0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:889-891
    pub fn InitialObjectViewBox() -> Option<*mut dyn BasicShape> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:893-895
    pub fn InitialOffsetAnchor() -> LengthPoint {
        LengthPoint::new(&Length::Auto(), &Length::Auto())
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:897-899
    pub fn InitialOffsetDistance() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:901-903
    pub fn InitialOffsetPath() -> Option<*mut dyn OffsetPathOperation> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:905-907
    pub fn InitialOffsetPosition() -> LengthPoint {
        LengthPoint::new(&Length::None(), &Length::None())
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:909-911
    pub fn InitialOffsetRotate() -> StyleOffsetRotation {
        StyleOffsetRotation::new(0.0, OffsetRotationType::kAuto)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:913-915
    pub fn InitialOpacity() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:917-919
    pub fn InitialOrder() -> i32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:921-923
    pub fn InitialOriginTrialTestProperty() -> EOriginTrialTestProperty {
        EOriginTrialTestProperty::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:925-927
    pub fn InitialOrphans() -> i16 {
        2
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:929-931
    pub fn InitialOutlineColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:933-935
    pub fn InitialOutlineOffset() -> i32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:937-939
    pub fn InitialOutlineStyle() -> EBorderStyle {
        EBorderStyle::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:941-943
    pub fn InitialOutlineWidth() -> i32 {
        3
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:945-947
    pub fn InitialOverflowAnchor() -> EOverflowAnchor {
        EOverflowAnchor::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:949-951
    pub fn InitialOverflowClipMargin() -> Option<StyleOverflowClipMargin> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:953-955
    pub fn InitialOverflowWrap() -> EOverflowWrap {
        EOverflowWrap::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:957-959
    pub fn InitialOverflowX() -> EOverflow {
        EOverflow::kVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:961-963
    pub fn InitialOverflowY() -> EOverflow {
        EOverflow::kVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:965-967
    pub fn InitialOverlay() -> EOverlay {
        EOverlay::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:969-971
    pub fn InitialOverscrollBehaviorX() -> EOverscrollBehavior {
        EOverscrollBehavior::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:973-975
    pub fn InitialOverscrollBehaviorY() -> EOverscrollBehavior {
        EOverscrollBehavior::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:977-979
    pub fn InitialOverscrollContainerType() -> EOverscrollContainerType {
        EOverscrollContainerType::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:981-983
    pub fn InitialPaddingBottom() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:985-987
    pub fn InitialPaddingLeft() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:989-991
    pub fn InitialPaddingRight() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:993-995
    pub fn InitialPaddingTop() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:997-999
    pub fn InitialPage() -> AtomicString {
        AtomicString::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1001-1003
    pub fn InitialPageMarginSafety() -> EPageMarginSafety {
        EPageMarginSafety::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1005-1007
    pub fn InitialPageOrientation() -> PageOrientation {
        PageOrientation::kUpright
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1009-1011
    pub fn InitialPaintOrder() -> EPaintOrder {
        EPaintOrder::kPaintOrderNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1013-1015
    pub fn InitialPathLength() -> Length {
        Length::None()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1017-1019
    pub fn InitialPerspective() -> f32 {
        -1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1021-1023
    pub fn InitialPerspectiveOrigin() -> LengthPoint {
        LengthPoint::new(&Length::Percent(50.0), &Length::Percent(50.0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1025-1027
    pub fn InitialPointerEvents() -> EPointerEvents {
        EPointerEvents::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1029-1031
    pub fn InitialPositionTryFallbacks() -> Option<*mut PositionTryFallbacks> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1033-1035
    pub fn InitialPositionTryOrder() -> EPositionTryOrder {
        EPositionTryOrder::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1037-1039
    pub fn InitialPositionVisibility() -> PositionVisibility {
        PositionVisibility::kAnchorsVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1041-1043
    pub fn InitialPrintColorAdjust() -> EPrintColorAdjust {
        EPrintColorAdjust::kEconomy
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1045-1047
    pub fn InitialQuotes() -> Option<*mut QuotesData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1049-1051
    pub fn InitialR() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1053-1055
    pub fn InitialReadingFlow() -> EReadingFlow {
        EReadingFlow::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1057-1059
    pub fn InitialReadingOrder() -> i32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1061-1063
    pub fn InitialResize() -> EResize {
        EResize::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1065-1067
    pub fn InitialRight() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1069-1071
    pub fn InitialRotate() -> Option<*mut RotateTransformOperation> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1073-1075
    pub fn InitialRowGap() -> Option<Length> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1077-1079
    pub fn InitialRowRuleBreak() -> RuleBreak {
        RuleBreak::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1081-1083
    pub fn InitialRowRuleColor() -> GapDataList<StyleColor> {
        GapDataList::<StyleColor>::DefaultGapColorDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1085-1087
    pub fn InitialRowRuleInsetCapEnd() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1089-1091
    pub fn InitialRowRuleInsetCapStart() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1093-1095
    pub fn InitialRowRuleInsetJunctionEnd() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1097-1099
    pub fn InitialRowRuleInsetJunctionStart() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1101-1103
    pub fn InitialRowRuleStyle() -> GapDataList<EBorderStyle> {
        GapDataList::<EBorderStyle>::DefaultGapStyleDataList()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1105-1107
    pub fn InitialRowRuleVisibilityItems() -> RuleVisibilityItems {
        RuleVisibilityItems::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1109-1111
    pub fn InitialRubyAlign() -> ERubyAlign {
        ERubyAlign::kSpaceAround
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1113-1115
    pub fn InitialRubyOverhang() -> ERubyOverhang {
        ERubyOverhang::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1117-1119
    pub fn InitialRubyPosition() -> RubyPosition {
        RubyPosition::kOver
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1121-1123
    pub fn InitialRuleOverlap() -> ERuleOverlap {
        ERuleOverlap::kRowOverColumn
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1125-1127
    pub fn InitialRx() -> Length {
        Length::Auto().clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1129-1131
    pub fn InitialRy() -> Length {
        Length::Auto().clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1133-1135
    pub fn InitialScale() -> Option<*mut ScaleTransformOperation> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1137-1139
    pub fn InitialScrollAxisLock() -> EScrollAxisLock {
        EScrollAxisLock::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1141-1143
    pub fn InitialScrollBehavior() -> super::scroll_enums::mojom::ScrollBehavior {
        super::scroll_enums::mojom::blink::ScrollBehavior::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1145-1147
    pub fn InitialScrollInitialTarget() -> EScrollInitialTarget {
        EScrollInitialTarget::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1149-1151
    pub fn InitialScrollMarginBottom() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1153-1155
    pub fn InitialScrollMarginLeft() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1157-1159
    pub fn InitialScrollMarginRight() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1161-1163
    pub fn InitialScrollMarginTop() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1165-1167
    pub fn InitialScrollMarkerGroup() -> Option<*mut ScrollMarkerGroup> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1169-1171
    pub fn InitialScrollPaddingBottom() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1173-1175
    pub fn InitialScrollPaddingLeft() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1177-1179
    pub fn InitialScrollPaddingRight() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1181-1183
    pub fn InitialScrollPaddingTop() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1185-1187
    pub fn InitialScrollSnapAlign() -> cc::ScrollSnapAlign {
        cc::ScrollSnapAlign::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1189-1191
    pub fn InitialScrollSnapStop() -> EScrollSnapStop {
        EScrollSnapStop::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1193-1195
    pub fn InitialScrollSnapType() -> cc::ScrollSnapType {
        cc::ScrollSnapType::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1197-1199
    pub fn InitialScrollTargetGroup() -> EScrollTargetGroup {
        EScrollTargetGroup::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1201-1203
    pub fn InitialScrollTimelineAxis() -> Vector<TimelineAxis> {
        Vector::<TimelineAxis>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1205-1207
    pub fn InitialScrollTimelineName() -> Vector<AtomicString> {
        Vector::<AtomicString>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1209-1211
    pub fn InitialScrollbarColor() -> Option<*mut StyleScrollbarColor> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1213-1215
    pub fn InitialScrollbarGutter() -> u32 {
        ScrollbarGutter::kScrollbarGutterAuto.value() as u32
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1217-1219
    pub fn InitialScrollbarWidth() -> EScrollbarWidth {
        EScrollbarWidth::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1221-1223
    pub fn InitialShapeImageThreshold() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1225-1227
    pub fn InitialShapeMargin() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1229-1231
    pub fn InitialShapeOutside() -> Option<*mut ShapeValue> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1233-1235
    pub fn InitialShapeRendering() -> EShapeRendering {
        EShapeRendering::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1237-1239
    pub fn InitialSpeak() -> ESpeak {
        ESpeak::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1241-1243
    pub fn InitialStopColor() -> StyleColor {
        StyleColor::from_color(Color::kBlack)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1245-1247
    pub fn InitialStopOpacity() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1249-1251
    pub fn InitialStrokePaint() -> SVGPaint {
        SVGPaint::CreateInitial()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1253-1255
    pub fn InitialStrokeDashArray() -> Option<*mut SVGDashArray> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1257-1259
    pub fn InitialStrokeDashOffset() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1261-1263
    pub fn InitialCapStyle() -> LineCap {
        LineCap::kButtCap
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1265-1267
    pub fn InitialJoinStyle() -> LineJoin {
        LineJoin::kMiterJoin
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1269-1271
    pub fn InitialStrokeMiterLimit() -> f32 {
        4.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1273-1275
    pub fn InitialStrokeOpacity() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1277-1279
    pub fn InitialStrokeWidth() -> UnzoomedLength {
        UnzoomedLength::new(&Length::Fixed(1))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1281-1283
    pub fn InitialTabSize() -> TabSize {
        TabSize::spaces(8.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1285-1287
    pub fn InitialTableLayout() -> ETableLayout {
        ETableLayout::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1289-1291
    pub fn InitialTextAlign() -> ETextAlign {
        ETextAlign::kStart
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1293-1295
    pub fn InitialTextAlignLast() -> ETextAlignLast {
        ETextAlignLast::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1297-1299
    pub fn InitialTextAnchor() -> ETextAnchor {
        ETextAnchor::kStart
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1301-1303
    pub fn InitialTextAutospace() -> ETextAutospace {
        ETextAutospace::kNoAutospace
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1305-1307
    pub fn InitialTextBoxEdge() -> TextBoxEdge {
        TextBoxEdge::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1309-1311
    pub fn InitialTextBoxTrim() -> ETextBoxTrim {
        ETextBoxTrim::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1313-1315
    pub fn InitialTextCombine() -> ETextCombine {
        ETextCombine::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1317-1319
    pub fn InitialTextDecorationColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1321-1323
    pub fn InitialTextDecorationInset() -> TextDecorationInset {
        TextDecorationInset::new(&Length::Fixed(0), &Length::Fixed(0))
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1325-1327
    pub fn InitialTextDecorationLine() -> TextDecorationLine {
        TextDecorationLine::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1329-1331
    pub fn InitialTextDecorationSkipInk() -> ETextDecorationSkipInk {
        ETextDecorationSkipInk::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1333-1335
    pub fn InitialTextDecorationSkipSpaces() -> TextDecorationSkipSpaces {
        TextDecorationSkipSpaces::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1337-1339
    pub fn InitialTextDecorationStyle() -> ETextDecorationStyle {
        ETextDecorationStyle::kSolid
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1341-1343
    pub fn InitialTextDecorationThickness() -> TextDecorationThickness {
        TextDecorationThickness::new(&Length::Auto())
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1345-1347
    pub fn InitialTextEmphasisColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1349-1351
    pub fn InitialTextFit() -> TextFit {
        TextFit::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1353-1355
    pub fn InitialTextIndent() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1357-1359
    pub fn InitialTextJustify() -> TextJustify {
        TextJustify::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1361-1363
    pub fn InitialTextOverflow() -> TextOverflowData {
        TextOverflowData::from_type(TextOverflowType::kClip)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1365-1367
    pub fn InitialTextShadow() -> Option<*mut ShadowList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1369-1371
    pub fn InitialTextTransform() -> ETextTransform {
        ETextTransform::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1373-1375
    pub fn InitialTextUnderlineOffset() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1377-1379
    pub fn InitialTextUnderlinePosition() -> TextUnderlinePosition {
        TextUnderlinePosition::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1381-1383
    pub fn InitialTextWrapMode() -> TextWrapMode {
        TextWrapMode::kWrap
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1385-1387
    pub fn InitialTextWrapStyle() -> TextWrapStyle {
        TextWrapStyle::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1389-1391
    pub fn InitialTimelineScope() -> StyleTimelineScope {
        StyleTimelineScope::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1393-1395
    pub fn InitialTop() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1397-1399
    pub fn InitialTouchAction() -> TouchAction {
        TouchAction::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1401-1403
    pub fn InitialTransform() -> TransformOperations {
        EmptyTransformOperations.into()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1405-1407
    pub fn InitialTransformBox() -> ETransformBox {
        ETransformBox::kViewBox
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1409-1411
    pub fn InitialTransformOrigin() -> TransformOrigin {
        TransformOrigin::new(&Length::Percent(50.0), &Length::Percent(50.0), 0.0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1413-1415
    pub fn InitialTransformStyle3D() -> ETransformStyle3D {
        ETransformStyle3D::kFlat
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1417-1419
    pub fn InitialTranslate() -> Option<*mut TranslateTransformOperation> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1421-1423
    pub fn InitialTriggerScope() -> StyleTriggerScope {
        StyleTriggerScope::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1425-1427
    pub fn InitialUnicodeBidi() -> UnicodeBidi {
        UnicodeBidi::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1429-1431
    pub fn InitialUserSelect() -> EUserSelect {
        EUserSelect::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1433-1435
    pub fn InitialVectorEffect() -> EVectorEffect {
        EVectorEffect::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1437-1439
    pub fn InitialViewTimelineAxis() -> Vector<TimelineAxis> {
        Vector::<TimelineAxis>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1441-1443
    pub fn InitialViewTimelineInset() -> Vector<TimelineInset> {
        Vector::<TimelineInset>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1445-1447
    pub fn InitialViewTimelineName() -> Vector<AtomicString> {
        Vector::<AtomicString>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1449-1451
    pub fn InitialViewTransitionClass() -> Option<*mut ScopedCSSNameList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1453-1455
    pub fn InitialViewTransitionGroup() -> StyleViewTransitionGroup {
        StyleViewTransitionGroup::Normal()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1457-1459
    pub fn InitialViewTransitionName() -> Option<*mut StyleViewTransitionName> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1461-1463
    pub fn InitialViewTransitionScope() -> EViewTransitionScope {
        EViewTransitionScope::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1465-1467
    pub fn InitialVisibility() -> EVisibility {
        EVisibility::kVisible
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1469-1471
    pub fn InitialHorizontalBorderSpacing() -> i16 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1473-1475
    pub fn InitialVerticalBorderSpacing() -> i16 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1477-1479
    pub fn InitialBoxAlign() -> EBoxAlignment {
        EBoxAlignment::kStretch
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1481-1483
    pub fn InitialBoxDirection() -> EBoxDirection {
        EBoxDirection::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1485-1487
    pub fn InitialBoxFlex() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1489-1491
    pub fn InitialBoxOrdinalGroup() -> u32 {
        1
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1493-1495
    pub fn InitialBoxOrient() -> EBoxOrient {
        EBoxOrient::kHorizontal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1497-1499
    pub fn InitialBoxPack() -> EBoxPack {
        EBoxPack::kStart
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1501-1503
    pub fn InitialBoxReflect() -> Option<*mut StyleReflection> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1505-1507
    pub fn InitialWebkitLineClamp() -> i32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1509-1511
    pub fn InitialRtlOrdering() -> EOrder {
        EOrder::kLogical
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1515-1517
    pub fn InitialTextFillColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1519-1521
    pub fn InitialTextSecurity() -> ETextSecurity {
        ETextSecurity::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1523-1525
    pub fn InitialTextStrokeColor() -> StyleColor {
        StyleColor::CurrentColor()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1527-1529
    pub fn InitialTextStrokeWidth() -> f32 {
        0.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1531-1533
    pub fn InitialUserDrag() -> EUserDrag {
        EUserDrag::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1535-1537
    pub fn InitialUserModify() -> EUserModify {
        EUserModify::kReadOnly
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1539-1541
    pub fn InitialWhiteSpaceCollapse() -> WhiteSpaceCollapse {
        WhiteSpaceCollapse::kCollapse
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1543-1545
    pub fn InitialWidows() -> i16 {
        2
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1547-1549
    pub fn InitialWidth() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1551-1553
    pub fn InitialWillChange() -> Option<*mut StyleWillChangeData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1555-1557
    pub fn InitialDraggableRegionMode() -> EDraggableRegionMode {
        EDraggableRegionMode::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1559-1561
    pub fn InitialWordBreak() -> EWordBreak {
        EWordBreak::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1563-1565
    pub fn InitialWordSpacing() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1567-1569
    pub fn InitialX() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1571-1573
    pub fn InitialY() -> Length {
        Length::Fixed(0)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1575-1577
    pub fn InitialZIndex() -> i32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1579-1581
    pub fn InitialDisplay() -> EDisplay {
        EDisplay::kInline
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1583-1585
    pub fn InitialDisplayLayoutCustomName() -> AtomicString {
        g_null_atom.clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1587-1589
    pub fn InitialDisplayLayoutCustomParentName() -> AtomicString {
        g_null_atom.clone()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1591-1593
    pub fn InitialIsOriginalDisplayInlineType() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1595-1597
    pub fn InitialInsideLink() -> EInsideLink {
        EInsideLink::kNotInsideLink
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1599-1601
    pub fn InitialInForcedColorsMode() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1603-1605
    pub fn InitialViewportUnitFlags() -> u32 {
        0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1607-1609
    pub fn InitialStyleType() -> PseudoId {
        PseudoId::kPseudoIdNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1611-1613
    pub fn InitialPseudoElementStyles() -> u32 {
        PseudoId::kPseudoIdNone.value() as u32
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1615-1617
    pub fn InitialHasNonUniversalHighlightPseudoStyles() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1619-1621
    pub fn InitialHasNonUaHighlightPseudoStyles() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1623-1625
    pub fn InitialHighlightsDependOnSizeContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1627-1629
    pub fn InitialCustomHighlightNames() -> Option<*mut HashSet<AtomicString>> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1631-1633
    pub fn InitialVerticalAlignLength() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1635-1637
    pub fn InitialBorderImage() -> NinePieceImage {
        NinePieceImage::new()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1639-1641
    pub fn InitialHasClipPath() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1643-1645
    pub fn InitialHasAutoClip() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1647-1649
    pub fn InitialHasAutoZIndex() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1653-1655
    pub fn InitialContainerFont() -> Option<*mut Font> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1657-1659
    pub fn InitialCursorData() -> Option<*mut CursorList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1661-1663
    pub fn InitialEffectiveZoom() -> f32 {
        1.0
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1665-1667
    pub fn InitialTextEmphasisFill() -> TextEmphasisFill {
        TextEmphasisFill::kFilled
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1669-1671
    pub fn InitialTextEmphasisMark() -> TextEmphasisMark {
        TextEmphasisMark::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1673-1675
    pub fn InitialTextIndentFlags() -> TextIndentFlags {
        TextIndentFlags::kDefault
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1677-1679
    pub fn InitialSubtreeWillChangeContents() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1681-1683
    pub fn InitialSubtreeIsSticky() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1685-1687
    pub fn InitialIsInShrinkToFitSubtree() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1689-1691
    pub fn InitialEffectiveTouchAction() -> TouchAction {
        TouchAction::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1693-1695
    pub fn InitialEffectiveAppearance() -> AppearanceValue {
        AppearanceValue::kNone
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1697-1699
    pub fn InitialTextEmphasisCustomMark() -> AtomicString {
        AtomicString::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1701-1703
    pub fn InitialBaseTextDecorationData() -> Option<*mut AppliedTextDecorationVector> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1705-1707
    pub fn InitialInheritedVariables() -> StyleInheritedVariables {
        StyleInheritedVariables::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1709-1711
    pub fn InitialHighlightData() -> StyleHighlightData {
        StyleHighlightData::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1713-1715
    pub fn InitialInitialData() -> Option<*mut StyleInitialData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1717-1719
    pub fn InitialCounterDirectives() -> Option<*mut CounterDirectiveMap> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1721-1723
    pub fn InitialCounterIncrementList() -> Option<*mut CounterPropertyList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1725-1727
    pub fn InitialCounterResetList() -> Option<*mut CounterPropertyList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1729-1731
    pub fn InitialCounterSetList() -> Option<*mut CounterPropertyList> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1733-1735
    pub fn InitialAnimations() -> Option<*mut CSSAnimationData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1737-1739
    pub fn InitialTransitions() -> Option<*mut CSSTransitionData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1741-1743
    pub fn InitialMaskBoxImage() -> NinePieceImage {
        NinePieceImage::MaskDefaults()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1745-1747
    pub fn InitialUnconditionalScrollbarSize() -> gfx::Size {
        gfx::Size::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1749-1751
    pub fn InitialPageSize() -> gfx::SizeF {
        gfx::SizeF::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1753-1755
    pub fn InitialOutlineStyleIsAuto() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1757-1759
    pub fn InitialCallbackSelectors() -> Vector<String> {
        Vector::<String>::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1761-1763
    pub fn InitialDocumentRulesSelectors() -> Option<*mut GCedHeapHashSet<WeakMember<StyleRule>>> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1765-1767
    pub fn InitialPaintImages() -> Option<*mut PaintImages> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1769-1771
    pub fn InitialNonInheritedVariables() -> StyleNonInheritedVariables {
        StyleNonInheritedVariables::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1773-1775
    pub fn InitialPageSizeType() -> PageSizeType {
        PageSizeType::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1777-1779
    pub fn InitialHasCurrentOpacityAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1781-1783
    pub fn InitialHasCurrentTranslateAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1785-1787
    pub fn InitialHasCurrentRotateAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1789-1791
    pub fn InitialHasCurrentScaleAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1793-1795
    pub fn InitialHasCurrentTransformAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1797-1799
    pub fn InitialHasCurrentFilterAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1801-1803
    pub fn InitialHasCurrentBackdropFilterAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1805-1807
    pub fn InitialHasCurrentClipPathAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1809-1811
    pub fn InitialHasCurrentBackgroundColorAnimation() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1813-1815
    pub fn InitialIsRunningOpacityAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1817-1819
    pub fn InitialIsRunningTransformAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1821-1823
    pub fn InitialIsRunningScaleAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1825-1827
    pub fn InitialIsRunningRotateAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1829-1831
    pub fn InitialIsRunningTranslateAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1833-1835
    pub fn InitialIsRunningFilterAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1837-1839
    pub fn InitialIsRunningBackdropFilterAnimationOnCompositor() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1841-1843
    pub fn InitialForcesStackingContext() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1845-1847
    pub fn InitialRequiresAcceleratedCompositingForExternalReasons() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1849-1851
    pub fn InitialColorIsCurrentColor() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1853-1855
    pub fn InitialInternalVisitedColorIsCurrentColor() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1857-1859
    pub fn InitialHasAutoColumnWidth() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1861-1863
    pub fn InitialHasAutoColumnHeight() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1865-1867
    pub fn InitialHasAutoColumnCount() -> bool {
        true
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1869-1871
    pub fn InitialDarkColorScheme() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1873-1875
    pub fn InitialColorSchemeFlagsIsNormal() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1877-1879
    pub fn InitialColorSchemeForced() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1881-1883
    pub fn InitialInlineBlockBaselineEdge() -> EInlineBlockBaselineEdge {
        EInlineBlockBaselineEdge::kMarginBox
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1885-1887
    pub fn InitialMathBaseline() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1889-1891
    pub fn InitialMathFractionBarThickness() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1893-1895
    pub fn InitialMathLSpace() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1897-1899
    pub fn InitialMathRSpace() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1901-1903
    pub fn InitialMathPaddedVOffset() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1905-1907
    pub fn InitialMathPaddedDepth() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1909-1911
    pub fn InitialMathMinSize() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1913-1915
    pub fn InitialMathMaxSize() -> Length {
        Length::default()
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1917-1919
    pub fn InitialAllowsZIndex() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1921-1923
    pub fn InitialBaselineShiftType() -> EBaselineShiftType {
        EBaselineShiftType::kLength
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1925-1927
    pub fn InitialCssDominantBaseline() -> EDominantBaseline {
        EDominantBaseline::kAuto
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1929-1931
    pub fn InitialDependsOnSizeContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1933-1935
    pub fn InitialDependsOnStyleContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1937-1939
    pub fn InitialDependsOnScrollStateContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1941-1943
    pub fn InitialDependsOnAnchoredContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1945-1947
    pub fn InitialFirstLineDependsOnSizeContainerQueries() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1949-1951
    pub fn InitialBaseData() -> Option<*mut StyleBaseData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1953-1955
    pub fn InitialUAShadowHostData() -> Option<*mut StyleUAShadowHostData> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1957-1959
    pub fn InitialSkipsContents() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1961-1963
    pub fn InitialHasSizeContainmentForViewTransitionScope() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1965-1967
    pub fn InitialIsHTMLInert() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1969-1971
    pub fn InitialIsCSSInert() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1973-1975
    pub fn InitialInlineStyleLostCascade() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1977-1979
    pub fn InitialPositionAreaOffsets() -> Option<PositionAreaOffsets> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1981-1983
    pub fn InitialAnchorCenterOffset() -> Option<PhysicalOffset> {
        None
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1985-1987
    pub fn InitialPrefersDefaultScrollbarStyles() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1989-1991
    pub fn InitialIsPageMarginBox() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1993-1995
    pub fn InitialInBaseAppearance() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1997-1999
    pub fn InitialIsBottomRelativeToSafeAreaInset() -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:2001-2003
    pub fn InitialContinue() -> EContinue {
        EContinue::kNormal
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:2005-2007
    pub fn InitialMaxLines() -> MaxLinesData {
        MaxLinesData::new(0, true)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:2009-2011
    pub fn InitialLineClampInternalBlockEllipsis() -> EBlockEllipsis {
        EBlockEllipsis::kNoEllipsis
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:2014-2020
    fn InitialFilterInternal() -> &'static FilterOperations {
        thread_local! {
            static OPS: OnceCell<&'static Persistent<FilterOperationsWrapper>> = OnceCell::new();
        }
        OPS.with(|slot| {
            let root = slot.get_or_init(|| {
                Box::leak(Box::new(Persistent::from_ptr(MakeGarbageCollected(
                    FilterOperationsWrapper::default(),
                ))))
            });
            unsafe { (&*root.Get()).Operations() }
        })
    }
}
