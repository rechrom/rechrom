use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    focus_controller::FocusController,
    input_event::MouseButton,
    ownership::{
        Attribute, HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement,
        ReadNode,
    },
    text_editor::{SyntheticEventDispatcher, TextEditor},
};
use dom::{
    dom_mutation::{DOMMutation, DOMMutationType},
    persistent_document::DOMNodeType,
    Document, UserInteractionState,
};
use std::{any::Any, cell::RefCell, rc::Rc};
// cpp: interaction/input_type.h:19-30
pub trait ActivationState: Any {
    fn AsAny(&self) -> &dyn Any;
}
#[derive(Clone)]
pub struct InputTypeContext<'a> {
    pub document: InteractionDocument,
    pub emit_mutation: InteractionDOMMutationEmitter,
    pub interaction_state: Rc<RefCell<UserInteractionState>>,
    pub editor: Rc<TextEditor>,
    pub focus_controller: FocusController<'a>,
    pub submit_form: Option<Rc<dyn Fn(u64, Option<u64>)>>,
    pub submitting_forms: Rc<RefCell<Vec<u64>>>,
    pub dispatch_synthetic: SyntheticEventDispatcher<'a>,
}
// cpp: interaction/input_type.h:35-47
// cpp: interaction/input_type.cc:475-483
pub trait InputType {
    fn HasActivationBehavior(&self) -> bool {
        false
    }
    fn LegacyPreActivationBehavior(&self, _: &mut Event) -> Option<Box<dyn ActivationState>> {
        None
    }
    fn RunActivationBehavior(&self, _: &mut Event, _: Option<&dyn ActivationState>) {}
    fn HandleDefaultEvent(&self, _: &mut Event) {}
}
struct BaseInputType<'a> {
    _node: u64,
    _context: InputTypeContext<'a>,
}
impl<'a> InputType for BaseInputType<'a> {}
// cpp: interaction/input_type.cc:18-30
fn LowerASCII(v: &str) -> String {
    v.to_ascii_lowercase()
}
fn AttributeValue(c: &InputTypeContext<'_>, id: u64, name: &str) -> String {
    Attribute(&c.document, id, name)
}
fn Element(c: &InputTypeContext<'_>, id: u64, name: &str) -> bool {
    ReadNode(&c.document, id, |_, n, _| IsElement(n, name)).unwrap_or(false)
}
// cpp: interaction/input_type.cc:32-37
fn ElementIds(document: &Document, root: usize) -> Vec<u64> {
    fn visit(d: &Document, i: usize, out: &mut Vec<u64>) {
        let n = d.Node(i);
        if n.Type() == DOMNodeType::kElement {
            out.push(n.Id())
        }
        for &child in n.Children() {
            visit(d, child, out)
        }
    }
    let mut ids = Vec::new();
    visit(document, root, &mut ids);
    ids
}
fn AllElements(c: &InputTypeContext<'_>) -> Vec<u64> {
    let owner = c.document.borrow();
    let d = owner.GetDocument();
    ElementIds(d, d.Root())
}
fn Descendants(c: &InputTypeContext<'_>, id: u64) -> Vec<u64> {
    ReadNode(&c.document, id, |d, _, i| ElementIds(d, i)).unwrap_or_default()
}
// cpp: interaction/input_type.cc:39-46
fn DispatchInputAndChange(id: u64, dispatch: &SyntheticEventDispatcher<'_>) {
    let mut input = MakeSyntheticEvent(EventType::kInput, id);
    dispatch(&mut input, id);
    let mut change = MakeSyntheticEvent(EventType::kChange, id);
    dispatch(&mut change, id);
}
// cpp: interaction/input_type.cc:48-64
fn DispatchKeyPress(event: &mut Event, id: u64, dispatch: &SyntheticEventDispatcher<'_>) -> bool {
    if event.r#type != EventType::kKeyDown
        || (event.text.is_empty() && event.key != "Enter" && event.key != " ")
    {
        return true;
    }
    let mut keypress = MakeSyntheticEvent(EventType::kKeyPress, id);
    keypress.key = event.key.clone();
    keypress.text = event.text.clone();
    keypress.modifiers = event.modifiers;
    keypress.repeat = event.repeat;
    dispatch(&mut keypress, id);
    if !keypress.default_prevented {
        return true;
    }
    event.default_handled = true;
    false
}
// cpp: interaction/input_type.cc:66-90
fn FindElementById(c: &InputTypeContext<'_>, id: &str) -> Option<u64> {
    AllElements(c)
        .into_iter()
        .find(|&node| AttributeValue(c, node, "id") == id && HasAttribute(&c.document, node, "id"))
}
fn FormOwner(c: &InputTypeContext<'_>, id: u64) -> Option<u64> {
    if HasAttribute(&c.document, id, "form") {
        return FindElementById(c, &AttributeValue(c, id, "form"))
            .filter(|&n| Element(c, n, "form"));
    }
    ReadNode(&c.document, id, |d, n, _| {
        let mut ancestor = n.Parent();
        while let Some(i) = ancestor {
            let n = d.Node(i);
            if IsElement(n, "form") {
                return Some(n.Id());
            }
            ancestor = n.Parent()
        }
        None
    })
    .flatten()
}
// cpp: interaction/input_type.cc:92-110
fn NodeText(c: &InputTypeContext<'_>, id: u64) -> String {
    ReadNode(&c.document, id, |d, _, i| {
        fn append(d: &Document, i: usize, s: &mut String) {
            let n = d.Node(i);
            if n.Type() == DOMNodeType::kText {
                s.push_str(n.Data())
            }
            for &child in n.Children() {
                append(d, child, s)
            }
        }
        let mut s = String::new();
        append(d, i, &mut s);
        s
    })
    .unwrap_or_default()
}
fn OptionValue(c: &InputTypeContext<'_>, id: u64) -> String {
    if HasAttribute(&c.document, id, "value") {
        AttributeValue(c, id, "value")
    } else {
        NodeText(c, id)
    }
}
// cpp: interaction/input_type.cc:112-126
// Source strtod consumes ASCII C-locale leading whitespace and a C string.
// Decimal parsing uses Rust's correctly rounded IEEE conversion; hexadecimal
// mantissas are rounded once directly into the IEEE significand below.
fn ParseNumber(text: &str) -> Option<f64> {
    let text = text.split('\0').next().unwrap_or("");
    let text =
        text.trim_start_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c'));
    if text.is_empty() {
        return None;
    }
    let (negative, body) = if let Some(v) = text.strip_prefix('-') {
        (true, v)
    } else {
        (false, text.strip_prefix('+').unwrap_or(text))
    };
    let value = if body.starts_with("0x") || body.starts_with("0X") {
        ParseHexFloat(&body[2..], negative)?
    } else {
        text.parse::<f64>().ok()?
    };
    value.is_finite().then_some(value)
}
fn ParseHexFloat(body: &str, negative: bool) -> Option<f64> {
    let (mantissa, exponent) = if let Some(i) = body.find(['p', 'P']) {
        let e = &body[i + 1..];
        if e.is_empty() {
            return None;
        }
        let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let parsed = e.parse::<i64>().unwrap_or(if e.starts_with('-') {
            i64::MIN
        } else {
            i64::MAX
        });
        (&body[..i], parsed)
    } else {
        (body, 0)
    };
    let mut bits = Vec::new();
    let mut after_point = false;
    let mut fractional = 0i64;
    let mut digits = 0;
    for byte in mantissa.bytes() {
        if byte == b'.' {
            if after_point {
                return None;
            }
            after_point = true;
            continue;
        }
        let v = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        digits += 1;
        if after_point {
            fractional = fractional.saturating_add(1)
        }
        for shift in (0..4).rev() {
            bits.push(v & (1 << shift) != 0)
        }
    }
    if digits == 0 {
        return None;
    }
    let sign = if negative { 1u64 << 63 } else { 0 };
    let Some(first) = bits.iter().position(|b| *b) else {
        return Some(f64::from_bits(sign));
    };
    let bits = &bits[first..];
    let scale = exponent.saturating_sub(fractional.saturating_mul(4));
    let mut high = scale.saturating_add(bits.len() as i64 - 1);
    if high > 1023 {
        return Some(if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    if high < -1075 {
        return Some(f64::from_bits(sign));
    }
    let subnormal = high < -1022;
    let count = if subnormal {
        scale
            .saturating_add(bits.len() as i64)
            .saturating_add(1074)
            .max(0) as usize
    } else {
        53
    };
    let mut significand = 0u64;
    for i in 0..count {
        significand = (significand << 1) | u64::from(bits.get(i).copied().unwrap_or(false));
    }
    let guard = bits.get(count).copied().unwrap_or(false);
    let sticky = bits.iter().skip(count + 1).any(|b| *b);
    if guard && (sticky || significand & 1 != 0) {
        significand += 1
    }
    if subnormal {
        return Some(f64::from_bits(sign | significand));
    }
    if significand == 1u64 << 53 {
        significand >>= 1;
        high += 1
    }
    if high > 1023 {
        return Some(if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    Some(f64::from_bits(
        sign | (((high + 1023) as u64) << 52) | (significand & ((1u64 << 52) - 1)),
    ))
}
fn NumberAttribute(c: &InputTypeContext<'_>, id: u64, name: &str, fallback: f64) -> f64 {
    ParseNumber(&AttributeValue(c, id, name)).unwrap_or(fallback)
}
// cpp: interaction/input_type.cc:128-132
// General format, 15 significant decimal digits, source exponent threshold
// [-4,15), source exponent sign/two-digit minimum and fractional zero removal.
fn SerializeNumber(value: f64) -> String {
    let scientific = format!("{:.14e}", value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let digits = mantissa.replace('.', "");
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if exponent < -4 || exponent >= 15 {
        let tail = digits[1..].trim_end_matches('0');
        let coefficient = if tail.is_empty() {
            digits[..1].to_owned()
        } else {
            format!("{}.{}", &digits[..1], tail)
        };
        return format!(
            "{}{}e{}{:02}",
            sign,
            coefficient,
            if exponent < 0 { "-" } else { "+" },
            exponent.unsigned_abs()
        );
    }
    let point = exponent + 1;
    let mut fixed = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if fixed.contains('.') {
        fixed = fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
    format!("{sign}{fixed}")
}
fn SetValue(c: &InputTypeContext<'_>, id: u64, value: String) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlValue,
        target_node_id: id,
        value,
        ..Default::default()
    })
}
fn SetChecked(c: &InputTypeContext<'_>, id: u64, checked: bool) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlChecked,
        target_node_id: id,
        bool_value: checked,
        ..Default::default()
    })
}
fn Checked(c: &InputTypeContext<'_>, id: u64) -> bool {
    ReadNode(&c.document, id, |d, _, i| d.ControlChecked(i)).unwrap_or(false)
}
fn Value(c: &InputTypeContext<'_>, id: u64) -> String {
    ReadNode(&c.document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default()
}
// cpp: interaction/input_type.cc:134-175
fn ResetForm(form: u64, c: &InputTypeContext<'_>) {
    for id in Descendants(c, form) {
        if Element(c, id, "input") {
            let ty = LowerASCII(&AttributeValue(c, id, "type"));
            if ty == "checkbox" || ty == "radio" {
                SetChecked(c, id, HasAttribute(&c.document, id, "checked"))
            } else {
                SetValue(c, id, AttributeValue(c, id, "value"))
            }
        } else if Element(c, id, "textarea") {
            SetValue(c, id, NodeText(c, id))
        } else if Element(c, id, "select") {
            let options: Vec<_> = Descendants(c, id)
                .into_iter()
                .filter(|&n| Element(c, n, "option"))
                .collect();
            let chosen = options
                .iter()
                .copied()
                .find(|&n| HasAttribute(&c.document, n, "selected"))
                .or_else(|| options.first().copied());
            SetValue(c, id, chosen.map_or(String::new(), |n| OptionValue(c, n)))
        }
    }
}
/// Script submit bypasses the cancelable submit event. Request/activation
/// submission suppresses recursive requestSubmit on the same form, as Blink's
/// HTMLFormElement::PrepareForSubmission does while firing that event.
pub(crate) fn SubmitForm(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
    dispatch_event: bool,
) {
    if !Element(c, form, "form")
        || !ReadNode(&c.document, form, |d, _, index| {
            let mut root = index;
            while let Some(parent) = d.Node(root).Parent() {
                root = parent;
            }
            root == d.Root()
        })
        .unwrap_or(false)
    {
        return;
    }
    if dispatch_event {
        if c.submitting_forms.borrow().contains(&form) {
            return;
        }
        c.submitting_forms.borrow_mut().push(form);
        struct SubmittingGuard(Rc<RefCell<Vec<u64>>>, u64);
        impl Drop for SubmittingGuard {
            fn drop(&mut self) {
                self.0.borrow_mut().retain(|&id| id != self.1);
            }
        }
        let _guard = SubmittingGuard(c.submitting_forms.clone(), form);
        let mut submit = MakeSyntheticEvent(EventType::kSubmit, form);
        submit.submitter_node_id = submitter;
        (c.dispatch_synthetic)(&mut submit, form);
        if submit.default_prevented {
            return;
        }
    }
    if let Some(callback) = &c.submit_form {
        callback(form, submitter);
    }
}
// cpp: interaction/input_type.cc:177-195
fn ActivateButton(id: u64, event: &mut Event, c: &InputTypeContext<'_>) {
    let ty = LowerASCII(&AttributeValue(c, id, "type"));
    let reset = ty == "reset";
    let submit = ty == "submit"
        || ty == "image"
        || (Element(c, id, "button") && !matches!(ty.as_str(), "button" | "reset"));
    if !reset && !submit {
        return;
    }
    let Some(form) = FormOwner(c, id) else { return };
    if reset {
        let mut e = MakeSyntheticEvent(EventType::kReset, form);
        (c.dispatch_synthetic)(&mut e, form);
        if !e.default_prevented {
            ResetForm(form, c);
        }
    } else {
        SubmitForm(c, form, Some(id), true);
    }
    event.default_handled = true;
}
// cpp: interaction/input_type.cc:197-202
struct CheckableActivationState {
    previous_checked: Vec<(u64, bool)>,
    checkedness_changed: bool,
}
impl ActivationState for CheckableActivationState {
    fn AsAny(&self) -> &dyn Any {
        self
    }
}
// cpp: interaction/input_type.cc:204-290
struct CheckableInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for CheckableInputType<'a> {
    fn HasActivationBehavior(&self) -> bool {
        true
    }
    fn LegacyPreActivationBehavior(&self, _: &mut Event) -> Option<Box<dyn ActivationState>> {
        let id = self.node;
        let c = &self.c;
        let mut state = CheckableActivationState {
            previous_checked: Vec::new(),
            checkedness_changed: false,
        };
        let ty = LowerASCII(&AttributeValue(c, id, "type"));
        if ty == "radio" {
            let name = AttributeValue(c, id, "name");
            let was_checked = Checked(c, id);
            state.checkedness_changed = !was_checked;
            if name.is_empty() {
                state.previous_checked.push((id, was_checked))
            } else {
                for candidate in AllElements(c) {
                    if !Element(c, candidate, "input")
                        || LowerASCII(&AttributeValue(c, candidate, "type")) != "radio"
                        || AttributeValue(c, candidate, "name") != name
                    {
                        continue;
                    }
                    let checked = Checked(c, candidate);
                    state.previous_checked.push((candidate, checked));
                    if checked && candidate != id {
                        SetChecked(c, candidate, false)
                    }
                }
            }
            if !was_checked {
                SetChecked(c, id, true)
            }
        } else {
            let checked = Checked(c, id);
            state.checkedness_changed = true;
            state.previous_checked.push((id, checked));
            SetChecked(c, id, !checked)
        }
        Some(Box::new(state))
    }
    fn RunActivationBehavior(&self, event: &mut Event, base: Option<&dyn ActivationState>) {
        let Some(state) = base.and_then(|s| s.AsAny().downcast_ref::<CheckableActivationState>())
        else {
            return;
        };
        if event.default_prevented || event.default_handled {
            for &(id, checked) in &state.previous_checked {
                SetChecked(&self.c, id, checked)
            }
        } else if state.checkedness_changed {
            DispatchInputAndChange(self.node, &self.c.dispatch_synthetic)
        }
        event.default_handled = true;
    }
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let c = &self.c;
        let id = self.node;
        let ty = LowerASCII(&AttributeValue(c, id, "type"));
        if ty == "radio"
            && event.r#type == EventType::kKeyDown
            && matches!(
                event.key.as_str(),
                "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight"
            )
            && !event.modifiers.control
            && !event.modifiers.meta
            && !event.modifiers.alt
        {
            let name = AttributeValue(c, id, "name");
            let group: Vec<_> = AllElements(c)
                .into_iter()
                .filter(|&n| {
                    Element(c, n, "input")
                        && LowerASCII(&AttributeValue(c, n, "type")) == "radio"
                        && AttributeValue(c, n, "name") == name
                        && !IsDisabledFormControl(&c.document, n)
                })
                .collect();
            if let Some(index) = group.iter().position(|&n| n == id) {
                if group.len() > 1 {
                    let forward = event.key == "ArrowDown" || event.key == "ArrowRight";
                    let index = if forward {
                        (index + 1) % group.len()
                    } else {
                        (index + group.len() - 1) % group.len()
                    };
                    let next = group[index];
                    c.focus_controller.Focus(next, true);
                    let mut click = MakeSyntheticEvent(EventType::kClick, next);
                    click.button = MouseButton::kPrimary;
                    (c.dispatch_synthetic)(&mut click, next);
                    event.default_handled = true;
                }
            }
            return;
        }
        if event.r#type == EventType::kKeyDown && event.key == " " {
            if !DispatchKeyPress(event, id, &c.dispatch_synthetic) {
                return;
            }
            c.interaction_state.borrow_mut().pressed_node_id = Some(id);
            return;
        }
        if event.r#type == EventType::kKeyUp
            && event.key == " "
            && c.interaction_state.borrow().pressed_node_id == Some(id)
        {
            c.interaction_state.borrow_mut().pressed_node_id = None;
            let mut click = MakeSyntheticEvent(EventType::kClick, id);
            click.button = MouseButton::kPrimary;
            (c.dispatch_synthetic)(&mut click, id);
            event.default_handled = true;
        }
    }
}
// cpp: interaction/input_type.cc:292-334
struct TextFieldInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for TextFieldInputType<'a> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let c = &self.c;
        let id = self.node;
        if event.r#type == EventType::kKeyDown
            && !DispatchKeyPress(event, id, &c.dispatch_synthetic)
        {
            return;
        }
        if event.r#type == EventType::kKeyDown
            && event.key == "Enter"
            && !Element(c, id, "textarea")
        {
            let mut before = MakeSyntheticEvent(EventType::kBeforeInput, id);
            before.input_type = "insertLineBreak".into();
            (c.dispatch_synthetic)(&mut before, id);
            if !before.default_prevented {
                if let Some(form) = FormOwner(c, id) {
                    let submitter = AllElements(c).into_iter().find(|&n| {
                        if FormOwner(c, n) != Some(form) {
                            return false;
                        }
                        let ty = LowerASCII(&AttributeValue(c, n, "type"));
                        if Element(c, n, "button") {
                            !matches!(ty.as_str(), "button" | "reset")
                        } else {
                            Element(c, n, "input") && (ty == "submit" || ty == "image")
                        }
                    });
                    if let Some(submitter) = submitter {
                        if IsDisabledFormControl(&c.document, submitter) {
                            event.default_handled = true;
                            return;
                        }
                        let mut click = MakeSyntheticEvent(EventType::kClick, submitter);
                        click.button = MouseButton::kPrimary;
                        (c.dispatch_synthetic)(&mut click, submitter)
                    } else {
                        let blocking_fields = AllElements(c)
                            .into_iter()
                            .filter(|&n| {
                                Element(c, n, "input")
                                    && FormOwner(c, n) == Some(form)
                                    && matches!(
                                        LowerASCII(&AttributeValue(c, n, "type")).as_str(),
                                        "" | "text"
                                            | "search"
                                            | "url"
                                            | "tel"
                                            | "email"
                                            | "password"
                                            | "date"
                                            | "month"
                                            | "week"
                                            | "time"
                                            | "datetime-local"
                                            | "number"
                                    )
                            })
                            .count();
                        if blocking_fields <= 1 {
                            SubmitForm(c, form, None, true);
                        }
                    }
                }
            }
            event.default_handled = true;
            return;
        }
        c.editor.HandleEvent(
            event,
            &c.document,
            id,
            &c.emit_mutation,
            &c.dispatch_synthetic,
        );
    }
}
// cpp: interaction/input_type.cc:336-369
struct ButtonInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for ButtonInputType<'a> {
    fn HasActivationBehavior(&self) -> bool {
        true
    }
    fn RunActivationBehavior(&self, event: &mut Event, _: Option<&dyn ActivationState>) {
        if !event.default_prevented {
            ActivateButton(self.node, event, &self.c)
        }
    }
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let c = &self.c;
        let id = self.node;
        if event.r#type == EventType::kKeyDown && event.key == " " {
            if !DispatchKeyPress(event, id, &c.dispatch_synthetic) {
                return;
            }
            c.interaction_state.borrow_mut().pressed_node_id = Some(id);
            return;
        }
        if event.r#type == EventType::kKeyDown && event.key == "Enter" {
            if !DispatchKeyPress(event, id, &c.dispatch_synthetic) {
                return;
            }
            let mut click = MakeSyntheticEvent(EventType::kClick, id);
            click.button = MouseButton::kPrimary;
            (c.dispatch_synthetic)(&mut click, id);
            event.default_handled = true;
            return;
        }
        if event.r#type == EventType::kKeyUp
            && event.key == " "
            && c.interaction_state.borrow().pressed_node_id == Some(id)
        {
            c.interaction_state.borrow_mut().pressed_node_id = None;
            let mut click = MakeSyntheticEvent(EventType::kClick, id);
            click.button = MouseButton::kPrimary;
            (c.dispatch_synthetic)(&mut click, id);
            event.default_handled = true;
        }
    }
}
// cpp: interaction/input_type.cc:371-404
struct RangeInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for RangeInputType<'a> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        if event.r#type != EventType::kKeyDown {
            return;
        }
        let c = &self.c;
        let id = self.node;
        let minimum = NumberAttribute(c, id, "min", 0.0);
        let maximum = minimum.max(NumberAttribute(c, id, "max", 100.0));
        let mut step = NumberAttribute(c, id, "step", 1.0);
        if !(step > 0.0) {
            step = 1.0
        }
        let mut value = NumberAttribute(c, id, "value", (minimum + maximum) / 2.0);
        let live = Value(c, id);
        if let Some(parsed) = ParseNumber(&live) {
            value = parsed
        }
        let next = match event.key.as_str() {
            "ArrowLeft" | "ArrowDown" => value - step,
            "ArrowRight" | "ArrowUp" => value + step,
            "Home" => minimum,
            "End" => maximum,
            _ => return,
        }
        .clamp(minimum, maximum);
        if next != value {
            SetValue(c, id, SerializeNumber(next));
            DispatchInputAndChange(id, &c.dispatch_synthetic)
        }
        event.default_handled = true;
    }
}
// cpp: interaction/input_type.cc:406-438
struct SelectInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for SelectInputType<'a> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        if event.r#type != EventType::kKeyDown {
            return;
        }
        let c = &self.c;
        let id = self.node;
        let options: Vec<_> = Descendants(c, id)
            .into_iter()
            .filter(|&n| Element(c, n, "option") && !HasAttribute(&c.document, n, "disabled"))
            .collect();
        if options.is_empty() {
            return;
        }
        let current = Value(c, id);
        let index = options
            .iter()
            .position(|&n| OptionValue(c, n) == current)
            .unwrap_or(0);
        let next = match event.key.as_str() {
            "ArrowDown" | "ArrowRight" => (index + 1).min(options.len() - 1),
            "ArrowUp" | "ArrowLeft" => index.saturating_sub(1),
            "Home" => 0,
            "End" => options.len() - 1,
            _ => return,
        };
        if next != index {
            SetValue(c, id, OptionValue(c, options[next]));
            DispatchInputAndChange(id, &c.dispatch_synthetic)
        }
        event.default_handled = true;
    }
}
// cpp: interaction/input_type.cc:442-466
pub fn IsDisabledFormControl(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |d, node, i| {
        if node.FindAttribute("disabled").is_some() {
            return true;
        }
        let mut ancestor = node.Parent();
        while let Some(a) = ancestor {
            let n = d.Node(a);
            ancestor = n.Parent();
            if !IsElement(n, "fieldset") || n.FindAttribute("disabled").is_none() {
                continue;
            }
            let legend = n
                .Children()
                .iter()
                .copied()
                .find(|&c| IsElement(d.Node(c), "legend"));
            let mut inside = Some(i);
            while let Some(v) = inside {
                if v == a {
                    break;
                }
                if Some(v) == legend {
                    return false;
                }
                inside = d.Node(v).Parent()
            }
            return true;
        }
        false
    })
    .unwrap_or(false)
}
// cpp: interaction/input_type.cc:468-473
pub fn IsTextField(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |_, n, _| {
        if IsElement(n, "textarea") {
            return true;
        }
        if !IsElement(n, "input") {
            return false;
        }
        let ty = n
            .FindAttribute("type")
            .map_or("", |a| a.value.as_str())
            .to_ascii_lowercase();
        matches!(
            ty.as_str(),
            "" | "text" | "search" | "email" | "password" | "tel" | "url" | "number"
        )
    })
    .unwrap_or(false)
}
// cpp: interaction/input_type.cc:485-503
pub fn CreateInputType<'a>(id: u64, c: InputTypeContext<'a>) -> Box<dyn InputType + 'a> {
    if IsTextField(&c.document, id) {
        return Box::new(TextFieldInputType { node: id, c });
    }
    if Element(&c, id, "input") {
        let ty = LowerASCII(&AttributeValue(&c, id, "type"));
        if ty == "checkbox" || ty == "radio" {
            return Box::new(CheckableInputType { node: id, c });
        }
        if ty == "range" {
            return Box::new(RangeInputType { node: id, c });
        }
        if matches!(ty.as_str(), "button" | "submit" | "reset" | "image") {
            return Box::new(ButtonInputType { node: id, c });
        }
    }
    if Element(&c, id, "select") {
        return Box::new(SelectInputType { node: id, c });
    }
    if Element(&c, id, "button") {
        return Box::new(ButtonInputType { node: id, c });
    }
    Box::new(BaseInputType {
        _node: id,
        _context: c,
    })
}

#[cfg(test)]
mod numeric_tests {
    use super::*;
    #[test]
    fn native_number_parsing_and_general_fifteen_digits_match() {
        for line in include_str!("../../../artifacts/parallel-zero-parity/shadow-state/interaction-native/numeric-results.tsv").lines(){let mut f=line.split('\t');let kind=f.next().unwrap();let input=f.next().unwrap();let expected=f.next().unwrap();if kind=="P"{let bytes:Vec<_>=(0..input.len()).step_by(2).map(|i|u8::from_str_radix(&input[i..i+2],16).unwrap()).collect();let text=String::from_utf8(bytes).unwrap();let actual=ParseNumber(&text).map_or("none".to_owned(),|v|format!("{:016x}",v.to_bits()));assert_eq!(actual,expected,"strtod input {text:?}")}else{let value=f64::from_bits(u64::from_str_radix(input,16).unwrap());assert_eq!(SerializeNumber(value),expected,"general format bits={input}")}}
    }
}
