// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: core/editing/ime/edit_context.h:181; editor_key_bindings.cc:93-97.
/// Identity-bearing capability of the active source EditContext. The IME
/// controller retains the original object; absence is not a boolean mirror.
/// The full EditContext API remains with that object's owning implementation.
pub trait ActiveEditContext {
    fn InsertText(&self, text: &[u16]) -> bool;
    // Source editor_command.cc:2163-2178. These are virtual operations of the
    // same admitted EditContext, not flags cached by Editor/Browser.
    fn DeleteBackward(&self);
    fn DeleteForward(&self);
    fn DeleteWordBackward(&self);
    fn DeleteWordForward(&self);
}
