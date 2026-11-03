//! Source-aligned generated-image client and image-size cache.

#![allow(non_snake_case)]

use super::{
    css_gradient_value::Image,
    css_to_length_conversion_data::ContainerSizes,
    css_value::{CSSValue, CSSValueClass},
};
use crate::style::{
    computed_style::ComputedStyle,
    forward::{ImageResourceObserver, Node},
};
use foundation::{gfx::SizeF, Member, Persistent, ScopedRefPtr, Traceable, Visitor};
use std::{cell::RefCell, collections::HashMap};

fn size_key(size: &SizeF) -> (u32, u32) {
    (size.width().to_bits(), size.height().to_bits())
}

// cpp: core/css/css_image_generator_value.h:56-72
#[derive(Default)]
struct GeneratedImageCache {
    sizes: RefCell<HashMap<(u32, u32), usize>>,
    images: RefCell<HashMap<(u32, u32), ScopedRefPtr<Image>>>,
}

impl GeneratedImageCache {
    // cpp: core/css/css_image_generator_value.cc:40-51
    fn GetImage(&self, size: &SizeF) -> ScopedRefPtr<Image> {
        if size.IsEmpty() {
            return ScopedRefPtr::default();
        }
        self.images
            .borrow()
            .get(&size_key(size))
            .cloned()
            .unwrap_or_default()
    }

    // cpp: core/css/css_image_generator_value.cc:53-57
    fn PutImage(&self, size: &SizeF, image: ScopedRefPtr<Image>) {
        assert!(!size.IsEmpty());
        self.images.borrow_mut().insert(size_key(size), image);
    }

    // cpp: core/css/css_image_generator_value.cc:59-62
    fn AddSize(&self, size: &SizeF) {
        assert!(!size.IsEmpty());
        *self.sizes.borrow_mut().entry(size_key(size)).or_default() += 1;
    }

    // cpp: core/css/css_image_generator_value.cc:64-72
    fn RemoveSize(&self, size: &SizeF) {
        assert!(!size.IsEmpty());
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
}

// cpp: core/css/css_image_generator_value.h:74-84
#[derive(Clone, Default)]
struct SizeAndCount {
    size: SizeF,
    count: usize,
}

// cpp: core/css/css_image_generator_value.h:89-134
#[repr(C)]
pub struct CSSImageGeneratorValue {
    value: CSSValue,
    clients: RefCell<HashMap<usize, (Member<ImageResourceObserver>, SizeAndCount)>>,
    cached_images: GeneratedImageCache,
    keep_alive: RefCell<Option<Persistent<CSSImageGeneratorValue>>>,
}

impl CSSImageGeneratorValue {
    // cpp: core/css/css_image_generator_value.cc:74-75
    pub(crate) fn new(class_type: CSSValueClass) -> Self {
        Self {
            value: CSSValue::new(class_type),
            clients: RefCell::new(HashMap::new()),
            cached_images: GeneratedImageCache::default(),
            keep_alive: RefCell::new(None),
        }
    }

    pub(crate) fn Value(&self) -> &CSSValue {
        &self.value
    }

    // cpp: core/css/css_image_generator_value.cc:79-89
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

    // cpp: core/css/css_image_generator_value.cc:91-110
    pub fn RemoveClient(&self, client: *mut ImageResourceObserver) {
        assert!(!client.is_null());
        let mut clients = self.clients.borrow_mut();
        let (_, state) = clients
            .get_mut(&(client as usize))
            .expect("registered image client");
        if !state.size.IsEmpty() {
            self.cached_images.RemoveSize(&state.size);
            state.size = SizeF::default();
        }
        state.count -= 1;
        if state.count == 0 {
            clients.remove(&(client as usize));
        }
        if clients.is_empty() {
            self.keep_alive.borrow_mut().take();
        }
    }

    // cpp: core/css/css_image_generator_value.cc:117-136
    pub(crate) fn HasClient(&self, client: &ImageResourceObserver) -> bool {
        self.clients
            .borrow()
            .contains_key(&(client as *const _ as usize))
    }

    pub(crate) fn GetCachedImage(
        &self,
        client: &ImageResourceObserver,
        size: &SizeF,
    ) -> ScopedRefPtr<Image> {
        let mut clients = self.clients.borrow_mut();
        if let Some((_, state)) = clients.get_mut(&(client as *const _ as usize)) {
            if state.size != *size {
                if !state.size.IsEmpty() {
                    self.cached_images.RemoveSize(&state.size);
                    state.size = SizeF::default();
                }
                if !size.IsEmpty() {
                    self.cached_images.AddSize(size);
                    state.size = size.clone();
                }
            }
        }
        self.cached_images.GetImage(size)
    }

    // cpp: core/css/css_image_generator_value.cc:138-141
    pub(crate) fn PutImage(&self, size: &SizeF, image: ScopedRefPtr<Image>) {
        self.cached_images.PutImage(size, image);
    }

    // cpp: core/css/css_image_generator_value.cc:143-169. This package's
    // admitted generator classes are the three gradient classes.
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        node: &Node,
        style: &ComputedStyle,
        container_sizes: &ContainerSizes,
        target_size: &SizeF,
    ) -> ScopedRefPtr<Image> {
        self.value
            .gradient()
            .GetImage(observer, node, style, container_sizes, target_size)
    }

    // cpp: core/css/css_image_generator_value.cc:181-195
    pub fn IsUsingCurrentColor(&self) -> bool {
        self.value.gradient().IsUsingCurrentColor()
    }

    // cpp: core/css/css_image_generator_value.cc:197-208. All admitted
    // gradients have crossed style resolution, so no container-relative unit
    // remains in their normalized geometry or stops.
    pub fn IsUsingContainerRelativeUnits(&self) -> bool {
        self.value.gradient().IsUsingContainerRelativeUnits()
    }

    // cpp: core/css/css_image_generator_value.cc:210-226. Gradients contain
    // no cross-origin resource in the currently admitted input classes.
    pub fn IsCorsSameOrigin(&self) -> bool {
        true
    }

    // cpp: core/css/css_image_generator_value.cc:228-246 and
    // core/css/css_gradient_value.cc:922-933.
    pub fn KnownToBeOpaque(&self, style: &ComputedStyle) -> bool {
        self.value
            .gradient()
            .Data()
            .Stops()
            .iter()
            .filter(|stop| !stop.is_hint)
            .all(|stop| {
                stop.color
                    .Resolve(style.GetCurrentColor(None), style.UsedColorScheme(), None)
                    .IsOpaque()
            })
    }

    // cpp: core/css/css_image_generator_value.cc:112-115
    fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        for (client, _) in self.clients.borrow().values() {
            visitor.Trace(client);
        }
    }
}

impl Traceable for CSSImageGeneratorValue {
    fn Trace(&self, visitor: &mut Visitor) {
        self.TraceAfterDispatch(visitor);
    }
}
