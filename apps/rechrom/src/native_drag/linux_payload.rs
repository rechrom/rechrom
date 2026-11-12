//! Wayland/X selection MIME representations. File promises carry actual bytes,
//! using Chromium's application/octet-stream;name="..." convention.
use super::{payload, Failure, SystemDragRequest};
use std::sync::Arc;
pub(super) type Offers = Vec<(String, Arc<[u8]>)>;
pub(super) fn build(
    request: &SystemDragRequest,
) -> Result<(Offers, Option<(String, Arc<[u8]>)>), Failure> {
    let payload = payload::build(request)?;
    let mut offers: Offers = Vec::new();
    for (mime, text) in payload.strings {
        let utf8 = String::from_utf16_lossy(&text).into_bytes();
        offers.push((mime.clone(), Arc::from(utf8)));
        // Preserve every code unit, including unpaired surrogates, for native
        // peers selecting the explicitly advertised UTF-16 representation.
        if !mime.contains("charset=") {
            offers.push((
                format!("{mime};charset=utf-16le"),
                Arc::from(
                    text.iter()
                        .flat_map(|unit| unit.to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ));
        }
    }
    let mut uris = String::new();
    let mut direct_save = None;
    for file in payload.files {
        if let Some(path) = file.path {
            let url = url::Url::from_file_path(path).map_err(|_| Failure::InvalidPayload)?;
            uris.push_str(url.as_str());
            uris.push_str("\r\n");
        } else {
            let name = String::from_utf16_lossy(&file.name);
            let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
            let mime = format!("application/octet-stream;name=\"{escaped}\"");
            if offers.iter().any(|(existing, _)| *existing == mime) {
                return Err(Failure::Unsupported);
            }
            offers.push((mime, file.bytes.clone()));
            if direct_save.is_none() {
                direct_save = Some((name.into_owned(), file.bytes));
            }
        }
    }
    if !uris.is_empty() {
        if let Some((_, bytes)) = offers.iter_mut().find(|(mime, _)| mime == "text/uri-list") {
            let mut combined = bytes.to_vec();
            if !combined.ends_with(b"\n") {
                combined.extend_from_slice(b"\r\n");
            }
            combined.extend_from_slice(uris.as_bytes());
            *bytes = combined.into();
        } else {
            offers.push(("text/uri-list".into(), uris.into_bytes().into()));
        }
    }
    Ok((offers, direct_save))
}
