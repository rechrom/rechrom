//! style_property_serializer.cc:109-143,2745-2768,3028-3283.
use super::basic::{AppendValue, IsIdentifier, SerializeBorderValues};
use super::{CSSValueID, CSSValuePayload, Id, Shorthand, String, StylePropertySerializer, Value};
use crate::css_primitive_value::UnitType;
use crate::production_css_value::{self as values, CSSValuePair};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
thread_local! {static INITIALS:RefCell<HashMap<Id,Rc<Value>>>=RefCell::new(HashMap::new());}
/// The actual Longhand::InitialValue constructors needed by the source
/// serializer. No string parsing or computed-style reconstruction occurs.
pub(super) fn InitialSpecified(id: Id) -> Rc<Value> {
    INITIALS.with(|cache| {
        cache
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| match id {
                // longhands_custom.cc:1461-1467,1485-1491,1511-1522,1535-1537,1561-1568.
                Id::kBorderImageSource => values::identifier(CSSValueID::kNone),
                Id::kBorderImageSlice => {
                    let value = values::numeric(100.0, UnitType::kPercentage);
                    let quad =
                        crate::production_border_image_values::quad(std::array::from_fn(|_| {
                            value.clone()
                        }));
                    crate::production_border_image_values::slice(quad, false)
                }
                Id::kBorderImageWidth | Id::kBorderImageOutset => {
                    let value = values::numeric(
                        if id == Id::kBorderImageWidth {
                            1.0
                        } else {
                            0.0
                        },
                        UnitType::kInteger,
                    );
                    crate::production_border_image_values::quad(std::array::from_fn(|_| {
                        value.clone()
                    }))
                }
                Id::kBorderImageRepeat => {
                    let stretch = values::identifier(CSSValueID::kStretch);
                    Rc::new(Value::new(CSSValuePayload::kValuePairClass(CSSValuePair {
                        first: stretch.clone(),
                        second: stretch,
                        drop_identical: true,
                    })))
                }
                // longhands_custom.cc:4218-4220,4350-4352,4950-4952,5039-5041,5149-5151.
                Id::kFlexDirection | Id::kGridAutoFlow => values::identifier(CSSValueID::kRow),
                Id::kFlexWrap => values::identifier(CSSValueID::kNowrap),
                Id::kGridAutoColumns | Id::kGridAutoRows => values::identifier(CSSValueID::kAuto),
                Id::kGridTemplateAreas => values::identifier(CSSValueID::kNone),
                _ => unreachable!(
                    "serializer InitialValue call requires its owning longhand constructor: {id:?}"
                ),
            })
            .clone()
    })
}
pub(super) fn Pair(value: &Value) -> &CSSValuePair {
    let CSSValuePayload::kValuePairClass(pair) = value.Payload() else {
        unreachable!("source typed CSSValuePair precondition");
    };
    pair
}
fn Numeric(value: &Value) -> Option<f64> {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(number) => Some(number.DoubleValue()),
        _ => None,
    }
}
fn SerializeFour(values: [&Value; 4]) -> String {
    let show_left = values[1] != values[3];
    let show_bottom = values[0] != values[2] || show_left;
    let show_right = values[0] != values[1] || show_bottom;
    let mut out = values[0].CssText();
    for (value, show) in [
        (values[1], show_right),
        (values[2], show_bottom),
        (values[3], show_left),
    ] {
        if show {
            AppendValue(&mut out, value, " ");
        }
    }
    out
}
fn SerializeCorner(radius: &Value, shape: &Value) -> String {
    let radius_pair = Pair(radius);
    if Numeric(&radius_pair.first) == Some(0.0) && Numeric(&radius_pair.second) == Some(0.0) {
        if IsIdentifier(shape, CSSValueID::kRound) {
            return String::from("normal");
        }
        let CSSValuePayload::kSuperellipseClass(shape) = shape.Payload() else {
            unreachable!("non-round corner shape is typed superellipse");
        };
        if Numeric(shape.Param())
            == Some(layoutng_style::style::superellipse::Superellipse::Round().Parameter())
        {
            return String::from("normal");
        }
    }
    // CSSValuePair::kKeepIdenticalValues preserves both original values.
    let mut out = radius.CssText();
    out.push(' ');
    out.push_string(&shape.CssText());
    out
}
impl StylePropertySerializer<'_> {
    pub(super) fn GetShorthandValueForDoubleBarCombinator(&self, shorthand: Shorthand) -> String {
        let mut out = String::from("");
        for &id in shorthand.properties {
            let initial = InitialSpecified(id);
            debug_assert!(!initial.IsInitialValue());
            let value = self.Required(id);
            if value == initial.as_ref() {
                continue;
            }
            AppendValue(&mut out, value, " ");
        }
        if out.empty() {
            InitialSpecified(shorthand.properties[0]).CssText()
        } else {
            out
        }
    }
    pub(super) fn BorderPropertyValue(
        &self,
        width: Shorthand,
        style: Shorthand,
        color: Shorthand,
    ) -> String {
        for id in [
            Id::kBorderImageSource,
            Id::kBorderImageSlice,
            Id::kBorderImageWidth,
            Id::kBorderImageOutset,
            Id::kBorderImageRepeat,
        ] {
            let value = self.Get(id);
            let initial = InitialSpecified(id);
            if value.is_some_and(|value| !value.IsInitialValue() && value != initial.as_ref()) {
                return String::new();
            }
        }
        let mut texts = Vec::with_capacity(3);
        for shorthand in [width, style, color] {
            let text = self.GetCommonValue(shorthand);
            if text.IsNull() {
                return String::new();
            }
            texts.push(text);
        }
        SerializeBorderValues(&texts)
    }
    pub(super) fn BorderImagePropertyValue(&self) -> String {
        let shorthand = Shorthand::For(Id::kBorderImage);
        if shorthand.properties[1..]
            .iter()
            .all(|&id| self.Required(id) == InitialSpecified(id).as_ref())
        {
            return self.Required(shorthand.properties[0]).CssText();
        }
        let mut out = String::from("");
        for (index, &id) in shorthand.properties.iter().enumerate() {
            if !out.empty() {
                out.push(' ');
            }
            if index == 2 || index == 3 {
                out.push_str("/ ");
            }
            out.push_string(&self.Required(id).CssText());
        }
        out
    }
    pub(super) fn BorderRadiusValue(&self) -> String {
        let ids = [
            Id::kBorderTopLeftRadius,
            Id::kBorderTopRightRadius,
            Id::kBorderBottomRightRadius,
            Id::kBorderBottomLeftRadius,
        ];
        let pairs = ids.map(|id| Pair(self.Required(id)));
        let mut out = SerializeFour(pairs.map(|pair| pair.first.as_ref()));
        if pairs.iter().any(|pair| pair.first != pair.second) {
            out.push_str(" / ");
            out.push_string(&SerializeFour(pairs.map(|pair| pair.second.as_ref())));
        }
        out
    }
    pub(super) fn CornerShapeValue(&self) -> String {
        SerializeFour(
            [
                Id::kCornerTopLeftShape,
                Id::kCornerTopRightShape,
                Id::kCornerBottomRightShape,
                Id::kCornerBottomLeftShape,
            ]
            .map(|id| self.Required(id)),
        )
    }
    pub(super) fn CornersValue(&self) -> String {
        let radii = [
            Id::kBorderTopLeftRadius,
            Id::kBorderTopRightRadius,
            Id::kBorderBottomRightRadius,
            Id::kBorderBottomLeftRadius,
        ];
        let shapes = [
            Id::kCornerTopLeftShape,
            Id::kCornerTopRightShape,
            Id::kCornerBottomRightShape,
            Id::kCornerBottomLeftShape,
        ];
        let values =
            std::array::from_fn::<_, 4, _>(|i| (self.Required(radii[i]), self.Required(shapes[i])));
        let show_left = values[1] != values[3];
        let show_bottom = show_left || values[0] != values[2];
        let show_right = show_bottom || values[0] != values[1];
        let mut out = SerializeCorner(values[0].0, values[0].1);
        for (value, show) in [
            (values[1], show_right),
            (values[2], show_bottom),
            (values[3], show_left),
        ] {
            if show {
                out.push_str(" / ");
                out.push_string(&SerializeCorner(value.0, value.1));
            }
        }
        out
    }
    pub(super) fn SingleCornerShorthandValue(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 2);
        let (Some(radius), Some(shape)) = (
            self.Get(shorthand.properties[0]),
            self.Get(shorthand.properties[1]),
        ) else {
            return String::new();
        };
        SerializeCorner(radius, shape)
    }
    pub(super) fn CornerPairShorthandValue(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 4);
        let values = shorthand
            .properties
            .iter()
            .map(|&id| self.Get(id))
            .collect::<Option<Vec<_>>>();
        let Some(values) = values else {
            return String::new();
        };
        let first = SerializeCorner(values[0], values[1]);
        let second = SerializeCorner(values[2], values[3]);
        if first == second {
            return first;
        }
        let mut out = first;
        out.push_str(" / ");
        out.push_string(&second);
        out
    }
}
