//! Minimal live DOM/native style facts consumed by the source FindBuffer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FindHTMLClass {
    Other,
    Input,
    IFrame,
    Image,
    Meter,
    Object,
    Progress,
    Select,
    Style,
    Script,
    Video,
    Audio,
    WBR,
    BR,
}
#[derive(Clone, Copy)]
pub struct FindBufferNodeFacts {
    pub is_element: bool,
    pub comment: bool,
    pub html_class: Option<FindHTMLClass>,
    pub serializes_end_tag: bool,
    pub text_control: bool,
    pub suggested_value_nonempty: bool,
    pub select_uses_menu_list: bool,
    pub locked: bool,
    pub find_activatable: bool,
}

#[derive(Clone, Copy)]
pub struct FindStyleFacts {
    pub display_none: bool,
    pub visible: bool,
    pub inert: bool,
    pub collapse_spaces: bool,
    pub ensured: bool,
}
