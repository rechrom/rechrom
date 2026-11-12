//! LayoutObject editing positions and node/object association.
//!
//! These queries enter inline/text traversal through assembly providers, so
//! they are physically and observably owned by the inline algorithm package.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:3454-3457
    pub fn CanBeSelectionLeafInternalBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanBeSelectionLeafInternal(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectCanBeSelectionLeafInternal(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2187-2187
    #[inline(never)]
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { DispatchLayoutObjectPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_object.h:2188-2189
    #[inline(never)]
    pub fn CreatePositionWithAffinity(
        &self,
        offset: i32,
        affinity: TextAffinity,
    ) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityProvider(self, offset, affinity) }
    }

    // cpp: layoutng/internal/layout_object.h:2190-2190
    #[inline(never)]
    pub fn CreatePositionWithAffinityDefault(&self, offset: i32) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityDefaultProvider(self, offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2191-2191
    #[inline(never)]
    pub fn FindPosition(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFindPositionProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2192-2192
    #[inline(never)]
    pub fn FirstPositionInOrBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFirstPositionInOrBeforeThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2193-2193
    #[inline(never)]
    pub fn LastPositionInOrAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectLastPositionInOrAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2194-2194
    #[inline(never)]
    pub fn PositionAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2195-2195
    #[inline(never)]
    pub fn PositionBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionBeforeThisProvider(self) }
    }
}

// cpp: layoutng/internal/layout_object.h:3949-3952
#[inline(never)]
pub fn AssociatedLayoutObjectOf(
    node: &Node,
    offset_in_node: i32,
    side: LayoutObjectSide,
) -> *const LayoutObject {
    unsafe { AssociatedLayoutObjectOfProvider(node, offset_in_node, side) }
}

pub fn AssociatedLayoutObjectOfDefault(node: &Node, offset_in_node: i32) -> *const LayoutObject {
    AssociatedLayoutObjectOf(
        node,
        offset_in_node,
        LayoutObjectSide::kRemainingTextIfOnBoundary,
    )
}

// DOM boundary adjustment is applied by the editing owner after these neutral
// values cross the package boundary. Layout retains point/caret resolution,
// source anchor kind, and native anonymous-object traversal.
fn LayoutObjectCreatePositionWithAffinityProvider(
    object: &LayoutObject,
    offset: i32,
    affinity: TextAffinity,
) -> PositionWithAffinity {
    let Some(node) = (unsafe { object.NonPseudoNode().as_ref() }) else {
        return object.FindPosition();
    };
    debug_assert!(offset >= 0);
    PositionWithAffinity {
        position: crate::EditingPosition::new(node.InputId(), offset as usize),
        affinity,
    }
}
fn LayoutObjectCreatePositionWithAffinityDefaultProvider(
    object: &LayoutObject,
    offset: i32,
) -> PositionWithAffinity {
    LayoutObjectCreatePositionWithAffinityProvider(object, offset, TextAffinity::Downstream)
}
fn FirstForNode(node: &Node) -> PositionWithAffinity {
    let facts = node.InputEditingFacts();
    PositionWithAffinity {
        position: if facts.ignores_content {
            crate::EditingPosition::WithAnchor(
                node.InputId(),
                0,
                crate::EditingPositionAnchorType::BeforeAnchor,
            )
        } else {
            crate::EditingPosition::new(node.InputId(), 0)
        },
        affinity: TextAffinity::Downstream,
    }
}
fn LastForNode(node: &Node) -> PositionWithAffinity {
    let facts = node.InputEditingFacts();
    PositionWithAffinity {
        position: if facts.ignores_content && !node.parentNode().is_null() {
            crate::EditingPosition::WithAnchor(
                node.InputId(),
                0,
                crate::EditingPositionAnchorType::AfterAnchor,
            )
        } else {
            crate::EditingPosition::new(node.InputId(), facts.last_offset)
        },
        affinity: TextAffinity::Downstream,
    }
}
// Source: core/layout/layout_object.cc:5035-5075.
fn LayoutObjectFindPositionProvider(object: &LayoutObject) -> PositionWithAffinity {
    let mut child = object as *const LayoutObject;
    while let Some(parent) = unsafe { (&*child).Parent().as_ref() } {
        let mut next = unsafe { &*child }.NextInPreOrder(parent);
        while let Some(candidate) = unsafe { next.as_ref() } {
            if let Some(node) = unsafe { candidate.NonPseudoNode().as_ref() } {
                return FirstForNode(node);
            }
            next = candidate.NextInPreOrder(parent);
        }
        let mut previous = unsafe { &*child }.PreviousInPreOrderDefault();
        while let Some(candidate) = unsafe { previous.as_ref() } {
            if std::ptr::eq(candidate, parent) {
                break;
            }
            if let Some(node) = unsafe { candidate.NonPseudoNode().as_ref() } {
                return LastForNode(node);
            }
            previous = candidate.PreviousInPreOrderDefault();
        }
        if let Some(node) = unsafe { parent.NonPseudoNode().as_ref() } {
            return FirstForNode(node);
        }
        child = parent;
    }
    PositionWithAffinity::default()
}
fn LayoutObjectFirstPositionInOrBeforeThisProvider(object: &LayoutObject) -> PositionWithAffinity {
    unsafe { object.NonPseudoNode().as_ref() }
        .map(FirstForNode)
        .unwrap_or_else(|| object.FindPosition())
}
fn LayoutObjectLastPositionInOrAfterThisProvider(object: &LayoutObject) -> PositionWithAffinity {
    unsafe { object.NonPseudoNode().as_ref() }
        .map(LastForNode)
        .unwrap_or_else(|| object.FindPosition())
}
fn LayoutObjectPositionAfterThisProvider(object: &LayoutObject) -> PositionWithAffinity {
    NeighborPosition(object, crate::EditingPositionAnchorType::AfterAnchor)
}
fn LayoutObjectPositionBeforeThisProvider(object: &LayoutObject) -> PositionWithAffinity {
    NeighborPosition(object, crate::EditingPositionAnchorType::BeforeAnchor)
}
fn NeighborPosition(
    object: &LayoutObject,
    anchor: crate::EditingPositionAnchorType,
) -> PositionWithAffinity {
    unsafe { object.NonPseudoNode().as_ref() }
        .map(|node| PositionWithAffinity {
            position: crate::EditingPosition::WithAnchor(node.InputId(), 0, anchor),
            affinity: TextAffinity::Downstream,
        })
        .unwrap_or_else(|| object.FindPosition())
}
#[cfg(feature = "translation_in_progress")]
fn DispatchLayoutObjectPositionForPoint(
    object: &LayoutObject,
    point: &PhysicalOffset,
) -> PositionWithAffinity {
    crate::layout_algo_inline_editing_caret::PositionForPoint(
        object,
        point,
        crate::EditingPositionOptions::default(),
    )
}

