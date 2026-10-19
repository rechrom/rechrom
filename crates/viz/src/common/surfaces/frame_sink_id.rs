// Source: components/viz/common/surfaces/frame_sink_id.h:1-86
// Source: components/viz/common/surfaces/frame_sink_id.cc:1-36
// Includes, export decoration, ostream and Perfetto adapters become Rust's
// standard traits and the embedding trace boundary. The complete value
// semantics and string forms remain here.
#![allow(non_snake_case)]

use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameSinkId {
    client_id_: u32,
    sink_id_: u32,
}

impl FrameSinkId {
    pub const fn new(client_id: u32, sink_id: u32) -> Self {
        Self {
            client_id_: client_id,
            sink_id_: sink_id,
        }
    }
    pub const fn is_valid(self) -> bool {
        self.client_id_ != 0 || self.sink_id_ != 0
    }
    pub const fn client_id(self) -> u32 {
        self.client_id_
    }
    pub const fn sink_id(self) -> u32 {
        self.sink_id_
    }
    pub fn hash(self) -> u64 {
        let mut state = std::collections::hash_map::DefaultHasher::new();
        Hash::hash(&self, &mut state);
        state.finish()
    }
    pub fn ToString(self) -> String {
        format!("{}_{}", self.client_id_, self.sink_id_)
    }
    pub fn ToStringWithLabel(self, debug_label: &str) -> String {
        if debug_label.is_empty() {
            self.ToString()
        } else {
            format!("{} ({debug_label})", self.ToString())
        }
    }
}

impl fmt::Display for FrameSinkId {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.ToString())
    }
}
