pub mod cc {
    // cpp: layoutng_style/style/scroll_snap_data.h:8-15
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(u32)]
    pub enum SnapAxis {
        #[default]
        kBoth,
        kX,
        kY,
        kBlock,
        kInline,
        kPair,
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:18
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u32)]
    pub enum SearchAxis {
        kX,
        kY,
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:21
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(u32)]
    pub enum SnapStrictness {
        #[default]
        kProximity,
        kMandatory,
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:24
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(u32)]
    pub enum SnapAlignment {
        #[default]
        kNone,
        kStart,
        kEnd,
        kCenter,
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:26-49
    #[derive(Clone, Copy, Debug)]
    pub struct ScrollSnapType {
        pub is_none: bool,
        pub axis: SnapAxis,
        pub strictness: SnapStrictness,
    }

    impl ScrollSnapType {
        // cpp: layoutng_style/style/scroll_snap_data.h:32-33
        pub fn new(snap_type_none: bool, axis: SnapAxis, strictness: SnapStrictness) -> Self {
            Self {
                is_none: snap_type_none,
                axis,
                strictness,
            }
        }
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:27-30
    impl Default for ScrollSnapType {
        fn default() -> Self {
            Self::new(true, SnapAxis::kBoth, SnapStrictness::kProximity)
        }
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:35-42
    impl PartialEq for ScrollSnapType {
        fn eq(&self, other: &Self) -> bool {
            self.is_none == other.is_none
                && self.axis == other.axis
                && self.strictness == other.strictness
        }
        fn ne(&self, other: &Self) -> bool {
            !self.eq(other)
        }
    }

    impl Eq for ScrollSnapType {}

    // cpp: layoutng_style/style/scroll_snap_data.h:51-73
    #[derive(Clone, Copy, Debug)]
    pub struct ScrollSnapAlign {
        pub alignment_block: SnapAlignment,
        pub alignment_inline: SnapAlignment,
    }

    impl ScrollSnapAlign {
        // cpp: layoutng_style/style/scroll_snap_data.h:56-57
        pub fn from_alignment(alignment: SnapAlignment) -> Self {
            Self {
                alignment_block: alignment,
                alignment_inline: alignment,
            }
        }

        // cpp: layoutng_style/style/scroll_snap_data.h:59-60
        pub fn new(b: SnapAlignment, i: SnapAlignment) -> Self {
            Self {
                alignment_block: b,
                alignment_inline: i,
            }
        }
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:52-54
    impl Default for ScrollSnapAlign {
        fn default() -> Self {
            Self::from_alignment(SnapAlignment::kNone)
        }
    }

    // cpp: layoutng_style/style/scroll_snap_data.h:62-69
    impl PartialEq for ScrollSnapAlign {
        fn eq(&self, other: &Self) -> bool {
            self.alignment_block == other.alignment_block
                && self.alignment_inline == other.alignment_inline
        }
        fn ne(&self, other: &Self) -> bool {
            !self.eq(other)
        }
    }

    impl Eq for ScrollSnapAlign {}
}
