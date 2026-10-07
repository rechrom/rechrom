#![allow(non_snake_case, non_upper_case_globals)]

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

// Interface values from icu_bidi/unicode/uscript.h:63-115.
pub type UScriptCode = i32;
const USCRIPT_INVALID_CODE: UScriptCode = -1;
const USCRIPT_COMMON: UScriptCode = 0;
const USCRIPT_INHERITED: UScriptCode = 1;
const USCRIPT_BOPOMOFO: UScriptCode = 5;
const USCRIPT_HAN: UScriptCode = 17;
const USCRIPT_HIRAGANA: UScriptCode = 20;
const USCRIPT_KATAKANA: UScriptCode = 22;
const USCRIPT_LATIN: UScriptCode = 25;
const USCRIPT_KATAKANA_OR_HIRAGANA: UScriptCode = 54;
const U_ZERO_ERROR: i32 = 0;
const U_BUFFER_OVERFLOW_ERROR: i32 = 15;
const UCHAR_EAST_ASIAN_WIDTH: i32 = 0x1004;
const UCHAR_BIDI_PAIRED_BRACKET_TYPE: i32 = 0x1015;
const U_EA_HALFWIDTH: i32 = 2;
const U_EA_FULLWIDTH: i32 = 3;
const U_EA_WIDE: i32 = 5;
const K_LEFT_CORNER_BRACKET: i32 = 0x300C;

use icu_bidi::native as icu_api;

// cpp: font_engine/fonts/script_run_iterator.cc:24-35
fn GetScriptForOpenType(ch: i32, status: &mut i32) -> UScriptCode {
    let script = unsafe { icu_api::uscript_getScript(ch, status) };
    if *status > U_ZERO_ERROR {
        return script;
    }
    if script == USCRIPT_KATAKANA || script == USCRIPT_KATAKANA_OR_HIRAGANA {
        return USCRIPT_HIRAGANA;
    }
    script
}

// cpp: font_engine/fonts/script_run_iterator.h:94-101
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairedBracketType {
    kBracketTypeNone,
    kBracketTypeOpen,
    kBracketTypeClose,
    kBracketTypeCount,
}

// C++ std::bitset<U+D800> is source-owned cached character membership.
// A fixed word array keeps the same bounds and bit behavior.
// cpp: font_engine/fonts/script_run_iterator.h:104-106
pub struct UnicodeBitSet([AtomicU64; 0xD800 / 64]);

impl Default for UnicodeBitSet {
    fn default() -> Self {
        Self(std::array::from_fn(|_| AtomicU64::new(0)))
    }
}

impl UnicodeBitSet {
    pub fn size(&self) -> usize {
        0xD800
    }
    pub fn test(&self, index: u32) -> bool {
        let index = index as usize;
        assert!(index < self.size());
        self.0[index / 64].load(Ordering::Relaxed) & (1_u64 << (index % 64)) != 0
    }
    pub fn set(&self, index: u32) {
        let index = index as usize;
        assert!(index < self.size());
        self.0[index / 64].fetch_or(1_u64 << (index % 64), Ordering::Relaxed);
    }
}

// cpp: font_engine/fonts/script_run_iterator.h:130-135
pub struct RunExtensionLookups<'a> {
    pub can_remain_in_script: &'a UnicodeBitSet,
    pub inherited_not_common_chars: &'a UnicodeBitSet,
}

// C++ virtual interface maps to a Rust trait.
// cpp: font_engine/fonts/script_run_iterator.h:81-137
pub trait ScriptData {
    fn GetScripts(&self, character: i32, dst: &mut Vec<UScriptCode>);
    fn GetPairedBracket(&self, character: i32) -> i32;
    fn GetPairedBracketType(&self, character: i32) -> PairedBracketType;
    fn GetSafeToExtendExistingRun(&self, script: UScriptCode) -> RunExtensionLookups<'_>;
}

// cpp: font_engine/fonts/script_run_iterator.h:43-46
struct BracketRec {
    ch: i32,
    script: UScriptCode,
}

// Source-deleted copy operations map to a non-Clone borrowed Rust type.
// cpp: font_engine/fonts/script_run_iterator.h:24-77
pub struct ScriptRunIterator<'a> {
    text_: &'a [u16],
    length_: u32,
    brackets_: VecDeque<BracketRec>,
    brackets_fixup_depth_: usize,
    current_set_: Vec<UScriptCode>,
    next_set_: Box<Vec<UScriptCode>>,
    ahead_set_: Box<Vec<UScriptCode>>,
    ahead_character_: i32,
    ahead_pos_: u32,
    common_preferred_: UScriptCode,
    script_data_: &'a dyn ScriptData,
}

