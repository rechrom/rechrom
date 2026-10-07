// cpp: font_engine/fonts/font_vertical_position_type.h:11-23
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontVerticalPositionType {
    TextTop,
    TextBottom,
    TopOfEmHeight,
    BottomOfEmHeight,
}

// cpp: font_engine/fonts/font_vertical_position_type.h:25-32
pub fn IsLineOverSide(position_type: FontVerticalPositionType) -> bool {
    position_type == FontVerticalPositionType::TextTop
        || position_type == FontVerticalPositionType::TopOfEmHeight
}
