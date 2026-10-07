use foundation::Visitor;

use super::filter_operations::FilterOperations;

#[allow(non_snake_case)]
impl FilterOperations {
    // cpp: layoutng_style/style/filter_operations_data.cc:28
    pub fn new() -> Self {
        Self {
            operations_: Default::default(),
        }
    }

    // cpp: layoutng_style/style/filter_operations.h:85
    // cpp: layoutng_style/style/filter_operations_data.cc:29-31
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.operations_);
    }
}
