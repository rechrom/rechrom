// Source: base/synchronization/lock_metrics_recorder_tags.h:41-80
#[derive(Clone, Copy)]
pub struct LockMetricTag {
    name_: &'static str,
    hash_: u64,
}
impl LockMetricTag {
    // Rust const fn cannot prohibit runtime invocation; consumers here use static const-evaluated tags.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name_: name,
            hash_: Self::HashName(name),
        }
    }
    pub const fn name(&self) -> &'static str {
        self.name_
    }
    pub const fn hash(&self) -> u64 {
        self.hash_
    }
    const fn empty() -> Self {
        Self {
            name_: "",
            hash_: 0,
        }
    }
    const fn HashName(name: &str) -> u64 {
        let mut hash = 14695981039346656037u64;
        let mut i = 0;
        while i < name.len() {
            hash ^= name.as_bytes()[i] as u64;
            hash = hash.wrapping_mul(1099511628211);
            i += 1;
        }
        hash
    }
}
// Source: base/synchronization/lock_metrics_recorder_tags.h:95-140
#[derive(Clone, Copy)]
pub struct LockMetricTagList {
    tags_: [LockMetricTag; 3],
    size_: u8,
}
impl LockMetricTagList {
    pub const kMaxTags: usize = 3;
    pub const fn new(tag1: &LockMetricTag) -> Self {
        Self {
            tags_: [*tag1, LockMetricTag::empty(), LockMetricTag::empty()],
            size_: 1,
        }
    }
    pub const fn new2(tag1: &LockMetricTag, tag2: &LockMetricTag) -> Self {
        Self {
            tags_: [*tag1, *tag2, LockMetricTag::empty()],
            size_: 2,
        }
    }
    pub const fn new3(tag1: &LockMetricTag, tag2: &LockMetricTag, tag3: &LockMetricTag) -> Self {
        Self {
            tags_: [*tag1, *tag2, *tag3],
            size_: 3,
        }
    }
    pub const fn size(&self) -> usize {
        self.size_ as usize
    }
    pub(crate) const fn empty() -> Self {
        Self {
            tags_: [LockMetricTag::empty(); 3],
            size_: 0,
        }
    }
    // Native index < kMaxTags DCHECK has a separate pending base/check_op dependency.
    pub(crate) fn get(&self, index: usize) -> Option<&LockMetricTag> {
        if index >= self.size() {
            return None;
        }
        Some(&self.tags_[index])
    }
}
