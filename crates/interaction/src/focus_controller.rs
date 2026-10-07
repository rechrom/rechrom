use crate::{
    event::{EventType, MakeSyntheticEvent},
    ownership::{InteractionDocument, ReadNode},
    text_editor::{SyntheticEventDispatcher, TextEditor},
};
use dom::UserInteractionState;
use std::{cell::RefCell, rc::Rc};
// cpp: interaction/focus_controller.h:17-29
// cpp: interaction/focus_controller.cc:9-13
#[derive(Clone)]
pub struct FocusController<'a> {
    document: InteractionDocument,
    state: Rc<RefCell<UserInteractionState>>,
    editor: Rc<TextEditor>,
    dispatch: SyntheticEventDispatcher<'a>,
}
impl<'a> FocusController<'a> {
    pub fn new(
        document: InteractionDocument,
        state: Rc<RefCell<UserInteractionState>>,
        editor: Rc<TextEditor>,
        dispatch: SyntheticEventDispatcher<'a>,
    ) -> Self {
        Self {
            document,
            state,
            editor,
            dispatch,
        }
    }
    // cpp: interaction/focus_controller.cc:15-44
    pub fn Focus(&self, id: u64, focus_visible: bool) {
        // Blink Element::IsFocusableStyle rejects display:none ancestry and
        // invisible controls. Ignore an ineligible focus request before blur.
        let eligible = ReadNode(&self.document, id, |document, node, index| {
            if node.FindAttribute("disabled").is_some()
                || (node.IsHTMLElement("input")
                    && node
                        .FindAttribute("type")
                        .is_some_and(|a| a.value.eq_ignore_ascii_case("hidden")))
            {
                return false;
            }
            if document
                .ResolvedStyleFor(index)
                .is_some_and(|style| !style.style.paint.visible)
            {
                return false;
            }
            let mut ancestor = Some(index);
            while let Some(i) = ancestor {
                if document
                    .ResolvedStyleFor(i)
                    .is_some_and(|style| !style.own_generates_box && !style.own_display_contents)
                {
                    return false;
                }
                ancestor = document.Node(i).Parent();
            }
            true
        })
        .unwrap_or(false);
        if !eligible {
            return;
        }
        {
            let mut state = self.state.borrow_mut();
            if state.focused_node_id == Some(id) {
                state.focus_visible_node_id = if focus_visible { Some(id) } else { None };
                return;
            }
        }
        let old_id = {
            let mut state = self.state.borrow_mut();
            let old = state.focused_node_id;
            state.focused_node_id = None;
            state.focus_visible_node_id = None;
            old
        };
        if let Some(old) = old_id.filter(|&id| ReadNode(&self.document, id, |_, _, _| ()).is_some())
        {
            let mut blur = MakeSyntheticEvent(EventType::kBlur, old);
            blur.related_target_node_id = Some(id);
            (self.dispatch)(&mut blur, old);
            let mut focusout = MakeSyntheticEvent(EventType::kFocusOut, old);
            focusout.related_target_node_id = Some(id);
            (self.dispatch)(&mut focusout, old);
        }
        {
            let mut state = self.state.borrow_mut();
            state.focused_node_id = Some(id);
            state.focus_visible_node_id = if focus_visible { Some(id) } else { None };
        }
        self.editor.Focus(&self.document, id);
        let mut focus = MakeSyntheticEvent(EventType::kFocus, id);
        focus.related_target_node_id = old_id;
        (self.dispatch)(&mut focus, id);
        let mut focusin = MakeSyntheticEvent(EventType::kFocusIn, id);
        focusin.related_target_node_id = old_id;
        (self.dispatch)(&mut focusin, id);
    }
    // cpp: interaction/focus_controller.cc:46-56
    pub fn Blur(&self, id: u64) {
        {
            let mut state = self.state.borrow_mut();
            if state.focused_node_id != Some(id) {
                return;
            }
            state.focused_node_id = None;
            state.focus_visible_node_id = None;
        }
        let mut blur = MakeSyntheticEvent(EventType::kBlur, id);
        (self.dispatch)(&mut blur, id);
        let mut focusout = MakeSyntheticEvent(EventType::kFocusOut, id);
        (self.dispatch)(&mut focusout, id);
    }
}
