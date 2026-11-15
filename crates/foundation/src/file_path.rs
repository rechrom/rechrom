//! Native FilePath value operations, Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! file_path.cc:65-75,291-297,505-554,781-786,1590-1611.
//! These preserve '.', '..' and double-root separators; std::path component
//! normalization must not change the source filename/control-state semantics.
pub fn IsSeparator(ch: u8) -> bool {
    ch == b'/' || cfg!(windows) && ch == b'\\'
}
fn Path(value: &str) -> &str {
    value
        .split('\0')
        .next()
        .expect("a string has a first split component")
}
fn Drive(path: &str) -> Option<usize> {
    let bytes = path.as_bytes();
    (cfg!(windows) && bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        .then_some(1)
}
fn StripTrailingSeparators(path: &str) -> &str {
    let bytes = path.as_bytes();
    let start = Drive(path).map_or(1, |letter| letter + 2);
    let mut end = path.len();
    let mut last_stripped = None;
    let mut pos = path.len();
    while pos > start && IsSeparator(bytes[pos - 1]) {
        if pos != start + 1 || last_stripped == Some(start + 2) || !IsSeparator(bytes[start - 1]) {
            end = pos - 1;
            last_stripped = Some(pos);
        }
        pos -= 1;
    }
    &path[..end]
}
pub fn DirName(value: &str) -> String {
    let path = StripTrailingSeparators(Path(value));
    if let Some(last) = path.as_bytes().iter().rposition(|&c| IsSeparator(c)) {
        StripTrailingSeparators(&path[..last + 1]).into()
    } else if let Some(letter) = Drive(path) {
        path[..letter + 1].into()
    } else {
        ".".into()
    }
}
pub fn BaseName(value: &str) -> String {
    let path = StripTrailingSeparators(Path(value));
    let path = Drive(path).map_or(path, |letter| &path[letter + 1..]);
    if let Some(last) = path.as_bytes().iter().rposition(|&c| IsSeparator(c)) {
        if last < path.len() - 1 {
            return path[last + 1..].into();
        }
    }
    path.into()
}
pub fn EndsWithSeparator(value: &str) -> bool {
    Path(value)
        .as_bytes()
        .last()
        .is_some_and(|&c| IsSeparator(c))
}
