#![allow(non_snake_case)]

use foundation::{
    gfx, AspectRatioFit, DynamicTo, EObjectFit, FloatValueForLength, IsLtr, LayoutUnit,
    MinimumValueForLength, PhysicalOffset, PhysicalRect, PhysicalSize, UnsupportedLayout,
};
use layoutng_assembly::inline_cursor::InlineCursor;
use layoutng_assembly::internal::editing::forward::PositionWithAffinity;
use layoutng_assembly::internal::hit_test_phase::HitTestPhase;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_assembly::internal::layout_invalidation_reason;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{
    HitTestLocation, HitTestResult, LayoutObject, LayoutObjectClass, StyleChangeContext,
};
use layoutng_assembly::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
use layoutng_assembly::internal::outline_geometry::OutlineOutsetExtent;
use layoutng_assembly::internal::outline_info::LayoutOutlineInfo;
use layoutng_assembly::internal::outline_rect_collector::VectorOutlineRectCollector;
use layoutng_assembly::internal::selection_state::SelectionState;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::basic_shapes::{BasicShape, BasicShapeInset, ShapeType};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::natural_sizing_info::{
    ConcreteObjectSizePhysical, PhysicalNaturalSizingInfo,
};
use layoutng_style::style::style_difference::StyleDifference;
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

// cpp: core/layout/layout_replaced.cc:67-68
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedDefaultWidth() -> i32 {
    300
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedDefaultHeight() -> i32 {
    150
}

// cpp: core/layout/layout_replaced.cc:70-70
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedNew(element: *mut Element) -> LayoutReplaced {
    layoutng_assembly::RegisterReplacedStyleDidChange(
        |object, difference, old_style, new_style, context| {
            LayoutReplacedStyleDidChange(
                unsafe { &mut *object.cast::<LayoutReplaced>() },
                difference,
                old_style,
                new_style,
                context,
            )
        },
    );
    layoutng_assembly::RegisterImageStyleDidChange(
        |object, difference, old_style, new_style, context| {
            LayoutInputReplacedStyleDidChange(
                unsafe { &mut *object.cast::<LayoutInputReplaced>() },
                difference,
                old_style,
                new_style,
                context,
            )
        },
    );
    layoutng_assembly::RegisterReplacedRecalcVisualOverflow(|object| {
        LayoutReplacedRecalcVisualOverflow(unsafe { &mut *object.cast::<LayoutReplaced>() })
    });
    let box_ = LayoutBox::new(unsafe { &mut (*element).container });
    box_.SetRuntimeClass(LayoutObjectClass::Replaced);
    LayoutReplaced::FromLayoutBox(box_)
}

// cpp: core/layout/layout_replaced.cc:243-285
fn LayoutReplacedComputeVisualEffectOverflowOutsets(this: &mut LayoutReplaced) -> PhysicalBoxStrut {
    this.CheckIsNotDestroyed();
    let style = unsafe { &*this.StylePtr() };
    debug_assert!(style.HasVisualOverflowingEffect());
    let mut outsets = style.BoxDecorationOutsets();
    if style.HasBorderShape() {
        std::panic::panic_any(UnsupportedLayout::new(
            "LayoutReplaced border-shape visual overflow requires BorderShapePainter",
        ));
    }
    if style.HasOutline() {
        let mut info = LayoutOutlineInfo::default();
        let box_: &LayoutBox = this;
        let mut collector = VectorOutlineRectCollector::default();
        box_.AddOutlineRects(
            &mut collector,
            &mut info,
            &PhysicalOffset::default(),
            style.OutlineRectsShouldIncludeBlockInkOverflow(),
        );
        let outline_rects = collector.TakeRects();
        let mut rect = PhysicalRect::default();
        for outline_rect in outline_rects.iter() {
            rect.Unite(outline_rect);
        }
        let size = this.StitchedSize();
        this.SetOutlineMayBeAffectedByDescendants(rect.size != size);
        if style.OutlineStyleIsAuto() || !style.HasBorderShape() {
            rect.Inflate(LayoutUnit::from_signed(OutlineOutsetExtent(style, &info)));
            outsets.Unite(&PhysicalBoxStrut::new(
                -rect.Y(),
                rect.Right() - size.width,
                rect.Bottom() - size.height,
                -rect.X(),
            ));
        }
    }
    outsets
}

// cpp: core/layout/layout_replaced.cc:287-299
fn LayoutReplacedAddVisualEffectOverflow(this: &mut LayoutReplaced) {
    this.CheckIsNotDestroyed();
    if !this.StyleRef().HasVisualOverflowingEffect() {
        return;
    }
    let mut visual_effect_overflow = this.PhysicalBorderBoxRect();
    let outsets = LayoutReplacedComputeVisualEffectOverflowOutsets(this);
    visual_effect_overflow.Expand(&outsets);
    this.AddSelfVisualOverflow(&visual_effect_overflow);
    this.UpdateHasSubpixelVisualEffectOutsets(&outsets);
}

// cpp: core/layout/layout_replaced.cc:301-319
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedRecalcVisualOverflow(this: &mut LayoutReplaced) {
    this.CheckIsNotDestroyed();
    this.ClearVisualOverflow();
    let object: &mut LayoutObject = this;
    object.RecalcVisualOverflowBase();
    LayoutReplacedAddVisualEffectOverflow(this);
    if this.RespectsCSSOverflow() {
        let content_rect = this.ReplacedContentRect();
        this.AddContentsVisualOverflow(&content_rect);
    }
}

// cpp: core/layout/layout_replaced.cc:74-93
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedStyleDidChange(
    this: &mut LayoutReplaced,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &StyleChangeContext,
) {
    this.CheckIsNotDestroyed();
    let box_: &mut LayoutBox = this;
    box_.StyleDidChange(diff, old_style, new_style, style_change_context);
    if diff.border_radius_changed() {
        this.SetNeedsPaintPropertyUpdate();
    }
    let old_zoom = if old_style.is_null() {
        ComputedStyleInitialValues::InitialZoom()
    } else {
        unsafe { &*old_style }.EffectiveZoom()
    };
    if new_style.EffectiveZoom() != old_zoom {
        LayoutReplacedNaturalSizeChanged(this);
    }
}

// cpp: core/layout/layout_image.cc:102-117
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputReplacedStyleDidChange(
    this: &mut LayoutInputReplaced,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &StyleChangeContext,
) {
    let old_zoom = if old_style.is_null() {
        ComputedStyleInitialValues::InitialZoom()
    } else {
        unsafe { &*old_style }.EffectiveZoom()
    };
    LayoutReplacedStyleDidChange(this, diff, old_style, new_style, style_change_context);
    if new_style.EffectiveZoom() != old_zoom {
        this.UpdateNaturalDimensionsFromElementInput();
    }
    let old_orientation = if old_style.is_null() {
        ComputedStyleInitialValues::InitialImageOrientation()
    } else {
        unsafe { &*old_style }.ImageOrientation()
    };
    if new_style.ImageOrientation() != old_orientation {
        this.SetNeedsPaintPropertyUpdate();
        LayoutReplacedNaturalSizeChanged(this);
    }
    // cpp: core/layout/layout_image.cc:118-129. The trace context already
    // carries frame identity; retain the source DOM node identity as event
    // data and avoid the unsized test when tracing is inactive.
    if browser_tracing::enabled() && this.IsUnsizedImage() {
        let node = this.GetNode();
        let node_id = if node.is_null() {
            0
        } else {
            unsafe { &*node }.InputId()
        };
        browser_tracing::instant(
            "devtools.timeline",
            "LayoutImageUnsized",
            &[("node_id", node_id as f64)],
        );
    }
}

// cpp: core/layout/layout_replaced.cc:95-99
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedNaturalSizeChanged(this: &mut LayoutReplaced) {
    this.CheckIsNotDestroyed();
    let object: &mut LayoutObject = this;
    object.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
        &raw const layout_invalidation_reason::kSizeChanged,
    );
}

