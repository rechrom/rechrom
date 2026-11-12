//! StylePropertySerializer, Chromium 6c1d401fcca5.
//! The declaration is borrowed in place. The synthetic `all` entries retain
//! references to the original all CSSValue; no property-set copy is produced.
#![allow(non_snake_case)]
use crate::css_value::CSSValuePayload;
use crate::production_css_value::{ProductionCSSValueDispatch as Dispatch, Value};
use foundation::css_property_id::{
    kIntFirstCSSProperty, kIntLastCSSProperty, kNumCSSPropertyIDs, IsCSSPropertyIDWithName,
};
use foundation::{CSSPropertyID as Id, CSSValueID, String};
use std::{collections::HashSet, rc::Rc};
use style_model::{
    css_property_name::CSSPropertyName,
    css_property_value::{CSSPropertyValue, CSSPropertyValueBackend},
    css_property_value_set::CSSPropertyValueSet,
    properties::css_property::CSSProperty,
};
pub(super) type PropertySet = CSSPropertyValueSet<Dispatch>;
pub(super) const ALL_COUNT: usize = (kIntLastCSSProperty - kIntFirstCSSProperty + 1) as usize;
#[derive(Clone, Copy)]
pub(super) struct Shorthand {
    pub id: Id,
    pub properties: &'static [Id],
}
impl Shorthand {
    pub fn For(id: Id) -> Self {
        Self {
            id,
            properties: style_model::property_metadata::ShorthandFor(id),
        }
    }
}
pub(super) struct PropertyValue<'a> {
    pub name: CSSPropertyName,
    pub value: &'a Value,
    pub important: bool,
}
impl<'a> PropertyValue<'a> {
    fn FromProperty(property: &'a CSSPropertyValue<Dispatch>) -> Self {
        Self {
            name: property.Name(),
            value: property.Value(),
            important: property.IsImportant(),
        }
    }
}
pub(super) struct CSSPropertyValueSetForSerializer<'a> {
    pub set: &'a PropertySet,
    all_index: i32,
    used: [bool; kNumCSSPropertyIDs as usize],
    after_all: [bool; kNumCSSPropertyIDs as usize],
    expand_all: bool,
}
impl<'a> CSSPropertyValueSetForSerializer<'a> {
    pub fn new(set: &'a PropertySet) -> Self {
        let all_index = set.FindPropertyIndex(Id::kAll);
        let mut owner = Self {
            set,
            all_index,
            used: [false; kNumCSSPropertyIDs as usize],
            after_all: [false; kNumCSSPropertyIDs as usize],
            expand_all: false,
        };
        if all_index == -1 {
            return owner;
        }
        let all = set.PropertyAt(all_index as usize);
        for (index, property) in set.Properties().iter().enumerate() {
            let id = property.PropertyID();
            if property.IsAffectedByAll() {
                if all.IsImportant() && !property.IsImportant() {
                    continue;
                }
                if all_index as usize >= index {
                    continue;
                }
                if IsCSSPropertyIDWithName(id) {
                    owner.after_all[id as usize] = true;
                }
                if property.Value() == all.Value() && property.IsImportant() == all.IsImportant() {
                    continue;
                }
                owner.expand_all = true;
            }
            if IsCSSPropertyIDWithName(id) {
                owner.used[id as usize] = true;
            }
        }
        owner
    }
    fn HasExpandedAll(&self) -> bool {
        self.all_index != -1 && self.expand_all
    }
    fn InSet(&self, index: usize) -> bool {
        if self.HasExpandedAll() {
            index >= ALL_COUNT
        } else {
            index < self.set.PropertyCount()
        }
    }
    pub fn PropertyCount(&self) -> usize {
        self.set.PropertyCount() + if self.HasExpandedAll() { ALL_COUNT } else { 0 }
    }
    pub fn PropertyAt(&self, index: usize) -> PropertyValue<'a> {
        if !self.HasExpandedAll() {
            return PropertyValue::FromProperty(self.set.PropertyAt(index));
        }
        if self.InSet(index) {
            return PropertyValue::FromProperty(self.set.PropertyAt(index - ALL_COUNT));
        }
        let id = foundation::ConvertToCSSPropertyID(index as i32 + kIntFirstCSSProperty);
        let all = self.set.PropertyAt(self.all_index as usize);
        PropertyValue {
            name: CSSPropertyName::new(id),
            value: all.Value(),
            important: all.IsImportant(),
        }
    }
    pub fn ShouldProcessPropertyAt(&self, index: usize) -> bool {
        if self.all_index == -1 {
            return true;
        }
        if !self.expand_all {
            let property = self.set.PropertyAt(index);
            let id = property.PropertyID();
            return id == Id::kAll
                || !property.IsAffectedByAll()
                || IsCSSPropertyIDWithName(id) && self.used[id as usize];
        }
        if self.InSet(index) {
            let property = self.set.PropertyAt(index - ALL_COUNT);
            let id = property.PropertyID();
            if id == Id::kVariable {
                return true;
            }
            if id == Id::kAll || !IsCSSPropertyIDWithName(id) {
                return false;
            }
            return if property.IsAffectedByAll() {
                self.after_all[id as usize]
            } else {
                self.used[id as usize]
            };
        }
        let id = foundation::ConvertToCSSPropertyID(index as i32 + kIntFirstCSSProperty);
        if CSSProperty::Get(id).IsShorthand() || id == Id::kAll || !Dispatch::IsAffectedByAll(id) {
            return false;
        }
        !self.after_all[id as usize]
    }
    pub fn FindPropertyIndex(&self, id: Id) -> i32 {
        if !self.expand_all {
            if self.all_index != -1 && IsCSSPropertyIDWithName(id) {
                return self.all_index;
            }
            return self.set.FindPropertyIndex(id);
        }
        if IsCSSPropertyIDWithName(id) && self.after_all[id as usize] {
            let index = self.set.FindPropertyIndex(id);
            debug_assert!(index >= 0);
            return ALL_COUNT as i32 + index;
        }
        id as i32 - kIntFirstCSSProperty
    }
    pub fn GetOwner(&self, id: Id) -> Option<Rc<Value>> {
        let index = self.FindPropertyIndex(id);
        if index == -1 {
            return None;
        }
        if self.HasExpandedAll() {
            if self.InSet(index as usize) {
                Some(self.set.PropertyAt(index as usize - ALL_COUNT).ValueRef())
            } else {
                Some(self.set.PropertyAt(self.all_index as usize).ValueRef())
            }
        } else {
            Some(self.set.PropertyAt(index as usize).ValueRef())
        }
    }
    pub fn Get(&self, id: Id) -> Option<&'a Value> {
        let index = self.FindPropertyIndex(id);
        (index != -1).then(|| self.PropertyAt(index as usize).value)
    }
}
pub struct StylePropertySerializer<'a> {
    pub(super) property_set: CSSPropertyValueSetForSerializer<'a>,
}
impl<'a> StylePropertySerializer<'a> {
    pub fn new(set: &'a PropertySet) -> Self {
        Self {
            property_set: CSSPropertyValueSetForSerializer::new(set),
        }
    }
    pub(super) fn Get(&self, id: Id) -> Option<&'a Value> {
        self.property_set.Get(id)
    }
    pub(super) fn RequiredOwner(&self, id: Id) -> Rc<Value> {
        self.property_set
            .GetOwner(id)
            .expect("common checks prove original longhand value owner")
    }
    pub(super) fn Required(&self, id: Id) -> &'a Value {
        self.Get(id)
            .expect("common shorthand checks proved required longhands")
    }
    pub fn AsText(&self) -> String {
        let mut result = String::from("");
        let mut serialized = HashSet::new();
        let mut appeared = HashSet::new();
        let mut count = 0usize;
        for index in 0..self.property_set.PropertyCount() {
            if !self.property_set.ShouldProcessPropertyAt(index) {
                continue;
            }
            let property = self.property_set.PropertyAt(index);
            let id = property.name.Id();
            if id == Id::kVariable {
                if count != 0 {
                    result.push(' ');
                }
                result.push_string(&style_model::css_markup::SerializeIdentifier(
                    &String::from_utf16(
                        property
                            .name
                            .ToAtomicString()
                            .utf16_units()
                            .unwrap_or_default(),
                    ),
                    count != 0,
                ));
                result.push_str(": ");
                result.push_string(&property.value.CssText());
                if property.important {
                    result.push_str(" !important");
                }
                result.push(';');
                count += 1;
                continue;
            }
            if id == Id::kAll {
                Self::AppendPropertyText(
                    &mut result,
                    &property.name,
                    &property.value.CssText(),
                    property.important,
                    count != 0,
                );
                count += 1;
                continue;
            }
            if serialized.contains(&id) {
                continue;
            }
            let mut as_shorthand = false;
            for shorthand_id in style_model::property_metadata::MatchingShorthands(id) {
                let shorthand = Shorthand::For(shorthand_id);
                if shorthand.properties.len() == 1 || !appeared.insert(shorthand_id) {
                    continue;
                }
                if shorthand
                    .properties
                    .iter()
                    .any(|id| serialized.contains(id))
                {
                    continue;
                }
                let text = self.SerializeShorthand(shorthand_id);
                if text.empty() {
                    continue;
                }
                Self::AppendPropertyText(
                    &mut result,
                    &CSSPropertyName::new(shorthand_id),
                    &text,
                    property.important,
                    count != 0,
                );
                count += 1;
                serialized.extend(shorthand.properties.iter().copied());
                as_shorthand = true;
                break;
            }
            if !as_shorthand {
                Self::AppendPropertyText(
                    &mut result,
                    &property.name,
                    &property.value.CssText(),
                    property.important,
                    count != 0,
                );
                count += 1;
            }
        }
        result
    }
    fn AppendPropertyText(
        result: &mut String,
        name: &CSSPropertyName,
        value: &String,
        important: bool,
        not_first: bool,
    ) {
        if not_first {
            result.push(' ');
        }
        result.push_string(&String::from_utf16(
            name.ToAtomicString().utf16_units().unwrap_or_default(),
        ));
        result.push_str(": ");
        result.push_string(value);
        if important {
            result.push_str(" !important");
        }
        result.push(';');
    }
    fn CommonShorthandChecks(&self, shorthand: Shorthand) -> Option<String> {
        assert!(
            !shorthand.properties.is_empty()
                && shorthand.properties.len()
                    <= foundation::css_property_id::kMaxShorthandExpansion
        );
        let mut longhands = Vec::with_capacity(shorthand.properties.len());
        let mut important = false;
        let mut not_important = false;
        for &id in shorthand.properties {
            let index = self.property_set.FindPropertyIndex(id);
            if index == -1 {
                return Some(String::from(""));
            }
            let property = self.property_set.PropertyAt(index as usize);
            important |= property.important;
            not_important |= !property.important;
            longhands.push(property.value);
        }
        if important && not_important {
            return Some(String::from(""));
        }
        let first = longhands[0];
        if (first.IsCSSWideKeyword() || first.IsPendingSubstitutionValue())
            && longhands.iter().all(|value| *value == first)
        {
            if let CSSValuePayload::kPendingSubstitutionValueClass(value) = first.Payload() {
                if value.shorthand != shorthand.id {
                    return Some(String::from(""));
                }
                return Some(value.value.CssText());
            }
            return Some(first.CssText());
        }
        let allow = AllowInitialInShorthand(shorthand.id);
        for value in longhands {
            if !allow && value.IsInitialValue()
                || value.IsCSSWideKeyword() && !value.IsInitialValue()
                || value.IsPendingSubstitutionValue()
                || value.IsUnparsedDeclaration()
            {
                return Some(String::from(""));
            }
        }
        None
    }
}
fn AllowInitialInShorthand(id: Id) -> bool {
    matches!(
        id,
        Id::kBackground
            | Id::kBorder
            | Id::kBorderTop
            | Id::kBorderRight
            | Id::kBorderBottom
            | Id::kBorderLeft
            | Id::kBorderBlockStart
            | Id::kBorderBlockEnd
            | Id::kBorderInlineStart
            | Id::kBorderInlineEnd
            | Id::kBorderBlock
            | Id::kBorderInline
            | Id::kOutline
            | Id::kColumnRule
            | Id::kColumns
            | Id::kGridColumn
            | Id::kGridRow
            | Id::kGridArea
            | Id::kGap
            | Id::kListStyle
            | Id::kTextDecoration
            | Id::kTextEmphasis
            | Id::kTextWrap
            | Id::kMask
            | Id::kWebkitTextStroke
            | Id::kWhiteSpace
    )
}

