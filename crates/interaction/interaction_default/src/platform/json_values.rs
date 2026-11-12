// Chromium platform/json/json_values.cc:45-115, revision
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
/// JSONValue::QuoteString preserves the original UTF16 units. Conversion to
/// console UTF8, if required, happens only at the console's existing boundary.
pub fn QuoteString(input: &[u16]) -> Vec<u16> {
    let mut output = Vec::with_capacity(input.len() + 2);
    output.push(34);
    for &c in input {
        let escape = match c {
            8 => Some(b'b'),
            12 => Some(b'f'),
            10 => Some(b'n'),
            13 => Some(b'r'),
            9 => Some(b't'),
            92 => Some(b'\\'),
            34 => Some(b'"'),
            _ => None,
        };
        if let Some(escape) = escape {
            output.extend([92, u16::from(escape)]);
        } else if c < 32 || c == 60 || c == 62 {
            output.extend([92, 117]);
            for shift in [12, 8, 4, 0] {
                output.push(u16::from(
                    b"0123456789ABCDEF"[usize::from((c >> shift) & 15)],
                ));
            }
        } else {
            output.push(c);
        }
    }
    output.push(34);
    output
}
