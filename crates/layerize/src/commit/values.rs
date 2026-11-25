//! Plain values carried by CommitState from LayerTreeHost to LayerTreeHostImpl.
//! Moved unchanged from compositor/src/adapter/{layer_tree_impl,commit_state}.rs;
//! the old paths re-export them.
use std::cmp::Ordering;

// base::UnguessableToken and gfx::OverlayTransform are base/gfx values that
// viz::CompositorFrameMetadata names too.
pub use compositor_values::overlay_transform::OverlayTransform;
pub use compositor_values::unguessable_token::UnguessableToken;

/// base::UnguessableToken::Create (base/unguessable_token.cc): 128 bits from
/// the OS cryptographic source, never the empty token. Both halves' viz
/// LocalSurfaceId embed tokens come from it (the main widget's parent id and
/// the impl ParentLocalSurfaceIdAllocator). Moved from
/// compositor/src/runtime/host_core.rs OsSurfaceTokenSource.
pub fn CreateUnguessableToken() -> UnguessableToken {
    loop {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("OS entropy for LocalSurfaceId token");
        let token = UnguessableToken([
            u64::from_ne_bytes(bytes[..8].try_into().expect("token half")),
            u64::from_ne_bytes(bytes[8..].try_into().expect("token half")),
        ]);
        if !token.is_empty() {
            return token;
        }
    }
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
