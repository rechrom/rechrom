// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/ime/ime_text_span.h:40-120
// Source: third_party/blink/renderer/core/editing/ime/ime_text_span.cc:32-68
use crate::core::editing::markers::styleable_marker::StyleableMarker;
use foundation::Color;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kComposition,
    kSuggestion,
    kMisspellingSuggestion,
    kAutocorrect,
    kGrammarSuggestion,
    kPreviewStylusGesture,
}

#[derive(Clone, Debug)]
pub struct ImeTextSpan {
    pub kind: Type,
    start_offset: usize,
    end_offset: usize,
    pub style: StyleableMarker,
    pub suggestion_highlight_color: Color,
    pub remove_on_finish_composing: bool,
    pub interim_char_selection: bool,
    // Host IME values cross the input queue before the document sequence owns
    // them. Retain immutable UTF-16 owners through marker publication.
    pub suggestions: Vec<Arc<[u16]>>,
    pub should_hide_suggestion_menu: bool,
}
impl ImeTextSpan {
    pub fn new(kind: Type, start: usize, end: usize, style: StyleableMarker) -> Self {
        let start_offset = start.min(usize::MAX - 1);
        Self {
            kind,
            start_offset,
            end_offset: end.max(start_offset + 1),
            style,
            suggestion_highlight_color: Color::kTransparent,
            remove_on_finish_composing: false,
            interim_char_selection: false,
            suggestions: Vec::new(),
            should_hide_suggestion_menu: false,
        }
    }
    pub fn StartOffset(&self) -> usize {
        self.start_offset
    }
    pub fn EndOffset(&self) -> usize {
        self.end_offset
    }
}
