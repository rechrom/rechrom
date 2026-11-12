//! Context shared by inline fragments while resolving propagated decorations.

#![allow(non_snake_case)]

use foundation::{PhysicalOffset, TextDecorationLine};
use layoutng_assembly::{
    fragment_item::{FragmentItem, ItemType},
    inline_cursor::InlineCursor,
    internal::{
        layout_inline::LayoutInline, layout_object::LayoutObject,
        style_variant::ToParentStyleVariant, used_font::UsedFont,
    },
};
use layoutng_style::style::{
    applied_text_decoration::AppliedTextDecorationVector, computed_style::ComputedStyle,
};

use crate::decorating_box::DecoratingBox;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub type DecoratingBoxList = Vec<DecoratingBox>;

// Source: third_party/blink/renderer/core/paint/inline_paint_context.h:17-113.
#[derive(Default)]
pub struct InlinePaintContext {
    decorating_boxes_: DecoratingBoxList,
    last_decorations_: *const AppliedTextDecorationVector,
    line_decorations_: *const AppliedTextDecorationVector,
    line_cursor_: Option<InlineCursor>,
    paint_offset_: PhysicalOffset,
}

impl InlinePaintContext {
    // Source: third_party/blink/renderer/core/paint/inline_paint_context.h:23-28.
    pub fn DecoratingBoxes(&self) -> &DecoratingBoxList {
        &self.decorating_boxes_
    }

