use std::sync::OnceLock;

use font_engine::Hyphenation;
use foundation::{
    gfx, AtomicString, CSSPropertyID, Color, EInsideLink, EPosition, ETextTransform,
    ETransformStyle3D, Hyphens, MakeGarbageCollected, Member, String, TextOffsetMap,
};

use super::applied_text_decoration::{AppliedTextDecoration, AppliedTextDecorationVector};
use super::computed_style::ComputedStyle;
use super::computed_style_base::ComputedStyleBase;
use super::computed_style_constants::{EFillAttachment, EFillBox, PseudoId};
use super::fill_layer::FillLayer;
use super::forward::Element;
use super::gap_data_list::GapDataList;
use super::style_cached_data::StyleCachedData;
use crate::css::style_color::StyleColor;
use crate::text_transform_services::CurrentNativeTextTransformResolver;

// cpp: layoutng_style/style/computed_style_layout.cc:34-48
#[allow(non_snake_case)]
fn ApplyMathAutoTransform(text: &String, offset_map: Option<&mut TextOffsetMap>) -> String {
    let mut source_units = text.Span16().unwrap_or_default().iter().copied();
    let original = match (source_units.next(), source_units.next()) {
        (Some(unit), None) => unit,
        _ => return text.clone(),
    };
    let transformed_char = foundation::ItalicMathVariant(u32::from(original));
    if transformed_char == u32::from(original) {
        return text.clone();
    }
    let character = char::from_u32(transformed_char).expect("ItalicMathVariant returns a scalar");
    let mut units = [0u16; 2];
    let transformed_text = character.encode_utf16(&mut units);
    let result = String::from_utf16(transformed_text);
    if let Some(offset_map) = offset_map {
        offset_map.Append(1, result.length());
    }
    result
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:320
    // cpp: layoutng_style/style/computed_style_layout.cc:52-56
    pub fn EnsureCachedData(&self) -> &mut StyleCachedData {
        if self.cached_data_.get().Get().is_null() {
            self.cached_data_.set(Member::from_ptr(MakeGarbageCollected(
                StyleCachedData::default(),
            )));
        }
        unsafe { &mut *self.cached_data_.get().Get() }
    }

    // cpp: layoutng_style/style/computed_style.h:2798-2799
    // cpp: layoutng_style/style/computed_style_layout.cc:58-63
    pub fn GetCurrentColor(&self, is_current_color: Option<&mut bool>) -> Color {
        debug_assert!(!self.Color().IsCurrentColor());
        if let Some(is_current_color) = is_current_color {
            *is_current_color = self.ColorIsCurrentColor();
        }
        self.Color()
            .Resolve(Color::default(), self.UsedColorScheme(), None)
    }

    // cpp: layoutng_style/style/computed_style.h:2800-2801
    // cpp: layoutng_style/style/computed_style_layout.cc:65-71
    pub fn GetInternalVisitedCurrentColor(&self, is_current_color: Option<&mut bool>) -> Color {
        debug_assert!(!self.InternalVisitedColor().IsCurrentColor());
        if let Some(is_current_color) = is_current_color {
            *is_current_color = self.InternalVisitedColorIsCurrentColor();
        }
        self.InternalVisitedColor()
            .Resolve(Color::default(), self.UsedColorScheme(), None)
    }

    // cpp: layoutng_style/style/computed_style.h:453
    // cpp: layoutng_style/style/computed_style_layout.cc:73-77
    pub fn GetBaseComputedStyle(&self) -> *const ComputedStyle {
        let base_data = self.BaseData();
        if !base_data.is_null() {
            return unsafe { (&*base_data).GetBaseComputedStyle() };
        }
        std::ptr::null()
    }

    // cpp: layoutng_style/style/computed_style.h:1239-1242
    // cpp: layoutng_style/style/computed_style_layout.cc:79-98
    pub fn ApplyTextTransform(
        &self,
        text: &String,
        previous_character: u16,
        offset_map: Option<&mut TextOffsetMap>,
    ) -> String {
        if self.TextTransform() == ETextTransform::kNone {
            return text.clone();
        }
        let resolver = CurrentNativeTextTransformResolver();
        let Some(resolver) = resolver else {
            return text.clone();
        };
        if self.TextTransform() == ETextTransform::kMathAuto {
            return ApplyMathAutoTransform(text, offset_map);
        }
        unsafe { resolver.as_ref() }.Transform(
            self.TextTransform(),
            self.Locale(),
            text,
            previous_character,
            offset_map,
        )
    }

    // cpp: layoutng_style/style/computed_style.h:1223
    // cpp: layoutng_style/style/computed_style_layout.cc:100-110
    pub fn GetHyphenationWithLimits(&self) -> *mut Hyphenation {
        if self.GetHyphens() != Hyphens::kAuto {
            return std::ptr::null_mut();
        }
        let hyphenation = self.GetHyphenation();
        if hyphenation.is_null() {
            return std::ptr::null_mut();
        }
        let limits = self.HyphenateLimitChars();
        unsafe {
            (&*hyphenation).SetLimits(
                limits.MinBeforeChars(),
                limits.MinAfterChars(),
                limits.MinWordChars(),
            );
        }
        hyphenation
    }

    // cpp: layoutng_style/style/computed_style.h:1222
    // cpp: layoutng_style/style/computed_style_layout.cc:112-115
    pub fn GetHyphenation(&self) -> *mut Hyphenation {
        let locale = self.GetFontDescription().Locale();
        if locale.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { (&*locale).GetHyphenation() }
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1224
    // cpp: layoutng_style/style/computed_style_layout.cc:117-131
    pub fn HyphenString(&self) -> &AtomicString {
        let specified = self.HyphenationString();
        if !specified.IsNull() {
            return specified;
        }
        static HYPHEN_MINUS: OnceLock<AtomicString> = OnceLock::new();
        static HYPHEN: OnceLock<AtomicString> = OnceLock::new();
        let hyphen_minus = HYPHEN_MINUS.get_or_init(|| AtomicString::from_utf16(&[0x002D]));
        let hyphen = HYPHEN.get_or_init(|| AtomicString::from_utf16(&[0x2010]));
        let primary_font = unsafe { (&*self.GetFont()).PrimaryFont() };
        debug_assert!(!primary_font.is_null());
        if !primary_font.is_null() && unsafe { (&*primary_font).GlyphForCharacter(0x2010) != 0 } {
            hyphen
        } else {
            hyphen_minus
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2398
    // cpp: layoutng_style/style/computed_style_layout.cc:133-140
    pub fn HasBackground(&self) -> bool {
        assert!(!self.HasBackgroundImage());
        !self
            .BackgroundColor()
            .Resolve(self.GetCurrentColor(None), self.UsedColorScheme(), None)
            .IsFullyTransparent()
    }

    // cpp: layoutng_style/style/computed_style.h:722
    // cpp: layoutng_style/style/computed_style_layout.cc:142-146
    pub fn HasCustomScrollbarStyle(&self, _element: *mut Element) -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style.h:1210
    // cpp: layoutng_style/style/computed_style_layout.cc:148-151
    pub fn HasWillChangeProperty(&self, id: CSSPropertyID) -> bool {
        debug_assert!(id != CSSPropertyID::kInvalid);
        let will_change = self.WillChange();
        !will_change.is_null() && unsafe { (&*will_change).resolved_longhand_ids().Has(id) }
    }

    // cpp: layoutng_style/style/computed_style.h:2816-2817
    // cpp: layoutng_style/style/computed_style_layout.cc:153-159
    pub fn ResolvedColor(&self, color: &StyleColor, is_current_color: Option<&mut bool>) -> Color {
        let visited = self.InsideLink() == EInsideLink::kInsideVisitedLink;
        let current = if visited {
            self.GetInternalVisitedCurrentColor(None)
        } else {
            self.GetCurrentColor(None)
        };
        color.Resolve(current, self.UsedColorScheme(), is_current_color)
    }

    // cpp: layoutng_style/style/computed_style.h:2872-2873
    // cpp: layoutng_style/style/computed_style_layout.cc:161-175
    pub fn GapRuleColorIsTransparent(&self, colors: &GapDataList<StyleColor>) -> bool {
        let current = self.GetCurrentColor(None);
        let scheme = self.UsedColorScheme();
        colors.GetGapDataList().iter().all(|data| {
            if !data.IsRepeaterData() {
                return data
                    .GetValue()
                    .Resolve(current.clone(), scheme, None)
                    .IsFullyTransparent();
            }
            data.GetValueRepeater()
                .RepeatedValues()
                .iter()
                .all(|value| {
                    value
                        .Resolve(current.clone(), scheme, None)
                        .IsFullyTransparent()
                })
        })
    }

    // cpp: layoutng_style/style/computed_style.h:329-330
    // cpp: layoutng_style/style/computed_style_layout.cc:177-193
    pub fn EnsureAppliedTextDecorationsCache(&self) -> *mut AppliedTextDecorationVector {
        debug_assert!(self.IsDecoratingBox());
        let cache = self.cached_data_.get().Get();
        if cache.is_null() || unsafe { (&*cache).applied_text_decorations_.Get().is_null() } {
            let decorations = MakeGarbageCollected(AppliedTextDecorationVector::default());
            let base = self.BaseTextDecorationData();
            if !base.is_null() {
                unsafe {
                    (&mut *decorations).ReserveInitialCapacity((&*base).size() + 1);
                    *decorations = (&*base).clone();
                }
            }
            unsafe {
                (&mut *decorations).emplace_back(AppliedTextDecoration::new(
                    self.GetTextDecorationLine(),
                    self.TextDecorationStyle(),
                    self.ResolvedColor(self.TextDecorationColor(), None),
                    self.GetTextDecorationThickness().clone(),
                    self.TextUnderlineOffset().clone(),
                    self.GetTextDecorationInset().clone(),
                    self.BoxDecorationBreak(),
                ));
            }
            self.EnsureCachedData().applied_text_decorations_ = Member::from_ptr(decorations);
        }
        unsafe {
            (&*self.cached_data_.get().Get())
                .applied_text_decorations_
                .Get()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2870
    // cpp: layoutng_style/style/computed_style_layout.cc:195-207
    pub fn CalculateIsStackingContextWithoutContainment(&self) -> bool {
        if self.TransformStyle3D() == ETransformStyle3D::kPreserve3d
            || self.ForcesStackingContext()
            || self.StyleType() == PseudoId::kPseudoIdBackdrop
            || self.HasTransformRelatedProperty()
            || self.HasStackingGroupingProperty(self.HasBoxReflect())
            || self.GetPosition() == EPosition::kFixed
            || self.GetPosition() == EPosition::kSticky
        {
            return true;
        }
        // cpp: core/style/computed_style.cc:1301-1339,2941-2949
        let will_change = self.WillChange();
        if !will_change.is_null() {
            let properties = unsafe { &*will_change }.resolved_longhand_ids();
            let creates_context = [
                CSSPropertyID::kOpacity,
                CSSPropertyID::kTransform,
                CSSPropertyID::kTransformStyle,
                CSSPropertyID::kPerspective,
                CSSPropertyID::kTranslate,
                CSSPropertyID::kRotate,
                CSSPropertyID::kScale,
                CSSPropertyID::kOffsetPath,
                CSSPropertyID::kOffsetPosition,
                CSSPropertyID::kMaskImage,
                CSSPropertyID::kWebkitMaskBoxImageSource,
                CSSPropertyID::kClipPath,
                CSSPropertyID::kWebkitBoxReflect,
                CSSPropertyID::kFilter,
                CSSPropertyID::kBackdropFilter,
                CSSPropertyID::kPosition,
                CSSPropertyID::kMixBlendMode,
                CSSPropertyID::kIsolation,
                CSSPropertyID::kContain,
                CSSPropertyID::kViewTransitionName,
            ]
            .into_iter()
            .any(|id| properties.Has(id));
            if creates_context || (self.AllowsZIndex() && properties.Has(CSSPropertyID::kZIndex)) {
                return true;
            }
        }
        self.ShouldCompositeForCurrentAnimations()
    }

    // cpp: layoutng_style/style/computed_style.h:2766-2770
    // cpp: layoutng_style/style/computed_style_layout.cc:220-228
    pub fn ApplyMotionPathTransform<B>(
        &self,
        _origin_x: f32,
        _origin_y: f32,
        _box: Option<&B>,
        _rect: &gfx::RectF,
        _transform: &mut gfx::Transform,
    ) {
        assert!(self.OffsetPath().GetNonNull().is_none());
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBase {
    // cpp: layoutng_style/style/computed_style_base.h:2763
    // cpp: layoutng_style/style/computed_style_layout.cc:209-218
    pub fn IsStackingContextWithoutContainment(&self) -> bool {
        let cached = self.data_.is_stacking_context_without_containment_bits();
        if cached == 0 {
            let style = unsafe { &*(self as *const Self as *const ComputedStyle) };
            let value = style.CalculateIsStackingContextWithoutContainment();
            self.data_
                .set_is_stacking_context_without_containment_bits(0b10 | (u32::from(value) & 1));
        }
        self.data_.is_stacking_context_without_containment_bits() & 1 != 0
    }
}

#[allow(non_snake_case)]
impl FillLayer {
    // cpp: layoutng_style/style/computed_style_layout.cc:230-254
    pub(crate) fn ComputeCachedProperties(&self) {
        debug_assert!(!self.cached_properties_.get().computed);
        let image = unsafe { self.GetImage().as_ref() };
        let effective_clip = self.Clip();
        let mut cache = self.cached_properties_.get();
        cache.layers_clip_max = effective_clip;
        cache.any_layer_uses_content_box =
            effective_clip == EFillBox::kContent || self.Origin() == EFillBox::kContent;
        cache.any_layer_has_image = image.is_some();
        cache.any_layer_has_url_image = image.is_some_and(|image| {
            // StyleImage::CssValue returns its live CSS value, as in Blink.
            unsafe { &*image.CssValue() }.MayContainUrl()
        });
        cache.any_layer_has_local_attachment = self.Attachment() == EFillAttachment::kLocal;
        cache.any_layer_has_fixed_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kFixed;
        cache.any_layer_has_default_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kScroll;
        // StyleGeneratedImage::DependsOnCurrentColor overrides the base with
        // IsUsingCurrentColor; use that real virtual dispatch after its tag check.
        cache.any_layer_uses_current_color =
            image.is_some_and(|image| image.IsGeneratedImage() && image.DependsOnCurrentColor());
        cache.computed = true;
        self.cached_properties_.set(cache);

        let next = self.Next();
        if !next.is_null() {
            let next = unsafe { &*next };
            next.ComputeCachedPropertiesIfNeeded();
            let current_clip = self.LayersClipMax();
            let next_clip = next.LayersClipMax();
            cache.layers_clip_max = if (current_clip as u32) < (next_clip as u32) {
                next_clip
            } else {
                current_clip
            };
            cache.any_layer_uses_content_box |=
                next.cached_properties_.get().any_layer_uses_content_box;
            cache.any_layer_has_image |= next.cached_properties_.get().any_layer_has_image;
            cache.any_layer_has_url_image |= next.cached_properties_.get().any_layer_has_url_image;
            cache.any_layer_has_local_attachment |=
                next.cached_properties_.get().any_layer_has_local_attachment;
            cache.any_layer_has_fixed_attachment_image |= next
                .cached_properties_
                .get()
                .any_layer_has_fixed_attachment_image;
            cache.any_layer_has_default_attachment_image |= next
                .cached_properties_
                .get()
                .any_layer_has_default_attachment_image;
            cache.any_layer_uses_current_color |=
                next.cached_properties_.get().any_layer_uses_current_color;
            self.cached_properties_.set(cache);
        }
    }
}
