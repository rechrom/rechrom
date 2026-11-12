//! StringTruncator: string_truncator.cc:43-222, fixed Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. Width is measured by the original
//! PlainTextPainter; CharacterBreakIterator preserves grapheme boundaries.
use crate::text::implementation::character_break_iterator::{
    kTextBreakDone, CharacterBreakIterator,
};
#[derive(Clone, Copy)]
pub enum Truncation {
    Center,
    Right,
}
pub fn TruncateString<E>(
    text: &[u16],
    max_width: f32,
    mode: Truncation,
    mut width: impl FnMut(&[u16]) -> Result<f32, E>,
) -> Result<Vec<u16>, E> {
    fn truncate(text: &[u16], keep: usize, mode: Truncation) -> Vec<u16> {
        let it = CharacterBreakIterator::from_utf16(text);
        let preceding = |offset: usize| {
            if it.IsBreak(offset as i32) {
                offset
            } else {
                let p = it.Preceding(offset as i32);
                if p == kTextBreakDone {
                    0
                } else {
                    p as usize
                }
            }
        };
        let (start, end) = match mode {
            Truncation::Right => (preceding(keep), text.len()),
            Truncation::Center => {
                let start = (keep + 1) / 2;
                let end = it.Following((start + text.len() - keep - 1) as i32);
                (
                    preceding(start),
                    if end == kTextBreakDone {
                        text.len()
                    } else {
                        end as usize
                    },
                )
            }
        };
        let mut out = Vec::with_capacity(start + 1 + text.len() - end);
        out.extend_from_slice(&text[..start]);
        out.push(0x2026);
        out.extend_from_slice(&text[end..]);
        out
    }
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let ellipsis = width(&[0x2026])?;
    let mut keep = text.len().min(2047);
    let mut current = if text.len() > 2048 {
        truncate(text, keep, Truncation::Center)
    } else {
        keep = text.len();
        text.to_vec()
    };
    let mut current_width = width(&current)?;
    if current_width <= max_width {
        return Ok(text.to_vec());
    }
    let (mut fits, mut fit_width) = (0, ellipsis);
    let (mut fails, mut fail_width) = (keep, current_width);
    if ellipsis >= max_width {
        fits = 1;
        fails = 2;
    }
    while fits + 1 < fails {
        let ratio = (fails - fits) as f32 / (fail_width - fit_width);
        keep = ((max_width * ratio) as usize).clamp(fits + 1, fails - 1);
        current = truncate(text, keep, mode);
        current_width = width(&current)?;
        if current_width <= max_width {
            fits = keep;
            fit_width = current_width;
        } else {
            fails = keep;
            fail_width = current_width;
        }
    }
    if fits == 0 {
        fits = 1;
    }
    if keep != fits {
        current = truncate(text, fits, mode);
    }
    Ok(current)
}
