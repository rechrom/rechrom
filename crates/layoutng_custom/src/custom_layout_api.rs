// The block profile stores an optional custom layout instance but does not
// install the custom algorithm. The remaining API and callbacks are pending.
mod pending {
    pub trait Sealed {}
}

// cpp: layoutng_custom/custom_layout_api.h:87-91
pub trait CustomLayout: pending::Sealed {
    fn CreateInstance(&self) -> Box<dyn CustomLayoutInstance>;
}

// The instance methods depend on the still-untranslated custom layout
// context/result records. This trait is only a type boundary for this profile;
// it cannot be implemented here without that method surface. Sealing both
// traits prevents a behaviorless custom-layout implementation from being used.
// cpp: layoutng_custom/custom_layout_api.h:79-84
pub trait CustomLayoutInstance: pending::Sealed {}
