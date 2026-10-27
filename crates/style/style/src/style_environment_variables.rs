// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/style_environment_variables.h
// cpp: third_party/blink/renderer/core/css/style_environment_variables.cc
// cpp: third_party/blink/renderer/core/css/document_style_environment_variables.h
// cpp: third_party/blink/renderer/core/css/document_style_environment_variables.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   style_environment_variables.h: 187 / 100 / 75 / 25 / 0.
//   style_environment_variables.cc: 314 / 259 / 238 / 21 / 0.
//   document_style_environment_variables.h: 67 / 35 / 15 / 20 / 0.
//   document_style_environment_variables.cc: 135 / 95 / 49 / 46 / 0.
// Effective excludes comments/blanks; braces retained. Production pending 0.
// Base and document observation/invalidation algorithms execute locally.
// style_environment_variables.h mapped: 25,29-32,37-40,48-53,60-63,66-67,70,73-79,87-88,90,95-100,103,106-109,121,124-128,131,134-135,139-140,143,147-148,150,157-161,163,165,168,172,175-176,178-183.
// style_environment_variables.h omitted: 5-6,8-14,16,18,89,111,113-118,152-154,174,185,187.
// style_environment_variables.cc mapped: 17,19,23-51,53-54,58-60,63-67,70-111,114,117,119-123,125-126,128-129,131-132,134-135,137-138,140,143,146,148-155,157-165,167-171,173-175,177-185,187-189,191-194,196-198,200-203,205-213,215-218,220-225,227-231,233-249,251-261,263-264,266,270-273,275-276,278-280,282-284,286-288,290-291,294-297,299-300,303-306,308-312.
// style_environment_variables.cc omitted: 5,7-9,11,13,56,112-113,116,124,127,130,133,136,139,141-142,145,267,314.
// document_style_environment_variables.h mapped: 21-22,26-27,38-40,42,47-48,50,55,61-63.
// document_style_environment_variables.h omitted: 5-6,8-12,14,16-17,23,29-32,52,57,59,65,67.
// document_style_environment_variables.cc mapped: 19-23,28-30,32-35,37-41,43-44,48-50,52-53,55-60,97-98,100-101,103-104,106,111-113,115,124-125,127-129,131-133.
// document_style_environment_variables.cc omitted: 5,7-15,17,24-25,45,62-64,66-74,78-92,94-95,114,126,135.
// Omitted: preprocessing/includes/namespace/access/forward/friend scaffolding,
// GC Trace/default destructor, DCHECK/unreachable exhaustive-enum guards,
// and the document UseCounter-only function/branch. No production logic omitted.
// The source pair exposes geometry values through SetVariable; platform
// rectangle calculations belong to its callers, not this class. The viewport
// segment feature checks are DCHECK only, with no production feature gate.

use crate::style_engine::{StyleEngine, StyleEngineBackend};
use crate::style_rule::StyleRuleDependencies;
use foundation::{AtomicString, String};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UADefinedVariable {
    kSafeAreaInsetTop,
    kSafeAreaInsetLeft,
    kSafeAreaInsetBottom,
    kSafeAreaInsetRight,
    kSafeAreaMaxInsetTop,
    kSafeAreaMaxInsetLeft,
    kSafeAreaMaxInsetBottom,
    kSafeAreaMaxInsetRight,
    kKeyboardInsetTop,
    kKeyboardInsetLeft,
    kKeyboardInsetBottom,
    kKeyboardInsetRight,
    kKeyboardInsetWidth,
    kKeyboardInsetHeight,
    kTitlebarAreaX,
    kTitlebarAreaY,
    kTitlebarAreaWidth,
    kTitlebarAreaHeight,
    kPreferredTextScale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UADefinedTwoDimensionalVariable {
    kViewportSegmentTop,
    kViewportSegmentRight,
    kViewportSegmentBottom,
    kViewportSegmentLeft,
    kViewportSegmentWidth,
    kViewportSegmentHeight,
}

/// External CSSVariableData allocation and the platform's existing general
/// floating-point formatter. The associated variable owner is the same one
/// consumed by actual StyleRule classes; this module adds no variable model.
pub trait StyleEnvironmentVariablesBackend: 'static {
    type RuleDependencies: StyleRuleDependencies<CSSVariableData: 'static> + 'static;
    type FeatureContext: 'static;
    fn CSSVariableDataCreate(
        value: &String,
        is_animation_tainted: bool,
        is_attr_tainted: bool,
        has_references: bool,
    ) -> Rc<VariableData<Self>>;
    /// Exact WTF Format("{:g}", value) conversion, without the px suffix.
    fn FormatFloatGeneral(value: f32) -> String;
}

