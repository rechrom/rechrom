// C++: layoutng_inline/paragraph_line_breaker.h/.cc.
#![allow(non_snake_case)]

use font_engine::fonts::simple_font_data::SimpleFontData;
use foundation::{LayoutUnit, RuntimeEnabledFeatures, String, WtfSizeT};
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::exclusions::line_layout_opportunity::LineLayoutOpportunity;
use layoutng::internal::inline_node::InlineNode;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;

use crate::inline_layout_algorithm::InlineLayoutAlgorithm;
use crate::leading_floats::LeadingFloats;
use crate::line_breaker::{LineBreaker, LineBreakerMode};
use crate::line_info::LineInfo;
use crate::score_line_break_context::kMaxLinesForBalance;

// cpp: layoutng_inline/paragraph_line_breaker.cc:22-24
struct LineBreakResult {
    width: LayoutUnit,
}

// cpp: layoutng_inline/paragraph_line_breaker.cc:26-118
struct LineBreakResults<'a> {
    node_: InlineNode,
    space_: &'a ConstraintSpace,
    lines_: Vec<LineBreakResult>,
    break_token_: *const InlineBreakToken,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// cpp: layoutng_inline/paragraph_line_breaker.cc:45-49
enum Status {
    Finished,
    NotApplicable,
    MaxLinesExceeded,
}

impl<'a> LineBreakResults<'a> {
    // cpp: layoutng_inline/paragraph_line_breaker.cc:30-31
    fn new(node: &InlineNode, space: &'a ConstraintSpace) -> Self {
        Self {
            node_: node.clone(),
            space_: space,
            lines_: Vec::with_capacity(kMaxLinesForBalance as usize),
            break_token_: std::ptr::null(),
        }
    }

    // cpp: layoutng_inline/paragraph_line_breaker.cc:33-42
    fn Size(&self) -> WtfSizeT {
        self.lines_.len() as WtfSizeT
    }

    fn LineWidthSum(&self) -> LayoutUnit {
        self.lines_
            .iter()
            .fold(LayoutUnit::default(), |sum, item| sum + item.width)
    }

    fn BreakToken(&self) -> *const InlineBreakToken {
        self.break_token_
    }

    fn Clear(&mut self) {
        self.break_token_ = std::ptr::null();
        self.lines_.clear();
    }

    // cpp: layoutng_inline/paragraph_line_breaker.cc:51-85
    fn BreakLines(
        &mut self,
        available_width: LayoutUnit,
        mut max_lines: WtfSizeT,
        line_clamp_ellipsis_width: LayoutUnit,
        stop_at: *const InlineBreakToken,
    ) -> Status {
        debug_assert!(self.lines_.is_empty());
        let line_opportunity = LineLayoutOpportunity::with_inline_size(available_width);
        let leading_floats = LeadingFloats::default();
        let mut exclusion_space = ExclusionSpace::default();
        let mut line_info = LineInfo::default();
        loop {
            let mut line_breaker = LineBreaker::new(
                self.node_.clone(),
                LineBreakerMode::kContent,
                self.space_,
                &line_opportunity,
                &leading_floats,
                self.break_token_,
                std::ptr::null(),
                &mut exclusion_space,
            );
            if max_lines == 1 && line_clamp_ellipsis_width != LayoutUnit::default() {
                line_breaker.SetLineClampEllipsisWidth(line_clamp_ellipsis_width);
            }
            line_breaker.NextLine(&mut line_info);
            debug_assert!(!line_info.HasForcedBreak());
            if line_breaker.ShouldDisableBisectLineBreak() {
                return Status::NotApplicable;
            }
            self.break_token_ = line_info.GetBreakToken();
            self.lines_.push(LineBreakResult {
                width: line_info.Width(),
            });
            debug_assert!(self.Size() <= kMaxLinesForBalance);
            if self.break_token_.is_null()
                || (!stop_at.is_null()
                    && unsafe { &*self.break_token_ }
                        .Start()
                        .GreaterOrEqual(unsafe { &*stop_at }.Start()))
            {
                return Status::Finished;
            }
            max_lines -= 1;
            if max_lines == 0 {
                return Status::MaxLinesExceeded;
            }
        }
    }

