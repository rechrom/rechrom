//! ValidationMessageClientImpl's Page-owned presentation state. Source:
//! third_party/blink/renderer/core/page/validation_message_client_impl.cc.
//! The author document contributes an anchor and text; the host owns the
//! independent overlay Page and its normal Surface/CompositorFrame lifecycle.
use super::*;
use interaction::adapter::page_form_validation::{
    ControlValidity, ValidationMessageClient, ValidationMessageEffect, ValidationRequired,
};

// The host's existing overlay consumes a single alert text owner. Preserve
// InputType's title submessage when projecting the source message pair.
fn ValidationAlertText(message: &[u16], sub_message: &[u16]) -> Arc<[u16]> {
    if sub_message.is_empty() {
        return Arc::from(message);
    }
    message
        .iter()
        .copied()
        .chain([b'\n' as u16])
        .chain(sub_message.iter().copied())
        .collect::<Vec<_>>()
        .into()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidationMessageRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationMessageUpdate {
    Show {
        control: u64,
        anchor: ValidationMessageRect,
        message: Arc<[u16]>,
    },
    Hide {
        control: u64,
        immediate: bool,
    },
}
/// Semantic AXObjectCache::HandleValidationMessageVisibilityChanged event.
/// A show carries alert text; hide clears the control's validation alert.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationMessageVisibility {
    pub sequence: u64,
    pub control: u64,
    pub visible: bool,
    pub message: Arc<[u16]>,
}
pub(super) struct VisibleValidationMessage {
    pub(super) control: u64,
    document: dom::persistent_document::DOMOwnerHandle,
    frame: u64,
    anchor_revision: u64,
    pub(super) anchor: ValidationMessageRect,
    pub(super) message: Arc<[u16]>,
}
pub(super) struct PageValidationMessageClient {
    page: Weak<PageState>,
    frame: u64,
    document: dom::persistent_document::DOMOwnerHandle,
}
impl PageValidationMessageClient {
    pub(super) fn new(state: &Rc<PageState>) -> Rc<Self> {
        Rc::new(Self {
            page: Rc::downgrade(state),
            frame: state.local_frames.get().unwrap().root,
            document: state.document.borrow().GetDocument().RootHandle(),
        })
    }
    pub(super) fn ForChild(
        frame: &Rc<local_frame::LocalFrame>,
        runtime: &Rc<local_frame::LocalFrameRuntime>,
    ) -> Rc<Self> {
        Rc::new(Self {
            page: frame
                .domain
                .upgrade()
                .map_or_else(Weak::new, |domain| domain.page.Downgrade()),
            frame: frame.state.Identity(),
            document: runtime.document.borrow().GetDocument().RootHandle(),
        })
    }
}
impl ValidationMessageClient for PageValidationMessageClient {
    fn Apply(&self, effect: ValidationMessageEffect) -> Result<(), ValidationRequired> {
        let Some(state) = self.page.upgrade() else {
            return Ok(());
        };
        let domain = state.local_frames.get().unwrap();
        let Some(document) = domain.Document(self.frame) else {
            return Ok(());
        };
        if document.borrow().GetDocument().RootHandle() != self.document {
            return Ok(());
        }
        match effect {
            ValidationMessageEffect::Show {
                anchor: control,
                message,
                sub_message,
            } => {
                if message.is_empty() {
                    state.HideValidationMessageFor(self.frame, control);
                    return Ok(());
                }
                // Focus/report ordering belongs to ListedElement. It has
                // completed before this host notification enters Page.
                if domain.EnsureFocusLayout(self.frame).is_err() {
                    return Ok(());
                }
                let Some(anchor) = state.ValidationAnchor(self.frame, control) else {
                    return Ok(());
                };
                state.HideValidationMessage();
                let message = ValidationAlertText(&message, &sub_message);
                let document = self.document;
                let Some(anchor_revision) = state.ValidationAnchorRevision(self.frame, control)
                else {
                    return Ok(());
                };
                *state.validation_message.borrow_mut() = Some(VisibleValidationMessage {
                    control,
                    document,
                    frame: self.frame,
                    anchor_revision,
                    anchor,
                    message: message.clone(),
                });
                let mut client = state.client.borrow_mut();
                client.DidChangeValidationMessage(ValidationMessageUpdate::Show {
                    control,
                    anchor,
                    message: message.clone(),
                });
                drop(client);
                state.NotifyValidationMessageVisibility(control, true, message);
            }
            ValidationMessageEffect::Hide { anchor } => {
                state.HideValidationMessageFor(self.frame, anchor)
            }
        }
        Ok(())
    }
}
impl PageState {
    fn ValidationAnchorRevision(&self, frame: u64, control: u64) -> Option<u64> {
        let owner = self.local_frames.get()?.Document(frame)?;
        let owner = owner.borrow();
        let document = owner.GetDocument();
        Some(document.ElementInternalsValidationAnchorRevision(document.FindNodeById(control)?))
    }
    fn ValidationAnchor(&self, frame: u64, control: u64) -> Option<ValidationMessageRect> {
        let domain = self.local_frames.get()?;
        let owner = domain.Document(frame)?;
        let anchor = {
            let owner = owner.borrow();
            let document = owner.GetDocument();
            let index = document.FindNodeById(control)?;
            if !document.IsActive() || !document.IsConnectedToDocument(index) {
                return None;
            }
            let anchor = document.ElementInternalsValidationAnchor(index);
            if !document.IsConnectedToDocument(anchor) {
                return None;
            }
            document.Node(anchor).Id()
        };
        let rect = if frame == domain.root {
            self.layout_engine
                .borrow()
                .Output()?
                .Node(anchor)?
                .VisibleBoundsInFrame()?
        } else {
            let child = domain.Frame(frame)?;
            let runtime = child.runtime.borrow().clone()?;
            let rect = runtime
                .layout
                .borrow()
                .Output()?
                .Node(anchor)?
                .VisibleBoundsInFrame()?;
            domain.MapValidationRectToRoot(frame, rect)?
        };
        let zoom = self
            .constraints
            .borrow()
            .layout_zoom_factor
            .max(f64::MIN_POSITIVE);
        Some(ValidationMessageRect {
            x: f64::from(rect.x()) / zoom,
            y: f64::from(rect.y()) / zoom,
            width: f64::from(rect.width()) / zoom,
            height: f64::from(rect.height()) / zoom,
        })
    }
    pub(super) fn HideValidationMessage(&self) {
        self.HideValidationMessageWithPolicy(true);
    }
    fn HideValidationMessageWithPolicy(&self, immediate: bool) {
        let visible = self.validation_message.borrow_mut().take();
        if let Some(visible) = visible {
            let mut client = self.client.borrow_mut();
            client.DidChangeValidationMessage(ValidationMessageUpdate::Hide {
                control: visible.control,
                immediate,
            });
            drop(client);
            self.NotifyValidationMessageVisibility(visible.control, false, Arc::from([]));
        }
    }
    fn NotifyValidationMessageVisibility(&self, control: u64, visible: bool, message: Arc<[u16]>) {
        let sequence = self
            .validation_message_visibility
            .borrow()
            .as_ref()
            .map_or(1, |event| event.sequence.saturating_add(1));
        let event = ValidationMessageVisibility {
            sequence,
            control,
            visible,
            message,
        };
        *self.validation_message_visibility.borrow_mut() = Some(event.clone());
        self.client
            .borrow_mut()
            .DidChangeValidationMessageVisibility(event);
    }
    fn HideValidationMessageFor(&self, frame: u64, control: u64) {
        if self
            .validation_message
            .borrow()
            .as_ref()
            .is_some_and(|v| v.frame == frame && v.control == control)
        {
            self.HideValidationMessageWithPolicy(false);
        }
    }
    pub(super) fn ValidationMessageFrameDetached(&self, frame: u64) {
        if self
            .validation_message
            .borrow()
            .as_ref()
            .is_some_and(|v| v.frame == frame)
        {
            self.HideValidationMessage();
        }
    }
    pub(super) fn ValidationMessageFocusChanged(&self, focused: Option<u64>) {
        if self
            .validation_message
            .borrow()
            .as_ref()
            .is_some_and(|v| Some(v.control) != focused)
        {
            self.HideValidationMessage();
        }
    }
    pub(super) fn UpdateValidationMessage(&self) {
        let Some((control, frame, identity, anchor_revision)) = self
            .validation_message
            .borrow()
            .as_ref()
            .map(|v| (v.control, v.frame, v.document, v.anchor_revision))
        else {
            return;
        };
        let domain = self.local_frames.get().unwrap();
        let Some(document) = domain.Document(frame) else {
            self.HideValidationMessage();
            return;
        };
        if document.borrow().GetDocument().RootHandle() != identity {
            self.HideValidationMessage();
            return;
        }
        if self.ValidationAnchorRevision(frame, control) != Some(anchor_revision) {
            self.HideValidationMessage();
            return;
        }
        let Some(source) = domain.FocusInteraction(frame) else {
            self.HideValidationMessage();
            return;
        };
        let Ok(validity) = ControlValidity(&document.Handle(), &source.FormControlState(), control)
        else {
            self.HideValidationMessage();
            return;
        };
        if !validity.will_validate || validity.validity.valid() || validity.message.is_empty() {
            self.HideValidationMessageWithPolicy(false);
            return;
        }
        let Some(anchor) = self.ValidationAnchor(frame, control) else {
            self.HideValidationMessageWithPolicy(false);
            return;
        };
        let message = ValidationAlertText(&validity.message, &validity.sub_message);
        let (update, message_changed) = {
            let mut visible = self.validation_message.borrow_mut();
            let Some(visible) = visible.as_mut() else {
                return;
            };
            if anchor == visible.anchor && message == visible.message {
                return;
            }
            let message_changed = message != visible.message;
            visible.anchor = anchor;
            visible.message = message.clone();
            (
                ValidationMessageUpdate::Show {
                    control,
                    anchor,
                    message: message.clone(),
                },
                message_changed,
            )
        };
        self.client.borrow_mut().DidChangeValidationMessage(update);
        if message_changed {
            self.NotifyValidationMessageVisibility(control, true, message);
        }
    }
}

impl local_frame::LocalFrameDomain {
    /// Compose each actual FrameTree owner: child viewport → embedded content
    /// box → parent's native visual-rect/clip chain → local root Surface.
    fn MapValidationRectToRoot(
        &self,
        token: u64,
        rect: foundation::gfx::RectF,
    ) -> Option<foundation::gfx::RectF> {
        let frame = self.Frame(token)?;
        let mapped = self.WithParentLayout(&frame, |layout| {
            layout
                .Output()?
                .Node(frame.state.Owner()?.Id())?
                .EmbeddedContentVisibleRectInFrame(rect)
        })?;
        if frame.parent == self.root {
            Some(mapped)
        } else {
            self.MapValidationRectToRoot(frame.parent, mapped)
        }
    }
}