    pub fn CursorForDescendantsOfLine(&self) -> InlineCursor {
        self.line_cursor_
            .as_ref()
            .expect("inline paint context has no line cursor")
            .CursorForDescendants()
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.h:30-38.
    pub fn PushDecoratingBox(
        &mut self,
        offset: &PhysicalOffset,
        style: &ComputedStyle,
        used_font: &UsedFont,
        decorations: *const AppliedTextDecorationVector,
    ) {
        self.decorating_boxes_
            .push(DecoratingBox::new(offset, style, used_font, decorations));
    }

    pub fn PushDecoratingBoxes(&mut self, boxes: &[DecoratingBox]) {
        self.decorating_boxes_.extend(boxes.iter().cloned());
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.h:115-118.
    pub fn PopDecoratingBox(&mut self, size: usize) {
        debug_assert!(size <= self.decorating_boxes_.len());
        self.decorating_boxes_
            .truncate(self.decorating_boxes_.len() - size);
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:12-20.
    pub fn ClearDecoratingBoxes(&mut self, saved: Option<&mut DecoratingBoxList>) {
        if let Some(saved) = saved {
            debug_assert!(saved.is_empty());
            std::mem::swap(&mut self.decorating_boxes_, saved);
        } else {
            self.decorating_boxes_.clear();
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.h:43-46.
    pub fn PaintOffset(&self) -> &PhysicalOffset {
        &self.paint_offset_
    }

    pub fn SetPaintOffset(&mut self, paint_offset: &PhysicalOffset) {
        self.paint_offset_ = *paint_offset;
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:22-32.
    pub fn ScopeInlineItem<'a>(&'a mut self, item: &FragmentItem) -> ScopedInlineItem<'a> {
        let last_decorations = self.last_decorations_;
        let mut saved_decorating_boxes = Vec::new();
        let push_count = self.SyncDecoratingBox(item, &mut saved_decorating_boxes);
        debug_assert_eq!(
            self.decorating_boxes_.len(),
            item.Style().AppliedTextDecorations().len()
        );
        ScopedInlineItem {
            inline_context_: self,
            last_decorations_: last_decorations,
            saved_decorating_boxes_: saved_decorating_boxes,
            push_count_: push_count,
            lifetime_: PhantomData,
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:39-268.
    fn SyncDecoratingBox(
        &mut self,
        item: &FragmentItem,
        saved_decorating_boxes: &mut DecoratingBoxList,
    ) -> usize {
        debug_assert!(saved_decorating_boxes.is_empty());
        let style = item.Style();
        let decorations = style.AppliedTextDecorations() as *const AppliedTextDecorationVector;
        debug_assert!(!self.last_decorations_.is_null());
        if decorations == self.last_decorations_ {
            return 0;
        }

        let stop_at = self.last_decorations_;
        let mut synchronizer = DecorationBoxSynchronizer {
            inline_context_: self,
            stop_at_: stop_at,
            line_cursor_: None,
            saved_decorating_boxes,
            style_variant_: ToParentStyleVariant(item.GetStyleVariant()),
        };
        let push_count =
            synchronizer.Sync(item, unsafe { &*item.GetLayoutObject() }, style, unsafe {
                &*decorations
            });
        self.last_decorations_ = decorations;
        push_count
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:278-302.
    pub fn PushDecoratingBoxAncestors(&mut self, inline_box: &InlineCursor) {
        debug_assert!(inline_box.Current().IsNotNull());
        debug_assert!(inline_box.Current().IsInlineBox());
        debug_assert!(self.decorating_boxes_.is_empty());

        let mut ancestor_items: Vec<*const FragmentItem> = Vec::new();
        let mut cursor = inline_box.clone();
        loop {
            cursor.MoveToParent();
            let current = cursor.Current();
            debug_assert!(current.IsNotNull());
            if current.IsLineBox() {
                self.SetLineBox(&cursor);
                let mut saved = Vec::new();
                for item in ancestor_items.into_iter().rev() {
                    self.SyncDecoratingBox(unsafe { &*item }, &mut saved);
                }
                return;
            }
            debug_assert!(current.IsInlineBox());
            ancestor_items.push(current.Item());
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:317-353.
    pub fn SetLineBox(&mut self, line_cursor: &InlineCursor) {
        debug_assert_eq!(
            unsafe { &*line_cursor.CurrentItem() }.Type(),
            ItemType::kLine
        );
        self.line_cursor_ = Some(line_cursor.clone());
        debug_assert!(self.decorating_boxes_.is_empty());

        let line_item = unsafe { &*line_cursor.CurrentItem() };
        let style = line_item.Style();
        let used_font = line_item.GetUsedFont();
        let decorations = style.AppliedTextDecorations();
        self.line_decorations_ = decorations;
        self.last_decorations_ = decorations;
        if decorations.is_empty() {
            return;
        }

        let mut offset = *line_item.OffsetInContainerFragment();
        let fragment = line_item.LineBoxFragment();
        if !fragment.is_null() && !used_font.PrimaryFont().is_null() {
            offset.top += unsafe { &*fragment }.Metrics().ascent;
            offset.top -= used_font.FixedAscent();
        }
        for _ in 0..decorations.len() {
            self.PushDecoratingBox(&offset, style, &used_font, decorations);
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:355-360.
    pub fn ClearLineBox(&mut self) {
        self.last_decorations_ = std::ptr::null();
        self.line_decorations_ = std::ptr::null();
        self.line_cursor_ = None;
        self.decorating_boxes_.clear();
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:270-276.
    pub fn ScopeInlineBoxAncestors<'a>(
        &'a mut self,
        inline_box: &InlineCursor,
    ) -> ScopedInlineBoxAncestors<'a> {
        self.PushDecoratingBoxAncestors(inline_box);
        ScopedInlineBoxAncestors {
            inline_context_: self,
            lifetime_: PhantomData,
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:309-315.
    pub fn ScopeLineBox<'a>(&'a mut self, line_cursor: &InlineCursor) -> ScopedLineBox<'a> {
        self.SetLineBox(line_cursor);
        ScopedLineBox {
            inline_context_: self,
            lifetime_: PhantomData,
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.h:90-100.
    pub fn ScopePaintOffset<'a>(
        &'a mut self,
        paint_offset: &PhysicalOffset,
    ) -> ScopedPaintOffset<'a> {
        let previous = self.paint_offset_;
        self.paint_offset_ = *paint_offset;
        ScopedPaintOffset {
            inline_context_: self,
            previous_: previous,
            lifetime_: PhantomData,
        }
    }
}

struct DecorationBoxSynchronizer<'a> {
    inline_context_: &'a mut InlinePaintContext,
    stop_at_: *const AppliedTextDecorationVector,
    line_cursor_: Option<InlineCursor>,
    saved_decorating_boxes: &'a mut DecoratingBoxList,
    style_variant_: layoutng_assembly::internal::style_variant::StyleVariant,
}

impl DecorationBoxSynchronizer<'_> {
    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:70-185.
    fn Sync(
        &mut self,
        mut item: *const FragmentItem,
        mut layout_object: *const LayoutObject,
        mut style: *const ComputedStyle,
        mut decorations: *const AppliedTextDecorationVector,
    ) -> usize {
        loop {
            debug_assert!(item.is_null() || unsafe { &*item }.GetLayoutObject() == layout_object);
            debug_assert!(std::ptr::eq(
                unsafe { &*layout_object }.EffectiveStyle(self.style_variant_),
                unsafe { &*style }
            ));
            debug_assert!(std::ptr::eq(
                unsafe { &*style }.AppliedTextDecorations(),
                unsafe { &*decorations }
            ));
            debug_assert_ne!(decorations, self.stop_at_);

            let parent = unsafe { &*layout_object }.Parent();
            debug_assert!(!parent.is_null());
            let parent_style = unsafe { &*parent }.EffectiveStyle(self.style_variant_);
            let parent_decorations = parent_style.AppliedTextDecorations();
            let parent_decorations_ptr = parent_decorations as *const AppliedTextDecorationVector;

            if decorations != parent_decorations_ptr {
                let decorations_ref = unsafe { &*decorations };
                if decorations_ref.len() > parent_decorations.len() {
                    let mut pushes = 0;
                    if parent_decorations_ptr != self.stop_at_ {
                        pushes =
                            self.Sync(std::ptr::null(), parent, parent_style, parent_decorations);
                    }
                    pushes += self.PushDecoratingBoxesUntilParent(
                        item,
                        unsafe { &*layout_object },
                        unsafe { &*style },
                        decorations_ref,
                        parent_decorations,
                    );
                    return pushes;
                }

                if decorations_ref.len() == parent_decorations.len()
                    && (unsafe { &*style }.GetTextDecorationLine() == TextDecorationLine::kNone
                        || unsafe { &*layout_object }.IsText())
                {
                    if parent_decorations_ptr == self.stop_at_ {
                        return 0;
                    }
                    return self.Sync(std::ptr::null(), parent, parent_style, parent_decorations);
                }

                if decorations_ref.is_empty() {
                    self.inline_context_
                        .ClearDecoratingBoxes(Some(self.saved_decorating_boxes));
                    return 0;
                }
                if decorations_ref.len() == 1
                    && decorations_ref[0].Lines() == unsafe { &*style }.GetTextDecorationLine()
                {
                    self.inline_context_
                        .ClearDecoratingBoxes(Some(self.saved_decorating_boxes));
                    self.PushDecoratingBox(
                        item,
                        unsafe { &*layout_object },
                        unsafe { &*style },
                        decorations_ref,
                    );
                    return 1;
                }

                let base = unsafe { &*style }.BaseTextDecorationData();
                if base != parent_decorations_ptr.cast_mut() {
                    self.inline_context_
                        .ClearDecoratingBoxes(Some(self.saved_decorating_boxes));
                    let size = self.saved_decorating_boxes.len().min(decorations_ref.len());
                    self.inline_context_
                        .PushDecoratingBoxes(&self.saved_decorating_boxes[..size]);
                    return size;
                }
                #[cfg(debug_assertions)]
                unsafe {
                    (&*layout_object).ShowLayoutTreeForThis();
                }
                panic!("unexpected applied text-decoration propagation tree");
            }

            if !unsafe { &*parent }.IsLayoutInline() {
                return 0;
            }
            #[cfg(debug_assertions)]
            unsafe {
                let layout_inline = &*parent.cast::<LayoutInline>();
                let grandparent = (&*parent).Parent();
                debug_assert!(!grandparent.is_null());
                let duplicated_parent_decorations = parent_decorations.len()
                    == (&*grandparent).StyleRef().AppliedTextDecorations().len()
                    && parent_style.GetTextDecorationLine() == TextDecorationLine::kNone;
                if !duplicated_parent_decorations && !(&*layout_object).IsText() {
                    debug_assert!(!layout_inline.ShouldCreateBoxFragment());
                    debug_assert!(!layout_inline.HasInlineFragments());
                }
            }
            item = std::ptr::null();
            layout_object = parent;
            style = parent_style;
            decorations = parent_decorations;
        }
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:187-232.
    fn PushDecoratingBoxesUntilParent(
        &mut self,
        item: *const FragmentItem,
        layout_object: &LayoutObject,
        style: &ComputedStyle,
        decorations: &AppliedTextDecorationVector,
        parent_decorations: &AppliedTextDecorationVector,
    ) -> usize {
        let base = style.BaseTextDecorationData();
        if std::ptr::eq(base, parent_decorations as *const _ as *mut _) {
            debug_assert_eq!(decorations.len(), parent_decorations.len() + 1);
            debug_assert_ne!(style.GetTextDecorationLine(), TextDecorationLine::kNone);
            self.PushDecoratingBox(item, layout_object, style, decorations);
            return 1;
        }
        if !base.is_null()
            && !std::ptr::eq(base, decorations as *const _ as *mut _)
            && decorations.len() == parent_decorations.len() + 2
        {
            debug_assert_ne!(style.GetTextDecorationLine(), TextDecorationLine::kNone);
            self.PushDecoratingBox(item, layout_object, style, unsafe { &*base });
            self.PushDecoratingBox(item, layout_object, style, decorations);
            return 2;
        }
        if decorations.len() == parent_decorations.len() + 1 {
            self.PushDecoratingBox(item, layout_object, style, decorations);
            return 1;
        }
        if decorations.len() > parent_decorations.len() {
            let count = decorations.len() - parent_decorations.len();
            for _ in 0..count {
                self.PushDecoratingBox(item, layout_object, style, decorations);
            }
            return count;
        }
        debug_assert_eq!(decorations.len(), parent_decorations.len());
        0
    }

    // Source: third_party/blink/renderer/core/paint/inline_paint_context.cc:234-253.
    fn PushDecoratingBox(
        &mut self,
        mut item: *const FragmentItem,
        layout_object: &LayoutObject,
        style: &ComputedStyle,
        decorations: &AppliedTextDecorationVector,
    ) {
        debug_assert!(item.is_null() || unsafe { &*item }.GetLayoutObject() == layout_object);
        if item.is_null() {
            if self.line_cursor_.is_none() {
                self.line_cursor_ = Some(self.inline_context_.CursorForDescendantsOfLine());
            }
            let cursor = self.line_cursor_.as_mut().expect("line cursor initialized");
            cursor.MoveToIncludingCulledInline(layout_object);
            debug_assert!(cursor.IsNotNull());
            item = cursor.CurrentItem();
        }
        debug_assert!(!item.is_null());
        let item = unsafe { &*item };
        self.inline_context_.PushDecoratingBox(
            &item.ContentOffsetInContainerFragment(),
            style,
            &item.GetUsedFont(),
            decorations,
        );
    }
}

// Source: third_party/blink/renderer/core/paint/inline_paint_context.h:48-62,120-130.
pub struct ScopedInlineItem<'a> {
    inline_context_: *mut InlinePaintContext,
    last_decorations_: *const AppliedTextDecorationVector,
    saved_decorating_boxes_: DecoratingBoxList,
    push_count_: usize,
    lifetime_: PhantomData<&'a mut InlinePaintContext>,
}

impl Drop for ScopedInlineItem<'_> {
    fn drop(&mut self) {
        let context = unsafe { &mut *self.inline_context_ };
        context.last_decorations_ = self.last_decorations_;
        if !self.saved_decorating_boxes_.is_empty() {
            std::mem::swap(
                &mut context.decorating_boxes_,
                &mut self.saved_decorating_boxes_,
            );
        } else if self.push_count_ != 0 {
            context.PopDecoratingBox(self.push_count_);
        }
    }
}

impl Deref for ScopedInlineItem<'_> {
    type Target = InlinePaintContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inline_context_ }
    }
}

impl DerefMut for ScopedInlineItem<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inline_context_ }
    }
}

// Source: third_party/blink/renderer/core/paint/inline_paint_context.h:64-75,132-136.
pub struct ScopedInlineBoxAncestors<'a> {
    inline_context_: *mut InlinePaintContext,
    lifetime_: PhantomData<&'a mut InlinePaintContext>,
}

