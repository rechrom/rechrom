#![allow(non_upper_case_globals)]

// cpp: font_engine/fonts/emoji_presentation_scanner.c:26-29
pub const _emoji_presentation_trans_keys: [u8; 27] = [
    0, 13, 14, 15, 0, 13, 9, 12, 10, 12, 10, 10, 4, 12, 4, 12, 6, 6, 9, 12, 8, 8, 8, 10, 9, 14, 0,
];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:31-34
pub const _emoji_presentation_key_spans: [u8; 13] = [14, 2, 14, 4, 3, 1, 9, 9, 1, 4, 1, 3, 6];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:36-39
pub const _emoji_presentation_index_offsets: [u8; 13] =
    [0, 15, 18, 33, 38, 42, 44, 54, 64, 66, 71, 73, 77];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:41-53
pub const _emoji_presentation_indicies: [u8; 85] = [
    1, 1, 1, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 4, 5, 3, 6, 6, 7, 8, 9, 9, 10, 11, 9, 9, 9, 9, 9,
    12, 9, 5, 13, 14, 15, 0, 13, 16, 17, 16, 13, 0, 17, 16, 16, 16, 16, 16, 13, 16, 17, 16, 17, 16,
    16, 16, 16, 5, 13, 14, 15, 16, 5, 18, 5, 13, 19, 20, 18, 14, 21, 23, 22, 13, 22, 5, 13, 14, 15,
    16, 4, 16, 0,
];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:55-59
pub const _emoji_presentation_trans_targs: [u8; 24] = [
    2, 4, 6, 2, 1, 2, 3, 3, 7, 2, 8, 9, 12, 0, 2, 5, 2, 5, 2, 10, 11, 2, 2, 2,
];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:61-65
pub const _emoji_presentation_trans_actions: [u8; 24] = [
    1, 2, 2, 3, 0, 4, 7, 2, 2, 8, 0, 7, 2, 0, 9, 10, 11, 2, 12, 0, 10, 13, 14, 15,
];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:67-70
pub const _emoji_presentation_to_state_actions: [u8; 13] = [0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:72-75
pub const _emoji_presentation_from_state_actions: [u8; 13] =
    [0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:77-80
pub const _emoji_presentation_eof_trans: [u8; 13] = [1, 4, 0, 1, 17, 1, 17, 17, 19, 19, 22, 23, 17];

// cpp: font_engine/fonts/emoji_presentation_scanner.c:82-84
pub const emoji_presentation_start: usize = 2;
pub const emoji_presentation_en_text_and_emoji_run: usize = 2;
