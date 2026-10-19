// Translation status: complete
// Source: cc/input/browser_controls_offset_manager.h:1-385
// Source: cc/input/browser_controls_offset_manager.cc:1-1405
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/browser_controls_offset_manager.h:1-32,44-49,202-210,243-244,381-385;
//         cc/input/browser_controls_offset_manager.cc:1-32,64-66,104-105,1405.
// Rust modules/imports replace guards, includes, namespaces and exports. A
// per-call live client context preserves the non-owning relationship; no Clone/Copy
// preserves the deleted copy members and automatic drop is the default dtor.
use crate::adapter::browser_controls::{
    BrowserControlsDiagnostic, BrowserControlsFeature, InMillisecondsF, Tween, TweenType,
};
use crate::adapter::input_geometry::Vector2dF;
use crate::adapter::input_services::OffsetTag;
use crate::adapter::time::{TimeDelta, TimeTicks};
use crate::cc::input::browser_controls_offset_manager_client::{
    BrowserControlsOffsetManagerClient, BrowserControlsOffsetManagerQueries,
};
use crate::cc::input::browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications;
use crate::cc::input::browser_controls_state::BrowserControlsState;
use crate::cc::input::scroll_velocity_tracker::ScrollVelocityTracker;
use crate::cc::trees::browser_controls_params::BrowserControlsParams;

// Source: cc/input/browser_controls_offset_manager.cc:33-54.
const kShowHideMaxDurationMs: i64 = 200;
const kHeightChangeDurationMs: i64 = 200;
const kShowHideMinDurationMs: i64 = 75;
const kAlwaysShownRegionMultiplier: f32 = 0.5;
const kCanHideRegionMinMultiplier: f32 = 1.5;
const kCanHideRegionMaxMultiplier: f32 = 3.0;
const _: () = assert!(kCanHideRegionMinMultiplier >= kAlwaysShownRegionMultiplier + 1.0);
const _: () = assert!(kAlwaysShownRegionMultiplier > 0.0);
const kSnapAnimationThresholdMinMultiplier: f32 = 0.2;
const kSnapAnimationReferenceThresholdDp: f32 = 30.0;

// std::min/max/clamp language dependency preserving operand-order/NaN behavior.
fn min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}
fn clamp(value: f32, lo: f32, hi: f32) -> f32 {
    debug_assert!(!(hi < lo));
    if value < lo {
        lo
    } else if hi < value {
        hi
    } else {
        value
    }
}

// Source: cc/input/browser_controls_offset_manager.cc:55-63.
fn NormalizeShownRatio(value: f32, min_shown_ratio: f32) -> f32 {
    if min_shown_ratio == 1.0 {
        return 1.0;
    }
    (clamp(value, min_shown_ratio, 1.0) - min_shown_ratio) / (1.0 - min_shown_ratio)
}

// Source: cc/input/browser_controls_offset_manager.h:33-38.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationDirection {
    kNoAnimation,
    kShowingControls,
    kHidingControls,
}

// Source: cc/input/browser_controls_offset_manager.h:243-312,377-380.
pub struct BrowserControlsOffsetManager {
    permitted_state_: BrowserControlsState,
    accumulated_scroll_delta_: f32,
    unapplied_scroll_delta_: f32,
    baseline_top_content_offset_: f32,
    baseline_bottom_content_offset_: f32,
    controls_show_threshold_: f32,
    controls_hide_threshold_: f32,
    pinch_gesture_active_: bool,
    constraint_changed_since_commit_: bool,
    old_browser_controls_params_: BrowserControlsParams,
    top_min_height_change_in_progress_: bool,
    bottom_min_height_change_in_progress_: bool,
    top_controls_min_height_offset_: f32,
    bottom_controls_min_height_offset_: f32,
    top_min_height_offset_animation_range_: Option<(f32, f32)>,
    bottom_min_height_offset_animation_range_: Option<(f32, f32)>,
    show_controls_when_scroll_completes_: bool,
    did_hide_this_scroll_: bool,
    use_snap_animation_: bool,
    in_viz_process_: bool,
    offset_tag_modifications_: BrowserControlsOffsetTagModifications,
    top_controls_animation_: Animation,
    bottom_controls_animation_: Animation,
    scroll_velocity_tracker_: ScrollVelocityTracker,
}

impl BrowserControlsOffsetManager {
    // Source: cc/input/browser_controls_offset_manager.h:39-43;
    //         cc/input/browser_controls_offset_manager.cc:67-76.
    // Rust makes the C++ default argument explicit; false is the source default.
    pub fn Create<C: BrowserControlsOffsetManagerClient + ?Sized>(
        client: &mut C,
        controls_show_threshold: f32,
        controls_hide_threshold: f32,
        in_viz_process: bool,
    ) -> Box<Self> {
        Box::new(Self::new(
            client,
            controls_show_threshold,
            controls_hide_threshold,
            in_viz_process,
        ))
    }
    // Source: cc/input/browser_controls_offset_manager.h:203-206;
    //         cc/input/browser_controls_offset_manager.cc:77-103.
    fn new<C: BrowserControlsOffsetManagerClient + ?Sized>(
        client: &mut C,
        controls_show_threshold: f32,
        controls_hide_threshold: f32,
        in_viz_process: bool,
    ) -> Self {
        let mut result = Self::Initialize(
            controls_show_threshold,
            controls_hide_threshold,
            in_viz_process,
            client.FeatureEnabled(BrowserControlsFeature::ScrollSnapAnimation),
        );
        // CHECK(client_) is enforced by the non-null per-call Rust client context.
        result.UpdateOldBrowserControlsParams(client);
        result
    }

    // Source: cc/input/browser_controls_offset_manager.cc:77-103.
    // Rust borrow split: one canonical initializer serves both the complete
    // live client and HostImpl construction, before other children exist.
    fn Initialize(
        controls_show_threshold: f32,
        controls_hide_threshold: f32,
        in_viz_process: bool,
        use_snap_animation: bool,
    ) -> Self {
        Self {
            permitted_state_: BrowserControlsState::kBoth,
            accumulated_scroll_delta_: 0.0,
            unapplied_scroll_delta_: 0.0,
            baseline_top_content_offset_: 0.0,
            baseline_bottom_content_offset_: 0.0,
            // Preserve the source initializer's threshold argument ordering.
            controls_show_threshold_: controls_hide_threshold,
            controls_hide_threshold_: controls_show_threshold,
            pinch_gesture_active_: false,
            constraint_changed_since_commit_: false,
            old_browser_controls_params_: BrowserControlsParams::default(),
            top_min_height_change_in_progress_: false,
            bottom_min_height_change_in_progress_: false,
            top_controls_min_height_offset_: 0.0,
            bottom_controls_min_height_offset_: 0.0,
            top_min_height_offset_animation_range_: None,
            bottom_min_height_offset_animation_range_: None,
            show_controls_when_scroll_completes_: false,
            did_hide_this_scroll_: false,
            use_snap_animation_: use_snap_animation,
            in_viz_process_: in_viz_process,
            offset_tag_modifications_: BrowserControlsOffsetTagModifications::default(),
            top_controls_animation_: Animation::new(),
            bottom_controls_animation_: Animation::new(),
            scroll_velocity_tracker_: ScrollVelocityTracker::new(TimeDelta::Milliseconds(
                kShowHideMaxDurationMs,
            )),
        }
    }
    // Source: cc/input/browser_controls_offset_manager.cc:77-103,1265-1274.
    // Direct borrowed active-tree parameters and the current feature result
    // are consumed during construction; no owner state is retained here.
    pub(crate) fn CreateForHost(
        params: &BrowserControlsParams,
        use_snap_animation: bool,
        controls_show_threshold: f32,
        controls_hide_threshold: f32,
        in_viz_process: bool,
    ) -> Box<Self> {
        let mut result = Self::Initialize(
            controls_show_threshold,
            controls_hide_threshold,
            in_viz_process,
            use_snap_animation,
        );
        result.UpdateOldBrowserControlsParamsFromValues(
            params.top_controls_height,
            params.top_controls_min_height,
            params.bottom_controls_height,
            params.bottom_controls_min_height,
        );
        Box::new(result)
    }

