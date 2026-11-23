//! cc::NodeId (DOM node identity carried by paint chunks and layers).
/// Source dependency cc/paint/node_id.h:17-20. Native invalid NodeId is zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(pub i32);
