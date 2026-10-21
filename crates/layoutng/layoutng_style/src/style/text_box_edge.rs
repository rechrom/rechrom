// cpp: layoutng_style/style/text_box_edge.h:21-34
// C++ enum class with u8 underlying type permits round-tripping even reserved
// bit patterns. A transparent newtype preserves those values in Rust.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextBoxEdgeType(u8);

#[allow(non_upper_case_globals)]
impl TextBoxEdgeType {
    pub const kAuto: Self = Self(0);
    pub const kText: Self = Self(1);
    pub const kCap: Self = Self(2);
    pub const kEx: Self = Self(3);
    pub const kAlphabetic: Self = Self(4);
    pub const fn value(self) -> u8 {
        self.0
    }
    pub const fn from_bits(value: u8) -> Self {
        Self(value)
    }
}

// cpp: layoutng_style/style/text_box_edge.h:36-39
pub const K_TYPE_BITS: u32 = 3;
pub const K_TYPE_MASK: u32 = (1 << K_TYPE_BITS) - 1;
pub const K_BITS: u32 = K_TYPE_BITS * 2;

// cpp: layoutng_style/style/text_box_edge.h:41-46
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextBoxEdge {
    over_: TextBoxEdgeType,
    under_: TextBoxEdgeType,
}

#[allow(non_snake_case)]
impl TextBoxEdge {
    pub fn from_over(over: TextBoxEdgeType) -> Self {
        Self::new(over, Self::UnderForOverChecked(over))
    }

    pub fn new(over: TextBoxEdgeType, under: TextBoxEdgeType) -> Self {
        debug_assert!(if over == TextBoxEdgeType::kAuto {
            under == TextBoxEdgeType::kAuto
        } else {
            under != TextBoxEdgeType::kAuto
        });
        Self {
            over_: over,
            under_: under,
        }
    }

    // cpp: layoutng_style/style/text_box_edge.h:48-55
    pub fn from_bits(value: u32) -> Self {
        Self::new(
            TextBoxEdgeType::from_bits((value & K_TYPE_MASK) as u8),
            TextBoxEdgeType::from_bits(((value >> K_TYPE_BITS) & K_TYPE_MASK) as u8),
        )
    }

    pub fn to_bits(&self) -> u32 {
        self.over_.value() as u32 | ((self.under_.value() as u32) << K_TYPE_BITS)
    }

    // cpp: layoutng_style/style/text_box_edge.h:57-64
    // Fieldwise equality is derived above.
    pub fn Over(&self) -> &TextBoxEdgeType {
        &self.over_
    }
    pub fn Under(&self) -> &TextBoxEdgeType {
        &self.under_
    }
    pub fn IsAuto(&self) -> bool {
        *self.Over() == TextBoxEdgeType::kAuto
    }

    // cpp: layoutng_style/style/text_box_edge.h:66-70
    pub fn IsUnderDefault(&self) -> bool {
        Some(*self.Under()) == Self::UnderForOver(*self.Over())
    }

    // cpp: layoutng_style/style/text_box_edge.h:82-98
    pub fn UnderForOver(over: TextBoxEdgeType) -> Option<TextBoxEdgeType> {
        match over.value() {
            0 | 1 => Some(over),
            2 | 3 => None,
            4 => unreachable!("C++ NOTREACHED: alphabetic over edge"),
            _ => unreachable!("C++ NOTREACHED: invalid text-box edge type"),
        }
    }

    // cpp: layoutng_style/style/text_box_edge.h:73
    // cpp: layoutng_style/style/text_box_edge.h:100-104
    fn UnderForOverChecked(over: TextBoxEdgeType) -> TextBoxEdgeType {
        let under = Self::UnderForOver(over);
        assert!(under.is_some(), "C++ CHECK: under value required");
        under.unwrap()
    }
}

// cpp: layoutng_style/style/text_box_edge.h:75-80
const _: () = assert!(TextBoxEdgeType::kAuto.value() == 0);
