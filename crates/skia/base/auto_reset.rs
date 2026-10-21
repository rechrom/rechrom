// Source: base/auto_reset.h: actual AutoReset Copy-valued instantiations (the recorder uses bool).
// Non-Copy native moved-from T domains are not claimed by this specialization.
#[must_use]
pub struct AutoReset<T: Copy> {
    scoped_variable_: *mut T,
    original_value_: T,
}
impl<T: Copy> AutoReset<T> {
    pub unsafe fn new(variable: *mut T, value: T) -> Self {
        let original_value_ = *variable;
        *variable = value;
        Self {
            scoped_variable_: variable,
            original_value_,
        }
    }
    pub unsafe fn move_from(other: &mut Self) -> Self {
        Self {
            scoped_variable_: core::mem::replace(
                &mut other.scoped_variable_,
                core::ptr::null_mut(),
            ),
            original_value_: other.original_value_,
        }
    }
    pub unsafe fn move_assign(&mut self, other: &mut Self) {
        self.Reset();
        self.scoped_variable_ =
            core::mem::replace(&mut other.scoped_variable_, core::ptr::null_mut());
        self.original_value_ = other.original_value_;
    }
    unsafe fn Reset(&mut self) {
        if !self.scoped_variable_.is_null() {
            *self.scoped_variable_ = self.original_value_;
        }
    }
}
impl<T: Copy> Drop for AutoReset<T> {
    fn drop(&mut self) {
        unsafe {
            self.Reset();
        }
    }
}
// expected_old_value constructor requires the actual streamed DCHECK_EQ protocol, separately pending.