#[derive(Clone, Copy)]
pub(super) enum GapDirection {
    Row,
    Column,
}

impl StylePropertySerializer<'_> {
    pub fn SerializeShorthand(&self, id: Id) -> String {
        let shorthand = Shorthand::For(id);
        if let Some(result) = self.CommonShorthandChecks(shorthand) {
            return result;
        }
        match id {
            Id::kAnimation => self.GetLayeredShorthandValue(Shorthand::For(Id::kAnimation)),
            Id::kAnimationRange => self.AnimationRangeShorthandValue(),
            Id::kBorderSpacing => self.Get2Values(Shorthand::For(Id::kBorderSpacing)),
            Id::kBackgroundPosition => {
                self.GetLayeredShorthandValue(Shorthand::For(Id::kBackgroundPosition))
            }
            Id::kBackground => self.GetLayeredShorthandValue(Shorthand::For(Id::kBackground)),
            Id::kBorder => self.BorderPropertyValue(
                Shorthand::For(Id::kBorderWidth),
                Shorthand::For(Id::kBorderStyle),
                Shorthand::For(Id::kBorderColor),
            ),
            Id::kBorderImage => self.BorderImagePropertyValue(),
            Id::kBorderTop => self.GetShorthandValueForBorder(Shorthand::For(Id::kBorderTop)),
            Id::kBorderRight => self.GetShorthandValueForBorder(Shorthand::For(Id::kBorderRight)),
            Id::kBorderBottom => self.GetShorthandValueForBorder(Shorthand::For(Id::kBorderBottom)),
            Id::kBorderLeft => self.GetShorthandValueForBorder(Shorthand::For(Id::kBorderLeft)),
            Id::kBorderBlock => self.BorderPropertyValue(
                Shorthand::For(Id::kBorderBlockWidth),
                Shorthand::For(Id::kBorderBlockStyle),
                Shorthand::For(Id::kBorderBlockColor),
            ),
            Id::kBorderBlockColor => self.Get2Values(Shorthand::For(Id::kBorderBlockColor)),
            Id::kBorderBlockStyle => self.Get2Values(Shorthand::For(Id::kBorderBlockStyle)),
            Id::kBorderBlockWidth => self.Get2Values(Shorthand::For(Id::kBorderBlockWidth)),
            Id::kBorderBlockStart => self.GetShorthandValue(Shorthand::For(Id::kBorderBlockStart)),
            Id::kBorderBlockEnd => self.GetShorthandValue(Shorthand::For(Id::kBorderBlockEnd)),
            Id::kBorderInline => self.BorderPropertyValue(
                Shorthand::For(Id::kBorderInlineWidth),
                Shorthand::For(Id::kBorderInlineStyle),
                Shorthand::For(Id::kBorderInlineColor),
            ),
            Id::kBorderInlineColor => self.Get2Values(Shorthand::For(Id::kBorderInlineColor)),
            Id::kBorderInlineStyle => self.Get2Values(Shorthand::For(Id::kBorderInlineStyle)),
            Id::kBorderInlineWidth => self.Get2Values(Shorthand::For(Id::kBorderInlineWidth)),
            Id::kBorderInlineStart => {
                self.GetShorthandValue(Shorthand::For(Id::kBorderInlineStart))
            }
            Id::kBorderInlineEnd => self.GetShorthandValue(Shorthand::For(Id::kBorderInlineEnd)),
            Id::kContainer => self.ContainerValue(),
            Id::kOutline => self.GetShorthandValue(Shorthand::For(Id::kOutline)),
            Id::kBorderColor => self.Get4Values(Shorthand::For(Id::kBorderColor)),
            Id::kBorderWidth => self.Get4Values(Shorthand::For(Id::kBorderWidth)),
            Id::kBorderStyle => self.Get4Values(Shorthand::For(Id::kBorderStyle)),
            Id::kColumnRule => self.GetShorthandValueForGapDecorationsRule(
                Shorthand::For(Id::kColumnRule),
                GapDirection::Column,
            ),
            Id::kRowRule => self.GetShorthandValueForGapDecorationsRule(
                Shorthand::For(Id::kRowRule),
                GapDirection::Row,
            ),
            Id::kColumnRuleInsetCap => self.GetShorthandValueForGapDecorationsRuleInsetCapJunction(
                Shorthand::For(Id::kColumnRuleInsetCap),
                GapDirection::Column,
                true,
            ),
            Id::kRowRuleInsetCap => self.GetShorthandValueForGapDecorationsRuleInsetCapJunction(
                Shorthand::For(Id::kRowRuleInsetCap),
                GapDirection::Row,
                true,
            ),
            Id::kColumnRuleInset => self.GetShorthandValueForGapDecorationsRuleInset(
                Shorthand::For(Id::kColumnRuleInset),
                GapDirection::Column,
            ),
            Id::kRowRuleInset => self.GetShorthandValueForGapDecorationsRuleInset(
                Shorthand::For(Id::kRowRuleInset),
                GapDirection::Row,
            ),
            Id::kColumnRuleInsetEnd => self.GetShorthandValueForGapDecorationsRuleInsetStartEnd(
                Shorthand::For(Id::kColumnRuleInsetEnd),
            ),
            Id::kColumnRuleInsetStart => self.GetShorthandValueForGapDecorationsRuleInsetStartEnd(
                Shorthand::For(Id::kColumnRuleInsetStart),
            ),
            Id::kRowRuleInsetEnd => self.GetShorthandValueForGapDecorationsRuleInsetStartEnd(
                Shorthand::For(Id::kRowRuleInsetEnd),
            ),
            Id::kRowRuleInsetStart => self.GetShorthandValueForGapDecorationsRuleInsetStartEnd(
                Shorthand::For(Id::kRowRuleInsetStart),
            ),
            Id::kColumnRuleInsetJunction => self
                .GetShorthandValueForGapDecorationsRuleInsetCapJunction(
                    Shorthand::For(Id::kColumnRuleInsetJunction),
                    GapDirection::Column,
                    false,
                ),
            Id::kRowRuleInsetJunction => self
                .GetShorthandValueForGapDecorationsRuleInsetCapJunction(
                    Shorthand::For(Id::kRowRuleInsetJunction),
                    GapDirection::Row,
                    false,
                ),
            Id::kColumns => self.GetShorthandValueForColumns(Shorthand::For(Id::kColumns)),
            Id::kContainIntrinsicSize => self.ContainIntrinsicSizeValue(),
            Id::kFlex => self.GetShorthandValue(Shorthand::For(Id::kFlex)),
            Id::kFlexFlow => {
                self.GetShorthandValueForDoubleBarCombinator(Shorthand::For(Id::kFlexFlow))
            }
            Id::kGap => self.Get2Values(Shorthand::For(Id::kGap)),
            Id::kGrid => self.GetShorthandValueForGrid(Shorthand::For(Id::kGrid)),
            Id::kGridArea => self.GetShorthandValueForGridArea(Shorthand::For(Id::kGridArea)),
            Id::kGridColumn => self.GetShorthandValueForGridLine(Shorthand::For(Id::kGridColumn)),
            Id::kGridLanes => self.GetShorthandValueForGridLanes(Shorthand::For(Id::kGridLanes)),
            Id::kGridRow => self.GetShorthandValueForGridLine(Shorthand::For(Id::kGridRow)),
            Id::kGridTemplate => {
                self.GetShorthandValueForGridTemplate(Shorthand::For(Id::kGridTemplate))
            }
            Id::kInset => self.Get4Values(Shorthand::For(Id::kInset)),
            Id::kInterestDelay => self.Get2Values(Shorthand::For(Id::kInterestDelay)),
            Id::kInsetBlock => self.Get2Values(Shorthand::For(Id::kInsetBlock)),
            Id::kInsetInline => self.Get2Values(Shorthand::For(Id::kInsetInline)),
            Id::kAlternativeLineClampShorthand => self.LineClampValue(false),
            Id::kAlternativeWebkitLineClampShorthand => self.LineClampValue(true),
            Id::kPlaceContent => self.Get2Values(Shorthand::For(Id::kPlaceContent)),
            Id::kPlaceItems => self.Get2Values(Shorthand::For(Id::kPlaceItems)),
            Id::kPlaceSelf => self.Get2Values(Shorthand::For(Id::kPlaceSelf)),
            Id::kFont => self.FontValue(),
            Id::kFontSynthesis => self.FontSynthesisValue(),
            Id::kFontVariant => self.FontVariantValue(),
            Id::kMargin => self.Get4Values(Shorthand::For(Id::kMargin)),
            Id::kMarginBlock => self.Get2Values(Shorthand::For(Id::kMarginBlock)),
            Id::kMarginInline => self.Get2Values(Shorthand::For(Id::kMarginInline)),
            Id::kOffset => self.OffsetValue(),
            Id::kOverflow => self.Get2Values(Shorthand::For(Id::kOverflow)),
            Id::kOverscrollBehavior => self.Get2Values(Shorthand::For(Id::kOverscrollBehavior)),
            Id::kPadding => self.Get4Values(Shorthand::For(Id::kPadding)),
            Id::kPaddingBlock => self.Get2Values(Shorthand::For(Id::kPaddingBlock)),
            Id::kPaddingInline => self.Get2Values(Shorthand::For(Id::kPaddingInline)),
            Id::kTextDecoration => self.TextDecorationValue(),
            Id::kTransition => self.GetLayeredShorthandValue(Shorthand::For(Id::kTransition)),
            Id::kListStyle => self.GetShorthandValue(Shorthand::For(Id::kListStyle)),
            Id::kMaskPosition => self.GetLayeredShorthandValue(Shorthand::For(Id::kMaskPosition)),
            Id::kMask => self.GetLayeredShorthandValue(Shorthand::For(Id::kMask)),
            Id::kRule => self.GetShorthandValueForRule(
                Shorthand::For(Id::kRowRule),
                Shorthand::For(Id::kColumnRule),
            ),
            Id::kRuleBreak => {
                self.GetShorthandValueForBidirectionalGapRules(Shorthand::For(Id::kRuleBreak))
            }
            Id::kRuleColor => {
                self.GetShorthandValueForBidirectionalGapRules(Shorthand::For(Id::kRuleColor))
            }
            Id::kRuleInsetCap => self.GetShorthandValueForBidirectionalGapRuleInsetCapJunction(
                Shorthand::For(Id::kRuleInsetCap),
            ),
            Id::kRuleInsetJunction => self
                .GetShorthandValueForBidirectionalGapRuleInsetCapJunction(Shorthand::For(
                    Id::kRuleInsetJunction,
                )),
            Id::kRuleInset => {
                self.GetShorthandValueForBidirectionalGapRuleInset(Shorthand::For(Id::kRuleInset))
            }
            Id::kRuleInsetEnd => self.GetShorthandValueForBidirectionalGapRuleInsetStartEnd(
                Shorthand::For(Id::kRuleInsetEnd),
            ),
            Id::kRuleInsetStart => self.GetShorthandValueForBidirectionalGapRuleInsetStartEnd(
                Shorthand::For(Id::kRuleInsetStart),
            ),
            Id::kRuleStyle => {
                self.GetShorthandValueForBidirectionalGapRules(Shorthand::For(Id::kRuleStyle))
            }
            Id::kRuleVisibilityItems => self.GetShorthandValueForBidirectionalGapRules(
                Shorthand::For(Id::kRuleVisibilityItems),
            ),
            Id::kRuleWidth => {
                self.GetShorthandValueForBidirectionalGapRules(Shorthand::For(Id::kRuleWidth))
            }
            Id::kTextBox => self.TextBoxValue(),
            Id::kTextEmphasis => self.GetShorthandValue(Shorthand::For(Id::kTextEmphasis)),
            Id::kTextSpacing => self.TextSpacingValue(),
            Id::kTimelineTrigger => {
                self.GetLayeredShorthandValue(Shorthand::For(Id::kTimelineTrigger))
            }
            Id::kTimelineTriggerActivationRange => {
                self.TimelineTriggerActivationRangeShorthandValue()
            }
            Id::kTimelineTriggerActiveRange => self.TimelineTriggerExitRangeShorthandValue(),
            Id::kWebkitTextStroke => self.GetShorthandValue(Shorthand::For(Id::kWebkitTextStroke)),
            Id::kTextWrap => self.TextWrapValue(),
            Id::kMarker => {
                if let (Some(start), Some(mid), Some(end)) = (
                    self.Get(Id::kMarkerStart),
                    self.Get(Id::kMarkerMid),
                    self.Get(Id::kMarkerEnd),
                ) {
                    if start == mid && start == end {
                        return start.CssText();
                    }
                }
                String::new()
            }
            Id::kBorderRadius => self.BorderRadiusValue(),
            Id::kCorner => self.CornersValue(),
            Id::kCornerTopLeft => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerTopLeft))
            }
            Id::kCornerTopRight => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerTopRight))
            }
            Id::kCornerBottomLeft => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerBottomLeft))
            }
            Id::kCornerBottomRight => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerBottomRight))
            }
            Id::kCornerStartStart => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerStartStart))
            }
            Id::kCornerStartEnd => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerStartEnd))
            }
            Id::kCornerEndStart => {
                self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerEndStart))
            }
            Id::kCornerEndEnd => self.SingleCornerShorthandValue(Shorthand::For(Id::kCornerEndEnd)),
            Id::kCornerTop => self.CornerPairShorthandValue(Shorthand::For(Id::kCornerTop)),
            Id::kCornerRight => self.CornerPairShorthandValue(Shorthand::For(Id::kCornerRight)),
            Id::kCornerBottom => self.CornerPairShorthandValue(Shorthand::For(Id::kCornerBottom)),
            Id::kCornerLeft => self.CornerPairShorthandValue(Shorthand::For(Id::kCornerLeft)),
            Id::kCornerBlockStart => {
                self.CornerPairShorthandValue(Shorthand::For(Id::kCornerBlockStart))
            }
            Id::kCornerBlockEnd => {
                self.CornerPairShorthandValue(Shorthand::For(Id::kCornerBlockEnd))
            }
            Id::kCornerInlineStart => {
                self.CornerPairShorthandValue(Shorthand::For(Id::kCornerInlineStart))
            }
            Id::kCornerInlineEnd => {
                self.CornerPairShorthandValue(Shorthand::For(Id::kCornerInlineEnd))
            }
            Id::kCornerShape => self.CornerShapeValue(),
            Id::kCornerTopShape => self.Get2Values(Shorthand::For(Id::kCornerTopShape)),
            Id::kCornerRightShape => self.Get2Values(Shorthand::For(Id::kCornerRightShape)),
            Id::kCornerBottomShape => self.Get2Values(Shorthand::For(Id::kCornerBottomShape)),
            Id::kCornerLeftShape => self.Get2Values(Shorthand::For(Id::kCornerLeftShape)),
            Id::kCornerBlockStartShape => {
                self.Get2Values(Shorthand::For(Id::kCornerBlockStartShape))
            }
            Id::kCornerBlockEndShape => self.Get2Values(Shorthand::For(Id::kCornerBlockEndShape)),
            Id::kCornerInlineStartShape => {
                self.Get2Values(Shorthand::For(Id::kCornerInlineStartShape))
            }
            Id::kCornerInlineEndShape => self.Get2Values(Shorthand::For(Id::kCornerInlineEndShape)),
            Id::kScrollPadding => self.Get4Values(Shorthand::For(Id::kScrollPadding)),
            Id::kScrollPaddingBlock => self.Get2Values(Shorthand::For(Id::kScrollPaddingBlock)),
            Id::kScrollPaddingInline => self.Get2Values(Shorthand::For(Id::kScrollPaddingInline)),
            Id::kScrollMargin => self.Get4Values(Shorthand::For(Id::kScrollMargin)),
            Id::kScrollMarginBlock => self.Get2Values(Shorthand::For(Id::kScrollMarginBlock)),
            Id::kScrollMarginInline => self.Get2Values(Shorthand::For(Id::kScrollMarginInline)),
            Id::kScrollTimeline => self.ScrollTimelineValue(),
            Id::kPageBreakAfter => self.PageBreakPropertyValue(Shorthand::For(Id::kPageBreakAfter)),
            Id::kPageBreakBefore => {
                self.PageBreakPropertyValue(Shorthand::For(Id::kPageBreakBefore))
            }
            Id::kPageBreakInside => {
                self.PageBreakPropertyValue(Shorthand::For(Id::kPageBreakInside))
            }
            Id::kViewTimeline => self.ViewTimelineValue(),
            Id::kWhiteSpace => self.WhiteSpaceValue(),
            Id::kWebkitColumnBreakAfter
            | Id::kWebkitColumnBreakBefore
            | Id::kWebkitColumnBreakInside
            | Id::kWebkitMaskBoxImage => String::new(),
            Id::kPositionTry => self.PositionTryValue(Shorthand::For(Id::kPositionTry)),
            _ => {
                unreachable!("source StylePropertySerializer missing a supported shorthand: {id:?}")
            }
        }
    }
}

#[path = "style_property_serializer_basic.rs"]
mod basic;
#[path = "style_property_serializer_border.rs"]
mod border;
#[path = "style_property_serializer_font.rs"]
mod font;
#[path = "style_property_serializer_gap.rs"]
mod gap;
#[path = "style_property_serializer_grid.rs"]
mod grid;
#[path = "style_property_serializer_layered.rs"]
mod layered;
