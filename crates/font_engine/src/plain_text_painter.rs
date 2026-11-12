//! PlainTextPainter kShared path: plain_text_painter.cc:69-177,179-183,231-235,
//! plain_text_node.cc:24-87,279-363,397-449, fixed Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. Retains the canonical FontEngine's
//! font resources; bidi and HarfBuzz remain in their original source owners.
use crate::text::implementation::{bidi_paragraph::BidiParagraph, character::Character};
use crate::{FontCompute, FontDescription, FontEngine, FontFace, FontShapeRequest, *};
use foundation::{String as BlinkString, StringView, TextDirection};
#[derive(Clone)]
pub struct PlainTextGlyph {
    pub id: u16,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone)]
pub struct PlainTextGlyphRun {
    pub face: FontFace,
    pub size: f32,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub variations: Vec<FontVariation>,
    pub glyphs: Vec<PlainTextGlyph>,
}
pub struct PlainTextPaint {
    pub width: f32,
    pub runs: Vec<PlainTextGlyphRun>,
    pub skip_drawing: bool,
}
pub struct PlainTextPainter;
impl PlainTextPainter {
    pub fn BaseDirection(text: &[u16]) -> TextDirection {
        BidiParagraph::BaseDirectionForStringOrLtr(&StringView::from(&BlinkString::from_utf16(
            text,
        )))
    }
    pub fn Shape(
        engine: &mut FontEngine,
        text: &[u16],
        description: &FontDescription,
        direction: TextDirection,
        supports_bidi: bool,
    ) -> FontCompute<PlainTextPaint> {
        let _scope = foundation::LayoutHeapScope::new();
        match engine.GetMetrics(description, engine.ContextVersion()) {
            FontCompute::Ready(_) => {}
            FontCompute::Pending(p) => return FontCompute::Pending(p),
            FontCompute::Error(e) => return FontCompute::Error(e),
        }
        let font = engine.ResolveDescription(description);
        let PlainTextNode { normalized, ranges } =
            PlainTextNode::new(text, direction, supports_bidi, false, unsafe { &*font });
        let full_text = BlinkString::from_utf16(&normalized);
        let mut spacing =
            crate::fonts::shaping::shape_result_spacing::ShapeResultSpacing::new(&full_text, false);
        spacing.SetSpacingWithNormalization(description, false);
        let mut paint = PlainTextPaint {
            width: 0.,
            runs: Vec::new(),
            skip_drawing: unsafe { &*font }.ShouldSkipDrawing(),
        };
        for range in ranges {
            let item = &normalized[range.start as usize..range.end as usize];
            let mut result = match engine.Shape(&FontShapeRequest {
                text: item,
                range: 0..item.len() as u32,
                description,
                direction: range.Direction(),
                segments: &[],
                context_version: engine.ContextVersion(),
            }) {
                FontCompute::Ready(result) => result,
                FontCompute::Pending(pending) => return FontCompute::Pending(pending),
                FontCompute::Error(error) => return FontCompute::Error(error),
            };
            if spacing.HasSpacing() {
                result.ApplyPlainTextSpacing(&mut spacing, range.start as i32);
            }
            let faces = engine.ResolvedFaces();
            let mut x = paint.width;
            for run in result.RunsOrParts() {
                let run = unsafe { &*run.Get() };
                let font = unsafe { &*run.font_data_.Get() };
                let platform = font.PlatformData();
                let Some(face) = faces.get(platform.ResourceFaceIndex() as usize) else {
                    return FontCompute::Error(FontError::InvalidResponse);
                };
                let data = run.GetGlyphDataRange();
                let offsets = data.Offsets();
                let mut glyphs = Vec::with_capacity(data.size() as usize);
                for (index, glyph) in data.Glyphs().iter().enumerate() {
                    let offset = offsets.get(index).copied().unwrap_or_default();
                    glyphs.push(PlainTextGlyph {
                        id: glyph.glyph() as u16,
                        x: x + offset.x(),
                        y: offset.y(),
                    });
                    x += glyph.advance.ToFloat();
                }
                paint.runs.push(PlainTextGlyphRun {
                    face: face.clone(),
                    size: platform.size(),
                    synthetic_bold: platform.SyntheticBold(),
                    synthetic_italic: platform.SyntheticItalic(),
                    variations: platform.ResourceVariations().to_vec(),
                    glyphs,
                });
            }
            paint.width += result.Width();
        }
        FontCompute::Ready(paint)
    }
}

