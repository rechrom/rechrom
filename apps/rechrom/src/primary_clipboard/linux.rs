//! ClipboardOzone text/HTML/smart-paste MIME contract. The standard clipboard
//! and PRIMARY are distinct native selections, never a process-local cache.
use super::{ClipboardHostError, PrimaryClipboardSnapshot};
use interaction::default::core::editing::commands::clipboard_commands::ClipboardHtml;

pub(super) const READ_FORMATS: &[&str] = &[
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "text/plain",
    "STRING",
    "text/html",
    "chromium/x-webkit-paste",
    "image/png",
    "image/svg+xml",
    "text/rtf",
    "text/uri-list",
    "chromium/x-web-custom-data",
];
pub(super) fn decode(
    data: Vec<(String, Vec<u8>)>,
) -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
    if !data
        .iter()
        .any(|(mime, _)| READ_FORMATS.contains(&mime.as_str()))
    {
        return Err(ClipboardHostError::Unavailable);
    }
    let value = |mime: &str| {
        data.iter()
            .find(|(format, _)| format == mime)
            .map(|(_, bytes)| bytes.as_slice())
    };
    let plain_text = READ_FORMATS[..4]
        .iter()
        .find_map(|mime| {
            value(mime).map(|bytes| {
                if *mime == "STRING" {
                    bytes.iter().copied().map(char::from).collect()
                } else {
                    String::from_utf8_lossy(bytes).into_owned()
                }
            })
        })
        .unwrap_or_default();
    let html = value("text/html").map(|bytes| {
        // ClipboardOzone::ReadHTML exposes all UTF-8 markup as the fragment and
        // an empty source URL (WriteHTML likewise ignores its source URL).
        let markup = String::from_utf8_lossy(bytes).into_owned();
        ClipboardHtml {
            fragment_start: 0,
            fragment_end: markup.encode_utf16().count() as u32,
            markup,
            source_url: String::new(),
        }
    });
    let mut snapshot = PrimaryClipboardSnapshot {
        plain_text,
        html,
        smart_replace: value("chromium/x-webkit-paste").is_some(),
        rtf: value("text/rtf").map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
        svg: value("image/svg+xml").map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
        uri_list: value("text/uri-list").map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
        png: value("image/png").map(|bytes| super::formats::png(bytes.to_vec())),
        custom_data: value("chromium/x-web-custom-data")
            .map(super::formats::decode_custom)
            .unwrap_or_default(),
        ..Default::default()
    };
    super::formats::admit_uris(&mut snapshot)?;
    Ok(snapshot)
}
pub(super) fn encode(
    snapshot: &PrimaryClipboardSnapshot,
) -> Result<Vec<(String, Vec<u8>)>, ClipboardHostError> {
    let mut data = [
        "text/plain",
        "TEXT",
        "STRING",
        "text/plain;charset=utf-8",
        "UTF8_STRING",
    ]
    .into_iter()
    .map(|mime| (mime.to_owned(), snapshot.plain_text.as_bytes().to_vec()))
    .collect::<Vec<_>>();
    if let Some(html) = &snapshot.html {
        data.push(("text/html".to_owned(), html.markup.as_bytes().to_vec()));
    }
    if snapshot.smart_replace {
        data.push(("chromium/x-webkit-paste".to_owned(), Vec::new()));
    }
    for (kind, value) in [
        ("text/rtf", &snapshot.rtf),
        ("image/svg+xml", &snapshot.svg),
        ("text/uri-list", &snapshot.uri_list),
    ] {
        if let Some(value) = value {
            data.push((kind.to_owned(), value.as_bytes().to_vec()));
        }
    }
    if let Some(png) = &snapshot.png {
        data.push(("image/png".into(), png.GetBlobDataHandle().to_vec()));
    }
    if !snapshot.files.is_empty() {
        let mut uris = Vec::new();
        for file in &snapshot.files {
            if !file.HasBackingFile() {
                return Err(ClipboardHostError::WriteFailed);
            }
            uris.push(
                url::Url::from_file_path(file.GetPath())
                    .map_err(|_| ClipboardHostError::WriteFailed)?
                    .to_string(),
            );
        }
        data.retain(|(kind, _)| kind != "text/uri-list");
        data.push(("text/uri-list".into(), uris.join("\r\n").into_bytes()));
    }
    if !snapshot.custom_data.is_empty() {
        data.push((
            "chromium/x-web-custom-data".into(),
            super::formats::encode_custom(&snapshot.custom_data)?,
        ));
    }
    Ok(data)
}