// Source: core/layout/layout_object.cc:5554-5580.
#[cfg(feature = "translation_in_progress")]
fn AssociatedLayoutObjectOfProvider(
    node: &Node,
    offset: i32,
    side: LayoutObjectSide,
) -> *const LayoutObject {
    debug_assert!(offset >= 0);
    let object = node.GetLayoutObject();
    let fragment = foundation::DynamicTo::<crate::layout_text_fragment::LayoutTextFragment>(object);
    if !node.IsTextNode() || fragment.is_null() {
        return object;
    }
    let fragment = unsafe { &*fragment };
    if !fragment.IsRemainingTextLayoutObject() {
        debug_assert!(offset as u32 <= fragment.Start() + fragment.FragmentLength());
        return object;
    }
    if fragment.FragmentLength() != 0 {
        let threshold = fragment.Start()
            + usize::from(matches!(side, LayoutObjectSide::kFirstLetterIfOnBoundary)) as u32;
        if offset as u32 >= threshold {
            return object;
        }
    }
    fragment.GetFirstLetterPart().cast::<LayoutObject>()
}

// Source: layout_object.h:3615-3619, layout_text.h:396-399,
// layout_image.h:149-152, layout_embedded_content.h:105-108.
fn DispatchLayoutObjectCanBeSelectionLeafInternal(object: *const LayoutObject) -> bool {
    let Some(object) = (unsafe { object.as_ref() }) else {
        return false;
    };
    object.IsText() || object.IsImage() || object.IsLayoutEmbeddedContent()
}

// The block-only assembly intentionally has no resident inline owners. Preserve
// its external dispatch contract; production's inline-enabled assembly above
// supplies the concrete algorithms instead of a fallback position algorithm.
#[cfg(not(feature = "translation_in_progress"))]
unsafe extern "Rust" {
    fn DispatchLayoutObjectPositionForPoint(
        object: &LayoutObject,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn AssociatedLayoutObjectOfProvider(
        node: &Node,
        offset: i32,
        side: LayoutObjectSide,
    ) -> *const LayoutObject;
}
