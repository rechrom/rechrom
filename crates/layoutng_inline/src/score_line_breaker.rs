// C++: layoutng_inline/score_line_breaker.h/.cc.
#![allow(non_snake_case)]

use foundation::{ETextAlign, LayoutUnit, RuntimeEnabledFeatures, WtfSizeT};
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::exclusions::line_layout_opportunity::LineLayoutOpportunity;
use layoutng::internal::inline_node::InlineNode;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;

use crate::leading_floats::LeadingFloats;
use crate::line_break_candidate::{
    LineBreakCandidate, LineBreakCandidateContext, LineBreakCandidates,
};
use crate::line_breaker::{LineBreaker, LineBreakerMode};
use crate::line_info_list::LineInfoList;
use crate::line_widths::LineWidths;
use crate::score_line_break_context::{
    kMaxLinesForBalance, kMaxLinesForOptimal, LineBreakPoints, ScoreLineBreakContext,
};

// cpp: layoutng_inline/score_line_breaker.cc:24-50
fn ShouldOptimize(line_info_list: &dyn LineInfoList, line_breaker: &mut LineBreaker) -> bool {
    let last_line = line_info_list.Back();
    const SHORT_LINE_DENOMINATOR: i32 = 3;
    if last_line.Width() < last_line.AvailableWidth() / SHORT_LINE_DENOMINATOR
        && !line_breaker.CanBreakInsideLine(last_line)
    {
        return true;
    }

    const NUM_LAST_HYPHENATED_LINES: WtfSizeT = 2;
    let num_lines = line_info_list.Size();
    if num_lines >= NUM_LAST_HYPHENATED_LINES + 1
        && line_info_list.At(num_lines - 2).IsHyphenated()
        && line_info_list.At(num_lines - 3).IsHyphenated()
    {
        return true;
    }
    false
}

// cpp: layoutng_inline/score_line_breaker.h:80-86
#[derive(Clone, Copy, Debug, Default)]
struct LineBreakScore {
    score: f32,
    prev_index: usize,
    line_index: usize,
}

// cpp: layoutng_inline/score_line_breaker.h:43-125
pub struct ScoreLineBreaker<'a> {
    node_: InlineNode,
    space_: &'a ConstraintSpace,
    line_widths_: &'a LineWidths,
    exclusion_space_: *mut ExclusionSpace,
    break_token_: *const InlineBreakToken,
    first_line_indent_: LayoutUnit,
    hyphen_penalty_: f32,
    line_penalty_: f32,
    zoom_: f32,
    is_balanced_: bool,
    is_justified_: bool,
    scores_out_for_testing_: *mut Vec<f32>,
}

impl<'a> ScoreLineBreaker<'a> {
    const SCORE_INFINITY: f32 = f32::MAX;
    const SCORE_OVERFULL: f32 = 1e12;
    const LAST_LINE_PENALTY_MULTIPLIER: f32 = 4.0;

    // The source stores InlineNode by value. Rust's InlineNode has the same
    // handle semantics and is cloned from the borrowed caller value.
    // cpp: layoutng_inline/score_line_breaker.h:47-58
    pub fn new(
        node: &InlineNode,
        space: &'a ConstraintSpace,
        line_widths: &'a LineWidths,
        break_token: *const InlineBreakToken,
        exclusion_space: &mut ExclusionSpace,
    ) -> Self {
        debug_assert!(!node.IsScoreLineBreakDisabled());
        Self {
            node_: node.clone(),
            space_: space,
            line_widths_: line_widths,
            exclusion_space_: exclusion_space,
            break_token_: break_token,
            first_line_indent_: LayoutUnit::default(),
            hyphen_penalty_: 0.0,
            line_penalty_: 0.0,
            zoom_: 0.0,
            is_balanced_: false,
            is_justified_: false,
            scores_out_for_testing_: std::ptr::null_mut(),
        }
    }

    // cpp: layoutng_inline/score_line_breaker.h:60-65,88-88
    pub fn MaxLines(&self) -> WtfSizeT {
        if self.is_balanced_ {
            kMaxLinesForBalance
        } else {
            kMaxLinesForOptimal
        }
    }
    pub fn GetConstraintSpace(&self) -> &ConstraintSpace {
        self.space_
    }
    pub fn BreakToken(&self) -> *const InlineBreakToken {
        self.break_token_
    }
    fn Node(&self) -> &InlineNode {
        &self.node_
    }

