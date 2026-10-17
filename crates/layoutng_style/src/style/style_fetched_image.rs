//! Source-aligned fetched `url()` style image identity.
//!
//! Chromium keeps the URL and the `ImageResourceContent` observer on
//! `StyleFetchedImage`. Rechrom's resource owner stays outside Style, so the
//! immutable computed-style object retains the same URL identity together
//! with the resource id, natural size and terminal error published by that owner.
#![allow(non_snake_case)]

use super::{
    forward::{CSSValue, ImageResourceObserver, Node},
    natural_sizing_info::NaturalSizingInfo,
    style_image::*,
};
use crate::css::{css_image_value::CSSImageValue, css_url_data::CSSUrlData};
use foundation::{gfx::SizeF, RespectImageOrientationEnum, ScopedRefPtr, Traceable, Visitor};

// cpp: core/style/style_fetched_image.h:42-110
#[repr(C)]
pub struct StyleFetchedImage {
    base: StyleImage,
    url_data: CSSUrlData,
    resource_id: u64,
    error_occurred: bool,
    natural_size: Option<SizeF>,
    resource_resolution_scale: f32,
    override_image_resolution: f32,
    is_document_image: bool,
    orientation: image_resource::ImageOrientation,
}

impl StyleFetchedImage {
    pub fn new(
        url_data: CSSUrlData,
        resource_id: u64,
        natural_size: Option<SizeF>,
        resource_resolution_scale: f32,
        override_image_resolution: f32,
        is_document_image: bool,
        orientation: image_resource::ImageOrientation,
        error_occurred: bool,
    ) -> Self {
        let mut base = StyleImage::new_for_derived(&FETCHED_VTABLE);
        base.set_image_resource_for_derived(true);
        Self {
            base,
            url_data,
            resource_id,
            error_occurred,
            natural_size,
            resource_resolution_scale: if resource_resolution_scale.is_finite()
                && resource_resolution_scale > 0.0
            {
                resource_resolution_scale
            } else {
                1.0
            },
            override_image_resolution: if override_image_resolution.is_finite()
                && override_image_resolution > 0.0
            {
                override_image_resolution
            } else {
                0.0
            },
            is_document_image,
            orientation,
        }
    }

    pub fn StyleImage(&self) -> &StyleImage {
        &self.base
    }

    pub fn SourceURL(&self) -> &str {
        self.url_data.resolved_url.as_ref()
    }

    pub fn IsDocumentImage(&self) -> bool {
        self.is_document_image
    }

    pub fn ResourceId(&self) -> u64 {
        self.resource_id
    }

    /// CSS-pixel intrinsic size after the resource resolution scale. This is
    /// the value LayoutImage's natural-size path consumes for generated
    /// `content: url(...)` children.
    pub fn NaturalSize(
        &self,
        respect_orientation: RespectImageOrientationEnum,
    ) -> Option<(f64, f64)> {
        self.natural_size.as_ref().map(|size| {
            let scale = f64::from(self.AppliedResolutionScale());
            let size = (
                f64::from(size.width()) / scale,
                f64::from(size.height()) / scale,
            );
            if respect_orientation == RespectImageOrientationEnum::kRespectImageOrientation {
                self.orientation.oriented_size(size.0, size.1)
            } else {
                size
            }
        })
    }

    pub fn ResolutionScale(&self) -> f64 {
        f64::from(self.AppliedResolutionScale())
    }

    // cpp: core/style/style_fetched_image.cc:131-139. image-set's override
    // resolution scales bitmap candidates only. A resource-owned Content-DPR
    // value still applies to SVG/document images.
    fn AppliedResolutionScale(&self) -> f32 {
        if self.override_image_resolution > 0.0 && !self.is_document_image {
            self.override_image_resolution
        } else {
            self.resource_resolution_scale
        }
    }

    pub fn Orientation(&self) -> image_resource::ImageOrientation {
        self.orientation
    }
}

impl Traceable for StyleFetchedImage {
    fn Trace(&self, _: &mut Visitor) {}
}

fn fetched(base: &StyleImage) -> &StyleFetchedImage {
    // This callback is installed only on a real StyleFetchedImage object.
    unsafe { &*(base as *const StyleImage).cast() }
}

