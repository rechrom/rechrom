//! Browser chrome controls are currently painted by a Page while BrowserView
//! owns their root backdrops. The host owns navigation and tabs. Geometry
//! follows Chromium's non-touch layout_constants / TabStyle; icon paths are
//! translated from its vector_icons (see chrome_icons/LICENSE).
pub const TABSTRIP_HEIGHT: f64 = 40.0;
pub const HEIGHT: f64 = TABSTRIP_HEIGHT + 46.0;
pub const TABSTRIP_BACKGROUND: [u8; 4] = [0xde, 0xe1, 0xe6, 0xff];
pub const NAVIGATION_BACKGROUND: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
pub const NAVIGATION_BORDER: [u8; 4] = [0xda, 0xdc, 0xe0, 0xff];

fn covered_color(background: [u8; 4], foreground: [u8; 4], coverage: f64) -> [u8; 4] {
    let coverage = coverage.clamp(0.0, 1.0);
    let channel = |background: u8, foreground: u8| {
        (f64::from(background) * (1.0 - coverage) + f64::from(foreground) * coverage).round() as u8
    };
    [
        channel(background[0], foreground[0]),
        channel(background[1], foreground[1]),
        channel(background[2], foreground[2]),
        0xff,
    ]
}

pub(crate) fn browser_root_frame(
    viewport: crate::engine::Viewport,
) -> crate::page_presentation::BrowserRootFrame {
    use crate::page_presentation::{BrowserRootFrame, BrowserRootSolidColorQuad, DeviceRect};

    let toolbar_height = viewport.toolbar_pixels().min(viewport.height);
    let tabstrip_height = (TABSTRIP_HEIGHT * viewport.scale)
        .round()
        .clamp(0.0, f64::from(toolbar_height)) as u32;
    let navigation_height = toolbar_height.saturating_sub(tabstrip_height);
    // The toolbar Page rasterizes a one-CSS-pixel border with device coverage:
    // whole device rows use the border color and the remaining top row is the
    // border composited over navigation white. Represent that coverage as an
    // opaque root quad so a stale toolbar Surface and its root gutter match at
    // fractional DPR without requiring a post-render pixel patch.
    let full_border_height = (viewport.scale.floor() as u32).min(navigation_height);
    let fractional_border_coverage = viewport.scale.fract();
    let full_border_y = toolbar_height.saturating_sub(full_border_height);
    let mut solid_color_quads = Vec::with_capacity(4);
    if full_border_height != 0 {
        solid_color_quads.push(BrowserRootSolidColorQuad {
            destination: DeviceRect::new(
                0,
                full_border_y as i32,
                viewport.width,
                full_border_height,
            ),
            color: NAVIGATION_BORDER,
        });
    }
    if fractional_border_coverage > f64::EPSILON && full_border_y != 0 {
        solid_color_quads.push(BrowserRootSolidColorQuad {
            destination: DeviceRect::new(0, full_border_y as i32 - 1, viewport.width, 1),
            color: covered_color(
                NAVIGATION_BACKGROUND,
                NAVIGATION_BORDER,
                fractional_border_coverage,
            ),
        });
    }
    solid_color_quads.extend([
        BrowserRootSolidColorQuad {
            destination: DeviceRect::new(
                0,
                tabstrip_height as i32,
                viewport.width,
                navigation_height,
            ),
            color: NAVIGATION_BACKGROUND,
        },
        BrowserRootSolidColorQuad {
            destination: DeviceRect::new(0, 0, viewport.width, tabstrip_height),
            color: TABSTRIP_BACKGROUND,
        },
    ]);

    BrowserRootFrame {
        background_color: NAVIGATION_BACKGROUND,
        // Aggregated render-pass quads are front-to-back. Border coverage
        // overlaps the navigation background and therefore precedes it.
        solid_color_quads,
    }
}

