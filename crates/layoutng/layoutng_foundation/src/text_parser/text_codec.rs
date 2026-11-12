//! Source TextCodecUtf8.cc:86-153,239-553 and TextCodecUtf16.cc:68-157.
//! Streaming decode state, invalid-sequence consumption and fetch EOF semantics.
use crate::BlinkString;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlushBehavior {
    DoNotFlush,
    FetchEof,
    DataEof,
}
#[derive(Default)]
pub struct TextCodecUtf8 {
    partial: [u8; 4],
    partial_size: usize,
}
impl TextCodecUtf8 {
    fn ConsumePartial(&mut self, n: usize) {
        self.partial.copy_within(n..self.partial_size, 0);
        self.partial_size -= n;
    }
    fn Partial(
        &mut self,
        source: &mut &[u8],
        flush: bool,
        saw_error: &mut bool,
        out: &mut Vec<u16>,
    ) -> bool {
        while self.partial_size != 0 {
            if self.partial[0] < 128 {
                out.push(u16::from(self.partial[0]));
                self.ConsumePartial(1);
                continue;
            }
            let mut count = SequenceLength(self.partial[0]);
            let character = if count == 0 {
                -1
            } else {
                if count > self.partial_size {
                    let n = (count - self.partial_size).min(source.len());
                    self.partial[self.partial_size..self.partial_size + n]
                        .copy_from_slice(&source[..n]);
                    self.partial_size += n;
                    *source = &source[n..];
                    self.partial[self.partial_size..count].fill(0);
                }
                let c = DecodeNonAsciiSequence(&self.partial[..count]);
                if count > self.partial_size && !flush && (-c) as usize == self.partial_size {
                    return false;
                }
                c
            };
            let character = if character < 0 {
                *saw_error = true;
                count = (-character) as usize;
                0xfffd
            } else {
                character
            };
            AppendCharacter(out, character as u32);
            self.ConsumePartial(count);
        }
        true
    }
    pub fn Decode(
        &mut self,
        mut bytes: &[u8],
        flush: FlushBehavior,
        saw_error: &mut bool,
    ) -> BlinkString {
        let mut out = Vec::with_capacity(bytes.len() + self.partial_size);
        let do_flush = flush != FlushBehavior::DoNotFlush;
        loop {
            if self.partial_size != 0 && !self.Partial(&mut bytes, do_flush, saw_error, &mut out) {
                break;
            }
            while !bytes.is_empty() {
                if bytes[0] < 128 {
                    let n = bytes.iter().take_while(|&&b| b < 128).count();
                    out.extend(bytes[..n].iter().map(|&b| u16::from(b)));
                    bytes = &bytes[n..];
                    continue;
                }
                let mut count = SequenceLength(bytes[0]);
                let character = if count == 0 {
                    -1
                } else {
                    if count > bytes.len() {
                        self.partial_size = bytes.len();
                        self.partial[..self.partial_size].copy_from_slice(bytes);
                        bytes = &[];
                        break;
                    }
                    DecodeNonAsciiSequence(&bytes[..count])
                };
                let character = if character < 0 {
                    *saw_error = true;
                    count = (-character) as usize;
                    0xfffd
                } else {
                    character
                };
                AppendCharacter(&mut out, character as u32);
                bytes = &bytes[count..];
            }
            if self.partial_size == 0 {
                break;
            }
        }
        BlinkString::Create8BitIfPossible(&out)
    }
}
fn SequenceLength(b: u8) -> usize {
    match b {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => 0,
    }
}
fn DecodeNonAsciiSequence(s: &[u8]) -> i32 {
    let second_valid = match s[0] {
        0xe0 => (0xa0..=0xbf).contains(&s[1]),
        0xed => (0x80..=0x9f).contains(&s[1]),
        0xf0 => (0x90..=0xbf).contains(&s[1]),
        0xf4 => (0x80..=0x8f).contains(&s[1]),
        _ => (0x80..=0xbf).contains(&s[1]),
    };
    if !second_valid {
        return -1;
    }
    if s.len() == 2 {
        return ((i32::from(s[0]) << 6) + i32::from(s[1])) - 0x3080;
    }
    if !(0x80..=0xbf).contains(&s[2]) {
        return -2;
    }
    if s.len() == 3 {
        return ((i32::from(s[0]) << 12) + (i32::from(s[1]) << 6) + i32::from(s[2])) - 0xe2080;
    }
    if !(0x80..=0xbf).contains(&s[3]) {
        return -3;
    }
    ((i32::from(s[0]) << 18) + (i32::from(s[1]) << 12) + (i32::from(s[2]) << 6) + i32::from(s[3]))
        - 0x3c82080
}
fn AppendCharacter(out: &mut Vec<u16>, c: u32) {
    if c <= 0xffff {
        out.push(c as u16);
    } else {
        let c = c - 0x10000;
        out.push((0xd800 + (c >> 10)) as u16);
        out.push((0xdc00 + (c & 1023)) as u16);
    }
}
pub struct TextCodecUtf16 {
    little_endian: bool,
    lead_byte: Option<u8>,
    lead_surrogate: Option<u16>,
}
impl TextCodecUtf16 {
    pub fn new(little_endian: bool) -> Self {
        Self {
            little_endian,
            lead_byte: None,
            lead_surrogate: None,
        }
    }
    fn Character(&mut self, c: u16, saw_error: &mut bool, out: &mut Vec<u16>) {
        if self.lead_surrogate.is_some() && (0xdc00..=0xdfff).contains(&c) {
            out.push(self.lead_surrogate.take().unwrap());
            out.push(c);
            return;
        }
        if self.lead_surrogate.take().is_some() {
            *saw_error = true;
            out.push(0xfffd);
        }
        if (0xd800..=0xdbff).contains(&c) {
            self.lead_surrogate = Some(c);
        } else if (0xdc00..=0xdfff).contains(&c) {
            *saw_error = true;
            out.push(0xfffd);
        } else {
            out.push(c);
        }
    }
    pub fn Decode(
        &mut self,
        mut bytes: &[u8],
        flush: FlushBehavior,
        saw_error: &mut bool,
    ) -> BlinkString {
        let really_flush = flush == FlushBehavior::DataEof;
        if bytes.is_empty() {
            if really_flush && (self.lead_byte.is_some() || self.lead_surrogate.is_some()) {
                self.lead_byte = None;
                self.lead_surrogate = None;
                *saw_error = true;
                return BlinkString::from_utf16(&[0xfffd]);
            }
            return BlinkString::default();
        }
        let mut out = Vec::with_capacity(bytes.len() / 2 + 2);
        if let Some(first) = self.lead_byte.take() {
            let mut c = u16::from_le_bytes([first, bytes[0]]);
            bytes = &bytes[1..];
            if !self.little_endian {
                c = c.swap_bytes();
            }
            self.Character(c, saw_error, &mut out);
        }
        while bytes.len() >= 2 {
            let mut c = u16::from_le_bytes([bytes[0], bytes[1]]);
            bytes = &bytes[2..];
            if !self.little_endian {
                c = c.swap_bytes();
            }
            self.Character(c, saw_error, &mut out);
        }
        if let Some(&last) = bytes.first() {
            self.lead_byte = Some(last);
        }
        if really_flush && (self.lead_byte.is_some() || self.lead_surrogate.is_some()) {
            self.lead_byte = None;
            self.lead_surrogate = None;
            *saw_error = true;
            out.push(0xfffd);
        }
        BlinkString::from_utf16(&out)
    }
}
