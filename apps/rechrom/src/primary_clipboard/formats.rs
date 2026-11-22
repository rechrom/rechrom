//! ui/base/clipboard/custom_data_helper.cc and base/pickle.cc wire format.
//! Only the Chromium DataTransfer custom-data container is exposed to script;
//! arbitrary platform-private clipboard flavors are not web MIME items.
use super::{ClipboardHostError, PrimaryClipboardSnapshot};
use std::sync::Arc;

pub(super) fn png(bytes: Vec<u8>) -> Arc<fileapi::File> {
    fileapi::File::Create(
        "image.png".into(),
        "image/png".into(),
        fileapi::File::NowMilliseconds(),
        bytes.into(),
    )
}
pub(super) fn admit_paths(
    snapshot: &mut PrimaryClipboardSnapshot,
    paths: impl IntoIterator<Item = std::path::PathBuf>,
) -> Result<(), ClipboardHostError> {
    for path in paths {
        let offer = winit::event::NativeDragFileOffer::open(path)
            .map_err(|_| ClipboardHostError::ReadFailed)?;
        snapshot.files.push(
            crate::native_file::from_offer(&offer).map_err(|_| ClipboardHostError::ReadFailed)?,
        );
    }
    Ok(())
}
pub(super) fn admit_uris(
    snapshot: &mut PrimaryClipboardSnapshot,
) -> Result<(), ClipboardHostError> {
    let paths = snapshot
        .uri_list
        .as_deref()
        .unwrap_or("")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| url::Url::parse(line).ok()?.to_file_path().ok())
        .collect::<Vec<_>>();
    admit_paths(snapshot, paths)
}
pub(super) fn decode_custom(bytes: &[u8]) -> Vec<(String, Vec<u16>)> {
    fn decode(bytes: &[u8]) -> Option<Vec<(String, Vec<u16>)>> {
        let size = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
        let bytes = bytes.get(4..4usize.checked_add(size)?)?;
        let mut position = 0usize;
        fn word(bytes: &[u8], position: &mut usize) -> Option<u32> {
            let end = position.checked_add(4)?;
            let value = u32::from_le_bytes(bytes.get(*position..end)?.try_into().ok()?);
            *position = end;
            Some(value)
        }
        fn string(bytes: &[u8], position: &mut usize) -> Option<Vec<u16>> {
            let length = word(bytes, position)? as usize;
            if length > i32::MAX as usize {
                return None;
            }
            let length = length.checked_mul(2)?;
            let end = position.checked_add(length)?;
            let value = bytes
                .get(*position..end)?
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect();
            *position = end.checked_add(3)? & !3;
            if *position > bytes.len() {
                return None;
            }
            Some(value)
        }
        let count = word(bytes, &mut position)? as usize;
        if count > bytes.len().saturating_sub(position) / 8 {
            return None;
        }
        let mut values = Vec::new();
        for _ in 0..count {
            let kind = String::from_utf16(&string(bytes, &mut position)?).ok()?;
            let value = string(bytes, &mut position)?;
            if !values.iter().any(|(key, _)| key == &kind) {
                values.push((kind, value));
            }
        }
        Some(values)
    }
    // Corrupt custom maps are discarded as a whole, like ReadCustomDataTypes.
    decode(bytes).unwrap_or_default()
}
pub(super) fn encode_custom(values: &[(String, Vec<u16>)]) -> Result<Vec<u8>, ClipboardHostError> {
    let error = || ClipboardHostError::WriteFailed;
    let mut bytes = vec![0; 4];
    bytes.extend_from_slice(
        &u32::try_from(values.len())
            .map_err(|_| error())?
            .to_le_bytes(),
    );
    for (kind, value) in values {
        for units in [kind.encode_utf16().collect::<Vec<_>>(), value.clone()] {
            bytes.extend_from_slice(
                &i32::try_from(units.len())
                    .map_err(|_| error())?
                    .to_le_bytes(),
            );
            for unit in units {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            bytes.resize((bytes.len() + 3) & !3, 0);
        }
    }
    let size = u32::try_from(bytes.len() - 4).map_err(|_| error())?;
    bytes[..4].copy_from_slice(&size.to_le_bytes());
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chromium_custom_pickle_preserves_utf16_and_rejects_truncation() {
        let values = vec![("application/x-test".into(), vec![0, 0xd800, 0x61])];
        let bytes = encode_custom(&values).unwrap();
        assert_eq!(decode_custom(&bytes), values);
        assert_eq!(
            u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize,
            bytes.len() - 4
        );
        for length in 0..bytes.len() {
            assert!(decode_custom(&bytes[..length]).is_empty());
        }
    }
}
