// C++: foundation/blink_base/heap/threading_traits.h. Rust trait impls model
// the C++ template specializations; the associated constant keeps the source
// default for types that explicitly implement the trait.

// cpp: foundation/blink_base/heap/threading_traits.h:10-13
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadAffinity {
    kAnyThread = 0,
    kMainThreadOnly = 1,
}

// cpp: foundation/blink_base/heap/threading_traits.h:15-19
pub trait ThreadingTrait {
    const kAffinity: ThreadAffinity = ThreadAffinity::kAnyThread;
}
