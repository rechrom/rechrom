#![allow(non_snake_case)]

use foundation::UChar;

use super::html_entity_table::{HTMLEntityTable, HTMLEntityTableEntry};

// cpp: html/parser/html_entity_search.h:36-58
pub struct HTMLEntitySearch {
    current_length_: u16,
    most_recent_match_: Option<&'static HTMLEntityTableEntry>,
    range_: &'static [HTMLEntityTableEntry],
}

impl Default for HTMLEntitySearch {
    // cpp: html/parser/html_entity_search.h:40
    // cpp: html/parser/html_entity_search.cc:33
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLEntitySearch {
    // cpp: html/parser/html_entity_search.h:40
    // cpp: html/parser/html_entity_search.cc:33
    pub fn new() -> Self {
        Self {
            current_length_: 0,
            most_recent_match_: None,
            range_: HTMLEntityTable::AllEntries(),
        }
    }

    // cpp: html/parser/html_entity_search.h:42
    // cpp: html/parser/html_entity_search.cc:35-62
    pub fn Advance(&mut self, next_character: UChar) {
        debug_assert!(self.IsEntityPrefix());
        if self.current_length_ == 0 {
            self.range_ = HTMLEntityTable::EntriesStartingWith(next_character);
        } else {
            // cpp: html/parser/html_entity_search.cc:40-52
            let current_length = self.current_length_;
            let projector = |entry: &HTMLEntityTableEntry| -> Option<UChar> {
                if entry.length < current_length + 1 {
                    return None;
                }
                let entity_string = HTMLEntityTable::EntityString(entry);
                Some(entity_string[current_length as usize] as UChar)
            };
            let target = Some(next_character);
            let range = self.range_;
            let first = range.partition_point(|entry| projector(entry) < target);
            let last = range.partition_point(|entry| projector(entry) <= target);
            self.range_ = &range[first..last];
        }
        if self.range_.is_empty() {
            return self.Fail();
        }
        self.current_length_ += 1;
        if self.range_[0].length != self.current_length_ {
            return;
        }
        self.most_recent_match_ = Some(&self.range_[0]);
    }

    // cpp: html/parser/html_entity_search.h:44
    pub fn IsEntityPrefix(&self) -> bool {
        !self.range_.is_empty()
    }

    // cpp: html/parser/html_entity_search.h:45
    pub fn CurrentLength(&self) -> u16 {
        self.current_length_
    }

    // cpp: html/parser/html_entity_search.h:47-49
    pub fn MostRecentMatch(&self) -> Option<&'static HTMLEntityTableEntry> {
        self.most_recent_match_
    }

    // cpp: html/parser/html_entity_search.h:52
    fn Fail(&mut self) {
        self.range_ = &[];
    }
}
