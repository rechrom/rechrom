use foundation::{AtomicString, HeapHashMap, Member, Visitor};

use super::computed_style::ComputedStyle;
use super::computed_style_constants::{IsHighlightPseudoElement, PseudoId};

// cpp: layoutng_style/style/style_highlight_data.h:24-25
pub type CustomHighlightsStyleMap = HeapHashMap<AtomicString, Member<ComputedStyle>>;

// cpp: layoutng_style/style/style_highlight_data.h:27-69
// The C++ members point to const ComputedStyle. Rust stores the same address
// in Member<ComputedStyle>; setters cast away pointer constness only for the
// GC edge, and all accessors expose *const ComputedStyle.
#[derive(Clone, Default)]
pub struct StyleHighlightData {
    selection_: Member<ComputedStyle>,
    search_text_current_: Member<ComputedStyle>,
    search_text_not_current_: Member<ComputedStyle>,
    target_text_: Member<ComputedStyle>,
    spelling_error_: Member<ComputedStyle>,
    grammar_error_: Member<ComputedStyle>,
    custom_highlights_: CustomHighlightsStyleMap,
}

#[allow(non_snake_case)]
impl StyleHighlightData {
    // cpp: layoutng_style/style/style_highlight_data.h:33-35
    // cpp: layoutng_style/style/style_highlight_data.cc:14-35
    pub fn Style(
        &self,
        pseudo_id: PseudoId,
        pseudo_argument: &AtomicString,
    ) -> *const ComputedStyle {
        debug_assert!(IsHighlightPseudoElement(pseudo_id));
        match pseudo_id {
            PseudoId::kPseudoIdSelection => self.Selection(),
            PseudoId::kPseudoIdSearchText => self.SearchTextNotCurrent(),
            PseudoId::kPseudoIdTargetText => self.TargetText(),
            PseudoId::kPseudoIdSpellingError => self.SpellingError(),
            PseudoId::kPseudoIdGrammarError => self.GrammarError(),
            PseudoId::kPseudoIdHighlight => self.CustomHighlight(pseudo_argument),
            _ => unreachable!("not a highlight pseudo-element"),
        }
    }

    // cpp: layoutng_style/style/style_highlight_data.h:35
    pub fn StyleWithoutArgument(&self, pseudo_id: PseudoId) -> *const ComputedStyle {
        self.Style(pseudo_id, &AtomicString::default())
    }