pub type VariableData<B> =
    <<B as StyleEnvironmentVariablesBackend>::RuleDependencies as StyleRuleDependencies>::CSSVariableData;
type TwoDimensionVariableValues<B> = Vec<Vec<Option<Rc<VariableData<B>>>>>;

// Virtual dispatch used only for the source's document subclass. Neither
// variable storage nor inherited-resolution/invalidation algorithms are hooks.
trait EnvironmentObserver<B: StyleEnvironmentVariablesBackend> {
    fn DidResolve(&self, name: &AtomicString);
    fn DidInvalidate(&self, name: &AtomicString);
    fn GetFeatureContext(&self) -> Option<Rc<B::FeatureContext>>;
}

thread_local! {
    // Exactly the source root singleton. One provider owns the native class;
    // selecting a different provider is an integration error, not a fallback.
    static ROOT_INSTANCE: RefCell<Option<Rc<dyn Any>>> = RefCell::new(None);
}

pub struct StyleEnvironmentVariables<B: StyleEnvironmentVariablesBackend> {
    children_: RefCell<Vec<Rc<Self>>>,
    data_: RefCell<HashMap<AtomicString, Rc<VariableData<B>>>>,
    two_dimension_data_: RefCell<HashMap<AtomicString, TwoDimensionVariableValues<B>>>,
    parent_: RefCell<Option<Rc<Self>>>,
    observer_: Option<Rc<dyn EnvironmentObserver<B>>>,
}

impl<B: StyleEnvironmentVariablesBackend> StyleEnvironmentVariables<B> {
    // cpp: style_environment_variables.cc:58-60
    pub fn new() -> Rc<Self> {
        let instance = Rc::new(Self {
            children_: RefCell::new(Vec::new()),
            data_: RefCell::new(HashMap::new()),
            two_dimension_data_: RefCell::new(HashMap::new()),
            parent_: RefCell::new(None),
            observer_: None,
        });
        instance.SetDefaultEnvironmentVariables();
        instance
    }

    // cpp: style_environment_variables.h:106-109
    pub fn WithParent(parent: Rc<Self>) -> Rc<Self> {
        Self::WithParentAndObserver(parent, None)
    }

    fn WithParentAndObserver(
        parent: Rc<Self>,
        observer: Option<Rc<dyn EnvironmentObserver<B>>>,
    ) -> Rc<Self> {
        let instance = Rc::new(Self {
            children_: RefCell::new(Vec::new()),
            data_: RefCell::new(HashMap::new()),
            two_dimension_data_: RefCell::new(HashMap::new()),
            parent_: RefCell::new(Some(parent.clone())),
            observer_: observer,
        });
        // Preserve both source GC graph edges. Owners must call the source's
        // DetachFromParent lifecycle operation (as StyleEngine::Dispose does)
        // to release these edges; no weak-child liveness change is introduced.
        parent.children_.borrow_mut().push(instance.clone());
        instance
    }

    // cpp: style_environment_variables.cc:63-67
    pub fn GetRootInstance() -> Rc<Self> {
        ROOT_INSTANCE.with(|slot| {
            if let Some(root) = slot.borrow().as_ref() {
                return root.clone().downcast::<Self>().unwrap_or_else(|_| {
                    panic!("StyleEnvironmentVariables root provider mismatch")
                });
            }
            let root = Self::new();
            *slot.borrow_mut() = Some(root.clone());
            root
        })
    }

    // cpp: style_environment_variables.cc:23-54
    fn SetDefaultEnvironmentVariables(&self) {
        use UADefinedVariable::*;
        for variable in [
            kSafeAreaInsetTop,
            kSafeAreaInsetLeft,
            kSafeAreaInsetBottom,
            kSafeAreaInsetRight,
            kSafeAreaMaxInsetTop,
            kSafeAreaMaxInsetLeft,
            kSafeAreaMaxInsetBottom,
            kSafeAreaMaxInsetRight,
            kKeyboardInsetTop,
            kKeyboardInsetLeft,
            kKeyboardInsetBottom,
            kKeyboardInsetRight,
            kKeyboardInsetWidth,
            kKeyboardInsetHeight,
        ] {
            self.SetVariable(variable, &String::from("0px"));
        }
        self.SetVariable(kPreferredTextScale, &String::from("1"));
    }

