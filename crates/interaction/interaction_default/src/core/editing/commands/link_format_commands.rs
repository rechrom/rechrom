// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! create_link_command.cc, unlink_command.cc and remove_format_command.cc.
//! These enter ApplyStyle's existing split/pushdown/run implementation.
use super::{
    apply_style_command::{self, ApplyStyleQuery as Q, ApplyStyleResult as R},
    paragraph_editing_context::{EditResult, EditingAborted},
    style_commands::{StyleCommandContext, StylePropertyValue},
};
use crate::core::editing::position::Position;
use foundation::{AtomicString, CSSPropertyID as P, CSSValueID as V, QualifiedName};

pub trait LinkFormatContext: StyleCommandContext {
    fn CreateCommandText(&self, text: &[u16]) -> EditResult<u64>;
}
pub fn CreateLink<C: LinkFormatContext>(
    c: &C,
    url: &str,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult<bool> {
    if url.is_empty() {
        return Ok(false);
    }
    let element = c.CreateElement("a")?;
    c.SetApplyAttribute(
        element,
        &QualifiedName::new(
            Default::default(),
            AtomicString::from_str("href"),
            Default::default(),
        ),
        Some(url.encode_utf16().collect::<Vec<_>>().into()),
    )?;
    if start != end {
        apply_style_command::ApplyStyledElement(c, element, start, end)?;
    } else {
        super::composite_edit_command::InsertNodeAt(
            c,
            element,
            &c.QueryPosition(
                start,
                super::structural_editing_context::StructuralPositionQuery::Visible,
            ),
        )?;
        let text = c.CreateCommandText(&url.encode_utf16().collect::<Vec<_>>())?;
        c.Append(text, element)?;
        c.SetApplyStyleSelection(
            &Position::FirstPositionInNode(element),
            &Position::LastPositionInNode(c, &element),
        );
    }
    Ok(true)
}
pub fn Unlink<C: LinkFormatContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    // Chromium intentionally leaves caret-only links unchanged.
    if start == end {
        return Ok(());
    }
    let element = c.CreateElement("a")?;
    apply_style_command::RemoveStyledElement(c, element, start, end)
}
pub fn RemoveFormat<C: LinkFormatContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    let root = start
        .ComputeContainerNode(c)
        .and_then(|node| c.RootEditable(node))
        .ok_or(EditingAborted)?;
    let default = c.CaptureStyle(&Position::FirstPositionInNode(root), false)?;
    let transparent = c
        .CreateCommandStyle(vec![(
            P::kBackgroundColor,
            StylePropertyValue::Identifier(V::kTransparent),
        )])?
        .ok_or(EditingAborted)?;
    let R::Style(style) = c.ApplyStyleQuery(Q::Merge(default, transparent))? else {
        return Err(EditingAborted);
    };
    apply_style_command::RemoveFormatRange(c, style, start, end)
}
