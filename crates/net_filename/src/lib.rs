//! Pure source-owned filename selection and sanitization, shared by downloads
//! and encoded image drag promises. No DOM, UI, filesystem access or OS object.
//! Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
pub mod http_content_disposition;
mod mime;
use http_content_disposition::HttpContentDisposition;
use icu_bidi::filename_unicode::{ConvertToUTF16, IsIllegalAnywhere, IsIllegalAtEnds};
pub use mime::GetWellKnownMimeTypeFromExtension;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilenamePlatform {
    MacPosix,
    Windows,
}
/// Results of the native MIME database query for this one requested MIME.
/// These are values, not a callback into the embedding or a second MIME DB.
#[derive(Clone, Copy, Debug)]
pub struct PlatformMimeFacts<'a> {
    pub mime_type: &'a str,
    pub preferred_extension: Option<&'a str>,
    pub extensions: &'a [&'a str],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilenameError {
    RequiredPlatformMimeFacts,
    RequiredNativeWindowsDecoding,
}
#[derive(Clone, Copy, Debug)]
pub struct GenerateFileNameInput<'a> {
    pub url: &'a str,
    pub content_disposition: &'a [u8],
    pub referrer_charset: &'a [u8],
    pub suggested_name: &'a str,
    pub mime_type: &'a str,
    pub default_name: &'a str,
    pub should_replace_extension: bool,
    pub platform: FilenamePlatform,
    pub platform_mime: Option<PlatformMimeFacts<'a>>,
}
/// Already-parsed URL components are also accepted directly, allowing the
/// embedding's canonical URL owner to preserve its parser/validity contract.
#[derive(Clone, Copy, Debug, Default)]
pub struct FilenameURL<'a> {
    pub is_valid: bool,
    pub scheme: &'a str,
    pub host: &'a str,
    pub path: &'a str,
    pub has_query: bool,
}
/// Source base/strings/escape.cc:538-590: binary percent decoding, including
/// reserved bytes, invalid UTF-8 and NUL. '+' remains a literal plus.
pub(crate) fn UnescapeBinaryURLComponent(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            if let (Some(high), Some(low)) = (
                (input[i + 1] as char).to_digit(16),
                (input[i + 2] as char).to_digit(16),
            ) {
                output.push((high * 16 + low) as u8);
                i += 3;
                continue;
            }
        }
        output.push(input[i]);
        i += 1;
    }
    output
}
// GURL::ExtractFileName -> url_parse.cc:722-749. This is deliberately not
// Path::file_name: both slash kinds delimit, and a semicolon ends the filename.
fn ExtractFileName(path: &str) -> &str {
    let mut end = path.len();
    for (i, b) in path.bytes().enumerate().rev() {
        if b == b';' {
            end = i;
        } else if matches!(b, b'/' | b'\\') {
            return &path[i + 1..end];
        }
    }
    &path[..end]
}
// net/base/filename_util_internal.cc:135-172, Apple native strings are UTF-8.
fn GetFileNameFromURL(url: FilenameURL<'_>, charset: &[u8]) -> (String, bool) {
    if !url.is_valid
        || url.scheme.eq_ignore_ascii_case("about")
        || url.scheme.eq_ignore_ascii_case("data")
    {
        return (String::new(), false);
    }
    let bytes = UnescapeBinaryURLComponent(ExtractFileName(url.path).as_bytes());
    let name = String::from_utf8(bytes.clone()).unwrap_or_else(|_| {
        if !charset.is_empty() {
            ConvertToUTF16(&bytes, charset)
                .and_then(|s| String::from_utf16(&s).ok())
                .unwrap_or_default()
        } else {
            String::new()
        }
    });
    let overwrite = !name.is_empty() && url.has_query;
    (name, overwrite)
}
// source filename_util_internal.cc:85-128.
fn SanitizeGeneratedFileName(name: &mut String, platform: FilenamePlatform) {
    if name.is_empty() {
        return;
    }
    if platform == FilenamePlatform::Windows {
        let original_length = name.encode_utf16().count();
        let trimmed = name
            .trim_end_matches([' ', '.'])
            .trim_end_matches(char::is_whitespace)
            .to_owned();
        if trimmed.is_empty() {
            name.clear();
            return;
        }
        let length = trimmed.encode_utf16().count();
        *name = trimmed;
        name.extend(std::iter::repeat('_').take(original_length - length));
    }
    *name = name.trim_matches('.').replace(['/', '\\'], "_");
}
fn CouldBeShortName(name: &str) -> bool {
    let length = name.encode_utf16().count();
    if length > 12 {
        return false;
    }
    let dots: Vec<_> = name
        .match_indices('.')
        .map(|(i, _)| name[..i].encode_utf16().count())
        .collect();
    let valid = match dots.as_slice() {
        [] => length <= 8,
        [dot] => *dot > 0 && *dot <= 8 && dot + 4 >= length,
        _ => false,
    };
    valid
        && !name.chars().any(|ch| {
            IsIllegalAtEnds(ch) && ch != '.' && ch != '~' || "\"\\/[]:+|<>=;?,*".contains(ch)
        })
}
/// base/i18n/file_util_icu.cc:190-291 with source's '_' replacement. The
/// general-purpose function's illegal-at-ends replacement branches are not
/// needed: '_' is legal at both ends. ICU owns Cc, Cf, WSpace properties.
fn ReplaceIllegalCharactersInPath(name: &mut String, platform: FilenamePlatform) {
    let short = platform == FilenamePlatform::Windows && CouldBeShortName(name);
    let count = name.chars().count();
    *name = name
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            if IsIllegalAnywhere(ch)
                || (short && ch == '~')
                || ((i == 0 || i + 1 == count) && IsIllegalAtEnds(ch))
            {
                '_'
            } else {
                ch
            }
        })
        .collect();
}
// base/files/file_path.cc:141-191,552-588,662-683.
fn ExtensionPosition(name: &str, platform: FilenamePlatform) -> Option<usize> {
    if matches!(name, "." | "..") {
        return None;
    }
    let last = name.rfind('.')?;
    if last == 0 {
        return Some(last);
    }
    let Some(previous) = name[..last].rfind('.') else {
        return Some(last);
    };
    let combined = &name[previous + 1..];
    let suffix = &name[last + 1..];
    if combined.eq_ignore_ascii_case("user.js")
        || (["bz", "bz2", "gz", "lz", "lzma", "lzo", "xz", "z", "zst"]
            .iter()
            .any(|ext| suffix.eq_ignore_ascii_case(ext))
            && (2..=5).contains(&if platform == FilenamePlatform::Windows {
                name[previous..last].encode_utf16().count()
            } else {
                last - previous
            }))
    {
        Some(previous)
    } else {
        Some(last)
    }
}
fn ReplaceExtension(name: &str, extension: &str, platform: FilenamePlatform) -> String {
    if matches!(name, "" | "." | "..") {
        return String::new();
    }
    // base::FilePath truncates embedded NUL on construction.
    let name = name.split('\0').next().unwrap_or_default();
    let no_extension = &name[..ExtensionPosition(name, platform).unwrap_or(name.len())];
    if extension.is_empty() || extension == "." {
        return no_extension.to_owned();
    }
    format!(
        "{}{}{}",
        no_extension,
        if extension.starts_with('.') { "" } else { "." },
        extension
    )
}
fn IsShellIntegratedExtension(extension: &str) -> bool {
    ["local", "lnk", "scf", "url"]
        .iter()
        .any(|e| extension.eq_ignore_ascii_case(e))
        || (extension.starts_with('{') && extension.ends_with('}'))
}
fn IsReservedNameOnWindows(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let device = name.split('.').next().unwrap_or_default();
    matches!(name.as_str(), "desktop.ini" | "thumbs.db")
        || matches!(device, "con" | "prn" | "aux" | "nul" | "clock$")
        || (device.len() == 4
            && (device.starts_with("com") || device.starts_with("lpt"))
            && matches!(device.as_bytes()[3], b'1'..=b'9'))
}
fn EnsureSafeExtension(
    name: &str,
    mime_type: &str,
    ignore: bool,
    platform: FilenamePlatform,
    platform_mime: Option<PlatformMimeFacts<'_>>,
) -> Result<String, FilenameError> {
    let mut extension = ExtensionPosition(name, platform)
        .map_or("", |i| &name[i + 1..])
        .to_owned();
    if !mime_type.is_empty() && (extension.is_empty() || ignore) {
        // net::GetPreferredExtensionForMimeType consults the OS before either
        // hardcoded table. An unavailable query cannot be silently skipped.
        let facts = platform_mime
            .filter(|facts| facts.mime_type.eq_ignore_ascii_case(mime_type))
            .ok_or(FilenameError::RequiredPlatformMimeFacts)?;
        let mappings = mime::PRIMARY.iter().chain(mime::SECONDARY);
        let preferred = facts.preferred_extension.or_else(|| {
            mappings.clone().find_map(|&(mime, ext)| {
                (mime == mime_type && mime_type != "application/octet-stream")
                    .then(|| ext.split(',').next())
                    .flatten()
            })
        });
        if let Some(preferred) = preferred {
            let known = |ext: &str| {
                if matches!(mime_type, "*" | "*/*") {
                    return false;
                }
                let prefix = mime_type.strip_suffix("/*");
                facts.extensions.contains(&ext)
                    || mappings
                        .clone()
                        .filter(|(mime, _)| {
                            prefix.map_or_else(
                                || mime.eq_ignore_ascii_case(mime_type),
                                |prefix| {
                                    mime.get(..prefix.len() + 1).is_some_and(|start| {
                                        start.eq_ignore_ascii_case(&format!("{prefix}/"))
                                    })
                                },
                            )
                        })
                        .any(|(_, extensions)| extensions.split(',').any(|known| known == ext))
            };
            let final_extension = name.rfind('.').map_or("", |i| &name[i + 1..]);
            if !known(&extension) && !known(final_extension) {
                extension = preferred.to_owned();
            }
        }
    }
    if platform == FilenamePlatform::Windows && IsShellIntegratedExtension(&extension) {
        extension = "download".into();
    }
    let mut result = ReplaceExtension(name, &extension, platform);
    if platform == FilenamePlatform::Windows && IsReservedNameOnWindows(&result) {
        result.insert(0, '_');
    }
    Ok(result)
}
/// net/base/filename_util_internal.cc:256-376; filename_util_icu.cc:75-108.
pub fn GenerateFileNameWithURL(
    input: &GenerateFileNameInput<'_>,
    url: FilenameURL<'_>,
) -> Result<String, FilenameError> {
    // Raw HTTP octets use Apple's strict native UTF-8 fallback below. A
    // Windows ACP converter has not been translated into this package; do not
    // silently substitute the Apple branch for it on malformed UTF-8 input.
    if input.platform == FilenamePlatform::Windows
        && std::str::from_utf8(input.content_disposition).is_err()
    {
        return Err(FilenameError::RequiredNativeWindowsDecoding);
    }
    let mut filename =
        HttpContentDisposition::Parse(input.content_disposition, input.referrer_charset).filename;
    let from_disposition = !filename.is_empty();
    let mut overwrite = false;
    if filename.is_empty() {
        filename = input.suggested_name.to_owned();
    }
    if filename.is_empty() {
        if input.platform == FilenamePlatform::Windows
            && url.is_valid
            && !url.scheme.eq_ignore_ascii_case("about")
            && !url.scheme.eq_ignore_ascii_case("data")
        {
            let bytes = UnescapeBinaryURLComponent(ExtractFileName(url.path).as_bytes());
            if std::str::from_utf8(&bytes).is_err()
                && (input.referrer_charset.is_empty()
                    || ConvertToUTF16(&bytes, input.referrer_charset).is_none())
            {
                return Err(FilenameError::RequiredNativeWindowsDecoding);
            }
        }
        (filename, overwrite) = GetFileNameFromURL(url, input.referrer_charset);
    }
    if filename.is_empty() && input.default_name.is_empty() && url.is_valid && !url.host.is_empty()
    {
        filename = url.host.to_owned();
    }
    SanitizeGeneratedFileName(&mut filename, input.platform);
    if filename.chars().all(|c| matches!(c, '-' | '_')) {
        filename = if input.default_name.is_empty() {
            "download".into()
        } else {
            input.default_name.into()
        };
        overwrite = false;
    }
    ReplaceIllegalCharactersInPath(&mut filename, input.platform);
    // Source does not append a MIME extension to Content-Disposition names.
    let result = EnsureSafeExtension(
        &filename,
        if from_disposition {
            ""
        } else {
            input.mime_type
        },
        !from_disposition && (overwrite || input.should_replace_extension),
        input.platform,
        input.platform_mime,
    )?;
    // FilePathToString16 fails on an empty POSIX path; source retries default,
    // then its nonlocalized final fallback. Valid UTF-8 is retained throughout.
    Ok(if !result.is_empty() {
        result
    } else if !input.default_name.is_empty() {
        input.default_name.into()
    } else {
        "download".into()
    })
}
/// URL parsing follows the repository's existing `url` package contract. The
/// source filename algorithm itself can instead receive FilenameURL directly.
pub fn GenerateFileName(input: &GenerateFileNameInput<'_>) -> Result<String, FilenameError> {
    let parsed = url::Url::parse(input.url).ok();
    let facts = parsed
        .as_ref()
        .map_or(FilenameURL::default(), |url| FilenameURL {
            is_valid: true,
            scheme: url.scheme(),
            host: url.host_str().unwrap_or_default(),
            path: url.path(),
            has_query: url.query().is_some(),
        });
    GenerateFileNameWithURL(input, facts)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeDragFileName {
    pub filename: String,
    pub well_known_mime_type: &'static str,
}
/// content/public/common/drop_data.cc:73-91. MIME validation is intentionally
/// well-known-only. A native GetMimeTypeFromFile query after this API is a
/// separate platform operation; well_known_mime_type does not claim its result.
pub fn GetSafeFilenameForImageFileContents(
    url: &str,
    disposition: &str,
    extension: &str,
) -> Option<SafeDragFileName> {
    let filename = GenerateFileName(&GenerateFileNameInput {
        url,
        content_disposition: disposition.as_bytes(),
        referrer_charset: b"",
        suggested_name: "",
        mime_type: "",
        default_name: "",
        should_replace_extension: false,
        platform: FilenamePlatform::MacPosix,
        platform_mime: None,
    })
    .expect("empty MIME does not require a platform MIME query");
    let mime_type = GetWellKnownMimeTypeFromExtension(extension)?;
    if !mime_type
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("image/"))
    {
        return None;
    }
    Some(SafeDragFileName {
        filename: ReplaceExtension(&filename, extension, FilenamePlatform::MacPosix),
        well_known_mime_type: mime_type,
    })
}

