//! Source-aligned computed gradient CSS values.
//!
//! Style resolution publishes a normalized representation of Chromium's
//! `CSSGradientValue` subclasses. Geometry that depends on the eventual image
//! size stays symbolic until the Style -> Paint boundary.

#![allow(non_snake_case)]

use super::{
    css_image_generator_value::CSSImageGeneratorValue,
    css_to_length_conversion_data::ContainerSizes, css_value::CSSValueClass,
    style_color::StyleColor,
};
use crate::style::{
    computed_style::ComputedStyle,
    forward::{ImageResourceObserver, Node},
};
use foundation::{
    gfx::SizeF, ColorSpace, HueInterpolationMethod, ScopedRefPtr, Traceable, Visitor,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradientLength {
    /// Fraction of the target axis/gradient basis (1.0 == 100%).
    pub percent: f64,
    pub pixels: f64,
}

#[derive(Clone, PartialEq)]
pub struct GradientStop {
    /// Fraction of the gradient basis. Values outside 0..1 are valid until the
    /// target-size normalization step.
    pub offset: f64,
    pub offset_length: f64,
    /// Keep the computed StyleColor symbolic so `currentColor` resolves against
    /// the final ComputedStyle at the Style -> Paint boundary.
    pub color: StyleColor,
    pub is_hint: bool,
}

impl std::fmt::Debug for GradientStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = formatter.debug_struct("GradientStop");
        debug
            .field("offset", &self.offset)
            .field("offset_length", &self.offset_length)
            .field("is_hint", &self.is_hint)
            .field(
                "color_depends_on_current_color",
                &self.color.DependsOnCurrentColor(),
            );
        if self.color.IsAbsoluteColor() {
            debug.field("color", &self.color.GetColor());
        }
        debug.finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradientData {
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub angle_degrees: Option<f64>,
    pub corner: Option<[i8; 2]>,
    pub repeating: bool,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialGradientShape {
    Circle,
    Ellipse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialGradientExtent {
    Explicit,
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradientData {
    pub center_x: GradientLength,
    pub center_y: GradientLength,
    pub radius_x: GradientLength,
    pub radius_y: GradientLength,
    pub shape: RadialGradientShape,
    pub extent: RadialGradientExtent,
    pub repeating: bool,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConicGradientData {
    pub center_x: GradientLength,
    pub center_y: GradientLength,
    pub angle_degrees: f64,
    pub repeating: bool,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratedGradientData {
    Linear(LinearGradientData),
    Radial(RadialGradientData),
    Conic(ConicGradientData),
}

pub struct GradientGeneratedImage {
    pub data: GeneratedGradientData,
    pub size: SizeF,
}

pub struct Image {
    pub generated: GradientGeneratedImage,
}

impl Image {
    pub fn Size(&self) -> SizeF {
        self.generated.size.clone()
    }
}

impl GeneratedGradientData {
    pub(crate) fn Stops(&self) -> &[GradientStop] {
        match self {
            Self::Linear(value) => &value.stops,
            Self::Radial(value) => &value.stops,
            Self::Conic(value) => &value.stops,
        }
    }

    pub(crate) fn IsUsingCurrentColor(&self) -> bool {
        self.Stops()
            .iter()
            .any(|stop| !stop.is_hint && stop.color.DependsOnCurrentColor())
    }
}

#[repr(C)]
pub struct CSSGradientValue {
    generator: CSSImageGeneratorValue,
    data: GeneratedGradientData,
}

impl CSSGradientValue {
    pub fn FromResolved(data: LinearGradientData) -> Self {
        Self::new(GeneratedGradientData::Linear(data))
    }

    pub fn FromRadial(data: RadialGradientData) -> Self {
        Self::new(GeneratedGradientData::Radial(data))
    }

    pub fn FromConic(data: ConicGradientData) -> Self {
        Self::new(GeneratedGradientData::Conic(data))
    }

    fn new(data: GeneratedGradientData) -> Self {
        assert!(!match &data {
            GeneratedGradientData::Linear(value) => value.stops.is_empty(),
            GeneratedGradientData::Radial(value) => value.stops.is_empty(),
            GeneratedGradientData::Conic(value) => value.stops.is_empty(),
        });
        let class_type = match &data {
            GeneratedGradientData::Linear(_) => CSSValueClass::LinearGradient,
            GeneratedGradientData::Radial(_) => CSSValueClass::RadialGradient,
            GeneratedGradientData::Conic(_) => CSSValueClass::ConicGradient,
        };
        Self {
            generator: CSSImageGeneratorValue::new(class_type),
            data,
        }
    }

    pub fn Generator(&self) -> &CSSImageGeneratorValue {
        &self.generator
    }

    pub fn Data(&self) -> &GeneratedGradientData {
        &self.data
    }

    // cpp: core/css/css_gradient_value.cc:152-170. Style resolution has
    // already normalized lengths; cache admission still follows the source's
    // color-dependent rule and registered-client requirement.
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        _node: &Node,
        _style: &ComputedStyle,
        _container_sizes: &ContainerSizes,
        target_size: &SizeF,
    ) -> ScopedRefPtr<Image> {
        if target_size.IsEmpty() {
            return ScopedRefPtr::default();
        }
        let cacheable = self
            .data
            .Stops()
            .iter()
            .filter(|stop| !stop.is_hint)
            .all(|stop| stop.color.IsAbsoluteColor());
        if cacheable {
            if !self.generator.HasClient(observer) {
                return ScopedRefPtr::default();
            }
            let cached = self.generator.GetCachedImage(observer, target_size);
            if !cached.IsNull() {
                return cached;
            }
        }
        let image = ScopedRefPtr::new(Image {
            generated: GradientGeneratedImage {
                data: self.data.clone(),
                size: target_size.clone(),
            },
        });
        if cacheable {
            self.generator.PutImage(target_size, image.clone());
        }
        image
    }

    pub fn IsUsingCurrentColor(&self) -> bool {
        self.data.IsUsingCurrentColor()
    }

    pub fn IsUsingContainerRelativeUnits(&self) -> bool {
        false
    }
}

pub type CSSGeneratedGradientValue = CSSGradientValue;
pub type CSSLinearGradientValue = CSSGradientValue;
pub type CSSRadialGradientValue = CSSGradientValue;
pub type CSSConicGradientValue = CSSGradientValue;

impl Traceable for CSSGradientValue {
    fn Trace(&self, visitor: &mut Visitor) {
        self.generator.Trace(visitor);
        for stop in self.data.Stops() {
            stop.color.Trace(visitor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::image_resource_observer::ImageResourceObserverVTable;
    use foundation::{Color, MakeGarbageCollected, String as BlinkString};

    static OBSERVER_VTABLE: ImageResourceObserverVTable = ImageResourceObserverVTable {
        DebugName: |_| BlinkString::from("CSSGradientValue test observer"),
    };

    fn linear(color: StyleColor) -> *mut CSSGradientValue {
        MakeGarbageCollected(CSSGradientValue::FromResolved(LinearGradientData {
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
                    color: color.clone(),
                    is_hint: false,
                },
                GradientStop {
                    offset: 1.0,
                    offset_length: 0.0,
                    color,
                    is_hint: false,
                },
            ],
        }))
    }

    #[test]
    fn cacheable_gradients_require_clients_but_current_color_does_not_cache() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut observer = ImageResourceObserver::new_for_derived(&OBSERVER_VTABLE);
        let node = unsafe { &*std::ptr::NonNull::<Node>::dangling().as_ptr() };
        let style = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let size = SizeF::new(32.0, 16.0);
        let containers = ContainerSizes::default();

        let opaque = unsafe { &*linear(StyleColor::from_color(Color::kBlack)) };
        assert!(opaque
            .GetImage(&observer, node, style, &containers, &size)
            .IsNull());
        opaque.Generator().AddClient(&mut observer);
        let first = opaque.GetImage(&observer, node, style, &containers, &size);
        let second = opaque.GetImage(&observer, node, style, &containers, &size);
        assert!(!first.IsNull());
        assert!(first == second);
        opaque.Generator().RemoveClient(&mut observer);

        let current_color = unsafe { &*linear(StyleColor::CurrentColor()) };
        assert!(!current_color
            .GetImage(&observer, node, style, &containers, &size)
            .IsNull());
    }
}