impl Drop for ScopedInlineBoxAncestors<'_> {
    fn drop(&mut self) {
        unsafe { &mut *self.inline_context_ }.ClearLineBox();
    }
}

impl Deref for ScopedInlineBoxAncestors<'_> {
    type Target = InlinePaintContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inline_context_ }
    }
}

impl DerefMut for ScopedInlineBoxAncestors<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inline_context_ }
    }
}

// Source: third_party/blink/renderer/core/paint/inline_paint_context.h:77-88,138-141.
pub struct ScopedLineBox<'a> {
    inline_context_: *mut InlinePaintContext,
    lifetime_: PhantomData<&'a mut InlinePaintContext>,
}

impl Drop for ScopedLineBox<'_> {
    fn drop(&mut self) {
        unsafe { &mut *self.inline_context_ }.ClearLineBox();
    }
}

impl Deref for ScopedLineBox<'_> {
    type Target = InlinePaintContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inline_context_ }
    }
}

impl DerefMut for ScopedLineBox<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inline_context_ }
    }
}

// Source: third_party/blink/renderer/core/paint/inline_paint_context.h:90-100.
pub struct ScopedPaintOffset<'a> {
    inline_context_: *mut InlinePaintContext,
    previous_: PhysicalOffset,
    lifetime_: PhantomData<&'a mut InlinePaintContext>,
}

