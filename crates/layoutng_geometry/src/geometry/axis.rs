// cpp: layoutng_geometry/geometry/axis.h:10-13
// Pending foundation writing-mode and String interfaces.
use foundation::{IsHorizontalWritingMode, String, WritingMode};

// cpp: layoutng_geometry/geometry/axis.h:17-18
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalAxis {
    kInline = 0b01,
    kBlock = 0b10,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalAxis {
    kHorizontal = 0b01,
    kVertical = 0b10,
}

// cpp: layoutng_geometry/geometry/axis.h:20-27
// StrongAlias uses a private value. #[repr(transparent)] retains the u8 layout.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicalAxes(u8);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LogicalAxes(u8);

impl PhysicalAxes {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
    pub const fn value(self) -> u8 {
        self.0
    }
    pub const fn is_nonzero(self) -> bool {
        self.0 != 0
    }
}
impl LogicalAxes {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
    pub const fn value(self) -> u8 {
        self.0
    }
    pub const fn is_nonzero(self) -> bool {
        self.0 != 0
    }
}

// cpp: layoutng_geometry/geometry/axis.h:29-36
impl std::ops::BitOr for LogicalAxes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.value() | rhs.value())
    }
}
impl std::ops::BitOrAssign for LogicalAxes {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:38-45
impl std::ops::BitAnd for LogicalAxes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.value() & rhs.value())
    }
}
impl std::ops::BitAndAssign for LogicalAxes {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:47-54
impl std::ops::BitXor for LogicalAxes {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.value() ^ rhs.value())
    }
}
impl std::ops::BitXorAssign for LogicalAxes {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:56-63
impl std::ops::Sub for LogicalAxes {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.value() & !rhs.value())
    }
}
impl std::ops::SubAssign for LogicalAxes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:65-72
impl std::ops::BitOr for PhysicalAxes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.value() | rhs.value())
    }
}
impl std::ops::BitOrAssign for PhysicalAxes {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:74-81
impl std::ops::BitAnd for PhysicalAxes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.value() & rhs.value())
    }
}
impl std::ops::BitAndAssign for PhysicalAxes {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:83-90
impl std::ops::BitXor for PhysicalAxes {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.value() ^ rhs.value())
    }
}
impl std::ops::BitXorAssign for PhysicalAxes {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:92-99
impl std::ops::Sub for PhysicalAxes {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.value() & !rhs.value())
    }
}
impl std::ops::SubAssign for PhysicalAxes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.value();
    }
}

// cpp: layoutng_geometry/geometry/axis.h:101-115
// Constant trait operators are unavailable in stable Rust; each value is the
// same u8 expression as the C++ constexpr constant.
#[allow(non_upper_case_globals)]
pub const kLogicalAxesNone: LogicalAxes = LogicalAxes::new(0);
#[allow(non_upper_case_globals)]
pub const kLogicalAxesInline: LogicalAxes = LogicalAxes::new(LogicalAxis::kInline as u8);
#[allow(non_upper_case_globals)]
pub const kLogicalAxesBlock: LogicalAxes = LogicalAxes::new(LogicalAxis::kBlock as u8);
#[allow(non_upper_case_globals)]
pub const kLogicalAxesBoth: LogicalAxes = LogicalAxes::new(0b11);
#[allow(non_upper_case_globals)]
pub const kPhysicalAxesNone: PhysicalAxes = PhysicalAxes::new(0);
#[allow(non_upper_case_globals)]
pub const kPhysicalAxesHorizontal: PhysicalAxes =
    PhysicalAxes::new(PhysicalAxis::kHorizontal as u8);
#[allow(non_upper_case_globals)]
pub const kPhysicalAxesVertical: PhysicalAxes = PhysicalAxes::new(PhysicalAxis::kVertical as u8);
#[allow(non_upper_case_globals)]
pub const kPhysicalAxesBoth: PhysicalAxes = PhysicalAxes::new(0b11);

// cpp: layoutng_geometry/geometry/axis.h:117-123
const _: () = {
    assert!(kLogicalAxesNone.value() == kPhysicalAxesNone.value());
    assert!(kLogicalAxesInline.value() == kPhysicalAxesHorizontal.value());
    assert!(kLogicalAxesBlock.value() == kPhysicalAxesVertical.value());
    assert!(kLogicalAxesBoth.value() == kPhysicalAxesBoth.value());
};

// cpp: layoutng_geometry/geometry/axis.h:125-130
pub trait AxisBits: Copy {
    fn from_bits(bits: u8) -> Self;
    fn value(self) -> u8;
}
impl AxisBits for LogicalAxes {
    fn from_bits(bits: u8) -> Self {
        Self::new(bits)
    }
    fn value(self) -> u8 {
        self.0
    }
}
impl AxisBits for PhysicalAxes {
    fn from_bits(bits: u8) -> Self {
        Self::new(bits)
    }
    fn value(self) -> u8 {
        self.0
    }
}
#[allow(non_snake_case)]
pub fn ConvertAxes<FromType: AxisBits, ToType: AxisBits>(
    from: FromType,
    mode: WritingMode,
) -> ToType {
    let shift = if IsHorizontalWritingMode(mode) { 0 } else { 1 };
    ToType::from_bits(((from.value() >> shift) & 1) | ((from.value() << shift) & 2))
}

// cpp: layoutng_geometry/geometry/axis.h:132-138
#[allow(non_snake_case)]
pub fn ToPhysicalAxes(logical: LogicalAxes, mode: WritingMode) -> PhysicalAxes {
    ConvertAxes::<LogicalAxes, PhysicalAxes>(logical, mode)
}
#[allow(non_snake_case)]
pub fn ToLogicalAxes(physical: PhysicalAxes, mode: WritingMode) -> LogicalAxes {
    ConvertAxes::<PhysicalAxes, LogicalAxes>(physical, mode)
}

// cpp: layoutng_geometry/geometry/axis.h:140-141
// Both C++ ToString overloads are declarations without definitions in the
// supplied source tree. Preserve them as unresolved external Rust-ABI symbols.
extern "Rust" {
    pub fn ToStringLogicalAxes(axes: LogicalAxes) -> String;
    pub fn ToStringPhysicalAxes(axes: PhysicalAxes) -> String;
}

// cpp: layoutng_geometry/geometry/axis.h:143-144
// The two ostream insertion declarations likewise have no supplied C++
// definitions. No Display implementation is invented from those declarations.
