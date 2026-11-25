/// Opaque Viz identity used by compositor input/browser-controls offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OffsetTag(u128);

impl OffsetTag {
    pub const fn FromToken(token: u128) -> Self {
        Self(token)
    }

    pub const fn IsEmpty(self) -> bool {
        self.0 == 0
    }

    pub const fn token(self) -> u128 {
        self.0
    }
}

