//! Cursor policy mirrors Blink EventHandler::SelectCursor/SelectAutoCursor.
//! It consumes the resolved style and hit node; no window API belongs here.
use dom::{persistent_document::DOMNodeType, Document};
pub use layoutng_assembly::internal::paint_input::ECursor as Cursor;

#[allow(non_snake_case)]
pub(crate) fn SelectCursor(document: &Document, hit: Option<u64>) -> Cursor {
    let Some(index) = hit.and_then(|id| document.FindNodeById(id)) else {
        return Cursor::kDefault;
    };
    let node = document.Node(index);
    let text = node.Type() == DOMNodeType::kText;
    let mut element = Some(index);
    while let Some(i) = element {
        let node = document.Node(i);
        if node.Type() == DOMNodeType::kElement {
            break;
        }
        element = node.Parent();
    }
    let Some(index) = element else {
        return Cursor::kDefault;
    };
    let style = document.ResolvedStyleFor(index).map(|s| &s.style);
    let cursor = style.map_or(Cursor::kAuto, |s| s.paint.cursor);
    if cursor != Cursor::kAuto {
        return cursor;
    }
    let vertical = style.is_some_and(|s| {
        s.writing_mode != layoutng_assembly::internal::layout_input::WritingMode::kHorizontalTb
    });
    let mut editable = false;
    let mut ancestor = Some(index);
    while let Some(i) = ancestor {
        let node = document.Node(i);
        if node.IsHTMLElement("textarea")
            || (node.IsHTMLElement("input")
                && matches!(
                    node.FindAttribute("type")
                        .map_or("text", |a| a.value.as_str())
                        .to_ascii_lowercase()
                        .as_str(),
                    "" | "text" | "search" | "email" | "password" | "tel" | "url" | "number"
                ))
        {
            editable = node.FindAttribute("disabled").is_none();
            break;
        }
        if let Some(attribute) = node.FindAttribute("contenteditable") {
            editable = matches!(
                attribute.value.to_ascii_lowercase().as_str(),
                "" | "true" | "plaintext-only"
            );
            break;
        }
        ancestor = node.Parent();
    }
    if text || editable {
        if vertical {
            Cursor::kVerticalText
        } else {
            Cursor::kText
        }
    } else {
        Cursor::kDefault
    }
}
