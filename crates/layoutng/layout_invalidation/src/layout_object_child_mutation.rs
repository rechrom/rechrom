//! LayoutObject child insertion and removal policy.
//!
//! Structural mutation may select downstream factories, normalize inline text,
//! transform text, or merge adjacent box-model objects. Keep those stages
//! under invalidation-owned sampling frames instead of core self time.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:581-582
    pub fn AddChildDefault(&mut self, new_child: *mut LayoutObject) {
        self.AddChild(new_child, std::ptr::null_mut());
    }

    #[inline(never)]
    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAddChild(self, new_child, before_child) };
    }

    // Base virtual implementation. Derived layouts may override insertion.
    // cpp: layoutng/internal/layout_object_core_services.cc:107-148
    #[inline(never)]
    pub fn AddChildBase(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        if children.is_null() {
            eprint!(
                "layout tree parent cannot contain children: {}",
                self.GetName()
            );
            let node = self.GetNode();
            if !node.is_null() {
                eprint!(" ({})", unsafe { &*node }.InputDebugName());
            }
            eprintln!();
        }
        debug_assert!(!children.is_null());
        if children.is_null() {
            return;
        }

        let factories = LayoutObjectFactoryScope::Objects();
        let new_child_ref = unsafe { &mut *new_child };
        if new_child_ref.IsTablePart() {
            let insert_table_child = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.insert_table_child
            };
            let Some(insert_table_child) = insert_table_child else {
                std::panic::panic_any(UnsupportedLayout::new(
                    "table layout module is not installed",
                ));
            };
            insert_table_child(self, new_child_ref, before_child);
        } else if new_child_ref.IsText()
            && !IsHorizontalTypographicMode(new_child_ref.StyleRef().GetWritingMode())
        {
            let insert_inline_text_child = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.insert_inline_text_child
            };
            if !insert_inline_text_child
                .is_some_and(|insert| insert(self, new_child_ref, before_child))
            {
                if insert_inline_text_child.is_none()
                    && !new_child_ref.IsSVGInlineText()
                    && new_child_ref.StyleRef().HasTextCombine()
                {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "inline layout module is not installed",
                    ));
                }
                unsafe { &mut *children }.InsertChildNodeDefault(self, new_child, before_child);
            }
        } else {
            unsafe { &mut *children }.InsertChildNodeDefault(self, new_child, before_child);
        }

        let text = DynamicTo::<LayoutText>(new_child);
        if !text.is_null()
            && (new_child_ref.StyleRef().TextTransform().bits()
                & ETextTransform::kCapitalize.bits())
                != 0
        {
            unsafe { &mut *text }.TransformAndSecureOriginalText();
        }
    }

    // Base virtual implementation. Derived layouts may override removal.
    // cpp: layoutng/internal/layout_object_core_services.cc:150-162
    #[inline(never)]
    pub fn RemoveChildBase(&mut self, old_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        debug_assert!(!children.is_null());
        if children.is_null() {
            return;
        }
        let previous = DynamicTo::<LayoutBoxModelObject>(unsafe { &*old_child }.PreviousSibling());
        let next = DynamicTo::<LayoutBoxModelObject>(unsafe { &*old_child }.NextSibling());
        unsafe { &mut *children }.RemoveChildNodeDefault(self, old_child);
        LayoutBoxModelObject::AttemptToMerge(previous, next);
    }

    #[inline(never)]
    pub fn RemoveChild(&mut self, old_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectRemoveChild(self, old_child) };
    }
}
