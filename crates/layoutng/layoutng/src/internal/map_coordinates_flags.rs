#![allow(non_upper_case_globals)]

// C++ EnumSet indexes bits by enum value. The minimum and maximum aliases
// remain associated constants because Rust variants cannot share a value.
// cpp: layoutng/internal/map_coordinates_flags.h:45-86
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MapCoordinatesMode {
    kIgnoreTransforms = 0,
    kTraverseDocumentBoundaries = 1,
    kIgnoreStickyOffset = 2,
    kIgnoreScrollOffset = 3,
    kIgnoreScrollOriginAndOffset = 4,
    kApplyRemoteMainFrameTransform = 5,
    kApplyRemoteViewportTransform = 6,
}

impl MapCoordinatesMode {
    pub const kMinValue: Self = Self::kIgnoreTransforms;
    pub const kMaxValue: Self = Self::kApplyRemoteViewportTransform;
}

// cpp: layoutng/internal/map_coordinates_flags.h:88-88
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MapCoordinatesFlags(u8);

impl MapCoordinatesFlags {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_mode(mode: MapCoordinatesMode) -> Self {
        Self(1 << mode as u8)
    }

    pub const fn contains(self, mode: MapCoordinatesMode) -> bool {
        self.0 & (1 << mode as u8) != 0
    }

    pub fn insert(&mut self, mode: MapCoordinatesMode) {
        self.0 |= 1 << mode as u8;
    }

    pub fn remove(&mut self, mode: MapCoordinatesMode) {
        self.0 &= !(1 << mode as u8);
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl From<MapCoordinatesMode> for MapCoordinatesFlags {
    fn from(mode: MapCoordinatesMode) -> Self {
        Self::from_mode(mode)
    }
}

impl std::ops::BitOr for MapCoordinatesFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MapCoordinatesFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for MapCoordinatesFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