    // cpp: layoutng_inline/score_line_breaker.cc:54-56
    pub fn SetScoresOutForTesting(&mut self, scores_out: &mut Vec<f32>) {
        self.scores_out_for_testing_ = scores_out;
    }

    // cpp: layoutng_inline/score_line_breaker.cc:58-166
    pub fn OptimalBreakPoints(
        &mut self,
        leading_floats: &LeadingFloats,
        context: &mut dyn ScoreLineBreakContext,
    ) {
        debug_assert!(!self.is_balanced_ || self.break_token_.is_null());
        debug_assert!(context.GetLineBreakPoints().is_empty());
        debug_assert!(!self.node_.IsScoreLineBreakDisabled());
        debug_assert!(context.IsActive());
        let max_lines = self.MaxLines();
        let mut line_index = 0;
        {
            let lines = context.GetLineInfoList();
            debug_assert!(lines.MaxLines() >= max_lines);
            debug_assert!(lines.Size() < max_lines);
            if !lines.IsEmpty() {
                line_index = lines.Size();
                let last_line = lines.Back();
                self.break_token_ = last_line.GetBreakToken();
                debug_assert!(!self.break_token_.is_null() && !last_line.HasForcedBreak());
            }
        }

        let mut line_width = self.line_widths_.At(line_index);
        let line_opportunity = LineLayoutOpportunity::with_inline_size(line_width);
        let mut line_breaker = LineBreaker::new(
            self.node_.clone(),
            LineBreakerMode::kContent,
            self.GetConstraintSpace(),
            &line_opportunity,
            leading_floats,
            self.break_token_,
            std::ptr::null(),
            self.exclusion_space_,
        );
        // The extracted C++ LineClampData::LinesUntilClamp has an optional
        // show_measured_lines argument; the default is false.
        let lines_until_clamp = self
            .space_
            .GetLineClampData()
            .LinesUntilClamp(false)
            .unwrap_or(0);
        if RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled()
            && lines_until_clamp != 0
        {
            context.SuspendUntilEndParagraph();
            return;
        }
        debug_assert!(
            !RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled()
                || lines_until_clamp == 0
        );
        loop {
            let lines = context.GetLineInfoList();
            let line_info = lines.Append();
            line_breaker.NextLine(line_info);
            self.break_token_ = line_info.GetBreakToken();
            if RuntimeEnabledFeatures::ScoreLineBreakerAbortEnabled()
                && line_info.HasUnsuccessfulBlockInInline()
            {
                context.SuspendUntilEndParagraph();
                return;
            }
            if line_breaker.ShouldDisableScoreLineBreak() {
                context.SuspendUntilEndParagraph();
                return;
            }
            let end_paragraph = line_info.IsEndParagraph();
            let results_empty = line_info.Results().is_empty();
            let at_clamp = lines_until_clamp > 0 && lines.Size() == lines_until_clamp as WtfSizeT;
            if end_paragraph || at_clamp {
                context.SuspendUntilEndParagraph();
                break;
            }
            debug_assert!(!results_empty);
            debug_assert!(!line_breaker.IsFinished());
            if lines.Size() >= max_lines {
                return;
            }

            line_index += 1;
            let next_line_width = self.line_widths_.At(line_index);
            if next_line_width != line_width {
                line_width = next_line_width;
                let line_opportunity = LineLayoutOpportunity::with_inline_size(line_width);
                line_breaker.SetLineOpportunity(&line_opportunity);
            }
        }
        let lines = context.GetLineInfoList();
        debug_assert!(!lines.IsEmpty());
        if lines.Size() <= 1 {
            return;
        }
        if !self.is_balanced_ && !ShouldOptimize(lines, &mut line_breaker) {
            return;
        }
        let (lines, break_points) = context.GetLineInfoListAndBreakPoints();
        if !self.Optimize(lines, &mut line_breaker, break_points) {
            debug_assert!(break_points.is_empty());
            return;
        }
        debug_assert!(!break_points.is_empty());
        debug_assert_eq!(lines.Size() as usize, break_points.len());
        for index in 0..lines.Size() {
            if lines.At(index).End() != break_points[index as usize].offset {
                lines.Shrink(index);
                break;
            }
        }
    }

