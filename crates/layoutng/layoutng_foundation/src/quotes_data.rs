use crate::{ScopedRefPtr, String};

// cpp: foundation/text/native/quotes_data.h:34-59
// Definitions: third_party/blink/renderer/platform/text/quotes_data.cc:26-58.
#[derive(Default)]
pub struct QuotesData {
    quote_pairs_: Vec<(String, String)>,
}

#[allow(non_snake_case)]
impl QuotesData {
    // cpp: foundation/text/native/quotes_data.h:38-40
    pub fn Create() -> ScopedRefPtr<Self> {
        ScopedRefPtr::new(Self::default())
    }

    // cpp: foundation/text/native/quotes_data.h:41-44
    pub fn CreateWithCharacters(
        open1: u16,
        close1: u16,
        open2: u16,
        close2: u16,
    ) -> ScopedRefPtr<Self> {
        let mut data = Self::default();
        data.AddPair((String::from_utf16(&[open1]), String::from_utf16(&[close1])));
        data.AddPair((String::from_utf16(&[open2]), String::from_utf16(&[close2])));
        ScopedRefPtr::new(data)
    }

    // cpp: foundation/text/native/quotes_data.h:49-52
    pub fn AddPair(&mut self, pair: (String, String)) {
        self.quote_pairs_.push(pair);
    }

    pub fn GetOpenQuote(&self, index: i32) -> String {
        debug_assert!(index >= 0);
        if index < 0 || self.quote_pairs_.is_empty() {
            return String::from("");
        }
        self.quote_pairs_[(index as usize).min(self.quote_pairs_.len() - 1)]
            .0
            .clone()
    }

    pub fn GetCloseQuote(&self, index: i32) -> String {
        debug_assert!(index >= -1);
        if index < 0 || self.quote_pairs_.is_empty() {
            return String::from("");
        }
        self.quote_pairs_[(index as usize).min(self.quote_pairs_.len() - 1)]
            .1
            .clone()
    }

    // cpp: foundation/text/native/quotes_data.h:53
    pub fn size(&self) -> i32 {
        i32::try_from(self.quote_pairs_.len()).expect("quotes data exceeds int size")
    }
}

// cpp: foundation/text/native/quotes_data.h:46-48
impl PartialEq for QuotesData {
    fn eq(&self, other: &Self) -> bool {
        self.quote_pairs_ == other.quote_pairs_
    }
}
impl Eq for QuotesData {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_pairs_repeat_last_pair_and_return_non_null_empty_quotes() {
        let data = QuotesData::CreateWithCharacters(0x00ab, 0x00bb, 0x2039, 0x203a);
        assert_eq!(data.size(), 2);
        assert_eq!(data.GetOpenQuote(0), String::from_utf16(&[0x00ab]));
        assert_eq!(data.GetCloseQuote(1), String::from_utf16(&[0x203a]));
        assert_eq!(data.GetOpenQuote(20), data.GetOpenQuote(1));
        assert_eq!(data.GetCloseQuote(-1), String::from(""));
        assert_eq!(QuotesData::Create().GetOpenQuote(0), String::from(""));
    }

    #[test]
    fn empty_quotes_are_ref_counted_and_equal() {
        let first = QuotesData::Create();
        let second = first.clone();
        assert!(first == second);
        assert_eq!(first.size(), 0);
        assert!(*first == *QuotesData::Create());
    }
}