    // cpp: style_environment_variables.cc:70-117
    pub fn GetVariableName(
        variable: UADefinedVariable,
        _feature_context: Option<&B::FeatureContext>,
    ) -> AtomicString {
        use UADefinedVariable::*;
        AtomicString::from_str(match variable {
            kSafeAreaInsetTop => "safe-area-inset-top",
            kSafeAreaInsetLeft => "safe-area-inset-left",
            kSafeAreaInsetBottom => "safe-area-inset-bottom",
            kSafeAreaInsetRight => "safe-area-inset-right",
            kSafeAreaMaxInsetTop => "safe-area-max-inset-top",
            kSafeAreaMaxInsetLeft => "safe-area-max-inset-left",
            kSafeAreaMaxInsetBottom => "safe-area-max-inset-bottom",
            kSafeAreaMaxInsetRight => "safe-area-max-inset-right",
            kKeyboardInsetTop => "keyboard-inset-top",
            kKeyboardInsetLeft => "keyboard-inset-left",
            kKeyboardInsetBottom => "keyboard-inset-bottom",
            kKeyboardInsetRight => "keyboard-inset-right",
            kKeyboardInsetWidth => "keyboard-inset-width",
            kKeyboardInsetHeight => "keyboard-inset-height",
            kTitlebarAreaX => "titlebar-area-x",
            kTitlebarAreaY => "titlebar-area-y",
            kTitlebarAreaWidth => "titlebar-area-width",
            kTitlebarAreaHeight => "titlebar-area-height",
            kPreferredTextScale => "preferred-text-scale",
        })
    }

    // cpp: style_environment_variables.cc:119-146
    pub fn GetTwoDimensionalVariableName(
        variable: UADefinedTwoDimensionalVariable,
        _feature_context: Option<&B::FeatureContext>,
    ) -> AtomicString {
        use UADefinedTwoDimensionalVariable::*;
        AtomicString::from_str(match variable {
            kViewportSegmentTop => "viewport-segment-top",
            kViewportSegmentRight => "viewport-segment-right",
            kViewportSegmentBottom => "viewport-segment-bottom",
            kViewportSegmentLeft => "viewport-segment-left",
            kViewportSegmentWidth => "viewport-segment-width",
            kViewportSegmentHeight => "viewport-segment-height",
        })
    }

    // cpp: style_environment_variables.cc:148-155
    pub fn SetVariableByName(&self, name: AtomicString, value: &String) {
        let data = B::CSSVariableDataCreate(value, false, false, false);
        self.data_.borrow_mut().insert(name.clone(), data);
        self.InvalidateVariable(&name);
    }

    // cpp: style_environment_variables.cc:157-198
    pub fn SetIndexedVariableByName(
        &self,
        name: AtomicString,
        first_dimension: u32,
        second_dimension: u32,
        value: &String,
    ) {
        let Some(first_size) = first_dimension.checked_add(1) else {
            return;
        };
        let Some(second_size) = second_dimension.checked_add(1) else {
            return;
        };
        let variable_data = B::CSSVariableDataCreate(value, false, false, false);
        {
            let mut grids = self.two_dimension_data_.borrow_mut();
            let values = grids.entry(name.clone()).or_default();
            if first_size as usize > values.len() {
                values.resize_with(first_size as usize, Vec::new);
            }
            let row = &mut values[first_dimension as usize];
            if second_size as usize > row.len() {
                row.resize_with(second_size as usize, || None);
            }
            row[second_dimension as usize] = Some(variable_data);
        }
        self.InvalidateVariable(&name);
    }

    // cpp: style_environment_variables.cc:200-203
    pub fn SetVariable(&self, variable: UADefinedVariable, value: &String) {
        let context = self.GetFeatureContext();
        self.SetVariableByName(Self::GetVariableName(variable, context.as_deref()), value);
    }

    // cpp: style_environment_variables.cc:205-213
    pub fn SetIndexedVariable(
        &self,
        variable: UADefinedTwoDimensionalVariable,
        first_dimension: u32,
        second_dimension: u32,
        value: &String,
        context: Option<&B::FeatureContext>,
    ) {
        self.SetIndexedVariableByName(
            Self::GetTwoDimensionalVariableName(variable, context),
            first_dimension,
            second_dimension,
            value,
        );
    }

