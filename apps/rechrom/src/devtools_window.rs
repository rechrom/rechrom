pub fn show(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn browser_show_devtools(address: *const std::ffi::c_char);
        }
        let url = std::ffi::CString::new(url).map_err(std::io::Error::other)?;
        // App calls this from winit's main UI thread.
        unsafe {
            browser_show_devtools(url.as_ptr());
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Native DevTools window unavailable: {url}"),
        ))
    }
}
