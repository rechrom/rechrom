#![allow(non_snake_case, unused_assignments, unused_variables)]

use super::emoji_presentation_scanner_tables::*;
use super::utf16_ragel_iterator::UTF16RagelIterator;

// Ragel's goto labels become a loop with an explicit pending EOF transition.
// All transition-table reads and scanner actions keep the generated C order.
// cpp: font_engine/fonts/emoji_presentation_scanner.c:91-249
pub fn scan_emoji_presentation<'a>(
    mut p: UTF16RagelIterator<'a>,
    pe: UTF16RagelIterator<'a>,
    is_emoji: &mut bool,
    has_vs: &mut bool,
) -> UTF16RagelIterator<'a> {
    let mut ts = p.AssignZero(0);
    let mut te = p.AssignZero(0);
    let eof = pe;
    let mut act = 0;
    let mut cs = emoji_presentation_start;
    let mut pending_eof_transition: Option<usize> = None;

    loop {
        let trans = if let Some(trans) = pending_eof_transition.take() {
            trans
        } else if p == pe {
            if p == eof && _emoji_presentation_eof_trans[cs] > 0 {
                (_emoji_presentation_eof_trans[cs] - 1) as usize
            } else {
                break;
            }
        } else {
            if _emoji_presentation_from_state_actions[cs] == 6 {
                ts = p;
            }

            let key_index = cs << 1;
            let first_key = _emoji_presentation_trans_keys[key_index];
            let last_key = _emoji_presentation_trans_keys[key_index + 1];
            let slen = _emoji_presentation_key_spans[cs] as usize;
            let category = p.Category();
            let index = if slen > 0 && first_key <= category && category <= last_key {
                (category - first_key) as usize
            } else {
                slen
            };
            _emoji_presentation_indicies[_emoji_presentation_index_offsets[cs] as usize + index]
                as usize
        };

        cs = _emoji_presentation_trans_targs[trans] as usize;
        match _emoji_presentation_trans_actions[trans] {
            0 => {}
            // cpp: font_engine/fonts/emoji_presentation_scanner.c:148-165
            9 => {
                te = p + 1;
                *is_emoji = false;
                *has_vs = true;
                return te;
            }
            15 => {
                te = p + 1;
                *is_emoji = true;
                *has_vs = true;
                return te;
            }
            4 => {
                te = p + 1;
                *is_emoji = true;
                *has_vs = false;
                return te;
            }
            8 => {
                te = p + 1;
                *is_emoji = false;
                *has_vs = false;
                return te;
            }
            // cpp: font_engine/fonts/emoji_presentation_scanner.c:166-185
            13 => {
                te = p;
                p.PreDecrement();
                *is_emoji = false;
                *has_vs = true;
                return te;
            }
            14 => {
                te = p;
                p.PreDecrement();
                *is_emoji = true;
                *has_vs = true;
                return te;
            }
            11 => {
                te = p;
                p.PreDecrement();
                *is_emoji = true;
                *has_vs = false;
                return te;
            }
            12 => {
                te = p;
                p.PreDecrement();
                *is_emoji = false;
                *has_vs = false;
                return te;
            }
            // cpp: font_engine/fonts/emoji_presentation_scanner.c:186-212
            3 => {
                p = te - 1;
                *is_emoji = true;
                *has_vs = false;
                return te;
            }
            1 => match act {
                2 => {
                    p = te - 1;
                    *is_emoji = true;
                    *has_vs = true;
                    return te;
                }
                3 => {
                    p = te - 1;
                    *is_emoji = true;
                    *has_vs = false;
                    return te;
                }
                4 => {
                    p = te - 1;
                    *is_emoji = false;
                    *has_vs = false;
                    return te;
                }
                _ => {}
            },
            10 => {
                te = p + 1;
                act = 2;
            }
            2 => {
                te = p + 1;
                act = 3;
            }
            7 => {
                te = p + 1;
                act = 4;
            }
            action => panic!("unexpected emoji scanner action {action}"),
        }

        if _emoji_presentation_to_state_actions[cs] == 5 {
            // C++'s operator=(int) returns a modified copy; it does not
            // mutate `ts`. Keep that generated-code behavior.
            let _ = ts.AssignZero(0);
        }

        p.PreIncrement();
        if p != pe {
            continue;
        }
        if p == eof && _emoji_presentation_eof_trans[cs] > 0 {
            pending_eof_transition = Some((_emoji_presentation_eof_trans[cs] - 1) as usize);
            continue;
        }
        break;
    }

    *is_emoji = false;
    *has_vs = false;
    p
}
