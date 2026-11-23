//! Plain values carried by CommitState from LayerTreeHost to LayerTreeHostImpl.
//! Moved unchanged from compositor/src/adapter/{layer_tree_impl,commit_state}.rs;
//! the old paths re-export them.
use std::cmp::Ordering;

// Source dependency: base/unguessable_token.h (two 64-bit halves; empty is 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnguessableToken(pub [u64; 2]);
impl UnguessableToken {
    pub fn is_empty(self) -> bool {
        self.0 == [0, 0]
    }
}

// Source dependency: ui/gfx/overlay_transform.h gfx::OverlayTransform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayTransform {
    #[default]
    None,
    FlipHorizontal,
    FlipVertical,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipVerticalRotate90,
    FlipVerticalRotate270,
    Invalid,
}

// Source dependency: cc/trees/layer_tree_host.h
// PropertyChangeForcesCommitCriteria.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PropertyChangeForcesCommitCriteria {
    #[default]
    kNone,
    kTransform,
    kAny,
}

// Source dependency: cc/resources/ui_resource_client.h:18 UIResourceId=int.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UIResourceId(pub i32);
impl PartialOrd for UIResourceId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for UIResourceId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

// Source: cc/resources/ui_resource_request.h:17-21 UIResourceRequest::Type.
// Moved from compositor/src/adapter/layer_tree_impl.rs, which re-exports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UIResourceRequestType {
    kCreate,
    kDelete,
}

// Source dependency: gfx::InsetsF native value (top,left,bottom,right).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InsetsF {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}
