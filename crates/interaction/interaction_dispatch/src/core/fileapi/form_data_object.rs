//! The script projection retains the same mutable FormData as formdata event
//! dispatch and final encoding. No list/string projection replaces that owner.
use crate::bindings::script_invocation::ScriptNativeObject;
use std::{any::Any, cell::RefCell, rc::Rc};
pub struct FormDataObject {
    data: RefCell<fileapi::form_data::FormData>,
}
impl FormDataObject {
    pub fn new(data: fileapi::form_data::FormData) -> Rc<Self> {
        Rc::new(Self {
            data: RefCell::new(data),
        })
    }
    pub fn Read<R>(&self, read: impl FnOnce(&fileapi::form_data::FormData) -> R) -> R {
        read(&self.data.borrow())
    }
    pub fn Mutate<R>(&self, mutate: impl FnOnce(&mut fileapi::form_data::FormData) -> R) -> R {
        mutate(&mut self.data.borrow_mut())
    }
}
impl ScriptNativeObject for FormDataObject {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "FormData"
    }
}

#[derive(Clone, Debug)]
pub enum FormDataConstructionError {
    TypeError(&'static str),
    NotFound(&'static str),
    InvalidState(&'static str),
}