    // cpp: layoutng_inline/paragraph_line_breaker.cc:87-110
    fn BisectAvailableWidth(
        &mut self,
        max_available_width: LayoutUnit,
        min_available_width: LayoutUnit,
        epsilon: LayoutUnit,
        line_clamp_ellipsis_width: LayoutUnit,
        num_lines: WtfSizeT,
        stop_at: *const InlineBreakToken,
    ) -> LayoutUnit {
        debug_assert!(epsilon > LayoutUnit::default());
        debug_assert!(num_lines > 0);
        debug_assert_eq!(self.Size(), 0);
        let mut upper = max_available_width;
        let mut lower = min_available_width;
        while lower + epsilon < upper {
            let middle = (upper + lower) / 2;
            let status = self.BreakLines(middle, num_lines, line_clamp_ellipsis_width, stop_at);
            if status != Status::Finished {
                lower = middle;
            } else {
                debug_assert!(self.Size() <= num_lines);
                upper = middle;
            }
            self.Clear();
        }
        debug_assert!(upper >= min_available_width);
        debug_assert!(upper <= max_available_width);
        upper
    }
}

// cpp: layoutng_inline/paragraph_line_breaker.cc:120-136
fn EstimateNumLines(
    text_content: &String,
    font: *const SimpleFontData,
    available_width: LayoutUnit,
) -> WtfSizeT {
    let space_width = unsafe { &*font }.SpaceWidth();
    if space_width <= 0.0 {
        return 0;
    }
    let num_line_chars = (available_width / space_width) as WtfSizeT;
    if num_line_chars == 0 {
        return WtfSizeT::MAX;
    }
    (text_content.length() + num_line_chars - 1) / num_line_chars
}

// cpp: layoutng_inline/paragraph_line_breaker.h:20-28
pub struct ParagraphLineBreaker;

impl ParagraphLineBreaker {
    // cpp: layoutng_inline/paragraph_line_breaker.cc:140-232
    pub fn AttemptParagraphBalancing(
        node: &InlineNode,
        space: &ConstraintSpace,
        line_opportunity: &LineLayoutOpportunity,
    ) -> Option<LayoutUnit> {
        if node.IsBisectLineBreakDisabled() {
            return None;
        }

        let block_style = node.Style();
        let available_width = line_opportunity.AvailableInlineSize();
        let mut line_clamp_ellipsis_width = LayoutUnit::default();
        let mut normal_lines = LineBreakResults::new(node, space);
        let max_lines = kMaxLinesForBalance;
        let lines_until_clamp = space.GetLineClampData().LinesUntilClamp(false).unwrap_or(0);
        if lines_until_clamp > 0 && (lines_until_clamp as WtfSizeT) <= max_lines {
            if lines_until_clamp == 1 {
                return None;
            }

            if RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled() {
                let ellipsis = InlineLayoutAlgorithm::ShapeLineClampEllipsis(node);
                line_clamp_ellipsis_width = unsafe { &*ellipsis.shape_result }.SnappedWidth();
            }

            let status = normal_lines.BreakLines(
                available_width,
                lines_until_clamp as WtfSizeT,
                line_clamp_ellipsis_width,
                std::ptr::null(),
            );
            if status == Status::NotApplicable {
                return None;
            }

            debug_assert!(normal_lines.Size() <= lines_until_clamp as WtfSizeT);
            if normal_lines.Size() != lines_until_clamp as WtfSizeT {
                debug_assert_eq!(status, Status::Finished);
                line_clamp_ellipsis_width = LayoutUnit::default();
            }
        } else {
            let items_data = node.ItemsData(false);
            let font = unsafe { &*block_style.GetFont() }.PrimaryFont();
            let estimated_num_lines = EstimateNumLines(
                &items_data.text_content,
                font,
                line_opportunity.AvailableInlineSize(),
            );
            if estimated_num_lines > max_lines * 2 {
                return None;
            }

            let status = normal_lines.BreakLines(
                available_width,
                max_lines,
                LayoutUnit::default(),
                std::ptr::null(),
            );
            if status != Status::Finished {
                return None;
            }
            debug_assert!(normal_lines.BreakToken().is_null());
        }
        let num_lines = normal_lines.Size();
        debug_assert!(num_lines <= max_lines);
        if num_lines <= 1 {
            return None;
        }

        let epsilon = LayoutUnit::FromFloatCeil(block_style.EffectiveZoom());
        let mut balanced_lines = LineBreakResults::new(node, space);
        let avg_line_width = normal_lines.LineWidthSum() / num_lines as i32;
        let min_available_width = LayoutUnit::FromFloatRound(avg_line_width * 0.8f32);
        Some(balanced_lines.BisectAvailableWidth(
            available_width,
            min_available_width,
            epsilon,
            line_clamp_ellipsis_width,
            num_lines,
            normal_lines.BreakToken(),
        ))
    }
}
