//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! element_style_resources.h: 99 / 40 / 26 / 14 / 0.
//! element_style_resources.cc: 532 / 316 / 310 / 6 / 0.
//! Effective excludes copyright/comments, blank/preprocessor/include/namespace
//! lines and lines containing only brackets/punctuation. Header omissions are
//! forward declarations 34-38,41, allocation macros 45,65, access labels
//! 47,57,67,81 and deleted copy boilerplate 69-70. cc omissions are allocation
//! macro/access labels 66,68,85 and debug assertions 154,233,260. All remaining
//! production declarations and cc:65-530 map, including the source-local loader,
//! cache selection, image-set/crossfade recursion, final builder property walks,
//! SVG/local-mask handling and images-before-SVG load order. External factories
//! bind the existing CSSValue payloads and native StyleImage; no resource/style
//! model or network/renderer dependency is introduced. RetainStyleImage is an
//! ownership bridge only: selection, mutations and traversal stay in Rust.
#![allow(non_snake_case)]
use super::style_resolver_state::{ResolverURIValue, ResolverValue, StyleResolverStateBackend};
use crate::css_value::{CSSValueDispatch, CSSValuePayload};
use foundation::{AtomicString, CSSPropertyID, Member};
use layoutng_style::style::clip_path_operation::{ClipPathOperation, OperationType as ClipType};
use layoutng_style::style::computed_style::ComputedStyleBuilder;
use layoutng_style::style::content_data::ImageContentData;
use layoutng_style::style::fill_layer::FillLayer;
use layoutng_style::style::filter_operation::{FilterOperation, OperationType as FilterType};
use layoutng_style::style::filter_operations::FilterOperationVector;
use layoutng_style::style::forward::SVGResource as NativeSVGResource;
use layoutng_style::style::style_image::StyleImage;
use layoutng_style::style::style_svg_resource::StyleSVGResource;
use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

type ImageValue<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSImageValue;
type ImageSetValue<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSImageSetValue;
type ImageSetOptionValue<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSImageSetOptionValue;
type CrossfadeValue<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSCrossfadeValue;

