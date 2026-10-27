//! CSS `path()` SVG data -> Blink's canonical SVGPathByteStream wire format.
//! The byte stream deliberately preserves relative and smooth segment kinds;
//! Style owns syntax while the Layout/Paint boundary owns path normalization.

use layoutng_style::style::svg_path_byte_stream::SVGPathByteStream;

struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    output: Vec<u8>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            cursor: 0,
            output: Vec::new(),
        }
    }

    fn skip_wsp(&mut self) {
        while self.cursor < self.input.len()
            && matches!(self.input[self.cursor], b' ' | b'\t' | b'\n' | b'\r' | 0x0c)
        {
            self.cursor += 1;
        }
    }

    fn skip_separator(&mut self) {
        self.skip_wsp();
        if self.input.get(self.cursor) == Some(&b',') {
            self.cursor += 1;
            self.skip_wsp();
        }
    }

    fn has_number(&mut self) -> bool {
        let saved = self.cursor;
        self.skip_separator();
        let result = self
            .input
            .get(self.cursor)
            .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.'));
        self.cursor = saved;
        result
    }

    fn number(&mut self) -> Option<f32> {
        self.skip_separator();
        let start = self.cursor;
        if self
            .input
            .get(self.cursor)
            .is_some_and(|b| matches!(b, b'+' | b'-'))
        {
            self.cursor += 1;
        }
        let integer = self.cursor;
        while self.input.get(self.cursor).is_some_and(u8::is_ascii_digit) {
            self.cursor += 1;
        }
        let mut digits = self.cursor != integer;
        if self.input.get(self.cursor) == Some(&b'.') {
            self.cursor += 1;
            let fraction = self.cursor;
            while self.input.get(self.cursor).is_some_and(u8::is_ascii_digit) {
                self.cursor += 1;
            }
            digits |= self.cursor != fraction;
        }
        if !digits {
            self.cursor = start;
            return None;
        }
        if self
            .input
            .get(self.cursor)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            let exponent = self.cursor;
            self.cursor += 1;
            if self
                .input
                .get(self.cursor)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                self.cursor += 1;
            }
            let digits = self.cursor;
            while self.input.get(self.cursor).is_some_and(u8::is_ascii_digit) {
                self.cursor += 1;
            }
            if self.cursor == digits {
                self.cursor = exponent;
            }
        }
        let value = std::str::from_utf8(&self.input[start..self.cursor])
            .ok()?
            .parse::<f32>()
            .ok()?;
        value.is_finite().then_some(value)
    }

    fn flag(&mut self) -> Option<bool> {
        self.skip_separator();
        let value = match self.input.get(self.cursor)? {
            b'0' => false,
            b'1' => true,
            _ => return None,
        };
        self.cursor += 1;
        Some(value)
    }

    fn push_u16(&mut self, value: u16) {
        self.output.extend(value.to_ne_bytes());
    }
    fn push_float(&mut self, value: f32) {
        self.output.extend(value.to_ne_bytes());
    }
    fn push_point(&mut self, x: f32, y: f32) {
        self.push_float(x);
        self.push_float(y);
    }
    fn point(&mut self) -> Option<(f32, f32)> {
        Some((self.number()?, self.number()?))
    }

    fn segment_type(command: u8) -> Option<u16> {
        Some(match command {
            b'Z' | b'z' => 1,
            b'M' => 2,
            b'm' => 3,
            b'L' => 4,
            b'l' => 5,
            b'C' => 6,
            b'c' => 7,
            b'Q' => 8,
            b'q' => 9,
            b'A' => 10,
            b'a' => 11,
            b'H' => 12,
            b'h' => 13,
            b'V' => 14,
            b'v' => 15,
            b'S' => 16,
            b's' => 17,
            b'T' => 18,
            b't' => 19,
            _ => return None,
        })
    }

    fn segment(&mut self, command: u8) -> Option<()> {
        self.push_u16(Self::segment_type(command)?);
        match command.to_ascii_uppercase() {
            b'M' | b'L' | b'T' => {
                let (x, y) = self.point()?;
                self.push_point(x, y);
            }
            b'H' | b'V' => {
                let value = self.number()?;
                self.push_float(value);
            }
            b'C' => {
                let p1 = self.point()?;
                let p2 = self.point()?;
                let target = self.point()?;
                self.push_point(p1.0, p1.1);
                self.push_point(p2.0, p2.1);
                self.push_point(target.0, target.1);
            }
            b'S' | b'Q' => {
                let control = self.point()?;
                let target = self.point()?;
                self.push_point(control.0, control.1);
                self.push_point(target.0, target.1);
            }
            b'A' => {
                let radii = self.point()?;
                let angle = self.number()?;
                let large = self.flag()?;
                let sweep = self.flag()?;
                let target = self.point()?;
                self.push_point(radii.0, radii.1);
                self.push_float(angle);
                self.output.push(large as u8);
                self.output.push(sweep as u8);
                self.push_point(target.0, target.1);
            }
            _ => return None,
        }
        Some(())
    }

    fn parse(mut self) -> Option<SVGPathByteStream> {
        self.skip_wsp();
        let mut command = *self.input.get(self.cursor)?;
        if !matches!(command, b'M' | b'm') {
            return None;
        }
        let mut first = true;
        while self.cursor < self.input.len() {
            self.skip_wsp();
            if self.cursor == self.input.len() {
                break;
            }
            let mut explicit_command = false;
            if self.input[self.cursor].is_ascii_alphabetic() {
                command = self.input[self.cursor];
                self.cursor += 1;
                explicit_command = true;
            } else if first {
                return None;
            }
            first = false;
            Self::segment_type(command)?;
            if matches!(command, b'Z' | b'z') {
                self.push_u16(1);
                // A close-path cannot be implicitly repeated by a number.
                self.skip_wsp();
                if self.cursor < self.input.len() && !self.input[self.cursor].is_ascii_alphabetic()
                {
                    return None;
                }
                continue;
            }
            // SVG's comma-wsp grammar permits a delimiter between numeric
            // arguments, but never directly between a command letter and its
            // first argument.
            if explicit_command {
                self.skip_wsp();
                if self.input.get(self.cursor) == Some(&b',') {
                    return None;
                }
            }
            self.segment(command)?;
            if matches!(command, b'M' | b'm') {
                command = if command == b'M' { b'L' } else { b'l' };
            }
            while self.has_number() {
                self.segment(command)?;
            }
            self.skip_wsp();
            if self.cursor < self.input.len() && !self.input[self.cursor].is_ascii_alphabetic() {
                return None;
            }
        }
        (!self.output.is_empty()).then(|| SVGPathByteStream::from_data(self.output))
    }
}

pub(super) fn ParseSVGPathByteStream(input: &str) -> Option<SVGPathByteStream> {
    Parser::new(input.as_bytes()).parse()
}

#[cfg(test)]
mod tests {
    use super::ParseSVGPathByteStream;

    #[test]
    fn preserves_all_svg_segment_kinds() {
        let path = ParseSVGPathByteStream("M1 2h3v4s5 6 7 8t9 10q11 12 13 14a15 16 20 0 1 17 18z")
            .unwrap();
        assert!(!path.IsEmpty());
    }
}
