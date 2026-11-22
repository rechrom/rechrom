use super::*;
impl DOMJavaScriptBindings {
    pub(super) fn CallFormValidation(
        &mut self,
        receiver: u64,
        member: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        let allowed = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(receiver).is_some_and(|i| {
                if member == "__validityBrand" {
                    if let Some(HostValue::String(expected)) = arguments.first() {
                        return d.Node(i).IsHTMLElement(expected);
                    }
                }
                d.FormControlClass(i).is_some() || d.ElementInternalsTargetIsFormAssociated(i)
            })
        };
        if !allowed {
            return Self::type_error("Illegal invocation");
        }
        if member == "__validityBrand" {
            return Self::value(HostValue::Boolean(true));
        }
        if member == "setCustomValidity" {
            let value: Rc<[u16]> = match arguments.first() {
                Some(HostValue::String16(value)) => value.clone(),
                Some(HostValue::String(value)) => value.encode_utf16().collect::<Vec<_>>().into(),
                _ => return Self::type_error("Custom validity message conversion required"),
            };
            let mut owner = self.document.borrow_mut();
            let d = owner.GetDocumentMut();
            let i = d.FindNodeById(receiver).unwrap();
            d.SetControlCustomValidationMessageUTF16(i, value);
            drop(owner);
            (self.emit_mutation.borrow_mut())(&DOMMutation {
                mutation_type: DOMMutationType::kSetControlNeedsValidityCheck,
                target_node_id: receiver,
                ..Default::default()
            });
            return HostResult::default();
        }
        let value = match interaction::adapter::page_form_validation::ControlValidity(
            &self.document,
            &self.form_validation_state,
            receiver,
        ) {
            Ok(value) => value,
            Err(error) => {
                return Self::type_error(&format!("Form validation service required: {error:?}"))
            }
        };
        let property = match arguments.first() {
            Some(HostValue::String(value)) => value.as_str(),
            _ => return Self::type_error("Validity property required"),
        };
        if property == "validationMessage" {
            return Self::value(HostValue::String16(value.message));
        }
        let state = value.validity;
        let result = match property {
            "willValidate" => value.will_validate,
            "valueMissing" => state.value_missing,
            "typeMismatch" => state.type_mismatch,
            "patternMismatch" => state.pattern_mismatch,
            "tooLong" => state.too_long,
            "tooShort" => state.too_short,
            "rangeUnderflow" => state.range_underflow,
            "rangeOverflow" => state.range_overflow,
            "stepMismatch" => state.step_mismatch,
            "badInput" => state.bad_input,
            "customError" => state.custom_error,
            "valid" => state.valid(),
            _ => return Self::type_error("Unknown validity property"),
        };
        Self::value(HostValue::Boolean(result))
    }
}
