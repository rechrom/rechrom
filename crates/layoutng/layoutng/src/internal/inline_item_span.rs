use super::inline_item::InlineItem;
use foundation::{Member, Visitor};
use layoutng_fragment_tree::inline_items_data::InlineItemsData;

// cpp: layoutng/internal/inline_item_span.h:12-21,64-67
// Hold the GC owner, not the vector's address: the vector is owned by data_.
pub struct InlineItemSpan {
    data_: Member<InlineItemsData>,
    begin_: u32,
    size_: u32,
}

impl Default for InlineItemSpan {
    fn default() -> Self {
        Self {
            data_: Member::default(),
            begin_: 0,
            size_: 0,
        }
    }
}

#[allow(non_snake_case)]
impl InlineItemSpan {
    // cpp: layoutng/internal/inline_item_span.h:23-31
    pub fn SetItems(&mut self, data: &InlineItemsData, begin: u32, size: u32) {
        assert!((begin as usize) < data.items.len());
        // Preserve the source's check of the *previous* span before updating.
        assert!((self.begin_ as usize + self.size_ as usize) <= data.items.len());
        self.data_ = Member::from_ptr(data as *const _ as *mut _);
        self.begin_ = begin;
        self.size_ = size;
    }

    // cpp: layoutng/internal/inline_item_span.h:32-36
    pub fn Clear(&mut self) {
        self.data_ = Member::default();
        self.begin_ = 0;
        self.size_ = 0;
    }

    // cpp: layoutng/internal/inline_item_span.h:38-39
    pub fn empty(&self) -> bool {
        self.size_ == 0
    }
    pub fn size(&self) -> u32 {
        self.size_
    }

    // cpp: layoutng/internal/inline_item_span.h:41-46
    pub fn Items(&self) -> &[Member<InlineItem>] {
        if self.empty() {
            return &[];
        }
        let data = unsafe { &*self.data_.Get() };
        &data.items[self.begin_ as usize..(self.begin_ + self.size_) as usize]
    }

    // cpp: layoutng/internal/inline_item_span.h:48-53
    pub fn begin(&self) -> *const Member<InlineItem> {
        self.Items().as_ptr()
    }
    pub fn end(&self) -> *const Member<InlineItem> {
        unsafe { self.Items().as_ptr().add(self.Items().len()) }
    }

    // cpp: layoutng/internal/inline_item_span.h:55-60
    pub fn front(&self) -> &InlineItem {
        unsafe { &*self.Items().first().expect("empty inline item span").Get() }
    }

    pub fn back(&self) -> &InlineItem {
        unsafe { &*self.Items().last().expect("empty inline item span").Get() }
    }

    // cpp: layoutng/internal/inline_item_span.h:62-62
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.data_);
    }
}