/// Only external class operations: DOM/tree scope, CSS image cache/options,
/// conversion data, missing concrete StyleImage subclasses and resource loader.
/// Every dispatch, property choice and traversal from this source stays below.
pub trait ElementStyleResourcesBackend: StyleResolverStateBackend {
    type CrossOrigin: Copy;
    fn ResourceDocument<'a>(&self, element: &'a Self::Element) -> &'a Self::Document;
    fn CrossOriginNotSet(&self) -> Self::CrossOrigin;
    fn CrossOriginAnonymous(&self) -> Self::CrossOrigin;
    fn EmptyContainerSizes(&self) -> Self::ContainerSizes;
    fn PreCachedContainerSizesCopy(
        &self,
        data: &Self::LengthConversionData,
    ) -> Self::ContainerSizes;
    fn ImageIsCachePending(&self, value: &ImageValue<Self>) -> bool;
    fn ImageRestoreCachedResource(&self, value: &ImageValue<Self>, document: &Self::Document);
    fn ImageCachedImage(&self, value: &ImageValue<Self>) -> Option<Rc<StyleImage>>;
    fn ImageCacheImage(
        &self,
        value: &ImageValue<Self>,
        document: &Self::Document,
        cross_origin: Self::CrossOrigin,
        override_resolution: f32,
    ) -> Option<Rc<StyleImage>>;
    fn ImageIsLocal(&self, value: &ImageValue<Self>, document: &Self::Document) -> bool;
    fn ImageNormalizedFragment(&self, value: &ImageValue<Self>) -> AtomicString;
    fn ImageEnsureSVGResource(&self, value: &ImageValue<Self>) -> Option<Rc<Self::SVGResource>>;
    fn ImageSetIsCachePending(&self, value: &ImageSetValue<Self>, device_scale: f32) -> bool;
    fn ImageSetCachedImage(
        &self,
        value: &ImageSetValue<Self>,
        device_scale: f32,
    ) -> Option<Rc<StyleImage>>;
    fn ImageSetCacheImage(
        &self,
        value: &ImageSetValue<Self>,
        image: Option<Rc<StyleImage>>,
        device_scale: f32,
    ) -> Option<Rc<StyleImage>>;
    fn ImageSetBestOption(
        &self,
        value: &ImageSetValue<Self>,
        resolver: &Self::LengthConversionData,
        device_scale: f32,
    ) -> Option<Rc<ImageSetOptionValue<Self>>>;
    fn ImageSetOptionImage(&self, option: &ImageSetOptionValue<Self>) -> Rc<ResolverValue<Self>>;
    fn ImageSetOptionResolution(
        &self,
        option: &ImageSetOptionValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> f32;
    fn CrossfadeImages(&self, value: &CrossfadeValue<Self>) -> Vec<Rc<ResolverValue<Self>>>;
    fn GradientUsesContainerUnits(&self, value: &ResolverValue<Self>) -> bool;
    fn NewStyleGeneratedImage(
        &self,
        value: Rc<ResolverValue<Self>>,
        sizes: &Self::ContainerSizes,
    ) -> Rc<StyleImage>;
    fn NewStylePendingImage(&self, value: Rc<ResolverValue<Self>>) -> Rc<StyleImage>;
    fn NewStyleCrossfadeImage(
        &self,
        value: Rc<ResolverValue<Self>>,
        images: Vec<Option<Rc<StyleImage>>>,
        resolver: &Self::LengthConversionData,
    ) -> Rc<StyleImage>;
    fn PendingImageCssValue(&self, image: &StyleImage) -> Rc<ResolverValue<Self>>;
    fn NewLocalStyleMaskSourceImage(
        &self,
        resource: Option<Rc<Self::SVGResource>>,
        value: Rc<ResolverValue<Self>>,
    ) -> Rc<StyleImage>;
    fn NewFetchedStyleMaskSourceImage(
        &self,
        image: Option<Rc<StyleImage>>,
        resource: Option<Rc<Self::SVGResource>>,
        value: Rc<ResolverValue<Self>>,
    ) -> Rc<StyleImage>;
    fn URIResourceReference(&self, value: &ResolverURIValue<Self>)
        -> Option<Rc<Self::SVGResource>>;
    fn URINormalizedFragment(&self, value: &ResolverURIValue<Self>) -> AtomicString;
    fn TreeScopeResourceForId(
        &self,
        element: &Self::Element,
        id: &AtomicString,
    ) -> Option<Rc<Self::SVGResource>>;
    fn SVGResourceFromNative(
        &self,
        resource: *mut NativeSVGResource,
    ) -> Option<Rc<Self::SVGResource>>;
    fn ReferenceFilterResource(&self, operation: &FilterOperation)
        -> Option<Rc<Self::SVGResource>>;
    fn ReferenceClipPathResource(
        &self,
        operation: &dyn ClipPathOperation,
    ) -> Option<Rc<Self::SVGResource>>;
    fn LoadSVGResource(
        &self,
        resource: &Self::SVGResource,
        document: &Self::Document,
        cross_origin: Self::CrossOrigin,
    );
    /// Ownership-only bridge into the native Member/GC builder. The returned
    /// pointer denotes this exact image and remains rooted for the built style.
    /// It must retain the image even after this per-element resources object dies.
    fn RetainStyleImage(
        &self,
        builder: &mut ComputedStyleBuilder,
        image: Rc<StyleImage>,
    ) -> *mut StyleImage;
}

pub struct PreCachedContainerSizes<B: ElementStyleResourcesBackend> {
    conversion_data_: Option<Rc<RefCell<B::LengthConversionData>>>,
    cache_: OnceCell<B::ContainerSizes>,
}
impl<B: ElementStyleResourcesBackend> PreCachedContainerSizes<B> {
    pub fn new(data: Option<Rc<RefCell<B::LengthConversionData>>>) -> Self {
        Self {
            conversion_data_: data,
            cache_: OnceCell::new(),
        }
    }
    pub fn Get(&self, backend: &B) -> &B::ContainerSizes {
        self.cache_.get_or_init(|| match &self.conversion_data_ {
            Some(data) => backend.PreCachedContainerSizesCopy(&data.borrow()),
            None => backend.EmptyContainerSizes(),
        })
    }
}