/// Chromium's `DynamicTo<StyleFetchedImage>` at package boundaries.
pub fn FetchedImage(base: &StyleImage) -> Option<&StyleFetchedImage> {
    base.IsImageResource().then(|| fetched(base))
}

fn image_size(
    image: &StyleFetchedImage,
    multiplier: f32,
    respect_orientation: RespectImageOrientationEnum,
) -> SizeF {
    image
        .natural_size
        .as_ref()
        .map(|size| {
            let scale = multiplier / image.AppliedResolutionScale();
            let (width, height) =
                if respect_orientation == RespectImageOrientationEnum::kRespectImageOrientation {
                    image
                        .orientation
                        .oriented_size(f64::from(size.width()), f64::from(size.height()))
                } else {
                    (f64::from(size.width()), f64::from(size.height()))
                };
            SizeF::new(width as f32 * scale, height as f32 * scale)
        })
        .unwrap_or_default()
}

// cpp: core/style/style_fetched_image.cc:96-105. Each query owns a computed
// CSS image value retaining this exact StyleImage, never a replacement image.
fn css_value(base: &StyleImage) -> *mut CSSValue {
    foundation::MakeGarbageCollected(CSSImageValue::new(
        fetched(base).url_data.MakeComputed(),
        base as *const StyleImage as *mut StyleImage,
    ))
    .cast()
}

