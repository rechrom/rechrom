//! Source import-map JSON cursor. C++ strings are bytes, including the UTF-8
//! encoding of individual surrogate escapes; retain them without replacement.
use std::collections::HashMap;

// cpp: browser/browser.cc:332-475
pub(crate) struct JSONCursor<'a> {
    input: &'a [u8],
    position: usize,
}
impl<'a> JSONCursor<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }
    pub(crate) fn ParseImportMap(mut self) -> Result<HashMap<Vec<u8>, Vec<u8>>, &'static str> {
        let mut imports = HashMap::new();
        self.expect(b'{')?;
        // Preserve the source early return, including its trailing-data behavior.
        if self.consume(b'}') {
            return Ok(imports);
        }
        loop {
            let key = self.string()?;
            self.expect(b':')?;
            if key == b"imports" {
                self.string_map(&mut imports)?;
            } else {
                self.skip_value()?;
            }
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        self.whitespace();
        if self.position != self.input.len() {
            return Err("invalid import map JSON");
        }
        Ok(imports)
    }
    fn whitespace(&mut self) {
        while self
            .input
            .get(self.position)
            .is_some_and(|b| matches!(b, b' ' | b'\t'..=b'\r'))
        {
            self.position += 1;
        }
    }
    fn consume(&mut self, value: u8) -> bool {
        self.whitespace();
        if self.input.get(self.position) != Some(&value) {
            return false;
        }
        self.position += 1;
        true
    }
    fn expect(&mut self, value: u8) -> Result<(), &'static str> {
        if self.consume(value) {
            Ok(())
        } else {
            Err("invalid import map JSON")
        }
    }
    fn string(&mut self) -> Result<Vec<u8>, &'static str> {
        self.expect(b'"')?;
        let mut result = Vec::new();
        while let Some(&value) = self.input.get(self.position) {
            self.position += 1;
            if value == b'"' {
                return Ok(result);
            }
            if value < 0x20 {
                return Err("invalid import map JSON");
            }
            if value != b'\\' {
                result.push(value);
                continue;
            }
            let escaped = *self
                .input
                .get(self.position)
                .ok_or("invalid import map JSON")?;
            self.position += 1;
            match escaped {
                b'"' | b'\\' | b'/' => result.push(escaped),
                b'b' => result.push(8),
                b'f' => result.push(12),
                b'n' => result.push(10),
                b'r' => result.push(13),
                b't' => result.push(9),
                b'u' => {
                    if self.position + 4 > self.input.len() {
                        return Err("invalid import map JSON");
                    }
                    let mut cp = 0u32;
                    for _ in 0..4 {
                        let hex = self.input[self.position];
                        self.position += 1;
                        let digit = match hex {
                            b'0'..=b'9' => hex - b'0',
                            b'a'..=b'f' => hex - b'a' + 10,
                            b'A'..=b'F' => hex - b'A' + 10,
                            _ => return Err("invalid import map JSON escape"),
                        };
                        cp = (cp << 4) | digit as u32;
                    }
                    // cpp: browser/browser.cc:193-203
                    if cp <= 0x7f {
                        result.push(cp as u8);
                    } else if cp <= 0x7ff {
                        result.extend([0xc0 | (cp >> 6) as u8, 0x80 | (cp & 0x3f) as u8]);
                    } else {
                        result.extend([
                            0xe0 | (cp >> 12) as u8,
                            0x80 | ((cp >> 6) & 0x3f) as u8,
                            0x80 | (cp & 0x3f) as u8,
                        ]);
                    }
                }
                _ => return Err("invalid import map JSON"),
            }
        }
        Err("invalid import map JSON")
    }
    fn string_map(&mut self, output: &mut HashMap<Vec<u8>, Vec<u8>>) -> Result<(), &'static str> {
        self.expect(b'{')?;
        if self.consume(b'}') {
            return Ok(());
        }
        loop {
            let key = self.string()?;
            self.expect(b':')?;
            self.whitespace();
            if self.input.get(self.position) == Some(&b'"') {
                output.insert(key, self.string()?);
            } else {
                self.skip_value()?;
            }
            if self.consume(b'}') {
                return Ok(());
            }
            self.expect(b',')?;
        }
    }
    fn skip_value(&mut self) -> Result<(), &'static str> {
        self.whitespace();
        if self.position == self.input.len() {
            return Err("invalid import map JSON");
        }
        if self.input[self.position] == b'"' {
            self.string()?;
            return Ok(());
        }
        if self.consume(b'{') {
            if self.consume(b'}') {
                return Ok(());
            }
            loop {
                self.string()?;
                self.expect(b':')?;
                self.skip_value()?;
                if self.consume(b'}') {
                    return Ok(());
                }
                self.expect(b',')?;
            }
        }
        if self.consume(b'[') {
            if self.consume(b']') {
                return Ok(());
            }
            loop {
                self.skip_value()?;
                if self.consume(b']') {
                    return Ok(());
                }
                self.expect(b',')?;
            }
        }
        let begin = self.position;
        while self
            .input
            .get(self.position)
            .is_some_and(|b| !matches!(b, b' ' | b'\t'..=b'\r' | b',' | b'}' | b']'))
        {
            self.position += 1;
        }
        if begin == self.position {
            Err("invalid import map JSON")
        } else {
            Ok(())
        }
    }
}