pub struct ElementStyleResources<B: ElementStyleResourcesBackend> {
    backend: Rc<B>,
    element_: Rc<B::Element>,
    pending_image_properties_: HashSet<CSSPropertyID>,
    pending_svg_resource_properties_: HashSet<CSSPropertyID>,
    device_scale_factor_: f32,
    pre_cached_container_sizes_: PreCachedContainerSizes<B>,
}
impl<B: ElementStyleResourcesBackend> ElementStyleResources<B> {
    pub fn new(backend: Rc<B>, element: Rc<B::Element>, device_scale_factor: f32) -> Self {
        Self {
            backend,
            element_: element,
            pending_image_properties_: HashSet::new(),
            pending_svg_resource_properties_: HashSet::new(),
            device_scale_factor_: device_scale_factor,
            pre_cached_container_sizes_: PreCachedContainerSizes::new(None),
        }
    }
    fn IsPending(&self, value: &ResolverValue<B>) -> bool {
        match value.Payload() {
            CSSValuePayload::kImageClass(image) => self.backend.ImageIsCachePending(image),
            CSSValuePayload::kPaintClass(_) | CSSValuePayload::kCrossfadeClass(_) => true,
            CSSValuePayload::kImageSetClass(set) => self
                .backend
                .ImageSetIsCachePending(set, self.device_scale_factor_),
            _ if value.IsGradientValue() => false,
            _ => unreachable!("unsupported style image CSSValue"),
        }
    }
    fn CachedStyleImage(&self, value: Rc<ResolverValue<B>>) -> Option<Rc<StyleImage>> {
        match value.Payload() {
            CSSValuePayload::kImageClass(image) => {
                self.backend.ImageRestoreCachedResource(
                    image,
                    self.backend.ResourceDocument(&self.element_),
                );
                self.backend.ImageCachedImage(image)
            }
            CSSValuePayload::kImageSetClass(set) => self
                .backend
                .ImageSetCachedImage(set, self.device_scale_factor_),
            _ if value.IsGradientValue() => {
                let empty;
                let sizes = if self.backend.GradientUsesContainerUnits(&value) {
                    self.pre_cached_container_sizes_.Get(&self.backend)
                } else {
                    empty = self.backend.EmptyContainerSizes();
                    &empty
                };
                Some(self.backend.NewStyleGeneratedImage(value, sizes))
            }
            _ => unreachable!("uncached style image"),
        }
    }
    pub fn GetStyleImage(
        &mut self,
        property: CSSPropertyID,
        value: Rc<ResolverValue<B>>,
    ) -> Option<Rc<StyleImage>> {
        if value.IsIdentifierValue() {
            return None;
        }
        if self.IsPending(&value) {
            self.pending_image_properties_.insert(property);
            return Some(self.backend.NewStylePendingImage(value));
        }
        self.CachedStyleImage(value)
    }
    pub fn GetSVGResourceFromValue(
        &mut self,
        property: CSSPropertyID,
        value: &ResolverURIValue<B>,
    ) -> Option<Rc<B::SVGResource>> {
        if self
            .backend
            .URIIsLocal(value, self.backend.ResourceDocument(&self.element_))
        {
            return self.backend.TreeScopeResourceForId(
                &self.element_,
                &self.backend.URINormalizedFragment(value),
            );
        }
        if AllowExternalResources(property) {
            self.pending_svg_resource_properties_.insert(property);
            return self.backend.URIResourceReference(value);
        }
        None
    }
    pub fn UpdateLengthConversionData(
        &mut self,
        data: Option<Rc<RefCell<B::LengthConversionData>>>,
    ) {
        self.pre_cached_container_sizes_ = PreCachedContainerSizes::new(data);
    }
    pub fn LoadPendingResources(
        &mut self,
        builder: &mut ComputedStyleBuilder,
        resolver: &B::LengthConversionData,
    ) {
        self.LoadPendingImages(builder, resolver);
        self.LoadPendingSVGResources(builder);
    }
    fn PendingCssValue(&self, image: *mut StyleImage) -> Option<Rc<ResolverValue<B>>> {
        // Builder-owned Member pointers remain rooted by the builder. Only the
        // actual pending subclass is handed to its required CSSValue owner.
        unsafe { image.as_ref() }
            .filter(|image| image.IsPendingImage())
            .map(|image| self.backend.PendingImageCssValue(image))
    }
    fn LoadMaskSource(&self, value: Rc<ResolverValue<B>>) -> Option<Rc<StyleImage>> {
        let CSSValuePayload::kImageClass(image_value) = value.Payload() else {
            return None;
        };
        let document = self.backend.ResourceDocument(&self.element_);
        if self.backend.ImageIsLocal(image_value, document) {
            let resource = self.backend.TreeScopeResourceForId(
                &self.element_,
                &self.backend.ImageNormalizedFragment(image_value),
            );
            return Some(self.backend.NewLocalStyleMaskSourceImage(resource, value));
        }
        let image = self.backend.ImageCacheImage(
            image_value,
            document,
            self.backend.CrossOriginAnonymous(),
            0.0,
        );
        let resource = self.backend.ImageEnsureSVGResource(image_value);
        let result = self
            .backend
            .NewFetchedStyleMaskSourceImage(image, resource, value);
        Some(result)
    }
    fn LoadResourcesForFilter(&self, operations: &FilterOperationVector) {
        for operation in operations.iter() {
            let operation = unsafe { &*operation.Get() };
            if operation.GetType() != FilterType::kReference {
                continue;
            }
            if let Some(resource) = self.backend.ReferenceFilterResource(operation) {
                self.backend.LoadSVGResource(
                    &resource,
                    self.backend.ResourceDocument(&self.element_),
                    self.backend.CrossOriginNotSet(),
                );
            }
        }
    }
    fn GetSVGResourceOrNull(&self, resource: *mut StyleSVGResource) -> Option<Rc<B::SVGResource>> {
        unsafe { resource.as_ref() }
            .and_then(|resource| self.backend.SVGResourceFromNative(resource.Resource()))
    }
    fn GetSingleSVGResource(
        &self,
        property: CSSPropertyID,
        builder: &ComputedStyleBuilder,
    ) -> Option<Rc<B::SVGResource>> {
        match property {
            CSSPropertyID::kClipPath => {
                let operation = builder.MutableClipPath()?;
                let operation = unsafe { &*operation };
                if operation.GetType() == ClipType::kReference {
                    self.backend.ReferenceClipPathResource(operation)
                } else {
                    None
                }
            }
            CSSPropertyID::kFill => self.GetSVGResourceOrNull(builder.FillPaint().Resource()),
            CSSPropertyID::kMarkerEnd => self.GetSVGResourceOrNull(builder.MarkerEndResource()),
            CSSPropertyID::kMarkerMid => self.GetSVGResourceOrNull(builder.MarkerMidResource()),
            CSSPropertyID::kMarkerStart => self.GetSVGResourceOrNull(builder.MarkerStartResource()),
            CSSPropertyID::kStroke => self.GetSVGResourceOrNull(builder.StrokePaint().Resource()),
            _ => unreachable!("not a single SVG resource property"),
        }
    }
    fn LoadPendingSVGResources(&self, builder: &mut ComputedStyleBuilder) {
        for property in &self.pending_svg_resource_properties_ {
            match property {
                CSSPropertyID::kBackdropFilter => {
                    self.LoadResourcesForFilter(builder.MutableBackdropFilterOperations())
                }
                CSSPropertyID::kFilter => {
                    self.LoadResourcesForFilter(builder.MutableFilterOperations())
                }
                CSSPropertyID::kClipPath
                | CSSPropertyID::kFill
                | CSSPropertyID::kMarkerEnd
                | CSSPropertyID::kMarkerMid
                | CSSPropertyID::kMarkerStart
                | CSSPropertyID::kStroke => {
                    if let Some(resource) = self.GetSingleSVGResource(*property, builder) {
                        self.backend.LoadSVGResource(
                            &resource,
                            self.backend.ResourceDocument(&self.element_),
                            self.backend.CrossOriginAnonymous(),
                        );
                    }
                }
                _ => unreachable!("unsupported pending SVG property"),
            }
        }
    }
    fn RetainImage(
        &self,
        builder: &mut ComputedStyleBuilder,
        image: Option<Rc<StyleImage>>,
    ) -> *mut StyleImage {
        image.map_or(std::ptr::null_mut(), |image| {
            self.backend.RetainStyleImage(builder, image)
        })
    }
    fn LoadPendingImages(
        &self,
        builder: &mut ComputedStyleBuilder,
        resolver: &B::LengthConversionData,
    ) {
        let loader = StyleImageLoader { resources: self };
        let not_set = self.backend.CrossOriginNotSet();
        let anonymous = self.backend.CrossOriginAnonymous();
        // Iterate properties, then inspect the final builder. Overridden pending
        // values therefore cause no fetch. Native list nodes remain builder-owned
        // throughout their traversal; image replacement cannot invalidate nodes.
        for property in &self.pending_image_properties_ {
            match property {
                CSSPropertyID::kBackgroundImage | CSSPropertyID::kMaskImage => {
                    let mask = *property == CSSPropertyID::kMaskImage;
                    let mut layer: *mut FillLayer = if mask {
                        builder.AccessMaskLayers()
                    } else {
                        builder.AccessBackgroundLayers()
                    } as *mut _;
                    while !layer.is_null() {
                        let layer_ref = unsafe { &mut *layer };
                        if let Some(value) = self.PendingCssValue(layer_ref.GetImage()) {
                            let image = if mask {
                                self.LoadMaskSource(value.clone()).or_else(|| {
                                    loader.Load(value, builder, resolver, anonymous, 0.0)
                                })
                            } else {
                                loader.Load(value, builder, resolver, not_set, 0.0)
                            };
                            layer_ref.SetImage(self.RetainImage(builder, image));
                        }
                        layer = layer_ref.NextMut();
                    }
                }
                CSSPropertyID::kContent => {
                    let mut content = builder.GetContentData();
                    while let Some(pointer) = content {
                        let content_data = unsafe { &mut *pointer };
                        if content_data.IsImage() {
                            // The native ContentData contract identifies the
                            // concrete ImageContentData object for this cast.
                            let image_content = unsafe { &mut *(pointer as *mut ImageContentData) };
                            if let Some(value) = self.PendingCssValue(image_content.GetImage()) {
                                let image = loader.Load(value, builder, resolver, not_set, 0.0);
                                image_content.SetImage(self.RetainImage(builder, image));
                            }
                        }
                        content = content_data.Next();
                    }
                }
                CSSPropertyID::kCursor => {
                    if let Some(cursors) = unsafe { builder.Cursors().as_mut() } {
                        for cursor in cursors.iter_mut() {
                            if let Some(value) = self.PendingCssValue(cursor.GetImage()) {
                                let image = loader.Load(value, builder, resolver, not_set, 0.0);
                                cursor.SetImage(self.RetainImage(builder, image));
                            }
                        }
                    }
                }
                CSSPropertyID::kListStyleImage => {
                    if let Some(value) = self.PendingCssValue(builder.ListStyleImage().Get()) {
                        let image = loader.Load(value, builder, resolver, not_set, 0.0);
                        let pointer = self.RetainImage(builder, image);
                        builder.SetListStyleImage(&Member::from_ptr(pointer));
                    }
                }
                CSSPropertyID::kBorderImageSource => {
                    if let Some(value) = self.PendingCssValue(builder.BorderImage().GetImage()) {
                        let image = loader.Load(value, builder, resolver, not_set, 0.0);
                        let pointer = self.RetainImage(builder, image);
                        builder.SetBorderImageSource(pointer);
                    }
                }
                CSSPropertyID::kWebkitBoxReflect => {
                    if let Some(reflection) = unsafe { builder.BoxReflect().as_mut() } {
                        if let Some(value) = self.PendingCssValue(reflection.Mask().GetImage()) {
                            let image = loader.Load(value, builder, resolver, not_set, 0.0);
                            // NinePieceImage's COW clone retains all six original
                            // slices/fill/outset/repeat fields; only image changes.
                            let mut mask = reflection.Mask().clone();
                            mask.SetImage(self.RetainImage(builder, image));
                            reflection.SetMask(&mask);
                        }
                    }
                }
                CSSPropertyID::kWebkitMaskBoxImageSource => {
                    if let Some(value) = self.PendingCssValue(builder.MaskBoxImageSource()) {
                        let image = loader.Load(value, builder, resolver, not_set, 0.0);
                        let pointer = self.RetainImage(builder, image);
                        builder.SetMaskBoxImageSource(pointer);
                    }
                }
                CSSPropertyID::kShapeOutside => {
                    if let Some(shape) = unsafe { builder.ShapeOutside().as_mut() } {
                        if let Some(value) = self.PendingCssValue(shape.GetImage()) {
                            let image = loader.Load(value, builder, resolver, anonymous, 0.0);
                            shape.SetImage(self.RetainImage(builder, image));
                        }
                    }
                }
                _ => unreachable!("unsupported pending image property"),
            }
        }
    }
}