pub fn document<P>(tabs: &crate::tabs::Tabs<P>) -> String {
    let ordered = tabs.ordered();
    let tabstrip_basis = ordered.len().saturating_mul(238).saturating_sub(6);
    let tabstrip = tab_strip(tabs);
    let mut html = String::from(
        r#"<!doctype html><html><head><style>
*{box-sizing:border-box}html,body{margin:0;width:100%;height:86px;font:13px sans-serif;color:#1f1f1f;background:transparent;overflow:hidden}
body.inactive{background:#e8eaed;color:#5f6368}button{display:flex;align-items:center;justify-content:center;padding:0;border:0;background:transparent;color:#444746;cursor:default;flex-shrink:0}
button.hovered{background:#d3d7dc}button[disabled]{opacity:1}button[disabled].hovered{background:transparent}button[disabled] path{fill:#bfc1c3;stroke:#bfc1c3}
svg{display:block;flex-shrink:0;width:20px;height:20px}#tabrow{display:flex;align-items:center;height:40px;padding:6px 12px 0 92px}
#tabsearch{width:28px;height:28px;margin-right:12px;border-radius:8px;background:#cbd0d8}#tabsearch.hovered{background:#bfc5cf}#tabsearch svg{width:18px;height:18px}
#tabs{display:flex;flex:0 1 auto;min-width:0;height:34px;gap:6px;overflow:visible}
.tab{position:relative;z-index:0;display:flex;align-items:center;flex:1;min-width:56px;max-width:232px;height:34px;padding:0 8px;gap:8px;border-radius:10px;cursor:default}
.hoverbackdrop{display:none;position:absolute;z-index:-1;left:0;right:0;top:0;height:28px;border-radius:10px;background:#cbd0d8}.tab.hovered .hoverbackdrop{display:block}.tab.active .hoverbackdrop{display:none}.tab.active,.tab.active.hovered{background:#fff;border-radius:10px 10px 0 0;z-index:2}
.favicon{width:16px;height:16px;flex-shrink:0}.favicon svg{width:16px;height:16px}.throbber{animation:throbber-rotate 1568ms linear infinite;transform-origin:center}.throbber circle{fill:none;stroke:#1a73e8;stroke-width:2;stroke-linecap:round;stroke-dasharray:28.27 9.42;transform:rotate(-90deg);transform-origin:8px 8px}@keyframes throbber-rotate{to{transform:rotate(360deg)}}.title{flex:1;min-width:0;height:20px;line-height:20px;overflow:hidden;white-space:nowrap;font-size:12px}
.close{height:28px;width:28px;min-width:28px;border-radius:14px;margin-left:-4px;margin-right:-4px}.close svg{height:16px;width:16px}.close.hovered{background:#c5c9d0}
.separator{position:absolute;right:-1px;top:9px;width:2px;height:16px;background:#aeb4be;border-radius:1px}.active .separator,.beforeactive .separator,.tab.hovered .separator{display:none}
.shoulder{display:none;position:absolute;bottom:0;width:12px;height:12px}.active .shoulder{display:block}.left{left:-12px}.right{right:-12px}
#newtab{height:28px;width:28px;min-width:28px;margin-left:8px;border-radius:14px}#newtab svg{width:16px;height:16px}#dragspace{flex:1;min-width:42px;height:34px}
#navigation{display:flex;align-items:center;height:46px;padding:6px;background:transparent}
.navbutton{height:34px;width:34px;border-radius:17px;margin-right:2px}#reload{margin-right:9px}
#omnibox{display:flex;align-items:center;flex:1;min-width:48px;height:34px;border:2px solid transparent;border-radius:17px;background:#f1f3f4;margin-right:9px}
#omnibox.hovered{background:#e8eaed}#omnibox.focused,#omnibox.focused.hovered{border-color:#1a73e8;background:white}
#site{height:28px;width:28px;min-width:28px;border-radius:14px;margin-left:2px;margin-right:4px}#site svg{height:16px;width:16px}
input{flex:1;min-width:8px;width:100%;height:30px;padding:0 10px 0 0;border:0;background:transparent;color:#202124;font:14px sans-serif;outline:none}
#menu{height:34px;width:34px;border-radius:17px}
</style></head><body id="chrome"><div id="tabrow"><button id="tabsearch" aria-label="Search tabs" title="Search tabs">ICON:chevron</button><div id="tabs" style="flex-basis:__RECHROM_TABSTRIP_BASIS__px">__RECHROM_TABSTRIP__</div><button id="newtab" aria-label="New tab" title="New tab (⌘T)">ICON:add</button><div id="dragspace"></div></div><div id="navigation"><button id="back" class="navbutton" aria-label="Back" title="Back">ICON:back</button><button id="forward" class="navbutton" aria-label="Forward" title="Forward">ICON:forward</button><button id="reload" class="navbutton" aria-label="Reload" title="Reload (⌘R)">ICON:reload</button><div id="omnibox"><button id="site" aria-label="Page information" title="Page information">ICON:site</button><input id="address" type="text" aria-label="Address" value="about:home"></div><button id="menu" aria-label="Browser menu" title="Browser menu">ICON:menu</button></div></body></html>"#,
    );
    html = html.replace("__RECHROM_TABSTRIP_BASIS__", &tabstrip_basis.to_string());
    html = html.replace("__RECHROM_TABSTRIP__", &tabstrip);
    for (name, svg) in [
        ("back", include_str!("chrome_icons/back.svg")),
        ("forward", include_str!("chrome_icons/forward.svg")),
        ("reload", include_str!("chrome_icons/reload.svg")),
        ("site", include_str!("chrome_icons/site.svg")),
        ("menu", include_str!("chrome_icons/menu.svg")),
        ("add", include_str!("chrome_icons/add.svg")),
        ("chevron", include_str!("chrome_icons/chevron.svg")),
    ] {
        html = html.replace(&format!("ICON:{name}"), svg);
    }
    html
}
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn tab_strip<P>(tabs: &crate::tabs::Tabs<P>) -> String {
    let ordered = tabs.ordered();
    ordered.iter().enumerate().map(|(index, tab)| {
        let active = tab.id == tabs.active.id;
        let before_active = ordered.get(index + 1).is_some_and(|next| next.id == tabs.active.id);
        format!(r##"<div id="tab-{}" class="tab{}{}" title="{}"><span class="hoverbackdrop"></span><svg class="shoulder left" width="12" height="12" viewBox="0 0 12 12"><path fill="#fff" d="M0 12 A12 12 0 0 0 12 0 L12 12 Z"/></svg><svg class="shoulder right" width="12" height="12" viewBox="0 0 12 12"><path fill="#fff" d="M0 0 A12 12 0 0 0 12 12 L0 12 Z"/></svg><span id="favicon-{}" class="favicon">{}</span><span id="title-{}" class="title">{}</span><button id="close-{}" class="close" aria-label="Close tab">{}</button><span class="separator"></span></div>"##,
            tab.id, if active { " active" } else { "" }, if before_active { " beforeactive" } else { "" }, escape(&tab.location),
            tab.id, tab_icon(tab.loading), tab.id, escape(&tab.title), tab.id, include_str!("chrome_icons/close.svg"))
    }).collect()
}

pub fn tab_icon(loading: bool) -> &'static str {
    if loading {
        r#"<svg class="throbber" viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="6"/></svg>"#
    } else {
        include_str!("chrome_icons/page.svg")
    }
}

/// Browser-owned popups are separate small Pages; their document never mutates
/// the site's DOM. Coordinates are in the content viewport below the toolbar.
pub fn popup_document(rows: &[(String, String, String)], width: f64, x: f64) -> String {
    let entries: String = rows.iter().map(|(id, label, shortcut)| format!(
        r#"<button id="{}"><span class="label">{}</span><span class="shortcut">{}</span></button>"#,
        escape(id), escape(label), escape(shortcut))).collect();
    format!(
        r#"<!doctype html><html><head><style>*{{box-sizing:border-box}}html,body{{margin:0;width:100%;height:100%;background:transparent;font:13px sans-serif;color:#202124}}#popup{{position:absolute;left:{x}px;top:4px;width:{width}px;padding:8px 0;background:#fff;border:1px solid #dadce0;border-radius:8px;box-shadow:0 4px 12px rgba(0,0,0,.2)}}button{{display:flex;align-items:center;width:100%;height:32px;padding:0 16px;background:transparent;border:0;text-align:left;color:#202124;font:13px sans-serif;cursor:default}}button.hovered{{background:#e8eaed}}button:focus{{background:#e8eaed}}.label{{flex:1;min-width:0;white-space:nowrap;overflow:hidden}}.shortcut{{margin-left:16px;color:#5f6368;font-size:12px}}</style></head><body><div id="popup">{entries}</div></body></html>"#
    )
}
pub fn reload_icon(loading: bool) -> &'static str {
    if loading {
        include_str!("chrome_icons/close.svg")
    } else {
        include_str!("chrome_icons/reload.svg")
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Viewport;

    #[test]
    fn initial_document_owns_the_first_tab_subtree() {
        let tabs = crate::tabs::Tabs::<()>::new();
        let html = super::document(&tabs);
        assert!(html.contains(r#"id="tabs" style="flex-basis:232px""#));
        assert!(html.contains(r#"id="tab-1" class="tab active""#));
        assert!(html.contains(r#"id="title-1" class="title">New tab"#));
        // BrowserView's root Viz frame owns both opaque backdrops and the
        // device-covered separator. ToolbarView must leave those pixels
        // transparent rather than duplicate them in its child Surface.
        assert!(html.contains("html,body{margin:0;width:100%;height:86px;font:13px sans-serif;color:#1f1f1f;background:transparent"));
        assert!(html.contains("#navigation{display:flex;align-items:center;height:46px;padding:6px;background:transparent}"));
        assert!(!html.contains("__RECHROM_"));
    }

    #[test]
    fn browser_root_backdrops_follow_toolbar_dpr_and_front_to_back_order() {
        for (scale, toolbar_height, tabstrip_height, full_border_height, fractional) in [
            (1.0, 86, 40, 1, None),
            (1.25, 108, 50, 1, Some([246, 246, 247, 255])),
            (1.5, 129, 60, 1, Some([237, 238, 240, 255])),
            (2.0, 172, 80, 2, None),
        ] {
            let root = super::browser_root_frame(Viewport {
                width: 320,
                height: 400,
                scale,
            });
            assert_eq!(root.background_color, super::NAVIGATION_BACKGROUND);
            assert_eq!(
                root.solid_color_quads.len(),
                3 + usize::from(fractional.is_some())
            );
            let border = root.solid_color_quads[0];
            assert_eq!(border.color, super::NAVIGATION_BORDER);
            assert_eq!(border.destination.y, toolbar_height - full_border_height);
            assert_eq!(border.destination.height, full_border_height as u32);
            let backdrop_index = if let Some(color) = fractional {
                let fractional_border = root.solid_color_quads[1];
                assert_eq!(fractional_border.color, color);
                assert_eq!(fractional_border.destination.y, border.destination.y - 1);
                assert_eq!(fractional_border.destination.height, 1);
                2
            } else {
                1
            };
            let navigation = root.solid_color_quads[backdrop_index];
            let tabstrip = root.solid_color_quads[backdrop_index + 1];
            assert_eq!(navigation.color, super::NAVIGATION_BACKGROUND);
            assert_eq!(navigation.destination.y, tabstrip_height);
            assert_eq!(
                navigation.destination.height,
                (toolbar_height - tabstrip_height) as u32
            );
            assert_eq!(tabstrip.color, super::TABSTRIP_BACKGROUND);
            assert_eq!(tabstrip.destination.height, tabstrip_height as u32);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl DragRegion {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// ValidationMessageOverlayDelegate's separate, transparent document. Retain
/// Chromium's resource styling; animation integration can use this Page's own
/// timeline without reaching backward into the author Paint lifecycle.
pub fn validation_bubble_document(message: &[u16]) -> String {
    let css = include_str!("chrome_validation_bubble.css");
    let icon = include_str!("chrome_input_alert.svg");
    let message = escape(&String::from_utf16_lossy(message));
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="color-scheme" content="light dark"><style>{css}
html,body{{width:100%;height:100%;background:transparent;pointer-events:none}}
</style></head><body><div id="container" class="" style="left:0;top:0"><div id="outer-arrow-top"></div><div id="inner-arrow-top"></div><div id="spacer-top"></div><main id="bubble-body" role="alert">{icon}<div dir="auto" id="main-message">{message}</div><div id="sub-message"></div></main><div id="outer-arrow-bottom"></div><div id="inner-arrow-bottom"></div><div id="spacer-bottom"></div></div></body></html>"#
    )
}

/// ValidationMessageOverlayDelegate::AdjustBubblePosition, cc:261-371. Inputs
/// are the anchor and the independent overlay Page's measured CSS geometry.
pub fn validation_bubble_position(
    anchor: browser::page::ValidationMessageRect,
    bubble: (f64, f64),
    viewport: crate::engine::Viewport,
) -> Vec<crate::page_runtime::Patch> {
    use crate::page_runtime::Patch;
    let height = f64::from(viewport.height) / viewport.scale;
    let (width, bubble_height) = bubble;
    let below = height - anchor.y - anchor.height >= bubble_height;
    let y = if below {
        anchor.y + anchor.height
    } else {
        anchor.y - bubble_height
    };
    let x = (anchor.x + anchor.width * 0.5 - width * 0.5)
        .max(0.0)
        .min((viewport.logical_width() - width).max(0.0));
    let anchor_left = (anchor.x + 8.0).min(anchor.x + anchor.width * 0.5);
    let arrow_anchor = if x + 18.0 < anchor_left {
        (anchor_left - x).min(width - 18.0)
    } else {
        18.0
    };
    let arrow_x = arrow_anchor - 8.0;
    let origin = if below { "top" } else { "bottom" };
    let class = if below {
        "shown-fully"
    } else {
        "shown-fully bottom-arrow"
    };
    let mut patches = vec![
        Patch::Attribute {
            id: "container".into(),
            name: "style".into(),
            value: Some(format!(
                "left:{x}px;top:{y}px;transform-origin:{:.2}% {origin}",
                arrow_anchor * 100.0 / width
            )),
        },
        Patch::Attribute {
            id: "container".into(),
            name: "class".into(),
            value: Some(class.into()),
        },
    ];
    for id in if below {
        ["outer-arrow-top", "inner-arrow-top"]
    } else {
        ["outer-arrow-bottom", "inner-arrow-bottom"]
    } {
        patches.push(Patch::Attribute {
            id: id.into(),
            name: "style".into(),
            value: Some(format!("left:{arrow_x}px")),
        });
    }
    patches
}
