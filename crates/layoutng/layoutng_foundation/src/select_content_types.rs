//! Typed native class facts and inspector effects used by select's content
//! observer. These values carry identities, never a second subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectContentNodeKind {
    Text,
    Comment,
    HTML,
    SVG,
    Other,
    Select,
    Option,
    SelectedContent,
    OptGroup,
    Button,
    Div,
    Span,
    NoScript,
    Script,
    Template,
    Legend,
    HR,
    DataList,
    Object,
}
impl SelectContentNodeKind {
    pub fn IsHTML(self) -> bool {
        !matches!(self, Self::Text | Self::Comment | Self::SVG | Self::Other)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectAccessibilityIssueReason {
    ValidChild,
    DisallowedSelectChild,
    DisallowedOptGroupChild,
    NonPhrasingContentOptionChild,
    InteractiveContentOptionChild,
    InteractiveContentLegendChild,
}
