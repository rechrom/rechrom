use crate::Length;

// cpp: layoutng_style/style/transform_origin.h:35-42
// Rust has no std::ostream object. This is the required output capability,
// not an implementation: the host still must provide Length's stream format,
// whose C++ declaration has no definition in the supplied source tree.
#[allow(non_snake_case)]
pub trait CppOstream {
    fn WriteStr(&mut self, value: &str);
    fn WriteLength(&mut self, value: &Length);
    fn WriteFloat(&mut self, value: f32);
}