// cpp: font_engine/fonts/script_run_iterator.cc:38-41
fn IsHanScript(script: UScriptCode) -> bool {
    script == USCRIPT_HAN || script == USCRIPT_HIRAGANA || script == USCRIPT_BOPOMOFO
}

// cpp: font_engine/fonts/script_run_iterator.cc:43-49
fn FirstHanScript(list: &[UScriptCode]) -> UScriptCode {
    list.iter()
        .copied()
        .find(|script| IsHanScript(*script))
        .unwrap_or(USCRIPT_INVALID_CODE)
}

// cpp: font_engine/fonts/script_run_iterator.cc:51-67
fn GetHanScriptExtensions() -> Vec<UScriptCode> {
    let mut status = U_ZERO_ERROR;
    let mut list = vec![0; ScriptRunIterator::kMaxScriptCount - 1];
    let count = unsafe {
        icu_api::uscript_getScriptExtensions(
            K_LEFT_CORNER_BRACKET,
            list.as_mut_ptr(),
            list.len() as i32,
            &mut status,
        )
    };
    assert!(status <= U_ZERO_ERROR && count > 0 && count as usize <= list.len());
    list.truncate(count as usize);
    list
}

// cpp: font_engine/fonts/script_run_iterator.cc:69-112
fn FixScriptsByEastAsianWidth(ch: i32, set: &mut Vec<UScriptCode>) {
    assert!(!set.is_empty());
    if set.len() > 1 || set[0] != USCRIPT_COMMON {
        debug_assert!(!set.contains(&USCRIPT_COMMON));
        return;
    }
    let eaw = unsafe { icu_api::u_getIntPropertyValue(ch, UCHAR_EAST_ASIAN_WIDTH) };
    if eaw == U_EA_WIDE || eaw == U_EA_FULLWIDTH || eaw == U_EA_HALFWIDTH {
        static HAN_SCRIPTS: OnceLock<Vec<UScriptCode>> = OnceLock::new();
        let han_scripts = HAN_SCRIPTS.get_or_init(GetHanScriptExtensions);
        assert!(!han_scripts.is_empty());
        set.clear();
        set.extend_from_slice(han_scripts);
    }
}

// cpp: font_engine/fonts/script_run_iterator.h:139-190
pub struct ICUScriptData {
    bits_cache_: Mutex<HashMap<UScriptCode, &'static UnicodeBitSet>>,
    inherited_not_common_chars_: UnicodeBitSet,
}

impl ICUScriptData {
    // cpp: font_engine/fonts/script_run_iterator.cc:225-229
    pub fn Instance() -> &'static Self {
        static INSTANCE: OnceLock<ICUScriptData> = OnceLock::new();
        INSTANCE.get_or_init(|| Self {
            bits_cache_: Mutex::new(HashMap::new()),
            inherited_not_common_chars_: UnicodeBitSet::default(),
        })
    }
}

