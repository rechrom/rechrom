//! TextControlElement::ValueWithHardLineBreaks, cc:82-129,1215-1251.
//! Native InlineCursor/OffsetMapping and Node/Text owners remain in Layout;
//! only the final lossless value crosses its boundary.
use crate::{
    internal::{
        algorithm_forward::OffsetMapping,
        editing::forward::Position,
        inline_node::InlineNode,
        layout_block_flow::LayoutBlockFlow,
        layout_node_metadata::{Node, Text},
    },
    layout_engine::LayoutEngine,
};
use layout_fragment::inline_cursor::InlineCursor;
use std::rc::Rc;
fn GetNextSoftBreak(mapping: &OffsetMapping, cursor: &mut InlineCursor) -> Position {
    while cursor.IsNotNull() {
        debug_assert!(cursor.Current().IsLineBox());
        let token = cursor.Current().GetInlineBreakToken();
        cursor.MoveToNextLine();
        if cursor.IsNull() {
            return Position::default();
        }
        if let Some(token) = unsafe { token.as_ref() }.filter(|t| !t.IsForcedBreak()) {
            return mapping.GetFirstPosition(token.StartTextOffset());
        }
    }
    Position::default()
}
fn AppendWrappedNode(
    node: &Node,
    mapping: &OffsetMapping,
    cursor: &mut InlineCursor,
    break_position: &mut Position,
    result: &mut Vec<u16>,
) {
    let object = unsafe { &*node.GetLayoutObject() };
    if object.IsBR() {
        let element = foundation::DynamicTo::<crate::internal::layout_node_metadata::Element>(
            node as *const Node,
        );
        if !unsafe { element.as_ref() }
            .is_some_and(|element| element.InputIsTextControlPlaceholder())
        {
            result.push(10);
        }
    } else if node.IsTextNode() {
        let text = unsafe { &*(node as *const Node as *const Text) };
        let data = text.data().Span16().unwrap_or(&[]);
        let mut position = 0;
        while break_position.node_id == Some(node.InputId()) && break_position.offset <= data.len()
        {
            let offset = break_position.offset;
            if offset > position {
                result.extend_from_slice(&data[position..offset]);
                position = offset;
                result.push(10);
            }
            *break_position = GetNextSoftBreak(mapping, cursor);
        }
        result.extend_from_slice(&data[position..]);
    }
    while break_position.node_id == Some(node.InputId()) {
        *break_position = GetNextSoftBreak(mapping, cursor);
    }
}
impl LayoutEngine {
    /// None is the source's no LayoutObject/no valid IFC branch; the control
    /// then returns its own unchanged Value(), as cc:1220-1225,1251 specifies.
    pub fn TextControlValueWithHardLineBreaks(&self, inner_editor: u64) -> Option<Rc<[u16]>> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.GetLayoutObjectForNode(inner_editor)?;
        assert!(object.IsLayoutBlockFlow());
        let mut anonymous = object.SlowFirstChild();
        let mut result = Vec::new();
        let mut valid = false;
        while let Some(flow_object) = unsafe { anonymous.as_ref() } {
            assert!(flow_object.IsLayoutBlockFlow());
            let flow = unsafe { &*(anonymous as *const LayoutBlockFlow) };
            let mut cursor = InlineCursor::from_container(flow);
            if cursor.IsNotNull() {
                if let Some(mapping) =
                    unsafe { InlineNode::GetOffsetMapping(anonymous.cast()).as_ref() }
                {
                    valid = true;
                    let mut break_position = GetNextSoftBreak(mapping, &mut cursor);
                    let mut node = unsafe { flow_object.SlowFirstChild().as_ref() }
                        .map_or(std::ptr::null_mut(), |first| first.GetNode());
                    while let Some(current) = unsafe { node.as_ref() } {
                        let Some(layout) = (unsafe { current.GetLayoutObject().as_ref() }) else {
                            break;
                        };
                        if layout.Parent() != anonymous {
                            break;
                        }
                        AppendWrappedNode(
                            current,
                            mapping,
                            &mut cursor,
                            &mut break_position,
                            &mut result,
                        );
                        node = current.nextSibling();
                    }
                }
            }
            anonymous = flow_object.NextSibling();
        }
        valid.then(|| result.into())
    }
}
