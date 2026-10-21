// C++: layoutng_inline/layout_text_fragment.h and layout_text_fragment_layout.cc.
// The few declarations with no source definition retain typed unresolved providers.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::graphics_types::graphics::dom_node_id::DOMNodeId;
use foundation::{
    DynamicTo, MakeGarbageCollected, Member, PhysicalOffset, String, Traceable, UChar, Visitor,
};
use layoutng::internal::editing::forward::Position;
use layoutng::internal::layout_block::LayoutBlock;
use layoutng::internal::layout_node_metadata::{Element, Node, Text};
use layoutng::internal::layout_object::{HitTestResult, LayoutObject};
use layoutng::internal::layout_text::LayoutText;

unsafe extern "Rust" {
    // These declarations have no definitions in the supplied source checkout.
    // cpp: layoutng_inline/layout_text_fragment.h:54-57,119-124
    fn LayoutTextFragmentPositionForCaretOffset(this: &LayoutTextFragment, offset: u32)
        -> Position;
    fn LayoutTextFragmentCaretOffsetForPosition(
        this: &LayoutTextFragment,
        position: &Position,
    ) -> Option<u32>;
    fn LayoutTextFragmentBlockForAccompanyingFirstLetter(
        this: &LayoutTextFragment,
    ) -> *mut LayoutBlock;
    fn LayoutTextFragmentUpdateHitTestResult(
        this: &LayoutTextFragment,
        result: &mut HitTestResult,
        offset: &PhysicalOffset,
    );
    fn LayoutTextFragmentOwnerNodeId(
        this: &LayoutTextFragment,
        is_internal_content: bool,
    ) -> DOMNodeId;
    fn LayoutTextInsertedIntoTree(this: &mut LayoutText);
}

// String::DeprecatedSubstring uses UTF-16 code-unit offsets and preserves
// the distinction between a null string and an allocated empty string.
fn deprecated_substring(text: &String, start: u32, length: u32) -> String {
    if text.IsNull() {
        return String::new();
    }
    let units = text.Span16().expect("non-null string has code units");
    let start = (start as usize).min(units.len());
    let end = start.saturating_add(length as usize).min(units.len());
    String::from_utf16(&units[start..end])
}

// cpp: layoutng_inline/layout_text_fragment.h:38-41,126-132
// The public LayoutText base remains at offset zero, as in the other derived
// inline layout objects. The five derived fields retain source declaration order.
#[repr(C)]
pub struct LayoutTextFragment {
    text_: LayoutText,
    start_: u32,
    fragment_length_: u32,
    is_remaining_text_layout_object_: bool,
    content_string_: String,
    first_letter_pseudo_element_: Member<Element>,
}

const _: () = assert!(std::mem::offset_of!(LayoutTextFragment, text_) == 0);

impl Deref for LayoutTextFragment {
    type Target = LayoutText;
    fn deref(&self) -> &LayoutText {
        &self.text_
    }
}
impl DerefMut for LayoutTextFragment {
    fn deref_mut(&mut self) -> &mut LayoutText {
        &mut self.text_
    }
}

