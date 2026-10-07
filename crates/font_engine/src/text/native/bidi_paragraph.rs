#![allow(non_snake_case)]

use foundation::{DirectionFromLevel, IsLtr, String, StringView, TextDirection};
use icu_bidi::UBiDiLevel;
use std::ptr::NonNull;

const UBIDI_LTR: UBiDiLevel = 0;
const UBIDI_RTL: UBiDiLevel = 1;
const UBIDI_DEFAULT_LTR: UBiDiLevel = 0xFE;
const UBIDI_MIXED: i32 = 2;
const U_ZERO_ERROR: i32 = 0;
const U_LEFT_TO_RIGHT: u8 = 0;
const U_RIGHT_TO_LEFT: u8 = 1;
const U_RIGHT_TO_LEFT_ARABIC: u8 = 13;
const K_LEFT_TO_RIGHT_OVERRIDE: u16 = 0x202D;
const K_RIGHT_TO_LEFT_OVERRIDE: u16 = 0x202E;
const K_POP_DIRECTIONAL_FORMATTING: u16 = 0x202C;

use icu_bidi::native as icu_api;

// cpp: font_engine/text/native/bidi_paragraph.h:74-90
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Run {
    pub start: u32,
    pub end: u32,
    pub level: UBiDiLevel,
}

impl Run {
    pub fn new(start: u32, end: u32, level: UBiDiLevel) -> Self {
        debug_assert!(end > start);
        Self { start, end, level }
    }

    pub fn Length(&self) -> u32 {
        self.end - self.start
    }

    pub fn Direction(&self) -> TextDirection {
        DirectionFromLevel(u32::from(self.level))
    }
}

// cpp: font_engine/text/native/bidi_paragraph.h:25-133
pub struct BidiParagraph {
    ubidi_: Option<NonNull<icu_api::UBidi>>,
    base_direction_: TextDirection,
    // ICU may retain the UTF-16 pointer passed to ubidi_setPara. Keeping a
    // cloned Arc-backed String ensures the original buffer stays alive.
    text_: Option<String>,
}

impl Default for BidiParagraph {
    // cpp: font_engine/text/native/bidi_paragraph.h:31-31,128-129
    fn default() -> Self {
        Self {
            ubidi_: None,
            base_direction_: TextDirection::kLtr,
            text_: None,
        }
    }
}

impl Drop for BidiParagraph {
    // cpp: font_engine/text/native/bidi_paragraph.h:122-126
    fn drop(&mut self) {
        if let Some(bidi) = self.ubidi_ {
            unsafe { icu_api::ubidi_close(bidi.as_ptr()) };
        }
    }
}

