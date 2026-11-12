//! ClipboardEvent target and Editor::CanEditRichly read the canonical visible
//! DOM selection within the same call-scoped native position owner.
use super::*;
use interaction_default::core::editing::commands::clipboard_commands::{
    self, ClipboardEventTargetContext,
};
pub(super) fn QueryClipboardSelection(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
    focused: Option<u64>,
    has_focus: bool,
) -> (Option<u64>, bool) {
    let visible = CreateVisibleDOMSelectionForFocus(document, layout, selection);
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    struct Target<'a> {
        ctx: EditingPositionContext<'a>,
        visible: Option<DOMSelection>,
        focused: Option<u64>,
        has_focus: bool,
    }
    impl ClipboardEventTargetContext for Target<'_> {
        type Node = u64;
        fn SelectionIsHidden(&self) -> bool {
            let start = self.visible.map(|s| DOMSelectionStart(&self.ctx, s));
            interaction_default::core::editing::frame_selection::SelectionIsHidden(
                self.has_focus,
                start.as_ref().and_then(|s| s.AnchorNode()).copied(),
                self.visible.is_some_and(|s| s.anchor != s.focus),
                |n| {
                    text_offset_mapping::EnclosingTextControl(
                        &self.ctx,
                        &Position::FirstPositionInNode(*n),
                    )
                    .is_some()
                },
            )
        }
        fn FocusedElement(&self) -> Option<u64> {
            self.focused
        }
        fn IsEditable(&self, n: &u64) -> bool {
            self.ctx.IsEditableNodeValue(n)
        }
        fn Body(&self) -> Option<u64> {
            self.ctx
                .document
                .BodyElement()
                .map(|i| self.ctx.document.Node(i).Id())
        }
        fn SelectionStartContainer(&self) -> Option<u64> {
            DOMSelectionStart(&self.ctx, self.visible?).ComputeContainerNode(&self.ctx)
        }
        fn IsElement(&self, n: &u64) -> bool {
            self.ctx
                .Index(n)
                .is_some_and(|i| self.ctx.document.Node(i).Type() == DOMNodeType::kElement)
        }
        fn ParentElement(&self, n: &u64) -> Option<u64> {
            let mut n = self
                .ctx
                .Index(n)
                .and_then(|i| self.ctx.document.Node(i).Parent());
            while let Some(i) = n {
                if self.ctx.document.Node(i).Type() == DOMNodeType::kElement {
                    return Some(self.ctx.document.Node(i).Id());
                }
                n = self.ctx.document.Node(i).Parent();
            }
            None
        }
        fn IsInUserAgentShadowRoot(&self, n: &u64) -> bool {
            self.ctx
                .Index(n)
                .and_then(|i| self.ctx.document.ContainingShadowRoot(i))
                .is_some_and(|r| {
                    self.ctx.document.ShadowRootMode(r)
                        == Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
                })
        }
        fn OwnerShadowHost(&self, n: &u64) -> Option<u64> {
            self.ctx
                .document
                .OwnerShadowHost(self.ctx.Index(n)?)
                .map(|i| self.ctx.document.Node(i).Id())
        }
    }
    let target = Target {
        ctx: context,
        visible,
        focused,
        has_focus,
    };
    let richly_editable = visible.and_then(|s| s.anchor.node_id).is_some_and(|n| {
        foundation::editable::IsRichlyEditable(
            &dom::persistent_document::DOMEditableFacts(document),
            n,
            Default::default(),
        )
    });
    (
        clipboard_commands::FindEventTargetForClipboardEvent(&target),
        richly_editable,
    )
}
