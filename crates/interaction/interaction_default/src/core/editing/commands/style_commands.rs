// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source style_commands.{h,cc} and Editor::ComputeAndSetTypingStyle.
//! Commands retain compiled declarations, and use the same ApplyStyleCommand
//! and Editor typing-style owner as insertion and deletion.
use super::editor_command_table::{EditingTriState as T, StateFunction as S, ValueFunction as L};
use super::{
    apply_style_command::{
        ApplyStyleContext, ApplyStyleQuery as Q, ApplyStyleRange, ApplyStyleResult as R,
        StyleExtraction,
    },
    editor_command_table::ExecuteFunction as F,
    paragraph_editing_context::{EditResult, EditingAborted},
};
use crate::core::editing::{position::Position, text_offset_mapping::NextNode};
use foundation::{CSSPropertyID as P, CSSValueID as V};
use interaction_dispatch::core::events::input_event::InputType as I;

/// Read leaves borrow the same computed declarations and typing style as ApplyStyle.
pub trait StyleCommandQueryContext {
    fn IsTag(&self, node: u64, tag: &str) -> bool;
    fn RootEditableElement(&self, node: u64) -> Option<u64>;
    fn HasActiveEditContext(&self) -> bool;
    fn ShouldStyleWithCss(&self) -> bool;
    fn ToggleBasedOnStart(&self) -> bool;
    fn Selection(&self) -> Option<(u64, u64, bool)>;
    fn NextNode(&self, node: u64) -> Option<u64>;
    fn Parent(&self, node: u64) -> Option<u64>;
    fn IsText(&self, node: u64) -> bool;
    fn IsStyledElement(&self, node: u64) -> bool;
    fn HasEditableLayout(&self, node: u64) -> bool;
    fn IsBlock(&self, node: u64) -> bool;
    fn Present(&self, node: u64, property: P, value: V, typing: bool) -> EditResult<bool>;
    fn Value(&self, node: u64, property: P) -> EditResult<String>;
    fn Bidi(&self, node: u64) -> EditResult<(V, Option<foundation::TextDirection>)>;
    fn TypingDirection(&self) -> EditResult<Option<Option<foundation::TextDirection>>>;
}
fn Contains<C: StyleCommandQueryContext>(c: &C, ancestor: u64, mut node: u64) -> bool {
    loop {
        if node == ancestor {
            return true;
        }
        let Some(parent) = c.Parent(node) else {
            return false;
        };
        node = parent;
    }
}
// editor_command.cc SelectionListState and FormatBlockCommand's value query.
fn ListState<C: StyleCommandQueryContext>(c: &C, ordered: bool) -> T {
    if c.HasActiveEditContext() {
        return T::kFalse;
    }
    let Some((start, end, caret)) = c.Selection() else {
        return T::kFalse;
    };
    let tag = if ordered { "ol" } else { "ul" };
    let enclosing = |mut node| loop {
        if c.IsTag(node, tag) {
            return Some(node);
        }
        let Some(parent) = c.Parent(node) else {
            return None;
        };
        node = parent;
    };
    let Some(list) = enclosing(start) else {
        return T::kFalse;
    };
    if caret {
        return T::kTrue;
    }
    if enclosing(end) != Some(list) {
        return T::kFalse;
    }
    let mut node = c.NextNode(list);
    while let Some(current) = node {
        if !Contains(c, list, current) {
            break;
        }
        if c.IsTag(current, if ordered { "ul" } else { "ol" }) {
            return T::kFalse;
        }
        node = c.NextNode(current);
    }
    T::kTrue
}
pub fn FormatBlockValue<C: StyleCommandQueryContext>(c: &C) -> String {
    if c.HasActiveEditContext() {
        return String::new();
    }
    let Some((start, end, _)) = c.Selection() else {
        return String::new();
    };
    let mut common = Some(start);
    while common.is_some_and(|node| !Contains(c, node, end)) {
        common = common.and_then(|node| c.Parent(node));
    }
    const TAGS: &[&str] = &[
        "address",
        "article",
        "aside",
        "blockquote",
        "dd",
        "div",
        "dl",
        "dt",
        "footer",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hgroup",
        "main",
        "nav",
        "p",
        "pre",
        "section",
    ];
    while let Some(node) = common {
        if let Some(tag) = TAGS.iter().find(|tag| c.IsTag(node, tag)) {
            return if c
                .RootEditableElement(start)
                .is_some_and(|root| !Contains(c, node, root))
            {
                (*tag).into()
            } else {
                String::new()
            };
        }
        common = c.Parent(node);
    }
    String::new()
}
pub fn StateStyle<C: StyleCommandQueryContext>(c: &C, property: P, value: V) -> EditResult<T> {
    if c.HasActiveEditContext() {
        return Ok(T::kFalse);
    }
    let Some((start, end, caret)) = c.Selection() else {
        return Ok(T::kFalse);
    };
    if caret || c.ToggleBasedOnStart() {
        return Ok(if c.Present(start, property, value, true)? {
            T::kTrue
        } else {
            T::kFalse
        });
    }
    let mut state = None;
    let mut node = Some(start);
    while let Some(current) = node {
        if c.HasEditableLayout(current) {
            let present = c.Present(current, property, value, false)?;
            match state {
                None => state = Some(present),
                Some(previous) if previous != present && c.IsText(current) => return Ok(T::kMixed),
                _ => {}
            }
        }
        if current == end {
            break;
        }
        node = c.NextNode(current);
    }
    Ok(if state == Some(true) {
        T::kTrue
    } else {
        T::kFalse
    })
}
fn TextDirection<C: StyleCommandQueryContext>(
    c: &C,
) -> EditResult<(Option<foundation::TextDirection>, bool)> {
    let Some((start, end, caret)) = c.Selection() else {
        return Ok((None, true));
    };
    if !caret {
        let mut node = Some(start);
        while let Some(current) = node {
            if c.IsStyledElement(current)
                && matches!(
                    c.Bidi(current)?.0,
                    V::kEmbed
                        | V::kBidiOverride
                        | V::kWebkitIsolate
                        | V::kWebkitIsolateOverride
                        | V::kWebkitPlaintext
                        | V::kIsolate
                        | V::kIsolateOverride
                        | V::kPlaintext
                )
            {
                return Ok((None, true));
            }
            if current == end {
                break;
            }
            node = c.NextNode(current);
        }
    } else if let Some(direction) = c.TypingDirection()? {
        return Ok((direction, false));
    }
    let mut found = None;
    let mut node = Some(start);
    while let Some(current) = node {
        if c.IsBlock(current) {
            break;
        }
        if c.IsStyledElement(current) {
            let (bidi, direction) = c.Bidi(current)?;
            if bidi == V::kBidiOverride {
                return Ok((None, true));
            }
            if bidi != V::kNormal {
                if let Some(direction) = direction {
                    if found.is_some() {
                        return Ok((None, true));
                    }
                    if !caret {
                        let mut runner = c.Parent(end);
                        while runner.is_some_and(|runner| runner != current) {
                            runner = runner.and_then(|runner| c.Parent(runner));
                        }
                        if runner.is_none() {
                            return Ok((None, true));
                        }
                    }
                    found = Some(direction);
                }
            }
        }
        node = c.Parent(current);
    }
    Ok((found, false))
}
pub fn State<C: StyleCommandQueryContext>(c: &C, state: S) -> Option<EditResult<T>> {
    if matches!(state, S::StateOrderedList | S::StateUnorderedList) {
        return Some(Ok(ListState(c, state == S::StateOrderedList)));
    }
    let (property, value) = match state {
        S::StyleCommands_StateBold => (P::kFontWeight, V::kBold),
        S::StyleCommands_StateItalic => (P::kFontStyle, V::kItalic),
        S::StyleCommands_StateSubscript => (P::kVerticalAlign, V::kSub),
        S::StyleCommands_StateSuperscript => (P::kVerticalAlign, V::kSuper),
        S::StyleCommands_StateUnderline => (P::kWebkitTextDecorationsInEffect, V::kUnderline),
        S::StyleCommands_StateStrikethrough => (P::kWebkitTextDecorationsInEffect, V::kLineThrough),
        S::StateJustifyCenter => (P::kTextAlign, V::kCenter),
        S::StateJustifyFull => (P::kTextAlign, V::kJustify),
        S::StateJustifyLeft => (P::kTextAlign, V::kLeft),
        S::StateJustifyRight => (P::kTextAlign, V::kRight),
        S::StyleCommands_StateStyleWithCss => {
            return Some(Ok(if !c.HasActiveEditContext() && c.ShouldStyleWithCss() {
                T::kTrue
            } else {
                T::kFalse
            }))
        }
        S::StyleCommands_StateTextWritingDirectionLeftToRight
        | S::StyleCommands_StateTextWritingDirectionNatural
        | S::StyleCommands_StateTextWritingDirectionRightToLeft => {
            let desired = match state {
                S::StyleCommands_StateTextWritingDirectionLeftToRight => {
                    Some(foundation::TextDirection::kLtr)
                }
                S::StyleCommands_StateTextWritingDirectionRightToLeft => {
                    Some(foundation::TextDirection::kRtl)
                }
                _ => None,
            };
            return Some(if c.HasActiveEditContext() {
                Ok(T::kFalse)
            } else {
                TextDirection(c).map(|(actual, nested)| {
                    if actual == desired && !nested {
                        T::kTrue
                    } else {
                        T::kFalse
                    }
                })
            });
        }
        _ => return None,
    };
    Some(StateStyle(c, property, value))
}
pub fn Value<C: StyleCommandQueryContext>(c: &C, value: L) -> Option<EditResult<String>> {
    let property = match value {
        L::StyleCommands_ValueBackColor => P::kBackgroundColor,
        L::StyleCommands_ValueForeColor => P::kColor,
        L::StyleCommands_ValueFontName => P::kFontFamily,
        L::StyleCommands_ValueFontSize => P::kFontSize,
        L::StyleCommands_ValueFontSizeDelta => P::kInternalFontSizeDelta,
        L::ValueJustifyOrStateOrNull => P::kTextAlign,
        _ => return None,
    };
    Some((|| {
        if c.HasActiveEditContext() {
            return Ok(String::new());
        }
        let Some((start, _, _)) = c.Selection() else {
            return Ok(String::new());
        };
        let value = c.Value(start, property)?;
        if property == P::kTextAlign && matches!(value.as_str(), "start" | "end") {
            let ltr = c.Value(start, P::kDirection)? != "rtl";
            return Ok(if (value == "start") == ltr {
                "left"
            } else {
                "right"
            }
            .into());
        }
        Ok(value)
    })())
}

