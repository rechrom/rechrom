//! Native generated-image objects for resolved, non-repeating linear gradients.
//! The supported values have absolute colors and percentage stop positions;
//! unresolved CSS colors, container units, paint(), and fetched images are not
//! represented by this subset.
#![allow(non_snake_case)]

use super::{
    forward::ImageResourceObserver, natural_sizing_info::NaturalSizingInfo, style_image::*,
};
use foundation::{gfx::SizeF, Color, Member, Persistent, ScopedRefPtr, Traceable, Visitor};
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
};

// cpp: core/css/css_value.h: CSSValue::ClassType, Hash, Equals
#[repr(C)]
pub struct CSSValue {
    class_type: CSSValueClass,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum CSSValueClass {
    LinearGradient,
}
impl CSSValue {
    pub fn Hash(&self) -> u32 {
        self.linear().data.Hash()
    }
    fn linear(&self) -> &CSSLinearGradientValue {
        // CSSValue is the first base of CSSImageGeneratorValue, itself the
        // first base of this tagged, live CSSLinearGradientValue allocation.
        match self.class_type {
            CSSValueClass::LinearGradient => unsafe { &*(self as *const Self).cast() },
        }
    }
    pub fn IsImageGeneratorValue(&self) -> bool {
        true
    }
    // cpp: core/css/css_value.cc:161-166
    // The supported gradient class is neither an image/URI reference nor a
    // value list. Keep this exhaustive when adding real CSS value classes.
    pub fn MayContainUrl(&self) -> bool {
        match self.class_type {
            CSSValueClass::LinearGradient => false,
        }
    }
}
impl PartialEq for CSSValue {
    fn eq(&self, other: &Self) -> bool {
        self.class_type == other.class_type && self.linear().data == other.linear().data
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Color,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradientData {
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub stops: Vec<GradientStop>,
}
impl LinearGradientData {
    fn Hash(&self) -> u32 {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        let bits = |value: f64| if value == 0.0 { 0 } else { value.to_bits() };
        for value in self.start.into_iter().chain(self.end) {
            bits(value).hash(&mut hash);
        }
        for stop in &self.stops {
            bits(stop.offset).hash(&mut hash);
            for value in [
                stop.color.Param0(),
                stop.color.Param1(),
                stop.color.Param2(),
                stop.color.Alpha(),
            ] {
                bits(value as f64).hash(&mut hash);
            }
        }
        hash.finish() as u32
    }
}

// cpp: platform/graphics/gradient.h: Gradient::CreateLinear, AddColorStop
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub stops: Vec<GradientStop>,
}
// cpp: platform/graphics/gradient_generated_image.h: GradientGeneratedImage
// GeneratedImage's refcount ownership is ScopedRefPtr/Arc. This is a real
// generated-image payload with target-sized endpoints and color stops.
pub struct GradientGeneratedImage {
    pub gradient: Gradient,
    pub size: SizeF,
}
// The native Image dispatch currently has this supported concrete variant.
// Rendering the descriptor remains at the existing paint/raster boundary.
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

// cpp: core/css/css_image_generator_value.h: CSSImageGeneratorValue
#[repr(C)]
pub struct CSSImageGeneratorValue {
    value: CSSValue,
    clients: RefCell<HashMap<usize, (Member<ImageResourceObserver>, SizeAndCount)>>,
    sizes: RefCell<HashMap<(u32, u32), usize>>,
    images: RefCell<HashMap<(u32, u32), ScopedRefPtr<Image>>>,
    keep_alive: RefCell<Option<Persistent<CSSImageGeneratorValue>>>,
}
impl CSSImageGeneratorValue {
    fn new() -> Self {
        Self {
            value: CSSValue {
                class_type: CSSValueClass::LinearGradient,
            },
            clients: RefCell::new(HashMap::new()),
            sizes: RefCell::new(HashMap::new()),
            images: RefCell::new(HashMap::new()),
            keep_alive: RefCell::new(None),
        }
    }
    // cpp: core/css/css_image_generator_value.cc:79-90
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
    // cpp: core/css/css_image_generator_value.cc:91-115
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
    // cpp: core/css/css_image_generator_value.cc:117-141
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        target: &SizeF,
    ) -> ScopedRefPtr<Image> {
        let mut clients = self.clients.borrow_mut();
        // This supported resolved-gradient subset is cacheable. Source
        // CSSGradientValue::GetImage rejects unregistered observers.
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
        // Source gradients have no image for an empty target.
        if target.IsEmpty() {
            return ScopedRefPtr::default();
        }
        let key = size_key(target);
        if let Some(image) = self.images.borrow().get(&key) {
            return image.clone();
        }
        let linear = self.value.linear();
        let data = &linear.data;
        let image = ScopedRefPtr::new(Image {
            generated: GradientGeneratedImage {
                gradient: Gradient {
                    start: [
                        data.start[0] * target.width() as f64,
                        data.start[1] * target.height() as f64,
                    ],
                    end: [
                        data.end[0] * target.width() as f64,
                        data.end[1] * target.height() as f64,
                    ],
                    stops: data.stops.clone(),
                },
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

// cpp: core/css/css_gradient_value.h: CSSLinearGradientValue
#[repr(C)]
pub struct CSSLinearGradientValue {
    generator: CSSImageGeneratorValue,
    data: LinearGradientData,
}
impl CSSLinearGradientValue {
    pub fn FromResolved(data: LinearGradientData) -> Self {
        assert!(data.stops.len() >= 2);
        Self {
            generator: CSSImageGeneratorValue::new(),
            data,
        }
    }
    pub fn Generator(&self) -> &CSSImageGeneratorValue {
        &self.generator
    }
    pub fn Data(&self) -> &LinearGradientData {
        &self.data
    }
}
impl Traceable for CSSLinearGradientValue {
    fn Trace(&self, visitor: &mut Visitor) {
        self.generator.Trace(visitor);
    }
}

// cpp: core/style/style_generated_image.h, .cc
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
    // This callback is installed only on a real StyleGeneratedImage object.
    unsafe { &*(base as *const StyleImage).cast() }
}
fn css_value(base: &StyleImage) -> *mut CSSValue {
    &generated(base).generator().value as *const CSSValue as *mut CSSValue
}
static GENERATED_VTABLE: StyleImageVTable = StyleImageVTable {
    CssValue: css_value,
    // Every admitted value already contains resolved colors and stop positions.
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
    // The boundary admits only already resolved absolute colors.
    DependsOnCurrentColor: StyleImageDefaultDependsOnCurrentColor,
    IsLoadedAfterMouseover: StyleImageDefaultIsLoadedAfterMouseover,
    Trace: |base, visitor| generated(base).Trace(visitor),
    IsEqual: |base, other| {
        other.IsGeneratedImage()
            && generated(base).generator().value == generated(other).generator().value
    },
};
