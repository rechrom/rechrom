#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::internal::algorithm_entry::LayoutAlgorithmEntryFor;
use layoutng_assembly::internal::layout_input_node::LayoutInputNode;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::layout_replaced::{
    LayoutInputReplaced, LayoutMedia, LayoutReplaced, LayoutVideo,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

use crate::layout_replaced::LayoutReplacedRespectsCSSOverflow;
use crate::replaced_layout_algorithm::ReplacedLayoutAlgorithm;

// cpp: layoutng_replaced/assembly.cc:14-18
fn CreateReplacedObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    if !node.InputIsStyleGenerated() && node.InputIsHTMLVideoElement() {
        MakeGarbageCollected(LayoutVideo::new(To::<Element>(node))).cast()
    } else if !node.InputIsStyleGenerated() && node.InputIsHTMLMediaElement() {
        MakeGarbageCollected(LayoutMedia::new(To::<Element>(node))).cast()
    } else {
        MakeGarbageCollected(LayoutInputReplaced::new(To::<Element>(node))).cast()
    }
}

// cpp: layoutng_replaced/assembly.cc:19-23
#[inline(never)]
fn NaturalSizingInfo(node: &LayoutInputNode) -> PhysicalNaturalSizingInfo {
    let _trace = browser_tracing::span("layout", "layout_algo_replaced.NaturalSizingInfo");
    let replaced = To::<LayoutReplaced>(node.GetLayoutBox().cast::<LayoutObject>());
    unsafe { &*replaced }.ComputeNaturalSizingInfo()
}

// Chromium updates this LayoutImage member when the image resource changes.
// The callback keeps the retained-tree update in layout_invalidation while the
// concrete derived representation remains owned by layout_algo_replaced.
fn UpdateImageState(replaced: &mut LayoutReplaced, ratio: f64, resource_error: bool) {
    assert!(replaced.IsLayoutImage());
    let image = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() };
    image.SetImageDevicePixelRatio(ratio as f32);
    if !resource_error {
        image.UpdateNaturalDimensionsFromElementInput();
    }
}

// Source: core/loader/image_loader.cc, ImageLoader::UpdateLayoutObject;
// core/layout/layout_image_resource.cc, LayoutImageResource::SetImageResource.
fn UpdateImageResource(
    replaced: &mut LayoutReplaced,
    update: image_resource::ImageResourceContentUpdate,
) -> bool {
    assert!(replaced.IsLayoutImage());
    let image = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() };
    let notified = image.UpdateImageResource(update);
    if notified && !image.ImageResource().ErrorOccurred() {
        image.UpdateNaturalDimensionsFromImageResource();
    }
    notified
}

// Source: HTMLVideoElement::UpdatePosterImage -> LayoutImageResource::SetImageResource,
// followed by LayoutVideo::ImageChanged/UpdateNaturalSize on the same object.
fn UpdateVideoPosterResource(
    replaced: &mut LayoutReplaced,
    update: image_resource::ImageResourceContentUpdate,
) -> bool {
    assert!(replaced.IsVideo());
    let video = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutVideo>() };
    let notified = video.UpdateImageResource(update);
    if notified && !video.ImageResource().ErrorOccurred() {
        video.UpdateNaturalDimensionsFromImageResource();
    }
    if notified {
        crate::layout_video::UpdateNaturalSize(video);
        video.InvalidateCompositing();
        video.SetShouldDoFullPaintInvalidation();
    }
    notified
}

fn UseBrokenImageResource(
    replaced: &mut LayoutReplaced,
    variants: image_resource::platform_resources::BrokenImageResourceVariants,
    device_scale_factor: f64,
) -> bool {
    assert!(replaced.IsLayoutImage());
    let image = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() };
    let notified = image.UseBrokenImage(variants, device_scale_factor);
    if notified {
        image.UpdateNaturalDimensionsFromImageResource();
    }
    notified
}

fn ImageResourceFacts(
    replaced: &LayoutReplaced,
) -> layoutng_assembly::internal::layout_object_factory_set::ReplacedImageResourceFacts {
    assert!(replaced.IsLayoutImage());
    let image = unsafe { &*(replaced as *const LayoutReplaced).cast::<LayoutInputReplaced>() };
    let resource = image.ImageResource();
    layoutng_assembly::internal::layout_object_factory_set::ReplacedImageResourceFacts {
        has_image: resource.HasImage(),
        metadata: resource.Metadata(),
        error: resource.ErrorOccurred(),
        loading: resource.IsLoading(),
        device_pixel_ratio: image.ImageDevicePixelRatio(),
    }
}

// Source: core/layout/layout_video.cc:165-173.
fn UpdateVideoState(replaced: &mut LayoutReplaced) {
    assert!(replaced.IsVideo());
    let video = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutVideo>() };
    video.UpdateFromElement();
}

// cpp: layoutng_replaced/assembly.h:3-3
// cpp: layoutng_replaced/assembly.cc:26-30
pub fn InstallReplacedAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.algorithms.replaced_sizing.performance_owner = Some("layout_algo_replaced");
    assembly.objects.replaced = Some(CreateReplacedObject);
    assembly.objects.replaced_respects_css_overflow = Some(LayoutReplacedRespectsCSSOverflow);
    assembly.objects.replaced_image_state_updater = Some(UpdateImageState);
    assembly.objects.replaced_image_resource_updater = Some(UpdateImageResource);
    assembly.objects.replaced_video_poster_resource_updater = Some(UpdateVideoPosterResource);
    assembly.objects.replaced_broken_image_resource_updater = Some(UseBrokenImageResource);
    assembly.objects.replaced_image_resource_facts = Some(ImageResourceFacts);
    assembly.objects.replaced_video_state_updater = Some(UpdateVideoState);
    assembly.algorithms.replaced =
        LayoutAlgorithmEntryFor::<ReplacedLayoutAlgorithm>("layout_algo_replaced");
    assembly.algorithms.replaced_sizing.natural_sizing_info = Some(NaturalSizingInfo);
}