// cpp: core/layout/layout_replaced.cc:594-597
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedComputeSpeculativeDecodeSize(
    this: &LayoutReplaced,
) -> gfx::Size {
    this.CheckIsNotDestroyed();
    this.ReplacedContentRect().PixelSnappedSize()
}

// cpp: core/layout/layout_replaced.cc:599-617
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedLocalSelectionVisualRect(this: &LayoutReplaced) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    if matches!(
        this.GetSelectionState(),
        SelectionState::kNone | SelectionState::kContain
    ) {
        return PhysicalRect::default();
    }
    if this.IsInline() && this.IsInLayoutNGInlineFormattingContext() {
        let mut rect = PhysicalRect::default();
        let mut cursor = InlineCursor::default();
        let object: &LayoutObject = this;
        cursor.MoveToLayoutObject(object);
        while cursor.IsNotNull() {
            rect.Unite(&cursor.CurrentLocalSelectionRectForReplaced());
            cursor.MoveToNextForSameLayoutObject();
        }
        return rect;
    }
    PhysicalRect::new(PhysicalOffset::default(), this.StitchedSize())
}

// cpp: core/layout/layout_replaced.cc:524-559
fn SelectionTopAndBottom(
    layout_replaced: &LayoutReplaced,
    rect: &LogicalRect,
) -> (LayoutUnit, LayoutUnit) {
    let fallback = (rect.BlockStartOffset(), rect.BlockEndOffset());
    if layout_replaced.IsInline() && layout_replaced.IsInLayoutNGInlineFormattingContext() {
        let mut line_box = InlineCursor::default();
        let object: &LayoutObject = layout_replaced;
        line_box.MoveToLayoutObject(object);
        if line_box.IsNull() {
            return fallback;
        }
        line_box.MoveToContainingLine();
        if line_box.IsNull() {
            return fallback;
        }
        let line_style = line_box.Current().Style();
        let converter = WritingModeConverter::new(
            line_style.GetWritingDirection(),
            line_box.ContainerFragment().Size(),
        );
        let physical_rect = line_box.CurrentRectInFirstContainerFragment();
        let logical_rect = converter.ToLogicalRect(physical_rect);
        return (
            logical_rect.offset.block_offset,
            logical_rect.BlockEndOffset(),
        );
    }
    fallback
}

