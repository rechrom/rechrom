//! Native host serialization, independent of the document's live DataTransfer.
use super::{Failure, SystemDragRequest};
use std::sync::Arc;

pub(super) struct File {
    pub name: Vec<u16>,
    pub mime: String,
    pub path: Option<String>,
    pub bytes: Arc<[u8]>,
}
pub(super) struct Payload {
    pub strings: Vec<(String, Vec<u16>)>,
    pub files: Vec<File>,
}
pub(super) fn build(request: &SystemDragRequest) -> Result<Payload, Failure> {
    if !request.data.file_system_files.is_empty() {
        // These are sandboxed filesystem URLs, not native paths or byte promises.
        return Err(Failure::Unsupported);
    }
    let mut strings: Vec<_> = request
        .data
        .strings
        .iter()
        .map(|(mime, text)| (mime.clone(), text.encode_utf16().collect()))
        .collect();
    strings.extend(
        request
            .data
            .utf16_strings
            .iter()
            .map(|(mime, text)| (mime.clone(), text.to_vec())),
    );
    for (mime, text) in [
        (
            "org.chromium.source-effect-allowed",
            &request.data.source_effect_allowed,
        ),
        ("org.chromium.html-base-url", &request.data.html_base_url),
        ("public.url-name", &request.data.url_title),
    ] {
        if let Some(text) = text {
            strings.push((mime.into(), text.encode_utf16().collect()));
        }
    }
    if strings
        .iter()
        .any(|(mime, _)| mime.is_empty() || mime.contains('\0'))
    {
        return Err(Failure::InvalidPayload);
    }
    let mut files = Vec::new();
    for file in &request.data.files {
        if !file.is_image_accessible {
            return Err(Failure::Unsupported);
        }
        let safe = net_filename::GetSafeFilenameForImageFileContents(
            &file.source_url,
            &file.content_disposition,
            &file.filename_extension,
        )
        .ok_or(Failure::Unsupported)?;
        files.push(File {
            name: safe.filename.encode_utf16().collect(),
            mime: safe.well_known_mime_type.into(),
            path: None,
            bytes: file.bytes.clone(),
        });
    }
    for file in &request.data.file_items {
        let path = file.HasBackingFile().then(|| file.GetPath().to_owned());
        let bytes = if path.is_some() {
            Arc::from([])
        } else {
            file.CachedBytes().map_err(|_| Failure::Unsupported)?
        };
        files.push(File {
            name: file.name().encode_utf16().collect(),
            mime: file.type_().into(),
            path,
            bytes,
        });
    }
    if files.iter().any(|file| {
        file.name.contains(&0)
            || file.mime.contains('\0')
            || file.path.as_ref().is_some_and(|path| path.contains('\0'))
    }) {
        return Err(Failure::InvalidPayload);
    }
    if let Some(image) = &request.image {
        if image.width == 0
            || image.height == 0
            || image.row_bytes < image.width as usize * 4
            || image
                .row_bytes
                .checked_mul(image.height as usize)
                .is_none_or(|size| size > image.rgba8.len())
        {
            return Err(Failure::InvalidPayload);
        }
    }
    Ok(Payload { strings, files })
}