struct PlainTextNode {
    normalized: Vec<u16>,
    ranges: Vec<crate::text::implementation::bidi_paragraph::Run>,
}
impl PlainTextNode {
    fn new(
        text: &[u16],
        direction: TextDirection,
        supports_bidi: bool,
        directional_override: bool,
        font: &Font,
    ) -> Self {
        let original = BlinkString::from_utf16(text);
        let mut normalized = text.to_vec();
        let mut maybe_bidi = false;
        let mut i = 0;
        while i < text.len() {
            let first = text[i];
            let (ch, n) = if (0xd800..=0xdbff).contains(&first)
                && i + 1 < text.len()
                && (0xdc00..=0xdfff).contains(&text[i + 1])
            {
                (
                    0x10000
                        + ((u32::from(first) - 0xd800) << 10)
                        + (u32::from(text[i + 1]) - 0xdc00),
                    2,
                )
            } else {
                (u32::from(first), 1)
            };
            if Character::TreatAsSpace(ch as i32) && ch != 0xa0 {
                normalized[i] = 0x20;
            } else if Character::TreatAsZeroWidthSpaceInComplexScriptLegacy(ch as i32) {
                normalized[i] = 0x200b;
            }
            maybe_bidi |= Character::MaybeBidiRtl(ch as i32);
            i += n;
        }
        let mut ranges = Vec::new();
        if supports_bidi && (maybe_bidi || direction == TextDirection::kRtl || directional_override)
        {
            let bidi_text = if directional_override {
                BidiParagraph::StringWithDirectionalOverride(
                    &StringView::from(&original),
                    direction,
                )
            } else {
                original.clone()
            };
            let bidi = BidiParagraph::new(&bidi_text, Some(direction));
            bidi.GetVisualRuns(&bidi_text, &mut ranges);
            if directional_override {
                ranges = ranges
                    .into_iter()
                    .filter_map(|mut run| {
                        let start = run.start.max(1);
                        let end = run.end.min(bidi_text.length() - 1);
                        if start >= end {
                            return None;
                        }
                        run.start = start - 1;
                        run.end = end - 1;
                        Some(run)
                    })
                    .collect();
            }
        }
        if ranges.is_empty() && !text.is_empty() {
            ranges.push(crate::text::implementation::bidi_paragraph::Run::new(
                0,
                text.len() as u32,
                if direction == TextDirection::kRtl {
                    1
                } else {
                    0
                },
            ));
        }
        if font.CanShapeWordByWord() {
            let mut words = Vec::new();
            for range in ranges {
                let insertion = words.len();
                let item = &normalized[range.start as usize..range.end as usize];
                let mut start = 0;
                while start < item.len() {
                    let end = NextWordEndIndex(item, start);
                    let word = crate::text::implementation::bidi_paragraph::Run::new(
                        range.start + start as u32,
                        range.start + end as u32,
                        range.level,
                    );
                    if range.Direction() == TextDirection::kLtr {
                        words.push(word);
                    } else {
                        words.insert(insertion, word);
                    }
                    start = end;
                }
            }
            ranges = words;
        }
        Self { normalized, ranges }
    }
}

impl PlainTextPainter {
    /// core/layout/text_utils.cc14-24 and PlainTextNode::Shape cc397-449.
    /// Uses the original FontEngine-resolved Font; pending platform queries
    /// remain on its canonical collector and never execute I/O here.
    pub(crate) fn ComputeInlineSizeForFont(
        font: &Font,
        text: &[u16],
        direction: TextDirection,
        directional_override: bool,
    ) -> f32 {
        if text.is_empty() {
            return 0.;
        }
        let PlainTextNode { normalized, ranges } =
            PlainTextNode::new(text, direction, true, directional_override, font);
        let string = BlinkString::from_utf16(&normalized);
        let mut spacing =
            crate::fonts::shaping::shape_result_spacing::ShapeResultSpacing::new(&string, false);
        spacing.SetSpacingWithNormalization(font.GetFontDescription(), false);
        let mut width = 0.;
        for item in ranges {
            let text = BlinkString::from_utf16(&normalized[item.start as usize..item.end as usize]);
            let shaper = crate::HarfBuzzShaper::new(text);
            let result = shaper.Shape(font, item.Direction());
            let result = unsafe { &mut *result };
            if spacing.HasSpacing() {
                result.ApplySpacing(&mut spacing, item.start as i32);
            }
            width += result.Width();
        }
        width
    }
}

// PlainTextNode::NextWordEndIndex, source.cc:91-162. UTF-16 positions remain
// unchanged by normalization; CJK/emoji modifiers stay with their base.
fn NextWordEndIndex(text: &[u16], start: usize) -> usize {
    fn next(text: &[u16], index: &mut usize) -> i32 {
        let first = text[*index];
        *index += 1;
        if (0xd800..=0xdbff).contains(&first)
            && *index < text.len()
            && (0xdc00..=0xdfff).contains(&text[*index])
        {
            let ch =
                0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(text[*index]) - 0xdc00);
            *index += 1;
            ch as i32
        } else {
            i32::from(first)
        }
    }
    let delimiter = |c: i32, zws: bool| c == 0x20 || c == 9 || (zws && c == 0x200b);
    if start >= text.len() {
        return 0;
    }
    if start + 1 == text.len() || delimiter(i32::from(text[start]), true) {
        return start + 1;
    }
    let mut end = start;
    let mut ch = next(text, &mut end);
    if !Character::IsCjkIdeographOrSymbol(ch) {
        let mut next_end = end;
        while end < text.len() {
            ch = next(text, &mut next_end);
            if delimiter(ch, true) || Character::IsCjkIdeographOrSymbolBase(ch) {
                return end;
            }
            end = next_end;
        }
        return text.len();
    }
    let mut has_script = !Character::IsCommonOrInheritedScript(ch);
    let mut next_end = end;
    while end < text.len() {
        ch = next(text, &mut next_end);
        use foundation::unicode::*;
        let category = crate::text::implementation::unicode_category::Category(ch);
        if category
            & (kMark_SpacingCombining
                | kMark_Enclosing
                | kMark_NonSpacing
                | kLetter_Modifier
                | kSymbol_Modifier)
            != 0
            || ch == 0x200d
            || Character::IsEmojiComponent(ch)
            || Character::IsExtendedPictographic(ch)
        {
            end = next_end;
            continue;
        }
        if Character::IsCjkIdeographOrSymbol(ch) {
            if Character::IsCommonOrInheritedScript(ch) {
                end = next_end;
                continue;
            }
            if !has_script {
                has_script = true;
                end = next_end;
                continue;
            }
        }
        return end;
    }
    text.len()
}

/// SystemFontFamily(kNone) reaches DefaultGUIFont on all pinned platforms,
/// including Windows' default switch arm: layout_theme_font_provider_default.cc
/// 37-40, layout_theme_font_provider_win.cc43-55, shared provider.cc45-47.
pub fn DragLabelSystemFontFamily() -> &'static str {
    "Arial"
}
