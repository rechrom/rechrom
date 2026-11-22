//! The document paint resources a fragment's painters read, for PrePaint's
//! retained-node admission. Chromium painters read these values from the
//! Document / LocalFrame / LocalFrameView; PrePaint records the usage on its
//! retained node and compares the recorded document resources against the
//! current ones only for the resource kinds that node paints with.

use std::sync::Arc;

use super::ResidentBoxGeometry as _;
use super::{FragmentKind, FragmentPaintData, PaintResources};

/// The document paint resources a fragment's painters read (viewport,
/// fonts, images), as a usage mask for SamePaintResourcesFor. PrePaint
/// records it on its retained node; it changes only with the fragment's own
/// style or content, which the source dirty bits already report.
pub fn PaintResourceUsage(fragment: &FragmentPaintData) -> u8 {
    let paint = &fragment.paint;
    // Only box fragments with a source paint nine-piece images (the former
    // ExportNinePieceImages admission).
    let nine_piece_images = fragment.kind == FragmentKind::kBox && paint.HasSource();
    // One style-source resolution; the fixed-attachment check is
    // ComputedStyle::HasFixedAttachmentBackgroundImage rather than a full
    // projection of every background layer.
    let (fixed_background, style_images) = paint.style.ResourceUsageFacts(nine_piece_images);
    // Viewport-relative painters, which also read the base background facts.
    // The canvas background is painted by the LayoutView root fragment
    // (IsLayoutView), so it needs no separate admission here.
    let viewport = paint.IsLayoutView()
        || super::ResidentBoxGeometry::IsPageContainer(fragment)
        || fixed_background;
    let images = paint.replaced_is_image
        || paint.VideoFrameResourceId().is_some()
        || fragment
            .ViewReplacedContent()
            .is_some_and(|content| content.is_canvas)
        || style_images
        || fragment.ResidentBoxFragment().is_some_and(|box_fragment| box_fragment.IsTable())
        || paint.SvgShape().is_some()
        || fragment.ViewSvgText().is_some();
    u8::from(viewport) * RESOURCE_VIEWPORT
        | u8::from(!paint.glyph_runs.is_empty()) * RESOURCE_FONTS
        | u8::from(images) * RESOURCE_IMAGES
}

/// Whether two document paint-resource values paint the same for a fragment
/// with `usage` (PaintResourceUsage). Uncached; callers memoize per pass.
pub fn SamePaintResourcesFor(a: &Arc<PaintResources>, b: &Arc<PaintResources>, usage: u8) -> bool {
    Arc::ptr_eq(a, b) || resources_equal_for_usage(a, b, usage)
}

const RESOURCE_VIEWPORT: u8 = 1 << 0;
const RESOURCE_FONTS: u8 = 1 << 1;
const RESOURCE_IMAGES: u8 = 1 << 2;

fn resources_equal_for_usage(a: &PaintResources, b: &PaintResources, usage: u8) -> bool {
    // These are immutable resource-provider identities, not the catalog
    // allocation rebuilt by every export. New storage conservatively dirties.
    let PaintResources {
        fonts,
        images,
        canvases,
        device_pixel_ratio,
        preferred_color_scheme,
        layout_zoom_factor,
        base_background_color,
        paints_opaque_base_background,
        should_clear_document_background,
        viewport,
    } = a;
    *device_pixel_ratio == b.device_pixel_ratio
        && *preferred_color_scheme == b.preferred_color_scheme
        && *layout_zoom_factor == b.layout_zoom_factor
        // The base background (ViewPainter's opaque backdrop and clear)
        // is read only by the view, canvas-background and page-container
        // painters, which all carry RESOURCE_VIEWPORT. A document laid out
        // both as itself and as a document image alternates these facts;
        // that must not dirty every other fragment of the document.
        && (usage & RESOURCE_VIEWPORT == 0
            || (*viewport == b.viewport
                && *base_background_color == b.base_background_color
                && *paints_opaque_base_background == b.paints_opaque_base_background
                && *should_clear_document_background == b.should_clear_document_background))
        && (usage & RESOURCE_FONTS == 0
            || (fonts.len() == b.fonts.len()
                && (fonts.ptr_eq(&b.fonts)
                    || fonts.iter().zip(b.fonts.iter()).all(|(a, b)| {
                        font_engine::SharedFontBytes::ptr_eq(&a.bytes, &b.bytes) && a == b
                    }))))
        && (usage & RESOURCE_IMAGES == 0
            || (images.len() == b.images.len()
                && images.iter().zip(b.images.iter()).all(|(a, b)| a == b)
                && canvases.len() == b.canvases.len()
                && canvases.iter().zip(b.canvases.iter()).all(|(a, b)| a == b)))
}
