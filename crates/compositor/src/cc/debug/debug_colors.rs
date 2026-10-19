// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/debug/debug_colors.h:1-16,139-141; cc/debug/debug_colors.cc:1-11,377.
// Modules and an uninstantiable namespace type express includes/export/namespace.
use crate::adapter::layer_tree_impl::Color4f;
pub enum DebugColors {}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:64.
    pub const kFadeSteps: i32 = 50;
}
// Source: cc/debug/debug_colors.cc:13-15.
fn Scale(width: f32, device_scale_factor: f32) -> f32 {
    return width * device_scale_factor;
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:18.
    // Source: cc/debug/debug_colors.cc:20-22.
    pub fn TiledContentLayerBorderColor() -> Color4f {
        return Color4f {
            red: 1.0f32,
            green: 0.5f32,
            blue: 0.0f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:19.
    // Source: cc/debug/debug_colors.cc:23-25.
    pub fn TiledContentLayerBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:21.
    // Source: cc/debug/debug_colors.cc:28-30.
    pub fn ImageLayerBorderColor() -> Color4f {
        return Color4f {
            red: 0.5f32,
            green: 0.5f32,
            blue: 0.0f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:22.
    // Source: cc/debug/debug_colors.cc:31-33.
    pub fn ImageLayerBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:24.
    // Source: cc/debug/debug_colors.cc:36-38.
    pub fn ContentLayerBorderColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.5f32,
            blue: 32.0f32 / 255.0f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:25.
    // Source: cc/debug/debug_colors.cc:39-41.
    pub fn ContentLayerBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:27.
    // Source: cc/debug/debug_colors.cc:44-46.
    pub fn ContainerLayerBorderColor() -> Color4f {
        return Color4f {
            red: 1.0f32,
            green: 1.0f32,
            blue: 0.0f32,
            alpha: 0.75f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:28.
    // Source: cc/debug/debug_colors.cc:47-49.
    pub fn ContainerLayerBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:30.
    // Source: cc/debug/debug_colors.cc:52-54.
    pub fn SurfaceLayerBorderColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 1.0f32,
            blue: 136.0f32 / 255.0f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:31.
    // Source: cc/debug/debug_colors.cc:55-57.
    pub fn SurfaceLayerBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:33.
    // Source: cc/debug/debug_colors.cc:60-62.
    pub fn SurfaceBorderColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.0f32,
            blue: 1.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:34.
    // Source: cc/debug/debug_colors.cc:63-65.
    pub fn SurfaceBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:36.
    // Source: cc/debug/debug_colors.cc:70-72.
    pub fn HighResTileBorderColor() -> Color4f {
        return Color4f {
            red: 80.0f32 / 255.0f32,
            green: 200.0f32 / 255.0f32,
            blue: 200.0f32 / 255.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:37.
    // Source: cc/debug/debug_colors.cc:73-75.
    pub fn HighResTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(1., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:39.
    // Source: cc/debug/debug_colors.cc:78-80.
    pub fn AboveHighResTileBorderColor() -> Color4f {
        return Color4f {
            red: 239.0f32 / 255.0f32,
            green: 231.0f32 / 255.0f32,
            blue: 20.0f32 / 255.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:40.
    // Source: cc/debug/debug_colors.cc:81-83.
    pub fn AboveHighResTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:42.
    // Source: cc/debug/debug_colors.cc:86-88.
    pub fn BelowHighResTileBorderColor() -> Color4f {
        return Color4f {
            red: 93.0f32 / 255.0f32,
            green: 186.0f32 / 255.0f32,
            blue: 18.0f32 / 255.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:43.
    // Source: cc/debug/debug_colors.cc:89-91.
    pub fn BelowHighResTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:45.
    // Source: cc/debug/debug_colors.cc:94-96.
    pub fn MissingTileBorderColor() -> Color4f {
        return Color4f {
            red: 0.25f32,
            green: 0.25f32,
            blue: 0.0f32,
            alpha: 0.25f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:46.
    // Source: cc/debug/debug_colors.cc:97-99.
    pub fn MissingTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(1., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:48.
    // Source: cc/debug/debug_colors.cc:102-104.
    pub fn SolidColorTileBorderColor() -> Color4f {
        return Color4f {
            red: 0.5f32,
            green: 0.5f32,
            blue: 0.5f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:49.
    // Source: cc/debug/debug_colors.cc:105-107.
    pub fn SolidColorTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(1., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:51.
    // Source: cc/debug/debug_colors.cc:110-112.
    pub fn OOMTileBorderColor() -> Color4f {
        return Color4f {
            red: 1.0f32,
            green: 0.0f32,
            blue: 0.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:52.
    // Source: cc/debug/debug_colors.cc:113-115.
    pub fn OOMTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(1., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:54.
    // Source: cc/debug/debug_colors.cc:118-120.
    pub fn DirectPictureBorderColor() -> Color4f {
        return Color4f {
            red: 127.0f32 / 255.0f32,
            green: 1.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:55.
    // Source: cc/debug/debug_colors.cc:121-123.
    pub fn DirectPictureBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(1., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:123.
    // Source: cc/debug/debug_colors.cc:126-140.
    pub fn TintCompositedContentColorTransformMatrix() -> &'static [f32] {
        static COLOR_TRANSFORM: [f32; 16] = [
            1., 0., 0., 0., 0.3, 0.7, 0., 0., 0.3, 0., 0.7, 0., 0., 0., 0., 1.,
        ];
        return &COLOR_TRANSFORM;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:57.
    // Source: cc/debug/debug_colors.cc:143-145.
    pub fn CompressedTileBorderColor() -> Color4f {
        return Color4f {
            red: 20.0f32 / 255.0f32,
            green: 20.0f32 / 255.0f32,
            blue: 240.0f32 / 255.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:58.
    // Source: cc/debug/debug_colors.cc:146-148.
    pub fn CompressedTileBorderWidth(device_scale_factor: f32) -> i32 {
        return Scale(2., device_scale_factor) as i32;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:60.
    // Source: cc/debug/debug_colors.cc:153-155.
    pub fn DefaultCheckerboardColor() -> Color4f {
        return Color4f {
            red: 241.0f32 / 255.0f32,
            green: 241.0f32 / 255.0f32,
            blue: 241.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:62.
    // Source: cc/debug/debug_colors.cc:158-160.
    pub fn InvalidatedTileCheckerboardColor() -> Color4f {
        return Color4f {
            red: 0.5f32,
            green: 200.0f32 / 255.0f32,
            blue: 245.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:61.
    // Source: cc/debug/debug_colors.cc:163-165.
    pub fn EvictedTileCheckerboardColor() -> Color4f {
        return Color4f {
            red: 1.0f32,
            green: 200.0f32 / 255.0f32,
            blue: 200.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
// Source: cc/debug/debug_colors.cc:169-174.
fn FadedGreen(initial_value: i32, step: i32) -> Color4f {
    debug_assert!(step >= 0);
    debug_assert!(step <= DebugColors::kFadeSteps);
    let value = step * initial_value / DebugColors::kFadeSteps;
    return Color4f {
        red: 0.0f32,
        green: 195.0f32 / 255.0f32,
        blue: 0.0f32,
        alpha: value as f32 / 255.0f32,
    };
}
// Source: cc/debug/debug_colors.cc:176-181.
fn FadedOrange(initial_value: i32, step: i32) -> Color4f {
    debug_assert!(step >= 0);
    debug_assert!(step <= DebugColors::kFadeSteps);
    let value = step * initial_value / DebugColors::kFadeSteps;
    return Color4f {
        red: 1.0f32,
        green: 140.0f32 / 255.0f32,
        blue: 0.0f32,
        alpha: value as f32 / 255.0f32,
    };
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:65.
    // Source: cc/debug/debug_colors.cc:184-186.
    pub fn PaintRectBorderColor(step: i32) -> Color4f {
        return FadedGreen(255, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:66.
    // Source: cc/debug/debug_colors.cc:187-187.
    pub fn PaintRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:67.
    // Source: cc/debug/debug_colors.cc:188-190.
    pub fn PaintRectFillColor(step: i32) -> Color4f {
        return FadedGreen(60, step);
    }
}
// Source: cc/debug/debug_colors.cc:192-197.
fn FadedBlue(initial_value: i32, step: i32) -> Color4f {
    debug_assert!(step >= 0);
    debug_assert!(step <= DebugColors::kFadeSteps);
    let value = step * initial_value / DebugColors::kFadeSteps;
    return Color4f {
        red: 0.0f32,
        green: 0.0f32,
        blue: 1.0f32,
        alpha: value as f32 / 255.0f32,
    };
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:69.
    // Source: cc/debug/debug_colors.cc:199-201.
    pub fn LayoutShiftRectBorderColor(step: i32) -> Color4f {
        return FadedBlue(255, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:70.
    // Source: cc/debug/debug_colors.cc:202-206.
    pub fn LayoutShiftRectBorderWidth() -> i32 {
        return 0;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:71.
    // Source: cc/debug/debug_colors.cc:207-209.
    pub fn LayoutShiftRectFillColor(step: i32) -> Color4f {
        return FadedBlue(60, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:73.
    // Source: cc/debug/debug_colors.cc:211-213.
    pub fn InteractionContentfulPaintRectBorderColor(step: i32) -> Color4f {
        return FadedOrange(255, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:74.
    // Source: cc/debug/debug_colors.cc:214-216.
    pub fn InteractionContentfulPaintRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:75.
    // Source: cc/debug/debug_colors.cc:217-219.
    pub fn InteractionContentfulPaintRectFillColor(step: i32) -> Color4f {
        return FadedOrange(60, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:77.
    // Source: cc/debug/debug_colors.cc:221-223.
    pub fn NavigationContentfulPaintRectBorderColor(step: i32) -> Color4f {
        return FadedGreen(255, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:78.
    // Source: cc/debug/debug_colors.cc:224-226.
    pub fn NavigationContentfulPaintRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:79.
    // Source: cc/debug/debug_colors.cc:227-229.
    pub fn NavigationContentfulPaintRectFillColor(step: i32) -> Color4f {
        return FadedGreen(60, step);
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:81.
    // Source: cc/debug/debug_colors.cc:232-234.
    pub fn PropertyChangedRectBorderColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.0f32,
            blue: 1.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:82.
    // Source: cc/debug/debug_colors.cc:235-235.
    pub fn PropertyChangedRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:83.
    // Source: cc/debug/debug_colors.cc:236-238.
    pub fn PropertyChangedRectFillColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.0f32,
            blue: 1.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:85.
    // Source: cc/debug/debug_colors.cc:241-243.
    pub fn SurfaceDamageRectBorderColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:86.
    // Source: cc/debug/debug_colors.cc:244-244.
    pub fn SurfaceDamageRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:87.
    // Source: cc/debug/debug_colors.cc:245-247.
    pub fn SurfaceDamageRectFillColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:89.
    // Source: cc/debug/debug_colors.cc:250-252.
    pub fn ScreenSpaceLayerRectBorderColor() -> Color4f {
        return Color4f {
            red: 100.0f32 / 255.0f32,
            green: 200.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:90.
    // Source: cc/debug/debug_colors.cc:253-253.
    pub fn ScreenSpaceLayerRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:91.
    // Source: cc/debug/debug_colors.cc:254-256.
    pub fn ScreenSpaceLayerRectFillColor() -> Color4f {
        return Color4f {
            red: 100.0f32 / 255.0f32,
            green: 200.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:93.
    // Source: cc/debug/debug_colors.cc:259-261.
    pub fn TouchEventHandlerRectBorderColor() -> Color4f {
        return Color4f {
            red: 239.0f32 / 255.0f32,
            green: 229.0f32 / 255.0f32,
            blue: 60.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:94.
    // Source: cc/debug/debug_colors.cc:262-262.
    pub fn TouchEventHandlerRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:95.
    // Source: cc/debug/debug_colors.cc:263-265.
    pub fn TouchEventHandlerRectFillColor() -> Color4f {
        return Color4f {
            red: 239.0f32 / 255.0f32,
            green: 229.0f32 / 255.0f32,
            blue: 60.0f32 / 255.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:97.
    // Source: cc/debug/debug_colors.cc:268-270.
    pub fn WheelEventHandlerRectBorderColor() -> Color4f {
        return Color4f {
            red: 189.0f32 / 255.0f32,
            green: 209.0f32 / 255.0f32,
            blue: 57.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:98.
    // Source: cc/debug/debug_colors.cc:271-271.
    pub fn WheelEventHandlerRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:99.
    // Source: cc/debug/debug_colors.cc:272-274.
    pub fn WheelEventHandlerRectFillColor() -> Color4f {
        return Color4f {
            red: 189.0f32 / 255.0f32,
            green: 209.0f32 / 255.0f32,
            blue: 57.0f32 / 255.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:101.
    // Source: cc/debug/debug_colors.cc:277-279.
    pub fn ScrollEventHandlerRectBorderColor() -> Color4f {
        return Color4f {
            red: 24.0f32 / 255.0f32,
            green: 167.0f32 / 255.0f32,
            blue: 181.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:102.
    // Source: cc/debug/debug_colors.cc:280-280.
    pub fn ScrollEventHandlerRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:103.
    // Source: cc/debug/debug_colors.cc:281-283.
    pub fn ScrollEventHandlerRectFillColor() -> Color4f {
        return Color4f {
            red: 24.0f32 / 255.0f32,
            green: 167.0f32 / 255.0f32,
            blue: 181.0f32 / 255.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:105.
    // Source: cc/debug/debug_colors.cc:286-288.
    pub fn MainThreadScrollHitTestRectBorderColor() -> Color4f {
        return Color4f {
            red: 238.0f32 / 255.0f32,
            green: 163.0f32 / 255.0f32,
            blue: 59.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:106.
    // Source: cc/debug/debug_colors.cc:289-291.
    pub fn MainThreadScrollHitTestRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:107.
    // Source: cc/debug/debug_colors.cc:292-294.
    pub fn MainThreadScrollHitTestRectFillColor() -> Color4f {
        return Color4f {
            red: 238.0f32 / 255.0f32,
            green: 163.0f32 / 255.0f32,
            blue: 59.0f32 / 255.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:109.
    // Source: cc/debug/debug_colors.cc:297-299.
    pub fn MainThreadScrollRepaintRectBorderColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:110.
    // Source: cc/debug/debug_colors.cc:300-302.
    pub fn MainThreadScrollRepaintRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:111.
    // Source: cc/debug/debug_colors.cc:303-305.
    pub fn MainThreadScrollRepaintRectFillColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:113.
    // Source: cc/debug/debug_colors.cc:308-310.
    pub fn RasterInducingScrollRectBorderColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 0.5f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:114.
    // Source: cc/debug/debug_colors.cc:311-313.
    pub fn RasterInducingScrollRectBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:115.
    // Source: cc/debug/debug_colors.cc:314-316.
    pub fn RasterInducingScrollRectFillColor() -> Color4f {
        return Color4f {
            red: 200.0f32 / 255.0f32,
            green: 100.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 15.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:117.
    // Source: cc/debug/debug_colors.cc:319-321.
    pub fn LayerAnimationBoundsBorderColor() -> Color4f {
        return Color4f {
            red: 112.0f32 / 255.0f32,
            green: 229.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:118.
    // Source: cc/debug/debug_colors.cc:322-322.
    pub fn LayerAnimationBoundsBorderWidth() -> i32 {
        return 2;
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:119.
    // Source: cc/debug/debug_colors.cc:323-325.
    pub fn LayerAnimationBoundsFillColor() -> Color4f {
        return Color4f {
            red: 112.0f32 / 255.0f32,
            green: 229.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 30.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:121.
    // Source: cc/debug/debug_colors.cc:328-330.
    pub fn PictureBorderColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.0f32,
            blue: 200.0f32 / 255.0f32,
            alpha: 100.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:125.
    // Source: cc/debug/debug_colors.cc:334-336.
    pub fn HUDBackgroundColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 0.0f32,
            blue: 0.0f32,
            alpha: 217.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:126.
    // Source: cc/debug/debug_colors.cc:337-339.
    pub fn HUDSeparatorLineColor() -> Color4f {
        return Color4f {
            red: 0.0f32,
            green: 1.0f32,
            blue: 0.0f32,
            alpha: 0.25f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:127.
    // Source: cc/debug/debug_colors.cc:340-342.
    pub fn HUDIndicatorLineColor() -> Color4f {
        return Color4f {
            red: 1.,
            green: 1.,
            blue: 0.,
            alpha: 1.,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:128.
    // Source: cc/debug/debug_colors.cc:343-345.
    pub fn HUDTitleColor() -> Color4f {
        return Color4f {
            red: 232.0f32 / 255.0f32,
            green: 232.0f32 / 255.0f32,
            blue: 232.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:130.
    // Source: cc/debug/debug_colors.cc:347-349.
    pub fn PlatformLayerTreeTextColor() -> Color4f {
        return Color4f {
            red: 1.,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:131.
    // Source: cc/debug/debug_colors.cc:350-352.
    pub fn FPSDisplayTextAndGraphColor() -> Color4f {
        return Color4f {
            red: 0.,
            green: 1.,
            blue: 0.,
            alpha: 1.,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:132.
    // Source: cc/debug/debug_colors.cc:355-357.
    pub fn FPSDisplayDroppedFrame() -> Color4f {
        return Color4f {
            red: 202.0f32 / 255.0f32,
            green: 91.0f32 / 255.0f32,
            blue: 29.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:133.
    // Source: cc/debug/debug_colors.cc:360-362.
    pub fn FPSDisplayMissedFrame() -> Color4f {
        return Color4f {
            red: 1.0f32,
            green: 245.0f32 / 255.0f32,
            blue: 0.0f32,
            alpha: 1.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:134.
    // Source: cc/debug/debug_colors.cc:364-366.
    pub fn FPSDisplaySuccessfulFrame() -> Color4f {
        return Color4f {
            red: 174.0f32 / 255.0f32,
            green: 221.0f32 / 255.0f32,
            blue: 1.0f32,
            alpha: 191.0f32 / 255.0f32,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:135.
    // Source: cc/debug/debug_colors.cc:368-370.
    pub fn MemoryDisplayTextColor() -> Color4f {
        return Color4f {
            red: 0.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        };
    }
}
impl DebugColors {
    // Source: cc/debug/debug_colors.h:136.
    // Source: cc/debug/debug_colors.cc:373-375.
    pub fn PaintTimeDisplayTextAndGraphColor() -> Color4f {
        return Color4f {
            red: 75.0f32 / 255.0f32,
            green: 155.0f32 / 255.0f32,
            blue: 55.0f32 / 255.0f32,
            alpha: 1.0f32,
        };
    }
}
