//! FontEngine-owned shaping and glyph-outline extraction for SVG clip text.
//! The DOM/Paint boundary supplies immutable catalog faces; this module owns
//! fallback selection, HarfBuzz shaping and variable-font outline expansion.

use crate::{OpenTypeFont, OpenTypeVariation, SharedFontBytes, TextDirection};
use ttf_parser::{Face, GlyphId, OutlineBuilder, Tag};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutlineVerb {
    MoveTo,
    LineTo,
    QuadraticTo,
    CubicTo,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineCommand {
    pub verb: OutlineVerb,
    pub values: [f64; 6],
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutlinedGlyph {
    pub commands: Vec<OutlineCommand>,
    /// UTF-8 byte offset in the original request. This preserves HarfBuzz's
    /// cluster contract across fallback spans for SVG per-character layout.
    pub cluster: u32,
    pub x_offset: f64,
    pub y_offset: f64,
    pub advance: f64,
}

pub struct ClipTextFace<'a> {
    pub bytes: &'a SharedFontBytes,
    pub face_index: u32,
    pub family: &'a str,
    pub weight: f64,
    pub italic: bool,
    pub variations: &'a [(u32, f32)],
    pub unicode_ranges: &'a [(u32, u32)],
}

pub struct ClipTextRequest<'a> {
    pub text: &'a str,
    pub families: &'a [String],
    pub size: f64,
    pub weight: f64,
    pub italic: bool,
    pub direction: TextDirection,
    pub language: &'a str,
}

struct Builder {
    commands: Vec<OutlineCommand>,
    scale: f64,
}
impl Builder {
    fn point(&self, x: f32, y: f32) -> (f64, f64) {
        (f64::from(x) * self.scale, -f64::from(y) * self.scale)
    }
}
impl OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.commands.push(OutlineCommand {
            verb: OutlineVerb::MoveTo,
            values: [x, y, 0.0, 0.0, 0.0, 0.0],
        });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.commands.push(OutlineCommand {
            verb: OutlineVerb::LineTo,
            values: [x, y, 0.0, 0.0, 0.0, 0.0],
        });
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x, y) = self.point(x, y);
        self.commands.push(OutlineCommand {
            verb: OutlineVerb::QuadraticTo,
            values: [x1, y1, x, y, 0.0, 0.0],
        });
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x2, y2) = self.point(x2, y2);
        let (x, y) = self.point(x, y);
        self.commands.push(OutlineCommand {
            verb: OutlineVerb::CubicTo,
            values: [x1, y1, x2, y2, x, y],
        });
    }
    fn close(&mut self) {
        self.commands.push(OutlineCommand {
            verb: OutlineVerb::Close,
            values: [0.0; 6],
        });
    }
}

fn covers(face: &ClipTextFace<'_>, character: char) -> bool {
    let code = character as u32;
    (face.unicode_ranges.is_empty()
        || face
            .unicode_ranges
            .iter()
            .any(|&(a, b)| (a..=b).contains(&code)))
        && Face::parse(face.bytes, face.face_index & 0xffff)
            .ok()
            .and_then(|face| face.glyph_index(character))
            .is_some()
}
fn select_face(
    faces: &[ClipTextFace<'_>],
    request: &ClipTextRequest<'_>,
    character: char,
) -> Option<usize> {
    faces
        .iter()
        .enumerate()
        .filter(|(_, face)| covers(face, character))
        .min_by(|(_, a), (_, b)| {
            let family_rank = |face: &ClipTextFace<'_>| {
                request
                    .families
                    .iter()
                    .position(|family| family.eq_ignore_ascii_case(face.family))
                    .unwrap_or(usize::MAX / 2)
            };
            family_rank(a)
                .cmp(&family_rank(b))
                .then_with(|| (a.italic != request.italic).cmp(&(b.italic != request.italic)))
                .then_with(|| {
                    (a.weight - request.weight)
                        .abs()
                        .total_cmp(&(b.weight - request.weight).abs())
                })
        })
        .map(|(index, _)| index)
}

pub fn ShapeClipText(
    request: &ClipTextRequest<'_>,
    faces: &[ClipTextFace<'_>],
) -> Result<Vec<OutlinedGlyph>, String> {
    if request.text.is_empty() {
        return Ok(Vec::new());
    }
    if !request.size.is_finite() || request.size <= 0.0 {
        return Err("invalid SVG clip text font size".into());
    }
    let mut spans = Vec::<(usize, usize, usize)>::new();
    let mut current: Option<(usize, usize)> = None;
    for (byte, character) in request.text.char_indices() {
        let face = select_face(faces, request, character)
            .ok_or_else(|| format!("missing SVG clip glyph U+{:04X}", character as u32))?;
        match current {
            Some((start, old)) if old == face => current = Some((start, old)),
            Some((start, old)) => {
                spans.push((start, byte, old));
                current = Some((byte, face));
            }
            None => current = Some((byte, face)),
        }
    }
    if let Some((start, face)) = current {
        spans.push((start, request.text.len(), face));
    }
    let mut output = Vec::new();
    for (start, end, face_index) in spans {
        let candidate = &faces[face_index];
        let variations = candidate
            .variations
            .iter()
            .map(|&(tag, value)| OpenTypeVariation { tag, value })
            .collect::<Vec<_>>();
        let font = OpenTypeFont::new_shared(
            (*candidate.bytes).clone(),
            candidate.face_index,
            request.size,
            &variations,
        );
        let shaped = font.ShapeRun(
            &request.text[start..end],
            request.direction,
            request.language,
            "",
        );
        let mut parsed = Face::parse(candidate.bytes, candidate.face_index & 0xffff)
            .map_err(|_| "invalid SVG clip font face".to_owned())?;
        for &(tag, value) in candidate.variations {
            let _ = parsed.set_variation(Tag::from_bytes(&tag.to_be_bytes()), value);
        }
        let scale = request.size / f64::from(parsed.units_per_em());
        for glyph in shaped.glyphs {
            let id =
                u16::try_from(glyph.id).map_err(|_| "SVG clip glyph id overflow".to_owned())?;
            let mut builder = Builder {
                commands: Vec::new(),
                scale,
            };
            if parsed.outline_glyph(GlyphId(id), &mut builder).is_none() {
                // Space and other blank glyphs legitimately have no outline.
                if font.Extents(glyph.id).is_some() {
                    return Err("missing SVG clip glyph outline".into());
                }
            }
            output.push(OutlinedGlyph {
                commands: builder.commands,
                cluster: u32::try_from(start)
                    .unwrap_or(u32::MAX)
                    .saturating_add(glyph.cluster),
                x_offset: glyph.x_offset,
                y_offset: -glyph.y_offset,
                advance: glyph.x_advance,
            });
        }
    }
    Ok(output)
}