#[derive(Clone)]
pub enum StylePropertyValue {
    Identifier(V),
    Text(String),
    LegacyFontSize(String),
}
pub trait StyleCommandContext: ApplyStyleContext {
    fn CreateCommandStyle(
        &self,
        properties: Vec<(P, StylePropertyValue)>,
    ) -> EditResult<Option<Self::Style>>;
    fn CommandStyleAtStart(&self, node: u64) -> EditResult<Self::Style>;
    fn CommandStylePresent(
        &self,
        node: u64,
        desired: &Self::Style,
        include_typing: bool,
    ) -> EditResult<bool>;
    fn ToggleCommandList(
        &self,
        selected: Self::Style,
        property: P,
        value: V,
    ) -> EditResult<(Self::Style, bool)>;
    fn CommandTypingStyle(&self) -> Option<Self::Style>;
    fn ShouldToggleStyleBasedOnStartOfSelection(&self) -> bool;
    fn SetShouldStyleWithCss(&self, value: bool);
}
#[derive(Clone, Copy)]
pub struct StyleCommandOutcome {
    pub accepted: bool,
    pub applied: bool,
    pub input_type: I,
}
pub fn IsStyleCommand(command: F) -> bool {
    use F::*;
    matches!(
        command,
        StyleCommands_ExecuteBackColor
            | StyleCommands_ExecuteForeColor
            | StyleCommands_ExecuteFontName
            | StyleCommands_ExecuteFontSize
            | StyleCommands_ExecuteFontSizeDelta
            | StyleCommands_ExecuteToggleBold
            | StyleCommands_ExecuteToggleItalic
            | StyleCommands_ExecuteSubscript
            | StyleCommands_ExecuteSuperscript
            | StyleCommands_ExecuteUnscript
            | StyleCommands_ExecuteStrikethrough
            | StyleCommands_ExecuteUnderline
            | StyleCommands_ExecuteMakeTextWritingDirectionLeftToRight
            | StyleCommands_ExecuteMakeTextWritingDirectionNatural
            | StyleCommands_ExecuteMakeTextWritingDirectionRightToLeft
            | StyleCommands_ExecuteStyleWithCss
            | StyleCommands_ExecuteUseCss
    )
}
pub fn StyleWithCssValue(command: F, value: &str) -> Option<bool> {
    match command {
        F::StyleCommands_ExecuteStyleWithCss => Some(!value.eq_ignore_ascii_case("false")),
        F::StyleCommands_ExecuteUseCss => Some(value.eq_ignore_ascii_case("false")),
        _ => None,
    }
}
fn Style<C: StyleCommandContext>(c: &C, query: Q<C::Style>) -> EditResult<C::Style> {
    match c.ApplyStyleQuery(query)? {
        R::Style(style) => Ok(style),
        _ => Err(EditingAborted),
    }
}
fn Count<C: StyleCommandContext>(c: &C, style: &C::Style) -> EditResult<usize> {
    match c.ApplyStyleQuery(Q::Count(style.clone()))? {
        R::Count(n) => Ok(n),
        _ => Err(EditingAborted),
    }
}
fn SelectionHasStyle<C: StyleCommandContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
    desired: &C::Style,
) -> EditResult<bool> {
    if start == end || c.ShouldToggleStyleBasedOnStartOfSelection() {
        let style_start = StyleStart(c, start, end);
        return c.CommandStylePresent(
            *style_start.AnchorNode().ok_or(EditingAborted)?,
            desired,
            true,
        );
    }
    let first = *start.AnchorNode().ok_or(EditingAborted)?;
    // EditingStyle::TriStateOfStyle: element state establishes the initial
    // value; only a differing text node makes a range mixed.
    let mut state = None;
    let mut node = Some(first);
    while let Some(current) = node {
        if c.NodeFacts(current).has_layout && c.Editable(current) {
            let present = c.CommandStylePresent(current, desired, false)?;
            match state {
                None => state = Some(present),
                Some(previous) if previous != present && c.IsText(&current) => return Ok(false),
                _ => {}
            }
        }
        if end.AnchorNode() == Some(&current) {
            break;
        }
        node = NextNode(c, &current, None);
    }
    Ok(state.unwrap_or(false))
}
fn StyleStart<C: StyleCommandContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
) -> Position<u64> {
    if start != end
        && start.AnchorNode().is_some_and(|node| {
            c.IsText(node)
                && Some(start.ComputeOffsetInContainerNode(c)) == c.CharacterDataLength(node)
        })
    {
        c.ApplyDistinctCandidate(start, true)
    } else {
        start.clone()
    }
}
fn Apply<C: StyleCommandContext>(
    c: &C,
    mut style: C::Style,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult<bool> {
    if Count(c, &style)? == 0 {
        return Ok(false);
    }
    if start == end {
        if let Some(typing) = c.CommandTypingStyle() {
            style = Style(c, Q::Merge(typing, style))?;
        }
        let style = c.PrepareStyle(&style, start)?;
        let Some(style) = style else {
            c.SetTypingStyle(None);
            return Ok(false);
        };
        let R::Extract {
            extracted,
            remaining,
        } = c.ApplyStyleQuery(Q::Extract(style, StyleExtraction::Block))?
        else {
            return Err(EditingAborted);
        };
        c.SetTypingStyle(Some(remaining));
        if Count(c, &extracted)? == 0 {
            return Ok(false);
        }
        ApplyStyleRange(c, &extracted, start, end)?;
    } else {
        ApplyStyleRange(c, &style, start, end)?;
    }
    Ok(true)
}
pub fn Execute<C: StyleCommandContext>(
    c: &C,
    command: F,
    value: &str,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult<StyleCommandOutcome> {
    use StylePropertyValue::{Identifier as Id, Text};
    use F::*;
    let (properties, input_type) = match command {
        StyleCommands_ExecuteStyleWithCss | StyleCommands_ExecuteUseCss => {
            c.SetShouldStyleWithCss(StyleWithCssValue(command, value).unwrap());
            return Ok(StyleCommandOutcome {
                accepted: true,
                applied: false,
                input_type: I::kNone,
            });
        }
        StyleCommands_ExecuteBackColor => {
            (vec![(P::kBackgroundColor, Text(value.into()))], I::kNone)
        }
        StyleCommands_ExecuteForeColor => (vec![(P::kColor, Text(value.into()))], I::kNone),
        StyleCommands_ExecuteFontName => (vec![(P::kFontFamily, Text(value.into()))], I::kNone),
        StyleCommands_ExecuteFontSize => (
            vec![(
                P::kFontSize,
                StylePropertyValue::LegacyFontSize(value.into()),
            )],
            I::kNone,
        ),
        StyleCommands_ExecuteFontSizeDelta => (
            vec![(P::kInternalFontSizeDelta, Text(value.into()))],
            I::kNone,
        ),
        StyleCommands_ExecuteUnscript => (vec![(P::kVerticalAlign, Id(V::kBaseline))], I::kNone),
        StyleCommands_ExecuteMakeTextWritingDirectionLeftToRight => (
            vec![
                (P::kUnicodeBidi, Id(V::kIsolate)),
                (P::kDirection, Id(V::kLtr)),
            ],
            I::kFormatSetBlockTextDirection,
        ),
        StyleCommands_ExecuteMakeTextWritingDirectionRightToLeft => (
            vec![
                (P::kUnicodeBidi, Id(V::kIsolate)),
                (P::kDirection, Id(V::kRtl)),
            ],
            I::kFormatSetBlockTextDirection,
        ),
        StyleCommands_ExecuteMakeTextWritingDirectionNatural => (
            vec![(P::kUnicodeBidi, Id(V::kNormal))],
            I::kFormatSetBlockTextDirection,
        ),
        StyleCommands_ExecuteToggleBold
        | StyleCommands_ExecuteToggleItalic
        | StyleCommands_ExecuteSubscript
        | StyleCommands_ExecuteSuperscript => {
            let (property, off, on, input) = match command {
                StyleCommands_ExecuteToggleBold => {
                    (P::kFontWeight, V::kNormal, V::kBold, I::kFormatBold)
                }
                StyleCommands_ExecuteToggleItalic => {
                    (P::kFontStyle, V::kNormal, V::kItalic, I::kFormatItalic)
                }
                StyleCommands_ExecuteSubscript => (
                    P::kVerticalAlign,
                    V::kBaseline,
                    V::kSub,
                    I::kFormatSubscript,
                ),
                _ => (
                    P::kVerticalAlign,
                    V::kBaseline,
                    V::kSuper,
                    I::kFormatSuperscript,
                ),
            };
            let desired = c
                .CreateCommandStyle(vec![(property, Id(on))])?
                .ok_or(EditingAborted)?;
            let value = if SelectionHasStyle(c, start, end, &desired)? {
                off
            } else {
                on
            };
            (vec![(property, Id(value))], input)
        }
        StyleCommands_ExecuteStrikethrough | StyleCommands_ExecuteUnderline => {
            let (value, input) = if command == StyleCommands_ExecuteUnderline {
                (V::kUnderline, I::kFormatUnderline)
            } else {
                (V::kLineThrough, I::kFormatStrikeThrough)
            };
            let style_start = StyleStart(c, start, end);
            let selected =
                c.CommandStyleAtStart(*style_start.AnchorNode().ok_or(EditingAborted)?)?;
            let (style, is_none) =
                c.ToggleCommandList(selected, P::kWebkitTextDecorationsInEffect, value)?;
            if is_none && start == end {
                let mut element = start.AnchorNode().copied();
                while element.is_some_and(|node| !c.IsElement(node)) {
                    element = element.and_then(|node| c.Parent(&node));
                }
                if element.is_some_and(|node| c.ChildCount(&node) == 0) {
                    if let Some(typing) = c.CommandTypingStyle() {
                        let typing = Style(
                            c,
                            Q::RemoveProperties(
                                typing,
                                vec![P::kWebkitTextDecorationsInEffect, P::kTextDecoration],
                            ),
                        )?;
                        c.SetTypingStyle(Some(typing));
                        return Ok(StyleCommandOutcome {
                            accepted: true,
                            applied: false,
                            input_type: input,
                        });
                    }
                }
            }
            return Ok(StyleCommandOutcome {
                accepted: true,
                applied: Apply(c, style, start, end)?,
                input_type: input,
            });
        }
        _ => return Err(EditingAborted),
    };
    let Some(style) = c.CreateCommandStyle(properties)? else {
        return Ok(StyleCommandOutcome {
            accepted: false,
            applied: false,
            input_type,
        });
    };
    Ok(StyleCommandOutcome {
        accepted: true,
        applied: Apply(c, style, start, end)?,
        input_type,
    })
}
