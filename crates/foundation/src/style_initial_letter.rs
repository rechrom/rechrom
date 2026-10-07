// C++: src/foundation/style_values/style/style_initial_letter.h:13-50
// C++: src/foundation/style_values/style/style_initial_letter.cc:12-52

// C++ private SinkType enum: style_initial_letter.h:37-43.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SinkType {
    #[default]
    None,
    Omitted,
    Integer,
    Drop,
    Raise,
}

// C++: style_initial_letter.h:15-49.
#[derive(Clone, Copy, Debug, Default)]
pub struct StyleInitialLetter {
    size_: f32,
    sink_: i32,
    sink_type_: SinkType,
}

#[allow(non_snake_case)]
impl StyleInitialLetter {
    // C++: style_initial_letter.cc:14-21.
    pub fn with_size(size: f32) -> Self {
        let sink = size.floor() as i32;
        debug_assert!(size >= 1.0);
        debug_assert!(sink >= 1);
        Self {
            size_: size,
            sink_: sink,
            sink_type_: SinkType::Omitted,
        }
    }

    // C++: style_initial_letter.cc:23-28.
    pub fn with_sink(size: f32, sink: i32) -> Self {
        debug_assert!(size >= 1.0);
        debug_assert!(sink >= 1);
        Self {
            size_: size,
            sink_: sink,
            sink_type_: SinkType::Integer,
        }
    }

    // C++: style_initial_letter.cc:30-39.
    fn with_sink_type(size: f32, sink_type: SinkType) -> Self {
        let sink = if sink_type == SinkType::Drop {
            size.floor() as i32
        } else {
            1
        };
        debug_assert!(size >= 1.0);
        debug_assert!(sink >= 1);
        debug_assert!(sink_type == SinkType::Drop || sink_type == SinkType::Raise);
        Self {
            size_: size,
            sink_: sink,
            sink_type_: sink_type,
        }
    }

    // C++: style_initial_letter.h:27-33.
    pub fn IsDrop(&self) -> bool {
        self.sink_type_ == SinkType::Drop
    }
    pub fn IsIntegerSink(&self) -> bool {
        self.sink_type_ == SinkType::Integer
    }
    pub fn IsNormal(&self) -> bool {
        self.size_ == 0.0
    }
    pub fn IsRaise(&self) -> bool {
        self.sink_type_ == SinkType::Raise
    }
    pub fn Sink(&self) -> i32 {
        self.sink_
    }
    pub fn Size(&self) -> f32 {
        self.size_
    }

    // C++: style_initial_letter.h:35, style_initial_letter.cc:47-52.
    pub fn Normal() -> Self {
        Self::default()
    }
    pub fn Drop(size: f32) -> Self {
        Self::with_sink_type(size, SinkType::Drop)
    }
    pub fn Raise(size: f32) -> Self {
        Self::with_sink_type(size, SinkType::Raise)
    }
}

// C++: style_initial_letter.cc:41-44. Compare all three fields in source order.
impl PartialEq for StyleInitialLetter {
    fn eq(&self, other: &Self) -> bool {
        self.size_ == other.size_
            && self.sink_ == other.sink_
            && self.sink_type_ == other.sink_type_
    }
}

// The layoutng input translator calls these four source-owned constructors.
#[unsafe(no_mangle)]
pub extern "Rust" fn NativeInitialLetterOmitted(size: f32) -> StyleInitialLetter {
    StyleInitialLetter::with_size(size)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn NativeInitialLetterInteger(size: f32, sink: i32) -> StyleInitialLetter {
    StyleInitialLetter::with_sink(size, sink)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn NativeInitialLetterDrop(size: f32) -> StyleInitialLetter {
    StyleInitialLetter::Drop(size)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn NativeInitialLetterRaise(size: f32) -> StyleInitialLetter {
    StyleInitialLetter::Raise(size)
}
