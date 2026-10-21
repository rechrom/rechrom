#![allow(non_snake_case)]

use foundation::{
    gfx, AspectRatioFit, DynamicTo, EObjectFit, FloatValueForLength, LayoutRatioFromSizeF,
    LayoutUnit, MinimumValueForLength, PhysicalOffset, PhysicalRect, PhysicalSize,
};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObjectClass;
use layoutng_assembly::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
use layoutng_style::style::basic_shapes::{BasicShape, BasicShapeInset, ShapeType};
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::natural_sizing_info::{
    ConcreteObjectSizePhysical, PhysicalNaturalSizingInfo,
};
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

// cpp: layoutng_replaced/layout_replaced.cc:24-25
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedDefaultWidth() -> i32 {
    300
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedDefaultHeight() -> i32 {
    150
}

// cpp: layoutng_replaced/layout_replaced.cc:27-28
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedNew(element: *mut Element) -> LayoutReplaced {
    let box_ = LayoutBox::new(unsafe { &mut (*element).container });
    box_.SetRuntimeClass(LayoutObjectClass::Replaced);
    LayoutReplaced::FromLayoutBox(box_)
}

// cpp: layoutng_replaced/layout_replaced.cc:28-28
// The C++ destructor is defaulted and owns no additional fields; Rust drops
// the embedded LayoutBox through its ordinary field lifetime.

// cpp: layoutng_replaced/layout_replaced.cc:30-68
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedComputeObjectViewBoxRect(
    this: &LayoutReplaced,
    sizing_info: &PhysicalNaturalSizingInfo,
) -> Option<PhysicalRect> {
    this.CheckIsNotDestroyed();
    let member = this.StyleRef().ObjectViewBox()?;
    let shape = unsafe { member.GetNonNull()?.as_ref() };
    if !sizing_info.has_width
        || !sizing_info.has_height
        || !this.ShouldApplyObjectViewBox()
        || sizing_info.size.IsEmpty()
    {
        return None;
    }
    debug_assert_eq!(shape.GetType(), ShapeType::kBasicShapeInsetType);
    // The source's checked To<BasicShapeInset> follows the validated dynamic
    // shape tag; the trait-object data pointer is the inset object's address.
    let inset = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeInset) };
    let natural_size = gfx::SizeF::new(
        sizing_info.size.width.ToFloat(),
        sizing_info.size.height.ToFloat(),
    );
    let left = FloatValueForLength(inset.Left(), natural_size.width());
    let top = FloatValueForLength(inset.Top(), natural_size.height());
    let inset_rect = gfx::RectF::new(
        gfx::PointF::new(left, top),
        gfx::SizeF::new(
            (natural_size.width()
                - left
                - FloatValueForLength(inset.Right(), natural_size.width()))
            .max(0.0),
            (natural_size.height()
                - top
                - FloatValueForLength(inset.Bottom(), natural_size.height()))
            .max(0.0),
        ),
    );
    let view_box_rect = PhysicalRect::EnclosingRect(&inset_rect);
    if view_box_rect.IsEmpty() {
        return None;
    }
    let natural_rect = PhysicalRect::new(PhysicalOffset::default(), sizing_info.size);
    if view_box_rect == natural_rect {
        None
    } else {
        Some(view_box_rect)
    }
}

