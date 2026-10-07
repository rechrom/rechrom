// C++: foundation/blink_base/wtf/text/text_offset_map.h/.cc
use crate::WtfSizeT;

// cpp: foundation/blink_base/wtf/text/text_offset_map.h:27-37
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextOffsetMapEntry {
    pub source: WtfSizeT,
    pub target: WtfSizeT,
}

// cpp: foundation/blink_base/wtf/text/text_offset_map.h:25-74
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextOffsetMap {
    entries_: Vec<TextOffsetMapEntry>,
}

#[allow(non_snake_case)]
impl TextOffsetMap {
    // cpp: foundation/blink_base/wtf/text/text_offset_map.cc:32-74
    pub fn new(
        length1: WtfSizeT,
        map12: &Self,
        length2: WtfSizeT,
        map23: &Self,
        length3: WtfSizeT,
    ) -> Self {
        if map12.IsEmpty() {
            return map23.clone();
        }
        if map23.IsEmpty() {
            return map12.clone();
        }

        let length_map12 = map12.CreateLengthMap(length1, length2);
        let mut length_map = map23.CreateLengthMap(length2, length3);
        let mut index12 = 0;
        for index23 in 0..length_map.len() {
            let length = length_map[index23];
            let mut sum = 0u32;
            let is_starting_at_middle = length > 0 && length_map12[index12] == 0;
            for _ in 0..length {
                sum += length_map12[index12];
                index12 += 1;
            }
            length_map[index23] = sum;
            if is_starting_at_middle {
                length_map[index23] = 0;
                length_map[index23 - 1] += sum;
            }
        }

        let mut result = Self::default();
        let mut source_index = 0u32;
        let mut dest_index = 0;
        while dest_index < length_map.len() {
            let source_length = length_map[dest_index];
            source_index += source_length;
            let mut dest_length = 1u32;
            if source_length == 1 {
                while dest_index + 1 < length_map.len() && length_map[dest_index + 1] == 0 {
                    dest_index += 1;
                    dest_length += 1;
                }
            }
            if source_length != dest_length {
                result.Append(source_index, (dest_index + 1) as u32);
            }
            dest_index += 1;
        }
        result
    }

    // cpp: foundation/blink_base/wtf/text/text_offset_map.h:52-56
    pub fn IsEmpty(&self) -> bool {
        self.entries_.is_empty()
    }
    pub fn Entries(&self) -> &[TextOffsetMapEntry] {
        &self.entries_
    }
    pub fn Clear(&mut self) {
        self.entries_.clear();
    }

    // cpp: foundation/blink_base/wtf/text/text_offset_map.cc:76-80
    pub fn Append(&mut self, source: WtfSizeT, target: WtfSizeT) {
        assert!(self
            .entries_
            .last()
            .is_none_or(|entry| source >= entry.source && target >= entry.target));
        self.entries_.push(TextOffsetMapEntry { source, target });
    }

    // cpp: foundation/blink_base/wtf/text/text_offset_map.cc:84-129
    pub fn CreateLengthMap(&self, old_length: WtfSizeT, new_length: WtfSizeT) -> Vec<u32> {
        let mut map = Vec::new();
        if self.IsEmpty() {
            return map;
        }
        map.reserve(new_length as usize);
        let mut old_offset = 0;
        let mut new_offset = 0;
        for entry in self.Entries() {
            let old_chunk_length = entry.source - old_offset;
            let new_chunk_length = entry.target - new_offset;
            if old_chunk_length < new_chunk_length {
                for _ in 0..old_chunk_length {
                    map.push(1);
                }
                for _ in old_chunk_length..new_chunk_length {
                    map.push(0);
                }
            } else if old_chunk_length > new_chunk_length {
                assert!(new_chunk_length >= 1);
                for _ in 0..new_chunk_length - 1 {
                    map.push(1);
                }
                map.push(1 + old_chunk_length - new_chunk_length);
            } else {
                for _ in 0..new_chunk_length {
                    map.push(1);
                }
            }
            old_offset = entry.source;
            new_offset = entry.target;
        }
        debug_assert_eq!(old_length - old_offset, new_length - new_offset);
        for _ in new_offset..new_length {
            map.push(1);
        }
        debug_assert_eq!(map.len(), new_length as usize);
        map
    }

    // cpp: foundation/blink_base/wtf/text/text_offset_map.cc:131-142
    pub fn MapOffset(&self, offset: WtfSizeT) -> WtfSizeT {
        let mut last_entry = None;
        for entry in &self.entries_ {
            if entry.source > offset {
                break;
            }
            last_entry = Some(entry);
        }
        last_entry.map_or(offset, |entry| offset + entry.target - entry.source)
    }

    // cpp: foundation/blink_base/wtf/text/text_offset_map.cc:144-155
    pub fn InverseMapOffset(&self, offset: WtfSizeT) -> WtfSizeT {
        let mut last_entry = None;
        for entry in &self.entries_ {
            if entry.target > offset {
                break;
            }
            last_entry = Some(entry);
        }
        last_entry.map_or(offset, |entry| offset + entry.source - entry.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_deletion_and_composition() {
        let mut insertion = TextOffsetMap::default();
        insertion.Append(3, 4);
        assert_eq!(insertion.CreateLengthMap(5, 6), [1, 1, 1, 0, 1, 1]);
        assert_eq!(insertion.MapOffset(3), 4);
        assert_eq!(insertion.InverseMapOffset(4), 3);

        let mut deletion = TextOffsetMap::default();
        deletion.Append(4, 3);
        assert_eq!(deletion.CreateLengthMap(5, 4), [1, 1, 2, 1]);
        assert_eq!(deletion.MapOffset(4), 3);
        assert_eq!(deletion.InverseMapOffset(3), 4);

        let mut second = TextOffsetMap::default();
        second.Append(4, 5);
        let composed = TextOffsetMap::new(5, &insertion, 6, &second, 7);
        assert_eq!(composed.MapOffset(3), 5);
        assert_eq!(composed.MapOffset(5), 7);
    }
}
