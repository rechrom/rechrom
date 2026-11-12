//! Native copy/paste clipboard, following ui/base/clipboard/clipboard_win.cc
//! and clipboard_util_win.cc. PRIMARY selection is deliberately unrelated.
use super::{ClipboardHostError, PrimaryClipboardSnapshot};
use interaction::default::core::editing::commands::clipboard_commands::ClipboardHtml;
use std::{ffi::c_void, ptr, time::Duration};

type Handle = *mut c_void;
#[link(name = "user32")]
extern "system" {
    fn OpenClipboard(owner: Handle) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn IsClipboardFormatAvailable(format: u32) -> i32;
    fn GetClipboardData(format: u32) -> Handle;
    fn SetClipboardData(format: u32, data: Handle) -> Handle;
    fn RegisterClipboardFormatW(name: *const u16) -> u32;
    fn GetClipboardSequenceNumber() -> u32;
}
#[link(name = "shell32")]
extern "system" {
    fn DragQueryFileW(drop: Handle, index: u32, name: *mut u16, length: u32) -> u32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(flags: u32, size: usize) -> Handle;
    fn GlobalFree(data: Handle) -> Handle;
    fn GlobalLock(data: Handle) -> Handle;
    fn GlobalUnlock(data: Handle) -> i32;
    fn GlobalSize(data: Handle) -> usize;
}
const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;

