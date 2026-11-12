//! Native Range and DOMSelection projection.
//!
//! Chromium: core/dom/range.{h,cc}; core/editing/dom_selection.{h,cc}.
//! DOM retains live Range boundaries and Interaction retains FrameSelection.
//! JavaScript wrappers carry neither a copied range nor a copied selection.

use super::*;
use interaction::dispatch::bindings::script_invocation::ScriptNativeObject;
use std::any::Any;

struct NativeRange(dom::DOMRangeHandle);

impl ScriptNativeObject for NativeRange {
    fn AsAny(&self) -> &dyn Any {
        self
    }

    fn ScriptInterfaceName(&self) -> &'static str {
        "Range"
    }
}

impl DOMJavaScriptBindings {
    fn RangeError(error: dom::DOMRangeError) -> HostResult {
        Self::popup_error(&dom::error::DOMException::from(error))
    }

    fn NativeRange(&self, id: HostObjectId) -> Option<Rc<NativeRange>> {
        self.native_abort
            .NativeFromValue(&HostValue::Object(HostObjectRef { id }))
    }

    fn NewNativeRangeValue(
        &self,
        start: dom::DOMBoundaryPoint,
        end: dom::DOMBoundaryPoint,
    ) -> Result<HostValue, dom::DOMRangeError> {
        let range = self
            .document
            .borrow_mut()
            .GetDocumentMut()
            .CreateRange(start, end)?;
        let native: Rc<dyn ScriptNativeObject> = Rc::new(NativeRange(range));
        let id = self.native_abort.EnsureObject(native);
        Ok(self.native_abort.NativeValue(id))
    }

    fn SelectionRange(
        &self,
        snapshot: interaction::DOMSelectionSnapshot,
    ) -> Result<dom::DOMStaticRange, dom::DOMRangeError> {
        let document = self.document.borrow();
        let document = document.GetDocument();
        document.ValidateRangeBoundaryPoints(snapshot.anchor, snapshot.focus)?;
        let order = document.CompareBoundaryPoints(snapshot.anchor, snapshot.focus)?;
        let (start, end) = if order == std::cmp::Ordering::Greater {
            (snapshot.focus, snapshot.anchor)
        } else {
            (snapshot.anchor, snapshot.focus)
        };
        Ok(dom::DOMStaticRange { start, end })
    }

    fn NodeContentsEnd(
        document: &dom::Document,
        node: usize,
    ) -> dom::DOMBoundaryPoint {
        let value = document.Node(node);
        let offset = if matches!(
            value.Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            document.CharacterDataUTF16(node).len()
        } else {
            value.Children().len()
        };
        dom::DOMBoundaryPoint {
            node_id: value.Id(),
            offset,
        }
    }

