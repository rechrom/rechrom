//! Platform boundary for the CPU floating-point control state used by
//! Chromium's `ScopedSubnormalFloatDisabler`.
//!
//! Reading or writing the control register belongs to the embedding runtime.
//! The translated cc type still owns the scoped restore order.

pub trait SubnormalFloatControl {
    type State;

    fn DisableSubnormalFloats(&mut self) -> Self::State;
    fn RestoreSubnormalFloatState(&mut self, state: Self::State);
}
