// Root module wiring for delivered external source translations.
// Unresolved true native dependencies remain visible; zero Skia row credit.
pub mod time;
pub mod time_apple;
pub mod time_conversion_posix;
pub mod time_exploded_posix;
pub mod time_now_posix;
pub mod time_override;

#[cfg(not(base_native_time_inputs_configured))]
compile_error!("Supply actual resolved native unistd feature inputs; no host clock defaults");
