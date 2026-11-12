//! ClipboardHost's platform-owned standard clipboard and PRIMARY selection. A read stays
//! inside the Page's editing transaction; no copied selection is kept by App.
use interaction::default::core::editing::commands::clipboard_commands::{
    ClipboardHost, ClipboardHostError, PrimaryClipboardSnapshot,
};
use std::sync::Arc;

pub(crate) struct PrimaryClipboard {
    #[cfg(target_os = "linux")]
    backend: Backend,
    #[cfg(target_os = "windows")]
    owner: Option<usize>,
    #[cfg(target_os = "windows")]
    _window: Option<Arc<winit::window::Window>>,
}
#[cfg(target_os = "linux")]
enum Backend {
    Unavailable,
    X11(x11::Clipboard),
    Wayland {
        // Drop the foreign-display protocol queue before its display owner.
        clipboard: std::sync::Mutex<smithay_clipboard::Clipboard>,
        _window: Arc<winit::window::Window>,
    },
}
impl PrimaryClipboard {
    pub(crate) fn unavailable() -> Arc<Self> {
        Arc::new(Self {
            #[cfg(target_os = "linux")]
            backend: Backend::Unavailable,
            #[cfg(target_os = "windows")]
            owner: None,
            #[cfg(target_os = "windows")]
            _window: None,
        })
    }
    pub(crate) fn for_window(window: Arc<winit::window::Window>) -> Arc<Self> {
        #[cfg(target_os = "linux")]
        {
            use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
            let backend = match window.display_handle().map(|handle| handle.as_raw()) {
                Ok(RawDisplayHandle::Wayland(display)) => Backend::Wayland {
                    // SAFETY: the retained winit window keeps this exact display
                    // alive until after Clipboard and its protocol queue drop.
                    clipboard: std::sync::Mutex::new(unsafe {
                        smithay_clipboard::Clipboard::new(display.display.as_ptr())
                    }),
                    _window: window,
                },
                Ok(RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_)) => {
                    Backend::X11(x11::Clipboard::default())
                }
                _ => Backend::Unavailable,
            };
            Arc::new(Self { backend })
        }
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let owner = match window.window_handle().map(|handle| handle.as_raw()) {
                Ok(RawWindowHandle::Win32(window)) => Some(window.hwnd.get() as usize),
                _ => None,
            };
            Arc::new(Self {
                owner,
                _window: Some(window),
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = window;
            // Chromium's macOS editing behavior has no global
            // selection; the ordinary OS copy/paste clipboard is not PRIMARY.
            Self::unavailable()
        }
    }
}
impl ClipboardHost for PrimaryClipboard {
    fn ReadClipboard(&self) -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
        #[cfg(target_os = "macos")]
        {
            return standard::read();
        }
        #[cfg(target_os = "windows")]
        {
            return windows::read(self.owner.ok_or(ClipboardHostError::Unavailable)?);
        }
        #[cfg(target_os = "linux")]
        {
            return match &self.backend {
                Backend::Unavailable => Err(ClipboardHostError::Unavailable),
                Backend::X11(clipboard) => clipboard.read(),
                Backend::Wayland { clipboard, .. } => {
                    let data = clipboard
                        .lock()
                        .map_err(|_| ClipboardHostError::ReadFailed)?
                        .load_formats(linux::READ_FORMATS)
                        .map_err(|_| ClipboardHostError::ReadFailed)?;
                    linux::decode(data)
                }
            };
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        Err(ClipboardHostError::Unavailable)
    }
    fn WriteClipboard(
        &self,
        snapshot: &PrimaryClipboardSnapshot,
    ) -> Result<(), ClipboardHostError> {
        #[cfg(target_os = "macos")]
        {
            return standard::write(snapshot);
        }
        #[cfg(target_os = "windows")]
        {
            return windows::write(self.owner.ok_or(ClipboardHostError::Unavailable)?, snapshot);
        }
        #[cfg(target_os = "linux")]
        {
            return match &self.backend {
                Backend::Unavailable => Err(ClipboardHostError::Unavailable),
                Backend::X11(clipboard) => clipboard.write(snapshot),
                Backend::Wayland { clipboard, .. } => clipboard
                    .lock()
                    .map_err(|_| ClipboardHostError::WriteFailed)?
                    .store_formats(linux::encode(snapshot)?)
                    .map_err(|_| ClipboardHostError::WriteFailed),
            };
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            let _ = snapshot;
            Err(ClipboardHostError::Unavailable)
        }
    }
    fn ReadPrimarySelection(&self) -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
        #[cfg(target_os = "linux")]
        {
            let text = match &self.backend {
                Backend::Unavailable => return Err(ClipboardHostError::Unavailable),
                Backend::X11(_) => x11::read_primary()?,
                Backend::Wayland { clipboard, .. } => clipboard
                    .lock()
                    .map_err(|_| ClipboardHostError::ReadFailed)?
                    .load_primary()
                    .map_err(|_| ClipboardHostError::ReadFailed)?,
            };
            Ok(PrimaryClipboardSnapshot {
                plain_text: text,
                html: None,
                smart_replace: false,
                ..Default::default()
            })
        }
        #[cfg(not(target_os = "linux"))]
        Err(ClipboardHostError::Unavailable)
    }
}

