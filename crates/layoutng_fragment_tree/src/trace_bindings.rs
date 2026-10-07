// cpp: foundation/blink_base/heap/trace_traits.h:20-28
// Bind the fragment package's source-mapped Trace methods to the shared
// layout heap. The algorithms remain in their corresponding source files.
use foundation::{Traceable, Visitor};
use std::ops::Deref;

use crate::block_break_token::BlockBreakToken;
use crate::break_token::BreakToken;
use crate::break_token_algorithm_data::{BreakTokenAlgorithmData, BreakTokenAlgorithmDataVirtual};
use crate::fragment_data::{FragmentData, FragmentDataList, FragmentDataRareData};
use crate::fragment_item::FragmentItem;
use crate::fragment_item::{BoxItem, GeneratedTextItem, LineItem, TextFragmentRareData, TextItem};
use crate::fragment_items::FragmentItems;
use crate::fragment_items_builder::FragmentItemWithOffset;
use crate::inline_break_token::InlineBreakToken;
use crate::inline_break_token::RubyBreakTokenData;
use crate::inline_items_data::InlineItemsData;
use crate::layout_result::LayoutResult;
use crate::layout_result::LayoutResultRareData;
use crate::logical_fragment_link::LogicalFragmentLink;
use crate::logical_line_container::{AnnotationLine, LogicalLineContainer};
use crate::logical_line_item::{LogicalLineItem, LogicalLineItems};
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment::{OofData, PhysicalFragment, PropagatedData};
use crate::physical_fragment_link::PhysicalFragmentLink;
use crate::physical_fragment_rare_data::PhysicalFragmentRareData;
use crate::physical_line_box_fragment::PhysicalLineBoxFragment;

macro_rules! trace_source_method {
    ($($type:ty),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                <$type>::Trace(self, visitor);
            }
        })+
    };
}

trace_source_method!(
    FragmentItems,
    OofData,
    PhysicalFragment,
    PhysicalFragmentLink,
    PhysicalFragmentRareData,
    PropagatedData,
    BreakToken,
    LayoutResult,
    FragmentItem,
    AnnotationLine,
    LogicalLineContainer,
    FragmentData,
    FragmentDataRareData,
    BoxItem,
    GeneratedTextItem,
    LineItem,
    TextFragmentRareData,
    TextItem,
    FragmentItemWithOffset,
    RubyBreakTokenData,
    InlineItemsData,
    LayoutResultRareData,
    LogicalFragmentLink,
    LogicalLineItem,
    LogicalLineItems,
);

// The base virtual Trace body is intentionally empty in the source. Algorithm
// packages supply derived dispatch when they install their break-token data.
// cpp: layoutng_fragment_tree/break_token_algorithm_data.h:62-62
impl Traceable for BreakTokenAlgorithmData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakTokenAlgorithmDataVirtual::Trace(self, visitor);
    }
}

// The derived break tokens use the source base type tag for tracing.
impl Traceable for BlockBreakToken {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakToken::Trace(self.deref(), visitor);
    }
}

impl Traceable for InlineBreakToken {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakToken::Trace(self.deref(), visitor);
    }
}

// C++ traces a derived fragment through the PhysicalFragment type tag and
// dispatches to the derived TraceAfterDispatch implementation.
impl Traceable for PhysicalBoxFragment {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        PhysicalFragment::Trace(self.deref(), visitor);
    }
}

impl Traceable for PhysicalLineBoxFragment {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        PhysicalFragment::Trace(self.deref(), visitor);
    }
}

// The list owns an inline FragmentData as its first element and inherits its
// rare-data GC edge.
// cpp: layoutng_fragment_tree/fragment_data.h:37-37,65-69
impl Traceable for FragmentDataList {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FragmentData::Trace(self.deref(), visitor);
    }
}
