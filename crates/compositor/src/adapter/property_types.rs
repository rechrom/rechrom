//! Typed property-node dependency boundary. Arithmetic remains in the owning
//! geometry/paint backends, while node state and control flow stay in cc.

use super::geometry::{self, Value};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

pub trait PropertyTransform: geometry::Transform + Clone + Default + PartialEq {
    fn IsInvertible(&self) -> bool;
}
pub trait FilterOperations: Clone + Default + PartialEq {
    fn IsEmpty(&self) -> bool;
    fn ToString(&self) -> String;
}
pub trait PropertyPath: geometry::Path + Clone + PartialEq {}
impl<T: geometry::Path + Clone + PartialEq> PropertyPath for T {}

/// Inline-storage counterpart of absl::InlinedVector used by clip caches.
/// The first N elements need no heap allocation; overflow retains its capacity.
#[derive(Clone, Debug)]
pub struct InlinedVector<T: Default + Clone, const N: usize> {
    inline: [T; N],
    len: usize,
    heap: Vec<T>,
    using_heap: bool,
}
impl<T: Default + Clone, const N: usize> Default for InlinedVector<T, N> {
    fn default() -> Self {
        Self {
            inline: std::array::from_fn(|_| T::default()),
            len: 0,
            heap: Vec::new(),
            using_heap: false,
        }
    }
}
impl<T: Default + Clone, const N: usize> InlinedVector<T, N> {
    pub fn push(&mut self, value: T) {
        if self.using_heap {
            self.heap.push(value);
        } else if self.len < N {
            self.inline[self.len] = value;
            self.len += 1;
        } else {
            self.heap.extend_from_slice(&self.inline);
            self.heap.push(value);
            self.using_heap = true;
        }
    }
    pub fn clear(&mut self) {
        if self.using_heap {
            self.heap.clear();
        } else {
            for i in 0..self.len {
                self.inline[i] = T::default();
            }
            self.len = 0;
        }
    }
    pub fn is_inline(&self) -> bool {
        !self.using_heap
    }
}
impl<T: Default + Clone, const N: usize> Deref for InlinedVector<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        if self.using_heap {
            &self.heap
        } else {
            &self.inline[..self.len]
        }
    }
}
impl<T: Default + Clone, const N: usize> DerefMut for InlinedVector<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        if self.using_heap {
            &mut self.heap
        } else {
            &mut self.inline[..self.len]
        }
    }
}
impl<T: Default + Clone + PartialEq, const N: usize> PartialEq for InlinedVector<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

pub trait PropertyTrace: geometry::TracedValue {
    fn BeginDictionaryUnnamed(&mut self);
    fn SetBoolean(&mut self, name: &str, value: bool);
    fn SetString(&mut self, name: &str, value: &str);
    fn AppendString(&mut self, value: &str);
}
struct TraceFrame {
    name: Option<String>,
    value: Value,
}
pub struct TracedValueJSON {
    stack: Vec<TraceFrame>,
}
impl Default for TracedValueJSON {
    fn default() -> Self {
        Self {
            stack: vec![TraceFrame {
                name: None,
                value: Value::Dictionary(BTreeMap::new()),
            }],
        }
    }
}
impl TracedValueJSON {
    fn set(&mut self, name: &str, value: Value) {
        match &mut self.stack.last_mut().expect("trace root").value {
            Value::Dictionary(dict) => {
                dict.insert(name.into(), value);
            }
            _ => panic!("trace property requires a dictionary"),
        }
    }
    fn append(&mut self, value: Value) {
        match &mut self.stack.last_mut().expect("trace root").value {
            Value::List(list) => list.push(value),
            _ => panic!("trace append requires an array"),
        }
    }
    fn close(&mut self) {
        assert!(self.stack.len() > 1);
        let frame = self.stack.pop().unwrap();
        if let Some(name) = frame.name {
            self.set(&name, frame.value);
        } else {
            self.append(frame.value);
        }
    }
    pub fn ToFormattedJSON(&self) -> String {
        assert_eq!(self.stack.len(), 1);
        fn quote(s: &str) -> String {
            let mut out = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        fn json(v: &Value, depth: usize) -> String {
            match v {
                Value::None => "null".into(),
                Value::Bool(v) => v.to_string(),
                Value::Int(v) => v.to_string(),
                Value::Double(v) => {
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "null".into()
                    }
                }
                Value::String(v) => quote(v),
                Value::Binary(_) => panic!("binary trace value cannot be serialized as JSON"),
                Value::List(values) => {
                    let parts: Vec<_> = values.iter().map(|v| json(v, depth + 1)).collect();
                    format!("[{}]", parts.join(", "))
                }
                Value::Dictionary(values) => {
                    if values.is_empty() {
                        return "{}".into();
                    }
                    let indent = "  ".repeat(depth + 1);
                    let parts: Vec<_> = values
                        .iter()
                        .map(|(k, v)| format!("{}{}: {}", indent, quote(k), json(v, depth + 1)))
                        .collect();
                    format!("{{\n{}\n{}}}", parts.join(",\n"), "  ".repeat(depth))
                }
            }
        }
        json(&self.stack[0].value, 0)
    }
}
impl geometry::TracedValue for TracedValueJSON {
    fn BeginDictionary(&mut self, name: &str) {
        self.stack.push(TraceFrame {
            name: Some(name.into()),
            value: Value::Dictionary(BTreeMap::new()),
        });
    }
    fn EndDictionary(&mut self) {
        assert!(matches!(
            self.stack.last().unwrap().value,
            Value::Dictionary(_)
        ));
        self.close();
    }
    fn BeginArray(&mut self, name: &str) {
        self.stack.push(TraceFrame {
            name: Some(name.into()),
            value: Value::List(Vec::new()),
        });
    }
    fn EndArray(&mut self) {
        assert!(matches!(self.stack.last().unwrap().value, Value::List(_)));
        self.close();
    }
    fn SetDouble(&mut self, name: &str, value: f64) {
        self.set(name, Value::Double(value));
    }
    fn SetInteger(&mut self, name: &str, value: i32) {
        self.set(name, Value::Int(value));
    }
    fn AppendDouble(&mut self, value: f64) {
        self.append(Value::Double(value));
    }
    fn AppendInteger(&mut self, value: i32) {
        self.append(Value::Int(value));
    }
}
impl PropertyTrace for TracedValueJSON {
    fn BeginDictionaryUnnamed(&mut self) {
        self.stack.push(TraceFrame {
            name: None,
            value: Value::Dictionary(BTreeMap::new()),
        });
    }
    fn SetBoolean(&mut self, name: &str, value: bool) {
        self.set(name, Value::Bool(value));
    }
    fn SetString(&mut self, name: &str, value: &str) {
        self.set(name, Value::String(value.into()));
    }
    fn AppendString(&mut self, value: &str) {
        self.append(Value::String(value.into()));
    }
}