#[cfg(target_os = "macos")]
mod standard {
    use super::*;
    use interaction::default::core::editing::commands::clipboard_commands::ClipboardHtml;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Bytes {
        bytes: *const u8,
        length: usize,
    }
    impl Bytes {
        fn from_bytes(value: &[u8]) -> Self {
            Self {
                bytes: value.as_ptr(),
                length: value.len(),
            }
        }
        fn from_str(value: &str) -> Self {
            Self {
                bytes: value.as_ptr(),
                length: value.len(),
            }
        }
        unsafe fn take(self) -> String {
            String::from_utf8_lossy(&unsafe { self.take_bytes() }).into_owned()
        }
        unsafe fn take_bytes(self) -> Vec<u8> {
            if self.bytes.is_null() {
                return Vec::new();
            }
            let value = unsafe { std::slice::from_raw_parts(self.bytes, self.length) }.to_vec();
            unsafe {
                libc::free(self.bytes.cast_mut().cast());
            }
            value
        }
    }
    #[repr(C)]
    struct Value {
        text: Bytes,
        html: Bytes,
        source_url: Bytes,
        has_html: bool,
        smart: bool,
        png: Bytes,
        svg: Bytes,
        rtf: Bytes,
        uris: Bytes,
        custom: Bytes,
        has_png: bool,
        has_svg: bool,
        has_rtf: bool,
        has_uris: bool,
    }
    extern "C" {
        fn rechrom_clipboard_read(value: *mut Value) -> bool;
        fn rechrom_clipboard_write(value: *const Value) -> bool;
    }
    pub(super) fn read() -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
        let empty = Bytes {
            bytes: std::ptr::null(),
            length: 0,
        };
        let mut value = Value {
            text: empty,
            html: empty,
            source_url: empty,
            has_html: false,
            smart: false,
            png: empty,
            svg: empty,
            rtf: empty,
            uris: empty,
            custom: empty,
            has_png: false,
            has_svg: false,
            has_rtf: false,
            has_uris: false,
        };
        // Each returned buffer is malloc-owned, even for an empty flavor. Free
        // all buffers on failure as well as success; no AppKit owner escapes.
        let ok = unsafe { rechrom_clipboard_read(&mut value) };
        let (plain_text, markup, source_url) = unsafe {
            (
                value.text.take(),
                value.html.take(),
                value.source_url.take(),
            )
        };
        let (png, svg, rtf, uris, custom) = unsafe {
            (
                value.png.take_bytes(),
                value.svg.take(),
                value.rtf.take(),
                value.uris.take(),
                value.custom.take_bytes(),
            )
        };
        if !ok {
            return Err(ClipboardHostError::ReadFailed);
        }
        let mut snapshot = PrimaryClipboardSnapshot {
            plain_text,
            smart_replace: value.smart,
            html: value.has_html.then(|| ClipboardHtml {
                fragment_start: 0,
                fragment_end: markup.encode_utf16().count() as u32,
                markup,
                source_url,
            }),
            png: value.has_png.then(|| super::formats::png(png)),
            svg: value.has_svg.then_some(svg),
            rtf: value.has_rtf.then_some(rtf),
            uri_list: value.has_uris.then_some(uris),
            custom_data: super::formats::decode_custom(&custom),
            ..Default::default()
        };
        super::formats::admit_uris(&mut snapshot)?;
        Ok(snapshot)
    }
    pub(super) fn write(snapshot: &PrimaryClipboardSnapshot) -> Result<(), ClipboardHostError> {
        let custom = super::formats::encode_custom(&snapshot.custom_data)?;
        let png = snapshot.png.as_ref().map(|file| file.GetBlobDataHandle());
        let uris = if snapshot.files.is_empty() {
            snapshot.uri_list.clone()
        } else {
            Some(
                snapshot
                    .files
                    .iter()
                    .map(|file| {
                        if !file.HasBackingFile() {
                            return Err(ClipboardHostError::WriteFailed);
                        }
                        url::Url::from_file_path(file.GetPath())
                            .map(|url| url.to_string())
                            .map_err(|_| ClipboardHostError::WriteFailed)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join("\r\n"),
            )
        };
        let value = Value {
            text: Bytes::from_str(&snapshot.plain_text),
            html: Bytes::from_str(
                snapshot
                    .html
                    .as_ref()
                    .map_or("", |html| html.markup.as_str()),
            ),
            source_url: Bytes::from_str(
                snapshot
                    .html
                    .as_ref()
                    .map_or("", |html| html.source_url.as_str()),
            ),
            has_html: snapshot.html.is_some(),
            smart: snapshot.smart_replace,
            png: Bytes::from_bytes(png.as_deref().unwrap_or_default()),
            svg: Bytes::from_str(snapshot.svg.as_deref().unwrap_or_default()),
            rtf: Bytes::from_str(snapshot.rtf.as_deref().unwrap_or_default()),
            uris: Bytes::from_str(uris.as_deref().unwrap_or_default()),
            custom: Bytes::from_bytes(if snapshot.custom_data.is_empty() {
                &[]
            } else {
                &custom
            }),
            has_png: snapshot.png.is_some(),
            has_svg: snapshot.svg.is_some(),
            has_rtf: snapshot.rtf.is_some(),
            has_uris: uris.is_some(),
        };
        if unsafe { rechrom_clipboard_write(&value) } {
            Ok(())
        } else {
            Err(ClipboardHostError::WriteFailed)
        }
    }
}

#[cfg(target_os = "linux")]
#[path = "primary_clipboard_x11.rs"]
mod x11;

#[path = "primary_clipboard_formats.rs"]
mod formats;
#[cfg(target_os = "linux")]
#[path = "primary_clipboard_linux.rs"]
mod linux;
#[cfg(target_os = "windows")]
#[path = "primary_clipboard_windows.rs"]
mod windows;
