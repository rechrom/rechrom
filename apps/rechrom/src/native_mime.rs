//! One native MIME extension query shared by drag promises and lazy Files.
//! The source helper owns PRIMARY -> actual OS -> SECONDARY precedence.
pub(crate) fn from_file(filename: &str) -> Result<Option<String>, fileapi::NativeFileError> {
    match net_filename::GetMimeTypeFromFile(filename, None) {
        Ok(value) => Ok(value),
        Err(net_filename::MimeQueryRequired) => {
            let Some(extension) = net_filename::FileNameExtension(filename) else {
                return Ok(None);
            };
            #[cfg(target_os = "macos")]
            let native = {
                #[repr(C)]
                struct NativeString {
                    bytes: *mut u8,
                    len: usize,
                }
                unsafe extern "C" {
                    fn browser_drag_mime_from_extension(
                        extension: *const u8,
                        len: usize,
                    ) -> NativeString;
                }
                let value = unsafe {
                    browser_drag_mime_from_extension(extension.as_ptr(), extension.len())
                };
                if value.bytes.is_null() {
                    None
                } else {
                    let text = unsafe { std::slice::from_raw_parts(value.bytes, value.len) };
                    let owned = String::from_utf8(text.to_vec()).expect("native MIME UTF8");
                    unsafe { libc::free(value.bytes.cast()) };
                    Some(owned)
                }
            };
            #[cfg(windows)]
            let native = windows::GetPlatformMimeTypeFromExtension(extension);
            #[cfg(not(any(target_os = "macos", windows)))]
            return Err(fileapi::NativeFileError::Unavailable);
            #[cfg(any(target_os = "macos", windows))]
            Ok(net_filename::GetMimeTypeFromFile(
                filename,
                Some(net_filename::PlatformExtensionMime {
                    extension,
                    mime_type: native.as_deref(),
                }),
            )
            .expect("same extension/native MIME facts"))
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;

    const HKEY_CLASSES_ROOT: isize = 0x8000_0000_u32 as i32 as isize;
    const RRF_RT_REG_SZ: u32 = 0x0000_0002;
    const RRF_RT_REG_EXPAND_SZ: u32 = 0x0000_0004;
    const ERROR_SUCCESS: i32 = 0;
    const ERROR_MORE_DATA: i32 = 234;

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            key: isize,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            value_type: *mut u32,
            data: *mut c_void,
            data_size: *mut u32,
        ) -> i32;
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    // Source: net/base/platform_mime_util_win.cc:16-29. Check the registry
    // for the extension's MIME type (registry key names are not
    // case-sensitive); base::win::RegKey::ReadValue accepts REG_SZ and
    // REG_EXPAND_SZ.
    pub(super) fn GetPlatformMimeTypeFromExtension(ext: &str) -> Option<String> {
        let key = wide(&format!(".{ext}"));
        let name = wide("Content Type");
        let flags = RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ;
        let mut size = 0u32;
        let mut buffer: Vec<u16> = Vec::new();
        loop {
            let result = unsafe {
                RegGetValueW(
                    HKEY_CLASSES_ROOT,
                    key.as_ptr(),
                    name.as_ptr(),
                    flags,
                    std::ptr::null_mut(),
                    if buffer.is_empty() {
                        std::ptr::null_mut()
                    } else {
                        buffer.as_mut_ptr().cast()
                    },
                    &mut size,
                )
            };
            match result {
                ERROR_SUCCESS if !buffer.is_empty() => break,
                ERROR_SUCCESS | ERROR_MORE_DATA => {
                    buffer = vec![0; (size as usize).div_ceil(2).max(1)];
                }
                _ => return None,
            }
        }
        let length = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
        let value = String::from_utf16_lossy(&buffer[..length]);
        (!value.is_empty()).then_some(value)
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn registry_maps_common_extensions() {
            assert_eq!(
                super::GetPlatformMimeTypeFromExtension("txt").as_deref(),
                Some("text/plain")
            );
            assert_eq!(super::GetPlatformMimeTypeFromExtension("no-such-ext-rechrom"), None);
        }
    }
}
