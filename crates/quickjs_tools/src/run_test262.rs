//! run-test262.c, QuickJS 2026-06-04. Bellard/Gordon, MIT.
//! Byte-string, file-list and test-description algorithms. Thread agents and
//! the runner are separate source groups and are not represented by stubs.
#[cfg(windows)]
use crate::quickjs_libc_windows::crt as libc;
use quickjs::{quickjs::*, quickjs_header::*};
use std::ffi::{c_char, c_void, CStr, CString};
use std::{
    cmp::Ordering,
    fs,
    io::{self, Write},
};
fn c_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 11 | 12)
}
fn byte(s: &[u8], pos: usize) -> u8 {
    s.get(pos).copied().unwrap_or(0)
}
fn substring(s: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    let s = &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())];
    let needle = &needle[..needle.iter().position(|&c| c == 0).unwrap_or(needle.len())];
    if start > s.len() {
        return None;
    }
    if needle.is_empty() {
        Some(start)
    } else {
        s[start..]
            .windows(needle.len())
            .position(|v| v == needle)
            .map(|p| p + start)
    }
}
// run-test262.c:270..276.
pub fn strdup_len(source: &[u8], len: usize) -> Vec<u8> {
    source[..len].to_vec()
}
// run-test262.c:278..280.
pub fn str_equal(a: &[u8], b: &[u8]) -> bool {
    a == b
}
// run-test262.c:282..297.
pub fn str_append(value: &mut Option<Vec<u8>>, sep: &[u8], s: &[u8]) {
    let mut result = Vec::new();
    if let Some(old) = value.take() {
        result.extend(old);
        result.extend_from_slice(sep);
    }
    result.extend_from_slice(s);
    *value = Some(result);
}
// run-test262.c:299..307.
pub fn str_strip(s: &[u8]) -> &[u8] {
    let mut end = s.len();
    while end > 0 && c_space(s[end - 1]) {
        end -= 1;
    }
    let mut start = 0;
    while start < end && c_space(s[start]) {
        start += 1;
    }
    &s[start..end]
}
// run-test262.c:309..312.
pub fn has_prefix(s: &[u8], prefix: &[u8]) -> bool {
    s.starts_with(prefix)
}
// run-test262.c:314..326.
pub fn skip_prefix<'a>(s: &'a [u8], prefix: &[u8]) -> &'a [u8] {
    if has_prefix(s, prefix) {
        &s[prefix.len()..]
    } else {
        s
    }
}
// run-test262.c:328..336. Despite its name, this returns the directory prefix.
pub fn get_basename(filename: &[u8]) -> Option<Vec<u8>> {
    filename
        .iter()
        .rposition(|&c| c == b'/')
        .map(|p| filename[..p].to_vec())
}
// run-test262.c:338..359.
pub fn compose_path(path: Option<&[u8]>, name: &[u8]) -> Vec<u8> {
    if path.is_none_or(|p| p.is_empty()) || name.first() == Some(&b'/') {
        return name.to_vec();
    }
    let path = path.unwrap();
    let mut out = path.to_vec();
    if path.last() != Some(&b'/') {
        out.push(b'/');
    }
    out.extend_from_slice(name);
    out
}
// run-test262.c:361..386. C's signed-overflow domain is wrapped explicitly;
// all defined numeric comparisons retain the original modified ordering.
pub fn namelist_cmp(a: &[u8], b: &[u8]) -> i32 {
    let (mut pa, mut pb) = (0, 0);
    loop {
        let mut ca = byte(a, pa);
        let mut cb = byte(b, pb);
        pa += 1;
        pb += 1;
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let mut na = (ca - b'0') as i32;
            let mut nb = (cb - b'0') as i32;
            loop {
                ca = byte(a, pa);
                pa += 1;
                if !ca.is_ascii_digit() {
                    break;
                }
                na = na.wrapping_mul(10).wrapping_add((ca - b'0') as i32);
            }
            loop {
                cb = byte(b, pb);
                pb += 1;
                if !cb.is_ascii_digit() {
                    break;
                }
                nb = nb.wrapping_mul(10).wrapping_add((cb - b'0') as i32);
            }
            if na < nb {
                return -1;
            }
            if na > nb {
                return 1;
            }
        }
        if ca < cb {
            return -1;
        }
        if ca > cb {
            return 1;
        }
        if ca == 0 {
            return 0;
        }
    }
}
pub fn namelist_cmp_indirect(a: &Vec<u8>, b: &Vec<u8>) -> Ordering {
    namelist_cmp(a, b).cmp(&0)
}
#[derive(Default, Debug)]
pub struct NameList {
    pub array: Vec<Vec<u8>>,
}
// run-test262.c:393..409.
pub fn namelist_sort(list: &mut NameList) {
    list.array.sort_unstable_by(namelist_cmp_indirect);
    list.array.dedup_by(|b, a| namelist_cmp(a, b) == 0);
}
// run-test262.c:411..426.
pub fn namelist_find(list: &NameList, name: &[u8]) -> i32 {
    let (mut a, mut b) = (0, list.array.len());
    while a < b {
        let m = a + (b - a) / 2;
        let cmp = namelist_cmp(&list.array[m], name);
        if cmp < 0 {
            a = m + 1;
        } else if cmp > 0 {
            b = m;
        } else {
            return m as i32;
        }
    }
    -1
}
// run-test262.c:428..448.
pub fn namelist_add(list: &mut NameList, base: Option<&[u8]>, name: &[u8]) {
    let s = compose_path(base, name);
    if list.array.len() == list.array.capacity() {
        let next = list.array.capacity() + (list.array.capacity() >> 1) + 4;
        list.array.reserve_exact(next - list.array.len());
    }
    list.array.push(s);
}
#[cfg(unix)]
fn filename_path(name: &[u8]) -> &std::path::Path {
    use std::os::unix::ffi::OsStrExt;
    std::path::Path::new(std::ffi::OsStr::from_bytes(name))
}
#[cfg(windows)]
fn filename_path(name: &[u8]) -> std::path::PathBuf {
    // Decode with the same ANSI code page used by the CRT narrow argv/file
    // interface. The C functions diagnose any invalid resulting path later.
    std::path::PathBuf::from(crate::quickjs_libc::byte_os(name).unwrap_or_default())
}
#[cfg(not(any(unix, windows)))]
fn filename_path(name: &[u8]) -> std::path::PathBuf {
    std::path::PathBuf::from(String::from_utf8_lossy(name).as_ref())
}
// run-test262.c:450..471. fgets reads at most 1023 bytes per chunk.
pub fn namelist_load(list: &mut NameList, filename: &[u8]) -> io::Result<()> {
    let content = fs::read(filename_path(filename))?;
    let base = get_basename(filename);
    let mut pos = 0;
    while pos < content.len() {
        let max = (pos + 1023).min(content.len());
        let end = content[pos..max]
            .iter()
            .position(|&c| c == b'\n')
            .map(|p| pos + p + 1)
            .unwrap_or(max);
        let line = &content[pos..end];
        let line = &line[..line.iter().position(|&c| c == 0).unwrap_or(line.len())];
        let p = str_strip(line);
        if !p.is_empty() && p[0] != b'#' && p[0] != b';' {
            namelist_add(list, base.as_deref(), p);
        }
        pos = end;
    }
    Ok(())
}
// run-test262.c:473..485.
pub fn namelist_add_from_error_file(list: &mut NameList, file: &[u8]) {
    let mut p = 0;
    while let Some(found) = substring(file, b".js:", p) {
        let mut p0 = found;
        while p0 > 0 && file[p0 - 1] != b'\n' {
            p0 -= 1;
        }
        namelist_add(list, None, &file[p0..found + 3]);
        p = found + 1;
    }
}
// run-test262.c:487..495.
pub fn namelist_free(list: &mut NameList) {
    while list.array.pop().is_some() {}
    list.array = Vec::new();
}
// run-test262.c:1023..1026.
pub fn is_line_sep(c: u8) -> bool {
    matches!(c, 0 | b'\n' | b'\r')
}
// run-test262.c:1028..1039.
pub fn find_line(s: Option<&[u8]>, line: &[u8]) -> Option<usize> {
    let s = s?;
    let mut p = 0;
    while let Some(found) = substring(s, line, p) {
        if (found == 0 || is_line_sep(byte(s, found - 1)))
            && is_line_sep(byte(s, found + line.len()))
        {
            return Some(found);
        }
        p = found + line.len() + 1;
    }
    None
}
// run-test262.c:1041..1044.
pub fn is_word_sep(c: u8) -> bool {
    c == 0 || c_space(c) || c == b','
}
// run-test262.c:1046..1058.
pub fn find_word(s: Option<&[u8]>, word: &[u8]) -> Option<usize> {
    let s = s?;
    if word.is_empty() {
        return None;
    }
    let mut p = 0;
    while let Some(found) = substring(s, word, p) {
        if (found == 0 || is_word_sep(byte(s, found - 1)))
            && is_word_sep(byte(s, found + word.len()))
        {
            return Some(found);
        }
        p = found + word.len();
    }
    None
}
// run-test262.c:1061..1098. Return the increment of the original test_excluded.
pub fn update_exclude_dirs(
    tests: &mut NameList,
    excluded: &mut NameList,
    dirs: &mut NameList,
) -> i32 {
    let mut files = Vec::new();
    for name in excluded.array.drain(..) {
        if name.last() == Some(&b'/') {
            namelist_add(dirs, None, &name);
        } else {
            files.push(name);
        }
    }
    excluded.array = files;
    namelist_sort(dirs);
    let mut count = 0;
    tests.array.retain(|name| {
        if dirs.array.iter().any(|dir| has_prefix(name, dir)) {
            count += 1;
            false
        } else {
            true
        }
    });
    count
}
// run-test262.c:1257..1292.
pub fn find_error(file: Option<&[u8]>, filename: &[u8], is_strict: bool) -> Option<(Vec<u8>, i32)> {
    let file = file?;
    if filename.is_empty() {
        return None;
    }
    let mut p = 0;
    while let Some(found) = substring(file, filename, p) {
        p = found + filename.len();
        if (found != 0 && file[found - 1] != b'\n' && file[found - 1] != b'(')
            || byte(file, p) != b':'
        {
            continue;
        }
        let mut q = p;
        let mut line = 1;
        if byte(file, q) == b':' {
            q += 1;
            let original = q;
            while c_space(byte(file, q)) {
                q += 1;
            }
            let negative = byte(file, q) == b'-';
            if matches!(byte(file, q), b'-' | b'+') {
                q += 1;
            }
            let digits = q;
            let mut n = 0u64;
            while byte(file, q).is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add((file[q] - b'0') as u64);
                q += 1;
            }
            if q == digits {
                q = original;
                line = 0;
            } else {
                line = if negative {
                    if n >= (i64::MAX as u64) + 1 {
                        i64::MIN as i32
                    } else {
                        (-(n as i64)) as i32
                    }
                } else {
                    n.min(i64::MAX as u64) as i32
                };
            }
            if byte(file, q) == b':' {
                q += 1;
            }
        }
        while byte(file, q) == b' ' {
            q += 1;
        }
        let strict = file[q..].starts_with(b"strict mode: ");
        if strict {
            q += 13;
        }
        if strict != is_strict {
            continue;
        }
        if file[q..].starts_with(b"unexpected error: ") {
            q += 18;
        }
        let mut r = q;
        while byte(file, r) != 0 && byte(file, r) != b'\n' {
            r += 1;
        }
        loop {
            if byte(file, r) != b'\n' || byte(file, r + 1) == 0 {
                break;
            }
            let equal = (0..8).all(|i| byte(file, r + 1 + i) == byte(filename, i));
            if equal {
                break;
            }
            r += 1;
            while byte(file, r) != 0 && byte(file, r) != b'\n' {
                r += 1;
            }
        }
        return Some((file[q..r].to_vec(), line));
    }
    None
}
// run-test262.c:1294..1330. Return the original offset after consuming a char.
pub fn skip_comments(s: &[u8], mut line: i32) -> (usize, i32) {
    let mut p = 0;
    loop {
        let c = byte(s, p);
        p += 1;
        if c == 0 {
            break;
        }
        if c_space(c) {
            if c == b'\n' {
                line += 1;
            }
            continue;
        }
        if c == b'/' && byte(s, p) == b'/' {
            loop {
                p += 1;
                if matches!(byte(s, p), 0 | b'\n') {
                    break;
                }
            }
            continue;
        }
        if c == b'/' && byte(s, p) == b'*' {
            p += 1;
            while byte(s, p) != 0 {
                if byte(s, p) == b'\n' {
                    line += 1;
                    p += 1;
                    continue;
                }
                if byte(s, p) == b'*' && byte(s, p + 1) == b'/' {
                    p += 2;
                    break;
                }
                p += 1;
            }
            continue;
        }
        break;
    }
    (p, line)
}
// run-test262.c:1332..1359.
pub fn longest_match(
    s: &[u8],
    find: &[u8],
    pos: usize,
    mut line: i32,
) -> (usize, Option<usize>, Option<i32>) {
    let s = &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())];
    let find = &find[..find.iter().position(|&c| c == 0).unwrap_or(find.len())];
    let mut maxlen = 0;
    let mut best = None;
    let mut bestline = None;
    if !find.is_empty() {
        for p in pos..s.len() {
            if s[p] == find[0] {
                let mut len = 1;
                while byte(s, p + len) != 0 && byte(s, p + len) == byte(find, len) {
                    len += 1;
                }
                if len > maxlen {
                    maxlen = len;
                    best = Some(p);
                    bestline = Some(line);
                    if byte(find, len) == 0 {
                        break;
                    }
                }
            }
            if s[p] == b'\n' {
                line += 1;
            }
        }
    }
    (maxlen, best, bestline)
}
// run-test262.c:1611..1620.
pub fn find_tag(desc: &[u8], tag: &[u8], state: &mut i32) -> Option<usize> {
    substring(desc, tag, 0).map(|p| {
        *state = 0;
        p + tag.len()
    })
}
// run-test262.c:1622..1663.
pub fn get_option(desc: &[u8], pointer: &mut Option<usize>, state: &mut i32) -> Option<Vec<u8>> {
    let mut p = (*pointer)?;
    loop {
        match byte(desc, p) {
            b'[' => {
                *state += 1;
                p += 1;
            }
            b']' => {
                *state -= 1;
                if *state > 0 {
                    p += 1;
                    continue;
                }
                *pointer = None;
                return None;
            }
            b' ' | b'\t' | b'\r' | b',' | b'-' => p += 1,
            b'\n' => {
                if *state > 0 || byte(desc, p + 1) == b' ' {
                    p += 1;
                    continue;
                }
                *pointer = None;
                return None;
            }
            0 => {
                *pointer = None;
                return None;
            }
            _ => {
                let start = p;
                while !matches!(
                    byte(desc, p),
                    0 | b' ' | b'\t' | b'\r' | b'\n' | b',' | b']'
                ) {
                    p += 1;
                }
                *pointer = Some(p);
                return Some(desc[start..p].to_vec());
            }
        }
    }
}