// cpp: layoutng_replaced/layout_replaced.cc:70-91
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedComputeReplacedContentRect(
    this: &LayoutReplaced,
    base_content_rect: &PhysicalRect,
    sizing_info: &PhysicalNaturalSizingInfo,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    let Some(view_box) = LayoutReplacedComputeObjectViewBoxRect(this, sizing_info) else {
        return LayoutReplacedComputeObjectFitAndPositionRect(this, base_content_rect, sizing_info);
    };
    debug_assert!(!view_box.IsEmpty());
    let view_box_paint_rect = LayoutReplacedComputeObjectFitAndPositionRect(
        this,
        base_content_rect,
        &PhysicalNaturalSizingInfo::MakeFixed(&view_box.size),
    );
    if view_box_paint_rect.IsEmpty() {
        return view_box_paint_rect;
    }
    let natural_size = sizing_info.size;
    let scaled_image_size = PhysicalSize::new(
        natural_size
            .width
            .MulDiv(view_box_paint_rect.Width(), view_box.Width()),
        natural_size
            .height
            .MulDiv(view_box_paint_rect.Height(), view_box.Height()),
    );
    let scaled_offset = PhysicalOffset::new(
        view_box
            .X()
            .MulDiv(view_box_paint_rect.Width(), view_box.Width()),
        view_box
            .Y()
            .MulDiv(view_box_paint_rect.Height(), view_box.Height()),
    );
    PhysicalRect::new(
        view_box_paint_rect.offset - scaled_offset,
        scaled_image_size,
    )
}

// cpp: layoutng_replaced/layout_replaced.cc:93-146
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedComputeObjectFitAndPositionRect(
    this: &LayoutReplaced,
    base_content_rect: &PhysicalRect,
    sizing_info: &PhysicalNaturalSizingInfo,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    let object_fit = this.StyleRef().GetObjectFit();
    if object_fit == EObjectFit::kFill
        && this.StyleRef().ObjectPosition() == &ComputedStyleInitialValues::InitialObjectPosition()
    {
        return *base_content_rect;
    }
    let intrinsic_size = sizing_info.size;
    let aspect_ratio = sizing_info.aspect_ratio;
    if intrinsic_size.IsEmpty() && aspect_ratio.IsEmpty() {
        return *base_content_rect;
    }
    let mut scaled_intrinsic_size = intrinsic_size;
    let mut object_size = base_content_rect.size;
    match object_fit {
        EObjectFit::kScaleDown | EObjectFit::kContain | EObjectFit::kCover => {
            if object_fit == EObjectFit::kScaleDown {
                let element = DynamicTo::<Element>(this.GetNode());
                let data = if element.is_null() {
                    None
                } else {
                    unsafe { &*element }.InputElementData().as_ref()
                };
                let density = data.map_or(1.0, |data| data.image_device_pixel_ratio);
                scaled_intrinsic_size.ScaleFloat((1.0 / density) as f32);
            }
            if !aspect_ratio.IsEmpty() {
                object_size = object_size.FitToAspectRatio(
                    &aspect_ratio,
                    if object_fit == EObjectFit::kCover {
                        AspectRatioFit::kAspectRatioFitGrow
                    } else {
                        AspectRatioFit::kAspectRatioFitShrink
                    },
                );
            }
            if object_fit == EObjectFit::kScaleDown
                && object_size.width > scaled_intrinsic_size.width
            {
                object_size = if intrinsic_size.IsEmpty() {
                    ConcreteObjectSizePhysical(sizing_info, &base_content_rect.size)
                } else {
                    scaled_intrinsic_size
                };
            }
        }
        EObjectFit::kNone => {
            object_size = if intrinsic_size.IsEmpty() {
                ConcreteObjectSizePhysical(sizing_info, &base_content_rect.size)
            } else {
                scaled_intrinsic_size
            };
        }
        EObjectFit::kFill => {}
    }
    let object_position = PhysicalOffset::new(
        MinimumValueForLength(
            this.StyleRef().ObjectPosition().X(),
            base_content_rect.Width() - object_size.width,
        ),
        MinimumValueForLength(
            this.StyleRef().ObjectPosition().Y(),
            base_content_rect.Height() - object_size.height,
        ),
    );
    PhysicalRect::new(base_content_rect.offset + object_position, object_size)
}

// cpp: layoutng_replaced/layout_replaced.cc:148-152
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedContentRect(this: &LayoutReplaced) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    LayoutReplacedContentRectFrom(this, &this.PhysicalContentBoxRect())
}

// cpp: layoutng_replaced/layout_replaced.cc:154-158
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedContentRectFrom(
    this: &LayoutReplaced,
    base_content_rect: &PhysicalRect,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    LayoutReplacedComputeReplacedContentRect(this, base_content_rect, &this.GetNaturalDimensions())
}

