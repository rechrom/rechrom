//! HTML responsive-image candidate parsing.
//!
//! This is the package-local translation of Blink's
//! `html/parser/html_srcset_parser.cc`.  It deliberately owns the HTML token
//! recovery rules (in particular commas in data URLs); callers only supply
//! the source-size and effective device scale selected by CSS/media owners.
#![allow(non_snake_case)]

#[derive(Clone, Debug, PartialEq)]
pub struct ImageCandidate {
    pub source: String,
    pub density: f64,
    pub resource_width: Option<u32>,
    src_origin: bool,
}

#[derive(Default)]
struct DescriptorParsingResult {
    density: Option<f64>,
    resource_width: Option<u32>,
    resource_height: Option<u32>,
}

fn is_html_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ')
}

fn valid_float(text: &str) -> Option<f64> {
    // HTML's "valid floating-point number" grammar.  A leading plus is
    // rejected by Blink's descriptor parser even though Rust accepts it.
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes[0] == b'+' {
        return None;
    }
    let mut i = usize::from(bytes[0] == b'-');
    let integer_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let integer_digits = i - integer_start;
    let mut fraction_digits = 0;
    let mut has_decimal_point = false;
    if i < bytes.len() && bytes[i] == b'.' {
        has_decimal_point = true;
        i += 1;
        let fraction_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        fraction_digits = i - fraction_start;
    }
    if (integer_digits == 0 && fraction_digits == 0) || (has_decimal_point && fraction_digits == 0)
    {
        return None;
    }
    if i < bytes.len() && matches!(bytes[i], b'e' | b'E') {
        i += 1;
        if i < bytes.len() && matches!(bytes[i], b'+' | b'-') {
            i += 1;
        }
        let exponent_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == exponent_start {
            return None;
        }
    }
    if i != bytes.len() {
        return None;
    }
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn parse_descriptors(
    attribute: &str,
    descriptors: &[(usize, usize)],
) -> Option<DescriptorParsingResult> {
    let mut result = DescriptorParsingResult::default();
    for &(start, end) in descriptors {
        if start == end {
            continue;
        }
        let descriptor = &attribute[start..end];
        let (number, suffix) = descriptor.split_at(descriptor.len() - 1);
        match suffix.as_bytes()[0] {
            b'w' => {
                if result.density.is_some() || result.resource_width.is_some() {
                    return None;
                }
                if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                result.resource_width = number.parse::<u32>().ok().filter(|value| *value > 0);
                result.resource_width?;
            }
            b'h' => {
                if result.density.is_some() || result.resource_height.is_some() {
                    return None;
                }
                if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                result.resource_height = number.parse::<u32>().ok().filter(|value| *value > 0);
                result.resource_height?;
            }
            b'x' => {
                if result.density.is_some()
                    || result.resource_width.is_some()
                    || result.resource_height.is_some()
                {
                    return None;
                }
                result.density = valid_float(number).filter(|value| *value >= 0.0);
                result.density?;
            }
            _ => return None,
        }
    }
    if result.resource_height.is_some() && result.resource_width.is_none() {
        return None;
    }
    Some(result)
}

fn tokenize_descriptors(attribute: &str, position: &mut usize) -> Vec<(usize, usize)> {
    #[derive(Clone, Copy)]
    enum State {
        TokenStart,
        InParenthesis,
        AfterToken,
    }
    let bytes = attribute.as_bytes();
    let mut state = State::TokenStart;
    let mut descriptor_start = Some(*position);
    let mut descriptors = Vec::new();
    let append = |descriptors: &mut Vec<_>, start: &mut Option<usize>, end| {
        if let Some(begin) = *start {
            if end > begin {
                descriptors.push((begin, end));
            }
        }
        *start = None;
    };
    loop {
        match state {
            State::TokenStart => {
                if *position >= bytes.len() {
                    append(&mut descriptors, &mut descriptor_start, bytes.len());
                    return descriptors;
                }
                match bytes[*position] {
                    b',' => {
                        append(&mut descriptors, &mut descriptor_start, *position);
                        *position += 1;
                        return descriptors;
                    }
                    byte if is_html_space(byte) => {
                        append(&mut descriptors, &mut descriptor_start, *position);
                        descriptor_start = Some(*position + 1);
                        state = State::AfterToken;
                    }
                    b'(' => {
                        descriptor_start.get_or_insert(*position);
                        state = State::InParenthesis;
                    }
                    _ => {
                        descriptor_start.get_or_insert(*position);
                    }
                }
            }
            State::InParenthesis => {
                if *position >= bytes.len() {
                    append(&mut descriptors, &mut descriptor_start, bytes.len());
                    return descriptors;
                }
                if bytes[*position] == b')' {
                    descriptor_start.get_or_insert(*position + 1);
                    state = State::TokenStart;
                } else {
                    descriptor_start.get_or_insert(*position);
                }
            }
            State::AfterToken => {
                if *position >= bytes.len() {
                    return descriptors;
                }
                if !is_html_space(bytes[*position]) {
                    state = State::TokenStart;
                    descriptor_start = Some(*position);
                    continue;
                }
            }
        }
        *position += 1;
    }
}

// cpp: html_srcset_parser.cc ParseImageCandidatesFromSrcsetAttribute.
pub fn ParseImageCandidatesFromSrcsetAttribute(attribute: &str) -> Vec<ImageCandidate> {
    let bytes = attribute.as_bytes();
    let mut position = 0;
    let mut candidates = Vec::new();
    while position < bytes.len() {
        while position < bytes.len() && (is_html_space(bytes[position]) || bytes[position] == b',')
        {
            position += 1;
        }
        if position == bytes.len() {
            break;
        }
        let url_start = position;
        while position < bytes.len() && !is_html_space(bytes[position]) {
            position += 1;
        }
        let mut url_end = position;
        let result = if bytes[position - 1] == b',' {
            while url_end > url_start && bytes[url_end - 1] == b',' {
                url_end -= 1;
            }
            if url_end == url_start {
                continue;
            }
            DescriptorParsingResult::default()
        } else {
            while position < bytes.len() && is_html_space(bytes[position]) {
                position += 1;
            }
            let descriptors = tokenize_descriptors(attribute, &mut position);
            let Some(result) = parse_descriptors(attribute, &descriptors) else {
                continue;
            };
            result
        };
        candidates.push(ImageCandidate {
            source: attribute[url_start..url_end].to_owned(),
            density: result.density.unwrap_or(-1.0),
            resource_width: result.resource_width,
            src_origin: false,
        });
    }
    candidates
}

// cpp: html_srcset_parser.cc PickBestImageCandidate and current enabled
// SelectionLogic branch. Cache preference stays with the resource owner.
pub fn BestFitSourceForImageAttributes(
    device_scale_factor: f64,
    source_size: f64,
    src: Option<&str>,
    srcset: Option<&str>,
) -> Option<ImageCandidate> {
    BestFitSourceForImageAttributesWithCache(device_scale_factor, source_size, src, srcset, |_| {
        false
    })
}

fn is_data_url(source: &str) -> bool {
    source
        .trim_matches(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
        .get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
}

// cpp: html_srcset_parser.cc PickBestImageCandidate and
// AvoidDownloadIfHigherDensityResourceIsInCache. The HTML owner retains
// candidate ordering; the loader supplies a package-neutral loaded-resource
// predicate, so this parser never depends on ResourceFetcher or its cache.
pub fn BestFitSourceForImageAttributesWithCache(
    device_scale_factor: f64,
    source_size: f64,
    src: Option<&str>,
    srcset: Option<&str>,
    mut is_loaded: impl FnMut(&str) -> bool,
) -> Option<ImageCandidate> {
    let Some(srcset) = srcset else {
        return src.map(|source| ImageCandidate {
            source: source.to_owned(),
            density: 1.0,
            resource_width: None,
            src_origin: true,
        });
    };
    let mut candidates = ParseImageCandidatesFromSrcsetAttribute(srcset);
    if let Some(source) = src.filter(|source| !source.is_empty()) {
        candidates.push(ImageCandidate {
            source: source.to_owned(),
            density: -1.0,
            resource_width: None,
            src_origin: true,
        });
    }
    if candidates.is_empty() {
        return None;
    }
    let mut ignore_src = false;
    for candidate in &mut candidates {
        if let Some(width) = candidate.resource_width {
            candidate.density = width as f64 / source_size;
            ignore_src = true;
        } else if candidate.density < 0.0 {
            candidate.density = 1.0;
        }
    }
    candidates.sort_by(|left, right| left.density.total_cmp(&right.density));
    let mut deduplicated = Vec::new();
    let mut previous_density = -1.0;
    for candidate in candidates {
        let density = candidate.density;
        if density != previous_density && (!ignore_src || !candidate.src_origin) {
            deduplicated.push(candidate);
        }
        previous_density = density;
    }
    let target = if device_scale_factor.is_finite() && device_scale_factor > 0.0 {
        device_scale_factor
    } else {
        1.0
    };
    let mut winner = deduplicated
        .iter()
        .position(|candidate| candidate.density >= target)
        .unwrap_or(deduplicated.len().saturating_sub(1));
    // Blink scans from the highest-density candidate down to the ordinary
    // winner. A loaded higher-density resource avoids a new download; data
    // URLs have the same preference because they require no network fetch.
    for index in ((winner + 1)..deduplicated.len()).rev() {
        let source = &deduplicated[index].source;
        if is_loaded(source) || is_data_url(source) {
            winner = index;
            break;
        }
    }
    deduplicated.into_iter().nth(winner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_urls_and_descriptor_recovery_match_blink() {
        let candidates = ParseImageCandidatesFromSrcsetAttribute(
            "data:,a 1w, bad 1x 2x, data:,b 2x, ignored 3h, final 4w 2h",
        );
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.source.as_str())
                .collect::<Vec<_>>(),
            ["data:,a", "data:,b", "final"]
        );
        assert_eq!(candidates[0].resource_width, Some(1));
        assert_eq!(candidates[1].density, 2.0);
        assert_eq!(candidates[2].resource_width, Some(4));
    }

    #[test]
    fn loaded_higher_density_candidate_avoids_a_download() {
        let candidate = BestFitSourceForImageAttributesWithCache(
            1.0,
            100.0,
            None,
            Some("one.png 1x, two.png 2x, three.png 3x"),
            |source| source == "two.png",
        )
        .unwrap();
        assert_eq!(candidate.source, "two.png");
    }
}
