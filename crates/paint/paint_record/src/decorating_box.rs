//! CSS text-decoration decorating-box state carried across inline fragments.

#![allow(non_snake_case)]

use foundation::{Member, PhysicalOffset, Visitor};
use layoutng_assembly::{fragment_item::FragmentItem, internal::used_font::UsedFont};
use layoutng_style::style::{
    applied_text_decoration::AppliedTextDecorationVector, computed_style::ComputedStyle,
};

// Source: third_party/blink/renderer/core/paint/decorating_box.h:19-69.
pub struct DecoratingBox {
    content_offset_in_container_: PhysicalOffset,
    used_font_: UsedFont,
    style_: Member<ComputedStyle>,
    decorations_: Member<AppliedTextDecorationVector>,
}

impl Clone for DecoratingBox {
    fn clone(&self) -> Self {
        Self::new(
            &self.content_offset_in_container_,
            self.Style(),
            &self.used_font_,
            self.decorations_.Get(),
        )
    }
}

impl DecoratingBox {
    // Source: third_party/blink/renderer/core/paint/decorating_box.h:23-33.
    pub fn new(
        content_offset_in_container: &PhysicalOffset,
        style: &ComputedStyle,
        used_font: &UsedFont,
        decorations: *const AppliedTextDecorationVector,
    ) -> Self {
        let decorations = if decorations.is_null() {
            style.AppliedTextDecorations() as *const AppliedTextDecorationVector
        } else {
            decorations
        };
        Self {
            content_offset_in_container_: *content_offset_in_container,
            used_font_: UsedFont::new(used_font.GetFont(), used_font.ScalingFactor()),
            style_: Member::from_ptr(style as *const ComputedStyle as *mut ComputedStyle),
            decorations_: Member::from_ptr(decorations.cast_mut()),
        }
    }

    // Source: third_party/blink/renderer/core/paint/decorating_box.h:34-43.
    pub fn from_item_with_style(
        item: &FragmentItem,
        style: &ComputedStyle,
        used_font: &UsedFont,
        decorations: *const AppliedTextDecorationVector,
    ) -> Self {
        Self::new(
            &item.ContentOffsetInContainerFragment(),
            style,
            used_font,
            decorations,
        )
    }

    // Source: third_party/blink/renderer/core/paint/decorating_box.h:44-49.
    pub fn from_item(item: &FragmentItem) -> Self {
        Self::from_item_with_style(item, item.Style(), &item.GetUsedFont(), std::ptr::null())
    }

    // Source: third_party/blink/renderer/core/paint/decorating_box.h:51-55.
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.used_font_.Trace(visitor);
        visitor.Trace(&self.style_);
        visitor.Trace(&self.decorations_);
    }

    // Source: third_party/blink/renderer/core/paint/decorating_box.h:57-68.
    pub fn ContentOffsetInContainer(&self) -> &PhysicalOffset {
        &self.content_offset_in_container_
    }

    pub fn Style(&self) -> &ComputedStyle {
        unsafe { &*self.style_.Get() }
    }

    pub fn GetUsedFont(&self) -> &UsedFont {
        &self.used_font_
    }

    pub fn AppliedTextDecorations(&self) -> *const AppliedTextDecorationVector {
        self.decorations_.Get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_engine::Font;
    use foundation::{LayoutHeapScope, LayoutUnit, MakeGarbageCollected};

    #[test]
    fn null_decoration_override_uses_the_style_vector() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let font = unsafe { &*MakeGarbageCollected(Font::default()) };
        let used_font = UsedFont::new(font, 1.25);
        let offset = PhysicalOffset::new(LayoutUnit::from_signed(3), LayoutUnit::from_signed(5));
        let decorating_box = DecoratingBox::new(&offset, &style, &used_font, std::ptr::null());

        assert_eq!(decorating_box.ContentOffsetInContainer(), &offset);
        assert!(std::ptr::eq(decorating_box.Style(), &style));
        assert_eq!(decorating_box.GetUsedFont().ScalingFactor(), 1.25);
        assert!(std::ptr::eq(
            decorating_box.AppliedTextDecorations(),
            style.AppliedTextDecorations(),
        ));
    }

    #[test]
    fn explicit_decoration_vector_keeps_its_identity() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let font = unsafe { &*MakeGarbageCollected(Font::default()) };
        let used_font = UsedFont::new(font, 1.0);
        let decorations = MakeGarbageCollected(AppliedTextDecorationVector::default());
        let decorating_box =
            DecoratingBox::new(&PhysicalOffset::default(), &style, &used_font, decorations);

        assert!(std::ptr::eq(
            decorating_box.AppliedTextDecorations(),
            decorations,
        ));
    }
}