// cpp: layoutng_replaced/layout_replaced.cc:160-171
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedComputeNaturalSizingInfo(
    this: &LayoutReplaced,
) -> PhysicalNaturalSizingInfo {
    this.CheckIsNotDestroyed();
    debug_assert!(!this.ShouldApplySizeContainment());
    let mut sizing_info = this.GetNaturalDimensions();
    if let Some(view_box) = LayoutReplacedComputeObjectViewBoxRect(this, &sizing_info) {
        sizing_info.size = view_box.size;
        if !sizing_info.aspect_ratio.IsEmpty() {
            sizing_info.aspect_ratio = sizing_info.size;
        }
    }
    sizing_info
}

// cpp: layoutng_replaced/layout_replaced.cc:173-179
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedRespectsCSSOverflow(this: &LayoutReplaced) -> bool {
    this.CheckIsNotDestroyed();
    let element = DynamicTo::<Element>(this.GetNode());
    if element.is_null() {
        return false;
    }
    unsafe { &*element }
        .InputElementData()
        .as_ref()
        .is_some_and(|data| data.replaced_respects_css_overflow)
}

// cpp: layoutng_replaced/layout_replaced.cc:181-194
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedClipsToContentBox(this: &LayoutReplaced) -> bool {
    this.CheckIsNotDestroyed();
    if !LayoutReplacedRespectsCSSOverflow(this) {
        if this.IsSVGRoot() {
            return this.ShouldClipOverflowAlongBothAxis();
        }
        return true;
    }
    let overflow_clip_margin = this.StyleRef().OverflowClipMargin();
    this.ShouldClipOverflowAlongBothAxis()
        && overflow_clip_margin.as_ref().is_some_and(|margin| {
            margin.GetReferenceBox() == ReferenceBox::kContentBox
                && margin.GetMargin() == LayoutUnit::new()
        })
}

// cpp: layoutng_replaced/layout_replaced.cc:196-226
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputReplacedGetNaturalDimensions(
    this: &LayoutInputReplaced,
) -> PhysicalNaturalSizingInfo {
    this.CheckIsNotDestroyed();
    let element = DynamicTo::<Element>(this.GetNode());
    assert!(!element.is_null());
    let data = unsafe { &*element }.InputElementData();
    let mut sizing_info = PhysicalNaturalSizingInfo::None();
    let Some(data) = data.as_ref() else {
        return sizing_info;
    };
    let zoom = this.StyleRef().EffectiveZoom();
    sizing_info.has_width = data.natural_width.is_some();
    sizing_info.has_height = data.natural_height.is_some();
    sizing_info.size = PhysicalSize::new(
        LayoutUnit::from_f64(data.natural_width.unwrap_or(0.0) * zoom as f64),
        LayoutUnit::from_f64(data.natural_height.unwrap_or(0.0) * zoom as f64),
    );
    if let Some(ratio) = data.natural_aspect_ratio {
        sizing_info.aspect_ratio = LayoutRatioFromSizeF(gfx::SizeF::new(ratio as f32, 1.0));
    } else if !sizing_info.size.IsEmpty() {
        sizing_info.aspect_ratio = sizing_info.size;
    }
    sizing_info
}

// cpp: layoutng/internal/layout_replaced.h:89-89
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutReplacedGetNaturalDimensions(
    this: &LayoutReplaced,
) -> PhysicalNaturalSizingInfo {
    if this.RuntimeClass() == LayoutObjectClass::SvgRoot {
        let algorithms =
            layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::Algorithms();
        let dimensions = unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.root_natural_dimensions)
            .expect("SVG root natural dimensions are not installed");
        return dimensions(unsafe {
            &*(this as *const LayoutReplaced)
                .cast::<layoutng_assembly::internal::layout_object::LayoutObject>()
        });
    }
    assert_eq!(this.RuntimeClass(), LayoutObjectClass::Replaced);
    LayoutInputReplacedGetNaturalDimensions(unsafe {
        &*(this as *const LayoutReplaced as *const LayoutInputReplaced)
    })
}
