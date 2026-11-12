//! Native command leaves retain the existing elementary command and undo owner.
use super::*;
use interaction_default::core::editing::commands::{
    editor_command_table::ExecuteFunction as F,
    format_block_command::{FormatBlockCommand, FormatBlockContext},
    link_format_commands::{self, LinkFormatContext},
};
impl FormatBlockContext for StructuralCommand {
    fn CopyFormatBlockStyle(&self, from: u64, to: u64) -> EditResult {
        use interaction_default::core::editing::commands::apply_style_command::ApplyStyleContext;
        let style = self
            .dom
            .Read(from, |_, node, _| node.AttributeValueUTF16("style"))
            .flatten();
        if let Some(style) = style {
            self.SetApplyAttribute(
                to,
                &QualifiedName::new(
                    Default::default(),
                    foundation::AtomicString::from_str("style"),
                    Default::default(),
                ),
                Some(style),
            )?;
        }
        Ok(())
    }
}
impl LinkFormatContext for StructuralCommand {
    fn CreateCommandText(&self, text: &[u16]) -> EditResult<u64> {
        let node = self.dom.CreateText(text);
        self.dom.Read(node, |_, _, _| node).ok_or(EditingAborted)
    }
}
pub(crate) fn ApplyLinkFormat(
    host: &Host<'_, '_, '_, '_>,
    command: F,
    value: &str,
    composite: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> EditResult<bool> {
    let c = Context(host, composite);
    let range = host
        .Range(&composite.EndingSelection())
        .ok_or(EditingAborted)?;
    let start = Position::new(range.start.node_id, range.start.offset);
    let end = Position::new(range.end.node_id, range.end.offset);
    match command {
        F::ExecuteCreateLink => return link_format_commands::CreateLink(&c, value, &start, &end),
        F::ExecuteUnlink => link_format_commands::Unlink(&c, &start, &end)?,
        F::ExecuteRemoveFormat => link_format_commands::RemoveFormat(&c, &start, &end)?,
        F::ExecuteFormatBlock => {
            let Some(command) = FormatBlockCommand::FromCommandValue(value) else {
                return Ok(false);
            };
            command.Apply(&c, start, end)?;
            return Ok(command.DidApply());
        }
        _ => return Err(EditingAborted),
    }
    Ok(true)
}