static FETCHED_VTABLE: StyleImageVTable = StyleImageVTable {
    CssValue: css_value,
    ComputedCSSValue: |base, _, _, _| css_value(base),
    // cpp: core/style/style_fetched_image.cc:107-120 and
    // core/loader/resource/image_resource_content.cc:681-692. A terminal
    // fetch/decode error is loaded, but has no renderable image.
    CanRender: |base| fetched(base).resource_id != 0 && !fetched(base).error_occurred,
    IsLoaded: |base| fetched(base).resource_id != 0 || fetched(base).error_occurred,
    IsLoading: |base| fetched(base).resource_id == 0 && !fetched(base).error_occurred,
    ErrorOccurred: |base| fetched(base).error_occurred,
    IsCorsSameOrigin: |_| true,
    GetNaturalSizingInfo: |base, multiplier, orientation| {
        let image = fetched(base);
        image
            .natural_size
            .as_ref()
            .map(|_| NaturalSizingInfo::MakeFixed(&image_size(image, multiplier, orientation)))
            .unwrap_or_else(NaturalSizingInfo::None)
    },
    ImageSize: |base, multiplier, default_size, orientation| {
        let image = fetched(base);
        if image.natural_size.is_some() {
            image_size(image, multiplier, orientation)
        } else {
            default_size.clone()
        }
    },
    HasIntrinsicSize: |base| fetched(base).natural_size.is_some(),
    AddClient: |_, _| {},
    RemoveClient: |_, _| {},
    GetImage: |_, _: &ImageResourceObserver, _: &Node, _, _| ScopedRefPtr::default(),
    Data: |base| fetched(base) as *const StyleFetchedImage as WrappedImagePtr,
    ImageScaleFactor: |base| {
        let image = fetched(base);
        if image.override_image_resolution > 0.0 {
            image.override_image_resolution
        } else {
            image.resource_resolution_scale
        }
    },
    CachedImage: StyleImageDefaultCachedImage,
    DependsOnCurrentColor: StyleImageDefaultDependsOnCurrentColor,
    IsLoadedAfterMouseover: StyleImageDefaultIsLoadedAfterMouseover,
    Trace: |base, visitor| fetched(base).Trace(visitor),
    IsEqual: |base, other| {
        FetchedImage(other).is_some_and(|other| {
            let this = fetched(base);
            this.url_data == other.url_data
                && this.resource_id == other.resource_id
                && this.error_occurred == other.error_occurred
                && this.natural_size == other.natural_size
                && EqualResolutions(
                    this.resource_resolution_scale,
                    other.resource_resolution_scale,
                )
                && EqualResolutions(
                    this.override_image_resolution,
                    other.override_image_resolution,
                )
                && this.is_document_image == other.is_document_image
                && this.orientation == other.orientation
        })
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetched_css_value_is_a_url_image() {
        let _heap = foundation::LayoutHeapScope::new();
        let fetched = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            CSSUrlData::unresolved(foundation::String::from(
                "https://example.test/background.svg#icon",
            )),
            0,
            None,
            1.0,
            0.0,
            true,
            image_resource::ImageOrientation::kDefault,
            false,
        ));
        let value = unsafe { &*fetched }.StyleImage().CssValue();
        assert!(
            !value.is_null(),
            "fetched images must expose their CSS image value"
        );
        let value = unsafe { &*value };
        assert!(value.MayContainUrl());
        assert!(!value.IsImageGeneratorValue());
    }

    #[test]
    fn computed_css_values_keep_the_same_owner_and_url_semantics() {
        use crate::css::css_url_data::ProductionURL;
        use crate::style::{computed_style::ComputedStyle, forward::CSSValuePhase};

        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        for (unresolved, resolved, expected) in [
            (
                "../image.svg",
                "https://example.test/image.svg",
                "https://example.test/image.svg",
            ),
            ("#icon", "https://example.test/page#icon", "#icon"),
            ("invalid", "", "invalid"),
        ] {
            let fetched = foundation::MakeGarbageCollected(StyleFetchedImage::new(
                CSSUrlData::new(
                    foundation::String::from(unresolved),
                    ProductionURL::FromString(resolved),
                ),
                7,
                Some(SizeF::new(40.0, 20.0)),
                1.0,
                0.0,
                true,
                image_resource::ImageOrientation::kDefault,
                false,
            ));
            let owner = unsafe { &*fetched }.StyleImage();
            let original = owner.CssValue();
            for phase in [CSSValuePhase::kComputedValue, CSSValuePhase::kResolvedValue] {
                for visited in [false, true] {
                    let computed = owner.ComputedCSSValue(initial, visited, phase);
                    assert!(!computed.is_null());
                    assert_ne!(original, computed, "each query allocates its own CSS value");
                    let value = unsafe { &*computed };
                    assert!(value == unsafe { &*original });
                    assert_eq!(value.Hash(), computed as usize as u32);
                    let image = value.AsImageValue().unwrap();
                    assert_eq!(image.CachedImage(), fetched.cast::<StyleImage>());
                    assert_eq!(image.UrlData().unresolved_url.as_ref(), expected);
                    assert_eq!(image.UrlData().resolved_url.as_ref(), resolved);
                }
            }
        }
    }

    #[test]
    fn css_value_root_retains_fetched_owner_until_value_is_collected() {
        use foundation::{LayoutHeapScope, MakeGarbageCollected, Persistent, WeakPersistent};

        let value_root;
        let weak_image;
        let weak_value;
        {
            let _heap = LayoutHeapScope::new();
            let fetched = MakeGarbageCollected(StyleFetchedImage::new(
                CSSUrlData::unresolved(foundation::String::from("https://example.test/alive.svg")),
                17,
                None,
                1.0,
                0.0,
                true,
                image_resource::ImageOrientation::kDefault,
                false,
            ));
            weak_image = WeakPersistent::from_ptr(fetched);
            let value = unsafe { &*fetched }.StyleImage().CssValue();
            weak_value = WeakPersistent::from_ptr(value);
            value_root = Persistent::from_ptr(value);
        }
        assert!(
            !weak_image.Get().is_null(),
            "returned value strongly retains its image"
        );
        let value = unsafe { &*value_root.Get() }.AsImageValue().unwrap();
        assert_eq!(value.CachedImage(), weak_image.Get().cast::<StyleImage>());
        assert_eq!(
            FetchedImage(unsafe { &*value.CachedImage() })
                .unwrap()
                .ResourceId(),
            17
        );
        assert_eq!(
            value.UrlData().resolved_url.as_ref(),
            "https://example.test/alive.svg"
        );
        drop(value_root);
        {
            let _heap = LayoutHeapScope::new();
        }
        assert!(weak_value.Get().is_null());
        assert!(
            weak_image.Get().is_null(),
            "no wrapper/image keep-alive cycle or leaked root"
        );
    }

    #[test]
    fn failed_url_images_are_terminal_immutable_snapshots() {
        let _heap = foundation::LayoutHeapScope::new();
        let image = |failed| {
            foundation::MakeGarbageCollected(StyleFetchedImage::new(
                CSSUrlData::unresolved(foundation::String::from(
                    "https://example.test/mask.svg#icon",
                )),
                0,
                None,
                1.0,
                0.0,
                false,
                image_resource::ImageOrientation::kDefault,
                failed,
            ))
        };
        let pending = image(false);
        let failed = image(true);
        let same_failure = image(true);
        let pending = unsafe { &*pending }.StyleImage();
        let failed = unsafe { &*failed }.StyleImage();
        assert!(failed.IsImageResource());
        assert!(failed.IsLoaded());
        assert!(!failed.IsLoading());
        assert!(!failed.CanRender());
        assert!(failed.ErrorOccurred());
        assert!(!failed.HasIntrinsicSize());
        assert_eq!(FetchedImage(failed).unwrap().ResourceId(), 0);
        assert_eq!(
            FetchedImage(failed).unwrap().SourceURL(),
            "https://example.test/mask.svg#icon"
        );
        assert!(
            pending != failed,
            "resource failure changes the style snapshot"
        );
        assert!(failed == unsafe { &*same_failure }.StyleImage());
        assert!(pending.IsLoading() && !pending.IsLoaded());
        assert!(!pending.ErrorOccurred(), "older snapshots stay immutable");
        let value = failed.CssValue();
        let value = unsafe { &*value }.AsImageValue().unwrap();
        assert_eq!(
            value.CachedImage(),
            failed as *const StyleImage as *mut StyleImage
        );
        assert_eq!(
            value.UrlData().resolved_url.as_ref(),
            "https://example.test/mask.svg#icon"
        );
    }

    #[test]
    fn unloaded_and_loaded_url_images_preserve_semantic_identity() {
        let _heap = foundation::LayoutHeapScope::new();
        let pending = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            CSSUrlData::unresolved(foundation::String::from("https://example.test/a.png")),
            0,
            None,
            1.0,
            0.0,
            false,
            image_resource::ImageOrientation::kDefault,
            false,
        ));
        let loaded = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            CSSUrlData::unresolved(foundation::String::from("https://example.test/a.png")),
            7,
            Some(SizeF::new(40.0, 20.0)),
            1.0,
            2.0,
            false,
            image_resource::ImageOrientation::kRotate90,
            false,
        ));
        let pending = unsafe { &*pending }.StyleImage();
        let loaded = unsafe { &*loaded }.StyleImage();
        assert!(pending.IsImageResource() && pending.IsLoading() && !pending.CanRender());
        assert!(loaded.IsLoaded() && loaded.CanRender());
        assert!(!loaded.IsLoading() && !loaded.ErrorOccurred());
        assert_eq!(FetchedImage(loaded).unwrap().ResourceId(), 7);
        assert_eq!(
            loaded.ImageSize(
                1.0,
                &SizeF::default(),
                foundation::RespectImageOrientationEnum::kRespectImageOrientation
            ),
            SizeF::new(10.0, 20.0)
        );
        assert_eq!(
            loaded.ImageSize(
                1.0,
                &SizeF::default(),
                foundation::RespectImageOrientationEnum::kDoNotRespectImageOrientation
            ),
            SizeF::new(20.0, 10.0)
        );
        let svg = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            CSSUrlData::unresolved(foundation::String::from("https://example.test/a.svg")),
            8,
            Some(SizeF::new(40.0, 20.0)),
            1.0,
            2.0,
            true,
            image_resource::ImageOrientation::kDefault,
            false,
        ));
        let svg = unsafe { &*svg }.StyleImage();
        assert_eq!(
            svg.ImageSize(
                1.0,
                &SizeF::default(),
                foundation::RespectImageOrientationEnum::kRespectImageOrientation,
            ),
            SizeF::new(40.0, 20.0),
            "image-set resolution does not scale SVG natural dimensions",
        );
        assert_eq!(
            FetchedImage(svg).unwrap().ResolutionScale(),
            1.0,
            "paint geometry consumes the same SVG sizing resolution",
        );
        assert_eq!(svg.ImageScaleFactor(), 2.0);
        assert!(pending != loaded);
    }
}
