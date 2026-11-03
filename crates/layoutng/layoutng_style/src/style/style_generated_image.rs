//! Source-aligned `StyleGeneratedImage` owner.

#![allow(non_snake_case)]

use super::{
    computed_style::ComputedStyle, forward::CSSValue, natural_sizing_info::NaturalSizingInfo,
    style_image::*,
};
use crate::css::{
    css_gradient_value::{
        CSSGeneratedGradientValue, GeneratedGradientData, GradientStop, LinearGradientData,
    },
    css_image_generator_value::CSSImageGeneratorValue,
    css_to_length_conversion_data::ContainerSizes,
    style_color::StyleColor,
};
use foundation::{
    Color, ColorSpace, HueInterpolationMethod, MakeGarbageCollected, Member, Traceable, Visitor,
};

// cpp: core/style/style_generated_image.h:41-88
#[repr(C)]
pub struct StyleGeneratedImage {
    base: StyleImage,
    image_generator_value: Member<CSSImageGeneratorValue>,
    container_sizes: ContainerSizes,
}

impl StyleGeneratedImage {
    // cpp: core/style/style_generated_image.cc:34-42
    pub fn new(generator: *mut CSSImageGeneratorValue, container_sizes: ContainerSizes) -> Self {
        assert!(!generator.is_null());
        let mut base = StyleImage::new_for_derived(&GENERATED_VTABLE);
        base.set_generated_image_for_derived(true);
        Self {
            base,
            image_generator_value: Member::from_ptr(generator),
            container_sizes,
        }
    }

    pub fn StyleImage(&self) -> &StyleImage {
        &self.base
    }

    fn generator(&self) -> &CSSImageGeneratorValue {
        unsafe { &*self.image_generator_value.Get() }
    }
}

impl Traceable for StyleGeneratedImage {
    // cpp: core/style/style_generated_image.cc:122-126
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.image_generator_value);
        self.container_sizes.Trace(visitor);
    }
}

fn generated(base: &StyleImage) -> &StyleGeneratedImage {
    unsafe { &*(base as *const StyleImage).cast() }
}

pub fn ResolvedGradient(base: &StyleImage) -> Option<&GeneratedGradientData> {
    if !base.IsGeneratedImage() {
        return None;
    }
    Some(generated(base).generator().Value().gradient().Data())
}

pub fn ResolvedLinearGradient(base: &StyleImage) -> Option<&LinearGradientData> {
    match ResolvedGradient(base)? {
        GeneratedGradientData::Linear(value) => Some(value),
        _ => None,
    }
}

fn css_value(base: &StyleImage) -> *mut CSSValue {
    generated(base).generator().Value() as *const CSSValue as *mut CSSValue
}

static GENERATED_VTABLE: StyleImageVTable = StyleImageVTable {
    CssValue: css_value,
    ComputedCSSValue: |base, _, _, _| css_value(base),
    CanRender: StyleImageDefaultCanRender,
    IsLoaded: StyleImageDefaultIsLoaded,
    IsLoading: StyleImageDefaultIsLoading,
    ErrorOccurred: StyleImageDefaultErrorOccurred,
    IsCorsSameOrigin: |base| generated(base).generator().IsCorsSameOrigin(),
    GetNaturalSizingInfo: |_, _, _| NaturalSizingInfo::None(),
    ImageSize: |_, _, size, _| *size,
    HasIntrinsicSize: |_| false,
    AddClient: |base, observer| generated(base).generator().AddClient(observer),
    RemoveClient: |base, observer| generated(base).generator().RemoveClient(observer),
    GetImage: |base, observer, node, style, size| {
        let generated = generated(base);
        generated
            .generator()
            .GetImage(observer, node, style, &generated.container_sizes, size)
    },
    Data: |base| generated(base).image_generator_value.Get().cast(),
    ImageScaleFactor: StyleImageDefaultImageScaleFactor,
    // cpp: core/style/style_generated_image.cc:117-120 and
    // core/css/css_gradient_value.cc:922-933.
    KnownToBeOpaque: |base, style| generated(base).generator().KnownToBeOpaque(style),
    CachedImage: StyleImageDefaultCachedImage,
    DependsOnCurrentColor: |base| generated(base).generator().IsUsingCurrentColor(),
    IsLoadedAfterMouseover: StyleImageDefaultIsLoadedAfterMouseover,
    Trace: |base, visitor| generated(base).Trace(visitor),
    // cpp: core/style/style_generated_image.cc:44-54
    IsEqual: |base, other| {
        if !other.IsGeneratedImage() {
            return false;
        }
        let this = generated(base);
        let other = generated(other);
        this.container_sizes.SizesEqual(&other.container_sizes)
            && this.generator().Value() == other.generator().Value()
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_with_color(color: Color) -> *mut StyleGeneratedImage {
        let value = MakeGarbageCollected(CSSGeneratedGradientValue::FromResolved(
            LinearGradientData {
                start: [0.0, 0.0],
                end: [1.0, 0.0],
                angle_degrees: None,
                corner: None,
                repeating: false,
                color_interpolation_space: ColorSpace::kSRGB,
                hue_interpolation_method: HueInterpolationMethod::kShorter,
                stops: vec![
                    GradientStop {
                        offset: 0.0,
                        offset_length: 0.0,
                        color: StyleColor::from_color(color),
                        is_hint: false,
                    },
                    GradientStop {
                        offset: 1.0,
                        offset_length: 0.0,
                        color: StyleColor::from_color(color),
                        is_hint: false,
                    },
                ],
            },
        ));
        let generator = unsafe { &*value }.Generator() as *const CSSImageGeneratorValue
            as *mut CSSImageGeneratorValue;
        MakeGarbageCollected(StyleGeneratedImage::new(
            generator,
            ContainerSizes::default(),
        ))
    }

    #[test]
    fn opacity_and_container_sizes_follow_generated_image_contract() {
        let _heap = foundation::LayoutHeapScope::new();
        let style = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let opaque = generated_with_color(Color::kBlack);
        let same_opaque = generated_with_color(Color::kBlack);
        let translucent = generated_with_color(Color::FromRGBA(0, 0, 0, 127));
        assert!(unsafe { &*opaque }.StyleImage().KnownToBeOpaque(style));
        assert!(!unsafe { &*translucent }.StyleImage().KnownToBeOpaque(style));
        assert!(unsafe { &*opaque }.StyleImage() == unsafe { &*same_opaque }.StyleImage());
        assert_ne!(
            unsafe { &*opaque }.generator().Value().Hash(),
            unsafe { &*same_opaque }.generator().Value().Hash()
        );
        assert!(!unsafe { &*opaque }
            .generator()
            .IsUsingContainerRelativeUnits());

        let generator = unsafe { &*opaque }.image_generator_value.Get();
        let narrow = MakeGarbageCollected(StyleGeneratedImage::new(
            generator,
            ContainerSizes::PreCached(Some(100.0), Some(50.0)),
        ));
        let wide = MakeGarbageCollected(StyleGeneratedImage::new(
            generator,
            ContainerSizes::PreCached(Some(200.0), Some(50.0)),
        ));
        assert!(unsafe { &*narrow }.StyleImage() != unsafe { &*wide }.StyleImage());
    }
}
