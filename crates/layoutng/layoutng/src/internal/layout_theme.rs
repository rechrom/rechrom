//! Platform marker colors owned by Chromium's LayoutTheme boundary.

#![allow(non_snake_case)]

use foundation::Color;

// Source: third_party/blink/renderer/core/layout/layout_theme.h:117-118.
pub struct LayoutTheme;

static THEME: LayoutTheme = LayoutTheme;

impl LayoutTheme {
    pub fn GetTheme() -> &'static Self {
        &THEME
    }

    // Source: third_party/blink/public/common/renderer_preferences/
    // renderer_preferences.h:26-27 and core/layout/layout_theme.cc:361-371.
    // The opaque default #1967d2 is represented by Chromium's
    // Color::BlendWithWhite as rgba(0, 65, 198, .8).
    pub fn ActiveSelectionBackgroundColor(&self) -> Color {
        Color::FromRGBA(0, 65, 198, 204)
    }

    // Source: renderer_preferences.h:27 and layout_theme.cc:379-382.
    #[cfg(not(target_vendor = "apple"))]
    pub fn ActiveSelectionForegroundColor(&self) -> Color {
        Color::FromRGB(255, 255, 255)
    }

    // Source: third_party/blink/renderer/core/layout/layout_theme.cc:409-411.
    // Source: third_party/blink/renderer/core/layout/layout_theme_mac.mm:83-87.
    // Source: third_party/blink/renderer/core/layout/layout_theme_ios.mm:40-44.
    pub fn PlatformSpellingMarkerUnderlineColor(&self) -> Color {
        #[cfg(target_vendor = "apple")]
        {
            return Color::FromRGBA(255, 59, 48, 191);
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            Color::FromRGB(255, 0, 0)
        }
    }

    // Source: third_party/blink/renderer/core/layout/layout_theme.cc:413-415.
    // Source: third_party/blink/renderer/core/layout/layout_theme_mac.mm:89-93.
    // Source: third_party/blink/renderer/core/layout/layout_theme_ios.mm:46-50.
    pub fn PlatformGrammarMarkerUnderlineColor(&self) -> Color {
        #[cfg(target_vendor = "apple")]
        {
            return Color::FromRGBA(25, 175, 50, 191);
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            Color::FromRGB(192, 192, 192)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_colors_follow_the_platform_layout_theme() {
        let theme = LayoutTheme::GetTheme();
        #[cfg(target_vendor = "apple")]
        {
            assert_eq!(
                theme.PlatformSpellingMarkerUnderlineColor(),
                Color::FromRGBA(255, 59, 48, 191)
            );
            assert_eq!(
                theme.PlatformGrammarMarkerUnderlineColor(),
                Color::FromRGBA(25, 175, 50, 191)
            );
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            assert_eq!(
                theme.PlatformSpellingMarkerUnderlineColor(),
                Color::FromRGB(255, 0, 0)
            );
            assert_eq!(
                theme.PlatformGrammarMarkerUnderlineColor(),
                Color::FromRGB(192, 192, 192)
            );
        }
    }

    #[test]
    fn default_active_selection_background_matches_chromium_blend() {
        assert_eq!(
            LayoutTheme::GetTheme().ActiveSelectionBackgroundColor(),
            Color::FromRGBA(0, 65, 198, 204)
        );
    }
}
