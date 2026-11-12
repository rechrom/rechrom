//! DOM ownership adapter for InputEvent's source StaticRange -> Range ->
//! StaticRange lifecycle. Geometry remains in the resident editing query.
use super::{EditorCommandError, EventDispatcher};
use crate::ownership::InteractionDocument;
use dom::{DOMRangeHandle, DOMStaticRange};
use interaction_dispatch::core::events::input_event_range::{
    LiveRange, LiveRangeRef, RangeConversionError, RangeExceptionState, StaticRange,
    StaticRangeRef, StaticRangeSnapshot,
};
use std::rc::Rc;

struct DOMInputEventLiveRange {
    document: InteractionDocument,
    identity: u128,
    range: Option<DOMRangeHandle>,
}
struct DOMInputEventStaticRange {
    document: InteractionDocument,
    identity: u128,
    snapshot: Option<DOMStaticRange>,
}
impl StaticRange for DOMInputEventStaticRange {
    fn Snapshot(&self) -> Option<StaticRangeSnapshot> {
        let snapshot = self.snapshot?;
        // Numeric node IDs may be reused after navigation. A stale event must
        // never project the new Document's nodes into its original ranges.
        if self.document.borrow().GetDocument().RootHandle().Identity() != self.identity {
            return None;
        }
        Some(StaticRangeSnapshot {
            document: self.identity,
            start_container: snapshot.start.node_id,
            start_offset: snapshot.start.offset,
            end_container: snapshot.end.node_id,
            end_offset: snapshot.end.offset,
        })
    }
    fn ToRange(&self, exception_state: &mut RangeExceptionState) -> LiveRangeRef {
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        let Some(snapshot) = self
            .snapshot
            .filter(|_| document.RootHandle().Identity() == self.identity)
        else {
            exception_state.Throw(RangeConversionError {
                message: "range belongs to a detached document".into(),
            });
            // Explicit failed conversion retains no boundary points. A foreign
            // Document must never supply default/reused nodes for this Range.
            return Rc::new(DOMInputEventLiveRange {
                document: self.document.clone(),
                identity: self.identity,
                range: None,
            });
        };
        // StaticRange::toRange creates a new live Range then performs setStart
        // and setEnd. The source still returns that owner after offset errors.
        let root = dom::DOMBoundaryPoint {
            node_id: document.RootHandle().Id(),
            offset: 0,
        };
        let range = document
            .CreateRange(root, root)
            .expect("Document root is a valid Range boundary");
        for result in [
            document.SetRangeStart(&range, snapshot.start),
            document.SetRangeEnd(&range, snapshot.end),
        ] {
            if let Err(error) = result {
                exception_state.Throw(RangeConversionError {
                    message: dom::error::DOMException::from(error).message.into(),
                });
            }
        }
        Rc::new(DOMInputEventLiveRange {
            document: self.document.clone(),
            identity: self.identity,
            range: Some(range),
        })
    }
}
impl LiveRange for DOMInputEventLiveRange {
    fn CreateStaticRange(&self) -> StaticRangeRef {
        let snapshot = self
            .range
            .as_ref()
            .and_then(|range| self.document.borrow().GetDocument().Range(range));
        Rc::new(DOMInputEventStaticRange {
            document: self.document.clone(),
            identity: self.identity,
            snapshot,
        })
    }
}
/// Construct the DOM StaticRange projection used by native and constructed
/// InputEvents without registering a second, temporary live range.
pub fn StaticRangeForInputEvent(
    document: InteractionDocument,
    snapshot: DOMStaticRange,
) -> Result<StaticRangeRef, dom::DOMRangeError> {
    let identity = {
        let owner = document.borrow();
        let dom = owner.GetDocument();
        dom.ValidateRangeBoundaryPoints(snapshot.start, snapshot.end)?;
        dom.RootHandle().Identity()
    };
    Ok(Rc::new(DOMInputEventStaticRange {
        document,
        identity,
        snapshot: Some(snapshot),
    }))
}
impl EventDispatcher<'_, '_> {
    pub(super) fn EditorCommandTargetRanges(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
    ) -> Result<Option<Vec<StaticRangeRef>>, EditorCommandError> {
        if command.Descriptor().is_none() || !self.FrameSelection().IsAvailable() {
            return Ok(None);
        }
        let Some(target) = self
            .targets
            .EventTargetNodeForDocument(self.state.borrow().focused_node_id)
        else {
            return Ok(None);
        };
        self.TargetRangesForEditingCommand(command, &target, false)
    }
    pub(crate) fn TextInputTargetRanges(
        &self,
        target: &interaction_dispatch::core::dom::events::event::EventTargetRef,
    ) -> Result<Option<Vec<StaticRangeRef>>, EditorCommandError> {
        // TargetRangesForInputEvent synchronizes editing layout even when the
        // original target is a plain text control and yields no ranges.
        self.FrameSelection().PrepareEditing();
        self.TargetRangesForEditingCommand(
            interaction_default::core::editing::commands::editor_command_table::EditingCommandType::InsertText,
            target,
            true,
        )
    }
    fn TargetRangesForEditingCommand(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
        target: &interaction_dispatch::core::dom::events::event::EventTargetRef,
        prepared: bool,
    ) -> Result<Option<Vec<StaticRangeRef>>, EditorCommandError> {
        use crate::adapter::page_frame_selection::{CompositionTextQuery, CompositionTextResult};
        use interaction_default::core::editing::commands::editor_command_input_type::DeletionTargetGranularity;
        let Some(node) = target.NodeId() else {
            return Ok(None);
        };
        let identity = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            if self.targets.OwnerDocumentHandle(&target) != Some(document.RootHandle())
                || !document.IsActive()
                || !document
                    .FindNodeById(node)
                    .is_some_and(|node| document.IsConnectedToDocument(node))
                || !foundation::editable::IsRichlyEditable(
                    &dom::persistent_document::DOMEditableFacts(document),
                    node,
                    Default::default(),
                )
            {
                return Ok(None);
            }
            document.RootHandle().Identity()
        };
        let extend = DeletionTargetGranularity(command);
        if extend.is_some()
            && self
                .input_method_state
                .as_ref()
                .and_then(|state| state.GetActiveEditContext())
                .is_some()
        {
            return Ok(None);
        }
        // Update lifecycle before lending the live Document/Layout owners to
        // the source selection algorithm. No dispatch occurs under that borrow.
        if !prepared {
            self.FrameSelection().PrepareEditing();
        }
        if !self.FrameSelection().IsAvailable()
            || self.document.borrow().GetDocument().RootHandle().Identity() != identity
        {
            return Ok(None);
        }
        let settings = self.FrameSelection().InteractionTextSettings().ok_or(
            EditorCommandError::RequiredPort("target range editing environment"),
        )?;
        let result =
            self.FrameSelection()
                .CompositionText(CompositionTextQuery::EditorCommandTargetRange {
                    selection: self.FrameSelection().Current(),
                    directional: self.FrameSelection().SelectionIsDirectional(),
                    extend,
                    behavior: settings.editing_behavior,
                    ui_locale: settings.ui_locale,
                });
        let Some(CompositionTextResult::EditorCommandTargetRange(range)) = result else {
            return Err(EditorCommandError::RequiredPort(
                "resident editor command target range geometry",
            ));
        };
        match range {
            Some(range) => {
                if self.document.borrow().GetDocument().RootHandle().Identity() != identity {
                    return Ok(None);
                }
                let range =
                    StaticRangeForInputEvent(self.document.clone(), range).map_err(|_| {
                        EditorCommandError::RequiredPort(
                            "invalid DOM target range from resident editing geometry",
                        )
                    })?;
                Ok(Some(vec![range]))
            }
            // Source delete extension returns an allocated empty vector when
            // selection is none; TargetRangesForInputEvent returns nullptr.
            None => Ok(extend.map(|_| Vec::new())),
        }
    }
}
