pub use style_parser::production_property_parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::css_parser_mode::CSSParserMode;
    use crate::{media_queries::MediaValuesCachedData, resolver::production_style_builder::Apply};
    use foundation::CSSPropertyID;
    use layoutng_style::style::{
        computed_style::{ComputedStyle, ComputedStyleBuilder},
        computed_style_constants::{
            CompositingOperator, EFillBox, EFillMaskMode, EFillRepeat, EFillSizeType,
        },
        nine_piece_image::ENinePieceImageRule,
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn snapshot(b: &ComputedStyleBuilder) -> &ComputedStyle {
        unsafe { &*b.CloneStyle() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    #[test]
    fn production_mask_image_gradient_layers_and_clear() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "mask-image:linear-gradient(to bottom,red,blue),none,linear-gradient(red,blue)",
        );
        let l = snapshot(&b).MaskLayers();
        assert!(!l.GetImage().is_null());
        assert!(unsafe { (*l.GetImage()).IsGeneratedImage() });
        let second = unsafe { &*l.Next() };
        assert!(second.GetImage().is_null() && second.IsImageSet());
        let third = unsafe { &*second.Next() };
        assert!(!third.GetImage().is_null());
        apply(&mut b, None, "mask-image:none");
        let l = snapshot(&b).MaskLayers();
        assert!(l.GetImage().is_null() && l.IsImageSet());
        assert!(!unsafe { &*l.Next() }.IsImageSet());
    }
    #[test]
    fn production_mask_shorthand_fields_layers_alignment_and_resets() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"-webkit-mask-box-image:20 / 3px;mask:linear-gradient(red,blue) left top / 25% 10px no-repeat padding-box content-box subtract luminance,none right bottom / cover repeat-x fill-box alpha");
        let s = snapshot(&b);
        let a = s.MaskLayers();
        let c = unsafe { &*a.Next() };
        assert_eq!(a.PositionX().PercentValue(), 0.0);
        assert_eq!(c.PositionX().PercentValue(), 100.0);
        assert_eq!(a.SizeLength().Width().PercentValue(), 25.0);
        assert_eq!(a.SizeLength().Height().Pixels(), 10.0);
        assert_eq!(a.Origin(), EFillBox::kPadding);
        assert_eq!(a.Clip(), EFillBox::kContent);
        assert_eq!(a.CompositingOperator(), CompositingOperator::kSubtract);
        assert_eq!(a.MaskMode(), EFillMaskMode::kLuminance);
        assert_eq!(c.SizeType(), EFillSizeType::kCover);
        assert_eq!(c.Origin(), EFillBox::kFillBox);
        assert_eq!(c.Clip(), EFillBox::kFillBox);
        assert_eq!(c.Repeat().x, EFillRepeat::kRepeatFill);
        assert_eq!(c.Repeat().y, EFillRepeat::kNoRepeatFill);
        assert_eq!(c.MaskMode(), EFillMaskMode::kAlpha);
        assert_eq!(c.CompositingOperator(), CompositingOperator::kAdd);
        // Literal source defaults reset all nine layer fields, leaving mask-box.
        apply(&mut b, None, "mask:none");
        let s = snapshot(&b);
        let a = s.MaskLayers();
        assert!(a.GetImage().is_null());
        assert_eq!(a.SizeType(), EFillSizeType::kSizeLength);
        assert!(a.SizeLength().Width().IsAuto() && a.SizeLength().Height().IsAuto());
        assert_eq!(a.Origin(), EFillBox::kBorder);
        assert_eq!(a.Clip(), EFillBox::kBorder);
        assert_eq!(a.CompositingOperator(), CompositingOperator::kAdd);
        assert_eq!(a.MaskMode(), EFillMaskMode::kMatchSource);
        assert!(!unsafe { &*a.Next() }.IsSizeSet());
        assert_eq!(b.MaskBoxImage().ImageSlices().Top().Pixels(), 20.0);
        assert_eq!(b.MaskBoxImage().BorderSlices().Top().length().Pixels(), 3.0);
        // Native FillUnsetProperties cycles independent longhand lists.
        apply(
            &mut b,
            None,
            "mask-image:none,none,none;mask-mode:alpha,luminance;mask-size:contain",
        );
        b.AdjustMaskLayers();
        let a = snapshot(&b).MaskLayers();
        let c = unsafe { &*a.Next() };
        let d = unsafe { &*c.Next() };
        assert_eq!(d.MaskMode(), EFillMaskMode::kAlpha);
        assert_eq!(d.SizeType(), EFillSizeType::kContain);
    }
    #[test]
    fn production_mask_box_shared_nine_piece_defaults_gradient_and_reset() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"-webkit-mask-box-image:linear-gradient(red,blue) 10% 2.5 / 2 3px 25% auto / 1 4px round space");
        let image = b.MaskBoxImage();
        assert!(image.Fill());
        assert!(!image.GetImage().is_null());
        assert_eq!(image.ImageSlices().Top().PercentValue(), 10.0);
        assert_eq!(image.ImageSlices().Right().Pixels(), 3.0);
        assert_eq!(image.BorderSlices().Top().Number(), 2.0);
        assert!(image.BorderSlices().Left().length().IsAuto());
        assert_eq!(image.Outset().Right().length().Pixels(), 4.0);
        assert!(image.HorizontalRule() == ENinePieceImageRule::kRoundImageRule);
        assert!(image.VerticalRule() == ENinePieceImageRule::kSpaceImageRule);
        // Unlike legacy border-image, mask-box never changes physical widths.
        assert_eq!(b.BorderTopWidth(), initial().BorderTopWidth());
        apply(&mut b, None, "-webkit-mask-box-image:none");
        let image = b.MaskBoxImage();
        assert!(image.GetImage().is_null() && image.Fill());
        assert_eq!(image.ImageSlices().Top().Pixels(), 0.0);
        assert!(image.BorderSlices().Top().length().IsAuto());
        assert_eq!(image.Outset().Top().Number(), 0.0);
        assert!(image.HorizontalRule() == ENinePieceImageRule::kStretchImageRule);
        apply(
            &mut b,
            None,
            "-webkit-mask-box-image-slice:calc(2 + 3);-webkit-mask-box-image:20 // 4",
        );
        assert_eq!(b.MaskBoxImage().Outset().Top().Number(), 4.0);
        assert!(b.MaskBoxImage().BorderSlices().Top().length().IsAuto());
    }
    #[test]
    fn production_mask_css_wide_inherits_and_initial_real_native() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(&mut parent,None,"mask:linear-gradient(red,blue) right bottom / contain space view-box no-clip exclude alpha;-webkit-mask-box-image:15 / 2 / 3 round");
        let parent = snapshot(&parent);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut child,
            Some(parent),
            "mask:inherit;-webkit-mask-box-image:inherit",
        );
        let a = snapshot(&child).MaskLayers();
        assert_eq!(a.GetImage(), parent.MaskLayers().GetImage());
        assert_eq!(a.SizeType(), EFillSizeType::kContain);
        assert_eq!(a.Clip(), EFillBox::kNoClip);
        assert_eq!(a.CompositingOperator(), CompositingOperator::kExclude);
        assert_eq!(child.MaskBoxImage().Outset().Top().Number(), 3.0);
        apply(
            &mut child,
            Some(parent),
            "mask:unset;-webkit-mask-box-image:initial",
        );
        assert!(snapshot(&child).MaskLayers().GetImage().is_null());
        assert!(child.MaskBoxImage().BorderSlices().Top().length().IsAuto());
        apply(
            &mut child,
            None,
            "mask:inherit;-webkit-mask-box-image:inherit",
        );
        assert!(snapshot(&child).MaskLayers().GetImage().is_null());
        // Prefix aliases share the production resolved-ID path and native fields.
        apply(&mut child,None,"-webkit-mask:none center / contain border text add alpha;-webkit-mask-composite:source-in");
        assert_eq!(snapshot(&child).MaskLayers().Clip(), EFillBox::kText);
        assert_eq!(
            snapshot(&child).MaskLayers().CompositingOperator(),
            CompositingOperator::kSourceIn
        );
    }
    #[test]
    fn production_mask_invalid_and_resource_owner_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        for (name, value) in [
            ("mask", "none / cover"),
            ("mask", "none left repeat / cover"),
            ("mask", "none,,none"),
            ("mask", "none red"),
            ("mask", "none padding-box content-box border-box"),
            ("-webkit-mask-box-image", "20 /"),
            ("-webkit-mask-box-image-slice", "fill fill 10"),
            ("-webkit-mask-box-image-width", "-1"),
            ("-webkit-mask-box-image-outset", "20%"),
            ("mask-image", "none none"),
        ] {
            assert!(
                ParseDeclarationList(
                    &foundation::String::from(format!("{name}:{value}")),
                    CSSParserMode::kHTMLStandardMode
                )
                .properties
                .is_empty(),
                "{name}:{value}"
            );
        }
        for id in [
            CSSPropertyID::kMaskImage,
            CSSPropertyID::kWebkitMaskBoxImageSource,
        ] {
            let p = ParseProperty(
                id,
                &foundation::String::from("url(mask.png)"),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            let mut b = ComputedStyleBuilder::from_style(initial());
            assert!(
                matches!(Apply(id,&mut b,None,p[0].Value(),16.0,&MediaValuesCachedData::default()),Err(crate::properties::longhand_dispatch::LonghandApplicationError::Unsupported(i)) if i==id)
            );
            assert!(
                snapshot(&b).MaskLayers().GetImage().is_null()
                    && b.MaskBoxImage().GetImage().is_null()
            );
        }
    }
}
