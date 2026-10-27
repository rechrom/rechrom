use super::*;

use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

pub(crate) struct InlineClientGeometry {
    pub bounds: PhysicalRect,
    pub rects: Vec<(
        *const layout_fragment::physical_box_fragment::PhysicalBoxFragment,
        PhysicalRect,
    )>,
}

// cpp: core/layout/layout_inline.cc:430-508
#[inline(never)]
pub(crate) fn LayoutInlineClientRectsInContainers(inline: &LayoutInline) -> InlineClientGeometry {
    let native: &LayoutObject = inline;
    let mut cursor = InlineCursor::default();
    if native.IsInLayoutNGInlineFormattingContext() {
        cursor.MoveToIncludingCulledInline(native);
    }
    let mut result = InlineClientGeometry {
        bounds: PhysicalRect::default(),
        rects: Vec::new(),
    };
    let mut bounds: Option<PhysicalRect> = None;
    while cursor.IsNotNull() {
        let target = cursor.Current().GetLayoutObject();
        let mut in_ruby_text = false;
        if native.IsInlineRuby() && !target.is_null() && target != native as *const LayoutObject {
            let mut child = target;
            while !child.is_null()
                && unsafe { &*child }.Parent() != native as *const LayoutObject as *mut LayoutObject
            {
                child = unsafe { &*child }.Parent();
            }
            in_ruby_text = !child.is_null() && unsafe { &*child }.IsInlineRubyText();
        }
        if !in_ruby_text {
            let mut rect = cursor.Current().RectInContainerFragment();
            let box_fragment = cursor.Current().BoxFragment();
            if !box_fragment.is_null() {
                let offset = rect.offset;
                rect = LocalRectForBoxQuad(unsafe { &*box_fragment }, BoxQuadType::kBorder);
                rect.offset += offset;
            }
            rect.size.width = rect.size.width.ClampNegativeToZero();
            rect.size.height = rect.size.height.ClampNegativeToZero();
            let mut first_rect = rect;
            first_rect.offset += cursor.CurrentRectInFirstContainerFragment().offset
                - cursor.Current().RectInContainerFragment().offset;
            if let Some(bounds) = &mut bounds {
                bounds.UniteEvenIfEmpty(&first_rect);
            } else {
                bounds = Some(first_rect);
            }
            result.rects.push((cursor.ContainerFragment(), rect));
        }
        cursor.MoveToNextForSameLayoutObject();
    }
    if let Some(bounds) = bounds {
        result.bounds = bounds;
    } else {
        let block = native.ContainingBlock();
        let container = if cursor.HasRoot() {
            cursor.ContainerFragment() as *const _
        } else if !block.is_null() && unsafe { &*block }.PhysicalFragmentCount() != 0 {
            unsafe { &*block }.GetPhysicalFragment(0)
        } else {
            std::ptr::null()
        };
        if !container.is_null() {
            result.rects.push((container, PhysicalRect::default()));
        }
    }
    result
}
