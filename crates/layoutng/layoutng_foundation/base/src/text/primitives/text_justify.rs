// C++: src/foundation/text/source/text_justify.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextJustify {
    kAuto,
    kNone,
    kInterCharacter,
    kInterWord,
}
