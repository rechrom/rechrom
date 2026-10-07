/*
 * Regular Expression Engine
 *
 * Copyright (c) 2017-2018 Fabrice Bellard
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
// Direct table translation of libregexp-opcode.h.
// c: libregexp-opcode.h:27
pub const REOP_invalid: u8 = 0;
// c: libregexp-opcode.h:28
pub const REOP_char: u8 = 1;
// c: libregexp-opcode.h:29
pub const REOP_char_i: u8 = 2;
// c: libregexp-opcode.h:30
pub const REOP_char32: u8 = 3;
// c: libregexp-opcode.h:31
pub const REOP_char32_i: u8 = 4;
// c: libregexp-opcode.h:32
pub const REOP_dot: u8 = 5;
// c: libregexp-opcode.h:33
pub const REOP_any: u8 = 6;
// c: libregexp-opcode.h:34
pub const REOP_space: u8 = 7;
// c: libregexp-opcode.h:35
pub const REOP_not_space: u8 = 8;
// c: libregexp-opcode.h:36
pub const REOP_line_start: u8 = 9;
// c: libregexp-opcode.h:37
pub const REOP_line_start_m: u8 = 10;
// c: libregexp-opcode.h:38
pub const REOP_line_end: u8 = 11;
// c: libregexp-opcode.h:39
pub const REOP_line_end_m: u8 = 12;
// c: libregexp-opcode.h:40
pub const REOP_goto: u8 = 13;
// c: libregexp-opcode.h:41
pub const REOP_split_goto_first: u8 = 14;
// c: libregexp-opcode.h:42
pub const REOP_split_next_first: u8 = 15;
// c: libregexp-opcode.h:43
pub const REOP_match: u8 = 16;
// c: libregexp-opcode.h:44
pub const REOP_lookahead_match: u8 = 17;
// c: libregexp-opcode.h:45
pub const REOP_negative_lookahead_match: u8 = 18;
// c: libregexp-opcode.h:46
pub const REOP_save_start: u8 = 19;
// c: libregexp-opcode.h:47
pub const REOP_save_end: u8 = 20;
// c: libregexp-opcode.h:48
pub const REOP_save_reset: u8 = 21;
// c: libregexp-opcode.h:49
pub const REOP_loop: u8 = 22;
// c: libregexp-opcode.h:50
pub const REOP_loop_split_goto_first: u8 = 23;
// c: libregexp-opcode.h:51
pub const REOP_loop_split_next_first: u8 = 24;
// c: libregexp-opcode.h:52
pub const REOP_loop_check_adv_split_goto_first: u8 = 25;
// c: libregexp-opcode.h:53
pub const REOP_loop_check_adv_split_next_first: u8 = 26;
// c: libregexp-opcode.h:54
pub const REOP_set_i32: u8 = 27;
// c: libregexp-opcode.h:55
pub const REOP_word_boundary: u8 = 28;
// c: libregexp-opcode.h:56
pub const REOP_word_boundary_i: u8 = 29;
// c: libregexp-opcode.h:57
pub const REOP_not_word_boundary: u8 = 30;
// c: libregexp-opcode.h:58
pub const REOP_not_word_boundary_i: u8 = 31;
// c: libregexp-opcode.h:59
pub const REOP_back_reference: u8 = 32;
// c: libregexp-opcode.h:60
pub const REOP_back_reference_i: u8 = 33;
// c: libregexp-opcode.h:61
pub const REOP_backward_back_reference: u8 = 34;
// c: libregexp-opcode.h:62
pub const REOP_backward_back_reference_i: u8 = 35;
// c: libregexp-opcode.h:63
pub const REOP_range: u8 = 36;
// c: libregexp-opcode.h:64
pub const REOP_range_i: u8 = 37;
// c: libregexp-opcode.h:65
pub const REOP_range32: u8 = 38;
// c: libregexp-opcode.h:66
pub const REOP_range32_i: u8 = 39;
// c: libregexp-opcode.h:67
pub const REOP_lookahead: u8 = 40;
// c: libregexp-opcode.h:68
pub const REOP_negative_lookahead: u8 = 41;
// c: libregexp-opcode.h:69
pub const REOP_set_char_pos: u8 = 42;
// c: libregexp-opcode.h:70
pub const REOP_check_advance: u8 = 43;
// c: libregexp-opcode.h:71
pub const REOP_prev: u8 = 44;
pub const REOP_COUNT: usize = 45;
pub static reopcode_info: [u8; REOP_COUNT] = [
    1, 3, 3, 5, 5, 1, 1, 1, 1, 1, 1, 1, 1, 5, 5, 5, 1, 1, 1, 2, 2, 3, 6, 10, 10, 10, 10, 6, 1, 1,
    1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 5, 5, 2, 2, 1,
];
pub static REOPCODE_NAMES: [&str; REOP_COUNT] = [
    "invalid",
    "char",
    "char_i",
    "char32",
    "char32_i",
    "dot",
    "any",
    "space",
    "not_space",
    "line_start",
    "line_start_m",
    "line_end",
    "line_end_m",
    "goto",
    "split_goto_first",
    "split_next_first",
    "match",
    "lookahead_match",
    "negative_lookahead_match",
    "save_start",
    "save_end",
    "save_reset",
    "loop",
    "loop_split_goto_first",
    "loop_split_next_first",
    "loop_check_adv_split_goto_first",
    "loop_check_adv_split_next_first",
    "set_i32",
    "word_boundary",
    "word_boundary_i",
    "not_word_boundary",
    "not_word_boundary_i",
    "back_reference",
    "back_reference_i",
    "backward_back_reference",
    "backward_back_reference_i",
    "range",
    "range_i",
    "range32",
    "range32_i",
    "lookahead",
    "negative_lookahead",
    "set_char_pos",
    "check_advance",
    "prev",
];
