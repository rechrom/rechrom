//! Projection of immutable Blink computed values into Rechrom's neutral
//! DOM-to-layout snapshot. Layout keeps using the native `ComputedStyle`;
//! this projection supplies change classification and Web API snapshots.

use dom::persistent_document::ResolvedNodeStyle;
use foundation::Length;
use layoutng::internal::layout_input::{
    ExtendedStyle, FlexDirection, TextDirection as LayoutTextDirection,
    WritingMode as LayoutWritingMode,
};
use layoutng_style::style::{
    computed_style::ComputedStyle, computed_style_constants::FlexWrapMode,
};

#[derive(Default)]
struct ProjectedLength {
    pixels: Option<f64>,
    percent: Option<f64>,
    calculated: bool,
    auto: bool,
}

// cpp: foundation/blink_geometry/geometry/length.h:213-280
// cpp: foundation/blink_geometry/geometry/length.cc:138-157
fn ProjectLength(value: &Length) -> ProjectedLength {
    if value.IsAuto() {
        return ProjectedLength {
            auto: true,
            ..Default::default()
        };
    }
    if value.IsFixed() {
        return ProjectedLength {
            pixels: Some(value.Pixels() as f64),
            ..Default::default()
        };
    }
    if value.IsPercent() {
        return ProjectedLength {
            percent: Some(value.PercentValue() as f64),
            ..Default::default()
        };
    }
    if value.IsCalculated() {
        let calculation = value.GetCalculationValue();
        if !calculation.IsExpression() {
            let pair = calculation.GetPixelsAndPercent();
            return ProjectedLength {
                pixels: pair.has_explicit_pixels.then_some(pair.pixels as f64),
                percent: pair.has_explicit_percent.then_some(pair.percent as f64),
                calculated: true,
                auto: false,
            };
        }
        return ProjectedLength {
            calculated: true,
            ..Default::default()
        };
    }
    ProjectedLength::default()
}

fn ProjectDimension(
    value: &Length,
    pixels: &mut Option<f64>,
    percent: &mut Option<f64>,
    calculated: &mut bool,
) {
    let value = ProjectLength(value);
    *pixels = value.pixels;
    *percent = value.percent;
    *calculated = value.calculated;
}

fn ProjectEdge(
    value: &Length,
    pixels: &mut f64,
    percent: &mut Option<f64>,
    calculated: &mut bool,
    auto: Option<&mut bool>,
) {
    let value = ProjectLength(value);
    *pixels = value.pixels.unwrap_or(0.0);
    *percent = value.percent;
    *calculated = value.calculated;
    if let Some(auto) = auto {
        *auto = value.auto;
    }
}

