//! Native read-only leaves for EventHandler's keyboard context-menu placement.
//! Source event_handler.cc:153-165,2239-2382; editing_utilities.cc:1247-1258.
//! The caller borrows FrameSelection's current endpoints and prepared Layout;
//! VisibleSelection and the original LayoutNode queries own canonicalization.
use super::*;
use crate::core::input::event_handler::nonlocated_context_menu::{
    NonLocatedCaret, NonLocatedSelectionFlags,
};
use foundation::gfx;

pub enum NonLocatedGeometryQuery {
    Selection(Option<DOMSelection>),
    ElementVisibleBounds(u64),
    ElementOutlineRectsInRootFrame(u64),
}
pub enum NonLocatedGeometryResult {
    Selection {
        flags: NonLocatedSelectionFlags,
        start: Option<NonLocatedCaret>,
        end: Option<NonLocatedCaret>,
        anchor: Option<u64>,
    },
    Rect(gfx::Rect),
    Rects(Vec<gfx::Rect>),
}

pub fn QueryNonLocatedGeometry(
    document: &Document,
    layout: LayoutOutput<'_>,
    query: NonLocatedGeometryQuery,
) -> Option<NonLocatedGeometryResult> {
    use dom::document_lifecycle::LifecycleState;
    let state = document.Lifecycle().GetState();
    if state < LifecycleState::LayoutClean || state >= LifecycleState::Stopping {
        return None;
    }
    let _lifecycle = document.Lifecycle().DisallowTransition();
    match query {
        NonLocatedGeometryQuery::ElementVisibleBounds(node) => {
            Some(NonLocatedGeometryResult::Rect(
                layout.Node(node).map_or_else(gfx::Rect::default, |node| {
                    node.ElementVisibleBoundsInLocalRoot()
                }),
            ))
        }
        NonLocatedGeometryQuery::ElementOutlineRectsInRootFrame(node) => {
            Some(NonLocatedGeometryResult::Rects(
                layout
                    .Node(node)
                    .map_or_else(Vec::new, |node| node.ElementOutlineRectsInRootFrame()),
            ))
        }
        NonLocatedGeometryQuery::Selection(selection) => {
            let context = EditingPositionContext {
                document,
                layout: Some(layout),
                flat: false,
                ui_locale: None,
                behavior: None,
            };
            let Some(visible) = CreateVisibleDOMSelectionForFocus(document, layout, selection)
            else {
                return Some(NonLocatedGeometryResult::Selection {
                    flags: NonLocatedSelectionFlags {
                        is_range: false,
                        has_editable_root: false,
                        is_caret: false,
                    },
                    start: None,
                    end: None,
                    anchor: None,
                });
            };
            let a = SourcePosition(EditingPositionWithAffinity {
                position: visible.anchor,
                affinity: visible.affinity,
            });
            let f = SourcePosition(EditingPositionWithAffinity {
                position: visible.focus,
                affinity: visible.affinity,
            });
            let (start, end) =
                if compare_positions::CompareDOMPositions(&context, &a.position, &f.position)
                    == Some(Ordering::Greater)
                {
                    (&f, &a)
                } else {
                    (&a, &f)
                };
            let is_caret = a.position.IsEquivalent(&context, &f.position);
            let flags = NonLocatedSelectionFlags {
                is_range: !is_caret,
                has_editable_root: EditableBoundaryContext::RootEditableElementOf(
                    &context,
                    &start.position,
                )
                .is_some(),
                is_caret,
            };
            let caret = |position: &PositionWithAffinity<u64>| -> Option<NonLocatedCaret> {
                let native = NativePosition(&position.position, position.affinity)?;
                let node = layout.Node(native.position.node_id?)?;
                let editing_offset = position
                    .position
                    .ComputeEditingOffset(&context)
                    .min(i32::MAX as usize) as i32;
                // Source uses LocalCaretRect's own writing mode, rather than
                // the anchor's style (which may differ at an inline boundary).
                let (quad, empty, horizontal) = node.CaretQuadEmptyAndHorizontal(
                    &native,
                    Default::default(),
                    editing_offset,
                    node.CaretBarWidth(),
                );
                (!empty).then(|| NonLocatedCaret {
                    absolute_rect: gfx::ToEnclosingRect(quad.BoundingBox()),
                    horizontal,
                })
            };
            // Source EnclosingAnchorElement uses DOM ElementTraversal and the
            // IsLink source flag; this intentionally does not walk flat parents.
            let mut ancestor = start
                .position
                .AnchorNode()
                .and_then(|n| document.FindNodeById(*n));
            let mut anchor = None;
            while let Some(index) = ancestor {
                if document.IsLinkForEditing(index) {
                    anchor = Some(document.Node(index).Id());
                    break;
                }
                ancestor = document.Node(index).Parent();
            }
            Some(NonLocatedGeometryResult::Selection {
                flags,
                start: caret(start),
                end: caret(end),
                anchor,
            })
        }
    }
}