    // cpp: style_environment_variables.cc:215-218
    pub fn RemoveVariable(&self, variable: UADefinedVariable) {
        let context = self.GetFeatureContext();
        self.RemoveVariableByName(&Self::GetVariableName(variable, context.as_deref()));
    }

    // cpp: style_environment_variables.cc:220-225
    pub fn RemoveIndexedVariable(
        &self,
        variable: UADefinedTwoDimensionalVariable,
        context: Option<&B::FeatureContext>,
    ) {
        self.RemoveVariableByName(&Self::GetTwoDimensionalVariableName(variable, context));
    }

    // cpp: style_environment_variables.cc:227-231
    pub fn RemoveVariableByName(&self, name: &AtomicString) {
        self.data_.borrow_mut().remove(name);
        self.two_dimension_data_.borrow_mut().remove(name);
        self.InvalidateVariable(name);
    }

    // cpp: style_environment_variables.cc:233-264
    pub fn ResolveVariable(
        &self,
        name: &AtomicString,
        indices: &[u32],
    ) -> Option<Rc<VariableData<B>>> {
        if let Some(observer) = &self.observer_ {
            observer.DidResolve(name);
        }
        match indices {
            [] => {
                if let Some(data) = self.data_.borrow().get(name) {
                    return Some(data.clone());
                }
            }
            [first, second] => {
                if let Some(grid) = self.two_dimension_data_.borrow().get(name) {
                    return grid
                        .get(*first as usize)
                        .and_then(|row| row.get(*second as usize))
                        .cloned()
                        .flatten();
                }
            }
            _ => return None,
        }
        let parent = self.parent_.borrow().clone();
        parent.and_then(|parent| parent.ResolveVariable(name, indices))
    }

    // cpp: style_environment_variables.cc:266-276
    pub fn DetachFromParent(&self) {
        let parent = self
            .parent_
            .borrow()
            .clone()
            .expect("environment variable parent");
        {
            let mut children = parent.children_.borrow_mut();
            if let Some(index) = children
                .iter()
                .position(|child| std::ptr::eq(child.as_ref(), self))
            {
                children.remove(index);
            }
        }
        *self.parent_.borrow_mut() = None;
    }

    // cpp: style_environment_variables.cc:278-280
    pub fn FormatFloatPx(value: f32) -> String {
        let mut result = B::FormatFloatGeneral(value);
        result.push_str("px");
        result
    }

    // cpp: style_environment_variables.cc:282-284
    pub fn FormatPx(value: i32) -> String {
        String::from(format!("{value}px"))
    }

    // cpp: style_environment_variables.cc:286-288
    pub fn GetFeatureContext(&self) -> Option<Rc<B::FeatureContext>> {
        self.observer_
            .as_ref()
            .and_then(|observer| observer.GetFeatureContext())
    }

    // cpp: style_environment_variables.cc:290-297
    pub fn ClearForTesting(&self) {
        self.data_.borrow_mut().clear();
        // The source intentionally retains the two-dimensional map.
        if self.parent_.borrow().is_none() {
            self.SetDefaultEnvironmentVariables();
        }
    }

    // cpp: style_environment_variables.cc:299-306
    fn ParentInvalidatedVariable(&self, name: &AtomicString) {
        if !self.data_.borrow().contains_key(name)
            && !self.two_dimension_data_.borrow().contains_key(name)
        {
            self.InvalidateVariable(name);
        }
    }

    // cpp: style_environment_variables.cc:308-312
    fn InvalidateVariable(&self, name: &AtomicString) {
        if let Some(observer) = &self.observer_ {
            observer.DidInvalidate(name);
        }
        let children = self.children_.borrow().clone();
        for child in children {
            child.ParentInvalidatedVariable(name);
        }
    }
}