// cpp: layoutng_style/style/computed_style_base.h Width/Height/Margin*/Padding*/Inset
// cpp: core/css/resolver/style_resolver.cc ApplyProperties
pub(crate) fn ProjectGeometry(
    native: &ComputedStyle,
    resolved: &mut ResolvedNodeStyle,
    ext: &mut ExtendedStyle,
) {
    ProjectDimension(
        native.Width(),
        &mut resolved.style.width,
        &mut ext.width_percent,
        &mut ext.width_calculated,
    );
    ProjectDimension(
        native.Height(),
        &mut resolved.style.height,
        &mut ext.height_percent,
        &mut ext.height_calculated,
    );
    ProjectDimension(
        native.MinWidth(),
        &mut resolved.style.min_width,
        &mut ext.min_width_percent,
        &mut ext.min_width_calculated,
    );
    ProjectDimension(
        native.MaxWidth(),
        &mut resolved.style.max_width,
        &mut ext.max_width_percent,
        &mut ext.max_width_calculated,
    );
    ProjectDimension(
        native.MinHeight(),
        &mut resolved.style.min_height,
        &mut ext.min_height_percent,
        &mut ext.min_height_calculated,
    );
    ProjectDimension(
        native.MaxHeight(),
        &mut resolved.style.max_height,
        &mut ext.max_height_percent,
        &mut ext.max_height_calculated,
    );
    ProjectDimension(
        native.FlexBasis(),
        &mut resolved.style.flex_basis,
        &mut ext.flex_basis_percent,
        &mut ext.flex_basis_calculated,
    );

    let margins = [
        native.MarginTop(),
        native.MarginRight(),
        native.MarginBottom(),
        native.MarginLeft(),
    ];
    let margin_pixels = [
        &mut resolved.style.margin.top,
        &mut resolved.style.margin.right,
        &mut resolved.style.margin.bottom,
        &mut resolved.style.margin.left,
    ];
    for (index, (value, pixels)) in margins.into_iter().zip(margin_pixels).enumerate() {
        ProjectEdge(
            value,
            pixels,
            &mut ext.margin_percentages[index],
            &mut ext.margin_calculated[index],
            Some(&mut ext.margin_auto[index]),
        );
    }

    let paddings = [
        native.PaddingTop(),
        native.PaddingRight(),
        native.PaddingBottom(),
        native.PaddingLeft(),
    ];
    let padding_pixels = [
        &mut resolved.style.padding.top,
        &mut resolved.style.padding.right,
        &mut resolved.style.padding.bottom,
        &mut resolved.style.padding.left,
    ];
    for (index, (value, pixels)) in paddings.into_iter().zip(padding_pixels).enumerate() {
        ProjectEdge(
            value,
            pixels,
            &mut ext.padding_percentages[index],
            &mut ext.padding_calculated[index],
            None,
        );
    }

    let insets = [native.Top(), native.Right(), native.Bottom(), native.Left()];
    let inset_pixels = [
        &mut resolved.style.top,
        &mut resolved.style.right,
        &mut resolved.style.bottom,
        &mut resolved.style.left,
    ];
    for (index, (value, pixels)) in insets.into_iter().zip(inset_pixels).enumerate() {
        let projected = ProjectLength(value);
        *pixels = projected.pixels;
        ext.inset_percentages[index] = projected.percent;
        ext.inset_calculated[index] = projected.calculated;
    }

    resolved.style.flex_grow = native.FlexGrow();
    resolved.style.flex_shrink = native.FlexShrink();
    resolved.style.flex_direction = match native.FlexDirection() {
        foundation::EFlexDirection::kRow => FlexDirection::kRow,
        foundation::EFlexDirection::kRowReverse => FlexDirection::kRowReverse,
        foundation::EFlexDirection::kColumn => FlexDirection::kColumn,
        foundation::EFlexDirection::kColumnReverse => FlexDirection::kColumnReverse,
    };
    resolved.style.flex_wrap = native.FlexWrap().GetWrapMode() != FlexWrapMode::kNowrap;
    ext.wrap_reverse = native.FlexWrap().GetWrapMode() == FlexWrapMode::kWrapReverse;
    resolved.style.direction = match native.Direction() {
        foundation::TextDirection::kLtr => LayoutTextDirection::kLtr,
        foundation::TextDirection::kRtl => LayoutTextDirection::kRtl,
    };
    resolved.style.writing_mode = match native.GetWritingMode() {
        foundation::WritingMode::kHorizontalTb => LayoutWritingMode::kHorizontalTb,
        foundation::WritingMode::kVerticalRl | foundation::WritingMode::kSidewaysRl => {
            LayoutWritingMode::kVerticalRl
        }
        foundation::WritingMode::kVerticalLr | foundation::WritingMode::kSidewaysLr => {
            LayoutWritingMode::kVerticalLr
        }
    };
    resolved.style.column_count = native.ColumnCount() as u32;

    let line_height = ProjectLength(native.LineHeight());
    ext.line_height = line_height.pixels;
    ext.line_height_percent = line_height.percent;
    let text_indent = ProjectLength(native.TextIndent());
    ext.text_indent = text_indent.pixels.unwrap_or(0.0);
    ext.text_indent_percent = text_indent.percent;
    ext.text_indent_calculated = text_indent.calculated;

    let row_gap = native.RowGap().as_ref().map(ProjectLength);
    let column_gap = native.ColumnGap().as_ref().map(ProjectLength);
    if let Some(value) = row_gap {
        ext.row_gap = value.pixels;
        ext.row_gap_percent = value.percent;
        ext.row_gap_calculated = value.calculated;
    }
    if let Some(value) = column_gap {
        ext.column_gap = value.pixels;
        ext.column_gap_percent = value.percent;
        ext.column_gap_calculated = value.calculated;
    }
    if ext.row_gap == ext.column_gap && ext.row_gap_percent == ext.column_gap_percent {
        resolved.style.gap = ext.row_gap.unwrap_or(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_style::style::computed_style::ComputedStyleBuilder;

    #[test]
    fn projects_fixed_percent_auto_and_calc_geometry_without_reparsing_css() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetWidthOwned(Length::Fixed(42.0));
        builder.SetHeightOwned(Length::Percent(25.0));
        builder.SetMarginTop(Length::Auto());
        builder.SetPaddingLeft(&Length::Fixed(7.0));
        builder.SetLeftOwned(Length::Percent(10.0));
        let native = unsafe { &*builder.TakeStyle() };
        let mut resolved = ResolvedNodeStyle::default();
        let mut ext = ExtendedStyle::default();
        ProjectGeometry(native, &mut resolved, &mut ext);
        assert_eq!(resolved.style.width, Some(42.0));
        assert_eq!(ext.height_percent, Some(25.0));
        assert!(ext.margin_auto[0]);
        assert_eq!(resolved.style.padding.left, 7.0);
        assert_eq!(ext.inset_percentages[3], Some(10.0));
    }
}
