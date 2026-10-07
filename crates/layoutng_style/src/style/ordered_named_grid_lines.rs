use foundation::AtomicString;

// cpp: layoutng_style/style/ordered_named_grid_lines.h:15-31
#[derive(Clone, PartialEq, Eq)]
pub struct NamedGridLine {
    pub line_name: AtomicString,
    pub is_in_repeat: bool,
    pub is_first_repeat: bool,
}

impl NamedGridLine {
    // cpp: layoutng_style/style/ordered_named_grid_lines.h:16-21
    pub fn new(line_name: &AtomicString, is_in_repeat: bool, is_first_repeat: bool) -> Self {
        Self {
            line_name: line_name.clone(),
            is_in_repeat,
            is_first_repeat,
        }
    }
    pub fn with_default_flags(line_name: &AtomicString) -> Self {
        Self::new(line_name, false, false)
    }
    pub fn with_repeat(line_name: &AtomicString, is_in_repeat: bool) -> Self {
        Self::new(line_name, is_in_repeat, false)
    }
}

// cpp: layoutng_style/style/ordered_named_grid_lines.h:33-34
pub type OrderedNamedGridLines = foundation::HashMap<
    usize,
    foundation::Vector<NamedGridLine>,
    foundation::IntWithZeroKeyHashTraits<usize>,
>;
