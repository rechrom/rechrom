//! Browser chrome remains an ordinary Page. The host owns navigation and tabs.
//! Geometry follows Chromium's non-touch layout_constants / TabStyle; icon
//! paths are translated from its vector_icons (see chrome_icons/LICENSE).
pub const TABSTRIP_HEIGHT: f64 = 40.0;
pub const HEIGHT: f64 = TABSTRIP_HEIGHT + 46.0;
pub fn document() -> String {
    let mut html = String::from(
        r#"<!doctype html><html><head><style>
*{box-sizing:border-box}html,body{margin:0;width:100%;height:86px;font:13px sans-serif;color:#1f1f1f;background:#dee1e6;overflow:hidden}
body.inactive{background:#e8eaed;color:#5f6368}button{display:flex;align-items:center;justify-content:center;padding:0;border:0;background:transparent;color:#444746;cursor:default;flex-shrink:0}
button.hovered{background:#d3d7dc}button[disabled]{opacity:1}button[disabled].hovered{background:transparent}button[disabled] path{fill:#bfc1c3;stroke:#bfc1c3}
svg{display:block;flex-shrink:0;width:20px;height:20px}#tabrow{display:flex;align-items:center;height:40px;padding:6px 12px 0 92px}
#tabsearch{width:28px;height:28px;margin-right:12px;border-radius:8px;background:#cbd0d8}#tabsearch.hovered{background:#bfc5cf}#tabsearch svg{width:18px;height:18px}
#tabs{display:flex;flex:0 1 auto;min-width:0;height:34px;gap:6px;overflow:visible}
.tab{position:relative;z-index:0;display:flex;align-items:center;flex:1;min-width:56px;max-width:232px;height:34px;padding:0 8px;gap:8px;border-radius:10px;cursor:default}
.hoverbackdrop{display:none;position:absolute;z-index:-1;left:0;right:0;top:0;height:28px;border-radius:10px;background:#cbd0d8}.tab.hovered .hoverbackdrop{display:block}.tab.active .hoverbackdrop{display:none}.tab.active,.tab.active.hovered{background:#fff;border-radius:10px 10px 0 0;z-index:2}
.favicon{width:16px;height:16px;flex-shrink:0}.favicon svg{width:16px;height:16px}.title{flex:1;min-width:0;height:20px;line-height:20px;overflow:hidden;white-space:nowrap;font-size:12px}
.close{height:28px;width:28px;min-width:28px;border-radius:14px;margin-left:-4px;margin-right:-4px}.close svg{height:16px;width:16px}.close.hovered{background:#c5c9d0}
.separator{position:absolute;right:-1px;top:9px;width:2px;height:16px;background:#aeb4be;border-radius:1px}.active .separator,.beforeactive .separator,.tab.hovered .separator{display:none}
.shoulder{display:none;position:absolute;bottom:0;width:12px;height:12px}.active .shoulder{display:block}.left{left:-12px}.right{right:-12px}
#newtab{height:28px;width:28px;min-width:28px;margin-left:8px;border-radius:14px}#newtab svg{width:16px;height:16px}#dragspace{flex:1;min-width:42px;height:34px}
#navigation{display:flex;align-items:center;height:46px;padding:6px;background:#fff;border-bottom:1px solid #dadce0}
.navbutton{height:34px;width:34px;border-radius:17px;margin-right:2px}#reload{margin-right:9px}
#omnibox{display:flex;align-items:center;flex:1;min-width:48px;height:34px;border:2px solid transparent;border-radius:17px;background:#f1f3f4;margin-right:9px}
#omnibox.hovered{background:#e8eaed}#omnibox.focused,#omnibox.focused.hovered{border-color:#1a73e8;background:white}
#site{height:28px;width:28px;min-width:28px;border-radius:14px;margin-left:2px;margin-right:4px}#site svg{height:16px;width:16px}
input{flex:1;min-width:8px;width:100%;height:30px;padding:0 10px 0 0;border:0;background:transparent;color:#202124;font:14px sans-serif;outline:none}
#menu{height:34px;width:34px;border-radius:17px}
</style></head><body id="chrome"><div id="tabrow"><button id="tabsearch" aria-label="Search tabs" title="Search tabs">ICON:chevron</button><div id="tabs"></div><button id="newtab" aria-label="New tab" title="New tab (⌘T)">ICON:add</button><div id="dragspace"></div></div><div id="navigation"><button id="back" class="navbutton" aria-label="Back" title="Back">ICON:back</button><button id="forward" class="navbutton" aria-label="Forward" title="Forward">ICON:forward</button><button id="reload" class="navbutton" aria-label="Reload" title="Reload (⌘R)">ICON:reload</button><div id="omnibox"><button id="site" aria-label="Page information" title="Page information">ICON:site</button><input id="address" type="text" aria-label="Address" value="about:home"></div><button id="menu" aria-label="Browser menu" title="Browser menu">ICON:menu</button></div></body></html>"#,
    );
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
        format!(r##"<div id="tab-{}" class="tab{}{}" title="{}"><span class="hoverbackdrop"></span><svg class="shoulder left" width="12" height="12" viewBox="0 0 12 12"><path fill="#fff" d="M0 12 A12 12 0 0 0 12 0 L12 12 Z"/></svg><svg class="shoulder right" width="12" height="12" viewBox="0 0 12 12"><path fill="#fff" d="M0 0 A12 12 0 0 0 12 12 L0 12 Z"/></svg><span class="favicon">{}</span><span id="title-{}" class="title">{}</span><button id="close-{}" class="close" aria-label="Close tab">{}</button><span class="separator"></span></div>"##,
            tab.id, if active { " active" } else { "" }, if before_active { " beforeactive" } else { "" }, escape(&tab.location),
            include_str!("chrome_icons/page.svg"), tab.id, escape(&tab.title), tab.id, include_str!("chrome_icons/close.svg"))
    }).collect()
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
/// Compose two paint artifacts in paint order. Popup font slots and both
/// semantic index spaces are rebased; native IDs/property references survive.
pub fn content_with_popup(
    content: Option<&raster::surface::PaintArtifact>,
    popup: &raster::surface::PaintArtifact,
) -> std::io::Result<raster::surface::PaintArtifact> {
    use paint::paint_engine::{DisplayItem, DisplayItemType};
    let mut list = content.cloned().unwrap_or_default();
    let mut resources = list.resources.as_deref().cloned().unwrap_or_default();
    let font_offset = resources.fonts.len() as u32;
    if let Some(popup_resources) = &popup.resources {
        if !popup_resources.images.is_empty() {
            return Err(std::io::Error::other(
                "chrome popup may not load image resources",
            ));
        }
        resources
            .fonts
            .extend(popup_resources.fonts.iter().cloned());
    }
    list.resources = Some(std::sync::Arc::new(resources));
    let display_item_offset = u32::try_from(list.display_items.len())
        .map_err(|_| std::io::Error::other("too many content display items"))?;
    std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
        r#type: DisplayItemType::kSave,
        ..Default::default()
    });
    // The wrapper save occupies one replay-op slot, but no semantic item slot.
    let record_offset = list.items.len();
    std::sync::Arc::make_mut(&mut list.items).extend(popup.items.iter().cloned().map(
        |mut item| {
            item.font_face_index += font_offset;
            item
        },
    ));
    list.display_items
        .extend(popup.display_items.iter().cloned().map(|mut item| {
            item.record_begin += record_offset;
            item.record_end += record_offset;
            item
        }));
    for popup_chunk in &popup.chunks {
        let mut chunk = popup_chunk.clone();
        chunk.begin_index = chunk
            .begin_index
            .checked_add(display_item_offset)
            .ok_or_else(|| std::io::Error::other("too many popup display items"))?;
        chunk.end_index = chunk
            .end_index
            .checked_add(display_item_offset)
            .ok_or_else(|| std::io::Error::other("too many popup display items"))?;
        // Each Page has its own native clients. Clone retains property-node
        // Arcs and IDs; neither DOM IDs nor dump-local property IDs replace them.
        list.chunks.push(chunk);
    }
    std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
        r#type: DisplayItemType::kRestore,
        ..Default::default()
    });
    Ok(list)
}

pub fn reload_icon(loading: bool) -> &'static str {
    if loading {
        include_str!("chrome_icons/close.svg")
    } else {
        include_str!("chrome_icons/reload.svg")
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
