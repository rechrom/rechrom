// These definitions are absent from the reduced LayoutNG source snapshot.
// Their bytes match Chromium's layout_invalidation_reason.cc so callers can
// pass stable NUL-terminated tracing labels without changing layout behavior.
#![allow(non_upper_case_globals)]

macro_rules! reason {
    ($name:ident, $bytes:expr) => {
        #[unsafe(no_mangle)]
        pub static $name: [u8; $bytes.len()] = *$bytes;
    };
}

reason!(kAddedToLayout, b"Added to layout\0");
reason!(kAnonymousBlockChange, b"Anonymous block change\0");
reason!(
    kChildAnonymousBlockChanged,
    b"Child anonymous block changed\0"
);
reason!(kFontsChanged, b"Fonts changed\0");
reason!(kRemovedFromLayout, b"Removed from layout\0");
reason!(kScrollbarChanged, b"Scrollbar changed\0");
reason!(kSizeChanged, b"Size changed\0");
reason!(kStyleChange, b"Style changed\0");
reason!(kTextChanged, b"Text changed\0");
reason!(kChildChanged, b"Child changed\0");

// Labels also verified against the unchanged source build's native
// layout_invalidation_reason.cc.o string data.
reason!(kTableChanged, b"Table changed\0");
reason!(kAttributeChanged, b"Attribute changed\0");

reason!(kTextControlChanged, b"Text control changed\0");
