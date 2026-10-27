use crate::PaintRect;

/// Whether an old retained clip recorded at least the successor's coverage.
pub fn ClipContextCovers(old: Option<PaintRect>, new: Option<PaintRect>) -> bool {
    match (old, new) {
        (None, None) => true,
        (Some(old), Some(new)) => old.contains(new),
        _ => false,
    }
}

/// Whether a retained cull window covers a successor window.
///
/// `None` is the unbounded source state, so it covers every finite successor.
pub fn CullContextCovers(old: Option<PaintRect>, new: Option<PaintRect>) -> bool {
    match (old, new) {
        (None, _) => true,
        (Some(old), Some(new)) => old.contains(new),
        (Some(_), None) => false,
    }
}
