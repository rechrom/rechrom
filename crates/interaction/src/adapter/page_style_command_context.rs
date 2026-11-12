//! StyleCommands leaves over the resident StructuralCommand and compiled CSS.
use super::*;
use cssom::editing_style::{
    ApplyStyleOperation as Op, ApplyStyleResult as Out, EditingStylePropertyValue,
};
use interaction_default::core::editing::commands::style_commands::{
    self, StyleCommandContext, StyleCommandOutcome, StylePropertyValue,
};
impl StyleCommandContext for StructuralCommand {
    fn CreateCommandStyle(
        &self,
        properties: Vec<(foundation::CSSPropertyID, StylePropertyValue)>,
    ) -> EditResult<Option<Self::Style>> {
        let properties = properties
            .into_iter()
            .map(|(p, value)| {
                (
                    p,
                    match value {
                        StylePropertyValue::Identifier(value) => {
                            EditingStylePropertyValue::Identifier(value)
                        }
                        StylePropertyValue::Text(value) => EditingStylePropertyValue::Text(value),
                        StylePropertyValue::LegacyFontSize(value) => {
                            EditingStylePropertyValue::LegacyFontSize(value)
                        }
                    },
                )
            })
            .collect();
        match self.replacement_apply_style_query(Op::Create { properties })? {
            Out::Declaration(declaration) => Ok(Some(NativeEditingStyle::new(declaration))),
            Out::InvalidValue => Ok(None),
            _ => Err(EditingAborted),
        }
    }
    fn CommandStyleAtStart(&self, node: u64) -> EditResult<Self::Style> {
        match self.replacement_apply_style_query(Op::CommandStart {
            node_id: node,
            typing: self.CommandTypingStyle().map(|s| s.declaration),
        })? {
            Out::Declaration(declaration) => Ok(NativeEditingStyle::new(declaration)),
            _ => Err(EditingAborted),
        }
    }
    fn CommandStylePresent(
        &self,
        node: u64,
        desired: &Self::Style,
        include_typing: bool,
    ) -> EditResult<bool> {
        match self.replacement_apply_style_query(Op::CommandPresent {
            node_id: node,
            declaration: desired.declaration.clone(),
            typing: include_typing
                .then(|| self.CommandTypingStyle())
                .flatten()
                .map(|s| s.declaration),
        })? {
            Out::Bool(value) => Ok(value),
            _ => Err(EditingAborted),
        }
    }
    fn ToggleCommandList(
        &self,
        selected: Self::Style,
        property: foundation::CSSPropertyID,
        value: foundation::CSSValueID,
    ) -> EditResult<(Self::Style, bool)> {
        match self.replacement_apply_style_query(Op::ToggleList {
            declaration: selected.declaration,
            property,
            value,
        })? {
            Out::ToggledList {
                declaration,
                is_none,
            } => Ok((NativeEditingStyle::new(declaration), is_none)),
            _ => Err(EditingAborted),
        }
    }
    fn CommandTypingStyle(&self) -> Option<Self::Style> {
        self.dom
            .editor
            .TypingStyle()
            .map(|style| style.borrow().clone())
    }
    fn ShouldToggleStyleBasedOnStartOfSelection(&self) -> bool {
        self.dom
            .frame
            .upgrade()
            .and_then(|frame| frame.InteractionTextSettings())
            .is_some_and(|settings| {
                settings
                    .editing_behavior
                    .ShouldToggleStyleBasedOnStartOfSelection()
            })
    }
    fn SetShouldStyleWithCss(&self, value: bool) {
        self.dom.editor.SetShouldStyleWithCss(value);
    }
}
impl StructuralCommand {
    fn replacement_apply_style_query(&self, operation: Op) -> EditResult<Out> {
        self.dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .ApplyEditingStyle(operation)
            .map_err(|_| EditingAborted)
    }
}
pub(crate) fn ExecuteStyle(
    host: &Host<'_, '_, '_, '_>,
    command: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
    composite: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> EditResult<StyleCommandOutcome> {
    let context = Context(host, composite);
    let selection = host
        .Frame()
        .CreateVisibleSelection(host.Frame().Current())
        .ok_or(EditingAborted)?;
    let range = host
        .Range(&ImeTypingSelection(selection))
        .ok_or(EditingAborted)?;
    style_commands::Execute(
        &context,
        command,
        value,
        &Position::new(range.start.node_id, range.start.offset),
        &Position::new(range.end.node_id, range.end.offset),
    )
}
