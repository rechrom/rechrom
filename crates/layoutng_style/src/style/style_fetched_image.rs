//! Source-aligned fetched `url()` style image identity.
//!
//! Chromium keeps the URL and the `ImageResourceContent` observer on
//! `StyleFetchedImage`. Rechrom's resource owner stays outside Style, so the
//! immutable computed-style object retains the same URL identity together
//! with the resource id and natural size published by that owner.
#![allow(non_snake_case)]

use super::{
    forward::{CSSValue, CSSValuePhase, ImageResourceObserver, Node},
    natural_sizing_info::NaturalSizingInfo,
    style_image::*,
};
use foundation::{gfx::SizeF, RespectImageOrientationEnum, ScopedRefPtr, Traceable, Visitor};

// cpp: core/style/style_fetched_image.h:42-110
#[repr(C)]
pub struct StyleFetchedImage {
    base: StyleImage,
    source_url: String,
    resource_id: u64,
    natural_size: Option<SizeF>,
    resource_resolution_scale: f32,
    override_image_resolution: f32,
    is_document_image: bool,
    orientation: image_resource::ImageOrientation,
}

impl StyleFetchedImage {
    pub fn new(
        source_url: String,
        resource_id: u64,
        natural_size: Option<SizeF>,
        resource_resolution_scale: f32,
        override_image_resolution: f32,
        is_document_image: bool,
        orientation: image_resource::ImageOrientation,
    ) -> Self {
        let mut base = StyleImage::new_for_derived(&FETCHED_VTABLE);
        base.set_image_resource_for_derived(true);
        Self {
            base,
            source_url,
            resource_id,
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
        &self.source_url
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

static FETCHED_VTABLE: StyleImageVTable = StyleImageVTable {
    // CSS serialization is owned by the translated CSS value package. The
    // typed URL accessor above is the package-neutral paint/resource boundary.
    CssValue: |_| std::ptr::null_mut::<CSSValue>(),
    ComputedCSSValue: |_, _, _, _: CSSValuePhase| std::ptr::null_mut::<CSSValue>(),
    CanRender: |base| fetched(base).resource_id != 0,
    IsLoaded: |base| fetched(base).resource_id != 0,
    IsLoading: |base| fetched(base).resource_id == 0,
    ErrorOccurred: StyleImageDefaultErrorOccurred,
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
            this.source_url == other.source_url
                && this.resource_id == other.resource_id
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
    fn unloaded_and_loaded_url_images_preserve_semantic_identity() {
        let _heap = foundation::LayoutHeapScope::new();
        let pending = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            "https://example.test/a.png".into(),
            0,
            None,
            1.0,
            0.0,
            false,
            image_resource::ImageOrientation::kDefault,
        ));
        let loaded = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            "https://example.test/a.png".into(),
            7,
            Some(SizeF::new(40.0, 20.0)),
            1.0,
            2.0,
            false,
            image_resource::ImageOrientation::kRotate90,
        ));
        let pending = unsafe { &*pending }.StyleImage();
        let loaded = unsafe { &*loaded }.StyleImage();
        assert!(pending.IsImageResource() && pending.IsLoading() && !pending.CanRender());
        assert!(loaded.IsLoaded() && loaded.CanRender());
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
            "https://example.test/a.svg".into(),
            8,
            Some(SizeF::new(40.0, 20.0)),
            1.0,
            2.0,
            true,
            image_resource::ImageOrientation::kDefault,
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