impl LayoutTextFragment {
    // cpp: layoutng_inline/layout_text_fragment.h:42-47
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:17-29
    pub fn new(node: *mut Node, text: &String, start_offset: i32, length: i32) -> Self {
        let fragment_text = deprecated_substring(text, start_offset as u32, length as u32);
        let mut layout_text = LayoutText::new(node, fragment_text);
        layout_text.SetIsTextFragmentForDerived();
        Self {
            text_: layout_text,
            start_: start_offset as u32,
            fragment_length_: length as u32,
            is_remaining_text_layout_object_: false,
            content_string_: text.clone(),
            first_letter_pseudo_element_: Member::default(),
        }
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:35-41
    pub fn Create(node: *mut Node, text: &String, start_offset: i32, length: i32) -> *mut Self {
        MakeGarbageCollected(Self::new(node, text, start_offset, length))
    }
    // cpp: layoutng_inline/layout_text_fragment.h:54-57
    pub fn PositionForCaretOffset(&self, offset: u32) -> Position {
        unsafe { LayoutTextFragmentPositionForCaretOffset(self, offset) }
    }
    pub fn CaretOffsetForPosition(&self, position: &Position) -> Option<u32> {
        unsafe { LayoutTextFragmentCaretOffsetForPosition(self, position) }
    }
    // cpp: layoutng_inline/layout_text_fragment.h:59-74
    pub fn Start(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.start_
    }
    pub fn FragmentLength(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.fragment_length_
    }
    pub fn TextStartOffset(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.Start()
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:61-65
    pub fn SetContentString(&mut self, content: &String) {
        self.CheckIsNotDestroyed();
        self.content_string_ = content.clone();
        self.text_.SetTextIfNeeded(content.clone());
    }
    pub fn ContentString(&self) -> &String {
        self.CheckIsNotDestroyed();
        &self.content_string_
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:54-59
    pub fn CompleteText(&self) -> String {
        self.CheckIsNotDestroyed();
        let text = self.AssociatedTextNode();
        if !text.is_null() {
            return unsafe { &*text }.data().clone();
        }
        self.ContentString().clone()
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:67-72
    pub fn OriginalText(&self) -> String {
        self.CheckIsNotDestroyed();
        let complete = self.CompleteText();
        deprecated_substring(&complete, self.Start(), self.FragmentLength())
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:81-91
    pub fn SetTextFragment(&mut self, text: String, start: u32, length: u32) {
        self.CheckIsNotDestroyed();
        if self.TransformedText() != &text {
            self.text_.SetTextInternal(text);
            self.text_.TextDidChange();
        }
        self.start_ = start;
        self.fragment_length_ = length;
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:93-99
    pub fn TransformAndSecureOriginalText(&mut self) {
        self.CheckIsNotDestroyed();
        let text = self.OriginalText();
        if !text.IsNull() {
            self.text_.SetTextInternal(text);
            self.text_.TextDidChange();
        }
    }
    // cpp: layoutng_inline/layout_text_fragment.h:85-105
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutTextFragment"
    }
    pub fn SetFirstLetterPseudoElement(&mut self, element: *mut Element) {
        self.CheckIsNotDestroyed();
        self.first_letter_pseudo_element_ = Member::from_ptr(element);
    }
    pub fn GetFirstLetterPseudoElement(&self) -> *mut Element {
        self.CheckIsNotDestroyed();
        self.first_letter_pseudo_element_.Get()
    }
    pub fn SetIsRemainingTextLayoutObject(&mut self, remaining: bool) {
        self.CheckIsNotDestroyed();
        self.is_remaining_text_layout_object_ = remaining;
    }
    pub fn IsRemainingTextLayoutObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.is_remaining_text_layout_object_
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:111-114
    pub fn AssociatedTextNode(&self) -> *mut Text {
        self.CheckIsNotDestroyed();
        DynamicTo::<Text>(self.GetNode())
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:116-127
    pub fn GetFirstLetterPart(&self) -> *mut LayoutText {
        self.CheckIsNotDestroyed();
        if !self.is_remaining_text_layout_object_
            || self.first_letter_pseudo_element_.Get().is_null()
        {
            return std::ptr::null_mut();
        }
        let container = unsafe { &*self.first_letter_pseudo_element_.Get() }.GetLayoutObject();
        if container.is_null() {
            return std::ptr::null_mut();
        }
        let mut child = unsafe { &*container }.SlowFirstChild();
        while !child.is_null() && !unsafe { &*child }.IsText() {
            child = unsafe { &*child }.NextSibling();
        }
        DynamicTo::<LayoutText>(child)
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:129-132
    pub fn PlainText(&self) -> String {
        self.CheckIsNotDestroyed();
        self.text_.PlainText()
    }
    // cpp: layoutng_inline/layout_text_fragment.h:113-117
    pub fn InsertedIntoTree(&mut self) {
        self.CheckIsNotDestroyed();
        self.text_.InvalidateInlineItems();
        unsafe { LayoutTextInsertedIntoTree(&mut self.text_) }
    }
    // cpp: layoutng_inline/layout_text_fragment.h:119-124
    pub fn BlockForAccompanyingFirstLetter(&self) -> *mut LayoutBlock {
        unsafe { LayoutTextFragmentBlockForAccompanyingFirstLetter(self) }
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:101-109
    pub fn PreviousCharacter(&self) -> UChar {
        self.CheckIsNotDestroyed();
        if self.Start() != 0 {
            let complete = self.CompleteText();
            if !complete.IsNull() && self.Start() <= complete.length() {
                return complete.Span16().expect("non-null string")[self.Start() as usize - 1];
            }
        }
        self.text_.PreviousCharacter()
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:74-79
    pub fn TextDidChange(&mut self) {
        self.CheckIsNotDestroyed();
        self.text_.TextDidChange();
        self.start_ = 0;
        self.fragment_length_ = self.TransformedTextLength();
    }
    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutTextFragmentUpdateHitTestResult(self, result, offset) }
    }
    pub fn OwnerNodeId(&self, is_internal_content: bool) -> DOMNodeId {
        unsafe { LayoutTextFragmentOwnerNodeId(self, is_internal_content) }
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:48-52
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        self.first_letter_pseudo_element_ = Member::default();
        self.text_.WillBeDestroyed();
    }
    // cpp: layoutng_inline/layout_text_fragment.h:137-146
    pub fn AllowFromObject(object: &LayoutObject) -> bool {
        object.IsText() && unsafe { &*(object as *const _ as *const LayoutText) }.IsTextFragment()
    }
    pub fn AllowFromText(text: &LayoutText) -> bool {
        text.IsTextFragment()
    }
}

impl Drop for LayoutTextFragment {
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:31-33
    fn drop(&mut self) {
        debug_assert!(self.first_letter_pseudo_element_.Get().is_null());
    }
}

impl Traceable for LayoutTextFragment {
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:43-46
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.first_letter_pseudo_element_);
        self.text_.Trace(visitor);
    }
}

// cpp: layoutng_inline/layout_text_fragment.h:136-146
impl foundation::DowncastFrom<LayoutObject> for LayoutTextFragment {
    fn AllowFrom(object: &LayoutObject) -> bool {
        Self::AllowFromObject(object)
    }
}

impl foundation::DowncastFrom<LayoutText> for LayoutTextFragment {
    fn AllowFrom(text: &LayoutText) -> bool {
        Self::AllowFromText(text)
    }
}
