use std::cell::Cell;
use std::ops::Index;

use foundation::{
    ClearCollectionScope, DynamicTo, HeapHashMap, HeapVector, IsParallelWritingMode, Member,
    RuntimeEnabledFeatures, String, To, Traceable, Visitor,
};
use layoutng::internal::fragmentation_utils::IsBreakInside;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;

use crate::fragment_item::{FragmentItem, FragmentItemType};
use crate::fragment_items_builder::FragmentItemsBuilder;
use crate::inline_cursor::InlineCursor;
use crate::layout_result::LayoutResult;
use crate::physical_box_fragment::PhysicalBoxFragment;

// C++ keeps the parent GC object plus an index because the traceable item may
// move with its vector. Never keep a bare FragmentItem pointer in this map.
// cpp: layoutng_fragment_tree/fragment_items.cc:80-106
struct FragmentItemPtr {
    fragment_: Member<PhysicalBoxFragment>,
    index_: u32,
}

#[allow(non_snake_case)]
impl FragmentItemPtr {
    fn new(item: &FragmentItem, fragment: &PhysicalBoxFragment, items: &[FragmentItem]) -> Self {
        let index = unsafe { (item as *const FragmentItem).offset_from(items.as_ptr()) };
        assert!(index >= 0 && (index as usize) < items.len());
        #[cfg(debug_assertions)]
        {
            let fragment_items = fragment.Items();
            debug_assert!(!fragment_items.is_null());
            debug_assert_eq!(unsafe { &*fragment_items }.Items().as_ptr(), items.as_ptr());
        }
        Self {
            fragment_: Member::from_ptr(fragment as *const _ as *mut _),
            index_: index as u32,
        }
    }

    fn Get(&self) -> *const FragmentItem {
        let fragment = self.fragment_.Get();
        if fragment.is_null() {
            return std::ptr::null();
        }
        let items = unsafe { &*fragment }.Items();
        let fragment_items = unsafe { &*items };
        &fragment_items[self.index_ as usize]
    }

    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fragment_);
    }
}

// cpp: foundation/blink_base/heap/trace_traits.h:20-28
impl Traceable for FragmentItemPtr {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FragmentItemPtr::Trace(self, visitor);
    }
}

// cpp: layoutng_fragment_tree/fragment_items.cc:108-119
struct LastItem {
    item: FragmentItemPtr,
    fragment_id: u32,
    total_item_index: u32,
}

#[allow(non_snake_case)]
impl LastItem {
    fn Trace(&self, visitor: &mut Visitor) {
        self.item.Trace(visitor);
    }
}

impl Traceable for LastItem {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LastItem::Trace(self, visitor);
    }
}

