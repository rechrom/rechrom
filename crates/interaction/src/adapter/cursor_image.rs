//! Cursor image projections retain the resource's real immutable current frame.
//! Source: cursor_data.h; style_fetched_image.cc:107-182;
//! event_handler.cc:641-722; image.cc:396-430.
#![allow(non_snake_case)]
use crate::core::input::event_handler_cursor as source;
use image_resource::{DocumentImageBitmapRequest, ImmutableImageBitmap};
use layoutng_style::style::{
    cursor_data::CursorData, forward::ImageResourceContent, style_image::StyleImage,
};
use std::{rc::Rc, sync::Arc};

/// Per-item call-scoped projection, never a copied computed cursor list.
/// A preceding usable image prevents later fallbacks from being visited.
pub(super) struct CursorDataProjection<'a> {
    data: &'a CursorData,
    image: Option<CursorStyleImageProjection<'a>>,
}
impl<'a> CursorDataProjection<'a> {
    pub(super) fn new(data: &'a CursorData, gap: super::page_cursor::CursorGap) -> Self {
        Self {
            data,
            image: unsafe { data.GetImage().as_ref() }.map(|image| CursorStyleImageProjection {
                image,
                content: unsafe { image.CachedImage().as_ref() }
                    .map(|content| CursorImageProjection { content, gap }),
            }),
        }
    }
}
impl source::CursorData for CursorDataProjection<'_> {
    fn GetImage(&self) -> Option<&dyn source::CursorStyleImage> {
        self.image.as_ref().map(|image| image as _)
    }
    fn HotSpot(&self) -> source::Point {
        let point = self.data.HotSpot();
        source::Point {
            x: point.x(),
            y: point.y(),
        }
    }
    fn HotSpotSpecified(&self) -> bool {
        self.data.HotSpotSpecified()
    }
}
struct CursorStyleImageProjection<'a> {
    image: &'a StyleImage,
    content: Option<CursorImageProjection<'a>>,
}
impl source::CursorStyleImage for CursorStyleImageProjection<'_> {
    fn CanRender(&self) -> bool {
        self.image.CanRender() && self.content.is_some()
    }
    fn CachedImage(&self) -> Option<&dyn source::CursorImageContent> {
        self.content.as_ref().map(|content| content as _)
    }
    fn ImageScaleFactor(&self) -> f32 {
        self.image.ImageScaleFactor()
    }
    fn ImageSize(&self, multiplier: f32, default: source::SizeF) -> Option<source::SizeF> {
        if let Some(content) = self
            .content
            .as_ref()
            .filter(|content| content.content.image.IsDocumentImage())
        {
            // StyleFetchedImage::ImageSize uses the SVG owner's optional
            // intrinsic dimensions and aspect ratio, never the compatibility
            // width/height of its last paint snapshot. The owner also applies
            // the real Content-DPR; image-set override does not size SVGs.
            return match content
                .content
                .resource
                .ConcreteImageSize(multiplier, (default.width, default.height))
            {
                Ok(Some((width, height))) => Some(source::SizeF { width, height }),
                Ok(None) => {
                    content.Failure(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "SVG image owner did not publish concrete sizing",
                    ));
                    None
                }
                Err(error) => {
                    content.Failure(error);
                    None
                }
            };
        }
        let size = self.image.ImageSize(
            multiplier,
            &foundation::gfx::SizeF::new(default.width, default.height),
            foundation::RespectImageOrientationEnum::kRespectImageOrientation,
        );
        Some(source::SizeF {
            width: size.width(),
            height: size.height(),
        })
    }
}
struct CursorImageProjection<'a> {
    content: &'a ImageResourceContent,
    gap: super::page_cursor::CursorGap,
}
impl CursorImageProjection<'_> {
    fn Failure(&self, error: std::io::Error) {
        self.gap
            .ImageFailure(super::page_cursor::ResidentCursorImageFailure {
                resource_id: self.content.image.id,
                kind: error.kind(),
                message: error.to_string(),
            });
    }
    fn Bitmap(&self, request: Option<DocumentImageBitmapRequest>) -> Option<ImmutableImageBitmap> {
        match self.content.resource.BitmapForCurrentFrame(request) {
            Ok(bitmap) => Some(bitmap),
            Err(error) => {
                self.Failure(error);
                None
            }
        }
    }
}
impl source::CursorImageContent for CursorImageProjection<'_> {
    fn GetImage(&self) -> &dyn source::CursorImage {
        self
    }
}
impl source::CursorImage for CursorImageProjection<'_> {
    fn IsSVGImage(&self) -> bool {
        self.content.image.IsDocumentImage()
    }
    fn SizeRespectingOrientation(&self) -> source::Size {
        let image = &self.content.image;
        let (width, height) = image
            .orientation
            .oriented_size(image.width as f64, image.height as f64);
        source::Size {
            width: width as i32,
            height: height as i32,
        }
    }
    fn GetHotSpot(&self) -> Option<source::Point> {
        // BitmapImage::GetHotSpot, bitmap_image.cc:214-215. The immutable
        // snapshot retains the same selected frame and decoder metadata.
        self.content.image.hot_spot.map(|point| source::Point {
            x: point.x,
            y: point.y,
        })
    }
    fn RectContains(&self, point: source::Point) -> bool {
        Contains(self.SizeRespectingOrientation(), point)
    }
    fn AsBitmapForCurrentFrameRespectingOrientation(
        &self,
    ) -> Option<Arc<dyn source::CursorBitmap>> {
        self.Bitmap(None).map(|bitmap| Arc::new(bitmap) as _)
    }
    fn SVGImageForContainer(
        &self,
        size: source::SizeF,
        scale: f32,
        scheme: source::ColorScheme,
    ) -> Option<Rc<dyn source::CursorImage>> {
        let bitmap = self.Bitmap(Some(DocumentImageBitmapRequest {
            width: size.width,
            height: size.height,
            zoom: scale,
            preferred_color_scheme: if scheme == source::ColorScheme::kDark {
                1
            } else {
                0
            },
        }))?;
        Some(Rc::new(ContainerImage {
            bitmap: Arc::new(bitmap),
        }))
    }
}
// SVGImageForContainer retains the actual raster producer's completed bitmap.
// Keeping SVG identity is required by EventHandler's large-cursor scale policy.
struct ContainerImage {
    bitmap: Arc<ImmutableImageBitmap>,
}
impl source::CursorImage for ContainerImage {
    fn IsSVGImage(&self) -> bool {
        true
    }
    fn SizeRespectingOrientation(&self) -> source::Size {
        source::CursorBitmap::Size(self.bitmap.as_ref())
    }
    fn GetHotSpot(&self) -> Option<source::Point> {
        None
    }
    fn RectContains(&self, point: source::Point) -> bool {
        Contains(self.SizeRespectingOrientation(), point)
    }
    fn AsBitmapForCurrentFrameRespectingOrientation(
        &self,
    ) -> Option<Arc<dyn source::CursorBitmap>> {
        Some(self.bitmap.clone())
    }
    fn SVGImageForContainer(
        &self,
        _: source::SizeF,
        _: f32,
        _: source::ColorScheme,
    ) -> Option<Rc<dyn source::CursorImage>> {
        None
    }
}
fn Contains(size: source::Size, point: source::Point) -> bool {
    point.x >= 0 && point.y >= 0 && point.x < size.width && point.y < size.height
}
impl source::CursorBitmap for ImmutableImageBitmap {
    fn Size(&self) -> source::Size {
        source::Size {
            width: self.width as i32,
            height: self.height as i32,
        }
    }
    fn Rgba8(&self) -> &[u8] {
        self.rgba8.as_slice()
    }
    fn RowBytes(&self) -> usize {
        self.width as usize * 4
    }
}