    // Source: cc/input/browser_controls_offset_manager.h:50-52;
    //         cc/input/browser_controls_offset_manager.cc:106-109.
    pub fn ControlsTopOffset<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        self.ContentTopOffset(client) - self.TopControlsHeight(client)
    }
    // Source: cc/input/browser_controls_offset_manager.h:54-56;
    //         cc/input/browser_controls_offset_manager.cc:110-114.
    pub fn ContentTopOffset<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        if self.TopControlsHeight(client) > 0.0 {
            self.TopControlsShownRatio(client) * self.TopControlsHeight(client)
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:58;
    //         cc/input/browser_controls_offset_manager.cc:115-118.
    pub fn TopControlsShownRatio<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.CurrentTopControlsShownRatio()
    }
    // Source: cc/input/browser_controls_offset_manager.h:59;
    //         cc/input/browser_controls_offset_manager.cc:119-122.
    pub fn TopControlsHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.TopControlsHeight()
    }
    // Source: cc/input/browser_controls_offset_manager.h:60;
    //         cc/input/browser_controls_offset_manager.cc:123-126.
    pub fn TopControlsMinHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.TopControlsMinHeight()
    }
    // Source: cc/input/browser_controls_offset_manager.h:61;
    //         cc/input/browser_controls_offset_manager.cc:127-130.
    pub fn TopControlsAdditionalHeight(&self) -> i32 {
        self.offset_tag_modifications_
            .top_controls_additional_height
    }
    // Source: cc/input/browser_controls_offset_manager.h:84;
    //         cc/input/browser_controls_offset_manager.cc:131-134.
    pub fn BottomControlsAdditionalHeight(&self) -> i32 {
        self.offset_tag_modifications_
            .bottom_controls_additional_height
    }
    // Source: cc/input/browser_controls_offset_manager.h:95;
    //         cc/input/browser_controls_offset_manager.cc:135-138.
    pub fn BottomControlsOffsetTag(&self) -> OffsetTag {
        self.offset_tag_modifications_
            .tags
            .bottom_controls_offset_tag
    }
    // Source: cc/input/browser_controls_offset_manager.h:72;
    //         cc/input/browser_controls_offset_manager.cc:139-142.
    pub fn ContentOffsetTag(&self) -> OffsetTag {
        self.offset_tag_modifications_.tags.content_offset_tag
    }
    // Source: cc/input/browser_controls_offset_manager.h:73;
    //         cc/input/browser_controls_offset_manager.cc:143-146.
    pub fn TopControlsOffsetTag(&self) -> OffsetTag {
        self.offset_tag_modifications_.tags.top_controls_offset_tag
    }
    // Source: cc/input/browser_controls_offset_manager.h:63-64;
    //         cc/input/browser_controls_offset_manager.cc:147-151.
    pub fn TopControlsMinShownRatio<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        if self.TopControlsHeight(client) != 0.0 {
            self.TopControlsMinHeight(client) / self.TopControlsHeight(client)
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:79-81;
    //         cc/input/browser_controls_offset_manager.cc:152-155.
    pub fn BottomControlsHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.BottomControlsHeight()
    }
    // Source: cc/input/browser_controls_offset_manager.h:82;
    //         cc/input/browser_controls_offset_manager.cc:156-159.
    pub fn BottomControlsMinHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.BottomControlsMinHeight()
    }
    // Source: cc/input/browser_controls_offset_manager.h:86-87;
    //         cc/input/browser_controls_offset_manager.cc:160-165.
    pub fn BottomControlsMinShownRatio<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        if self.BottomControlsHeight(client) != 0.0 {
            self.BottomControlsMinHeight(client) / self.BottomControlsHeight(client)
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:75-77;
    //         cc/input/browser_controls_offset_manager.cc:166-170.
    pub fn ContentBottomOffset<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        if self.BottomControlsHeight(client) > 0.0 {
            self.BottomControlsShownRatio(client) * self.BottomControlsHeight(client)
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:83;
    //         cc/input/browser_controls_offset_manager.cc:171-174.
    pub fn BottomControlsShownRatio<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        client.CurrentBottomControlsShownRatio()
    }
    // Source: cc/input/browser_controls_offset_manager.h:66-70;
    //         cc/input/browser_controls_offset_manager.cc:175-178.
    pub fn TopControlsMinHeightOffset(&self) -> f32 {
        self.top_controls_min_height_offset_
    }
    // Source: cc/input/browser_controls_offset_manager.h:89-93;
    //         cc/input/browser_controls_offset_manager.cc:179-182.
    pub fn BottomControlsMinHeightOffset(&self) -> f32 {
        self.bottom_controls_min_height_offset_
    }
    // Source: cc/input/browser_controls_offset_manager.h:97;
    //         cc/input/browser_controls_offset_manager.cc:183-187.
    pub fn HasOffsetTag(&self) -> bool {
        !self.BottomControlsOffsetTag().IsEmpty()
            || !self.ContentOffsetTag().IsEmpty()
            || !self.TopControlsOffsetTag().IsEmpty()
    }
    // Source: cc/input/browser_controls_offset_manager.h:99-101;
    //         cc/input/browser_controls_offset_manager.cc:188-196.
    pub fn TopControlsShownRatioRange(&self) -> (f32, f32) {
        if self.top_controls_animation_.IsInitialized() {
            return (
                self.top_controls_animation_.min_value(),
                self.top_controls_animation_.max_value(),
            );
        }
        (0.0, 1.0)
    }
    // Source: cc/input/browser_controls_offset_manager.h:103-105;
    //         cc/input/browser_controls_offset_manager.cc:197-205.
    pub fn BottomControlsShownRatioRange(&self) -> (f32, f32) {
        if self.bottom_controls_animation_.IsInitialized() {
            return (
                self.bottom_controls_animation_.min_value(),
                self.bottom_controls_animation_.max_value(),
            );
        }
        (0.0, 1.0)
    }
    // Source: cc/input/browser_controls_offset_manager.h:239;
    //         cc/input/browser_controls_offset_manager.cc:206-213.
    fn ControlsAnimatedHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        let animated_height = if self.TopControlsHeight(client) > 0.0 {
            self.TopControlsAnimatedHeight(client)
        } else {
            self.BottomControlsAnimatedHeight(client)
        };
        debug_assert!(animated_height >= 0.0);
        animated_height
    }
    // Source: cc/input/browser_controls_offset_manager.h:240;
    //         cc/input/browser_controls_offset_manager.cc:214-217.
    fn TopControlsAnimatedHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        self.TopControlsHeight(client) - self.TopControlsMinHeight(client)
    }
    // Source: cc/input/browser_controls_offset_manager.h:241;
    //         cc/input/browser_controls_offset_manager.cc:218-221.
    fn BottomControlsAnimatedHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        self.BottomControlsHeight(client) - self.BottomControlsMinHeight(client)
    }
    // Source: cc/input/browser_controls_offset_manager.h:179-181;
    //         cc/input/browser_controls_offset_manager.cc:222-234.
    pub fn SnapAnimationAlwaysShownRegionHeight<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
    ) -> f32 {
        // Trigger early near page top to avoid shifting web content.
        if self.TopControlsHeight(client) == 0.0 {
            return 0.0;
        }
        self.TopControlsAnimatedHeight(client) * kAlwaysShownRegionMultiplier
    }
    // Source: cc/input/browser_controls_offset_manager.h:183-191;
    //         cc/input/browser_controls_offset_manager.cc:235-255.
    pub fn SnapAnimationCanHideRegionHeights<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
        slowness: f32,
    ) -> (f32, f32) {
        debug_assert!(slowness >= 0.0);
        debug_assert!(slowness <= 1.0);
        let multiplier = Tween::FloatValueBetween(
            slowness as f64,
            kCanHideRegionMinMultiplier,
            kCanHideRegionMaxMultiplier,
        );
        (
            self.TopControlsAnimatedHeight(client) * multiplier,
            self.BottomControlsAnimatedHeight(client) * multiplier,
        )
    }
    // Source: cc/input/browser_controls_offset_manager.h:193-200;
    //         cc/input/browser_controls_offset_manager.cc:256-275.
    pub fn SnapAnimationThreshold<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
        slowness: f32,
    ) -> f32 {
        debug_assert!(slowness >= 0.0);
        debug_assert!(slowness <= 1.0);
        let reference_threshold = client.DeviceScaleFactor() * kSnapAnimationReferenceThresholdDp;
        reference_threshold
            * Tween::FloatValueBetween(slowness as f64, kSnapAnimationThresholdMinMultiplier, 1.0)
    }

    // Source: cc/input/browser_controls_offset_manager.h:114-121;
    //         cc/input/browser_controls_offset_manager.cc:276-355.
    pub fn UpdateBrowserControlsState<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        constraints: BrowserControlsState,
        current: BrowserControlsState,
        animate: bool,
        offset_tag_modifications: Option<&BrowserControlsOffsetTagModifications>,
    ) {
        debug_assert!(
            !(constraints == BrowserControlsState::kShown
                && current == BrowserControlsState::kHidden)
        );
        debug_assert!(
            !(constraints == BrowserControlsState::kHidden
                && current == BrowserControlsState::kShown)
        );
        client.EmitDiagnostic(BrowserControlsDiagnostic::UpdateConstraints {
            constraints,
            current,
        });
        // Changes affect Blink main-thread scrolling and viewport layout.
        if self.permitted_state_ != constraints {
            self.constraint_changed_since_commit_ = true;
            client.SetNeedsCommitDefault();
        }
        self.permitted_state_ = constraints;
        if let Some(modifications) = offset_tag_modifications {
            self.offset_tag_modifications_ = *modifications;
        }
        if constraints == BrowserControlsState::kBoth && current == BrowserControlsState::kBoth {
            return;
        }
        let mut final_top_shown_ratio = 1.0;
        let mut final_bottom_shown_ratio = 1.0;
        let mut direction = AnimationDirection::kShowingControls;
        if constraints == BrowserControlsState::kHidden || current == BrowserControlsState::kHidden
        {
            final_top_shown_ratio = self.TopControlsMinShownRatio(client);
            final_bottom_shown_ratio = self.BottomControlsMinShownRatio(client);
            direction = AnimationDirection::kHidingControls;
        }
        if final_top_shown_ratio == self.TopControlsShownRatio(client)
            && final_bottom_shown_ratio == self.BottomControlsShownRatio(client)
        {
            client.EmitDiagnostic(BrowserControlsDiagnostic::RatiosUnchanged);
            self.ResetAnimations(client);
            return;
        }
        let animated_top_shown_ratio = if self.top_controls_animation_.IsInitialized() {
            self.top_controls_animation_.FinalValue()
        } else {
            self.TopControlsShownRatio(client)
        };
        let animated_bottom_shown_ratio = if self.bottom_controls_animation_.IsInitialized() {
            self.bottom_controls_animation_.FinalValue()
        } else {
            self.BottomControlsShownRatio(client)
        };
        if animated_top_shown_ratio == final_top_shown_ratio
            && animated_bottom_shown_ratio == final_bottom_shown_ratio
        {
            return;
        }
        self.ResetAnimations(client);
        // Restart showing animation after any current scroll completes.
        if animate && direction == AnimationDirection::kShowingControls {
            self.show_controls_when_scroll_completes_ = true;
        }
        if animate {
            self.SetupAnimation(client, direction, kShowHideMaxDurationMs, TweenType::LINEAR);
        } else {
            client.SetCurrentBrowserControlsShownRatio(
                final_top_shown_ratio,
                final_bottom_shown_ratio,
                self,
            );
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:123-129.
    pub fn SetOffsetTagModifications(
        &mut self,
        modifications: &BrowserControlsOffsetTagModifications,
    ) {
        self.offset_tag_modifications_ = *modifications;
    }
    // Source: cc/input/browser_controls_offset_manager.h:130-135.
    pub fn GetOffsetTagModifications(&self) -> &BrowserControlsOffsetTagModifications {
        &self.offset_tag_modifications_
    }
    // Source: cc/input/browser_controls_offset_manager.h:136-139;
    //         cc/input/browser_controls_offset_manager.cc:356-362.
    pub fn PullConstraintForMainThread(
        &self,
        out_changed_since_commit: &mut bool,
    ) -> BrowserControlsState {
        *out_changed_since_commit = self.constraint_changed_since_commit_;
        self.permitted_state_
    }
    // Source: cc/input/browser_controls_offset_manager.h:140-144;
    //         cc/input/browser_controls_offset_manager.cc:363-366.
    pub fn NotifyConstraintSyncedToMainThread(&mut self) {
        self.constraint_changed_since_commit_ = false;
    }

    // Source: cc/input/browser_controls_offset_manager.h:146;
    //         cc/input/browser_controls_offset_manager.cc:367-573.
    pub fn OnBrowserControlsParamsChanged<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        animate_changes: bool,
    ) {
        if self.old_browser_controls_params_.top_controls_height == self.TopControlsHeight(client)
            && self.old_browser_controls_params_.top_controls_min_height
                == self.TopControlsMinHeight(client)
            && self.old_browser_controls_params_.bottom_controls_height
                == self.BottomControlsHeight(client)
            && self.old_browser_controls_params_.bottom_controls_min_height
                == self.BottomControlsMinHeight(client)
        {
            return;
        }
        // Preserve shown ratios even at zero height so animations continue.
        let old_top_height = self.old_browser_controls_params_.top_controls_height;
        let mut new_top_ratio = if self.TopControlsHeight(client) != 0.0 {
            self.TopControlsShownRatio(client) * old_top_height / self.TopControlsHeight(client)
        } else {
            self.TopControlsShownRatio(client)
        };
        let old_bottom_height = self.old_browser_controls_params_.bottom_controls_height;
        let mut new_bottom_ratio = if self.BottomControlsHeight(client) != 0.0 {
            self.BottomControlsShownRatio(client) * old_bottom_height
                / self.BottomControlsHeight(client)
        } else {
            self.BottomControlsShownRatio(client)
        };
        if !animate_changes {
            // Snap hidden controls to minimum; keep fully shown controls fully
            // shown; track a changed minimum; otherwise preserve visible height.
            let hide_controls = self.permitted_state_ == BrowserControlsState::kHidden;
            if hide_controls {
                new_top_ratio = self.TopControlsMinShownRatio(client);
            } else if self.TopControlsShownRatio(client) == 1.0 {
                new_top_ratio = 1.0;
            } else if self.TopControlsShownRatio(client) == self.OldTopControlsMinShownRatio() {
                new_top_ratio = self.TopControlsMinShownRatio(client);
            }
            if hide_controls {
                new_bottom_ratio = self.BottomControlsMinShownRatio(client);
            } else if self.BottomControlsShownRatio(client) == 1.0 {
                new_bottom_ratio = 1.0;
            } else if self.BottomControlsShownRatio(client) == self.OldBottomControlsMinShownRatio()
            {
                new_bottom_ratio = self.BottomControlsMinShownRatio(client);
            }
            if client.FeatureEnabled(BrowserControlsFeature::HeightChangeCancelAnimations)
                && self.IsAnimatingHeightChange()
            {
                self.ResetAnimations(client);
            }
        }
        // Height-change animation covers: old minimum→new minimum, old full
        // height→new full height, and retargeting an animation ending at old
        // minimum. Intermediate scroll/show states retain their visible height.
        let mut top_controls_need_animation = animate_changes;
        let mut bottom_controls_need_animation = animate_changes;
        let mut effective_top_shown_ratio = self.TopControlsShownRatio(client);
        if self.top_controls_animation_.IsInitialized() {
            effective_top_shown_ratio = self.top_controls_animation_.FinalValue();
        }
        let mut effective_bottom_shown_ratio = self.BottomControlsShownRatio(client);
        if self.bottom_controls_animation_.IsInitialized() {
            effective_bottom_shown_ratio = self.bottom_controls_animation_.FinalValue();
        }
        // Option expresses C++ definite assignment: read a target only when its
        // branch set it and left the corresponding need-animation flag true.
        let mut top_target_ratio = None;
        if self.TopControlsHeight(client) == 0.0 {
            top_controls_need_animation = false;
        } else if self.TopControlsShownRatio(client) == 1.0
            && self.TopControlsHeight(client) != old_top_height
        {
            top_target_ratio = Some(1.0);
        } else if effective_top_shown_ratio == self.OldTopControlsMinShownRatio()
            && self.TopControlsMinHeight(client)
                != self.old_browser_controls_params_.top_controls_min_height
        {
            top_target_ratio = Some(self.TopControlsMinShownRatio(client));
        } else {
            top_controls_need_animation = false;
        }
        let mut bottom_target_ratio = None;
        if self.BottomControlsHeight(client) == 0.0 {
            bottom_controls_need_animation = false;
        } else if self.BottomControlsShownRatio(client) == 1.0
            && self.BottomControlsHeight(client) != old_bottom_height
        {
            bottom_target_ratio = Some(1.0);
        } else if effective_bottom_shown_ratio == self.OldBottomControlsMinShownRatio()
            && self.BottomControlsMinHeight(client)
                != self.old_browser_controls_params_.bottom_controls_min_height
        {
            bottom_target_ratio = Some(self.BottomControlsMinShownRatio(client));
        } else {
            bottom_controls_need_animation = false;
        }
        if top_controls_need_animation || bottom_controls_need_animation {
            self.MaybeRecordHasExistingAnimationHistogram(client);
        }
        if top_controls_need_animation {
            Self::InitAnimationForHeightChange(
                &mut self.top_controls_animation_,
                new_top_ratio,
                top_target_ratio.expect("initialized target"),
            );
            if self.old_browser_controls_params_.top_controls_min_height
                != self.TopControlsMinHeight(client)
            {
                self.top_controls_min_height_offset_ =
                    self.old_browser_controls_params_.top_controls_min_height;
                self.top_min_height_change_in_progress_ = true;
                self.SetTopMinHeightOffsetAnimationRange(
                    self.top_controls_min_height_offset_,
                    self.TopControlsMinHeight(client),
                );
            }
        } else {
            self.top_controls_min_height_offset_ = self.TopControlsMinHeight(client);
        }
        if bottom_controls_need_animation {
            Self::InitAnimationForHeightChange(
                &mut self.bottom_controls_animation_,
                new_bottom_ratio,
                bottom_target_ratio.expect("initialized target"),
            );
            if self.old_browser_controls_params_.bottom_controls_min_height
                != self.BottomControlsMinHeight(client)
            {
                self.bottom_controls_min_height_offset_ =
                    self.old_browser_controls_params_.bottom_controls_min_height;
                let height_delta = (self.BottomControlsHeight(client) - old_bottom_height) as i32;
                let min_height_delta = (self.BottomControlsMinHeight(client)
                    - self.old_browser_controls_params_.bottom_controls_min_height)
                    as i32;
                // Keep min-height delta aligned with height delta so content and
                // bottom controls do not leave a visible gap during animation.
                if min_height_delta > height_delta {
                    self.bottom_controls_min_height_offset_ +=
                        (min_height_delta - height_delta) as f32;
                }
                self.bottom_min_height_change_in_progress_ = true;
                self.SetBottomMinHeightOffsetAnimationRange(
                    self.bottom_controls_min_height_offset_,
                    self.BottomControlsMinHeight(client),
                );
            }
        } else {
            self.bottom_controls_min_height_offset_ = self.BottomControlsMinHeight(client);
        }
        self.UpdateOldBrowserControlsParams(client);
        client.SetCurrentBrowserControlsShownRatio(new_top_ratio, new_bottom_ratio, self);
    }

    // Source: cc/input/browser_controls_offset_manager.h:148;
    //         cc/input/browser_controls_offset_manager.cc:574-601.
    pub fn ScrollBegin<C: BrowserControlsOffsetManagerClient + ?Sized>(&mut self, client: &mut C) {
        if self.pinch_gesture_active_ {
            return;
        }
        if self.use_snap_animation_ {
            // An already-running snap animation is allowed to complete.
            if self.HasAnimation() {
                self.did_hide_this_scroll_ = (self.top_controls_animation_.IsInitialized()
                    && self.top_controls_animation_.Direction()
                        == AnimationDirection::kHidingControls)
                    || (self.bottom_controls_animation_.IsInitialized()
                        && self.bottom_controls_animation_.Direction()
                            == AnimationDirection::kHidingControls);
                return;
            }
        } else {
            // Reorder showing animation after gesture rather than accidentally
            // hiding controls with a gesture too small to normally hide them.
            self.show_controls_when_scroll_completes_ = self.IsAnimatingToShowControls();
        }
        self.ResetAnimations(client);
        self.ResetBaseline(client);
    }
    // Source: cc/input/browser_controls_offset_manager.h:177;
    //         cc/input/browser_controls_offset_manager.cc:602-622.
    pub fn MaximumShownRatioDeltaPerFrame<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
        min_ratio: f32,
    ) -> f32 {
        if !client.FeatureEnabled(BrowserControlsFeature::SmoothScroll) {
            return 1.0;
        }
        let mut animation_duration_ms = kShowHideMaxDurationMs as f32
            * min(
                1.0 - self.controls_show_threshold_,
                self.controls_hide_threshold_ - min_ratio,
            );
        let frame_duration_ms = InMillisecondsF(client.CurrentFrameInterval()) as f32;
        animation_duration_ms = clamp(
            animation_duration_ms,
            frame_duration_ms,
            kShowHideMaxDurationMs as f32,
        );
        frame_duration_ms / animation_duration_ms
    }
    // Source: cc/input/browser_controls_offset_manager.h:149-158;
    //         cc/input/browser_controls_offset_manager.cc:623-647.
    // is_inertial=false is the source default argument; caller supplies it here.
    pub fn ScrollBy<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        pending_delta: &Vector2dF,
        is_inertial: bool,
    ) -> Vector2dF {
        if self.TopControlsHeight(client) == 0.0 && self.BottomControlsHeight(client) == 0.0 {
            return *pending_delta;
        }
        if self.pinch_gesture_active_ {
            return *pending_delta;
        }
        if (self.permitted_state_ == BrowserControlsState::kShown && pending_delta.y() > 0.0)
            || (self.permitted_state_ == BrowserControlsState::kHidden && pending_delta.y() < 0.0)
        {
            return *pending_delta;
        }
        if self.use_snap_animation_ {
            self.ScrollBySnap(client, pending_delta, is_inertial);
            return *pending_delta;
        }
        self.ScrollByPrecise(client, pending_delta)
    }
    // Source: cc/input/browser_controls_offset_manager.h:237;
    //         cc/input/browser_controls_offset_manager.cc:648-754.
    fn ScrollByPrecise<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        pending_delta: &Vector2dF,
    ) -> Vector2dF {
        // Scroll content to page top before expanding controls when requested.
        let viewport_offset_y = client.ViewportScrollOffset().y();
        if client.OnlyExpandTopControlsAtPageTop()
            && pending_delta.y() < 0.0
            && viewport_offset_y > 0.0
        {
            self.ResetBaseline(client);
            self.accumulated_scroll_delta_ = min(0.0, pending_delta.y() + viewport_offset_y);
        } else {
            self.accumulated_scroll_delta_ += pending_delta.y();
        }
        // Top controls take priority to stay visually in sync with web content.
        let base_on_top_controls = self.TopControlsHeight(client) != 0.0;
        let controls_height = if base_on_top_controls {
            self.TopControlsHeight(client)
        } else {
            self.BottomControlsHeight(client)
        };
        let old_top_offset = self.ContentTopOffset(client);
        let baseline_content_offset = if base_on_top_controls {
            self.baseline_top_content_offset_
        } else {
            self.baseline_bottom_content_offset_
        };
        // Equal normalized ratios reach independent min/max heights together.
        let shown_ratio = (baseline_content_offset - self.accumulated_scroll_delta_
            + self.unapplied_scroll_delta_)
            / controls_height;
        let min_ratio = if base_on_top_controls {
            self.TopControlsMinShownRatio(client)
        } else {
            self.BottomControlsMinShownRatio(client)
        };
        let mut normalized_shown_ratio = NormalizeShownRatio(shown_ratio, min_ratio);
        let max_shown_ratio_delta = self.MaximumShownRatioDeltaPerFrame(client, min_ratio);
        let current_normalized_shown_ratio = NormalizeShownRatio(
            if base_on_top_controls {
                self.TopControlsShownRatio(client)
            } else {
                self.BottomControlsShownRatio(client)
            },
            min_ratio,
        );
        let clamped_normalized_shown_ratio = clamp(
            normalized_shown_ratio,
            current_normalized_shown_ratio - max_shown_ratio_delta,
            current_normalized_shown_ratio + max_shown_ratio_delta,
        );
        if clamped_normalized_shown_ratio != normalized_shown_ratio {
            // Spread capped delta across later ScrollBy calls; discard a pending
            // remainder when the new remainder changes direction.
            let unapplied_scroll_delta = (normalized_shown_ratio - clamped_normalized_shown_ratio)
                * (1.0 - min_ratio)
                * controls_height;
            if unapplied_scroll_delta.is_sign_negative()
                != self.unapplied_scroll_delta_.is_sign_negative()
            {
                self.unapplied_scroll_delta_ = 0.0;
            }
            self.unapplied_scroll_delta_ += unapplied_scroll_delta;
        }
        normalized_shown_ratio = clamped_normalized_shown_ratio;
        // Evaluate source callback arguments before the exclusive client borrow.
        let top_shown_ratio = self.TopControlsMinShownRatio(client)
            + normalized_shown_ratio * (1.0 - self.TopControlsMinShownRatio(client));
        let bottom_shown_ratio = self.BottomControlsMinShownRatio(client)
            + normalized_shown_ratio * (1.0 - self.BottomControlsMinShownRatio(client));
        client.SetCurrentBrowserControlsShownRatio(top_shown_ratio, bottom_shown_ratio, self);
        if self.TopControlsShownRatio(client) == 1.0 && self.BottomControlsShownRatio(client) == 1.0
        {
            self.ResetBaseline(client);
            self.show_controls_when_scroll_completes_ = false;
        }
        self.ResetAnimations(client);
        // Do not allow page-refresh overscroll until controls become visible.
        if normalized_shown_ratio < 1.0 && pending_delta.y() < 0.0 && viewport_offset_y == 0.0 {
            return Vector2dF::default();
        }
        // Content counter-scroll is consumed by top controls; bottom-only
        // controls do not delay content scrolling.
        let applied_delta = Vector2dF::new(0.0, old_top_offset - self.ContentTopOffset(client));
        *pending_delta - applied_delta
    }
    // Source: cc/input/browser_controls_offset_manager.h:238;
    //         cc/input/browser_controls_offset_manager.cc:755-792.
    fn ScrollBySnap<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        pending_delta: &Vector2dF,
        is_inertial: bool,
    ) {
        self.accumulated_scroll_delta_ += pending_delta.y();
        let velocity_y = self.scroll_velocity_tracker_.CurrentVelocity().y();
        let current_y = pending_delta.y();
        if !(velocity_y >= 0.0 && current_y >= 0.0) && !(velocity_y <= 0.0 && current_y <= 0.0) {
            self.scroll_velocity_tracker_.Reset();
        }
        // Source's TimeTicks::Now is supplied by the injected host clock service.
        self.scroll_velocity_tracker_
            .AddSample(client.MonotonicTimeNow(), pending_delta);
        let direction = if pending_delta.y() >= 0.0 {
            AnimationDirection::kHidingControls
        } else {
            AnimationDirection::kShowingControls
        };
        if direction == AnimationDirection::kHidingControls
            && self.did_hide_this_scroll_
            && !is_inertial
        {
            return;
        }
        if self.scroll_velocity_tracker_.num_samples() < 2 {
            return;
        }
        self.SetupSnapAnimation(client, direction, pending_delta, is_inertial);
    }
    // Source: cc/input/browser_controls_offset_manager.h:215-223;
    //         cc/input/browser_controls_offset_manager.cc:793-947.
    fn SetupSnapAnimation<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        direction: AnimationDirection,
        scroll_delta: &Vector2dF,
        use_minimum_can_hide_region: bool,
    ) {
        assert!(
            direction == AnimationDirection::kHidingControls
                || direction == AnimationDirection::kShowingControls
        );
        debug_assert!(
            (direction == AnimationDirection::kShowingControls && scroll_delta.y() <= 0.0)
                || (direction == AnimationDirection::kHidingControls && scroll_delta.y() >= 0.0)
        );
        if self.HasAnimation() {
            return;
        }
        let base_on_top_controls = self.TopControlsHeight(client) != 0.0;
        let controls_animated_height = self.ControlsAnimatedHeight(client);
        if controls_animated_height == 0.0 {
            return;
        }
        debug_assert!(controls_animated_height > 0.0);
        let shown_ratio = if base_on_top_controls {
            self.TopControlsShownRatio(client)
        } else {
            self.BottomControlsShownRatio(client)
        };
        let min_shown_ratio = if base_on_top_controls {
            self.TopControlsMinShownRatio(client)
        } else {
            self.BottomControlsMinShownRatio(client)
        };
        let viewport_offset_y = client.ViewportScrollOffset().y();
        let slowness = clamp(
            (controls_animated_height / self.scroll_velocity_tracker_.CurrentVelocity().y().abs())
                / kShowHideMaxDurationMs as f32,
            0.0,
            1.0,
        );
        let trigger_threshold = self.SnapAnimationThreshold(client, slowness);
        let curve;
        // Four source-defined page regions: always-shown, top gap, can-hide,
        // bottom gap. Gaps prevent control counter-scroll from triggering an
        // immediate reverse animation; short pages may have no can-hide region.
        if direction == AnimationDirection::kHidingControls {
            let (can_hide_region_height_top, can_hide_region_height_bottom) = self
                .SnapAnimationCanHideRegionHeights(
                    client,
                    if use_minimum_can_hide_region {
                        0.0
                    } else {
                        slowness
                    },
                );
            if viewport_offset_y <= can_hide_region_height_top
                || viewport_offset_y
                    >= client.MaxViewportScrollOffsetY() - can_hide_region_height_bottom
                || self.accumulated_scroll_delta_ < trigger_threshold
            {
                return;
            }
            if shown_ratio == min_shown_ratio {
                return;
            }
            curve = TweenType::ACCEL_30_DECEL_LIN;
            self.did_hide_this_scroll_ = true;
        } else {
            // Showing is unrestricted per gesture, favoring visible controls
            // if the user rapidly alternates scroll direction.
            if viewport_offset_y + scroll_delta.y()
                > self.SnapAnimationAlwaysShownRegionHeight(client)
                && self.accumulated_scroll_delta_ > -trigger_threshold
            {
                return;
            }
            if shown_ratio == 1.0 {
                return;
            }
            curve = if viewport_offset_y > controls_animated_height {
                TweenType::ACCEL_LIN_DECEL_60
            } else {
                TweenType::LINEAR
            };
        }
        self.SetupAnimation(
            client,
            direction,
            Tween::IntValueBetween(
                slowness as f64,
                kShowHideMinDurationMs as i32,
                kShowHideMaxDurationMs as i32,
            ) as i64,
            curve,
        );
    }

    // Source: cc/input/browser_controls_offset_manager.h:159-160;
    //         cc/input/browser_controls_offset_manager.cc:948-1010.
    // The source default compensated delta is Vector2dF(); Rust makes it explicit.
    pub fn ScrollEnd<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        compensated_scroll_delta: &Vector2dF,
    ) {
        if self.pinch_gesture_active_ {
            return;
        }
        if self.use_snap_animation_ {
            // At finger-up, minimum can-hide region allows a second hide within
            // the sequence and reliable slow hides near the page top.
            let direction = if self.scroll_velocity_tracker_.CurrentVelocity().y() >= 0.0 {
                AnimationDirection::kHidingControls
            } else {
                AnimationDirection::kShowingControls
            };
            self.SetupSnapAnimation(client, direction, &Vector2dF::default(), true);
            self.scroll_velocity_tracker_.Reset();
            self.did_hide_this_scroll_ = false;
            return;
        }
        let compensated_scroll_delta_y = compensated_scroll_delta.y();
        if !self.show_controls_when_scroll_completes_ && compensated_scroll_delta_y < 0.0 {
            let controls_height = if self.TopControlsHeight(client) != 0.0 {
                self.TopControlsHeight(client)
            } else {
                self.BottomControlsHeight(client)
            };
            let controls_shown_ratio = if self.TopControlsHeight(client) != 0.0 {
                self.TopControlsShownRatio(client)
            } else {
                self.BottomControlsShownRatio(client)
            };
            let controls_min_shown_ratio = if self.TopControlsHeight(client) != 0.0 {
                self.TopControlsMinShownRatio(client)
            } else {
                self.BottomControlsMinShownRatio(client)
            };
            let normalized_shown_ratio_after_compensation = NormalizeShownRatio(
                controls_shown_ratio + compensated_scroll_delta_y.abs() / controls_height,
                controls_min_shown_ratio,
            );
            if normalized_shown_ratio_after_compensation >= self.controls_show_threshold_ {
                self.show_controls_when_scroll_completes_ = true;
            }
        }
        if self.show_controls_when_scroll_completes_ && self.TopControlsShownRatio(client) != 1.0 {
            self.SetupAnimation(
                client,
                AnimationDirection::kShowingControls,
                kShowHideMaxDurationMs,
                TweenType::LINEAR,
            );
            return;
        }
        self.StartAnimationIfNecessary(client);
    }
    // Source: cc/input/browser_controls_offset_manager.h:162-164;
    //         cc/input/browser_controls_offset_manager.cc:1011-1016.
    pub fn PinchBegin<C: BrowserControlsOffsetManagerClient + ?Sized>(&mut self, client: &mut C) {
        debug_assert!(!self.pinch_gesture_active_);
        self.pinch_gesture_active_ = true;
        self.StartAnimationIfNecessary(client);
    }
    // Source: cc/input/browser_controls_offset_manager.h:165;
    //         cc/input/browser_controls_offset_manager.cc:1017-1024.
    pub fn PinchEnd<C: BrowserControlsOffsetManagerClient + ?Sized>(&mut self, client: &mut C) {
        debug_assert!(self.pinch_gesture_active_);
        // Pinch scope is inside the scroll scope: restore expected scroll state.
        self.pinch_gesture_active_ = false;
        self.ScrollBegin(client);
    }
    // Source: cc/input/browser_controls_offset_manager.h:167;
    //         cc/input/browser_controls_offset_manager.cc:1025-1113.
    pub fn Animate<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        monotonic_time: TimeTicks,
    ) -> Vector2dF {
        if !self.HasAnimation() || !client.HaveRootScrollNode() {
            return Vector2dF::default();
        }
        if self.unapplied_scroll_delta_ != 0.0 {
            let unapplied_scroll_delta = self.unapplied_scroll_delta_;
            let applied_delta =
                self.ScrollBy(client, &Vector2dF::new(0.0, unapplied_scroll_delta), false);
            self.unapplied_scroll_delta_ = unapplied_scroll_delta - applied_delta.y();
            // Controls slide over content; do not counter-scroll the page.
            return Vector2dF::default();
        }
        let old_top_offset = self.ContentTopOffset(client);
        let old_bottom_offset = self.ContentBottomOffset(client);
        let mut new_top_ratio = self.top_controls_animation_.Tick(monotonic_time);
        if new_top_ratio.is_none() {
            new_top_ratio = Some(self.TopControlsShownRatio(client));
        }
        let mut new_bottom_ratio = self.bottom_controls_animation_.Tick(monotonic_time);
        if new_bottom_ratio.is_none() {
            new_bottom_ratio = Some(self.BottomControlsShownRatio(client));
        }
        let animation_completed = !self.top_controls_animation_.IsInitialized()
            && !self.bottom_controls_animation_.IsInitialized();
        client.SetCurrentBrowserControlsShownRatio(
            new_top_ratio.unwrap(),
            new_bottom_ratio.unwrap(),
            self,
        );
        let top_offset_delta = self.ContentTopOffset(client) - old_top_offset;
        let bottom_offset_delta = self.ContentBottomOffset(client) - old_bottom_offset;
        if self.use_snap_animation_
            && animation_completed
            && client.ViewportScrollOffset().y()
                <= self.SnapAnimationAlwaysShownRegionHeight(client)
        {
            self.SetupSnapAnimation(
                client,
                AnimationDirection::kShowingControls,
                &Vector2dF::default(),
                false,
            );
        }
        if self.top_min_height_change_in_progress_ {
            let range = self
                .top_min_height_offset_animation_range_
                .expect("top min-height animation range");
            self.top_controls_min_height_offset_ = clamp(
                self.top_controls_min_height_offset_ + top_offset_delta,
                range.0,
                range.1,
            );
            self.top_min_height_change_in_progress_ = self.top_controls_animation_.IsInitialized();
        }
        if self.bottom_min_height_change_in_progress_ {
            let range = self
                .bottom_min_height_offset_animation_range_
                .expect("bottom min-height animation range");
            self.bottom_controls_min_height_offset_ = clamp(
                self.bottom_controls_min_height_offset_ + bottom_offset_delta,
                range.0,
                range.1,
            );
            self.bottom_min_height_change_in_progress_ =
                self.bottom_controls_animation_.IsInitialized();
            // Shrinking min-height more than total height leaves a remainder;
            // reset it at the end to keep controls adjacent to the web content.
            if !self.bottom_min_height_change_in_progress_ {
                self.bottom_controls_min_height_offset_ = self.BottomControlsMinHeight(client);
            }
        }
        Vector2dF::new(0.0, top_offset_delta)
    }
    // Source: cc/input/browser_controls_offset_manager.h:107;
    //         cc/input/browser_controls_offset_manager.cc:1114-1119.
    pub fn HasAnimation(&self) -> bool {
        self.top_controls_animation_.IsInitialized()
            || self.bottom_controls_animation_.IsInitialized()
            || self.unapplied_scroll_delta_ != 0.0
    }
    // Source: cc/input/browser_controls_offset_manager.h:108-113.
    pub fn IsAnimatingToShowControls(&self) -> bool {
        self.top_controls_animation_.IsInitialized()
            && self.top_controls_animation_.Direction() == AnimationDirection::kShowingControls
    }
    // Source: cc/input/browser_controls_offset_manager.h:235;
    //         cc/input/browser_controls_offset_manager.cc:1120-1127.
    fn MaybeRecordHasExistingAnimationHistogram<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &self,
        client: &mut C,
    ) {
        if !self.in_viz_process_ && self.use_snap_animation_ {
            client.EmitDiagnostic(BrowserControlsDiagnostic::HasExistingAnimation(
                self.HasAnimation(),
            ));
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:175;
    //         cc/input/browser_controls_offset_manager.cc:1128-1155.
    pub fn ResetAnimations<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.MaybeRecordHasExistingAnimationHistogram(client);
        let top_ratio = self.top_controls_animation_.Reset();
        let bottom_ratio = self.bottom_controls_animation_.Reset();
        if top_ratio.is_some() || bottom_ratio.is_some() {
            let resolved_top = top_ratio.unwrap_or_else(|| self.TopControlsShownRatio(client));
            let resolved_bottom =
                bottom_ratio.unwrap_or_else(|| self.BottomControlsShownRatio(client));
            client.SetCurrentBrowserControlsShownRatio(resolved_top, resolved_bottom, self);
            if self.top_min_height_change_in_progress_ {
                debug_assert!(top_ratio.is_some());
                self.top_controls_min_height_offset_ = self.TopControlsMinHeight(client);
            }
            if self.bottom_min_height_change_in_progress_ {
                debug_assert!(bottom_ratio.is_some());
                self.bottom_controls_min_height_offset_ = self.BottomControlsMinHeight(client);
            }
        }
        self.top_min_height_change_in_progress_ = false;
        self.bottom_min_height_change_in_progress_ = false;
        self.top_min_height_offset_animation_range_ = None;
        self.bottom_min_height_offset_animation_range_ = None;
    }
    // Source: cc/input/browser_controls_offset_manager.h:211-213;
    //         cc/input/browser_controls_offset_manager.cc:1156-1202.
    fn SetupAnimation<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
        direction: AnimationDirection,
        duration_ms: i64,
        tween_type: TweenType,
    ) {
        debug_assert!(direction != AnimationDirection::kNoAnimation);
        debug_assert!(
            direction != AnimationDirection::kHidingControls
                || self.TopControlsShownRatio(client) > 0.0
                || self.BottomControlsShownRatio(client) > 0.0
        );
        debug_assert!(
            direction != AnimationDirection::kShowingControls
                || self.TopControlsShownRatio(client) < 1.0
                || self.BottomControlsShownRatio(client) < 1.0
        );
        if self.top_controls_animation_.IsInitialized()
            && self.top_controls_animation_.Direction() == direction
            && self.bottom_controls_animation_.IsInitialized()
            && self.bottom_controls_animation_.Direction() == direction
        {
            return;
        }
        self.MaybeRecordHasExistingAnimationHistogram(client);
        if self.TopControlsHeight(client) == 0.0 && self.BottomControlsHeight(client) == 0.0 {
            let ratio = if direction == AnimationDirection::kHidingControls {
                0.0
            } else {
                1.0
            };
            client.SetCurrentBrowserControlsShownRatio(ratio, ratio, self);
            return;
        }
        // Overshoot stop ratios keep speed independent of distance to endpoint.
        let max_stop_ratio = if direction == AnimationDirection::kShowingControls {
            1.0
        } else {
            -1.0
        };
        let top_start_ratio = self.TopControlsShownRatio(client);
        let top_stop_ratio = top_start_ratio + max_stop_ratio;
        self.top_controls_animation_.Initialize(
            direction,
            top_start_ratio,
            top_stop_ratio,
            duration_ms,
            false,
            tween_type,
        );
        self.top_controls_animation_
            .SetBounds(self.TopControlsMinShownRatio(client), 1.0);
        let bottom_start_ratio = self.BottomControlsShownRatio(client);
        let bottom_stop_ratio = bottom_start_ratio + max_stop_ratio;
        self.bottom_controls_animation_.Initialize(
            direction,
            bottom_start_ratio,
            bottom_stop_ratio,
            duration_ms,
            false,
            tween_type,
        );
        self.bottom_controls_animation_
            .SetBounds(self.BottomControlsMinShownRatio(client), 1.0);
        self.unapplied_scroll_delta_ = 0.0;
        client.DidChangeBrowserControlsPosition(self);
    }
    // Source: cc/input/browser_controls_offset_manager.h:224;
    //         cc/input/browser_controls_offset_manager.cc:1203-1231.
    fn StartAnimationIfNecessary<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if (self.TopControlsShownRatio(client) == self.TopControlsMinShownRatio(client)
            || self.TopControlsShownRatio(client) == 1.0)
            && (self.BottomControlsShownRatio(client) == self.BottomControlsMinShownRatio(client)
                || self.BottomControlsShownRatio(client) == 1.0)
        {
            return;
        }
        let normalized_top_ratio = (self.TopControlsShownRatio(client)
            - self.TopControlsMinShownRatio(client))
            / (1.0 - self.TopControlsMinShownRatio(client));
        if normalized_top_ratio >= 1.0 - self.controls_hide_threshold_ {
            self.SetupAnimation(
                client,
                AnimationDirection::kShowingControls,
                kShowHideMaxDurationMs,
                TweenType::LINEAR,
            );
        } else if normalized_top_ratio <= self.controls_show_threshold_ {
            self.SetupAnimation(
                client,
                AnimationDirection::kHidingControls,
                kShowHideMaxDurationMs,
                TweenType::LINEAR,
            );
        } else {
            self.SetupAnimation(
                client,
                if self.accumulated_scroll_delta_ <= 0.0 {
                    AnimationDirection::kShowingControls
                } else {
                    AnimationDirection::kHidingControls
                },
                kShowHideMaxDurationMs,
                TweenType::LINEAR,
            );
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:225;
    //         cc/input/browser_controls_offset_manager.cc:1232-1238.
    fn ResetBaseline<C: BrowserControlsOffsetManagerQueries + ?Sized>(&mut self, client: &C) {
        self.accumulated_scroll_delta_ = 0.0;
        self.unapplied_scroll_delta_ = 0.0;
        self.baseline_top_content_offset_ = self.ContentTopOffset(client);
        self.baseline_bottom_content_offset_ = self.ContentBottomOffset(client);
    }
    // Source: cc/input/browser_controls_offset_manager.h:229-231;
    //         cc/input/browser_controls_offset_manager.cc:1239-1250.
    // Source method reads no manager state; static Rust form permits splitting
    // an exclusive animation field borrow without aliasing the whole manager.
    fn InitAnimationForHeightChange(animation: &mut Animation, start_ratio: f32, stop_ratio: f32) {
        let direction = if start_ratio < stop_ratio {
            AnimationDirection::kShowingControls
        } else {
            AnimationDirection::kHidingControls
        };
        animation.Initialize(
            direction,
            start_ratio,
            stop_ratio,
            kHeightChangeDurationMs,
            true,
            TweenType::LINEAR,
        );
    }
    // Source: cc/input/browser_controls_offset_manager.h:226;
    //         cc/input/browser_controls_offset_manager.cc:1251-1257.
    fn OldTopControlsMinShownRatio(&self) -> f32 {
        if self.old_browser_controls_params_.top_controls_height != 0.0 {
            self.old_browser_controls_params_.top_controls_min_height
                / self.old_browser_controls_params_.top_controls_height
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:227;
    //         cc/input/browser_controls_offset_manager.cc:1258-1264.
    fn OldBottomControlsMinShownRatio(&self) -> f32 {
        if self.old_browser_controls_params_.bottom_controls_height != 0.0 {
            self.old_browser_controls_params_.bottom_controls_min_height
                / self.old_browser_controls_params_.bottom_controls_height
        } else {
            0.0
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:228;
    //         cc/input/browser_controls_offset_manager.cc:1265-1274.
    fn UpdateOldBrowserControlsParams<C: BrowserControlsOffsetManagerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.UpdateOldBrowserControlsParamsFromValues(
            self.TopControlsHeight(client),
            self.TopControlsMinHeight(client),
            self.BottomControlsHeight(client),
            self.BottomControlsMinHeight(client),
        );
    }
    // Source: cc/input/browser_controls_offset_manager.cc:1265-1274.
    // Native HostImpl getters are pure field reads; share the assignments.
    fn UpdateOldBrowserControlsParamsFromValues(
        &mut self,
        top: f32,
        top_min: f32,
        bottom: f32,
        bottom_min: f32,
    ) {
        self.old_browser_controls_params_.top_controls_height = top;
        self.old_browser_controls_params_.top_controls_min_height = top_min;
        self.old_browser_controls_params_.bottom_controls_height = bottom;
        self.old_browser_controls_params_.bottom_controls_min_height = bottom_min;
    }
    // Source: cc/input/browser_controls_offset_manager.h:232;
    //         cc/input/browser_controls_offset_manager.cc:1275-1281.
    fn SetTopMinHeightOffsetAnimationRange(&mut self, from: f32, to: f32) {
        self.top_min_height_offset_animation_range_ = Some((min(from, to), max(from, to)));
    }
    // Source: cc/input/browser_controls_offset_manager.h:233;
    //         cc/input/browser_controls_offset_manager.cc:1282-1288.
    fn SetBottomMinHeightOffsetAnimationRange(&mut self, from: f32, to: f32) {
        self.bottom_min_height_offset_animation_range_ = Some((min(from, to), max(from, to)));
    }
    // Source: cc/input/browser_controls_offset_manager.h:234;
    //         cc/input/browser_controls_offset_manager.cc:1289-1295.
    fn IsAnimatingHeightChange(&self) -> bool {
        (self.top_controls_animation_.IsInitialized()
            && self.top_controls_animation_.jump_to_end_on_reset())
            || (self.bottom_controls_animation_.IsInitialized()
                && self.bottom_controls_animation_.jump_to_end_on_reset())
    }
    // Source: cc/input/browser_controls_offset_manager.h:169-173;
    //         cc/input/browser_controls_offset_manager.cc:1296-1322.
    pub fn PredictViewportBoundsDelta<C: BrowserControlsOffsetManagerQueries + ?Sized>(
        &self,
        client: &C,
        current_bounds_delta: f64,
        scroll_distance: Vector2dF,
    ) -> f64 {
        let mut adjustment = current_bounds_delta;
        if scroll_distance.y() > 0.0 && adjustment > 0.0 {
            if self.TopControlsShownRatio(client) < 1.0 {
                adjustment += self.ContentTopOffset(client) as f64;
            }
            if self.BottomControlsShownRatio(client) < 1.0 {
                adjustment += self.ContentBottomOffset(client) as f64;
            }
        }
        if scroll_distance.y() < 0.0 && adjustment < 0.0 {
            if self.TopControlsShownRatio(client) > 0.0 {
                adjustment -=
                    (self.TopControlsHeight(client) - self.ContentTopOffset(client)) as f64;
            }
            if self.BottomControlsShownRatio(client) > 0.0 {
                adjustment -=
                    (self.BottomControlsHeight(client) - self.ContentBottomOffset(client)) as f64;
            }
        }
        adjustment
    }
}

// Source: cc/input/browser_controls_offset_manager.h:314-318,349-375;
//         cc/input/browser_controls_offset_manager.cc:1323-1326.
// C++ nested private class becomes a private type in its parent's Rust module.
struct Animation {
    started_: bool,
    initialized_: bool,
    direction_: AnimationDirection,
    start_time_: TimeTicks,
    stop_time_: TimeTicks,
    duration_: TimeDelta,
    start_value_: f32,
    stop_value_: f32,
    min_value_: f32,
    max_value_: f32,
    jump_to_end_on_reset_: bool,
    tween_type_: TweenType,
}
impl Animation {
    // Source: cc/input/browser_controls_offset_manager.h:317,352-374;
    //         cc/input/browser_controls_offset_manager.cc:1325.
    fn new() -> Self {
        Self {
            started_: false,
            initialized_: false,
            direction_: AnimationDirection::kNoAnimation,
            start_time_: TimeTicks::default(),
            stop_time_: TimeTicks::default(),
            duration_: TimeDelta::default(),
            start_value_: 0.0,
            stop_value_: 0.0,
            min_value_: 0.0,
            max_value_: 1.0,
            jump_to_end_on_reset_: false,
            tween_type_: TweenType::LINEAR,
        }
    }
    // Source: cc/input/browser_controls_offset_manager.h:319-321.
    fn IsInitialized(&self) -> bool {
        self.initialized_
    }
    // Source: cc/input/browser_controls_offset_manager.h:322.
    fn Direction(&self) -> AnimationDirection {
        self.direction_
    }
    // Source: cc/input/browser_controls_offset_manager.h:323-328;
    //         cc/input/browser_controls_offset_manager.cc:1327-1344.
    fn Initialize(
        &mut self,
        direction: AnimationDirection,
        start_value: f32,
        stop_value: f32,
        duration: i64,
        jump_to_end_on_reset: bool,
        tween_type: TweenType,
    ) {
        self.direction_ = direction;
        self.start_value_ = start_value;
        self.stop_value_ = stop_value;
        self.duration_ = TimeDelta::Milliseconds(duration);
        self.initialized_ = true;
        self.jump_to_end_on_reset_ = jump_to_end_on_reset;
        self.tween_type_ = tween_type;
        self.SetBounds(
            min(self.start_value_, self.stop_value_),
            max(self.start_value_, self.stop_value_),
        );
    }
    // Source: cc/input/browser_controls_offset_manager.h:329-331;
    //         cc/input/browser_controls_offset_manager.cc:1345-1367.
    fn Tick(&mut self, monotonic_time: TimeTicks) -> Option<f32> {
        if !self.IsInitialized() {
            return None;
        }
        if !self.started_ {
            self.start_time_ = monotonic_time;
            self.stop_time_ = self.start_time_ + self.duration_;
            self.started_ = true;
        }
        let mut value = Tween::ClampedFloatValueBetween(
            monotonic_time,
            self.start_time_,
            self.start_value_,
            self.stop_time_,
            self.stop_value_,
            self.tween_type_,
        );
        if self.IsComplete(value) {
            value = self.FinalValue();
            self.Reset();
        }
        Some(value)
    }
    // Source: cc/input/browser_controls_offset_manager.h:332-333;
    //         cc/input/browser_controls_offset_manager.cc:1368-1372.
    fn SetBounds(&mut self, min: f32, max: f32) {
        self.min_value_ = min;
        self.max_value_ = max;
    }
    // Source: cc/input/browser_controls_offset_manager.h:334-337;
    //         cc/input/browser_controls_offset_manager.cc:1373-1393.
    fn Reset(&mut self) -> Option<f32> {
        let ret = if self.jump_to_end_on_reset_ {
            Some(clamp(self.stop_value_, self.min_value_, self.max_value_))
        } else {
            None
        };
        self.started_ = false;
        self.initialized_ = false;
        self.start_time_ = TimeTicks::default();
        self.start_value_ = 0.0;
        self.stop_time_ = TimeTicks::default();
        self.stop_value_ = 0.0;
        self.direction_ = AnimationDirection::kNoAnimation;
        self.duration_ = TimeDelta::default();
        self.min_value_ = 0.0;
        self.max_value_ = 1.0;
        self.jump_to_end_on_reset_ = false;
        ret
    }
    // Source: cc/input/browser_controls_offset_manager.h:350;
    //         cc/input/browser_controls_offset_manager.cc:1394-1400.
    fn IsComplete(&self, value: f32) -> bool {
        (self.direction_ == AnimationDirection::kShowingControls
            && (value >= self.stop_value_ || value >= self.max_value_))
            || (self.direction_ == AnimationDirection::kHidingControls
                && (value <= self.stop_value_ || value <= self.min_value_))
    }
    // Source: cc/input/browser_controls_offset_manager.h:338-341;
    //         cc/input/browser_controls_offset_manager.cc:1401-1404.
    fn FinalValue(&self) -> f32 {
        clamp(self.stop_value_, self.min_value_, self.max_value_)
    }
    // Source: cc/input/browser_controls_offset_manager.h:342-344.
    fn min_value(&self) -> f32 {
        self.min_value_
    }
    // Source: cc/input/browser_controls_offset_manager.h:345-346.
    fn max_value(&self) -> f32 {
        self.max_value_
    }
    // Source: cc/input/browser_controls_offset_manager.h:347-348.
    fn jump_to_end_on_reset(&self) -> bool {
        self.jump_to_end_on_reset_
    }
}
