use std::io;

// cpp: browser/browser.cc:253-288
fn normalize_hierarchical_url(url: String) -> String {
    let Some(scheme) = url.find("://") else {
        return url;
    };
    let Some(path_start) = url[scheme + 3..].find('/').map(|index| scheme + 3 + index) else {
        return url;
    };
    let suffix_start = url[path_start..]
        .find(['?', '#'])
        .map(|index| path_start + index)
        .unwrap_or(url.len());
    let path = &url[path_start..suffix_start];
    let mut segments = Vec::new();
    for segment in path[1..].split('/') {
        match segment {
            ".." => {
                segments.pop();
            }
            "" | "." => {}
            _ => segments.push(segment),
        }
    }
    let mut normalized = String::from(&url[..path_start]);
    normalized.push('/');
    normalized.push_str(&segments.join("/"));
    if path.len() > 1 && path.ends_with('/') && !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized.push_str(&url[suffix_start..]);
    normalized
}

#[allow(non_snake_case)]
pub fn ResolveUrl(base: &str, href: &str) -> io::Result<String> {
    // cpp: browser/browser.cc:290-329
    if href.is_empty() {
        return Ok(base.to_owned());
    }
    let colon = href.find(':');
    let slash = href.find('/');
    if colon.is_some_and(|colon| slash.is_none_or(|slash| colon < slash)) {
        return Ok(normalize_hierarchical_url(href.to_owned()));
    }
    if href.starts_with('#') {
        return Ok(format!(
            "{}{}",
            base.split('#').next().unwrap_or(base),
            href
        ));
    }
    if href.starts_with('?') {
        let end = base.find(['?', '#']).unwrap_or(base.len());
        return Ok(format!("{}{}", &base[..end], href));
    }
    if href.starts_with("//") {
        if let Some(scheme_end) = base.find(':') {
            return Ok(normalize_hierarchical_url(format!(
                "{}{}",
                &base[..scheme_end + 1],
                href
            )));
        }
    }
    if href.starts_with('/') {
        if let Some(authority) = base.find("//") {
            let path = base[authority + 2..]
                .find('/')
                .map(|offset| authority + 2 + offset)
                .unwrap_or(base.len());
            return Ok(normalize_hierarchical_url(format!(
                "{}{}",
                &base[..path],
                href
            )));
        }
    }
    let base_path = base.split(['?', '#']).next().unwrap_or(base);
    let Some(last_slash) = base_path.rfind('/') else {
        return Ok(href.to_owned());
    };
    Ok(normalize_hierarchical_url(format!(
        "{}{}",
        &base_path[..last_slash + 1],
        href
    )))
}

// cpp: browser/browser.cc:467-474
fn Unquote(text: &str) -> &str {
    let text = text.trim_matches(|character: char| character.is_ascii_whitespace());
    if text.len() >= 2
        && ((text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('"') && text.ends_with('"')))
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

// cpp: browser/browser.cc:525-573
#[allow(non_snake_case)]
pub fn ResolveCSSURLs(value: &mut String, base_url: &str) -> io::Result<Vec<String>> {
    let bytes = value.as_bytes();
    if !bytes.contains(&b'(') {
        return Ok(Vec::new());
    }
    let mut urls = Vec::new();
    // Most declarations have no URL. Search the original bytes and allocate
    // replacement storage only when a nonempty, complete URL is rewritten.
    let mut output = String::new();
    let mut copied = 0;
    let mut cursor = 0;
    while cursor < value.len() {
        let Some(found) = bytes[cursor..]
            .windows(4)
            .position(|token| token.eq_ignore_ascii_case(b"url("))
            .map(|offset| cursor + offset)
        else {
            break;
        };
        if found > 0
            && (bytes[found - 1].is_ascii_alphanumeric() || matches!(bytes[found - 1], b'-' | b'_'))
        {
            cursor = found + 4;
            continue;
        }
        let argument_start = found + 4;
        let mut quote = 0_u8;
        let mut end = argument_start;
        while end < bytes.len() {
            let character = bytes[end];
            if quote != 0 {
                if character == b'\\' && end + 1 < bytes.len() {
                    end += 1;
                } else if character == quote {
                    quote = 0;
                }
            } else if character == b'\'' || character == b'"' {
                quote = character;
            } else if character == b')' {
                break;
            }
            end += 1;
        }
        if end == bytes.len() {
            break;
        }
        let source = Unquote(&value[argument_start..end]);
        if source.is_empty() {
            cursor = end + 1;
            continue;
        }
        let resolved = ResolveUrl(base_url, source)?;
        if urls.is_empty() {
            output.reserve(value.len());
        }
        output.push_str(&value[copied..found]);
        output.push_str("url(\"");
        for character in resolved.chars() {
            if character == '\\' || character == '"' {
                output.push('\\');
            }
            output.push(character);
        }
        output.push_str("\")");
        urls.push(resolved);
        copied = end + 1;
        cursor = end + 1;
    }
    if !urls.is_empty() {
        output.push_str(&value[copied..]);
        *value = output;
    }
    Ok(urls)
}

#[cfg(test)]
mod tests;
