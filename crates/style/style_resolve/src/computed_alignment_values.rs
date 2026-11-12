//! Chromium ComputedStyleUtils.cc:913-986,5348-5364; original typed values.
use crate::production_css_value::{self as values, Value};
use foundation::{CSSValueID as V, Member, ScopedCSSNameList, StyleNameScope};
use layoutng_style::style::{
    computed_style_constants::{
        ContentDistributionType as D, ContentPosition as C, ItemPosition as I, ItemPositionType,
        OverflowAlignment as O,
    },
    style_content_alignment_data::StyleContentAlignmentData,
    style_self_alignment_data::StyleSelfAlignmentData,
};
use std::rc::Rc;
use style_model::css_value::CSSValuePayload;
fn ident(id: V) -> Rc<Value> {
    values::identifier(id)
}
fn pair(first: V, second: V) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kValuePairClass(
        values::CSSValuePair {
            first: ident(first),
            second: ident(second),
            drop_identical: true,
        },
    )))
}
fn overflow(value: O) -> V {
    match value {
        O::kDefault => V::kInvalid,
        O::kSafe => V::kSafe,
        O::kUnsafe => V::kUnsafe,
    }
}
fn item(value: I) -> V {
    match value {
        I::kLegacy => V::kLegacy,
        I::kAuto => V::kAuto,
        I::kNormal => V::kNormal,
        I::kStretch => V::kStretch,
        I::kBaseline => V::kBaseline,
        I::kLastBaseline => V::kLastBaseline,
        I::kAnchorCenter => V::kAnchorCenter,
        I::kCenter => V::kCenter,
        I::kStart => V::kStart,
        I::kEnd => V::kEnd,
        I::kSelfStart => V::kSelfStart,
        I::kSelfEnd => V::kSelfEnd,
        I::kFlexStart => V::kFlexStart,
        I::kFlexEnd => V::kFlexEnd,
        I::kLeft => V::kLeft,
        I::kRight => V::kRight,
    }
}
pub fn SelfAlignment(data: &StyleSelfAlignmentData) -> Rc<Value> {
    if data.PositionType() == ItemPositionType::kLegacy {
        debug_assert!(matches!(
            data.GetPosition(),
            I::kLeft | I::kRight | I::kCenter
        ));
        debug_assert_eq!(data.Overflow(), O::kDefault);
        return pair(V::kLegacy, item(data.GetPosition()));
    }
    match data.GetPosition() {
        I::kBaseline => ident(V::kBaseline),
        I::kLastBaseline => pair(V::kLast, V::kBaseline),
        position => {
            let id = if position == I::kLegacy {
                V::kNormal
            } else {
                item(position)
            };
            if position as u32 >= I::kCenter as u32 && data.Overflow() != O::kDefault {
                pair(overflow(data.Overflow()), id)
            } else {
                ident(id)
            }
        }
    }
}
pub fn ContentAlignment(data: &StyleContentAlignmentData) -> Rc<Value> {
    let distribution = match data.Distribution() {
        D::kDefault => V::kInvalid,
        D::kSpaceBetween => V::kSpaceBetween,
        D::kSpaceAround => V::kSpaceAround,
        D::kSpaceEvenly => V::kSpaceEvenly,
        D::kStretch => V::kStretch,
    };
    let mut overflow_id = V::kInvalid;
    let position = match data.GetPosition() {
        C::kNormal => {
            if data.Distribution() == D::kDefault {
                V::kNormal
            } else {
                V::kInvalid
            }
        }
        C::kLastBaseline => V::kLastBaseline,
        position => {
            if (position as u32 >= C::kCenter as u32 || data.Distribution() != D::kDefault)
                && data.Overflow() != O::kDefault
            {
                overflow_id = overflow(data.Overflow());
            }
            match position {
                C::kNormal | C::kLastBaseline => unreachable!(),
                C::kBaseline => V::kBaseline,
                C::kCenter => V::kCenter,
                C::kStart => V::kStart,
                C::kEnd => V::kEnd,
                C::kFlexStart => V::kFlexStart,
                C::kFlexEnd => V::kFlexEnd,
                C::kLeft => V::kLeft,
                C::kRight => V::kRight,
            }
        }
    };
    Rc::new(Value::new(CSSValuePayload::kCSSContentDistributionClass(
        values::CSSContentDistributionValue {
            distribution,
            position,
            overflow: overflow_id,
        },
    )))
}
pub fn Names(names: &ScopedCSSNameList) -> Rc<Value> {
    values::list(
        names
            .GetNames()
            .iter()
            .map(|name| values::scoped_custom_ident(unsafe { &*name.Get() }))
            .collect(),
        values::ListSeparator::Comma,
    )
}
pub fn NameList(names: &Member<ScopedCSSNameList>) -> Rc<Value> {
    match names.GetNonNull() {
        Some(names) => Names(unsafe { names.as_ref() }),
        None => ident(V::kNone),
    }
}
pub fn NameScope(scope: &StyleNameScope) -> Rc<Value> {
    if scope.IsNone() {
        ident(V::kNone)
    } else if scope.IsAll() {
        ident(V::kAll)
    } else {
        assert!(!scope.Names().is_null());
        Names(unsafe { &*scope.Names() })
    }
}
