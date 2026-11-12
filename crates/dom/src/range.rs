//! Live Range boundary ownership from Blink's core/dom/range.cc and
//! range_boundary_point.h. The document's existing nodes remain authoritative.
#![allow(non_snake_case)]

use crate::persistent_document::{DOMNodeType, DOMOwnerHandle, PersistentDocument};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
#[path = "range_transaction.rs"]
mod transaction;
pub(crate) use transaction::{DeleteContents, InsertNode};

/// CharacterData offsets count UTF-16 code units; container offsets count children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMBoundaryPoint {
    pub node_id: u64,
    pub offset: usize,
}

/// A snapshot retains its offsets even when subsequent DOM mutations move a Range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMStaticRange {
    pub start: DOMBoundaryPoint,
    pub end: DOMBoundaryPoint,
}

impl DOMStaticRange {
    pub fn Collapsed(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMRangeError {
    MissingNode,
    InvalidNodeType,
    InvalidOffset,
    WrongDocument,
    HierarchyRequest,
}

/// Like Blink's Range, this owner is attached weakly to its Document. Clones
/// name the same live range; dropping the last handle ends its lifetime.
#[derive(Clone, Debug)]
pub struct DOMRangeHandle {
    document: DOMOwnerHandle,
    data: Rc<RefCell<RangeData>>,
}

impl PartialEq for DOMRangeHandle {
    fn eq(&self, other: &Self) -> bool {
        self.document == other.document && Rc::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for DOMRangeHandle {}

impl From<DOMRangeError> for crate::error::DOMException {
    fn from(error: DOMRangeError) -> Self {
        Self {
            kind: crate::error::DOMExceptionKind::InvalidArgument,
            message: match error {
                DOMRangeError::MissingNode => "range boundary node does not exist",
                DOMRangeError::InvalidNodeType => "invalid range boundary node type",
                DOMRangeError::InvalidOffset => "range offset exceeds node length",
                DOMRangeError::WrongDocument => "range belongs to another document",
                DOMRangeError::HierarchyRequest => {
                    "range insertion would violate the node hierarchy"
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Boundary {
    container: usize,
    offset: usize,
    // RangeBoundaryPoint::child_before_boundary_: insertions do not invalidate
    // node identity, and the numeric container offset is computed on demand.
    child_before: Option<usize>,
}

impl Boundary {
    fn Offset(self, document: &PersistentDocument) -> usize {
        if let Some(child) = self.child_before {
            document
                .Node(self.container)
                .Children()
                .iter()
                .position(|&candidate| candidate == child)
                .expect("live range child-before must remain in its container")
                + 1
        } else {
            self.offset
        }
    }

    fn Project(self, document: &PersistentDocument) -> DOMBoundaryPoint {
        DOMBoundaryPoint {
            node_id: document.Node(self.container).Id(),
            offset: self.Offset(document),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RangeData {
    start: Boundary,
    end: Boundary,
}

impl PersistentDocument {
    fn RangeBoundary(&self, point: DOMBoundaryPoint) -> Result<Boundary, DOMRangeError> {
        let container = self
            .FindNodeById(point.node_id)
            .ok_or(DOMRangeError::MissingNode)?;
        let node = self.Node(container);
        if node.Type() == DOMNodeType::kDocumentType {
            return Err(DOMRangeError::InvalidNodeType);
        }
        let character_data = matches!(
            node.Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        );
        let length = if character_data {
            self.CharacterDataUTF16(container).len()
        } else {
            node.Children().len()
        };
        if point.offset > length || point.offset > i32::MAX as usize {
            return Err(DOMRangeError::InvalidOffset);
        }
        Ok(Boundary {
            container,
            offset: if character_data { point.offset } else { 0 },
            child_before: if character_data || point.offset == 0 {
                None
            } else {
                Some(node.Children()[point.offset - 1])
            },
        })
    }

    // ComparePositions through the canonical parent chain, never a copied tree.
    fn CompareRangeBoundaries(&self, a: Boundary, b: Boundary) -> Option<Ordering> {
        if a.container == b.container {
            return Some(a.Offset(self).cmp(&b.Offset(self)));
        }
        let ancestors = |mut node| {
            let mut path = vec![node];
            while let Some(parent) = self.Node(node).Parent() {
                path.push(parent);
                node = parent;
            }
            path
        };
        let a_path = ancestors(a.container);
        let b_path = ancestors(b.container);
        if a_path.last() != b_path.last() {
            return None;
        }
        if let Some(depth) = b_path.iter().position(|&node| node == a.container) {
            let child = b_path[depth - 1];
            let index = self
                .Node(a.container)
                .Children()
                .iter()
                .position(|&node| node == child)
                .unwrap();
            return Some(if a.Offset(self) <= index {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }
        if let Some(depth) = a_path.iter().position(|&node| node == b.container) {
            let child = a_path[depth - 1];
            let index = self
                .Node(b.container)
                .Children()
                .iter()
                .position(|&node| node == child)
                .unwrap();
            return Some(if index < b.Offset(self) {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }
        let mut a_depth = a_path.len() - 1;
        let mut b_depth = b_path.len() - 1;
        while a_path[a_depth - 1] == b_path[b_depth - 1] {
            a_depth -= 1;
            b_depth -= 1;
        }
        let children = self.Node(a_path[a_depth]).Children();
        let index = |node| {
            children
                .iter()
                .position(|&candidate| candidate == node)
                .unwrap()
        };
        Some(index(a_path[a_depth - 1]).cmp(&index(b_path[b_depth - 1])))
    }

    /// DOM ComparePositions over the canonical ancestry/boundary owner. Node
    /// numeric identifiers are identities and never imply document order.
    pub fn CompareBoundaryPoints(
        &self,
        first: DOMBoundaryPoint,
        second: DOMBoundaryPoint,
    ) -> Result<Ordering, DOMRangeError> {
        let first = self.RangeBoundary(first)?;
        let second = self.RangeBoundary(second)?;
        self.CompareRangeBoundaries(first, second)
            .ok_or(DOMRangeError::WrongDocument)
    }

    /// Validate static boundary values through Range's canonical node/offset
    /// checks without registering or copying a live Range owner.
    pub fn ValidateRangeBoundaryPoints(
        &self,
        start: DOMBoundaryPoint,
        end: DOMBoundaryPoint,
    ) -> Result<(), DOMRangeError> {
        self.RangeBoundary(start)?;
        self.RangeBoundary(end)?;
        Ok(())
    }

    pub fn CreateRange(
        &mut self,
        start: DOMBoundaryPoint,
        end: DOMBoundaryPoint,
    ) -> Result<DOMRangeHandle, DOMRangeError> {
        let start = self.RangeBoundary(start)?;
        let end = self.RangeBoundary(end)?;
        // Range(startContainer, startOffset, endContainer, endOffset) performs
        // setStart then setEnd; reversed or disconnected bounds collapse to end.
        let start = if self
            .CompareRangeBoundaries(start, end)
            .is_some_and(|order| order != Ordering::Greater)
        {
            start
        } else {
            end
        };
        let data = Rc::new(RefCell::new(RangeData { start, end }));
        self.ranges.retain(|range| range.strong_count() != 0);
        self.ranges.push(Rc::downgrade(&data));
        Ok(DOMRangeHandle {
            document: self.RootHandle(),
            data,
        })
    }

    /// Range::Dispose removes the native temporary owner from Document's
    /// weak live-range registry. It does not replace or reset its endpoints.
    pub fn DisposeRange(&mut self, range: &DOMRangeHandle) {
        if range.document != self.RootHandle() {
            return;
        }
        self.ranges
            .retain(|registered| registered.as_ptr() != Rc::as_ptr(&range.data));
    }
    pub fn Range(&self, range: &DOMRangeHandle) -> Option<DOMStaticRange> {
        if range.document != self.RootHandle() {
            return None;
        }
        let data = range.data.borrow();
        Some(DOMStaticRange {
            start: data.start.Project(self),
            end: data.end.Project(self),
        })
    }

    /// Range::commonAncestorContainer, range.cc:194-204. The containers are
    /// read from this same live range; traversal remains in its DOM tree.
    pub fn RangeCommonAncestorContainer(&self, range: &DOMRangeHandle) -> Option<u64> {
        if range.document != self.RootHandle() {
            return None;
        }
        let data = range.data.borrow();
        let (mut a, mut b) = (Some(data.start.container), Some(data.end.container));
        let depth = |mut node: Option<usize>| {
            let mut length = 0;
            while let Some(n) = node {
                length += 1;
                node = self.Node(n).Parent();
            }
            length
        };
        let (mut ad, mut bd) = (depth(a), depth(b));
        while ad > bd {
            a = a.and_then(|n| self.Node(n).Parent());
            ad -= 1;
        }
        while bd > ad {
            b = b.and_then(|n| self.Node(n).Parent());
            bd -= 1;
        }
        while a != b {
            a = a.and_then(|n| self.Node(n).Parent());
            b = b.and_then(|n| self.Node(n).Parent());
        }
        a.map(|n| self.Node(n).Id())
    }

    pub fn SetRange(
        &self,
        range: &DOMRangeHandle,
        start: DOMBoundaryPoint,
        end: DOMBoundaryPoint,
    ) -> Result<(), DOMRangeError> {
        // Validate both before changing a retained range.
        self.RangeBoundary(start)?;
        self.RangeBoundary(end)?;
        self.SetRangeStart(range, start)?;
        self.SetRangeEnd(range, end)
    }

    pub fn SetRangeStart(
        &self,
        range: &DOMRangeHandle,
        point: DOMBoundaryPoint,
    ) -> Result<(), DOMRangeError> {
        if range.document != self.RootHandle() {
            return Err(DOMRangeError::WrongDocument);
        }
        let start = self.RangeBoundary(point)?;
        let mut data = range.data.borrow_mut();
        data.start = start;
        if !self
            .CompareRangeBoundaries(start, data.end)
            .is_some_and(|order| order != Ordering::Greater)
        {
            data.end = start;
        }
        Ok(())
    }

    pub fn SetRangeEnd(
        &self,
        range: &DOMRangeHandle,
        point: DOMBoundaryPoint,
    ) -> Result<(), DOMRangeError> {
        if range.document != self.RootHandle() {
            return Err(DOMRangeError::WrongDocument);
        }
        let end = self.RangeBoundary(point)?;
        let mut data = range.data.borrow_mut();
        data.end = end;
        if !self
            .CompareRangeBoundaries(data.start, end)
            .is_some_and(|order| order != Ordering::Greater)
        {
            data.start = end;
        }
        Ok(())
    }

    fn RangeContainsNode(&self, container: usize, mut node: usize) -> bool {
        loop {
            if node == container {
                return true;
            }
            let Some(parent) = self.Node(node).Parent() else {
                return false;
            };
            node = parent;
        }
    }

    fn RangeNodeIndex(&self, node: usize) -> usize {
        self.Node(self.Node(node).Parent().expect("node has a parent"))
            .Children()
            .iter()
            .position(|&child| child == node)
            .unwrap()
    }

    fn RangeNextSibling(&self, node: usize) -> Option<usize> {
        self.Node(self.Node(node).Parent()?)
            .Children()
            .get(self.RangeNodeIndex(node) + 1)
            .copied()
    }

    fn RangeContentsLength(&self, node: usize) -> usize {
        if matches!(
            self.Node(node).Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            self.CharacterDataUTF16(node).len()
        } else {
            self.Node(node).Children().len()
        }
    }

    /// Range::deleteContents, with the same source transaction used by the
    /// shared arena's synchronous native removal lifecycle.
    pub fn DeleteRangeContents(&mut self, range: &DOMRangeHandle) -> Result<(), DOMRangeError> {
        DeleteContents(self, range)
    }

    /// Range::insertNode's pre-insertion check, before splitting text or detaching
    /// the inserted node. This uses only canonical immediate child identities.
    pub fn ValidateRangeInsertion(
        &self,
        range: &DOMRangeHandle,
        node_id: u64,
    ) -> Result<(), DOMRangeError> {
        let snapshot = self.Range(range).ok_or(DOMRangeError::WrongDocument)?;
        let start = self
            .FindNodeById(snapshot.start.node_id)
            .ok_or(DOMRangeError::MissingNode)?;
        let node = self
            .FindNodeById(node_id)
            .ok_or(DOMRangeError::MissingNode)?;
        if start == node
            || matches!(
                self.Node(start).Type(),
                DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
            )
        {
            return Err(DOMRangeError::HierarchyRequest);
        }
        let is_text = self.Node(start).Type() == DOMNodeType::kText;
        let parent = if is_text {
            self.Node(start)
                .Parent()
                .ok_or(DOMRangeError::HierarchyRequest)?
        } else {
            start
        };
        if !matches!(
            self.Node(parent).Type(),
            DOMNodeType::kElement | DOMNodeType::kDocument | DOMNodeType::kDocumentFragment
        ) || self.RangeContainsNode(node, parent)
            || self.Node(node).Type() == DOMNodeType::kDocument
        {
            return Err(DOMRangeError::HierarchyRequest);
        }
        let inserted = if self.Node(node).Type() == DOMNodeType::kDocumentFragment {
            self.Node(node).Children().to_vec()
        } else {
            vec![node]
        };
        if self.Node(parent).Type() != DOMNodeType::kDocument {
            if inserted.iter().any(|&child| {
                matches!(
                    self.Node(child).Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentType
                )
            }) {
                return Err(DOMRangeError::HierarchyRequest);
            }
            return Ok(());
        }
        let mut reference = if is_text {
            Some(start)
        } else {
            self.Node(start)
                .Children()
                .get(snapshot.start.offset)
                .copied()
        };
        if reference == Some(node) {
            reference = self.RangeNextSibling(node);
        }
        let mut children = self
            .Node(parent)
            .Children()
            .iter()
            .copied()
            .filter(|&child| child != node)
            .collect::<Vec<_>>();
        let index = reference
            .and_then(|reference| children.iter().position(|&child| child == reference))
            .unwrap_or(children.len());
        children.splice(index..index, inserted);
        let mut element = false;
        let mut doctype = false;
        for child in children {
            match self.Node(child).Type() {
                DOMNodeType::kElement if !element => element = true,
                DOMNodeType::kDocumentType if !doctype && !element => doctype = true,
                DOMNodeType::kComment | DOMNodeType::kProcessingInstruction => {}
                _ => return Err(DOMRangeError::HierarchyRequest),
            }
        }
        Ok(())
    }

    /// Range::insertNode, including Text::splitText and collapsed-end expansion.
    pub fn InsertRangeNode(
        &mut self,
        range: &DOMRangeHandle,
        node_id: u64,
    ) -> Result<(), DOMRangeError> {
        InsertNode(self, range, node_id)
    }

    // Range::NodeWillBeRemoved, before the canonical parent links change.
    pub(crate) fn RangeNodeWillBeRemoved(&self, child: usize) {
        if self.ranges.is_empty() {
            return;
        }
        let parent = self.Node(child).Parent().expect("removal has a parent");
        let children = self.Node(parent).Children();
        let index = children.iter().position(|&node| node == child).unwrap();
        for range in self.ranges.iter().filter_map(|range| range.upgrade()) {
            let mut range = range.borrow_mut();
            let RangeData { start, end } = &mut *range;
            for boundary in [start, end] {
                if boundary.child_before == Some(child) {
                    boundary.child_before = index.checked_sub(1).map(|index| children[index]);
                    boundary.offset = 0;
                    continue;
                }
                let mut ancestor = Some(boundary.container);
                while let Some(node) = ancestor {
                    if node == child {
                        *boundary = Boundary {
                            container: parent,
                            offset: 0,
                            child_before: index.checked_sub(1).map(|index| children[index]),
                        };
                        break;
                    }
                    ancestor = self.Node(node).Parent();
                }
            }
        }
    }

    // Range::DidRemoveText followed by DidInsertText, as CharacterData::replaceData.
    pub(crate) fn RangeCharacterDataReplaced(
        &self,
        node: usize,
        offset: usize,
        removed: usize,
        inserted: usize,
    ) {
        for range in self.ranges.iter().filter_map(|range| range.upgrade()) {
            let mut range = range.borrow_mut();
            let RangeData { start, end } = &mut *range;
            for boundary in [start, end] {
                if boundary.container != node || boundary.offset <= offset {
                    continue;
                }
                boundary.offset = if boundary.offset <= offset + removed {
                    offset
                } else {
                    boundary.offset - removed
                };
                if boundary.offset > offset {
                    boundary.offset += inserted;
                }
            }
        }
    }

    // Range::DidSplitTextNode. Parent boundaries after the old text move after
    // the new sibling; text boundaries past the split move to that sibling.
    pub(crate) fn RangeTextNodeSplit(&self, old: usize, next: usize, offset: usize) {
        for range in self.ranges.iter().filter_map(|range| range.upgrade()) {
            let mut range = range.borrow_mut();
            let RangeData { start, end } = &mut *range;
            for boundary in [start, end] {
                if boundary.child_before == Some(old) {
                    boundary.child_before = Some(next);
                } else if boundary.container == old && boundary.offset > offset {
                    boundary.container = next;
                    boundary.offset -= offset;
                }
            }
        }
    }
}