// cpp: browser/browser.cc:253-330
pub(crate) fn ResolveReferenceBytes(base: &[u8], reference: &[u8]) -> Vec<u8> {
    fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
        [a, b].concat()
    }
    fn normalize(url: Vec<u8>) -> Vec<u8> {
        let Some(scheme) = url.windows(3).position(|w| w == b"://") else {
            return url;
        };
        let Some(start) = url[scheme + 3..]
            .iter()
            .position(|&b| b == b'/')
            .map(|p| p + scheme + 3)
        else {
            return url;
        };
        let end = url[start..]
            .iter()
            .position(|b| matches!(b, b'?' | b'#'))
            .map_or(url.len(), |p| p + start);
        let path = &url[start..end];
        let mut segments = Vec::new();
        for part in path[1..].split(|&b| b == b'/') {
            match part {
                b".." => {
                    segments.pop();
                }
                b"" | b"." => {}
                _ => segments.push(part),
            }
        }
        let mut out = url[..start].to_vec();
        out.push(b'/');
        for (i, p) in segments.iter().enumerate() {
            if i != 0 {
                out.push(b'/');
            }
            out.extend_from_slice(p);
        }
        if path.len() > 1 && path.ends_with(b"/") && !out.ends_with(b"/") {
            out.push(b'/');
        }
        out.extend_from_slice(&url[end..]);
        out
    }
    if reference.is_empty() {
        return base.to_vec();
    }
    let colon = reference.iter().position(|&b| b == b':');
    let slash = reference.iter().position(|&b| b == b'/');
    if colon.is_some_and(|c| slash.is_none_or(|s| c < s)) {
        return normalize(reference.to_vec());
    }
    if reference[0] == b'#' {
        return concat(
            &base[..base.iter().position(|&b| b == b'#').unwrap_or(base.len())],
            reference,
        );
    }
    if reference[0] == b'?' {
        return concat(
            &base[..base
                .iter()
                .position(|b| matches!(b, b'?' | b'#'))
                .unwrap_or(base.len())],
            reference,
        );
    }
    if reference.starts_with(b"//") {
        if let Some(s) = base.iter().position(|&b| b == b':') {
            return normalize(concat(&base[..s + 1], reference));
        }
    }
    if reference[0] == b'/' {
        if let Some(authority) = base.windows(2).position(|w| w == b"//") {
            let end = base[authority + 2..]
                .iter()
                .position(|&b| b == b'/')
                .map_or(base.len(), |p| p + authority + 2);
            return normalize(concat(&base[..end], reference));
        }
    }
    let end = base
        .iter()
        .position(|b| matches!(b, b'?' | b'#'))
        .unwrap_or(base.len());
    let base = &base[..end];
    let Some(slash) = base.iter().rposition(|&b| b == b'/') else {
        return reference.to_vec();
    };
    normalize(concat(&base[..slash + 1], reference))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_json_cursor_and_byte_url_resolution_match_cpp() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/cpp-reference/import-map");
        fn hex(v: &[u8]) -> String {
            v.iter().map(|b| format!("{b:02x}")).collect()
        }
        let mut actual = String::new();
        for line in std::fs::read_to_string(root.join("inputs.tsv"))
            .unwrap()
            .lines()
        {
            let (id, value) = line.split_once('\t').unwrap();
            let input = (0..value.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>();
            actual.push_str(id);
            actual.push('\t');
            match JSONCursor::new(&input).ParseImportMap() {
                Ok(map) => {
                    let mut entries = map
                        .into_iter()
                        .filter(|(_, v)| !v.is_empty())
                        .map(|(key, value)| {
                            (
                                key,
                                ResolveReferenceBytes(
                                    b"https://page.test/assets/imports.json?x#f",
                                    &value,
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    entries.sort();
                    actual.push_str("ok");
                    for (key, value) in entries {
                        actual.push_str(&format!("\t{}:{}", hex(&key), hex(&value)));
                    }
                }
                Err(message) => actual.push_str(&format!("error\t{message}")),
            }
            actual.push('\n');
        }
        std::fs::write(root.join("results-rust.tsv"), &actual).unwrap();
        assert_eq!(
            actual,
            std::fs::read_to_string(root.join("results.tsv")).unwrap()
        );
    }
}
