use super::computed_style_constants::PseudoId;
use super::content_data::ContentData;
use foundation::EDisplay;

// cpp: layoutng_style/style/display_style.h:15-42
/// A borrowed view of the subset needed to decide whether a layout box exists.
pub struct DisplayStyle<'a> {
    display_: EDisplay,
    style_type_: PseudoId,
    content_data_: Option<&'a dyn ContentData>,
}

#[allow(non_snake_case)]
impl<'a> DisplayStyle<'a> {
    // cpp: layoutng_style/style/display_style.h:23-28
    pub const fn new(
        display: EDisplay,
        style_type: PseudoId,
        content_data: Option<&'a dyn ContentData>,
    ) -> Self {
        Self {
            display_: display,
            style_type_: style_type,
            content_data_: content_data,
        }
    }

    // cpp: layoutng_style/style/display_style.h:30-32
    pub fn Display(&self) -> EDisplay {
        self.display_
    }
    pub fn StyleType(&self) -> PseudoId {
        self.style_type_
    }
    pub fn GetContentData(&self) -> Option<&'a dyn ContentData> {
        self.content_data_
    }

    // cpp: layoutng_style/style/display_style.cc:8-15
    pub fn ContentBehavesAsNormal(&self) -> bool {
        if self.style_type_ == PseudoId::kPseudoIdMarker {
            return self.content_data_.is_none();
        }
        match self.content_data_ {
            None => true,
            Some(content_data) => content_data.IsNone(),
        }
    }

    // cpp: layoutng_style/style/display_style.cc:16-34
    pub fn ContentPreventsBoxGeneration(&self) -> bool {
        if self.style_type_ == PseudoId::kPseudoIdCheckMark
            || self.style_type_ == PseudoId::kPseudoIdBefore
            || self.style_type_ == PseudoId::kPseudoIdAfter
            || self.style_type_ == PseudoId::kPseudoIdExpandIcon
            || self.style_type_ == PseudoId::kPseudoIdPickerIcon
            || self.style_type_ == PseudoId::kPseudoIdInterestButton
            || self.style_type_ == PseudoId::kPseudoIdScrollButtonBlockStart
            || self.style_type_ == PseudoId::kPseudoIdScrollButtonInlineStart
            || self.style_type_ == PseudoId::kPseudoIdScrollButtonInlineEnd
            || self.style_type_ == PseudoId::kPseudoIdScrollButtonBlockEnd
        {
            return self.ContentBehavesAsNormal();
        }
        if self.style_type_ == PseudoId::kPseudoIdMarker {
            return match self.content_data_ {
                Some(content_data) => content_data.IsNone(),
                None => false,
            };
        }
        false
    }
}