// run-test262.c:497..504. The original ignores ftw's stat/flag argument.
pub fn add_test_file(list: &mut NameList, filename: &[u8]) {
    if filename.ends_with(b".js") && !filename.ends_with(b"_FIXTURE.js") {
        namelist_add(list, None, filename);
    }
}
// run-test262.c:506..513. ftw follows links; sorting affects this traversal's
// appended slice only. Inaccessible branches keep the original ignored error.
pub fn enumerate_tests(list: &mut NameList, path: &[u8]) {
    fn walk(
        list: &mut NameList,
        path: &[u8],
        ancestry: &mut std::collections::HashSet<std::path::PathBuf>,
    ) {
        add_test_file(list, path);
        let native = filename_path(path);
        if !fs::metadata(&native).is_ok_and(|m| m.is_dir()) {
            return;
        }
        let Ok(canonical) = fs::canonicalize(&native) else {
            return;
        };
        if !ancestry.insert(canonical.clone()) {
            return;
        }
        if let Ok(entries) = fs::read_dir(&native) {
            for entry in entries.flatten() {
                #[cfg(unix)]
                let name = {
                    use std::os::unix::ffi::OsStrExt;
                    entry.file_name().as_bytes().to_vec()
                };
                #[cfg(windows)]
                let name = match crate::quickjs_libc::os_bytes(entry.file_name()) {
                    Ok(name) => name,
                    Err(_) => continue,
                };
                #[cfg(not(any(unix, windows)))]
                let name = entry.file_name().to_string_lossy().as_bytes().to_vec();
                walk(list, &compose_path(Some(path), &name), ancestry);
            }
        }
        ancestry.remove(&canonical);
    }
    let start = list.array.len();
    walk(list, path, &mut std::collections::HashSet::new());
    list.array[start..].sort_unstable_by(namelist_cmp_indirect);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum TestMode {
    #[default]
    DefaultNoStrict = 0,
    DefaultStrict = 1,
    NoStrict = 2,
    Strict = 3,
    All = 4,
}
#[derive(Default, Debug)]
pub struct RunnerConfig {
    pub test_list: NameList,
    pub exclude_list: NameList,
    pub exclude_dir_list: NameList,
    pub new_style: bool,
    pub test_mode: TestMode,
    pub skip_async: bool,
    pub skip_module: bool,
    pub verbose: bool,
    pub harness_dir: Option<Vec<u8>>,
    pub harness_exclude: Option<Vec<u8>>,
    pub harness_features: Option<Vec<u8>>,
    pub harness_skip_features: Option<Vec<u8>>,
    pub error_filename: Option<Vec<u8>>,
    pub report_filename: Option<Vec<u8>>,
    pub harness_skip_features_count: Option<Vec<i32>>,
}
#[derive(Debug)]
pub struct RunnerError {
    pub exit: i32,
    pub message: Vec<u8>,
}
impl RunnerError {
    fn io(filename: &[u8], error: io::Error) -> Self {
        let mut message = b"run-test262: ".to_vec();
        message.extend_from_slice(filename);
        message.extend_from_slice(b": ");
        let text = error.to_string();
        let suffix = error
            .raw_os_error()
            .map(|code| format!(" (os error {code})"));
        let text = suffix
            .as_ref()
            .and_then(|suffix| text.strip_suffix(suffix))
            .unwrap_or(&text);
        message.extend_from_slice(text.as_bytes());
        message.push(b'\n');
        Self { exit: 1, message }
    }
}
// run-test262.c:1100..1255. Preserve fgets chunks, substring ignore matching,
// exact untrimmed names before '=' and the config->exclude fallthrough.
pub fn load_config(
    config: &mut RunnerConfig,
    filename: &[u8],
    ignore: &[u8],
    output: &mut dyn Write,
) -> Result<(), RunnerError> {
    let data = fs::read(filename_path(filename)).map_err(|e| RunnerError::io(filename, e))?;
    let base = get_basename(filename);
    let mut section = 0;
    let mut lineno = 0;
    let mut pos = 0;
    while pos < data.len() {
        lineno += 1;
        let max = (pos + 1023).min(data.len());
        let end = data[pos..max]
            .iter()
            .position(|&c| c == b'\n')
            .map(|i| pos + i + 1)
            .unwrap_or(max);
        let chunk = &data[pos..end];
        pos = end;
        let chunk = &chunk[..chunk.iter().position(|&c| c == 0).unwrap_or(chunk.len())];
        let line = str_strip(chunk);
        if line.is_empty() || matches!(line[0], b'#' | b';') {
            continue;
        }
        if line[0] == b'[' {
            let name = &line[1..];
            let name = &name[..name.iter().position(|&c| c == b']').unwrap_or(name.len())];
            section = match name {
                b"config" => 1,
                b"exclude" => 2,
                b"features" => 3,
                b"tests" => 4,
                _ => 0,
            };
            continue;
        }
        let (p, q) = if let Some(i) = line.iter().position(|&c| c == b'=') {
            (&line[..i], Some(str_strip(&line[i + 1..])))
        } else {
            (line, None)
        };
        if section == 1 {
            let Some(q) = q else {
                let _ = output.write_all(filename);
                let _ = writeln!(output, ":{lineno}: syntax error");
                continue;
            };
            if substring(ignore, p, 0).is_some() {
                let _ = output.write_all(filename);
                let _ = write!(output, ":{lineno}: ignoring ");
                let _ = output.write_all(p);
                let _ = output.write_all(b"=");
                let _ = output.write_all(q);
                let _ = output.write_all(b"\n");
                continue;
            }
            match p {
                b"style" => {
                    config.new_style = q == b"new";
                    continue;
                }
                b"testdir" => {
                    enumerate_tests(&mut config.test_list, &compose_path(base.as_deref(), q));
                    continue;
                }
                b"harnessdir" => {
                    config.harness_dir = Some(compose_path(base.as_deref(), q));
                    continue;
                }
                b"harnessexclude" => {
                    str_append(&mut config.harness_exclude, b" ", q);
                    continue;
                }
                b"features" => {
                    str_append(&mut config.harness_features, b" ", q);
                    continue;
                }
                b"skip-features" => {
                    str_append(&mut config.harness_skip_features, b" ", q);
                    continue;
                }
                b"mode" => {
                    config.test_mode = match q {
                        b"default" | b"default-nostrict" => TestMode::DefaultNoStrict,
                        b"default-strict" => TestMode::DefaultStrict,
                        b"nostrict" => TestMode::NoStrict,
                        b"strict" => TestMode::Strict,
                        b"all" | b"both" => TestMode::All,
                        _ => {
                            let mut message = b"run-test262: unknown test mode: ".to_vec();
                            message.extend_from_slice(q);
                            message.push(b'\n');
                            return Err(RunnerError { exit: 2, message });
                        }
                    };
                    continue;
                }
                b"strict" => {
                    if matches!(q, b"skip" | b"no") {
                        config.test_mode = TestMode::NoStrict;
                    }
                    continue;
                }
                b"nostrict" => {
                    if matches!(q, b"skip" | b"no") {
                        config.test_mode = TestMode::Strict;
                    }
                    continue;
                }
                b"async" => {
                    config.skip_async = q != b"yes";
                    continue;
                }
                b"module" => {
                    config.skip_module = q != b"yes";
                    continue;
                }
                b"verbose" => {
                    config.verbose = q == b"yes";
                    continue;
                }
                b"errorfile" => {
                    config.error_filename = Some(compose_path(base.as_deref(), q));
                    continue;
                }
                b"excludefile" => {
                    let path = compose_path(base.as_deref(), q);
                    namelist_load(&mut config.exclude_list, &path)
                        .map_err(|e| RunnerError::io(&path, e))?;
                    continue;
                }
                b"reportfile" => {
                    config.report_filename = Some(compose_path(base.as_deref(), q));
                    continue;
                }
                _ => {}
            }
        }
        match section {
            1 | 2 => namelist_add(&mut config.exclude_list, base.as_deref(), p),
            3 => {
                if q.is_none_or(|q| q == b"yes") {
                    str_append(&mut config.harness_features, b" ", p);
                } else {
                    str_append(&mut config.harness_skip_features, b" ", p);
                }
            }
            4 => namelist_add(&mut config.test_list, base.as_deref(), p),
            _ => {}
        }
    }
    Ok(())
}
// run-test262.c:1582..1609. Return an unterminated comment warning separately
// so the host preserves the original stdout-flush/stderr diagnostic boundary.
pub fn extract_desc(buffer: &[u8], style: u8) -> (Option<Vec<u8>>, Option<&'static [u8]>) {
    let mut p = 0;
    while byte(buffer, p) != 0 {
        if byte(buffer, p) == b'/'
            && byte(buffer, p + 1) == b'*'
            && byte(buffer, p + 2) == style
            && byte(buffer, p + 3) != b'/'
        {
            p += 3;
            let start = p;
            while byte(buffer, p) != 0 && (byte(buffer, p) != b'*' || byte(buffer, p + 1) != b'/') {
                p += 1;
            }
            if byte(buffer, p) == 0 {
                return (None, Some(b"run-test262: Expecting end of desc comment\n"));
            }
            return (Some(buffer[start..p].to_vec()), None);
        }
        p += 1;
    }
    (None, None)
}

#[derive(Default, Debug)]
pub struct PreparedTest {
    pub harness: Vec<u8>,
    pub include_list: NameList,
    pub error_type: Option<Vec<u8>>,
    pub is_negative: bool,
    pub is_nostrict: bool,
    pub is_onlystrict: bool,
    pub is_async: bool,
    pub is_module: bool,
    pub skip: bool,
    pub can_block: bool,
    pub use_strict: bool,
    pub use_nostrict: bool,
    pub diagnostics: Vec<u8>,
    pub warnings: Vec<u8>,
}
// run-test262.c:1772..1977. Complete preparation half of run_test, retaining
// both harness formats, include/features/negative flags and all five modes.
pub fn prepare_test(config: &mut RunnerConfig, filename: &[u8], buffer: &[u8]) -> PreparedTest {
    let mut result = PreparedTest {
        can_block: true,
        ..PreparedTest::default()
    };
    result.harness = if let Some(harness) = &config.harness_dir {
        harness.clone()
    } else {
        let mut harness = Vec::new();
        if let Some(p) = substring(filename, b"test/", 0) {
            harness.extend_from_slice(&filename[..p]);
            harness.extend_from_slice(if config.new_style {
                b"harness"
            } else {
                b"test/harness"
            });
        }
        harness.truncate(1023);
        harness
    };
    namelist_add(&mut result.include_list, None, b"sta.js");
    if config.new_style {
        namelist_add(&mut result.include_list, None, b"assert.js");
        let (desc, warning) = extract_desc(buffer, b'-');
        if let Some(warning) = warning {
            result.warnings.extend_from_slice(warning);
        }
        if let Some(desc) = desc {
            let mut state = 0;
            let mut p = find_tag(&desc, b"includes:", &mut state);
            while let Some(ifile) = get_option(&desc, &mut p, &mut state) {
                if find_word(config.harness_exclude.as_deref(), &ifile).is_some() {
                    result.skip = true;
                } else {
                    namelist_add(&mut result.include_list, None, &ifile);
                }
            }
            p = find_tag(&desc, b"flags:", &mut state);
            while let Some(option) = get_option(&desc, &mut p, &mut state) {
                match option.as_slice() {
                    b"noStrict" | b"raw" => {
                        result.is_nostrict = true;
                        result.skip |= config.test_mode == TestMode::Strict;
                    }
                    b"onlyStrict" => {
                        result.is_onlystrict = true;
                        result.skip |= config.test_mode == TestMode::NoStrict;
                    }
                    b"async" => {
                        result.is_async = true;
                        result.skip |= config.skip_async;
                    }
                    b"module" => {
                        result.is_module = true;
                        result.skip |= config.skip_module;
                    }
                    b"CanBlockIsFalse" => result.can_block = false,
                    _ => {}
                }
            }
            if let Some(p) = find_tag(&desc, b"negative:", &mut state) {
                if let Some(q) = find_tag(&desc[p..], b"type:", &mut state) {
                    let mut q = p + q;
                    while c_space(byte(&desc, q)) {
                        q += 1;
                    }
                    let start = q;
                    while !matches!(byte(&desc, q), 0 | b' ' | b'\n') {
                        q += 1;
                    }
                    result.error_type = Some(desc[start..q].to_vec());
                }
                result.is_negative = true;
            }
            p = find_tag(&desc, b"features:", &mut state);
            while let Some(option) = get_option(&desc, &mut p, &mut state) {
                if find_word(config.harness_features.as_deref(), &option).is_some() {
                    continue;
                }
                if let Some(offset) = find_word(config.harness_skip_features.as_deref(), &option) {
                    if let Some(counts) = &mut config.harness_skip_features_count {
                        counts[offset] = counts[offset].wrapping_add(1);
                    }
                } else {
                    result.diagnostics.extend_from_slice(filename);
                    result
                        .diagnostics
                        .extend_from_slice(b":1: unknown feature: ");
                    result.diagnostics.extend_from_slice(&option);
                    result.diagnostics.push(b'\n');
                }
                result.skip = true;
            }
        }
        if result.is_async {
            namelist_add(&mut result.include_list, None, b"doneprintHandle.js");
        }
    } else {
        let mut p = 0;
        while let Some(found) = substring(buffer, b"$INCLUDE(\"", p) {
            p = found + 10;
            let len = buffer[p..]
                .iter()
                .position(|&c| c == b'"' || c == 0)
                .unwrap_or(buffer.len() - p);
            let ifile = &buffer[p..p + len];
            if find_word(config.harness_exclude.as_deref(), ifile).is_some() {
                result.skip = true;
            } else {
                namelist_add(&mut result.include_list, None, ifile);
            }
            p += 1;
        }
        let (desc, warning) = extract_desc(buffer, b'*');
        if let Some(warning) = warning {
            result.warnings.extend_from_slice(warning);
        }
        if let Some(desc) = desc {
            if substring(&desc, b"@noStrict", 0).is_some() {
                result.is_nostrict = true;
                result.skip |= config.test_mode == TestMode::Strict;
            }
            if substring(&desc, b"@onlyStrict", 0).is_some() {
                result.is_onlystrict = true;
                result.skip |= config.test_mode == TestMode::NoStrict;
            }
            if substring(&desc, b"@negative", 0).is_some() {
                result.is_negative = true;
            }
        }
    }
    match config.test_mode {
        TestMode::DefaultNoStrict => {
            if result.is_onlystrict {
                result.use_strict = true;
            } else {
                result.use_nostrict = true;
            }
        }
        TestMode::DefaultStrict => {
            if result.is_nostrict {
                result.use_nostrict = true;
            } else {
                result.use_strict = true;
            }
        }
        TestMode::NoStrict => result.use_nostrict = !result.is_onlystrict,
        TestMode::Strict => result.use_strict = !result.is_nostrict,
        TestMode::All => {
            if result.is_module {
                result.use_nostrict = true;
            } else {
                result.use_strict = !result.is_nostrict;
                result.use_nostrict = !result.is_onlystrict;
            }
        }
    }
    result
}
// run-test262.c:1932..1943. stdout report bytes are distinct from diagnostics.
pub fn test_report_line(test: &PreparedTest, filename: &[u8], index: i32) -> Option<Vec<u8>> {
    if index < 0 {
        return None;
    }
    let mut output = format!("{index}: ").into_bytes();
    output.extend_from_slice(filename);
    for (enabled, text) in [
        (test.is_nostrict, &b"  @noStrict"[..]),
        (test.is_onlystrict, b"  @onlyStrict"),
        (test.is_async, b"  async"),
        (test.is_module, b"  module"),
        (test.is_negative, b"  @negative"),
        (test.skip, b"  SKIPPED"),
    ] {
        if enabled {
            output.extend_from_slice(text);
        }
    }
    output.push(b'\n');
    Some(output)
}

// run-test262.c:945..955. Transfer the null-context loader's bytes into owned
// storage before releasing them through its paired Rust allocator.
pub fn load_file(filename: &[u8]) -> Result<Vec<u8>, RunnerError> {
    let filename = &filename[..filename
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(filename.len())];
    let name = CString::new(filename).unwrap();
    let mut len = 0;
    unsafe {
        let buf = crate::quickjs_libc::js_load_file(std::ptr::null_mut(), &mut len, name.as_ptr());
        if buf.is_null() {
            return Err(RunnerError::io(filename, io::Error::last_os_error()));
        }
        let result = std::slice::from_raw_parts(buf, len).to_vec();
        crate::quickjs_libc::js_free_file_buffer(std::ptr::null_mut(), buf, len);
        Ok(result)
    }
}
// run-test262.c:957..963.
pub unsafe fn json_module_init_test(ctx: *mut JSContext, m: *mut JSModuleDef) -> i32 {
    let val = JS_GetModulePrivateValue(ctx, m);
    JS_SetModuleExport(ctx, m, c"default".as_ptr(), val);
    0
}
// run-test262.c:965..1021. Bare specifiers use the original test filename's
// directory; names already containing '/' remain untouched.
pub unsafe fn js_module_loader_test(
    ctx: *mut JSContext,
    module_name: *const c_char,
    opaque: *mut c_void,
    attributes: JSValueConst,
) -> *mut JSModuleDef {
    let original = CStr::from_ptr(module_name).to_bytes();
    let mut owned = None;
    if !original.contains(&b'/') {
        let filename = CStr::from_ptr(opaque.cast()).to_bytes();
        if let Some(slash) = filename.iter().rposition(|&c| c == b'/') {
            let mut path = filename[..slash].to_vec();
            path.push(b'/');
            path.extend_from_slice(original);
            path.truncate(1023);
            owned = Some(CString::new(path).unwrap());
        }
    }
    let name = owned
        .as_ref()
        .map(|name| name.as_ptr())
        .unwrap_or(module_name);
    let mut len = 0;
    let buf = crate::quickjs_libc::js_load_file(ctx, &mut len, name);
    if buf.is_null() {
        let parts = [
            &b"could not load module filename '"[..],
            CStr::from_ptr(name).to_bytes(),
            &b"'"[..],
        ];
        JS_ThrowReferenceError(ctx, JSErrorMessage::Pieces(&parts));
        return std::ptr::null_mut();
    }
    if crate::quickjs_libc::js_module_test_json(ctx, attributes) == 1 {
        let val = JS_ParseJSON(ctx, buf.cast(), len, name);
        js_free(ctx, buf.cast());
        if JS_IsException(val) != 0 {
            return std::ptr::null_mut();
        }
        let m = JS_NewCModule(ctx, name, Some(json_module_init_test));
        if m.is_null() {
            JS_FreeValue(ctx, val);
            return m;
        }
        JS_AddModuleExport(ctx, m, c"default".as_ptr());
        JS_SetModulePrivateValue(ctx, m, val);
        m
    } else {
        let func = JS_Eval(
            ctx,
            buf.cast(),
            len,
            name,
            JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY,
        );
        js_free(ctx, buf.cast());
        if JS_IsException(func) != 0 {
            return std::ptr::null_mut();
        }
        let m = JS_VALUE_GET_PTR(func).cast();
        JS_FreeValue(ctx, func);
        m
    }
}

#[cfg(any(unix, windows))]
include!("run_test262_agents.rs");
#[cfg(any(unix, windows))]
include!("run_test262_eval.rs");

#[cfg(any(unix, windows))]
include!("run_test262_runner.rs");

#[cfg(any(unix, windows))]
include!("run_test262_cli.rs");

#[cfg(any(unix, windows))]
include!("run_test262_main.rs");