impl ScriptData for ICUScriptData {
    // cpp: font_engine/fonts/script_run_iterator.cc:123-214
    fn GetScripts(&self, ch: i32, dst: &mut Vec<UScriptCode>) {
        let mut status = U_ZERO_ERROR;
        let primary_script = GetScriptForOpenType(ch, &mut status);
        if primary_script == USCRIPT_HIRAGANA && status <= U_ZERO_ERROR {
            dst.clear();
            dst.push(primary_script);
            return;
        }

        dst.resize(ScriptRunIterator::kMaxScriptCount - 1, 0);
        let mut count = unsafe {
            icu_api::uscript_getScriptExtensions(
                ch,
                dst.as_mut_ptr(),
                dst.len() as i32,
                &mut status,
            )
        };
        if status == U_BUFFER_OVERFLOW_ERROR {
            count = dst.len() as i32;
            status = U_ZERO_ERROR;
        }
        if status > U_ZERO_ERROR {
            dst.clear();
            return;
        }
        assert!(count >= 0 && count as usize <= dst.len());
        dst.truncate(count as usize);
        if primary_script == dst[0] {
            return;
        }
        if primary_script != USCRIPT_INHERITED
            && primary_script != USCRIPT_COMMON
            && primary_script != USCRIPT_INVALID_CODE
        {
            if let Some(index) = dst[1..].iter().position(|&script| script == primary_script) {
                dst.swap(0, index + 1);
            } else {
                dst.push(primary_script);
                let last = dst.len() - 1;
                dst.swap(0, last);
            }
            return;
        }
        if primary_script == USCRIPT_COMMON {
            if count == 1 {
                dst.insert(0, primary_script);
                return;
            }
            for i in 1..dst.len() {
                if dst[0] == USCRIPT_LATIN || dst[i] < dst[0] {
                    dst.swap(0, i);
                }
            }
            return;
        }

        dst.push(dst[0]);
        dst[0] = primary_script;
        for i in 2..dst.len() {
            if dst[1] == USCRIPT_LATIN || dst[i] < dst[1] {
                dst.swap(1, i);
            }
        }
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:216-218
    fn GetPairedBracket(&self, ch: i32) -> i32 {
        unsafe { icu_api::u_getBidiPairedBracket(ch) }
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:220-223
    fn GetPairedBracketType(&self, ch: i32) -> PairedBracketType {
        match unsafe { icu_api::u_getIntPropertyValue(ch, UCHAR_BIDI_PAIRED_BRACKET_TYPE) } {
            0 => PairedBracketType::kBracketTypeNone,
            1 => PairedBracketType::kBracketTypeOpen,
            2 => PairedBracketType::kBracketTypeClose,
            _ => PairedBracketType::kBracketTypeCount,
        }
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:231-285
    fn GetSafeToExtendExistingRun(&self, script: UScriptCode) -> RunExtensionLookups<'_> {
        let mut cache = self
            .bits_cache_
            .lock()
            .expect("script bitset cache poisoned");
        if let Some(&bits) = cache.get(&script) {
            return RunExtensionLookups {
                can_remain_in_script: bits,
                inherited_not_common_chars: &self.inherited_not_common_chars_,
            };
        }
        let bits = Box::new(UnicodeBitSet::default());
        let mut codes = [0; ScriptRunIterator::kMaxScriptCount];
        for ch in 0..bits.size() as i32 {
            if self.GetPairedBracketType(ch) != PairedBracketType::kBracketTypeNone {
                continue;
            }
            let mut status = U_ZERO_ERROR;
            let primary_script = GetScriptForOpenType(ch, &mut status);
            if status > U_ZERO_ERROR {
                continue;
            }
            if primary_script == script {
                bits.set(ch as u32);
                continue;
            }
            let count = unsafe {
                icu_api::uscript_getScriptExtensions(
                    ch,
                    codes.as_mut_ptr(),
                    codes.len() as i32,
                    &mut status,
                )
            };
            if status > U_ZERO_ERROR {
                continue;
            }
            if primary_script == USCRIPT_COMMON && count <= 1 {
                bits.set(ch as u32);
                continue;
            }
            if primary_script == USCRIPT_INHERITED {
                if count == 0 {
                    bits.set(ch as u32);
                    continue;
                } else {
                    self.inherited_not_common_chars_.set(ch as u32);
                }
            }
            if codes[..(count as usize).min(codes.len())].contains(&script) {
                bits.set(ch as u32);
            }
        }
        // The source cache lives for the process lifetime. The singleton's
        // per-script allocation is likewise retained so returned lookups do
        // not borrow the mutex guard.
        let bits: &'static UnicodeBitSet = Box::leak(bits);
        cache.insert(script, bits);
        RunExtensionLookups {
            can_remain_in_script: bits,
            inherited_not_common_chars: &self.inherited_not_common_chars_,
        }
    }
}

// cpp: font_engine/fonts/script_run_iterator.cc:317-320
fn IsSet(ch: u32, set: &UnicodeBitSet) -> bool {
    ch < 0xD800 && set.test(ch)
}

impl<'a> ScriptRunIterator<'a> {
    pub const kMaxUnicodeScriptExtensions: usize = 23;
    pub const kMaxScriptCount: usize = Self::kMaxUnicodeScriptExtensions + 1;
    const kMaxBrackets: usize = 32;

    // U16_NEXT leaves an unpaired surrogate as its code-unit value.
    // cpp: icu_bidi/unicode/utf16.h:309-318
    fn NextCodePoint(text: &[u16], pos: &mut u32) -> i32 {
        let lead = u32::from(text[*pos as usize]);
        *pos += 1;
        if (0xD800..=0xDBFF).contains(&lead) && (*pos as usize) < text.len() {
            let trail = u32::from(text[*pos as usize]);
            if (0xDC00..=0xDFFF).contains(&trail) {
                *pos += 1;
                return (0x10000 + ((lead - 0xD800) << 10) + (trail - 0xDC00)) as i32;
            }
        }
        lead as i32
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:288-312
    pub fn new_with_data(text: &'a [u16], data: &'a dyn ScriptData) -> Self {
        let mut result = Self {
            text_: text,
            length_: u32::try_from(text.len()).expect("script run text exceeds u32"),
            brackets_: VecDeque::new(),
            brackets_fixup_depth_: 0,
            current_set_: Vec::new(),
            next_set_: Box::new(Vec::new()),
            ahead_set_: Box::new(Vec::new()),
            ahead_character_: 0,
            ahead_pos_: 0,
            common_preferred_: USCRIPT_COMMON,
            script_data_: data,
        };
        if result.ahead_pos_ < result.length_ {
            result.current_set_.clear();
            result.current_set_.push(USCRIPT_COMMON);
            result.ahead_character_ = Self::NextCodePoint(text, &mut result.ahead_pos_);
            result
                .script_data_
                .GetScripts(result.ahead_character_, &mut result.ahead_set_);
        }
        result
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:314-315
    pub fn new(text: &'a [u16]) -> Self {
        Self::new_with_data(text, ICUScriptData::Instance())
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:322-426
    pub fn Consume(&mut self, limit: &mut u32, script: &mut UScriptCode) -> bool {
        if self.current_set_.is_empty() {
            return false;
        }

        let script_data = self.script_data_;
        let mut can_remain_in_script: Option<&UnicodeBitSet> = None;
        let mut inherited_not_common_chars: Option<&UnicodeBitSet> = None;
        let mut pos = 0;
        let mut ch = 0;
        while self.Fetch(&mut pos, &mut ch) {
            let paired_type = script_data.GetPairedBracketType(ch);
            match paired_type {
                PairedBracketType::kBracketTypeOpen => self.OpenBracket(ch),
                PairedBracketType::kBracketTypeClose => self.CloseBracket(ch),
                _ => {}
            }
            if !self.MergeSets() {
                *limit = pos;
                *script = self.ResolveCurrentScript();
                let exclude_last = paired_type == PairedBracketType::kBracketTypeOpen;
                self.FixupStack(*script, exclude_last);
                self.current_set_.clone_from(&self.next_set_);
                return true;
            }

            if can_remain_in_script.is_none()
                && self.current_set_.len() == 1
                && self.current_set_[0] > USCRIPT_INHERITED
            {
                let lookups = script_data.GetSafeToExtendExistingRun(self.current_set_[0]);
                can_remain_in_script = Some(lookups.can_remain_in_script);
                inherited_not_common_chars = Some(lookups.inherited_not_common_chars);
            }
            if let Some(can_remain) = can_remain_in_script {
                if self.ahead_pos_ <= self.length_
                    && IsSet(self.ahead_character_ as u32, can_remain)
                {
                    let mut ptr = self.ahead_pos_ as usize;
                    let end = self.length_ as usize;
                    while ptr != end {
                        let character = u32::from(self.text_[ptr]);
                        if !IsSet(character, can_remain) {
                            self.ahead_pos_ = ptr as u32;
                            if character >= 0xD800
                                || IsSet(character, inherited_not_common_chars.unwrap())
                            {
                                self.ahead_pos_ -= 1;
                            }
                            self.FetchNextCharacter();
                            break;
                        }
                        ptr += 1;
                    }

                    if ptr == end {
                        self.ahead_pos_ = self.length_ + 1;
                        break;
                    }
                }
            }
        }

        *limit = self.length_;
        *script = self.ResolveCurrentScript();
        self.current_set_.clear();
        true
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:428-438
    fn OpenBracket(&mut self, ch: i32) {
        if self.brackets_.len() == Self::kMaxBrackets {
            self.brackets_.pop_front();
            if self.brackets_fixup_depth_ == Self::kMaxBrackets {
                self.brackets_fixup_depth_ -= 1;
            }
        }
        FixScriptsByEastAsianWidth(ch, &mut self.next_set_);
        self.brackets_.push_back(BracketRec {
            ch,
            script: USCRIPT_COMMON,
        });
        self.brackets_fixup_depth_ += 1;
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:440-476
    fn CloseBracket(&mut self, ch: i32) {
        if !self.brackets_.is_empty() {
            let target = self.script_data_.GetPairedBracket(ch);
            for num_popped in 0..self.brackets_.len() {
                let index = self.brackets_.len() - 1 - num_popped;
                if self.brackets_[index].ch == target {
                    let mut script = self.brackets_[index].script;
                    if IsHanScript(script) {
                        let current_han_script = FirstHanScript(&self.current_set_);
                        if current_han_script != USCRIPT_INVALID_CODE {
                            script = current_han_script;
                        }
                    }
                    if script != USCRIPT_COMMON {
                        self.next_set_.clear();
                        self.next_set_.push(script);
                    }
                    for _ in 0..num_popped {
                        self.brackets_.pop_back();
                    }
                    self.brackets_fixup_depth_ =
                        self.brackets_fixup_depth_.saturating_sub(num_popped);
                    return;
                }
            }
        }
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:488-565
    fn MergeSets(&mut self) -> bool {
        if self.next_set_.is_empty() || self.current_set_.is_empty() {
            return false;
        }
        let mut priority_script = self.current_set_[0];
        if self.next_set_[0] <= USCRIPT_INHERITED {
            if self.next_set_.len() == 2
                && priority_script <= USCRIPT_INHERITED
                && self.common_preferred_ == USCRIPT_COMMON
            {
                self.common_preferred_ = self.next_set_[1];
            }
            return true;
        }
        if priority_script <= USCRIPT_INHERITED {
            self.current_set_.clone_from(&self.next_set_);
            return true;
        }
        let mut have_priority = self.next_set_.contains(&priority_script);
        if self.current_set_.len() == 1 {
            return have_priority;
        }
        let mut next_index = 0;
        if !have_priority {
            priority_script = self.next_set_[0];
            next_index = 1;
            have_priority = self.current_set_[1..].contains(&priority_script);
        }
        let mut write_index = 0;
        if have_priority {
            self.current_set_[write_index] = priority_script;
            write_index += 1;
        }
        if next_index < self.next_set_.len() {
            for read_index in 1..self.current_set_.len() {
                let script = self.current_set_[read_index];
                if self.next_set_[next_index..].contains(&script) {
                    self.current_set_[write_index] = script;
                    write_index += 1;
                }
            }
        }
        if write_index > 0 {
            self.current_set_.truncate(write_index);
            return true;
        }
        false
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:571-592
    fn FixupStack(&mut self, resolved_script: UScriptCode, exclude_last: bool) {
        let mut count = self.brackets_fixup_depth_;
        if count == 0 {
            return;
        }
        if count > self.brackets_.len() {
            count = self.brackets_.len();
        }
        let mut from_back = 0;
        if exclude_last {
            from_back += 1;
            count -= 1;
            self.brackets_fixup_depth_ = 1;
        } else {
            self.brackets_fixup_depth_ = 0;
        }
        while count != 0 {
            let index = self.brackets_.len() - 1 - from_back;
            self.brackets_[index].script = resolved_script;
            from_back += 1;
            count -= 1;
        }
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:594-634
    fn Fetch(&mut self, pos: &mut u32, ch: &mut i32) -> bool {
        if self.ahead_pos_ > self.length_ {
            return false;
        }
        *pos = self
            .ahead_pos_
            .wrapping_sub(if self.ahead_character_ >= 0x10000 {
                2
            } else {
                1
            });
        *ch = self.ahead_character_;
        std::mem::swap(&mut self.next_set_, &mut self.ahead_set_);
        if self.ahead_pos_ == self.length_ {
            self.ahead_pos_ += 1;
            return true;
        }
        self.FetchNextCharacter()
    }

    fn FetchNextCharacter(&mut self) -> bool {
        self.ahead_character_ = Self::NextCodePoint(self.text_, &mut self.ahead_pos_);
        self.script_data_
            .GetScripts(self.ahead_character_, &mut self.ahead_set_);
        if self.ahead_set_.is_empty() {
            return false;
        }
        if self.ahead_set_[0] == USCRIPT_INHERITED && self.ahead_set_.len() > 1 {
            if self.next_set_[0] == USCRIPT_COMMON {
                self.next_set_.clone_from(&self.ahead_set_);
                self.next_set_.remove(0);
                self.ahead_set_.truncate(1);
            } else {
                self.ahead_set_.truncate(1);
            }
        }
        true
    }

    // cpp: font_engine/fonts/script_run_iterator.cc:636-639
    fn ResolveCurrentScript(&self) -> UScriptCode {
        let result = self.current_set_[0];
        if result == USCRIPT_COMMON {
            self.common_preferred_
        } else {
            result
        }
    }
}