/// The source ElementId tracing overload converts its u64 to SetInteger's int.
pub fn AddElementIdToTracedValue(
    id: super::input_types::ElementId,
    value: &mut impl PropertyTrace,
) {
    value.BeginDictionary("element_id");
    value.SetInteger("id_", id.GetInternalValue() as i32);
    value.EndDictionary();
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaskFilterInfo {
    pub rounded_corner_bounds: geometry::RRectF,
    pub gradient_mask: Option<geometry::LinearGradient>,
    pub clip_id: Option<i32>,
}
impl MaskFilterInfo {
    pub fn bounds(&self) -> geometry::RectF {
        self.rounded_corner_bounds.rect
    }
    pub fn IsEmpty(&self) -> bool {
        self.bounds().size().IsEmpty()
    }
    pub fn HasRoundedCorners(&self) -> bool {
        !self.IsEmpty()
            && self
                .rounded_corner_bounds
                .corner_radii
                .iter()
                .any(|r| r.x() > 0. && r.y() > 0.)
    }
    pub fn HasGradientMask(&self) -> bool {
        !self.IsEmpty()
            && self
                .gradient_mask
                .as_ref()
                .is_some_and(|gradient| !gradient.steps.is_empty())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubtreeCaptureId {
    pub token: [u64; 2],
}
impl SubtreeCaptureId {
    pub fn is_valid(&self) -> bool {
        self.token != [0, 0]
    }
    pub fn ToString(&self) -> String {
        format!(
            "SubtreeCaptureId({:016X}{:016X})",
            self.token[0], self.token[1]
        )
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewTransitionElementResourceId {
    pub transition_token: Option<[u64; 2]>,
    pub local_id: u32,
    pub for_scope_snapshot: bool,
}
impl ViewTransitionElementResourceId {
    pub fn IsValid(&self) -> bool {
        self.local_id != 0
    }
    // Source dependency: components/viz/common/view_transition_element_resource_id.cc:49-56.
    // Scope snapshots pause subframe rendering and are excluded from capture.
    pub fn MatchesToken(&self, tokens: &std::collections::BTreeSet<[u64; 2]>) -> bool {
        !self.for_scope_snapshot
            && self
                .transition_token
                .is_some_and(|token| tokens.contains(&token))
    }
    pub fn ToString(&self) -> String {
        format!(
            "ViewTransitionElementResourceId : {} [transition: {}, for_scope: {}]",
            self.local_id,
            self.transition_token
                .map(|t| format!("{:016X}{:016X}", t[0], t[1]))
                .unwrap_or_else(|| "invalid".into()),
            u8::from(self.for_scope_snapshot)
        )
    }
}
impl PartialEq for ViewTransitionElementResourceId {
    fn eq(&self, other: &Self) -> bool {
        if !self.IsValid() {
            return !other.IsValid();
        }
        self.local_id == other.local_id
            && self.transition_token == other.transition_token
            && self.for_scope_snapshot == other.for_scope_snapshot
    }
}
impl Eq for ViewTransitionElementResourceId {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    kClear,
    kSrc,
    kDst,
    kSrcOver,
    kDstOver,
    kSrcIn,
    kDstIn,
    kSrcOut,
    kDstOut,
    kSrcATop,
    kDstATop,
    kXor,
    kPlus,
    kModulate,
    kScreen,
    kOverlay,
    kDarken,
    kLighten,
    kColorDodge,
    kColorBurn,
    kHardLight,
    kSoftLight,
    kDifference,
    kExclusion,
    kMultiply,
    kHue,
    kSaturation,
    kColor,
    kLuminosity,
}
impl BlendMode {
    pub fn Name(self) -> &'static str {
        match self {
            Self::kClear => "Clear",
            Self::kSrc => "Src",
            Self::kDst => "Dst",
            Self::kSrcOver => "SrcOver",
            Self::kDstOver => "DstOver",
            Self::kSrcIn => "SrcIn",
            Self::kDstIn => "DstIn",
            Self::kSrcOut => "SrcOut",
            Self::kDstOut => "DstOut",
            Self::kSrcATop => "SrcATop",
            Self::kDstATop => "DstATop",
            Self::kXor => "Xor",
            Self::kPlus => "Plus",
            Self::kModulate => "Modulate",
            Self::kScreen => "Screen",
            Self::kOverlay => "Overlay",
            Self::kDarken => "Darken",
            Self::kLighten => "Lighten",
            Self::kColorDodge => "ColorDodge",
            Self::kColorBurn => "ColorBurn",
            Self::kHardLight => "HardLight",
            Self::kSoftLight => "SoftLight",
            Self::kDifference => "Difference",
            Self::kExclusion => "Exclusion",
            Self::kMultiply => "Multiply",
            Self::kHue => "Hue",
            Self::kSaturation => "Saturation",
            Self::kColor => "Color",
            Self::kLuminosity => "Luminosity",
        }
    }
}

/// gfx RectF's trace-only %g formatting (six significant digits).
pub fn RectFToString(rect: geometry::RectF) -> String {
    fn g(value: f32) -> String {
        if value.is_nan() {
            return "nan".into();
        }
        if value.is_infinite() {
            return if value.is_sign_negative() {
                "-inf"
            } else {
                "inf"
            }
            .into();
        }
        let scientific = format!("{:.5e}", f64::from(value));
        let (mantissa, exponent) = scientific.split_once('e').unwrap();
        let exponent: i32 = exponent.parse().unwrap();
        if exponent < -4 || exponent >= 6 {
            format!(
                "{}e{}{:02}",
                mantissa.trim_end_matches('0').trim_end_matches('.'),
                if exponent < 0 { "-" } else { "+" },
                exponent.abs()
            )
        } else {
            let value = format!("{:.*}", (5 - exponent) as usize, f64::from(value));
            if value.contains('.') {
                value.trim_end_matches('0').trim_end_matches('.').into()
            } else {
                value
            }
        }
    }
    format!(
        "{},{} {}x{}",
        g(rect.x()),
        g(rect.y()),
        g(rect.width()),
        g(rect.height())
    )
}

// External SkBlendMode value boundary; all native enumerants retain identity.
impl From<BlendMode> for skia::BlendMode {
    fn from(value: BlendMode) -> Self {
        match value {
            BlendMode::kClear => Self::kClear,
            BlendMode::kSrc => Self::kSrc,
            BlendMode::kDst => Self::kDst,
            BlendMode::kSrcOver => Self::kSrcOver,
            BlendMode::kDstOver => Self::kDstOver,
            BlendMode::kSrcIn => Self::kSrcIn,
            BlendMode::kDstIn => Self::kDstIn,
            BlendMode::kSrcOut => Self::kSrcOut,
            BlendMode::kDstOut => Self::kDstOut,
            BlendMode::kSrcATop => Self::kSrcATop,
            BlendMode::kDstATop => Self::kDstATop,
            BlendMode::kXor => Self::kXor,
            BlendMode::kPlus => Self::kPlus,
            BlendMode::kModulate => Self::kModulate,
            BlendMode::kScreen => Self::kScreen,
            BlendMode::kOverlay => Self::kOverlay,
            BlendMode::kDarken => Self::kDarken,
            BlendMode::kLighten => Self::kLighten,
            BlendMode::kColorDodge => Self::kColorDodge,
            BlendMode::kColorBurn => Self::kColorBurn,
            BlendMode::kHardLight => Self::kHardLight,
            BlendMode::kSoftLight => Self::kSoftLight,
            BlendMode::kDifference => Self::kDifference,
            BlendMode::kExclusion => Self::kExclusion,
            BlendMode::kMultiply => Self::kMultiply,
            BlendMode::kHue => Self::kHue,
            BlendMode::kSaturation => Self::kSaturation,
            BlendMode::kColor => Self::kColor,
            BlendMode::kLuminosity => Self::kLuminosity,
        }
    }
}
