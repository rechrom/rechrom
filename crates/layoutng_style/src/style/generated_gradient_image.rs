//! Native generated-image ownership for resolved CSS gradients.
//!
//! The native object retains target-size-dependent geometry and stop positions
//! until the Style -> Paint boundary. Paint resolves that symbolic payload
//! against the concrete background/mask/content tile size.
#![allow(non_snake_case)]

use super::{
    forward::ImageResourceObserver, natural_sizing_info::NaturalSizingInfo, style_image::*,
};
use crate::css::style_color::StyleColor;
use foundation::{
    gfx::SizeF, ColorSpace, HueInterpolationMethod, Member, Persistent, ScopedRefPtr, Traceable,
    Visitor,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
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
    /// Keep the computed StyleColor symbolic. `currentColor` and color
    /// functions that depend on it resolve against the final ComputedStyle at
    /// the Style -> Paint boundary, matching CSSGradientValue::AddStops.
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
    /// CSS bearing angle. Corner directions are converted to an angle only
    /// after the target size is known.
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
impl GeneratedGradientData {
    fn Hash(&self) -> u32 {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        format!("{self:?}").hash(&mut hash);
        hash.finish() as u32
    }
    fn Stops(&self) -> &[GradientStop] {
        match self {
            Self::Linear(value) => &value.stops,
            Self::Radial(value) => &value.stops,
            Self::Conic(value) => &value.stops,
        }
    }
    fn IsUsingCurrentColor(&self) -> bool {
        self.Stops()
            .iter()
            .any(|stop| !stop.is_hint && stop.color.DependsOnCurrentColor())
    }
}

#[repr(C)]
pub struct CSSValue {
    class_type: CSSValueClass,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum CSSValueClass {
    LinearGradient,
    RadialGradient,
    ConicGradient,
}
impl CSSValue {
    fn gradient(&self) -> &CSSGeneratedGradientValue {
        unsafe { &*(self as *const Self).cast() }
    }
    pub fn Hash(&self) -> u32 {
        self.gradient().data.Hash()
    }
    pub fn IsImageGeneratorValue(&self) -> bool {
        true
    }
    pub fn MayContainUrl(&self) -> bool {
        false
    }
}
impl PartialEq for CSSValue {
    fn eq(&self, other: &Self) -> bool {
        self.class_type == other.class_type && self.gradient().data == other.gradient().data
    }
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

#[derive(Clone, Default)]
struct SizeAndCount {
    size: SizeF,
    count: usize,
}
fn size_key(size: &SizeF) -> (u32, u32) {
    (size.width().to_bits(), size.height().to_bits())
}

#[repr(C)]
pub struct CSSImageGeneratorValue {
    value: CSSValue,
    clients: RefCell<HashMap<usize, (Member<ImageResourceObserver>, SizeAndCount)>>,
    sizes: RefCell<HashMap<(u32, u32), usize>>,
    images: RefCell<HashMap<(u32, u32), ScopedRefPtr<Image>>>,
    keep_alive: RefCell<Option<Persistent<CSSImageGeneratorValue>>>,
}
impl CSSImageGeneratorValue {
    fn new(class_type: CSSValueClass) -> Self {
        Self {
            value: CSSValue { class_type },
            clients: RefCell::new(HashMap::new()),
            sizes: RefCell::new(HashMap::new()),
            images: RefCell::new(HashMap::new()),
            keep_alive: RefCell::new(None),
        }
    }
    pub fn AddClient(&self, client: *mut ImageResourceObserver) {
        assert!(!client.is_null());
        let mut clients = self.clients.borrow_mut();
        if clients.is_empty() {
            *self.keep_alive.borrow_mut() =
                Some(Persistent::from_ptr(self as *const Self as *mut Self));
        }
        clients
            .entry(client as usize)
            .or_insert_with(|| (Member::from_ptr(client), SizeAndCount::default()))
            .1
            .count += 1;
    }
    fn RemoveSize(&self, size: &SizeF) {
        if size.IsEmpty() {
            return;
        }
        let key = size_key(size);
        let mut sizes = self.sizes.borrow_mut();
        let count = sizes
            .get_mut(&key)
            .expect("registered generated-image size");
        *count -= 1;
        if *count == 0 {
            sizes.remove(&key);
            self.images.borrow_mut().remove(&key);
        }
    }
    pub fn RemoveClient(&self, client: *mut ImageResourceObserver) {
        let mut clients = self.clients.borrow_mut();
        let (_, state) = clients
            .get_mut(&(client as usize))
            .expect("registered image client");
        self.RemoveSize(&state.size);
        state.size = SizeF::default();
        state.count -= 1;
        if state.count == 0 {
            clients.remove(&(client as usize));
        }
        if clients.is_empty() {
            self.keep_alive.borrow_mut().take();
        }
    }
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        target: &SizeF,
    ) -> ScopedRefPtr<Image> {
        let mut clients = self.clients.borrow_mut();
        if !clients.contains_key(&(observer as *const _ as usize)) {
            return ScopedRefPtr::default();
        }
        if let Some((_, state)) = clients.get_mut(&(observer as *const _ as usize)) {
            if state.size != *target {
                self.RemoveSize(&state.size);
                state.size = target.clone();
                if !target.IsEmpty() {
                    *self.sizes.borrow_mut().entry(size_key(target)).or_default() += 1;
                }
            }
        }
        drop(clients);
        if target.IsEmpty() {
            return ScopedRefPtr::default();
        }
        let key = size_key(target);
        if let Some(image) = self.images.borrow().get(&key) {
            return image.clone();
        }
        let image = ScopedRefPtr::new(Image {
            generated: GradientGeneratedImage {
                data: self.value.gradient().data.clone(),
                size: target.clone(),
            },
        });
        if self.sizes.borrow().contains_key(&key) {
            self.images.borrow_mut().insert(key, image.clone());
        }
        image
    }
    fn Trace(&self, visitor: &mut Visitor) {
        for (client, _) in self.clients.borrow().values() {
            visitor.Trace(client);
        }
    }
}

#[repr(C)]
pub struct CSSGeneratedGradientValue {
    generator: CSSImageGeneratorValue,
    data: GeneratedGradientData,
}
impl CSSGeneratedGradientValue {
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
    pub fn IsUsingCurrentColor(&self) -> bool {
        self.data.IsUsingCurrentColor()
    }
}
pub type CSSLinearGradientValue = CSSGeneratedGradientValue;
pub type CSSRadialGradientValue = CSSGeneratedGradientValue;
pub type CSSConicGradientValue = CSSGeneratedGradientValue;
impl Traceable for CSSGeneratedGradientValue {
    fn Trace(&self, visitor: &mut Visitor) {
        self.generator.Trace(visitor);
        for stop in self.data.Stops() {
            stop.color.Trace(visitor);
        }
    }
}

#[repr(C)]
pub struct StyleGeneratedImage {
    base: StyleImage,
    image_generator_value: Member<CSSImageGeneratorValue>,
}
impl StyleGeneratedImage {
    pub fn new(generator: *mut CSSImageGeneratorValue) -> Self {
        assert!(!generator.is_null());
        let mut base = StyleImage::new_for_derived(&GENERATED_VTABLE);
        base.set_generated_image_for_derived(true);
        Self {
            base,
            image_generator_value: Member::from_ptr(generator),
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
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.image_generator_value);
    }
}
fn generated(base: &StyleImage) -> &StyleGeneratedImage {
    unsafe { &*(base as *const StyleImage).cast() }
}

pub fn ResolvedGradient(base: &StyleImage) -> Option<&GeneratedGradientData> {
    if !base.IsGeneratedImage() {
        return None;
    }
    Some(&generated(base).generator().value.gradient().data)
}
pub fn ResolvedLinearGradient(base: &StyleImage) -> Option<&LinearGradientData> {
    match ResolvedGradient(base)? {
        GeneratedGradientData::Linear(value) => Some(value),
        _ => None,
    }
}
fn css_value(base: &StyleImage) -> *mut CSSValue {
    &generated(base).generator().value as *const CSSValue as *mut CSSValue
}
static GENERATED_VTABLE: StyleImageVTable = StyleImageVTable {
    CssValue: css_value,
    ComputedCSSValue: |base, _, _, _| css_value(base),
    CanRender: StyleImageDefaultCanRender,
    IsLoaded: StyleImageDefaultIsLoaded,
    IsLoading: StyleImageDefaultIsLoading,
    ErrorOccurred: StyleImageDefaultErrorOccurred,
    IsCorsSameOrigin: |_| true,
    GetNaturalSizingInfo: |_, _, _| NaturalSizingInfo::None(),
    ImageSize: |_, _, size, _| size.clone(),
    HasIntrinsicSize: |_| false,
    AddClient: |base, observer| generated(base).generator().AddClient(observer),
    RemoveClient: |base, observer| generated(base).generator().RemoveClient(observer),
    GetImage: |base, observer, _, _, size| generated(base).generator().GetImage(observer, size),
    Data: |base| generated(base).image_generator_value.Get().cast(),
    ImageScaleFactor: StyleImageDefaultImageScaleFactor,
    CachedImage: StyleImageDefaultCachedImage,
    DependsOnCurrentColor: |base| {
        generated(base)
            .generator()
            .value
            .gradient()
            .IsUsingCurrentColor()
    },
    IsLoadedAfterMouseover: StyleImageDefaultIsLoadedAfterMouseover,
    Trace: |base, visitor| generated(base).Trace(visitor),
    IsEqual: |base, other| {
        other.IsGeneratedImage()
            && generated(base).generator().value == generated(other).generator().value
    },
};
