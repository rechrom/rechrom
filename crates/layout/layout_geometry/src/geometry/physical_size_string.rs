// cpp: layoutng_geometry/geometry/physical_size_string.cc:3-6
// PhysicalSize belongs to //src/foundation:blink_geometry_api. Rust extension
// traits cannot add inherent methods to a type owned by another crate.
use foundation::{PhysicalSize, StrCat, String};

// cpp: layoutng_geometry/geometry/physical_size_string.cc:9-11
#[allow(non_snake_case)]
pub trait PhysicalSizeStringExt {
    fn ToString(&self) -> String;
}

#[allow(non_snake_case)]
impl PhysicalSizeStringExt for PhysicalSize {
    fn ToString(&self) -> String {
        StrCat(&[
            self.width.ToString().into(),
            String::from("x"),
            self.height.ToString().into(),
        ])
    }
}

// cpp: layoutng_geometry/geometry/physical_size_string.cc:13-15
/// Display adapter for the C++ `operator<<(ostream&, PhysicalSize)`.
pub struct PhysicalSizeDisplay<'a>(pub &'a PhysicalSize);

impl std::fmt::Display for PhysicalSizeDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.ToString())
    }
}
