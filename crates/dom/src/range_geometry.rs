//! Range.cc1433-1438 / Position.cc311-345 and Range.cc1718-1771 DOM facts.
//! This cloneable cursor borrows the original DOM and restarts the source's
//! second pass without copying nodes, positions or a traversal list.
use crate::{
    persistent_document::{DOMNodeType, PersistentDocument},
    range::{DOMBoundaryPoint, DOMRangeHandle, DOMStaticRange},
};
use foundation::range_geometry::RangeGeometryNode;
#[derive(Clone)]
pub struct DOMRangeGeometryNodes<'a> {
    document: &'a PersistentDocument,
    range: DOMStaticRange,
    current: Option<usize>,
    past_last: Option<usize>,
}
fn IsCharacterData(kind: DOMNodeType) -> bool {
    matches!(
        kind,
        DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
    )
}
impl PersistentDocument {
    pub fn RangeNextSkippingChildren(&self, node: usize) -> Option<usize> {
        self.NextNodeWithin(node, None, true)
    }
    pub fn RangeNextNode(&self, node: usize) -> Option<usize> {
        self.NextNodeWithin(node, None, false)
    }
    pub fn RangeFirstNodeForPoint(&self, point: DOMBoundaryPoint) -> Option<usize> {
        let node = self.FindNodeById(point.node_id)?;
        if IsCharacterData(self.Node(node).Type()) {
            return Some(node);
        }
        if let Some(&child) = self.Node(node).Children().get(point.offset) {
            return Some(child);
        }
        if point.offset == 0 {
            return Some(node);
        }
        self.RangeNextSkippingChildren(node)
    }
    pub fn RangePastLastNodeForPoint(&self, point: DOMBoundaryPoint) -> Option<usize> {
        let node = self.FindNodeById(point.node_id)?;
        if IsCharacterData(self.Node(node).Type()) {
            return self.RangeNextSkippingChildren(node);
        }
        self.Node(node)
            .Children()
            .get(point.offset)
            .copied()
            .or_else(|| self.RangeNextSkippingChildren(node))
    }
    pub fn RangeGeometryNodes(&self, range: &DOMRangeHandle) -> Option<DOMRangeGeometryNodes<'_>> {
        let range = self.Range(range)?;
        Some(DOMRangeGeometryNodes {
            document: self,
            range,
            current: self.RangeFirstNodeForPoint(range.start),
            past_last: self.RangePastLastNodeForPoint(range.end),
        })
    }
}
impl DOMRangeGeometryNodes<'_> {
    fn Contains(&self, ancestor: usize, node_id: u64) -> bool {
        self.document
            .FindNodeById(node_id)
            .is_some_and(|node| self.document.ContainsNode(ancestor, node))
    }
}
impl Iterator for DOMRangeGeometryNodes<'_> {
    type Item = RangeGeometryNode;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.current?;
            if Some(node) == self.past_last {
                self.current = None;
                return None;
            }
            self.current = self.document.RangeNextNode(node);
            let current = self.document.Node(node);
            if current.Type() == DOMNodeType::kElement {
                return Some(RangeGeometryNode::Element {
                    node: current.Id(),
                    parent: current
                        .Parent()
                        .map(|parent| self.document.Node(parent).Id()),
                    contains_start: self.Contains(node, self.range.start.node_id),
                    contains_end: self.Contains(node, self.range.end.node_id),
                });
            }
            if current.Type() == DOMNodeType::kText {
                return Some(RangeGeometryNode::Text {
                    node: current.Id(),
                    start: if current.Id() == self.range.start.node_id {
                        u32::try_from(self.range.start.offset)
                            .expect("source Range unsigned offset")
                    } else {
                        0
                    },
                    end: if current.Id() == self.range.end.node_id {
                        u32::try_from(self.range.end.offset).expect("source Range unsigned offset")
                    } else {
                        u32::MAX
                    },
                });
            }
        }
    }
}
