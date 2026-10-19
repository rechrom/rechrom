// Translation status: complete
// Source: cc/input/scrollbar.h:1-134
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use crate::adapter::{
    geometry::{Rect, Size},
    scrollbar::{Color4f, Insets},
    time::TimeDelta,
};

// Source: cc/input/scrollbar.h:15-27
pub const kAutoscrollMultiplier: f32 = 20.;
pub const kInitialAutoscrollTimerDelay: TimeDelta = TimeDelta::Milliseconds(250);
pub const kOffSideMultiplier: i32 = 8;
pub const kDefaultWinScrollbarThickness: i32 = 17;

// Source: cc/input/scrollbar.h:31
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    kHorizontal,
    kVertical,
}
// Source: cc/input/scrollbar.h:33-40
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarPart {
    kThumb,
    kBackButton,
    kForwardButton,
    kBackTrack,
    kForwardTrack,
    kNoPart,
}

// Source: cc/input/scrollbar.h:42-130
// The caller retains this source-refcounted scrollbar through its typed handle.
// Canvas is owned by paint; the compositor does not create a paint backend.
pub trait Scrollbar<Canvas: ?Sized> {
    // Source: cc/input/scrollbar.h:46-46
    fn IsSame(&self, other: &dyn Scrollbar<Canvas>) -> bool;
    // Source: cc/input/scrollbar.h:48-48
    fn Orientation(&self) -> ScrollbarOrientation;
    // Source: cc/input/scrollbar.h:49-49
    fn IsLeftSideVerticalScrollbar(&self) -> bool;
    // Source: cc/input/scrollbar.h:50-50
    fn IsSolidColor(&self) -> bool;
    // Source: cc/input/scrollbar.h:51-51
    fn IsOverlay(&self) -> bool;
    // Source: cc/input/scrollbar.h:52-52
    fn IsRunningWebTest(&self) -> bool;
    // Source: cc/input/scrollbar.h:53-53
    fn IsFluentOverlayScrollbarMinimalMode(&self) -> bool;
    // Source: cc/input/scrollbar.h:54-54
    fn HasThumb(&self) -> bool;
    // Source: cc/input/scrollbar.h:55-55
    fn SupportsDragSnapBack(&self) -> bool;
    // Source: cc/input/scrollbar.h:56-56
    fn JumpOnTrackClick(&self) -> bool;
    // Source: cc/input/scrollbar.h:57-57
    fn IsOpaque(&self) -> bool;
    // Source: cc/input/scrollbar.h:58-58
    fn MinimumThumbLength(&self) -> i32;
    // Source: cc/input/scrollbar.h:64-64
    fn ThumbRect(&self) -> Rect;
    // Source: cc/input/scrollbar.h:65-65
    fn TrackRect(&self) -> Rect;
    // Source: cc/input/scrollbar.h:66-66
    fn BackButtonRect(&self) -> Rect;
    // Source: cc/input/scrollbar.h:67-67
    fn ForwardButtonRect(&self) -> Rect;
    // Source: cc/input/scrollbar.h:69-69
    fn Opacity(&self) -> f32;
    // Source: cc/input/scrollbar.h:70-70
    fn HasTickmarks(&self) -> bool;
    // Source: cc/input/scrollbar.h:72-72
    fn ThumbNeedsRepaint(&self) -> bool;
    // Source: cc/input/scrollbar.h:73-73
    fn ClearThumbNeedsRepaint(&mut self);
    // Source: cc/input/scrollbar.h:74-74
    fn TrackAndButtonsNeedRepaint(&self) -> bool;
    // Source: cc/input/scrollbar.h:81-81
    fn PaintThumb(&mut self, canvas: &mut Canvas, rect: &Rect);
    // Source: cc/input/scrollbar.h:84-85
    fn PaintTrackAndButtons(&mut self, canvas: &mut Canvas, rect: &Rect);
    // Source: cc/input/scrollbar.h:90-90
    fn ThumbColor(&self) -> Color4f;
    // Source: cc/input/scrollbar.h:96-96
    fn NeedsUpdateDisplay(&self) -> bool;
    // Source: cc/input/scrollbar.h:97-97
    fn ClearNeedsUpdateDisplay(&mut self);
    // Source: cc/input/scrollbar.h:102-102
    fn UsesNinePatchThumbResource(&self) -> bool;
    // Source: cc/input/scrollbar.h:103-103
    fn NinePatchThumbCanvasSize(&self) -> Size;
    // Source: cc/input/scrollbar.h:104-104
    fn NinePatchThumbAperture(&self) -> Rect;
    // Source: cc/input/scrollbar.h:108-108
    fn UsesSolidColorThumb(&self) -> bool;
    // Source: cc/input/scrollbar.h:110-110
    fn SolidColorThumbInsets(&self) -> Insets;
    // Source: cc/input/scrollbar.h:118-118
    fn UsesNinePatchTrackAndButtonsResource(&self) -> bool;
    // Source: cc/input/scrollbar.h:119-119
    fn NinePatchTrackAndButtonsCanvasSize(&self, scale: f32) -> Size;
    // Source: cc/input/scrollbar.h:120-120
    fn NinePatchTrackAndButtonsAperture(&self, scale: f32) -> Rect;
    // Source: cc/input/scrollbar.h:124-125
    fn ShrinkMainThreadedMinimalModeThumbRect(&self, rect: &mut Rect) -> Rect;
}
