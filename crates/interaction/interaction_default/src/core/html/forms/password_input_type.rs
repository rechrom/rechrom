// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/forms/password_input_type.cc:127-150.
//! The password view's indicator state belongs to its canonical control owner.
//! The platform supplies caps-lock state; Layout owns paint invalidation.
pub trait PasswordCapsLockContext {
    type Error;
    fn FrameIsFocusedAndActive(&self) -> bool;
    fn IsFocusedElement(&self) -> bool;
    fn CurrentCapsLockState(&self) -> Result<bool, Self::Error>;
    fn ShouldDrawCapsLockIndicator(&self) -> bool;
    fn SetShouldDrawCapsLockIndicator(&self, value: bool);
    fn SetShouldDoFullPaintInvalidation(&self);
}
pub fn CapsLockStateMayHaveChanged<C: PasswordCapsLockContext>(
    context: &C,
) -> Result<(), C::Error> {
    let should_draw = context.FrameIsFocusedAndActive()
        && context.IsFocusedElement()
        && context.CurrentCapsLockState()?;
    if should_draw != context.ShouldDrawCapsLockIndicator() {
        context.SetShouldDrawCapsLockIndicator(should_draw);
        context.SetShouldDoFullPaintInvalidation();
    }
    Ok(())
}
