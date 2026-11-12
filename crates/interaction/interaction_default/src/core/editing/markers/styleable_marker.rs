// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401f: styleable_marker.{h,cc}:18-83;
// suggestion_marker.{h,cc}:20-94; suggestion_marker_properties.h:20-112.
use foundation::Color;
use std::{
    cell::RefCell,
    sync::atomic::{AtomicI32, Ordering},
    sync::Arc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeTextSpanThickness {
    None,
    Thin,
    Thick,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeTextSpanUnderlineStyle {
    None,
    Solid,
    Dot,
    Dash,
    Squiggle,
}
#[derive(Clone, Debug)]
pub struct StyleableMarker {
    pub underline_color: Color,
    pub thickness: ImeTextSpanThickness,
    pub underline_style: ImeTextSpanUnderlineStyle,
    pub text_color: Color,
    pub background_color: Color,
}
impl Default for StyleableMarker {
    fn default() -> Self {
        Self {
            underline_color: Color::kTransparent,
            thickness: ImeTextSpanThickness::Thin,
            underline_style: ImeTextSpanUnderlineStyle::Solid,
            text_color: Color::kTransparent,
            background_color: Color::kTransparent,
        }
    }
}
impl StyleableMarker {
    pub fn UseTextColor(&self) -> bool {
        self.thickness != ImeTextSpanThickness::None && self.underline_color == Color::kTransparent
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionType {
    Misspelling,
    NotMisspelling,
    Autocorrect,
    Grammar,
}
#[derive(Clone, Debug)]
pub struct SuggestionMarkerProperties {
    pub style: StyleableMarker,
    pub kind: SuggestionType,
    pub remove_on_finish_composing: bool,
    pub suggestions: Vec<Arc<[u16]>>,
    pub highlight_color: Color,
    pub hide_suggestion_menu: bool,
}
impl Default for SuggestionMarkerProperties {
    fn default() -> Self {
        Self {
            style: Default::default(),
            kind: SuggestionType::NotMisspelling,
            remove_on_finish_composing: false,
            suggestions: Vec::new(),
            highlight_color: Color::kTransparent,
            hide_suggestion_menu: false,
        }
    }
}
pub struct SuggestionMarker {
    tag: i32,
    kind: SuggestionType,
    remove_on_finish_composing: bool,
    highlight_color: Color,
    hide_suggestion_menu: bool,
    suggestions: RefCell<Vec<Arc<[u16]>>>,
}
impl SuggestionMarker {
    pub fn new(properties: &SuggestionMarkerProperties) -> Self {
        static NEXT_TAG: AtomicI32 = AtomicI32::new(0);
        let tag = NEXT_TAG
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("suggestion marker tag overflow")
            + 1;
        Self {
            tag,
            kind: properties.kind,
            remove_on_finish_composing: properties.remove_on_finish_composing,
            highlight_color: properties.highlight_color,
            hide_suggestion_menu: properties.hide_suggestion_menu,
            suggestions: RefCell::new(properties.suggestions.clone()),
        }
    }
    pub fn Tag(&self) -> i32 {
        self.tag
    }
    pub fn GetSuggestionType(&self) -> SuggestionType {
        self.kind
    }
    pub fn Suggestions(&self) -> std::cell::Ref<'_, [Arc<[u16]>]> {
        std::cell::Ref::map(self.suggestions.borrow(), |s| s.as_slice())
    }
    pub fn SetSuggestion(&self, index: usize, text: Arc<[u16]>) {
        self.suggestions.borrow_mut()[index] = text;
    }
    pub fn IsMisspelling(&self) -> bool {
        self.kind == SuggestionType::Misspelling
    }
    pub fn IsGrammarError(&self) -> bool {
        self.kind == SuggestionType::Grammar
    }
    pub fn NeedsRemovalOnFinishComposing(&self) -> bool {
        self.remove_on_finish_composing
    }
    pub fn SuggestionHighlightColor(&self) -> Color {
        self.highlight_color
    }
    pub fn ShouldHideSuggestionMenu(&self) -> bool {
        self.hide_suggestion_menu
    }
}
// Same source scope owner is shared with the DOM neutral mutation-fact port.
pub use foundation::editing_marker_scope::SuggestionMarkerReplacementScope;