fn AllowExternalResources(property: CSSPropertyID) -> bool {
    matches!(
        property,
        CSSPropertyID::kBackdropFilter
            | CSSPropertyID::kClipPath
            | CSSPropertyID::kFill
            | CSSPropertyID::kFilter
            | CSSPropertyID::kMarkerEnd
            | CSSPropertyID::kMarkerMid
            | CSSPropertyID::kMarkerStart
            | CSSPropertyID::kStroke
    )
}

/// Source-local loader. Its builder is passed on each call so native list
/// traversal can borrow individual nodes without aliasing a mutable builder.
/// This is not a backend: all recursion and type selection remain here.
struct StyleImageLoader<'a, B: ElementStyleResourcesBackend> {
    resources: &'a ElementStyleResources<B>,
}
impl<B: ElementStyleResourcesBackend> StyleImageLoader<'_, B> {
    fn Load(
        &self,
        value: Rc<ResolverValue<B>>,
        builder: &mut ComputedStyleBuilder,
        resolver: &B::LengthConversionData,
        cross_origin: B::CrossOrigin,
        override_resolution: f32,
    ) -> Option<Rc<StyleImage>> {
        let resources = self.resources;
        let backend = &resources.backend;
        match value.Payload() {
            CSSValuePayload::kImageClass(image) => backend.ImageCacheImage(
                image,
                backend.ResourceDocument(&resources.element_),
                cross_origin,
                override_resolution,
            ),
            CSSValuePayload::kPaintClass(_) => {
                let image = backend.NewStyleGeneratedImage(value, &backend.EmptyContainerSizes());
                let pointer = backend.RetainStyleImage(builder, image.clone());
                builder.AddPaintImage(pointer);
                Some(image)
            }
            CSSValuePayload::kCrossfadeClass(crossfade) => {
                let images = backend
                    .CrossfadeImages(crossfade)
                    .into_iter()
                    .map(|image| self.CrossfadeArgument(image, builder, resolver, cross_origin))
                    .collect();
                Some(backend.NewStyleCrossfadeImage(value, images, resolver))
            }
            CSSValuePayload::kImageSetClass(image_set) => {
                let image = self.ResolveImageSet(image_set, builder, resolver, cross_origin);
                backend.ImageSetCacheImage(image_set, image, resources.device_scale_factor_)
            }
            _ if value.IsGradientValue() => {
                let empty;
                let sizes = if backend.GradientUsesContainerUnits(&value) {
                    resources.pre_cached_container_sizes_.Get(backend)
                } else {
                    empty = backend.EmptyContainerSizes();
                    &empty
                };
                Some(backend.NewStyleGeneratedImage(value, sizes))
            }
            _ => unreachable!("unsupported loaded CSS image"),
        }
    }
    fn CrossfadeArgument(
        &self,
        value: Rc<ResolverValue<B>>,
        builder: &mut ComputedStyleBuilder,
        resolver: &B::LengthConversionData,
        cross_origin: B::CrossOrigin,
    ) -> Option<Rc<StyleImage>> {
        if value.IsIdentifierValue() || value.IsPaintValue() {
            return None;
        }
        self.Load(value, builder, resolver, cross_origin, 0.0)
    }
    fn ResolveImageSet(
        &self,
        value: &ImageSetValue<B>,
        builder: &mut ComputedStyleBuilder,
        resolver: &B::LengthConversionData,
        cross_origin: B::CrossOrigin,
    ) -> Option<Rc<StyleImage>> {
        let backend = &self.resources.backend;
        let option =
            backend.ImageSetBestOption(value, resolver, self.resources.device_scale_factor_)?;
        let image = backend.ImageSetOptionImage(&option);
        if !image.IsImageValue() && !image.IsGradientValue() {
            return None;
        }
        self.Load(
            image,
            builder,
            resolver,
            cross_origin,
            backend.ImageSetOptionResolution(&option, resolver),
        )
    }
}
