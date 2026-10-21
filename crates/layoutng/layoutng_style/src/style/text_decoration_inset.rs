// cpp: layoutng_style/style/text_decoration_inset.h:8-9
// Pending connection to //src/foundation:blink_geometry_api.
use foundation::Length;

// cpp: layoutng_style/style/text_decoration_inset.h:13-18
/// Start and end insets for CSS text decoration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextDecorationInset {
    start_: Length,
    end_: Length,
}

#[allow(non_snake_case)]
impl TextDecorationInset {
    pub fn new(start: &Length, end: &Length) -> Self {
        Self {
            start_: start.clone(),
            end_: end.clone(),
        }
    }

    // cpp: layoutng_style/style/text_decoration_inset.h:20-21
    pub fn GetStart(&self) -> &Length {
        &self.start_
    }
    pub fn GetEnd(&self) -> &Length {
        &self.end_
    }
}

// cpp: layoutng_style/style/text_decoration_inset.h:23-28
// The derived PartialEq compares the two private Length fields in order.