struct Open;
impl Open {
    fn acquire(owner: usize, error: ClipboardHostError) -> Result<Self, ClipboardHostError> {
        // ClipboardWin::ScopedClipboard retries briefly when another process
        // holds the clipboard; never substitute cached process-local data.
        for _ in 0..5 {
            if unsafe { OpenClipboard(owner as Handle) } != 0 {
                return Ok(Self);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Err(error)
    }
}
impl Drop for Open {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}
fn format(name: &str) -> Result<u32, ClipboardHostError> {
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    match unsafe { RegisterClipboardFormatW(name.as_ptr()) } {
        0 => Err(ClipboardHostError::Unavailable),
        value => Ok(value),
    }
}
fn available(format: u32) -> bool {
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}
fn read_bytes(format: u32) -> Result<Vec<u8>, ClipboardHostError> {
    unsafe {
        let data = GetClipboardData(format);
        if data.is_null() {
            return Err(ClipboardHostError::ReadFailed);
        }
        let size = GlobalSize(data);
        let bytes = GlobalLock(data);
        if bytes.is_null() {
            return Err(ClipboardHostError::ReadFailed);
        }
        let result = std::slice::from_raw_parts(bytes.cast::<u8>(), size).to_vec();
        GlobalUnlock(data);
        Ok(result)
    }
}
pub(super) fn read(owner: usize) -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
    let html_type = format("HTML Format")?;
    let smart_type = format("WebKit Smart Paste Format")?;
    let png_type = format("PNG")?;
    let svg_type = format("image/svg+xml")?;
    let rtf_type = format("Rich Text Format")?;
    let custom_type = format("Chromium Web Custom MIME Data Format")?;
    let file_descriptor_type = format("FileGroupDescriptorW")?;
    let url_type = format("UniformResourceLocatorW")?;
    let filename_type = format("FileNameW")?;
    let open = Open::acquire(owner, ClipboardHostError::ReadFailed)?;
    let sequence = unsafe { GetClipboardSequenceNumber() };
    if ![
        CF_UNICODETEXT,
        html_type,
        png_type,
        svg_type,
        rtf_type,
        CF_HDROP,
        custom_type,
        file_descriptor_type,
        url_type,
        filename_type,
    ]
    .into_iter()
    .any(available)
    {
        return Err(ClipboardHostError::Unavailable);
    }
    let plain_text = if available(CF_UNICODETEXT) {
        let bytes = read_bytes(CF_UNICODETEXT)?;
        let text: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        String::from_utf16_lossy(&text)
    } else {
        String::new()
    };
    let html = if available(html_type) {
        Some(decode_html(&read_bytes(html_type)?)?)
    } else {
        None
    };
    let string = |kind| -> Result<Option<String>, ClipboardHostError> {
        if !available(kind) {
            return Ok(None);
        }
        let bytes = read_bytes(kind)?;
        Ok(Some(
            String::from_utf8_lossy(bytes.split(|byte| *byte == 0).next().unwrap_or_default())
                .into_owned(),
        ))
    };
    let wide = |kind| -> Result<Option<String>, ClipboardHostError> {
        if !available(kind) {
            return Ok(None);
        }
        let bytes = read_bytes(kind)?;
        let units: Vec<_> = bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        Ok(Some(String::from_utf16_lossy(&units)))
    };
    let mut snapshot = PrimaryClipboardSnapshot {
        plain_text,
        html,
        smart_replace: available(smart_type),
        png: if available(png_type) {
            Some(super::formats::png(read_bytes(png_type)?))
        } else {
            None
        },
        svg: string(svg_type)?,
        rtf: string(rtf_type)?,
        uri_list: wide(url_type)?,
        custom_data: if available(custom_type) {
            super::formats::decode_custom(&read_bytes(custom_type)?)
        } else {
            Vec::new()
        },
        ..Default::default()
    };
    let mut paths = Vec::new();
    if available(CF_HDROP) {
        let drop = unsafe { GetClipboardData(CF_HDROP) };
        if drop.is_null() {
            return Err(ClipboardHostError::ReadFailed);
        }
        for index in 0..unsafe { DragQueryFileW(drop, u32::MAX, ptr::null_mut(), 0) } {
            let length = unsafe { DragQueryFileW(drop, index, ptr::null_mut(), 0) };
            let mut name = vec![0u16; length as usize + 1];
            if unsafe { DragQueryFileW(drop, index, name.as_mut_ptr(), length + 1) } != length {
                return Err(ClipboardHostError::ReadFailed);
            }
            use std::os::windows::ffi::OsStringExt;
            paths.push(std::path::PathBuf::from(std::ffi::OsString::from_wide(
                &name[..length as usize],
            )));
        }
    } else if let Some(path) = wide(filename_type)? {
        paths.push(path.into());
    }
    let virtual_files = paths.is_empty() && available(file_descriptor_type);
    // OLE cannot acquire IDataObject while OpenClipboard is held. Its retained
    // streams are marshaled by the existing native drag importer, then checked
    // against the same clipboard sequence before publishing this snapshot.
    drop(open);
    super::formats::admit_paths(&mut snapshot, paths)?;
    if virtual_files {
        let offers = winit::platform::windows::clipboard_virtual_files()
            .map_err(|_| ClipboardHostError::ReadFailed)?;
        for offer in offers {
            snapshot.files.push(
                crate::native_file::from_virtual_offer(&offer)
                    .map_err(|_| ClipboardHostError::ReadFailed)?,
            );
        }
    }
    if unsafe { GetClipboardSequenceNumber() } != sequence {
        return Err(ClipboardHostError::ReadFailed);
    }
    Ok(snapshot)
}

// Allocate all flavors before EmptyClipboard so allocation/encoding failures
// cannot destroy the user's existing clipboard. SetClipboardData transfers
// each successful HGLOBAL to Windows; untransferred allocations are freed.
struct Global(Handle);
impl Global {
    fn new(bytes: &[u8]) -> Result<Self, ClipboardHostError> {
        unsafe {
            let value = Self(GlobalAlloc(0x0042, bytes.len().max(1))); // MOVEABLE | ZEROINIT
            if value.0.is_null() {
                return Err(ClipboardHostError::WriteFailed);
            }
            let target = GlobalLock(value.0);
            if target.is_null() {
                return Err(ClipboardHostError::WriteFailed);
            }
            ptr::copy_nonoverlapping(bytes.as_ptr(), target.cast(), bytes.len());
            GlobalUnlock(value.0);
            Ok(value)
        }
    }
    fn publish(mut self, format: u32) -> Result<(), ClipboardHostError> {
        if unsafe { SetClipboardData(format, self.0) }.is_null() {
            return Err(ClipboardHostError::WriteFailed);
        }
        self.0 = ptr::null_mut();
        Ok(())
    }
}
impl Drop for Global {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { GlobalFree(self.0) };
        }
    }
}
pub(super) fn write(
    owner: usize,
    snapshot: &PrimaryClipboardSnapshot,
) -> Result<(), ClipboardHostError> {
    let text: Vec<u8> = snapshot
        .plain_text
        .encode_utf16()
        .chain(Some(0))
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut values = vec![(CF_UNICODETEXT, Global::new(&text)?)];
    if let Some(html) = &snapshot.html {
        let mut bytes = encode_html(html)?.into_bytes();
        bytes.push(0);
        values.push((format("HTML Format")?, Global::new(&bytes)?));
    }
    if snapshot.smart_replace {
        values.push((format("WebKit Smart Paste Format")?, Global::new(&[])?));
    }
    for (kind, value) in [
        ("Rich Text Format", &snapshot.rtf),
        ("image/svg+xml", &snapshot.svg),
    ] {
        if let Some(value) = value {
            let mut bytes = value.as_bytes().to_vec();
            bytes.push(0);
            values.push((format(kind)?, Global::new(&bytes)?));
        }
    }
    if let Some(png) = &snapshot.png {
        values.push((format("PNG")?, Global::new(&png.GetBlobDataHandle())?));
    }
    if !snapshot.custom_data.is_empty() {
        values.push((
            format("Chromium Web Custom MIME Data Format")?,
            Global::new(&super::formats::encode_custom(&snapshot.custom_data)?)?,
        ));
    }
    if let Some(uris) = &snapshot.uri_list {
        if let Some(url) = uris
            .lines()
            .find(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let bytes: Vec<_> = url
                .encode_utf16()
                .chain(Some(0))
                .flat_map(u16::to_le_bytes)
                .collect();
            values.push((format("UniformResourceLocatorW")?, Global::new(&bytes)?));
        }
    }
    if !snapshot.files.is_empty() {
        // DROPFILES: byte offset 20, POINT(0,0), fNC=false, fWide=true.
        let mut bytes = [20u32, 0, 0, 0, 1]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();
        for file in &snapshot.files {
            if !file.HasBackingFile() || file.GetPath().contains('\0') {
                return Err(ClipboardHostError::WriteFailed);
            }
            bytes.extend(
                file.GetPath()
                    .encode_utf16()
                    .chain(Some(0))
                    .flat_map(u16::to_le_bytes),
            );
        }
        bytes.extend_from_slice(&[0, 0]);
        values.push((CF_HDROP, Global::new(&bytes)?));
    }
    let _open = Open::acquire(owner, ClipboardHostError::WriteFailed)?;
    if unsafe { EmptyClipboard() } == 0 {
        return Err(ClipboardHostError::WriteFailed);
    }
    for (format, data) in values {
        data.publish(format)?;
    }
    Ok(())
}