    fn RangeCall(
        &self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostResult> {
        if member == "__rangeCreate" {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            if document.Node(document.Root()).Id() != receiver {
                return Some(Self::type_error("Illegal Document invocation"));
            }
            let point = dom::DOMBoundaryPoint {
                node_id: receiver,
                offset: 0,
            };
            drop(owner);
            return Some(match self.NewNativeRangeValue(point, point) {
                Ok(value) => Self::value(value),
                Err(error) => Self::RangeError(error),
            });
        }
        let range = self.NativeRange(receiver)?;
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        let result = match member {
            "__rangeSnapshot" => {
                let Some(range) = document.Range(&range.0) else {
                    return Some(Self::RangeError(dom::DOMRangeError::WrongDocument));
                };
                HostValue::Record(vec![
                    (
                        "startContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.start.node_id,
                        }),
                    ),
                    (
                        "startOffset".into(),
                        HostValue::Number(range.start.offset as f64),
                    ),
                    (
                        "endContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.end.node_id,
                        }),
                    ),
                    (
                        "endOffset".into(),
                        HostValue::Number(range.end.offset as f64),
                    ),
                    ("collapsed".into(), HostValue::Boolean(range.Collapsed())),
                ])
            }
            "__rangeSelectNode" => {
                let [HostValue::Object(node)] = arguments else {
                    return Some(Self::type_error("Range node required"));
                };
                let Some(node) = document.FindNodeById(node.id) else {
                    return Some(Self::RangeError(dom::DOMRangeError::MissingNode));
                };
                let Some(parent) = document.Node(node).Parent() else {
                    return Some(Self::RangeError(dom::DOMRangeError::InvalidNodeType));
                };
                let index = document
                    .Node(parent)
                    .Children()
                    .iter()
                    .position(|child| *child == node)
                    .expect("Range node retains its parent child identity");
                let parent = document.Node(parent).Id();
                if let Err(error) = document.SetRange(
                    &range.0,
                    dom::DOMBoundaryPoint {
                        node_id: parent,
                        offset: index,
                    },
                    dom::DOMBoundaryPoint {
                        node_id: parent,
                        offset: index + 1,
                    },
                ) {
                    return Some(Self::RangeError(error));
                }
                HostValue::default()
            }
            "__rangeSelectNodeContents" => {
                let [HostValue::Object(node)] = arguments else {
                    return Some(Self::type_error("Range node required"));
                };
                let Some(node) = document.FindNodeById(node.id) else {
                    return Some(Self::RangeError(dom::DOMRangeError::MissingNode));
                };
                let start = dom::DOMBoundaryPoint {
                    node_id: document.Node(node).Id(),
                    offset: 0,
                };
                let end = Self::NodeContentsEnd(document, node);
                if let Err(error) = document.SetRange(&range.0, start, end) {
                    return Some(Self::RangeError(error));
                }
                HostValue::default()
            }
            "__rangeCollapse" => {
                let [HostValue::Boolean(to_start)] = arguments else {
                    return Some(Self::type_error("Range collapse requires a boolean"));
                };
                let Some(snapshot) = document.Range(&range.0) else {
                    return Some(Self::RangeError(dom::DOMRangeError::WrongDocument));
                };
                let point = if *to_start {
                    snapshot.start
                } else {
                    snapshot.end
                };
                if let Err(error) = document.SetRange(&range.0, point, point) {
                    return Some(Self::RangeError(error));
                }
                HostValue::default()
            }
            "__rangeSetStart" | "__rangeSetEnd" => {
                let [HostValue::Object(node), HostValue::Number(offset)] = arguments else {
                    return Some(Self::type_error("Range boundary required"));
                };
                if !offset.is_finite() || *offset < 0.0 {
                    return Some(Self::RangeError(dom::DOMRangeError::InvalidOffset));
                }
                let point = dom::DOMBoundaryPoint {
                    node_id: node.id,
                    offset: *offset as usize,
                };
                let changed = if member == "__rangeSetStart" {
                    document.SetRangeStart(&range.0, point)
                } else {
                    document.SetRangeEnd(&range.0, point)
                };
                if let Err(error) = changed {
                    return Some(Self::RangeError(error));
                }
                HostValue::default()
            }
            "__rangeDeleteContents" => {
                if let Err(error) = document.DeleteRangeContents(&range.0) {
                    return Some(Self::RangeError(error));
                }
                HostValue::default()
            }
            _ => return None,
        };
        Some(Self::value(result))
    }

    fn SelectionCall(
        &self,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostResult> {
        if !member.starts_with("__selection") {
            return None;
        }
        let Some(host) = self.services.host.selection.clone() else {
            return Some(Self::type_error("Selection host is unavailable"));
        };
        match member {
            "__selectionSnapshot" => {
                let Some(snapshot) = (host.read)() else {
                    return Some(Self::value(HostValue::Null(JavaScriptNull)));
                };
                let range = match self.SelectionRange(snapshot) {
                    Ok(range) => range,
                    Err(error) => return Some(Self::RangeError(error)),
                };
                Some(Self::value(HostValue::Record(vec![
                    (
                        "anchorNode".into(),
                        HostValue::Object(HostObjectRef {
                            id: snapshot.anchor.node_id,
                        }),
                    ),
                    (
                        "anchorOffset".into(),
                        HostValue::Number(snapshot.anchor.offset as f64),
                    ),
                    (
                        "focusNode".into(),
                        HostValue::Object(HostObjectRef {
                            id: snapshot.focus.node_id,
                        }),
                    ),
                    (
                        "focusOffset".into(),
                        HostValue::Number(snapshot.focus.offset as f64),
                    ),
                    (
                        "collapsed".into(),
                        HostValue::Boolean(snapshot.anchor == snapshot.focus),
                    ),
                    (
                        "startContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.start.node_id,
                        }),
                    ),
                    (
                        "startOffset".into(),
                        HostValue::Number(range.start.offset as f64),
                    ),
                    (
                        "endContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.end.node_id,
                        }),
                    ),
                    (
                        "endOffset".into(),
                        HostValue::Number(range.end.offset as f64),
                    ),
                ])))
            }
            "__selectionAddRange" => {
                let [HostValue::Object(value)] = arguments else {
                    return Some(Self::type_error("Range required"));
                };
                let Some(range) = self.NativeRange(value.id) else {
                    return Some(Self::type_error("Range required"));
                };
                // Selection API supports one range.  Like Chromium, adding a
                // second range leaves the original selection unchanged.
                if (host.read)().is_none() {
                    let Some(snapshot) = self.document.borrow().GetDocument().Range(&range.0)
                    else {
                        return Some(Self::RangeError(dom::DOMRangeError::WrongDocument));
                    };
                    (host.write)(Some(snapshot));
                }
                Some(Self::value(HostValue::default()))
            }
            "__selectionClear" => {
                (host.write)(None);
                Some(Self::value(HostValue::default()))
            }
            "__selectionCollapse" => {
                let [HostValue::Object(node), HostValue::Number(offset)] = arguments else {
                    return Some(Self::type_error("Selection collapse boundary required"));
                };
                if !offset.is_finite() || *offset < 0.0 {
                    return Some(Self::RangeError(dom::DOMRangeError::InvalidOffset));
                }
                let point = dom::DOMBoundaryPoint {
                    node_id: node.id,
                    offset: *offset as usize,
                };
                if let Err(error) = self
                    .document
                    .borrow()
                    .GetDocument()
                    .ValidateRangeBoundaryPoints(point, point)
                {
                    return Some(Self::RangeError(error));
                }
                (host.write)(Some(dom::DOMStaticRange {
                    start: point,
                    end: point,
                }));
                Some(Self::value(HostValue::default()))
            }
            "__selectionSelectAllChildren" => {
                let [HostValue::Object(node)] = arguments else {
                    return Some(Self::type_error("Selection node required"));
                };
                let owner = self.document.borrow();
                let document = owner.GetDocument();
                let Some(node) = document.FindNodeById(node.id) else {
                    return Some(Self::RangeError(dom::DOMRangeError::MissingNode));
                };
                let range = dom::DOMStaticRange {
                    start: dom::DOMBoundaryPoint {
                        node_id: document.Node(node).Id(),
                        offset: 0,
                    },
                    end: Self::NodeContentsEnd(document, node),
                };
                drop(owner);
                (host.write)(Some(range));
                Some(Self::value(HostValue::default()))
            }
            "__selectionGetRangeAt" => {
                let [HostValue::Number(index)] = arguments else {
                    return Some(Self::type_error("Selection index required"));
                };
                if *index != 0.0 {
                    return Some(Self::popup_error(&dom::error::DOMException {
                        kind: dom::error::DOMExceptionKind::InvalidArgument,
                        message: "selection range index is out of bounds",
                    }));
                }
                let Some(snapshot) = (host.read)() else {
                    return Some(Self::popup_error(&dom::error::DOMException {
                        kind: dom::error::DOMExceptionKind::InvalidArgument,
                        message: "selection has no range",
                    }));
                };
                let range = match self.SelectionRange(snapshot) {
                    Ok(range) => range,
                    Err(error) => return Some(Self::RangeError(error)),
                };
                Some(match self.NewNativeRangeValue(range.start, range.end) {
                    Ok(value) => Self::value(value),
                    Err(error) => Self::RangeError(error),
                })
            }
            "__selectionToString" => {
                Some(Self::value(HostValue::String16((host.text)().into())))
            }
            "__selectionDeleteFromDocument" => {
                let Some(snapshot) = (host.read)() else {
                    return Some(Self::value(HostValue::default()));
                };
                let range = match self.SelectionRange(snapshot) {
                    Ok(range) => range,
                    Err(error) => return Some(Self::RangeError(error)),
                };
                let handle = match self
                    .document
                    .borrow_mut()
                    .GetDocumentMut()
                    .CreateRange(range.start, range.end)
                {
                    Ok(range) => range,
                    Err(error) => return Some(Self::RangeError(error)),
                };
                if let Err(error) = self
                    .document
                    .borrow_mut()
                    .GetDocumentMut()
                    .DeleteRangeContents(&handle)
                {
                    return Some(Self::RangeError(error));
                }
                Some(Self::value(HostValue::default()))
            }
            _ => None,
        }
    }

    pub(super) fn PrepareRangeSelection(
        &self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostContinuation> {
        let result = self
            .RangeCall(receiver, member, arguments)
            .or_else(|| self.SelectionCall(member, arguments))?;
        Some(Box::new(move |_| result))
    }
}