// cpp: layoutng_fragment_tree/fragment_items.cc:23-34
#[cfg(debug_assertions)]
#[allow(non_snake_case)]
fn CheckNoItemsAreAssociated(fragment: &PhysicalBoxFragment) {
    let fragment_items = fragment.Items();
    if !fragment_items.is_null() {
        for item in unsafe { &*fragment_items }.Items() {
            if item.Type() == FragmentItemType::kLine {
                continue;
            }
            let layout_object = item.GetLayoutObject();
            if !layout_object.is_null() {
                debug_assert_eq!(unsafe { &*layout_object }.FirstInlineFragmentItemIndex(), 0);
            }
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_items.cc:36-43
#[cfg(debug_assertions)]
#[allow(non_snake_case)]
fn CheckIsLast(item: &FragmentItem) {
    let fragment = item.BoxFragment();
    if !fragment.is_null() && !unsafe { &*fragment }.IsInline() {
        debug_assert!(!unsafe { &*fragment }.IsInlineBox());
        debug_assert_eq!(
            item.IsLastForNode(),
            unsafe { &*fragment }.GetBreakToken().is_null()
        );
    }
}

// The C++ mutable field is written through a const FragmentItems during
// finalization. Cell preserves that narrow interior-mutability contract.
// cpp: layoutng_fragment_tree/fragment_items.h:19-24
// cpp: layoutng_fragment_tree/fragment_items.h:133-140
pub struct FragmentItems {
    text_content_: String,
    first_line_text_content_: String,
    size_of_earlier_fragments_: Cell<u32>,
    items_: HeapVector<FragmentItem>,
}

impl Clone for FragmentItems {
    // cpp: layoutng_fragment_tree/fragment_items.h:27
    // cpp: layoutng_fragment_tree/fragment_items.cc:57-70
    fn clone(&self) -> Self {
        let copied = Self {
            text_content_: self.text_content_.clone(),
            first_line_text_content_: self.first_line_text_content_.clone(),
            size_of_earlier_fragments_: Cell::new(self.size_of_earlier_fragments_.get()),
            items_: self.items_.clone(),
        };
        for other_item in &self.items_ {
            let layout_text = DynamicTo::<LayoutText>(other_item.GetMutableLayoutObject());
            if !layout_text.is_null() {
                unsafe { &mut *layout_text }.DetachAxHooksIfNeeded();
            }
        }
        copied
    }
}

impl Index<usize> for FragmentItems {
    type Output = FragmentItem;

    // cpp: layoutng_fragment_tree/fragment_items.h:39-41
    fn index(&self, index: usize) -> &Self::Output {
        &self.items_[index]
    }
}

#[allow(non_snake_case)]
impl FragmentItems {
    // cpp: layoutng_fragment_tree/fragment_items.h:28
    // cpp: layoutng_fragment_tree/fragment_items.cc:48-55
    pub fn from_builder(builder: &mut FragmentItemsBuilder) -> Self {
        let text_content = std::mem::take(&mut builder.text_content_);
        let first_line_text_content = std::mem::take(&mut builder.first_line_text_content_);
        let builder_items = std::mem::take(&mut builder.items_);
        let mut items = HeapVector::default();
        items.ReserveInitialCapacity(builder_items.len() as u32);
        for item in builder_items {
            items.push(item.item);
        }
        Self {
            text_content_: text_content,
            first_line_text_content_: first_line_text_content,
            size_of_earlier_fragments_: Cell::new(0),
            items_: items,
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:30-37
    pub fn Size(&self) -> u32 {
        self.items_.len() as u32
    }

    pub fn Items(&self) -> &[FragmentItem] {
        &self.items_
    }

    pub fn Equals(&self, span: &[FragmentItem]) -> bool {
        self.ItemsData() == span.as_ptr() && self.Size() as usize == span.len()
    }

    // cpp: layoutng_fragment_tree/fragment_items.cc:72-75
    pub fn IsSubSpan(&self, span: &[FragmentItem]) -> bool {
        span.is_empty()
            || (span.as_ptr().addr() >= self.ItemsData().addr()
                && !self.items_.is_empty()
                && unsafe { span.as_ptr().add(span.len() - 1) }.addr()
                    <= (&self.items_[self.items_.len() - 1] as *const FragmentItem).addr())
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:42-45
    pub fn front(&self) -> &FragmentItem {
        assert!(self.items_.len() >= 1);
        &self.items_[0]
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:47-59
    pub fn NormalText(&self) -> &String {
        &self.text_content_
    }

    pub fn FirstLineText(&self) -> &String {
        &self.first_line_text_content_
    }

    pub fn Text(&self, first_line: bool) -> &String {
        if first_line && !self.first_line_text_content_.IsNull() {
            return &self.first_line_text_content_;
        }
        &self.text_content_
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:61-71
    pub fn SizeOfEarlierFragments(&self) -> u32 {
        self.size_of_earlier_fragments_.get()
    }

    pub fn EndItemIndex(&self) -> u32 {
        self.SizeOfEarlierFragments() + self.Size()
    }

    pub fn HasItemIndex(&self, index: u32) -> bool {
        index >= self.SizeOfEarlierFragments() && index < self.EndItemIndex()
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:123-124
    fn ItemsData(&self) -> *const FragmentItem {
        self.items_.as_ptr()
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:73-77
    // cpp: layoutng_fragment_tree/fragment_items.cc:77-79
    #[allow(clippy::too_many_lines)]
    pub fn FinalizeAfterLayout(
        results: &HeapVector<Member<LayoutResult>, 1>,
        container: &mut LayoutBlockFlow,
    ) {
        // cpp: layoutng_fragment_tree/fragment_items.cc:120-134
        let mut last_items: HeapHashMap<Member<LayoutObject>, LastItem> = HeapHashMap::default();
        let _clear_scope = unsafe { ClearCollectionScope::new(&mut last_items as *mut _) };
        let mut total_item_index = 0u32;
        let mut line_fragment_id = FragmentItem::kInitialLineFragmentId;
        let mut may_be_non_contiguous_ifc = false;
        let mut has_regular_break = false;

        // cpp: layoutng_fragment_tree/fragment_items.cc:136-166
        for result in results {
            let physical = unsafe { &*result.Get() }.GetPhysicalFragment();
            let fragment = unsafe { &*To::<PhysicalBoxFragment>(physical as *const _) };
            let fragment_items = fragment.Items();
            if fragment_items.is_null() {
                may_be_non_contiguous_ifc = true;
                continue;
            }
            let break_token = fragment.GetBreakToken();
            if !break_token.is_null() {
                if IsBreakInside(break_token) {
                    has_regular_break = true;
                } else if has_regular_break && unsafe { &*break_token }.IsRepeated() {
                    may_be_non_contiguous_ifc = true;
                }
            }
            let mut found_inflow_content = false;
            let fragment_items = unsafe { &*fragment_items };
            fragment_items
                .size_of_earlier_fragments_
                .set(total_item_index);
            let items = fragment_items.Items();

            // cpp: layoutng_fragment_tree/fragment_items.cc:167-213
            for item in items {
                total_item_index += 1;
                if item.Type() == FragmentItemType::kLine {
                    debug_assert_eq!(item.DeltaToNextForSameLayoutObject(), 0);
                    item.SetFragmentId(line_fragment_id);
                    line_fragment_id += 1;
                    continue;
                } else if item.IsEllipsis() && item.GetLayoutObject() == fragment.GetLayoutObject()
                {
                    debug_assert!(
                        RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled()
                    );
                    continue;
                } else if !found_inflow_content {
                    found_inflow_content =
                        !item.IsFloating() && !item.IsInlineBox() && !item.IsBlockInInline();
                }
                let layout_object = item.GetMutableLayoutObject();
                debug_assert!(!unsafe { &*layout_object }.IsOutOfFlowPositioned());
                debug_assert!(unsafe { &*layout_object }.IsInLayoutNGInlineFormattingContext());
                item.SetDeltaToNextForSameLayoutObject(0);
                let use_break_token = unsafe { &*layout_object }.IsFloating()
                    || !unsafe { &*layout_object }.IsInline();
                if use_break_token {
                    let item_fragment = item.BoxFragment();
                    debug_assert!(!item_fragment.is_null());
                    debug_assert!(!unsafe { &*item_fragment }.IsInlineBox());
                    item.SetIsLastForNode(unsafe { &*item_fragment }.GetBreakToken().is_null());
                } else {
                    debug_assert!(unsafe { &*layout_object }.IsInline());
                    item.SetIsLastForNode(true);
                }

                // cpp: layoutng_fragment_tree/fragment_items.cc:215-225
                let key = Member::from_ptr(layout_object);
                let is_new_entry = last_items.insert(
                    key,
                    LastItem {
                        item: FragmentItemPtr::new(item, fragment, items),
                        fragment_id: 0,
                        total_item_index,
                    },
                );
                if is_new_entry {
                    item.SetFragmentId(0);
                    unsafe { &mut *layout_object }
                        .SetFirstInlineFragmentItemIndex(total_item_index as usize);
                    continue;
                }

                // cpp: layoutng_fragment_tree/fragment_items.cc:227-253
                let last = last_items.get_mut(&key).expect("inserted fragment item");
                let last_item = unsafe { &*last.item.Get() };
                debug_assert_eq!(last_item.DeltaToNextForSameLayoutObject(), 0);
                let last_index = last.total_item_index;
                debug_assert!(last_index > 0);
                debug_assert!(last_index < fragment_items.EndItemIndex());
                debug_assert!(last_index < total_item_index);
                last_item.SetDeltaToNextForSameLayoutObject(total_item_index - last_index);
                if !use_break_token && !(unsafe { &*layout_object }.IsBox() && item.IsEllipsis()) {
                    last_item.SetIsLastForNode(false);
                }
                #[cfg(debug_assertions)]
                CheckIsLast(last_item);
                last.fragment_id += 1;
                item.SetFragmentId(last.fragment_id);
                last.item = FragmentItemPtr::new(item, fragment, items);
                last.total_item_index = total_item_index;
            }

            // cpp: layoutng_fragment_tree/fragment_items.cc:255-258
            if !found_inflow_content {
                may_be_non_contiguous_ifc = true;
            }
        }

        // cpp: layoutng_fragment_tree/fragment_items.cc:260-265
        container.SetMayBeNonContiguousIfc(may_be_non_contiguous_ifc);
        #[cfg(debug_assertions)]
        for entry in last_items.values() {
            CheckIsLast(unsafe { &*entry.item.Get() });
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:80
    // cpp: layoutng_fragment_tree/fragment_items.cc:268-293
    pub fn ClearAssociatedFragments(container: *mut LayoutObject) {
        let mut child = unsafe { &*container }.SlowFirstChild();
        while !child.is_null() {
            let next = unsafe { &*child }.NextSibling();
            if unsafe { &*child }.IsInLayoutNGInlineFormattingContext()
                && !unsafe { &*child }.IsOutOfFlowPositioned()
            {
                unsafe { &mut *child }.ClearFirstInlineFragmentItemIndex();
                if unsafe { &*child }.IsLayoutInline() {
                    Self::ClearAssociatedFragments(child);
                }
            }
            child = next;
        }
        #[cfg(debug_assertions)]
        {
            let box_object = DynamicTo::<LayoutBox>(container);
            if !box_object.is_null() {
                for fragment in unsafe { &*box_object }.PhysicalFragments() {
                    CheckNoItemsAreAssociated(fragment);
                }
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:126
    // cpp: layoutng_fragment_tree/fragment_items.cc:295-307
    fn CanReuseAll(cursor: &mut InlineCursor) -> bool {
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.Current().Item() };
            if item.Type() != FragmentItemType::kLine && !item.CanReuse() {
                return false;
            }
            cursor.MoveToNext();
        }
        true
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:86-89
    // cpp: layoutng_fragment_tree/fragment_items.cc:309-361
    pub fn EndOfReusableItems(&self, container: &PhysicalBoxFragment) -> *const FragmentItem {
        let mut last_line_start = self.front() as *const FragmentItem;
        let mut cursor = InlineCursor::from_container_and_items(container, self);
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.Current().Item() };
            if item.IsDirty() {
                return item;
            }
            if item.Type() != FragmentItemType::kLine {
                return item;
            }
            let mut line = cursor.CursorForDescendants();
            if !Self::CanReuseAll(&mut line) {
                return last_line_start;
            }
            let line_box_fragment = unsafe { &*item.LineBoxFragment() };
            if line_box_fragment.HasPropagatedDescendants() {
                return item;
            }
            if line_box_fragment.IsEmptyLineBox() {
                return item;
            }
            if line_box_fragment.IsBlockInInline() {
                return item;
            }
            if line_box_fragment.GetBreakToken().is_null() {
                return item;
            }
            last_line_start = item;
            cursor.MoveToNextSkippingChildren();
        }
        std::ptr::null()
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:91-100
    // cpp: layoutng_fragment_tree/fragment_items.cc:363-450
    pub fn IsContainerForCulledInline(
        &self,
        layout_inline: &LayoutInline,
        is_first_container: &mut bool,
        is_last_container: &mut bool,
        child_has_any_child_items: &mut bool,
    ) -> bool {
        debug_assert!(!layout_inline.HasInlineFragments());
        let start_idx = self.size_of_earlier_fragments_.get();
        let end_idx = self.EndItemIndex();
        let mut found_item = false;
        let mut has_float_ahead = false;
        *is_first_container = true;
        *child_has_any_child_items = false;
        let mut descendant = layout_inline.FirstChild();
        while !descendant.is_null() {
            let mut item_idx = u32::try_from(unsafe { &*descendant }.FirstInlineFragmentItemIndex())
                .expect("fragment item index exceeds wtf_size_t");
            let next_descendant = if unsafe { &*descendant }.IsBox() || item_idx != 0 {
                unsafe { &*descendant }.NextInPreOrderAfterChildren(layout_inline as *const LayoutInline as *const LayoutObject)
            } else {
                unsafe { &*descendant }.NextInPreOrder(layout_inline as *const LayoutInline as *const LayoutObject)
            };
            if item_idx == 0 {
                descendant = next_descendant;
                continue;
            }
            *child_has_any_child_items = true;
            item_idx -= 1;
            if item_idx >= end_idx {
                if !found_item && unsafe { &*descendant }.IsFloating() {
                    has_float_ahead = true;
                    descendant = next_descendant;
                    continue;
                }
                *is_last_container = false;
                return found_item;
            }
            if item_idx < start_idx {
                *is_first_container = false;
                let mut cursor = InlineCursor::default();
                cursor.MoveToLayoutObject(unsafe { &*descendant });
                while cursor.IsNotNull() && item_idx < end_idx {
                    item_idx +=
                        unsafe { &*cursor.Current().Item() }.DeltaToNextForSameLayoutObject();
                    if item_idx >= start_idx {
                        if item_idx >= end_idx {
                            *is_last_container = false;
                            return found_item;
                        }
                        found_item = true;
                    }
                    cursor.MoveToNextForSameLayoutObject();
                }
                descendant = next_descendant;
                continue;
            }
            found_item = true;
            let mut item = &self.items_[(item_idx - start_idx) as usize] as *const FragmentItem;
            while !item.is_null() {
                let delta = unsafe { &*item }.DeltaToNextForSameLayoutObject();
                if delta != 0 {
                    item_idx += delta;
                    if item_idx >= end_idx {
                        *is_last_container = false;
                        return true;
                    }
                    item = &self.items_[(item_idx - start_idx) as usize];
                } else {
                    item = std::ptr::null();
                }
            }
            descendant = next_descendant;
        }
        *is_last_container = !has_float_ahead;
        found_item
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:127-128
    // cpp: layoutng_fragment_tree/fragment_items.cc:452-464
    fn TryDirtyFirstLineFor(layout_object: &LayoutObject, container: &LayoutBlockFlow) -> bool {
        debug_assert!(layout_object.IsDescendantOf(container as *const LayoutBlockFlow as *const LayoutObject));
        let mut cursor = InlineCursor::from_container(container);
        cursor.MoveToLayoutObject(layout_object);
        if cursor.IsNull() {
            return false;
        }
        debug_assert!(!cursor.Current().Item().is_null());
        debug_assert_eq!(
            layout_object as *const LayoutObject,
            cursor.Current().GetLayoutObject()
        );
        unsafe { &*cursor.Current().Item() }.SetDirty();
        true
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:129-130
    // cpp: layoutng_fragment_tree/fragment_items.cc:466-479
    fn TryDirtyLastLineFor(layout_object: &LayoutObject, container: &LayoutBlockFlow) -> bool {
        debug_assert!(layout_object.IsDescendantOf(container as *const LayoutBlockFlow as *const LayoutObject));
        let mut cursor = InlineCursor::from_container(container);
        cursor.MoveToLayoutObject(layout_object);
        if cursor.IsNull() {
            return false;
        }
        cursor.MoveToLastForSameLayoutObject();
        debug_assert!(!cursor.Current().Item().is_null());
        debug_assert_eq!(
            layout_object as *const LayoutObject,
            cursor.Current().GetLayoutObject()
        );
        unsafe { &*cursor.Current().Item() }.SetDirty();
        true
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:102-104
    // cpp: layoutng_fragment_tree/fragment_items.cc:481-523
    pub fn DirtyLinesFromChangedChild(child: &LayoutObject, container: &LayoutBlockFlow) {
        if child.IsInLayoutNGInlineFormattingContext()
            && !child.IsFloatingOrOutOfFlowPositioned()
            && Self::TryDirtyFirstLineFor(child, container)
        {
            return;
        }
        let mut current = child as *const LayoutObject;
        loop {
            let mut previous = unsafe { &*current }.PreviousSibling();
            if !previous.is_null() {
                loop {
                    let layout_inline = DynamicTo::<LayoutInline>(previous);
                    if layout_inline.is_null() {
                        break;
                    }
                    let last_child = unsafe { &*layout_inline }.LastChild();
                    if last_child.is_null() {
                        break;
                    }
                    previous = last_child;
                }
                current = previous;
                if unsafe { &*current }.IsFloatingOrOutOfFlowPositioned() {
                    continue;
                }
                if unsafe { &*current }.IsInLayoutNGInlineFormattingContext()
                    && Self::TryDirtyLastLineFor(unsafe { &*current }, container)
                {
                    return;
                }
                continue;
            }
            current = unsafe { &*current }.Parent();
            if current.is_null() || unsafe { &*current }.IsLayoutBlockFlow() {
                Self::DirtyFirstItem(container);
                return;
            }
            debug_assert!(unsafe { &*current }.IsLayoutInline());
            if unsafe { &*current }.IsInLayoutNGInlineFormattingContext()
                && Self::TryDirtyFirstLineFor(unsafe { &*current }, container)
            {
                return;
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:131
    // cpp: layoutng_fragment_tree/fragment_items.cc:525-533
    fn DirtyFirstItem(container: &LayoutBlockFlow) {
        for fragment in container.PhysicalFragments() {
            let items = fragment.Items();
            if !items.is_null() {
                unsafe { &*items }.front().SetDirty();
                return;
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:106-107
    // cpp: layoutng_fragment_tree/fragment_items.cc:535-561
    pub fn DirtyLinesFromNeedsLayout(container: &LayoutBlockFlow) {
        debug_assert!(container
            .PhysicalFragments()
            .into_iter()
            .any(|fragment| fragment.HasItems()));
        let writing_mode = container.StyleRef().GetWritingMode();
        let mut child = container.FirstChild();
        while !child.is_null() {
            if unsafe { &*child }.NeedsLayout()
                || !IsParallelWritingMode(
                    writing_mode,
                    unsafe { &*child }.StyleRef().GetWritingMode(),
                )
            {
                Self::DirtyLinesFromChangedChild(unsafe { &*child }, container);
                return;
            }
            child = unsafe { &*child }.NextSibling();
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:109-115
    // cpp: layoutng_fragment_tree/fragment_items.cc:563-576
    pub fn ReplaceBoxFragment(
        old_fragment: &PhysicalBoxFragment,
        new_fragment: &PhysicalBoxFragment,
        containing_fragment: &PhysicalBoxFragment,
    ) -> bool {
        let mut cursor = InlineCursor::from_fragment(containing_fragment);
        while cursor.IsNotNull() {
            let item = unsafe { &*cursor.Current().Item() };
            if std::ptr::eq(
                item.BoxFragment(),
                old_fragment as *const PhysicalBoxFragment,
            ) {
                item.GetMutableForCloning().ReplaceBoxFragment(new_fragment);
                return true;
            }
            cursor.MoveToNext();
        }
        false
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:84
    // cpp: layoutng_fragment_tree/fragment_items.cc:578-587
    pub fn LayoutObjectWillBeMoved(layout_object: &LayoutObject) {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(layout_object);
        while cursor.IsNotNull() {
            unsafe { &*cursor.Current().Item() }.LayoutObjectWillBeMoved();
            cursor.MoveToNextForSameLayoutObject();
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:83
    // cpp: layoutng_fragment_tree/fragment_items.cc:588-597
    pub fn LayoutObjectWillBeDestroyed(layout_object: &LayoutObject) {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(layout_object);
        while cursor.IsNotNull() {
            unsafe { &*cursor.Current().Item() }.LayoutObjectWillBeDestroyed();
            cursor.MoveToNextForSameLayoutObject();
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:117-119
    // cpp: layoutng_fragment_tree/fragment_items.cc:599-605
    #[cfg(debug_assertions)]
    pub fn CheckAllItemsAreValid(&self) {
        for item in self.Items() {
            debug_assert!(!item.IsLayoutObjectDestroyedOrMoved());
        }
    }

    // cpp: layoutng_fragment_tree/fragment_items.h:121
    // cpp: layoutng_fragment_tree/fragment_items.cc:607-609
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.items_);
    }
}
