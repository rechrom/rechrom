// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/css/properties/longhands/longhands_custom.cc (editing subset;
// individual method ranges below); shorthands/shorthands_custom.cc:5245-5300.
//! CSSProperty's typed resolved-value dispatch for editing. Unsupported
//! properties remain explicit rather than producing a partial style snapshot.
use crate::{
    computed_style_utils as utils,
    production_css_value::{self as values, Value},
};
use foundation::{
    CSSPropertyID as P, CSSValueID as V, EDisplay, EFloat, ETextAlign, ETextDecorationStyle,
    ETextTransform, IsLtr, TextWrapMode, UnicodeBidi,
};
use layoutng_style::{
    css::{style_color::StyleColor, white_space::WhiteSpaceCollapse},
    style::{computed_style::ComputedStyle, computed_style_constants::EVerticalAlign},
};
use std::rc::Rc;
use style_model::{
    css_primitive_value::UnitType, css_value::CSSValuePayload,
    properties::css_property::CSSValuePhase,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleValueError {
    UnsupportedProperty(P),
    ContextDependentExposure(P),
    VisitedColor(P),
}
#[derive(Clone, Copy, Debug)]
pub struct ComputedStyleValueOptions {
    pub allow_visited_style: bool,
    pub value_phase: CSSValuePhase,
    pub letter_and_word_spacing_percentage: bool,
}
impl Default for ComputedStyleValueOptions {
    fn default() -> Self {
        Self {
            allow_visited_style: false,
            value_phase: CSSValuePhase::kResolvedValue,
            // runtime_enabled_features.json5:1707-1709 (stable).
            letter_and_word_spacing_percentage: true,
        }
    }
}
fn Resolved(options: ComputedStyleValueOptions) -> bool {
    options.value_phase == CSSValuePhase::kResolvedValue
}
fn Ident(id: V) -> Rc<Value> {
    values::identifier(id)
}
fn List(items: Vec<Rc<Value>>) -> Rc<Value> {
    values::list(items, values::ListSeparator::Space)
}

pub fn CSSValueFromComputedStyle(
    property: P,
    style: &ComputedStyle,
    options: ComputedStyleValueOptions,
) -> Result<Rc<Value>, ComputedStyleValueError> {
    // CSSProperty::CSSValueFromComputedStyle cc60-67 resolves logical and
    // surrogate properties against this exact style's writing direction.
    let property = crate::longhand_dispatch::ResolvePhysical(property, style.GetWritingDirection());
    Ok(match property {
        P::kBackfaceVisibility => Ident(
            if style.BackfaceVisibility() == foundation::EBackfaceVisibility::kHidden {
                V::kHidden
            } else {
                V::kVisible
            },
        ),
        P::kBorderCollapse => Ident(
            if style.BorderCollapse() == foundation::EBorderCollapse::kCollapse {
                V::kCollapse
            } else {
                V::kSeparate
            },
        ),
        P::kBoxSizing => Ident(
            if style.BoxSizing() == foundation::EBoxSizing::kContentBox {
                V::kContentBox
            } else {
                V::kBorderBox
            },
        ),
        // longhands_custom.cc:2576-2587.
        P::kColumnCount => {
            if style.HasAutoColumnCount() {
                Ident(V::kAuto)
            } else {
                values::numeric(style.ColumnCount() as f64, UnitType::kNumber)
            }
        }
        // longhands_custom.cc:6566-6578; computed_style_utils.cc:989-1014.
        P::kLineHeight => {
            let length = style.LineHeight();
            if length.IsAuto() {
                Ident(V::kNormal)
            } else if !Resolved(options) && length.IsPercent() {
                values::numeric(length.PercentValue() as f64 / 100.0, UnitType::kNumber)
            } else {
                utils::ZoomAdjustedPixelValue(
                    foundation::FloatValueForLength(
                        length,
                        style.GetFontDescription().ComputedSize() as f32,
                    ) as f64,
                    style,
                )
            }
        }
        P::kBorderTopColor
        | P::kBorderRightColor
        | P::kBorderBottomColor
        | P::kBorderLeftColor
        | P::kOutlineColor => {
            if options.allow_visited_style {
                return Err(ComputedStyleValueError::VisitedColor(property));
            }
            let color = match property {
                P::kBorderTopColor => style.BorderTopColor(),
                P::kBorderRightColor => style.BorderRightColor(),
                P::kBorderBottomColor => style.BorderBottomColor(),
                P::kBorderLeftColor => style.BorderLeftColor(),
                P::kOutlineColor => style.OutlineColor(),
                _ => unreachable!(),
            };
            if Resolved(options) && style.ShouldForceColor(color) {
                let forced = if property == P::kOutlineColor {
                    style.InternalForcedOutlineColor()
                } else {
                    style.InternalForcedBorderColor()
                };
                values::color(forced.Resolve(
                    style.GetInternalForcedCurrentColor(None),
                    style.UsedColorScheme(),
                    None,
                ))
            } else {
                utils::CurrentColorOrValidColor(style, color)
            }
        }
        // longhands_custom.cc:1425-1436,1664-1675,1751-1762,1838-1849.
        P::kBorderTopWidth | P::kBorderRightWidth | P::kBorderBottomWidth | P::kBorderLeftWidth => {
            let width = match (property, Resolved(options)) {
                (P::kBorderTopWidth, true) => style.BorderTopWidth(),
                (P::kBorderTopWidth, false) => *style.SpecifiedBorderTopWidth(),
                (P::kBorderRightWidth, true) => style.BorderRightWidth(),
                (P::kBorderRightWidth, false) => *style.SpecifiedBorderRightWidth(),
                (P::kBorderBottomWidth, true) => style.BorderBottomWidth(),
                (P::kBorderBottomWidth, false) => *style.SpecifiedBorderBottomWidth(),
                (P::kBorderLeftWidth, true) => style.BorderLeftWidth(),
                (P::kBorderLeftWidth, false) => *style.SpecifiedBorderLeftWidth(),
                _ => unreachable!(),
            };
            utils::ZoomAdjustedPixelValue(width as f64, style)
        }
        P::kBorderTopLeftRadius
        | P::kBorderTopRightRadius
        | P::kBorderBottomLeftRadius
        | P::kBorderBottomRightRadius => {
            let radius = match property {
                P::kBorderTopLeftRadius => style.BorderTopLeftRadius(),
                P::kBorderTopRightRadius => style.BorderTopRightRadius(),
                P::kBorderBottomLeftRadius => style.BorderBottomLeftRadius(),
                P::kBorderBottomRightRadius => style.BorderBottomRightRadius(),
                _ => unreachable!(),
            };
            Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                values::CSSValuePair {
                    first: utils::ZoomAdjustedPixelValueForLength(radius.Width(), style),
                    second: utils::ZoomAdjustedPixelValueForLength(radius.Height(), style),
                    drop_identical: true,
                },
            )))
        }
        // longhands_custom.cc:164-220,337-362,7475-7497; identical
        // helpers serve the justify family (cc:6126-6168).
        P::kPosition => Ident(match style.PositionForComputedValue() {
            foundation::EPosition::kStatic => V::kStatic,
            foundation::EPosition::kRelative => V::kRelative,
            foundation::EPosition::kAbsolute => V::kAbsolute,
            foundation::EPosition::kFixed => V::kFixed,
            foundation::EPosition::kSticky => V::kSticky,
        }),
        P::kScrollbarWidth => Ident(match style.UsedScrollbarWidth() {
            foundation::EScrollbarWidth::kAuto => V::kAuto,
            foundation::EScrollbarWidth::kThin => V::kThin,
            foundation::EScrollbarWidth::kNone => V::kNone,
        }),
        P::kScrollBehavior => {
            use layoutng_style::style::scroll_enums::mojom::ScrollBehavior as S;
            Ident(match style.GetScrollBehavior() {
                S::kAuto => V::kAuto,
                S::kSmooth => V::kSmooth,
                _ => unreachable!("canonical ScrollBehavior"),
            })
        }
        P::kAccentColor => {
            if style.AccentColor().IsAutoColor() {
                Ident(V::kAuto)
            } else {
                utils::ValueForStyleAutoColor(style, style.AccentColor())
            }
        }
        P::kAlignContent => {
            crate::computed_alignment_values::ContentAlignment(style.AlignContent())
        }
        P::kJustifyContent => {
            crate::computed_alignment_values::ContentAlignment(style.JustifyContent())
        }
        P::kAlignItems => crate::computed_alignment_values::SelfAlignment(style.AlignItems()),
        P::kAlignSelf => crate::computed_alignment_values::SelfAlignment(style.AlignSelf()),
        P::kJustifyItems => crate::computed_alignment_values::SelfAlignment(style.JustifyItems()),
        P::kJustifySelf => crate::computed_alignment_values::SelfAlignment(style.JustifySelf()),
        P::kAnchorName => crate::computed_alignment_values::NameList(style.AnchorName()),
        P::kAnchorScope => crate::computed_alignment_values::NameScope(style.AnchorScope()),
        // longhands_custom.cc:1097-1118,2151-2189,2362-2376.
        P::kBackgroundColor | P::kColor | P::kCaretColor => {
            if options.allow_visited_style {
                return Err(ComputedStyleValueError::VisitedColor(property));
            }
            match property {
                P::kBackgroundColor
                    if Resolved(options) && style.ShouldForceColor(style.BackgroundColor()) =>
                {
                    let mut color = style.InternalForcedBackgroundColor().Resolve(
                        style.GetInternalForcedCurrentColor(None),
                        style.UsedColorScheme(),
                        None,
                    );
                    let alpha = style
                        .BackgroundColor()
                        .Resolve(style.GetCurrentColor(None), style.UsedColorScheme(), None)
                        .Alpha();
                    color.SetAlpha(alpha);
                    values::color(color)
                }
                P::kBackgroundColor => {
                    utils::CurrentColorOrValidColor(style, style.BackgroundColor())
                }
                P::kColor => values::color(
                    if Resolved(options) && style.ShouldForceColor(style.Color()) {
                        style.GetInternalForcedCurrentColor(None)
                    } else {
                        style.GetCurrentColor(None)
                    },
                ),
                _ => {
                    let caret = style.CaretColor();
                    let current = StyleColor::CurrentColor();
                    let fill = if caret.FillColor().IsAutoColor() {
                        &current
                    } else {
                        caret.FillColor().ToStyleColor()
                    };
                    if Resolved(options) && style.ShouldForceColor(fill) {
                        values::color(style.GetInternalForcedCurrentColor(None))
                    } else {
                        let first = utils::ValueForStyleAutoColor(style, caret.FillColor());
                        if caret.TextColor().IsAutoColor() {
                            first
                        } else {
                            List(vec![
                                first,
                                utils::ValueForStyleAutoColor(style, caret.TextColor()),
                            ])
                        }
                    }
                }
            }
        }
        // longhands_custom.cc:4409-4416,4489-4512,4553-4560,4622-4629,4779-4786.
        P::kFontFamily => utils::ValueForFontFamily(style),
        P::kFontSize => {
            utils::ZoomAdjustedPixelValue(style.GetFontDescription().ComputedSize(), style)
        }
        P::kFontStyle => utils::ValueForFontStyle(style),
        P::kFontVariantCaps => utils::ValueForFontVariantCaps(style),
        P::kFontVariantLigatures => utils::ValueForFontVariantLigatures(style),
        P::kFontWeight => values::numeric(
            style.GetFontDescription().Weight().ToFloat() as f64,
            UnitType::kNumber,
        ),
        // longhands_custom.cc:6276-6297,12612-12626.
        P::kLetterSpacing | P::kWordSpacing => {
            let letter = property == P::kLetterSpacing;
            if options.letter_and_word_spacing_percentage {
                let length = if letter {
                    style.ComputedLetterSpacing()
                } else {
                    style.ComputedWordSpacing()
                };
                if length.IsFixed() {
                    if letter && length.IsZero() {
                        Ident(V::kNormal)
                    } else {
                        utils::ZoomAdjustedPixelValue(length.Pixels(), style)
                    }
                } else {
                    Value::Create(length, style.Zoom())
                }
            } else {
                let spacing = if letter {
                    style.LetterSpacing()
                } else {
                    style.WordSpacing()
                };
                if letter && spacing == 0.0 {
                    Ident(V::kNormal)
                } else {
                    utils::ZoomAdjustedPixelValue(spacing, style)
                }
            }
        }
        // longhands_custom.cc:7455-7462,12395-12402.
        P::kOrphans => values::numeric(style.Orphans() as f64, UnitType::kNumber),
        P::kWidows => values::numeric(style.Widows() as f64, UnitType::kNumber),
        // longhands_custom.cc:9733-9740.
        P::kTextAlign => Ident(match style.GetTextAlign() {
            ETextAlign::kLeft => V::kLeft,
            ETextAlign::kRight => V::kRight,
            ETextAlign::kCenter => V::kCenter,
            ETextAlign::kJustify => V::kJustify,
            ETextAlign::kMatchParent => V::kMatchParent,
            ETextAlign::kWebkitLeft => V::kWebkitLeft,
            ETextAlign::kWebkitRight => V::kWebkitRight,
            ETextAlign::kWebkitCenter => V::kWebkitCenter,
            ETextAlign::kStart => V::kStart,
            ETextAlign::kEnd => V::kEnd,
        }),
        // longhands_custom.cc:9876-9884,11861-11869.
        P::kTextDecorationLine => {
            utils::RenderTextDecorationFlagsToCSSValue(style.GetTextDecorationLine())
        }
        P::kWebkitTextDecorationsInEffect => {
            utils::RenderTextDecorationFlagsToCSSValue(style.TextDecorationsInEffect())
        }
        // longhands_custom.cc:10051-10073.
        P::kTextIndent => {
            let length = utils::ZoomAdjustedPixelValueForLength(style.TextIndent(), style);
            if !style.IsTextIndentHanging() && !style.IsTextIndentEachLine() {
                length
            } else {
                let mut list = vec![length];
                if style.IsTextIndentHanging() {
                    list.push(Ident(V::kHanging));
                }
                if style.IsTextIndentEachLine() {
                    list.push(Ident(V::kEachLine));
                }
                List(list)
            }
        }
        // longhands_custom.cc:10240-10273.
        P::kTextTransform => {
            let flags = style.TextTransform();
            if flags == ETextTransform::kNone {
                Ident(V::kNone)
            } else if flags == ETextTransform::kMathAuto {
                Ident(V::kMathAuto)
            } else {
                let has = |flag: ETextTransform| flags.bits() & flag.bits() != 0;
                let mut list = Vec::new();
                if has(ETextTransform::kCapitalize) {
                    list.push(Ident(V::kCapitalize));
                } else if has(ETextTransform::kUppercase) {
                    list.push(Ident(V::kUppercase));
                } else if has(ETextTransform::kLowercase) {
                    list.push(Ident(V::kLowercase));
                }
                if has(ETextTransform::kFullWidth) {
                    list.push(Ident(V::kFullWidth));
                }
                if has(ETextTransform::kFullSizeKana) {
                    list.push(Ident(V::kFullSizeKana));
                }
                List(list)
            }
        }
        // longhands_custom.cc:12138-12146,12179-12187,12203-12210.
        P::kWebkitTextFillColor => utils::CurrentColorOrValidColor(style, style.TextFillColor()),
        P::kWebkitTextStrokeColor => {
            utils::CurrentColorOrValidColor(style, style.TextStrokeColor())
        }
        P::kWebkitTextStrokeWidth => utils::ZoomAdjustedPixelValue(style.TextStrokeWidth(), style),
        // longhands_custom.cc:12371-12385.
        P::kWhiteSpaceCollapse => Ident(match style.GetWhiteSpaceCollapse() {
            WhiteSpaceCollapse::kCollapse => V::kCollapse,
            WhiteSpaceCollapse::kPreserve => V::kPreserve,
            WhiteSpaceCollapse::kPreserveBreaks => V::kPreserveBreaks,
            WhiteSpaceCollapse::kBreakSpaces => V::kBreakSpaces,
            _ => unreachable!("canonical white-space-collapse"),
        }),
        P::kTextWrapMode => Ident(match style.GetTextWrapMode() {
            TextWrapMode::kNowrap => V::kNowrap,
            TextWrapMode::kWrap => V::kWrap,
        }),
        // longhands_custom.cc:3602-3609,10752-10759.
        P::kDirection => Ident(if IsLtr(style.Direction()) {
            V::kLtr
        } else {
            V::kRtl
        }),
        P::kUnicodeBidi => Ident(match style.GetUnicodeBidi() {
            UnicodeBidi::kNormal => V::kNormal,
            UnicodeBidi::kEmbed => V::kEmbed,
            UnicodeBidi::kBidiOverride => V::kBidiOverride,
            UnicodeBidi::kIsolate => V::kIsolate,
            UnicodeBidi::kPlaintext => V::kPlaintext,
            UnicodeBidi::kIsolateOverride => V::kIsolateOverride,
        }),
        // longhands_custom.cc:9845-9853,9954-9962,9976-9993.
        P::kTextDecorationColor => {
            utils::CurrentColorOrValidColor(style, style.TextDecorationColor())
        }
        P::kTextDecorationStyle => Ident(match style.TextDecorationStyle() {
            ETextDecorationStyle::kSolid => V::kSolid,
            ETextDecorationStyle::kDouble => V::kDouble,
            ETextDecorationStyle::kDotted => V::kDotted,
            ETextDecorationStyle::kDashed => V::kDashed,
            ETextDecorationStyle::kWavy => V::kWavy,
        }),
        P::kTextDecorationThickness => {
            let thickness = style.GetTextDecorationThickness();
            if thickness.IsFromFont() {
                Ident(V::kFromFont)
            } else if thickness.IsAuto() {
                Ident(V::kAuto)
            } else {
                utils::ZoomAdjustedPixelValueForLength(thickness.Thickness(), style)
            }
        }
        // shorthands_custom.cc:5245-5300. Keep generated longhand order and
        // actual typed values, including nested CSSValueList identity.
        P::kTextDecoration => {
            let mut list = Vec::new();
            for &id in style_model::property_metadata::ShorthandFor(P::kTextDecoration) {
                let value = CSSValueFromComputedStyle(id, style, options)?;
                let keyword = if let CSSValuePayload::kIdentifierClass(value) = value.Payload() {
                    Some(value.0)
                } else {
                    None
                };
                if (id == P::kTextDecorationThickness && keyword == Some(V::kAuto))
                    || (id == P::kTextDecorationLine && keyword == Some(V::kNone))
                    || (id == P::kTextDecorationStyle && keyword == Some(V::kSolid))
                    || (id == P::kTextDecorationColor
                        && style.TextDecorationColor().IsCurrentColor())
                {
                    continue;
                }
                list.push(value);
            }
            if list.is_empty() {
                list.push(Ident(V::kNone));
            }
            List(list)
        }
        // longhands_custom.cc:4354-4364,10790-10818.
        P::kFloat => Ident(if style.HasOutOfFlowPosition() {
            V::kNone
        } else {
            match style.Floating() {
                EFloat::kNone => V::kNone,
                EFloat::kLeft => V::kLeft,
                EFloat::kRight => V::kRight,
                EFloat::kInlineStart => V::kInlineStart,
                EFloat::kInlineEnd => V::kInlineEnd,
            }
        }),
        P::kVerticalAlign => match style.VerticalAlign() {
            EVerticalAlign::kLength => {
                utils::ZoomAdjustedPixelValueForLength(style.GetVerticalAlignLength(), style)
            }
            value => Ident(match value {
                EVerticalAlign::kBaseline => V::kBaseline,
                EVerticalAlign::kMiddle => V::kMiddle,
                EVerticalAlign::kSub => V::kSub,
                EVerticalAlign::kSuper => V::kSuper,
                EVerticalAlign::kTextTop => V::kTextTop,
                EVerticalAlign::kTextBottom => V::kTextBottom,
                EVerticalAlign::kTop => V::kTop,
                EVerticalAlign::kBottom => V::kBottom,
                EVerticalAlign::kBaselineMiddle => V::kWebkitBaselineMiddle,
                EVerticalAlign::kLength => unreachable!(),
            }),
        },
        P::kDisplay => return DisplayValue(style),
        // InternalFontSizeDelta is an editing PropertySet value, never a
        // fabricated computed property. Full computable copying stays explicit.
        _ => match crate::computed_keyword_values::KeywordValue(property, style)
            .or_else(|| crate::computed_scalar_values::ScalarValue(property, style))
            .or_else(|| crate::computed_font_values::FontValue(property, style))
        {
            Some(value) => value,
            None => return Err(ComputedStyleValueError::UnsupportedProperty(property)),
        },
    })
}

