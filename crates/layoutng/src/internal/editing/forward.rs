//! Opaque editing declarations. This package only uses these as names and
//! pointer/reference targets; their concrete definitions live in editing.

use std::convert::Infallible;
use std::marker::PhantomData;

// cpp: layoutng/internal/editing/forward.h:12-12
pub enum TextAffinity {}

// cpp: layoutng/internal/editing/forward.h:14-15
pub enum NodeTraversal {}
pub enum FlatTreeTraversal {}

// cpp: layoutng/internal/editing/forward.h:17-20
pub struct EditingAlgorithm<Traversal> {
    _opaque: Infallible,
    _traversal: PhantomData<Traversal>,
}
pub type EditingStrategy = EditingAlgorithm<NodeTraversal>;
pub type EditingInFlatTreeStrategy = EditingAlgorithm<FlatTreeTraversal>;

// cpp: layoutng/internal/editing/forward.h:22-25
pub struct PositionTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type Position = PositionTemplate<EditingStrategy>;
pub type PositionInFlatTree = PositionTemplate<EditingInFlatTreeStrategy>;

// cpp: layoutng/internal/editing/forward.h:27-31
pub struct EphemeralRangeTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type EphemeralRange = EphemeralRangeTemplate<EditingStrategy>;
pub type EphemeralRangeInFlatTree = EphemeralRangeTemplate<EditingInFlatTreeStrategy>;

// cpp: layoutng/internal/editing/forward.h:33-37
pub struct PositionWithAffinityTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type PositionWithAffinity = PositionWithAffinityTemplate<EditingStrategy>;
pub type PositionInFlatTreeWithAffinity = PositionWithAffinityTemplate<EditingInFlatTreeStrategy>;

// cpp: layoutng/internal/editing/forward.h:39-42
pub struct SelectionTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type SelectionInDomTree = SelectionTemplate<EditingStrategy>;
pub type SelectionInFlatTree = SelectionTemplate<EditingInFlatTreeStrategy>;

// cpp: layoutng/internal/editing/forward.h:44-48
pub struct VisiblePositionTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type VisiblePosition = VisiblePositionTemplate<EditingStrategy>;
pub type VisiblePositionInFlatTree = VisiblePositionTemplate<EditingInFlatTreeStrategy>;

// cpp: layoutng/internal/editing/forward.h:50-54
pub struct VisibleSelectionTemplate<Strategy> {
    _opaque: Infallible,
    _strategy: PhantomData<Strategy>,
}
pub type VisibleSelection = VisibleSelectionTemplate<EditingStrategy>;
pub type VisibleSelectionInFlatTree = VisibleSelectionTemplate<EditingInFlatTreeStrategy>;
