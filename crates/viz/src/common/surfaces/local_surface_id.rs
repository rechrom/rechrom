// Source: components/viz/common/surfaces/local_surface_id.h:1-34
// Source: components/viz/common/surfaces/local_surface_id.cc:1-16
// C++ includes, export macros and namespaces map to Rust module/import scope.
// Mojo and Perfetto serialization remain outer service adapters.
#![allow(non_snake_case)]

use std::fmt;
use std::hash::{Hash, Hasher};

// Source: components/viz/common/surfaces/local_surface_id.h:35-72
pub const kInvalidParentSequenceNumber: u32 = 0;
pub const kInvalidChildSequenceNumber: u32 = 0;
pub const kInitialParentSequenceNumber: u32 = 1;
pub const kInitialChildSequenceNumber: u32 = 1;
pub const kMaxParentSequenceNumber: u32 = u32::MAX;
pub const kMaxChildSequenceNumber: u32 = u32::MAX;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmbedToken {
    pub high: u64,
    pub low: u64,
}
impl EmbedToken {
    pub const fn new(high: u64, low: u64) -> Self {
        Self { high, low }
    }
    pub const fn is_empty(self) -> bool {
        self.high == 0 && self.low == 0
    }
}

// Source: components/viz/common/surfaces/local_surface_id.h:73-99
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSurfaceId {
    parent_sequence_number_: u32,
    child_sequence_number_: u32,
    embed_token_: EmbedToken,
}

impl LocalSurfaceId {
    pub const fn new(parent: u32, child: u32, token: EmbedToken) -> Self {
        Self {
            parent_sequence_number_: parent,
            child_sequence_number_: child,
            embed_token_: token,
        }
    }
    pub const fn MaxSequenceId() -> Self {
        Self::new(
            kMaxParentSequenceNumber,
            kMaxChildSequenceNumber,
            EmbedToken::new(0, 0),
        )
    }

    // Source: components/viz/common/surfaces/local_surface_id.h:100-125
    pub const fn is_valid(self) -> bool {
        self.parent_sequence_number_ != kInvalidParentSequenceNumber
            && self.child_sequence_number_ != kInvalidChildSequenceNumber
            && !self.embed_token_.is_empty()
    }
    pub const fn parent_sequence_number(self) -> u32 {
        self.parent_sequence_number_
    }
    pub const fn child_sequence_number(self) -> u32 {
        self.child_sequence_number_
    }
    pub const fn embed_token(self) -> EmbedToken {
        self.embed_token_
    }

    // Source: components/viz/common/surfaces/local_surface_id.h:126-139
    // Source: components/viz/common/surfaces/local_surface_id.cc:17-46
    pub fn hash(self) -> u64 {
        let mut state = std::collections::hash_map::DefaultHasher::new();
        Hash::hash(&self, &mut state);
        state.finish()
    }
    pub fn persistent_hash(self) -> u64 {
        // Chromium uses PersistentHash over the stable serialized tuple. FNV-1a
        // supplies the same required cross-process stability at this boundary.
        let mut value = 0xcbf29ce484222325u64;
        for byte in self
            .embed_token_
            .high
            .to_le_bytes()
            .into_iter()
            .chain(self.embed_token_.low.to_le_bytes())
            .chain(self.parent_sequence_number_.to_le_bytes())
            .chain(self.child_sequence_number_.to_le_bytes())
        {
            value ^= u64::from(byte);
            value = value.wrapping_mul(0x100000001b3);
        }
        value
    }
    pub fn embed_trace_id(self) -> u64 {
        self.persistent_hash() << 1
    }
    pub fn submission_trace_id(self) -> u64 {
        (self.persistent_hash() << 1) | 1
    }

    // Source: components/viz/common/surfaces/local_surface_id.h:140-162
    // Source: components/viz/common/surfaces/local_surface_id.cc:47-75
    pub fn IsNewerThan(self, other: Self) -> bool {
        self.embed_token_ == other.embed_token_ && self.IsNewerThanIgnoringEmbedToken(other)
    }
    pub fn IsNewerThanIgnoringEmbedToken(self, other: Self) -> bool {
        self.child_sequence_number_
            .wrapping_sub(other.child_sequence_number_)
            < (1u32 << 31)
            && self
                .parent_sequence_number_
                .wrapping_sub(other.parent_sequence_number_)
                < (1u32 << 31)
            && (self.child_sequence_number_ != other.child_sequence_number_
                || self.parent_sequence_number_ != other.parent_sequence_number_)
    }
    pub fn IsNewerThanOrEmbeddingChanged(self, other: Self) -> bool {
        self.IsNewerThan(other) || self.embed_token_ != other.embed_token_
    }
    pub fn IsSameOrNewerThan(self, other: Self) -> bool {
        self == other || self.IsNewerThan(other)
    }
    pub fn ToSmallestId(self) -> Self {
        Self::new(1, 1, self.embed_token_)
    }

    // Source: components/viz/common/surfaces/local_surface_id.cc:32-45
    pub fn ToString(self) -> String {
        format!(
            "LocalSurfaceId({}, {}, {:04x}...)",
            self.parent_sequence_number_,
            self.child_sequence_number_,
            self.embed_token_.high >> 48
        )
    }
}

// Source: components/viz/common/surfaces/local_surface_id.h:163-184
// Source: components/viz/common/surfaces/local_surface_id.cc:76-86
// Trace serialization is represented by the stable token/sequence accessors;
// Display provides the source ostream behavior.
impl fmt::Display for LocalSurfaceId {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.ToString())
    }
}