// longhands_custom.cc:3849-3894; css_value_id_mappings.h (EDisplay).
fn DisplayValue(style: &ComputedStyle) -> Result<Rc<Value>, ComputedStyleValueError> {
    use EDisplay as D;
    let ids: &[V] = match style.Display() {
        D::kLayoutCustom | D::kInlineLayoutCustom => {
            return Err(ComputedStyleValueError::UnsupportedProperty(P::kDisplay))
        }
        D::kBlockMath => &[V::kBlock, V::kMath],
        D::kBlockRuby => &[V::kBlock, V::kRuby],
        D::kInlineListItem => &[V::kInline, V::kListItem],
        D::kFlowRootListItem => &[V::kFlowRoot, V::kListItem],
        D::kInlineFlowRootListItem => &[V::kInline, V::kFlowRoot, V::kListItem],
        value => {
            return Ok(Ident(match value {
                D::kInline => V::kInline,
                D::kBlock => V::kBlock,
                D::kListItem => V::kListItem,
                D::kInlineBlock => V::kInlineBlock,
                D::kTable => V::kTable,
                D::kInlineTable => V::kInlineTable,
                D::kTableRowGroup => V::kTableRowGroup,
                D::kTableHeaderGroup => V::kTableHeaderGroup,
                D::kTableFooterGroup => V::kTableFooterGroup,
                D::kTableRow => V::kTableRow,
                D::kTableColumnGroup => V::kTableColumnGroup,
                D::kTableColumn => V::kTableColumn,
                D::kTableCell => V::kTableCell,
                D::kTableCaption => V::kTableCaption,
                D::kWebkitBox => V::kWebkitBox,
                D::kWebkitInlineBox => V::kWebkitInlineBox,
                D::kFlex => V::kFlex,
                D::kInlineFlex => V::kInlineFlex,
                D::kGrid => V::kGrid,
                D::kInlineGrid => V::kInlineGrid,
                D::kContents => V::kContents,
                D::kFlowRoot => V::kFlowRoot,
                D::kNone => V::kNone,
                D::kMath => V::kMath,
                D::kRuby => V::kRuby,
                D::kRubyText => V::kRubyText,
                D::kGridLanes => V::kGridLanes,
                D::kInlineGridLanes => V::kInlineGridLanes,
                _ => unreachable!(),
            }))
        }
    };
    Ok(List(ids.iter().copied().map(Ident).collect()))
}