    // cpp: layoutng_inline/score_line_breaker.cc:168-172
    pub fn BalanceBreakPoints(
        &mut self,
        leading_floats: &LeadingFloats,
        context: &mut dyn ScoreLineBreakContext,
    ) {
        self.is_balanced_ = true;
        self.OptimalBreakPoints(leading_floats, context);
    }

    // cpp: layoutng_inline/score_line_breaker.cc:174-241
    fn Optimize(
        &mut self,
        lines: &mut dyn LineInfoList,
        line_breaker: &mut LineBreaker,
        break_points: &mut LineBreakPoints,
    ) -> bool {
        debug_assert!(break_points.is_empty());
        self.SetupParameters();
        let mut candidates = LineBreakCandidates::new();
        if !self.ComputeCandidates(lines, line_breaker, &mut candidates) {
            debug_assert!(break_points.is_empty());
            return false;
        }
        debug_assert!(candidates.len() >= 2);
        const MIN_CANDIDATES: usize = 3;
        if candidates.len() < MIN_CANDIDATES + 2 {
            debug_assert!(break_points.is_empty());
            return false;
        }
        if candidates.len() >= 4 {
            const ORPHANS_PENALTY: f32 = 10000.0;
            let orphans_penalty = ORPHANS_PENALTY * self.zoom_;
            let before_last = candidates.len() - 1;
            for candidate in candidates[..before_last].iter_mut().rev() {
                candidate.penalty += orphans_penalty;
                if !candidate.is_hyphenated {
                    break;
                }
            }
        }
        self.ComputeLineWidths(lines);
        let mut scores = Vec::with_capacity(candidates.len());
        self.ComputeScores(&candidates, &mut scores);
        debug_assert_eq!(candidates.len(), scores.len());
        self.ComputeBreakPoints(&candidates, &scores, break_points);
        if break_points.len() != lines.Size() as usize {
            break_points.clear();
            return false;
        }
        if !self.scores_out_for_testing_.is_null() {
            let output = unsafe { &mut *self.scores_out_for_testing_ };
            for score in &scores {
                output.push(score.score);
            }
        }
        true
    }

    // cpp: layoutng_inline/score_line_breaker.cc:243-263
    fn ComputeCandidates(
        &self,
        lines: &mut dyn LineInfoList,
        line_breaker: &mut LineBreaker,
        candidates: &mut LineBreakCandidates,
    ) -> bool {
        debug_assert!(candidates.is_empty());
        let mut context = LineBreakCandidateContext::new(candidates);
        context.SetHyphenPenalty(self.hyphen_penalty_);
        context.EnsureFirstSentinel(lines.Front());
        for index in 0..lines.Size() {
            if !context.AppendLine(lines.AtMut(index), line_breaker) {
                drop(context);
                candidates.clear();
                return false;
            }
        }
        context.EnsureLastSentinel(lines.Back());
        true
    }

    // cpp: layoutng_inline/score_line_breaker.cc:265-271
    fn AvailableWidth(&self, line_index: usize) -> LayoutUnit {
        let mut width = self.line_widths_.At(line_index as WtfSizeT);
        if line_index == 0 {
            width -= self.first_line_indent_;
        }
        width.ClampNegativeToZero()
    }

    // cpp: layoutng_inline/score_line_breaker.h:89-92
    fn AvailableWidthToFit(&self, line_index: usize) -> LayoutUnit {
        self.AvailableWidth(line_index).AddEpsilon()
    }

    // cpp: layoutng_inline/score_line_breaker.cc:273-281
    fn ComputeLineWidths(&mut self, lines: &dyn LineInfoList) {
        self.first_line_indent_ = lines.Front().TextIndent();
        #[cfg(feature = "expensive_dchecks")]
        for index in 1..lines.Size() {
            debug_assert_eq!(lines.At(index).TextIndent(), LayoutUnit::default());
        }
    }

    // cpp: layoutng_inline/score_line_breaker.cc:283-305
    fn SetupParameters(&mut self) {
        let available_width = self.line_widths_.Default().ClampNegativeToZero();
        let block_style = self.node_.Style();
        let font_size = block_style.GetFontDescription().ComputedSize();
        self.zoom_ = block_style.EffectiveZoom();
        debug_assert!(self.zoom_ > 0.0);
        let width_times_font_size = available_width.ToFloat() * font_size / self.zoom_;
        self.is_justified_ = block_style.GetTextAlign() == ETextAlign::kJustify;
        if self.is_justified_ {
            self.hyphen_penalty_ = width_times_font_size / 2.0;
            self.line_penalty_ = 0.0;
        } else {
            self.hyphen_penalty_ = width_times_font_size * 2.0;
            self.line_penalty_ = self.hyphen_penalty_ * 2.0;
        }
    }