/// The native database result for this exact extension. `None` in the
/// `mime_type` field means an actual unsuccessful native query; absence of
/// this struct means the required platform query has not been made.
#[derive(Clone, Copy, Debug)]
pub struct PlatformExtensionMime<'a> {
    pub extension: &'a str,
    pub mime_type: Option<&'a str>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MimeQueryRequired;
/// Source MimeUtil::GetMimeTypeFromFile/ExtensionHelper, mime_util.cc:332-339,
/// 349-402. The static primary map precedes the OS; secondary follows it.
pub fn GetMimeTypeFromFile(
    filename: &str,
    platform_mime: Option<PlatformExtensionMime<'_>>,
) -> Result<Option<String>, MimeQueryRequired> {
    let Some(position) = ExtensionPosition(filename, FilenamePlatform::MacPosix) else {
        return Ok(None);
    };
    let extension = &filename[position + 1..];
    if extension.len() > 65536 || extension.contains('\0') {
        return Ok(None);
    }
    let lookup = |mappings: &[(&str, &str)]| {
        mappings.iter().find_map(|&(mime, ext)| {
            ext.split(',')
                .any(|known| known.eq_ignore_ascii_case(extension))
                .then(|| mime.to_owned())
        })
    };
    if let Some(mime) = lookup(mime::PRIMARY) {
        return Ok(Some(mime));
    }
    let platform = platform_mime
        .filter(|facts| facts.extension == extension)
        .ok_or(MimeQueryRequired)?;
    Ok(platform
        .mime_type
        .map(str::to_owned)
        .or_else(|| lookup(mime::SECONDARY)))
}
/// Same source extension projection used to prepare one native MIME query.
pub fn FileNameExtension(filename: &str) -> Option<&str> {
    ExtensionPosition(filename, FilenamePlatform::MacPosix)
        .map(|position| &filename[position + 1..])
}
