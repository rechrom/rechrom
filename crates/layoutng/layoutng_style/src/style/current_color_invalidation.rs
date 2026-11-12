//! ComputedStyle::HasPropertyDependingOnCurrentColor cc2317-2327 and
//! generated kCSSIncludesCurrentColorProperties (29), dispatching the source
//! longhands' IsAffectedByCurrentColor virtuals over the same retained style.
use super::{computed_style::ComputedStyle, gap_data_list::GapDataList, style_image::StyleImage};
use crate::css::style_color::StyleColor;
fn image_uses_current_color(image: *const StyleImage) -> bool {
    unsafe { image.as_ref() }.is_some_and(StyleImage::DependsOnCurrentColor)
}
// CSSGapDecorationUtils::RuleColorMaybeDependsOnCurrentColor cc53-69.
fn rule_color(colors: &GapDataList<StyleColor>) -> bool {
    colors.GetGapDataList().iter().any(|data| {
        if !data.IsRepeaterData() {
            return data.GetValue().DependsOnCurrentColor();
        }
        data.GetValueRepeater()
            .RepeatedValues()
            .iter()
            .any(StyleColor::DependsOnCurrentColor)
    })
}
impl ComputedStyle {
    pub(crate) fn HasPropertyDependingOnCurrentColorSource(&self) -> bool {
        // Keep the generated property's source order and short-circuit behavior.
        self.AccentColor().DependsOnCurrentColor()
            || (self.HasBackdropFilter() && self.BackdropFilter().UsesCurrentColor())
            || self.BackgroundColor().DependsOnCurrentColor()
            || self.BackgroundInternal().AnyLayerUsesCurrentColor()
            || (self.BorderBottomWidth() != 0 && self.BorderBottomColor().DependsOnCurrentColor())
            || image_uses_current_color(self.BorderImageSource())
            || (self.BorderLeftWidth() != 0 && self.BorderLeftColor().DependsOnCurrentColor())
            || (self.BorderRightWidth() != 0 && self.BorderRightColor().DependsOnCurrentColor())
            || (self.BorderTopWidth() != 0 && self.BorderTopColor().DependsOnCurrentColor())
            || Self::ShadowListHasCurrentColor(self.BoxShadow())
            || self.CaretColor().DependsOnCurrentColor()
            || (self.HasGapRule() && rule_color(self.ColumnRuleColor()))
            || (self.FillPaint().HasColor() && self.FillPaint().GetColor().DependsOnCurrentColor())
            || self.Filter().UsesCurrentColor()
            || self.FloodColor().DependsOnCurrentColor()
            || self.LightingColor().DependsOnCurrentColor()
            || image_uses_current_color(self.ListStyleImage().Get())
            || self.MaskLayers().AnyLayerUsesCurrentColor()
            || self.HasOutlineWithCurrentColor()
            || (self.HasGapRule() && rule_color(self.RowRuleColor()))
            || self.StopColor().DependsOnCurrentColor()
            || (self.StrokePaint().HasColor()
                && self.StrokePaint().GetColor().DependsOnCurrentColor())
            || (self.HasAppliedTextDecorations()
                && self.TextDecorationColor().DependsOnCurrentColor())
            || (self.GetTextEmphasisMark() != foundation::TextEmphasisMark::kNone
                && self.TextEmphasisColor().DependsOnCurrentColor())
            || Self::ShadowListHasCurrentColor(self.TextShadow())
            || unsafe { self.BoxReflect().as_ref() }.is_some_and(|reflection| {
                reflection.Mask().HasImage()
                    && image_uses_current_color(reflection.Mask().GetImage())
            })
            || image_uses_current_color(self.MaskBoxImageSource())
            || self.TapHighlightColor().DependsOnCurrentColor()
            || (self.TextStrokeWidth() != 0.0 && self.TextStrokeColor().DependsOnCurrentColor())
    }
}
