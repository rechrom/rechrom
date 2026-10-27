#![allow(non_snake_case, non_camel_case_types)]

use std::cell::Cell;

use foundation::css_property_id::CSSPropertyID;
use foundation::style_values::css::anchor_query::AnchorQuery;
use foundation::{LayoutUnit, PhysicalOffset, Visitor, WritingDirectionMode};
use layoutng_style::style::computed_style::ComputedStyleBuilder;
use layoutng_style::style::default_anchor_data::DefaultAnchorData;
use layoutng_style::style::position_area::PositionAreaOffsets;

// cpp: layoutng_out_of_flow/anchor_evaluator.h:55-67
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnchorEvaluatorMode {
    #[default]
    kNone,
    kLeft,
    kRight,
    kTop,
    kBottom,
    kWidth,
    kHeight,
}

// The C++ base class stores mode_ and supplies virtual methods. Rust keeps
// that state in an interior-mutable cell so AnchorScope can restore it while
// callers retain the evaluator reference used for virtual dispatch.
// cpp: layoutng_out_of_flow/anchor_evaluator.h:26-30
// cpp: layoutng_out_of_flow/anchor_evaluator.h:69-101
pub trait AnchorEvaluator {
    fn Evaluate(
        &mut self,
        query: &AnchorQuery,
        default_anchor_data: &DefaultAnchorData,
        position_area_offsets: &Option<PositionAreaOffsets>,
    ) -> Option<LayoutUnit>;

    fn ComputePositionAreaOffsetsForLayout(
        &mut self,
        default_anchor_data: &DefaultAnchorData,
    ) -> Option<PositionAreaOffsets>;

    fn ComputeAnchorCenterOffsets(
        &mut self,
        builder: &ComputedStyleBuilder,
    ) -> Option<PhysicalOffset>;

    fn GetContainerWritingDirection(&self) -> WritingDirectionMode;

    // cpp: layoutng_out_of_flow/anchor_evaluator.h:91-91
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}

    fn ModeCell(&self) -> &Cell<AnchorEvaluatorMode>;

    // cpp: layoutng_out_of_flow/anchor_evaluator.h:93-100
    fn GetMode(&self) -> AnchorEvaluatorMode {
        self.ModeCell().get()
    }
}

// cpp: layoutng_out_of_flow/anchor_evaluator.h:103-160
pub struct AnchorScope<'a, T: AnchorEvaluator + ?Sized> {
    target_: Option<&'a mut T>,
    original_: AnchorEvaluatorMode,
}

impl<'a, T: AnchorEvaluator + ?Sized> AnchorScope<'a, T> {
    // cpp: layoutng_out_of_flow/anchor_evaluator.h:115-121
    pub fn new(mode: AnchorEvaluatorMode, mut anchor_evaluator: Option<&'a mut T>) -> Self {
        let original_ = anchor_evaluator
            .as_ref()
            .map_or(AnchorEvaluatorMode::kNone, |evaluator| evaluator.GetMode());
        if let Some(evaluator) = anchor_evaluator.as_mut() {
            evaluator.ModeCell().set(mode);
        }
        Self {
            target_: anchor_evaluator,
            original_,
        }
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator.h:122-123
    pub fn FromProperty(property: CSSPropertyID, anchor_evaluator: Option<&'a mut T>) -> Self {
        Self::new(Self::PropertyMode(property), anchor_evaluator)
    }

    // The scope owns the mutable borrow while active, and restores the mode
    // when it is dropped after the query.
    pub fn Evaluator(&mut self) -> Option<&mut T> {
        self.target_.as_deref_mut()
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator.h:130-156
    fn PropertyMode(property: CSSPropertyID) -> AnchorEvaluatorMode {
        match property {
            CSSPropertyID::kTop => AnchorEvaluatorMode::kTop,
            CSSPropertyID::kRight => AnchorEvaluatorMode::kRight,
            CSSPropertyID::kBottom => AnchorEvaluatorMode::kBottom,
            CSSPropertyID::kLeft => AnchorEvaluatorMode::kLeft,
            CSSPropertyID::kWidth
            | CSSPropertyID::kMinWidth
            | CSSPropertyID::kMaxWidth
            | CSSPropertyID::kMarginLeft
            | CSSPropertyID::kMarginRight => AnchorEvaluatorMode::kWidth,
            CSSPropertyID::kHeight
            | CSSPropertyID::kMinHeight
            | CSSPropertyID::kMaxHeight
            | CSSPropertyID::kMarginTop
            | CSSPropertyID::kMarginBottom => AnchorEvaluatorMode::kHeight,
            _ => AnchorEvaluatorMode::kNone,
        }
    }
}

// cpp: layoutng_out_of_flow/anchor_evaluator.h:124-128
impl<T: AnchorEvaluator + ?Sized> Drop for AnchorScope<'_, T> {
    fn drop(&mut self) {
        if let Some(evaluator) = self.target_.as_mut() {
            evaluator.ModeCell().set(self.original_);
        }
    }
}
