//! Chromium frame_selection.cc:1049-1130. Root selection and publication stay
//! with FrameSelection; native facts are borrowed only for each synchronous call.
use super::{
    frame_selection::{FrameSelection, SelectionValue},
    set_selection_options::{SetSelectionBy, SetSelectionOptions},
};
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;

pub struct SelectAllFacts {
    pub visible_selection_is_none: bool,
    pub hidden: bool,
    pub editable_root: Option<u64>,
    pub editable_root_is_empty: bool,
    pub shadow_root: Option<u64>,
    pub shadow_host: Option<u64>,
    pub document_element: Option<u64>,
    pub body: Option<u64>,
}
/// editor_command.cc:1320-1364 (native/menu source).
pub fn EnabledSelectAll(facts: &SelectAllFacts) -> bool {
    EnabledSelectAllForSource(facts, SetSelectionBy::kUser)
}
pub fn EnabledSelectAllForSource(facts: &SelectAllFacts, by: SetSelectionBy) -> bool {
    facts.visible_selection_is_none
        || (by == SetSelectionBy::kUser && facts.hidden)
        || !facts.editable_root_is_empty
}
pub trait FrameSelectionSelectAllContext<S: SelectionValue> {
    fn SelectFocusedControl(&self) -> Result<bool, &'static str>;
    fn Facts(&self) -> Option<SelectAllFacts>;
    fn EditingIgnoresContent(&self, root: u64) -> bool;
    fn DispatchSelectStart(&self, target: u64) -> DispatchEventResult;
    fn IsConnected(&self, root: u64) -> bool;
    fn SelectAllChildren(&self, root: u64, canonicalize: bool) -> S;
    fn ShowSelectionContextMenu(&self) -> Result<(), &'static str>;
}
impl<S: SelectionValue> FrameSelection<S> {
    pub fn SelectAll<C: FrameSelectionSelectAllContext<S>>(
        &self,
        context: &C,
        by: SetSelectionBy,
        canonicalize: bool,
    ) -> Result<(), &'static str> {
        if context.SelectFocusedControl()? {
            return Ok(());
        }
        let Some(facts) = context.Facts() else {
            return Ok(());
        };
        let (root, target) = if by == SetSelectionBy::kUser && facts.hidden {
            (facts.document_element, facts.body)
        } else if let Some(root) = facts.editable_root {
            (Some(root), facts.shadow_host.or(Some(root)))
        } else if let Some(root) = facts.shadow_root {
            (Some(root), facts.shadow_host)
        } else {
            (facts.document_element, facts.body)
        };
        let Some(root) = root.filter(|root| !context.EditingIgnoresContent(*root)) else {
            return Ok(());
        };
        if let Some(target) = target {
            let document = self.owner.DocumentIdentity();
            if context.DispatchSelectStart(target) != DispatchEventResult::kNotCanceled
                || !self.owner.IsAvailable()
                || document != self.owner.DocumentIdentity()
                || !context.IsConnected(root)
            {
                return Ok(());
            }
        }
        self.SetSelection(
            &context.SelectAllChildren(root, canonicalize),
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                should_show_handle: self.is_handle_visible.get(),
                ..Default::default()
            },
        );
        self.effects.SelectFrameElementInParentIfFullySelected();
        self.effects
            .NotifyTextControlOfSelectionChange(SetSelectionBy::kUser);
        if self.is_handle_visible.get() {
            context.ShowSelectionContextMenu()?;
        }
        Ok(())
    }
}
