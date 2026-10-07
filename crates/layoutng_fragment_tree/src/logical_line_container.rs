use font_engine::FontHeight;
use foundation::{HeapVector, LayoutUnit, MakeGarbageCollected, Member, Visitor};

use crate::logical_line_item::LogicalLineItems;

// C++ stores annotation values in a GC heap vector and disables separate
// allocation of each value. Rust keeps the same owned-value relationship.
// cpp: layoutng_fragment_tree/logical_line_container.h:25-30
pub struct AnnotationLine {
    pub metrics: FontHeight,
    pub line_items: Member<LogicalLineItems>,
}

#[allow(non_snake_case)]
impl AnnotationLine {
    // cpp: layoutng_fragment_tree/logical_line_container.h:32-33
    pub fn new(metrics: FontHeight, line_items: &LogicalLineItems) -> Self {
        Self {
            metrics,
            line_items: Member::from_ptr(line_items as *const _ as *mut _),
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:34
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.line_items);
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:35
    pub fn get(&self) -> *mut LogicalLineItems {
        self.line_items.Get()
    }
}

// cpp: layoutng_fragment_tree/logical_line_container.h:19
// cpp: layoutng_fragment_tree/logical_line_container.h:55-59
pub struct LogicalLineContainer {
    base_line_: Member<LogicalLineItems>,
    annotation_line_list_: HeapVector<AnnotationLine>,
    text_fit_scale_: f32,
}

impl Default for LogicalLineContainer {
    // cpp: layoutng_fragment_tree/logical_line_container.h:21
    // cpp: layoutng_fragment_tree/logical_line_container.cc:11-12
    fn default() -> Self {
        Self {
            base_line_: Member::from_ptr(MakeGarbageCollected(LogicalLineItems::default())),
            annotation_line_list_: HeapVector::default(),
            text_fit_scale_: 1.0,
        }
    }
}

#[allow(non_snake_case)]
impl LogicalLineContainer {
    // C++ returns a mutable reference even through a const container.
    // Rust exposes the GC pointer, leaving mutable access explicit.
    // cpp: layoutng_fragment_tree/logical_line_container.h:24
    pub fn BaseLine(&self) -> *mut LogicalLineItems {
        self.base_line_.Get()
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:37-39
    pub fn AnnotationLineList(&self) -> &HeapVector<AnnotationLine> {
        &self.annotation_line_list_
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:42-44
    pub fn AddAnnotation(&mut self, metrics: FontHeight, line_items: &LogicalLineItems) {
        self.annotation_line_list_
            .push(AnnotationLine::new(metrics, line_items));
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:46
    pub fn SetTextFitScale(&mut self, scale: f32) {
        self.text_fit_scale_ = scale;
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:47
    pub fn TextFitScale(&self) -> f32 {
        self.text_fit_scale_
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:22
    // cpp: layoutng_fragment_tree/logical_line_container.cc:14-17
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.base_line_);
        visitor.Trace(&self.annotation_line_list_);
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:50
    // cpp: layoutng_fragment_tree/logical_line_container.cc:19-26
    pub fn Clear(&mut self) {
        unsafe { &mut *self.base_line_.Get() }.clear();
        for line in &mut self.annotation_line_list_ {
            unsafe { &mut *line.get() }.clear();
        }
        self.annotation_line_list_.clear();
        self.text_fit_scale_ = 1.0;
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:52
    // cpp: layoutng_fragment_tree/logical_line_container.cc:28-35
    pub fn Shrink(&mut self) {
        unsafe { &mut *self.base_line_.Get() }.Shrink(0);
        for line in &mut self.annotation_line_list_ {
            unsafe { &mut *line.get() }.clear();
        }
        self.annotation_line_list_.Shrink(0);
        self.text_fit_scale_ = 1.0;
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:53
    // cpp: layoutng_fragment_tree/logical_line_container.cc:37-42
    pub fn MoveInBlockDirection(&mut self, delta: LayoutUnit) {
        unsafe { &mut *self.base_line_.Get() }.MoveInBlockDirection(delta);
        for line in &mut self.annotation_line_list_ {
            unsafe { &mut *line.get() }.MoveInBlockDirection(delta);
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_container.h:40
    // cpp: layoutng_fragment_tree/logical_line_container.cc:44-50
    pub fn EstimatedFragmentItemCount(&self) -> u32 {
        let mut count = 1 + unsafe { &*self.base_line_.Get() }.size();
        for line in &self.annotation_line_list_ {
            count += 1 + unsafe { &*line.get() }.size();
        }
        count
    }
}

// cpp: layoutng_fragment_tree/logical_line_container.h:63-64
pub struct AnnotationLineVectorTraits;

#[allow(non_upper_case_globals)]
impl AnnotationLineVectorTraits {
    pub const kCanClearUnusedSlotsWithMemset: bool = true;
}
