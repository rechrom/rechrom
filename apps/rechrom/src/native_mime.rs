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
            #[cfg(not(target_os = "macos"))]
            return Err(fileapi::NativeFileError::Unavailable);
            #[cfg(target_os = "macos")]
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
