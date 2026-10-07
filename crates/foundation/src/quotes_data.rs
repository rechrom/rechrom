use crate::{ScopedRefPtr, String};

// cpp: foundation/text/native/quotes_data.h:34-59
// Only Create(), equality, and size() have supplied definitions. The source
// tree contains no quotes_data.cc for the four-character constructor, pair
// insertion, or quote lookup; those remain explicit link requirements.
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
        unsafe { QuotesDataCreateWithCharacters(open1, close1, open2, close2) }
    }

    // cpp: foundation/text/native/quotes_data.h:49-52
    pub fn AddPair(&mut self, pair: (String, String)) {
        unsafe { QuotesDataAddPair(self, pair) }
    }

    pub fn GetOpenQuote(&self, index: i32) -> String {
        unsafe { QuotesDataGetOpenQuote(self, index) }
    }

    pub fn GetCloseQuote(&self, index: i32) -> String {
        unsafe { QuotesDataGetCloseQuote(self, index) }
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

unsafe extern "Rust" {
    fn QuotesDataCreateWithCharacters(
        open1: u16,
        close1: u16,
        open2: u16,
        close2: u16,
    ) -> ScopedRefPtr<QuotesData>;
    fn QuotesDataAddPair(value: &mut QuotesData, pair: (String, String));
    fn QuotesDataGetOpenQuote(value: &QuotesData, index: i32) -> String;
    fn QuotesDataGetCloseQuote(value: &QuotesData, index: i32) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_quotes_are_ref_counted_and_equal() {
        let first = QuotesData::Create();
        let second = first.clone();
        assert!(first == second);
        assert_eq!(first.size(), 0);
        assert!(*first == *QuotesData::Create());
    }
}
