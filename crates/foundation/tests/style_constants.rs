use foundation::{
    kETextTransformBits, kTextDecorationLineBits, kTextDecorationSkipSpacesBits, EDisplay,
    ETextTransform, TextDecorationLine, TextDecorationSkipSpaces,
};

#[test]
fn style_enum_discriminants_and_max_alias_match_cpp() {
    assert_eq!(EDisplay::kInline as u8, 0);
    assert_eq!(EDisplay::kBlock as u8, 1);
    assert_eq!(EDisplay::kMaxEnumValue, EDisplay::kInlineGridLanes);
}

#[test]
fn style_flags_preserve_unnamed_combinations_and_unsigned_not() {
    assert_eq!(kETextTransformBits, 6);
    assert_eq!(kTextDecorationLineBits, 6);
    assert_eq!(kTextDecorationSkipSpacesBits, 3);

    let transform = ETextTransform::kUppercase | ETextTransform::kFullWidth;
    assert_eq!(transform.bits(), 10);
    assert_eq!((transform ^ ETextTransform::kUppercase).bits(), 8);
    assert_eq!((transform & ETextTransform::kUppercase).bits(), 2);
    assert_eq!((!ETextTransform::kNone).bits(), u32::MAX);

    let decorations = TextDecorationLine::kUnderline | TextDecorationLine::kOverline;
    assert_eq!(decorations.bits(), 3);
    assert_eq!(TextDecorationSkipSpaces::from_bits(3).bits(), 3);
}