    // cpp: layoutng_style/style/style_highlight_data.h:36
    // cpp: layoutng_style/style/style_highlight_data.cc:37-39
    pub fn Selection(&self) -> *const ComputedStyle {
        self.selection_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:37
    // cpp: layoutng_style/style/style_highlight_data.cc:41-43
    pub fn SearchTextCurrent(&self) -> *const ComputedStyle {
        self.search_text_current_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:38
    // cpp: layoutng_style/style/style_highlight_data.cc:45-47
    pub fn SearchTextNotCurrent(&self) -> *const ComputedStyle {
        self.search_text_not_current_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:39
    // cpp: layoutng_style/style/style_highlight_data.cc:49-51
    pub fn TargetText(&self) -> *const ComputedStyle {
        self.target_text_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:40
    // cpp: layoutng_style/style/style_highlight_data.cc:53-55
    pub fn SpellingError(&self) -> *const ComputedStyle {
        self.spelling_error_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:41
    // cpp: layoutng_style/style/style_highlight_data.cc:57-59
    pub fn GrammarError(&self) -> *const ComputedStyle {
        self.grammar_error_.Get()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:42
    // cpp: layoutng_style/style/style_highlight_data.cc:61-71
    pub fn CustomHighlight(&self, highlight_name: &AtomicString) -> *const ComputedStyle {
        if !highlight_name.IsNull() {
            if let Some(value) = self.custom_highlights_.get(highlight_name) {
                let style = value.Get();
                assert!(!style.is_null());
                return style;
            }
        }
        std::ptr::null()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:43-45
    pub fn CustomHighlights(&self) -> &CustomHighlightsStyleMap {
        &self.custom_highlights_
    }

    // cpp: layoutng_style/style/style_highlight_data.h:46
    // cpp: layoutng_style/style/style_highlight_data.cc:73-75
    pub fn SetSelection(&mut self, style: *const ComputedStyle) {
        self.selection_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:47
    // cpp: layoutng_style/style/style_highlight_data.cc:77-79
    pub fn SetSearchTextCurrent(&mut self, style: *const ComputedStyle) {
        self.search_text_current_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:48
    // cpp: layoutng_style/style/style_highlight_data.cc:81-83
    pub fn SetSearchTextNotCurrent(&mut self, style: *const ComputedStyle) {
        self.search_text_not_current_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:49
    // cpp: layoutng_style/style/style_highlight_data.cc:85-87
    pub fn SetTargetText(&mut self, style: *const ComputedStyle) {
        self.target_text_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:50
    // cpp: layoutng_style/style/style_highlight_data.cc:89-91
    pub fn SetSpellingError(&mut self, style: *const ComputedStyle) {
        self.spelling_error_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:51
    // cpp: layoutng_style/style/style_highlight_data.cc:93-95
    pub fn SetGrammarError(&mut self, style: *const ComputedStyle) {
        self.grammar_error_ = Member::from_ptr(style.cast_mut());
    }

    // cpp: layoutng_style/style/style_highlight_data.h:52
    // cpp: layoutng_style/style/style_highlight_data.cc:97-105
    pub fn SetCustomHighlight(
        &mut self,
        highlight_name: &AtomicString,
        style: *const ComputedStyle,
    ) {
        debug_assert!(!highlight_name.IsNull());
        if !style.is_null() {
            self.custom_highlights_
                .Set(highlight_name.clone(), Member::from_ptr(style.cast_mut()));
        } else {
            self.custom_highlights_.erase(highlight_name);
        }
    }

    // cpp: layoutng_style/style/style_highlight_data.h:54-55
    // No definition exists in the supplied C++ tree.
    pub fn StylesDependOnFunc(&self, func: &dyn Fn(&ComputedStyle) -> bool) -> bool {
        unsafe { StyleHighlightDataStylesDependOnFunc(self, func) }
    }

    // cpp: layoutng_style/style/style_highlight_data.h:57
    // cpp: layoutng_style/style/style_highlight_data.cc:107-125
    pub fn DependsOnSizeContainerQueries(&self) -> bool {
        if Self::style_depends_on_size(&self.selection_)
            || Self::style_depends_on_size(&self.target_text_)
            || Self::style_depends_on_size(&self.spelling_error_)
            || Self::style_depends_on_size(&self.grammar_error_)
        {
            return true;
        }
        for (_, style) in self.custom_highlights_.iter() {
            let style = unsafe { &*style.Get() };
            if style.DependsOnSizeContainerQueries() || style.HasContainerRelativeValue() {
                return true;
            }
        }
        false
    }

    fn style_depends_on_size(style: &Member<ComputedStyle>) -> bool {
        let style = style.Get();
        if style.is_null() {
            return false;
        }
        let style = unsafe { &*style };
        style.DependsOnSizeContainerQueries() || style.HasContainerRelativeValue()
    }

    // cpp: layoutng_style/style/style_highlight_data.h:59
    // cpp: layoutng_style/style/style_highlight_data.cc:127-135
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.selection_);
        visitor.Trace(&self.search_text_current_);
        visitor.Trace(&self.search_text_not_current_);
        visitor.Trace(&self.target_text_);
        visitor.Trace(&self.spelling_error_);
        visitor.Trace(&self.grammar_error_);
        visitor.Trace(&self.custom_highlights_);
    }
}

// cpp: layoutng_style/style/style_highlight_data.h:31
// No definition exists in the supplied C++ tree.
impl PartialEq for StyleHighlightData {
    fn eq(&self, other: &Self) -> bool {
        unsafe { StyleHighlightDataEqual(self, other) }
    }
}

unsafe extern "Rust" {
    fn StyleHighlightDataEqual(value: &StyleHighlightData, other: &StyleHighlightData) -> bool;
    fn StyleHighlightDataStylesDependOnFunc(
        value: &StyleHighlightData,
        func: &dyn Fn(&ComputedStyle) -> bool,
    ) -> bool;
}