impl Drop for ScopedPaintOffset<'_> {
    fn drop(&mut self) {
        unsafe { &mut *self.inline_context_ }.paint_offset_ = self.previous_;
    }
}

impl Deref for ScopedPaintOffset<'_> {
    type Target = InlinePaintContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inline_context_ }
    }
}

impl DerefMut for ScopedPaintOffset<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inline_context_ }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_engine::Font;
    use foundation::{LayoutHeapScope, LayoutUnit, MakeGarbageCollected};

    #[test]
    fn scoped_paint_offset_restores_the_previous_value() {
        let mut context = InlinePaintContext::default();
        let previous = PhysicalOffset::new(LayoutUnit::from_signed(3), LayoutUnit::from_signed(4));
        let scoped = PhysicalOffset::new(LayoutUnit::from_signed(8), LayoutUnit::from_signed(9));
        context.SetPaintOffset(&previous);
        {
            let scope = context.ScopePaintOffset(&scoped);
            assert_eq!(scope.PaintOffset(), &scoped);
        }
        assert_eq!(context.PaintOffset(), &previous);
    }

    #[test]
    fn saved_decorating_boxes_restore_pointer_identity_and_order() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let font = unsafe { &*MakeGarbageCollected(Font::default()) };
        let used_font = UsedFont::new(font, 1.0);
        let first = PhysicalOffset::new(LayoutUnit::from_signed(1), LayoutUnit::default());
        let second = PhysicalOffset::new(LayoutUnit::from_signed(2), LayoutUnit::default());
        let mut context = InlinePaintContext::default();
        context.PushDecoratingBox(&first, &style, &used_font, std::ptr::null());
        context.PushDecoratingBox(&second, &style, &used_font, std::ptr::null());

        let mut saved = Vec::new();
        context.ClearDecoratingBoxes(Some(&mut saved));
        assert!(context.DecoratingBoxes().is_empty());
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].ContentOffsetInContainer(), &first);
        assert_eq!(saved[1].ContentOffsetInContainer(), &second);

        context.PushDecoratingBoxes(&saved);
        context.PopDecoratingBox(1);
        assert_eq!(context.DecoratingBoxes().len(), 1);
        assert_eq!(
            context.DecoratingBoxes()[0].ContentOffsetInContainer(),
            &first
        );
        assert!(std::ptr::eq(
            context.DecoratingBoxes()[0].AppliedTextDecorations(),
            style.AppliedTextDecorations(),
        ));
    }
}
