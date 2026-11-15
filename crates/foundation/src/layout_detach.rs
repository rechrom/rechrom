//! Small values crossing Node/Style/Layout detach stages. No native pointer or
//! engine storage crosses the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutDetachedParent {
    pub element: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutDetachParentFlags {
    pub whitespace_children_may_change: bool,
    pub was_notified_of_subtree_change: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutDetachParentMutation {
    pub set_whitespace_children_may_change: bool,
    pub notify_of_subtree_change: bool,
}