// cpp: core/layout/layout_replaced.cc:561-592
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedPositionForPoint(
    this: &LayoutReplaced,
    point: &PhysicalOffset,
) -> PositionWithAffinity {
    this.CheckIsNotDestroyed();
    let logical_rect = this.LogicalRectInContainer();
    let (top, bottom) = SelectionTopAndBottom(this, &logical_rect);
    let location_container = this.LocationContainer();
    assert!(!location_container.is_null());
    let logical_point = unsafe { &*location_container }
        .CreateWritingModeConverter()
        .ToLogicalOffset(*point + this.PhysicalLocation(), PhysicalSize::default());
    let block_direction_position = logical_point.block_offset;
    let line_direction_position = logical_point.inline_offset;
    if block_direction_position < top {
        return this.PositionBeforeThis();
    }
    if block_direction_position >= bottom {
        return this.PositionAfterThis();
    }
    if !this.GetNode().is_null() {
        let is_at_left_side = line_direction_position
            <= logical_rect.offset.inline_offset + logical_rect.InlineSize() / 2;
        let is_at_start = is_at_left_side == IsLtr(this.ResolvedDirection());
        if is_at_start {
            return this.PositionBeforeThis();
        }
        return this.PositionAfterThis();
    }
    let object: &LayoutObject = this;
    object.CreatePositionWithAffinityDefault(0)
}

// cpp: core/layout/layout_replaced.cc:211-236
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedHitTestChildren(
    this: &LayoutReplaced,
    result: &mut HitTestResult,
    location: &HitTestLocation,
    accumulated_offset: &PhysicalOffset,
    phase: HitTestPhase,
) -> bool {
    this.CheckIsNotDestroyed();
    let mut child = this.SlowLastChild();
    while !child.is_null() {
        let child_ref = unsafe { &*child };
        let previous_sibling = child_ref.PreviousSibling();
        if child_ref.HasLayer() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(child);
            debug_assert!(!box_model.is_null());
            if unsafe { &*box_model }.HasSelfPaintingLayer() {
                child = previous_sibling;
                continue;
            }
        }
        let mut child_accumulated_offset = *accumulated_offset;
        let box_ = DynamicTo::<LayoutBox>(child);
        if !box_.is_null() {
            child_accumulated_offset += unsafe { &*box_ }.PhysicalLocation();
        }
        if unsafe { &mut *child }.NodeAtPoint(result, location, &child_accumulated_offset, phase) {
            return true;
        }
        child = previous_sibling;
    }
    false
}

// cpp: core/layout/layout_replaced.cc:72-72
// The C++ destructor is defaulted and owns no additional fields; Rust drops
// the embedded LayoutBox through its ordinary field lifetime.

// cpp: core/layout/layout_replaced.cc:321-358
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

// cpp: core/layout/layout_replaced.cc:360-424
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

// cpp: core/layout/layout_replaced.cc:426-488
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
                if this.IsLayoutImage() {
                    let image =
                        unsafe { &*(this as *const LayoutReplaced).cast::<LayoutInputReplaced>() };
                    scaled_intrinsic_size.ScaleFloat(1.0 / image.ImageDevicePixelRatio());
                }
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

// cpp: core/layout/layout_replaced.cc:490-495
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedContentRect(this: &LayoutReplaced) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    LayoutReplacedContentRectFrom(this, &this.PhysicalContentBoxRect())
}

// cpp: core/layout/layout_replaced.cc:497-502
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutReplacedContentRectFrom(
    this: &LayoutReplaced,
    base_content_rect: &PhysicalRect,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    LayoutReplacedComputeReplacedContentRect(this, base_content_rect, &this.GetNaturalDimensions())
}

// cpp: core/layout/layout_replaced.cc:509-522
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

// cpp: core/layout/layout_replaced.cc:619-623
pub(crate) fn LayoutReplacedRespectsCSSOverflow(this: &LayoutReplaced) -> bool {
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

// cpp: core/layout/layout_replaced.cc:625-644
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

// LayoutImage's natural-dimension virtual returns its retained member.
// cpp: core/layout/layout_image.cc:393-396
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputReplacedGetNaturalDimensions(
    this: &LayoutInputReplaced,
) -> PhysicalNaturalSizingInfo {
    this.CheckIsNotDestroyed();
    this.NaturalDimensions()
}

// cpp: core/layout/layout_replaced.h:98-103
// cpp: core/layout/layout_image.cc:393-396
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
