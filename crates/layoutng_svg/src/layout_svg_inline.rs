#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{gfx, DowncastFrom, Traceable, Visitor};
use layoutng_assembly::inline_cursor::InlineCursor;
use layoutng_assembly::internal::layout_box_model_object::PaintLayerType;
use layoutng_assembly::internal::layout_inline::LayoutInline;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_text::LayoutText;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/layout_svg_inline.h:30-51
#[repr(C)]
pub struct LayoutSVGInline {
    inline: LayoutInline,
}

impl LayoutSVGInline {
    // cpp: layoutng_svg/layout_svg_inline.cc:9-11
    pub fn new(element: *mut Element) -> Self {
        let mut inline = LayoutInline::new(element);
        inline.SetRuntimeClass(LayoutObjectClass::SvgInline);
        inline.SetAlwaysCreateLineBoxes(true);
        Self { inline }
    }

    // cpp: layoutng_svg/layout_svg_inline.h:33-48
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGInline"
    }
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        PaintLayerType::kNoPaintLayer
    }
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGInline(&self) -> bool {
        true
    }

    // Chromium layout_svg_inline.cc:76-105. Union the canonical, repositioned
    // fragment bounds in SVG user coordinates. The immutable export requests
    // this after layout; do not derive bounds from the unrotated inline box.
    pub fn ObjectBoundingBox(&self) -> gfx::RectF {
        let mut bounds = gfx::RectF::default();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return bounds;
        }
        let mut cursor = InlineCursor::default();
        cursor.MoveToIncludingCulledInline(self);
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.CurrentItem() };
            if item.IsSvgText() {
                bounds.Union(cursor.Current().ObjectBoundingBox(&cursor));
            } else {
                let mut descendants = cursor.CursorForDescendants();
                while descendants.IsNotNull() {
                    let item = unsafe { &*descendants.CurrentItem() };
                    if item.IsSvgText() {
                        bounds.Union(descendants.Current().ObjectBoundingBox(&descendants));
                    }
                    descendants.MoveToNext();
                }
            }
            cursor.MoveToNextForSameLayoutObject();
        }
        bounds
    }

    // cpp: layoutng_svg/layout_svg_inline.cc:13-24
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, style: &ComputedStyle) -> bool {
        assert!(!child.is_null());
        let child_ref = unsafe { &*child };
        if child_ref.IsText() {
            return child_ref.IsSVGInlineText()
                && !unsafe { &*child.cast::<LayoutText>() }.HasEmptyText();
        }
        if !child_ref.IsSVGInline() && !child_ref.IsSVGInlineText() {
            return false;
        }
        self.IsChildAllowedBase(child, style)
    }
}

impl Deref for LayoutSVGInline {
    type Target = LayoutInline;
    fn deref(&self) -> &Self::Target {
        &self.inline
    }
}
impl DerefMut for LayoutSVGInline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGInline, inline) == 0);
impl Traceable for LayoutSVGInline {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline.Trace(visitor);
    }
}
impl DowncastFrom<LayoutObject> for LayoutSVGInline {
    // cpp: layoutng_svg/layout_svg_inline.h:52-57
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGInline()
    }
}