impl BidiParagraph {
    // cpp: font_engine/text/native/bidi_paragraph.h:32-36
    pub fn new(text: &String, base_direction: Option<TextDirection>) -> Self {
        let mut result = Self::default();
        result.SetParagraph(text, base_direction);
        result
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:13-46
    pub fn SetParagraph(&mut self, text: &String, base_direction: Option<TextDirection>) -> bool {
        const K_ICU_RUN_SIZE: usize = std::mem::size_of::<i32>() * 3;
        assert!((text.length() as usize) <= i32::MAX as usize / K_ICU_RUN_SIZE);
        debug_assert!(!text.IsNull());
        if self.ubidi_.is_none() {
            self.ubidi_ = NonNull::new(unsafe { icu_api::ubidi_open() });
        }
        let bidi = self.ubidi_.expect("ICU failed to open a bidi paragraph");
        let para_level = if let Some(direction) = base_direction {
            self.base_direction_ = direction;
            if IsLtr(direction) {
                UBIDI_LTR
            } else {
                UBIDI_RTL
            }
        } else {
            UBIDI_DEFAULT_LTR
        };
        self.text_ = Some(text.clone());
        let units = self
            .text_
            .as_ref()
            .unwrap()
            .Span16()
            .expect("non-null text");
        let mut error = U_ZERO_ERROR;
        unsafe {
            icu_api::ubidi_setPara(
                bidi.as_ptr(),
                units.as_ptr(),
                text.length() as i32,
                para_level,
                std::ptr::null_mut(),
                &mut error,
            );
        }
        assert!(error <= U_ZERO_ERROR, "ICU failed to set a bidi paragraph");
        if base_direction.is_none() {
            self.base_direction_ = DirectionFromLevel(u32::from(unsafe {
                icu_api::ubidi_getParaLevel(bidi.as_ptr())
            }));
        }
        true
    }

    // cpp: font_engine/text/native/bidi_paragraph.h:45-53
    pub fn IsUnidirectional(&self) -> bool {
        let bidi = self.ubidi_.expect("paragraph not initialized");
        unsafe { icu_api::ubidi_getDirection(bidi.as_ptr()) != UBIDI_MIXED }
    }

    pub fn BaseDirection(&self) -> TextDirection {
        self.base_direction_
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:49-104
    pub fn BaseDirectionForString(
        text: &StringView,
        stop_at: Option<fn(u16) -> bool>,
    ) -> Option<TextDirection> {
        let units = text.Span16();
        let mut i = 0;
        while i < units.len() {
            let mut ch = u32::from(units[i]);
            i += 1;
            if (0xD800..=0xDBFF).contains(&ch) && i < units.len() {
                let trail = u32::from(units[i]);
                if (0xDC00..=0xDFFF).contains(&trail) {
                    ch = 0x10000 + ((ch - 0xD800) << 10) + (trail - 0xDC00);
                    i += 1;
                }
            }
            let bidi_class = foundation::unicode_data::BidiClass(ch);
            match bidi_class {
                U_LEFT_TO_RIGHT => return Some(TextDirection::kLtr),
                U_RIGHT_TO_LEFT | U_RIGHT_TO_LEFT_ARABIC => return Some(TextDirection::kRtl),
                _ => {}
            }
            if stop_at.is_some_and(|predicate| predicate(ch as u16)) {
                break;
            }
        }
        None
    }

    // cpp: font_engine/text/native/bidi_paragraph.h:136-145
    pub fn BaseDirectionForStringOrLtr(text: &StringView) -> TextDirection {
        Self::BaseDirectionForStringOrLtrWithStopAt(text, None)
    }

    pub fn BaseDirectionForStringOrLtrWithStopAt(
        text: &StringView,
        stop_at: Option<fn(u16) -> bool>,
    ) -> TextDirection {
        if text.IsEmpty() || text.Span16().iter().all(|&unit| unit <= 0xFF) {
            return TextDirection::kLtr;
        }
        Self::BaseDirectionForString(text, stop_at).unwrap_or(TextDirection::kLtr)
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:107-116
    pub fn StringWithDirectionalOverride(text: &StringView, direction: TextDirection) -> String {
        let mut units = Vec::with_capacity(text.length() as usize + 2);
        units.push(if IsLtr(direction) {
            K_LEFT_TO_RIGHT_OVERRIDE
        } else {
            K_RIGHT_TO_LEFT_OVERRIDE
        });
        units.extend_from_slice(text.Span16());
        units.push(K_POP_DIRECTIONAL_FORMATTING);
        String::from_utf16(&units)
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:118-122
    pub fn GetLogicalRun(&self, start: u32, level: &mut UBiDiLevel) -> u32 {
        let bidi = self.ubidi_.expect("paragraph not initialized");
        let mut end = 0;
        unsafe { icu_api::ubidi_getLogicalRun(bidi.as_ptr(), start as i32, &mut end, level) };
        end as u32
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:124-133
    pub fn GetLogicalRuns(&self, text: &String, runs: &mut Vec<Run>) {
        debug_assert!(runs.is_empty());
        let mut start = 0;
        while start < text.length() {
            let mut level = 0;
            let end = self.GetLogicalRun(start, &mut level);
            debug_assert!(end > start);
            runs.push(Run::new(start, end, level));
            start = end;
        }
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:135-153
    pub fn GetVisualRuns(&self, text: &String, runs: &mut Vec<Run>) {
        debug_assert!(runs.is_empty());
        let mut logical_runs = Vec::new();
        self.GetLogicalRuns(text, &mut logical_runs);
        let mut levels = Vec::with_capacity(logical_runs.len());
        for run in &logical_runs {
            levels.push(run.level);
        }
        let mut indices_in_visual_order = vec![0; logical_runs.len()];
        Self::IndicesInVisualOrder(&levels, &mut indices_in_visual_order);
        for index in indices_in_visual_order {
            runs.push(logical_runs[index as usize]);
        }
    }

    // cpp: font_engine/text/native/bidi_paragraph.cc:155-159
    pub fn IndicesInVisualOrder(levels: &[UBiDiLevel], indices: &mut [i32]) {
        assert_eq!(levels.len(), indices.len());
        unsafe {
            icu_api::ubidi_reorderVisual(
                levels.as_ptr(),
                levels.len() as i32,
                indices.as_mut_ptr(),
            );
        }
    }
}