/// Required Document/Settings and FontSizeFunctions owner operations. All
/// preferred-scale branches and document observer state execute below.
pub trait DocumentStyleEnvironmentVariablesBackend: StyleEnvironmentVariablesBackend {
    type Document: 'static;
    type Settings;
    type EngineBackend: StyleEngineBackend<Document = Self::Document>;
    fn DocumentStyleEngine(document: &Self::Document) -> Rc<StyleEngine<Self::EngineBackend>>;
    fn DocumentExecutionContext(document: &Self::Document) -> Option<Rc<Self::FeatureContext>>;
    fn DocumentSettings(document: &Self::Document) -> Option<&Self::Settings>;
    fn TextScaleMetaTagPresent(document: &Self::Document) -> bool;
    fn AccessibilityFontScaleFactor(settings: &Self::Settings) -> f64;
    fn DefaultFontSize(settings: &Self::Settings) -> i32;
    fn ScaleAllFontsIfNoMetaTextScaleTag(settings: &Self::Settings) -> bool;
    fn IsAndroid() -> bool;
    fn SnapToClosestFontScaleBucket(factor: f64) -> f64;
}

struct DocumentObserver<B: DocumentStyleEnvironmentVariablesBackend> {
    seen_variables_: RefCell<HashSet<AtomicString>>,
    document_: Rc<B::Document>,
}
impl<B: DocumentStyleEnvironmentVariablesBackend> EnvironmentObserver<B> for DocumentObserver<B> {
    // cpp: document_style_environment_variables.cc:19-30
    fn DidResolve(&self, name: &AtomicString) {
        self.seen_variables_.borrow_mut().insert(name.clone());
    }
    // cpp: document_style_environment_variables.cc:43-53
    fn DidInvalidate(&self, name: &AtomicString) {
        if self.seen_variables_.borrow().contains(name) {
            B::DocumentStyleEngine(&self.document_).EnvironmentVariableChanged();
        }
    }
    // cpp: document_style_environment_variables.cc:32-35
    fn GetFeatureContext(&self) -> Option<Rc<B::FeatureContext>> {
        B::DocumentExecutionContext(&self.document_)
    }
}

pub struct DocumentStyleEnvironmentVariables<B: DocumentStyleEnvironmentVariablesBackend> {
    variables_: Rc<StyleEnvironmentVariables<B>>,
    document_: Rc<B::Document>,
}
impl<B: DocumentStyleEnvironmentVariablesBackend> Deref for DocumentStyleEnvironmentVariables<B> {
    type Target = StyleEnvironmentVariables<B>;
    fn deref(&self) -> &Self::Target {
        &self.variables_
    }
}
impl<B: DocumentStyleEnvironmentVariablesBackend> DocumentStyleEnvironmentVariables<B> {
    // cpp: document_style_environment_variables.cc:55-60
    pub fn new(parent: Rc<StyleEnvironmentVariables<B>>, document: Rc<B::Document>) -> Rc<Self> {
        let observer = Rc::new(DocumentObserver::<B> {
            seen_variables_: RefCell::new(HashSet::new()),
            document_: document.clone(),
        });
        let variables = StyleEnvironmentVariables::WithParentAndObserver(parent, Some(observer));
        let instance = Rc::new(Self {
            variables_: variables,
            document_: document,
        });
        instance.UpdatePreferredTextScaleFromDocument();
        instance
    }

    pub fn AsEnvironmentVariables(&self) -> Rc<StyleEnvironmentVariables<B>> {
        self.variables_.clone()
    }

    // cpp: document_style_environment_variables.cc:19-41
    pub fn ResolveVariableWithMetrics(
        &self,
        name: &AtomicString,
        indices: &[u32],
        _record_metrics: bool,
    ) -> Option<Rc<VariableData<B>>> {
        self.variables_.ResolveVariable(name, indices)
    }

    // cpp: document_style_environment_variables.cc:97-133
    pub fn UpdatePreferredTextScaleFromDocument(&self) {
        let Some(settings) = B::DocumentSettings(&self.document_) else {
            return;
        };
        let scale_factor = if B::TextScaleMetaTagPresent(&self.document_) {
            B::SnapToClosestFontScaleBucket(B::AccessibilityFontScaleFactor(settings))
                * (f64::from(B::DefaultFontSize(settings)) / 16.0)
        } else if B::IsAndroid() && !B::ScaleAllFontsIfNoMetaTextScaleTag(settings) {
            B::SnapToClosestFontScaleBucket(B::AccessibilityFontScaleFactor(settings))
        } else {
            1.0
        };
        self.SetVariable(
            UADefinedVariable::kPreferredTextScale,
            &String::Number(scale_factor),
        );
    }
}