fn decode_html(bytes: &[u8]) -> Result<ClipboardHtml, ClipboardHostError> {
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    let header = String::from_utf8_lossy(bytes);
    let number = |field: &str| -> Option<usize> {
        header
            .lines()
            .find_map(|line| line.strip_prefix(field)?.trim().parse().ok())
    };
    let html_start = bytes
        .windows(5)
        .position(|part| part.eq_ignore_ascii_case(b"<html"));
    // CFHtmlExtractMetadata prefers fragment comments, then numeric headers.
    let marker = html_start.and_then(|html_start| {
        bytes[html_start..]
            .windows(b"<!--StartFragment".len())
            .position(|part| part == b"<!--StartFragment")
            .map(|offset| offset + html_start)
    });
    let (start, end) = if let Some(marker) = marker {
        (
            bytes[marker..]
                .iter()
                .position(|byte| *byte == b'>')
                .map(|offset| marker + offset + 1),
            bytes
                .windows(b"<!--EndFragment".len())
                .rposition(|part| part == b"<!--EndFragment"),
        )
    } else {
        (number("StartFragment:"), number("EndFragment:"))
    };
    let (Some(html_start), Some(start), Some(end)) = (html_start, start, end) else {
        return Err(ClipboardHostError::ReadFailed);
    };
    if html_start > start || start > end || end > bytes.len() {
        return Err(ClipboardHostError::ReadFailed);
    }
    let fragment_start = String::from_utf8_lossy(&bytes[html_start..start])
        .encode_utf16()
        .count();
    let fragment_end = String::from_utf8_lossy(&bytes[html_start..end])
        .encode_utf16()
        .count();
    Ok(ClipboardHtml {
        markup: String::from_utf8_lossy(&bytes[html_start..]).into_owned(),
        source_url: header
            .lines()
            .find_map(|line| line.strip_prefix("SourceURL:"))
            .unwrap_or("")
            .trim_matches(|character: char| character.is_ascii_whitespace())
            .to_owned(),
        fragment_start: u32::try_from(fragment_start)
            .map_err(|_| ClipboardHostError::ReadFailed)?,
        fragment_end: u32::try_from(fragment_end).map_err(|_| ClipboardHostError::ReadFailed)?,
    })
}
fn encode_html(html: &ClipboardHtml) -> Result<String, ClipboardHostError> {
    // ClipboardWin::WriteHTML treats markup as the fragment; fragment indices
    // belong to the read-side representation, not the writer's CF_HTML header.
    let source = if html.source_url.is_empty() {
        String::new()
    } else {
        if html.source_url.contains(['\r', '\n', '\0']) {
            return Err(ClipboardHostError::WriteFailed);
        }
        format!("SourceURL:{}\r\n", html.source_url)
    };
    let prefix = "<html>\r\n<body>\r\n<!--StartFragment-->";
    let suffix = "<!--EndFragment-->\r\n</body>\r\n</html>";
    let header = |start, end, fragment_start, fragment_end| {
        format!(
        "Version:0.9\r\nStartHTML:{start:010}\r\nEndHTML:{end:010}\r\nStartFragment:{fragment_start:010}\r\nEndFragment:{fragment_end:010}\r\n{source}")
    };
    let start = header(0, 0, 0, 0).len();
    let fragment_start = start + prefix.len();
    let fragment_end = fragment_start + html.markup.len();
    let end = fragment_end + suffix.len();
    if end > 9_999_999_999 {
        return Err(ClipboardHostError::WriteFailed);
    }
    Ok(format!(
        "{}{prefix}{}{suffix}",
        header(start, end, fragment_start, fragment_end),
        html.markup
    ))
}
