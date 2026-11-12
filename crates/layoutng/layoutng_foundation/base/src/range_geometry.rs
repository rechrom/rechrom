//! Small per-node DOM Range inputs. Produced lazily from the live range/tree;
//! neither the iterator nor the geometry consumer retains a document snapshot.
#[derive(Clone, Copy, Debug)]
pub enum RangeGeometryNode {
    Element {
        node: u64,
        parent: Option<u64>,
        contains_start: bool,
        contains_end: bool,
    },
    Text {
        node: u64,
        start: u32,
        end: u32,
    },
}
