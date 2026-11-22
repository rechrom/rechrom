// cpp: layoutng_style/css/style_color.h:49-56
// Only declarations are supplied here. These opaque references preserve the
// source namespaces and pointer types; their implementations belong to
// external CSS/UI packages.
pub mod ui {
    #[repr(C)]
    pub struct ColorProvider {
        _private: [u8; 0],
    }
}

#[repr(C)]
pub struct CSSLengthResolver {
    _private: [u8; 0],
}