    // cpp: layoutng_inline/score_line_breaker.cc:307-384
    fn ComputeScores(&self, candidates: &LineBreakCandidates, scores: &mut Vec<LineBreakScore>) {
        debug_assert!(candidates.len() >= 2);
        debug_assert!(scores.is_empty());
        scores.push(LineBreakScore::default());
        let mut active = 0;
        for end in 1..candidates.len() {
            let end_candidate = &candidates[end];
            let is_end_last_candidate = end == candidates.len() - 1;
            let mut best = Self::SCORE_INFINITY;
            let mut best_prev_index = 0;
            let mut last_line_index = scores[active].line_index;
            let mut available_width = self.AvailableWidthToFit(last_line_index);
            let mut start_edge = end_candidate.pos_if_break - available_width.ToFloat();
            let mut best_hope = 0.0;
            for start in active..end {
                let start_score = scores[start];
                let line_index = start_score.line_index;
                if line_index != last_line_index {
                    last_line_index = line_index;
                    let new_available_width = self.AvailableWidthToFit(line_index);
                    if new_available_width != available_width {
                        available_width = new_available_width;
                        start_edge = end_candidate.pos_if_break - available_width.ToFloat();
                        best_hope = 0.0;
                    }
                }
                let start_score_value = start_score.score;
                if start_score_value + best_hope >= best {
                    continue;
                }
                let start_candidate = &candidates[start];
                let delta = start_candidate.pos_no_break - start_edge;
                let mut width_score = 0.0;
                let mut additional_penalty = 0.0;
                if (is_end_last_candidate || !self.is_justified_) && delta < 0.0 {
                    width_score = Self::SCORE_OVERFULL;
                } else if is_end_last_candidate && !self.is_balanced_ {
                    additional_penalty =
                        Self::LAST_LINE_PENALTY_MULTIPLIER * start_candidate.penalty;
                } else if delta < 0.0 {
                    width_score = Self::SCORE_OVERFULL;
                } else {
                    width_score = delta * delta / self.zoom_;
                }
                if delta < 0.0 {
                    active = start + 1;
                } else {
                    best_hope = width_score;
                }
                let score = start_score_value + width_score + additional_penalty;
                if score <= best {
                    best = score;
                    best_prev_index = start;
                }
            }
            scores.push(LineBreakScore {
                score: best + end_candidate.penalty + self.line_penalty_,
                prev_index: best_prev_index,
                line_index: scores[best_prev_index].line_index + 1,
            });
        }
    }

    // cpp: layoutng_inline/score_line_breaker.cc:386-417
    fn ComputeBreakPoints(
        &self,
        candidates: &LineBreakCandidates,
        scores: &[LineBreakScore],
        break_points: &mut LineBreakPoints,
    ) {
        debug_assert!(candidates.len() >= 3);
        debug_assert_eq!(candidates.len(), scores.len());
        debug_assert!(break_points.is_empty());
        debug_assert!(scores.last().unwrap().line_index <= self.MaxLines() as usize);
        let mut index = scores.len() - 1;
        while index > 0 {
            let prev_index = scores[index].prev_index;
            let candidate: &LineBreakCandidate = &candidates[index];
            break_points.push(candidate.base);
            #[cfg(feature = "expensive_dchecks")]
            {
                let previous = &candidates[prev_index];
                let line_width =
                    LayoutUnit::FromFloatCeil(candidate.pos_if_break - previous.pos_no_break);
                debug_assert!(line_width >= LayoutUnit::default());
                break_points.last_mut().unwrap().line_width = line_width;
            }
            index = prev_index;
        }
        debug_assert_eq!(break_points.len(), scores.last().unwrap().line_index);
        break_points.reverse();
        #[cfg(feature = "expensive_dchecks")]
        {
            debug_assert_eq!(break_points.len(), scores.last().unwrap().line_index);
            for index in 1..break_points.len() {
                debug_assert!(break_points[index]
                    .offset
                    .GreaterThan(&break_points[index - 1].offset));
            }
        }
    }
}
