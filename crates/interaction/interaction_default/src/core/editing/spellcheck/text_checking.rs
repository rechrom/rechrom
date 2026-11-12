// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source core/editing/spellcheck/text_checking.h:42-64;
// public/web/web_spelling_marker.h:18-34, web_text_check_client.h.
use std::rc::Rc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecorationType {
    Spelling,
    Grammar,
}
#[derive(Clone, Debug)]
pub struct GrammarDetail {
    pub location: i32,
    pub length: i32,
    pub guesses: Vec<Rc<[u16]>>,
    pub user_description: Rc<[u16]>,
}
#[derive(Clone, Debug)]
pub struct TextCheckingResult {
    pub decoration: TextDecorationType,
    pub location: i32,
    pub length: i32,
    pub details: Vec<GrammarDetail>,
    pub replacements: Vec<Rc<[u16]>>,
    pub should_hide_suggestion_menu: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct WebSpellingMarker {
    pub start: u32,
    pub end: u32,
    pub marker_type: TextDecorationType,
}
pub trait TextCheckingCompletion {
    fn DidFinishCheckingText(&self, results: Vec<TextCheckingResult>);
    fn DidCancelCheckingText(&self);
}
