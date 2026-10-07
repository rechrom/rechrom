#![allow(non_snake_case)]
use crate::persistent_document::{DOMNamespace, DOMNodeType};
use crate::{Child, Document, ParsedDocument};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, ComputedStyle, Display, Edges, ExtendedStyle, ListStyleType, MathShift,
    MathStyle, TextAlign, TextDirection, TextTransform, VerticalAlign, WritingMode,
};
use layoutng_assembly::internal::paint_input::PaintTransformOrigin;

// Read-only ownership adapters. Both call the same source initial-style logic;
// the persistent path never serializes, reparses or copies the DOM arena.
trait NodeView {
    fn namespace(&self) -> DOMNamespace;
    fn name(&self) -> &str;
    fn attribute(&self, name: &str) -> Option<&str>;
    fn parent(&self) -> Option<(DOMNamespace, &str)>;
    fn element_index(&self) -> usize;
}
struct PersistentNode<'a> {
    document: &'a Document,
    index: usize,
}
impl NodeView for PersistentNode<'_> {
    fn namespace(&self) -> DOMNamespace {
        self.document.Node(self.index).Namespace()
    }
    fn name(&self) -> &str {
        self.document.Node(self.index).Name()
    }
    fn attribute(&self, name: &str) -> Option<&str> {
        self.document
            .Node(self.index)
            .FindAttribute(name)
            .map(|a| a.value.as_str())
    }
    fn parent(&self) -> Option<(DOMNamespace, &str)> {
        self.document.Node(self.index).Parent().map(|i| {
            let p = self.document.Node(i);
            (p.Namespace(), p.Name())
        })
    }
    fn element_index(&self) -> usize {
        self.document.Node(self.index).Parent().map_or(0, |i| {
            self.document
                .Node(i)
                .Children()
                .iter()
                .take_while(|&&i| i != self.index)
                .filter(|&&i| self.document.Node(i).Type() == DOMNodeType::kElement)
                .count()
        })
    }
}
struct StaticNode<'a> {
    document: &'a ParsedDocument,
    index: usize,
}
impl NodeView for StaticNode<'_> {
    fn namespace(&self) -> DOMNamespace {
        self.document.elements[self.index].namespace
    }
    fn name(&self) -> &str {
        &self.document.elements[self.index].tag
    }
    fn attribute(&self, name: &str) -> Option<&str> {
        self.document.elements[self.index]
            .attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
    fn parent(&self) -> Option<(DOMNamespace, &str)> {
        self.document.elements[self.index].parent.map(|i| {
            let p = &self.document.elements[i];
            (p.namespace, p.tag.as_str())
        })
    }
    fn element_index(&self) -> usize {
        self.document.elements[self.index].parent.map_or(0, |p| {
            self.document.elements[p]
                .children
                .iter()
                .take_while(|c| !matches!(c,Child::Element(i)if *i==self.index))
                .filter(|c| matches!(c, Child::Element(_)))
                .count()
        })
    }
}
pub fn InitialStyle(
    document: &Document,
    index: usize,
    parent: Option<&ComputedStyle>,
    pseudo: bool,
) -> ComputedStyle {
    initial(&PersistentNode { document, index }, parent, pseudo)
}
pub(crate) fn StaticInitialStyle(
    document: &ParsedDocument,
    index: usize,
    parent: Option<&ComputedStyle>,
    pseudo: bool,
) -> ComputedStyle {
    initial(&StaticNode { document, index }, parent, pseudo)
}
fn Extra(style: &mut ComputedStyle) -> &mut ExtendedStyle {
    style.extended.get_or_insert_with(Default::default)
}
// cpp: style_resolver/style_resolver.cc:3850-3859
fn IsHTMLBlockElement(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "body"
            | "center"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "frame"
            | "frameset"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "legend"
            | "listing"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "plaintext"
            | "pre"
            | "optgroup"
            | "option"
            | "search"
            | "section"
            | "summary"
            | "ul"
            | "xmp"
    )
}
// cpp: style_resolver/style_resolver.cc:3861-3866
fn IsHTMLInlineBlockElement(name: &str) -> bool {
    matches!(
        name,
        "button"
            | "img"
            | "input"
            | "marquee"
            | "meter"
            | "progress"
            | "select"
            | "textarea"
            | "keygen"
    )
}
// cpp: style_resolver/style_resolver.cc:3876-3910
fn initial(node: &impl NodeView, parent: Option<&ComputedStyle>, pseudo: bool) -> ComputedStyle {
    let mut style = ComputedStyle::default();
    let name = node.name();
    let ns = node.namespace();
    style.display = Display::kInline;
    style.border_styles.fill(BorderLineStyle::kNone);
    if !pseudo {
        style.display = if ns == DOMNamespace::kMathML {
            if name == "math" {
                Display::kMath
            } else {
                Display::kBlockMath
            }
        } else if name == "ruby" {
            Display::kRuby
        } else if name == "rt" {
            Display::kRubyText
        } else if ns == DOMNamespace::kHTML && IsHTMLInlineBlockElement(name) {
            Display::kInlineBlock
        } else {
            match name {
                "table" => Display::kTable,
                "thead" => Display::kTableHeaderGroup,
                "tbody" => Display::kTableSection,
                "tfoot" => Display::kTableFooterGroup,
                "tr" => Display::kTableRow,
                "td" | "th" => Display::kTableCell,
                "caption" => Display::kTableCaption,
                "colgroup" => Display::kTableColumnGroup,
                "col" => Display::kTableColumn,
                "li" => Display::kListItem,
                _ if ns == DOMNamespace::kHTML && IsHTMLBlockElement(name) => Display::kBlock,
                _ => Display::kInline,
            }
        };
    }
    if !pseudo && name == "body" {
        style.margin = Edges {
            top: 8.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0,
        };
    }
    // cpp: style_resolver/style_resolver.cc:3910-3994
    if let Some(parent) = parent {
        style.direction = parent.direction;
        style.writing_mode = parent.writing_mode;
        style.paint.color = parent.paint.color.clone();
        style.paint.visible = parent.paint.visible.clone();
        style.paint.pointer_events_none = parent.paint.pointer_events_none;
        style.paint.cursor = parent.paint.cursor;
        style.paint.text_shadows = parent.paint.text_shadows.clone();
        style.paint.svg_fill = parent.paint.svg_fill.clone();
        style.paint.svg_stroke = parent.paint.svg_stroke.clone();
        style.paint.svg_fill_current_color = parent.paint.svg_fill_current_color.clone();
        style.paint.svg_stroke_current_color = parent.paint.svg_stroke_current_color.clone();
        style.paint.svg_fill_server = parent.paint.svg_fill_server.clone();
        style.paint.svg_stroke_server = parent.paint.svg_stroke_server.clone();
        style.paint.svg_fill_reference = parent.paint.svg_fill_reference.clone();
        style.paint.svg_stroke_reference = parent.paint.svg_stroke_reference.clone();
        style.paint.svg_stroke_width = parent.paint.svg_stroke_width.clone();
        style.paint.svg_stroke_dash_array = parent.paint.svg_stroke_dash_array.clone();
        style.paint.svg_stroke_dash_offset = parent.paint.svg_stroke_dash_offset.clone();
        style.paint.svg_stroke_line_cap = parent.paint.svg_stroke_line_cap.clone();
        style.paint.svg_stroke_line_join = parent.paint.svg_stroke_line_join.clone();
        style.paint.svg_stroke_miter_limit = parent.paint.svg_stroke_miter_limit.clone();
        style.paint.svg_fill_even_odd = parent.paint.svg_fill_even_odd.clone();
        style.paint.svg_non_scaling_stroke = parent.paint.svg_non_scaling_stroke.clone();
        style.paint.svg_shape_antialias = parent.paint.svg_shape_antialias.clone();
        style.paint.svg_paint_order = parent.paint.svg_paint_order.clone();
        if let Some(parent) = &parent.extended {
            let extra = Extra(&mut style);
            extra.effective_zoom = parent.effective_zoom;
            extra.font_size = parent.font_size.clone();
            extra.font_families = parent.font_families.clone();
            extra.font_weight = parent.font_weight.clone();
            extra.font_italic = parent.font_italic.clone();
            extra.font_smoothing = parent.font_smoothing.clone();
            extra.line_height = parent.line_height.clone();
            extra.line_height_percent = parent.line_height_percent.clone();
            extra.white_space = parent.white_space.clone();
            extra.text_align = parent.text_align.clone();
            extra.text_align_last = parent.text_align_last.clone();
            extra.language = parent.language.clone();
            extra.letter_spacing = parent.letter_spacing.clone();
            extra.word_spacing = parent.word_spacing.clone();
            extra.text_indent = parent.text_indent.clone();
            extra.text_indent_percent = parent.text_indent_percent.clone();
            extra.text_indent_calculated = parent.text_indent_calculated.clone();
            extra.text_indent_each_line = parent.text_indent_each_line.clone();
            extra.text_indent_hanging = parent.text_indent_hanging.clone();
            extra.overflow_wrap = parent.overflow_wrap.clone();
            extra.word_break = parent.word_break.clone();
            extra.line_break = parent.line_break.clone();
            extra.hyphens = parent.hyphens.clone();
            extra.tab_size = parent.tab_size.clone();
            extra.tab_size_is_length = parent.tab_size_is_length.clone();
            extra.text_orientation = parent.text_orientation.clone();
            extra.text_combine = parent.text_combine.clone();
            extra.text_transform = parent.text_transform.clone();
            extra.math_depth = parent.math_depth.clone();
            extra.math_style = parent.math_style.clone();
            extra.math_shift = parent.math_shift.clone();
            extra.text_wrap_mode = parent.text_wrap_mode.clone();
            extra.text_wrap_style = parent.text_wrap_style.clone();
            extra.ruby_position = parent.ruby_position.clone();
            extra.ruby_align = parent.ruby_align.clone();
            extra.ruby_overhang = parent.ruby_overhang.clone();
            extra.list_style_type = parent.list_style_type.clone();
            extra.list_style_position = parent.list_style_position.clone();
            extra.empty_cells = parent.empty_cells.clone();
            extra.border_collapse = parent.border_collapse.clone();
            extra.border_spacing = parent.border_spacing.clone();
            extra.vertical_border_spacing = parent.vertical_border_spacing.clone();
            extra.caption_side = parent.caption_side.clone();
            extra.widows = parent.widows.clone();
            extra.orphans = parent.orphans.clone();
            extra.vertical_align = parent.vertical_align.clone();
            extra.vertical_align_length = parent.vertical_align_length.clone();
            extra.vertical_align_percent = parent.vertical_align_percent.clone();
            extra.vertical_align_calculated = parent.vertical_align_calculated.clone();
        }
    }
    // cpp: style_resolver/style_resolver.cc:3995-4085
    if !pseudo && ns == DOMNamespace::kSVG && name != "svg" {
        style.paint.transform_origin = Some(PaintTransformOrigin::default());
    }
    if !pseudo && ns == DOMNamespace::kHTML {
        if matches!(name, "button" | "input" | "select" | "textarea") {
            let e = Extra(&mut style);
            e.font_size = 40.0 / 3.0;
            e.font_families = vec!["Arial".into()];
            e.font_weight = 400.0;
            e.font_italic = false;
            e.line_height = None;
            e.line_height_percent = None;
        }
        if matches!(name, "thead" | "tbody" | "tfoot") {
            Extra(&mut style).vertical_align = VerticalAlign::kMiddle;
        } else if matches!(name, "tr" | "td" | "th") {
            if let Some(parent) = parent.and_then(|p| p.extended.as_ref()) {
                let e = Extra(&mut style);
                e.vertical_align = parent.vertical_align;
                e.vertical_align_length = parent.vertical_align_length;
                e.vertical_align_percent = parent.vertical_align_percent;
                e.vertical_align_calculated = parent.vertical_align_calculated;
            }
        }
        if matches!(name, "th" | "caption") {
            Extra(&mut style).text_align = TextAlign::kCenter;
        }
        if name == "p" {
            let size = Extra(&mut style).font_size;
            style.margin.top = size;
            style.margin.bottom = size;
        }
        if name == "figure" {
            let size = Extra(&mut style).font_size;
            style.margin = Edges {
                top: size,
                right: 40.0,
                bottom: size,
                left: 40.0,
            };
        }
        if matches!(
            name,
            "b" | "strong" | "th" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
        ) {
            Extra(&mut style).font_weight = 700.0;
        }
        if matches!(name, "i" | "em" | "cite" | "var") {
            Extra(&mut style).font_italic = true;
        }
        if name == "small" {
            Extra(&mut style).font_size *= 5.0 / 6.0;
        }
        if matches!(name, "sub" | "sup") {
            let e = Extra(&mut style);
            e.font_size *= 5.0 / 6.0;
            e.vertical_align = if name == "sub" {
                VerticalAlign::kSub
            } else {
                VerticalAlign::kSuper
            };
        }
        if matches!(name, "ruby" | "rt") {
            let e = Extra(&mut style);
            e.text_indent = 0.0;
            e.text_indent_percent = Some(0.0);
            e.text_indent_calculated = false;
        }
        if name == "rt" {
            let e = Extra(&mut style);
            e.line_height = None;
            e.line_height_percent = None;
            if node.parent() == Some((DOMNamespace::kHTML, "ruby")) {
                e.font_size *= 0.5;
                e.text_align = TextAlign::kStart;
            }
        }
    }
    // cpp: style_resolver/style_resolver.cc:4086-4183
    if !pseudo && ns == DOMNamespace::kMathML && name == "mi" {
        Extra(&mut style).text_transform = TextTransform::kMathAuto;
    }
    if !pseudo && ns == DOMNamespace::kMathML {
        style.writing_mode = WritingMode::kHorizontalTb;
        match name {
            "mtable" => style.display = Display::kInlineTable,
            "mtr" | "mlabeledtr" => style.display = Display::kTableRow,
            "mtd" => style.display = Display::kTableCell,
            _ => {}
        }
        if name == "mtable" {
            let e = Extra(&mut style);
            e.math_style = MathStyle::kCompact;
            e.border_spacing = 0.0;
            e.vertical_border_spacing = Some(0.0);
        } else if name == "mtd" {
            let e = Extra(&mut style);
            e.text_align = TextAlign::kCenter;
            let size = e.font_size;
            style.padding = Edges {
                top: size * 0.2365,
                right: size * 0.4,
                bottom: size * 0.2365,
                left: size * 0.4,
            };
        }
        if name == "mfrac" {
            style.padding.left = 1.0;
            style.padding.right = 1.0;
        }
        if name == "math" {
            style.direction = TextDirection::kLtr;
            let e = Extra(&mut style);
            e.font_families = vec!["math".into()];
            e.text_indent = 0.0;
            e.letter_spacing = 0.0;
            e.word_spacing = 0.0;
            e.line_height = None;
            e.line_height_percent = None;
            e.font_weight = 400.0;
            e.font_italic = false;
            e.math_depth = 0;
            e.math_style = MathStyle::kCompact;
            e.math_shift = MathShift::kNormal;
            e.font_size_math = false;
            if node.attribute("display").is_some_and(|v| {
                v.trim_matches(|c: char| c == ' ' || ('\t'..='\r').contains(&c))
                    .eq_ignore_ascii_case("block")
            }) {
                style.display = Display::kBlockMath;
                Extra(&mut style).math_style = MathStyle::kNormal;
            }
        } else {
            Extra(&mut style).font_size_math = true;
            if let Some((DOMNamespace::kMathML, parent_name)) = node.parent() {
                let index = node.element_index();
                let e = Extra(&mut style);
                if parent_name == "mfrac" {
                    if e.math_style == MathStyle::kCompact {
                        e.math_depth += 1;
                    }
                    e.math_style = MathStyle::kCompact;
                } else if parent_name == "mroot" && index > 0 {
                    e.math_depth += 2;
                    e.math_style = MathStyle::kCompact;
                } else if matches!(
                    parent_name,
                    "msub"
                        | "msup"
                        | "msubsup"
                        | "mmultiscripts"
                        | "munder"
                        | "mover"
                        | "munderover"
                ) && index > 0
                {
                    e.math_depth += 1;
                    e.math_style = MathStyle::kCompact;
                }
                if (parent_name == "mfrac" && index == 1)
                    || (matches!(parent_name, "msub" | "msubsup" | "mmultiscripts") && index == 1)
                {
                    e.math_shift = MathShift::kCompact;
                }
            }
            if matches!(name, "mroot" | "msqrt") {
                Extra(&mut style).math_shift = MathShift::kCompact;
            }
        }
    }
    // cpp: style_resolver/style_resolver.cc:4184-4199
    if !pseudo && ns == DOMNamespace::kHTML && name == "ol" {
        Extra(&mut style).list_style_type = ListStyleType::kDecimal;
    } else if ns == DOMNamespace::kHTML && matches!(name, "ul" | "menu" | "dir") {
        Extra(&mut style).list_style_type = ListStyleType::kDisc;
    }
    if !pseudo {
        if let Some(lang) = node.attribute("lang") {
            Extra(&mut style).language = lang.into();
        } else if let Some(lang) = node.attribute("xml:lang") {
            Extra(&mut style).language = lang.into();
        }
        if let Some(dir) = node.attribute("dir") {
            style.direction = if dir.eq_ignore_ascii_case("rtl") {
                TextDirection::kRtl
            } else {
                TextDirection::kLtr
            };
        }
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{persistent_document::DOMAttribute, DOM};
    use layoutng_assembly::internal::layout_input::{OverflowWrap, TextWrapMode, TextWrapStyle};

    fn element(
        document: &mut Document,
        parent: usize,
        namespace: DOMNamespace,
        name: &str,
    ) -> usize {
        let index = document.CreateElementDefault(namespace, name.into());
        document.AppendChild(parent, index);
        index
    }
    fn attribute(document: &mut Document, index: usize, name: &str, value: &str) {
        document.SetAttribute(
            index,
            DOMAttribute {
                local_name: name.into(),
                value: value.into(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn persistent_inheritance_control_defaults_and_live_ruby_reparenting() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let root = document.Root();
        let ruby = element(document, root, DOMNamespace::kHTML, "ruby");
        let rt = element(document, ruby, DOMNamespace::kHTML, "rt");
        let paragraph = element(document, root, DOMNamespace::kHTML, "p");
        let input = element(document, root, DOMNamespace::kHTML, "input");
        let svg_input = element(document, root, DOMNamespace::kSVG, "input");
        let mut parent = ComputedStyle::default();
        parent.width = Some(123.0);
        parent.direction = TextDirection::kRtl;
        let extra = Extra(&mut parent);
        extra.font_size = 24.0;
        extra.font_families = vec!["Source Sans".into()];
        extra.font_weight = 600.0;
        extra.line_height = Some(32.0);
        extra.text_indent = 19.0;
        extra.text_indent_percent = Some(0.25);
        extra.text_indent_each_line = true;
        extra.overflow_wrap = OverflowWrap::kAnywhere;
        extra.text_wrap_mode = TextWrapMode::kNowrap;
        extra.text_wrap_style = TextWrapStyle::kBalance;
        let p = InitialStyle(document, paragraph, Some(&parent), false);
        let e = p.extended.as_ref().unwrap();
        assert_eq!(p.width, None);
        assert_eq!(p.direction, TextDirection::kRtl);
        assert_eq!((p.margin.top, p.margin.bottom), (24.0, 24.0));
        assert_eq!(e.text_indent, 19.0);
        assert_eq!(e.text_indent_percent, Some(0.25));
        assert!(e.text_indent_each_line);
        assert_eq!(e.overflow_wrap, OverflowWrap::kAnywhere);
        assert_eq!(e.text_wrap_mode, TextWrapMode::kNowrap);
        assert_eq!(e.text_wrap_style, TextWrapStyle::kBalance);
        let annotation = InitialStyle(document, rt, Some(&parent), false);
        let e = annotation.extended.as_ref().unwrap();
        assert_eq!(e.font_size, 12.0);
        assert_eq!(e.text_indent, 0.0);
        assert_eq!(e.text_indent_percent, Some(0.0));
        assert_eq!(e.line_height, None);
        document.AppendChild(paragraph, rt);
        let moved = InitialStyle(document, rt, Some(&parent), false);
        assert_eq!(moved.extended.as_ref().unwrap().font_size, 24.0);
        attribute(document, paragraph, "dir", "LTR");
        attribute(document, paragraph, "lang", "zh-CN");
        let changed = InitialStyle(document, paragraph, Some(&parent), false);
        assert_eq!(changed.direction, TextDirection::kLtr);
        assert_eq!(changed.extended.as_ref().unwrap().language, "zh-CN");
        let control = InitialStyle(document, input, Some(&parent), false);
        assert_eq!(control.display, Display::kInlineBlock);
        let e = control.extended.as_ref().unwrap();
        assert_eq!(e.font_size, 40.0 / 3.0);
        assert_eq!(e.font_families, ["Arial"]);
        assert_eq!(e.font_weight, 400.0);
        assert_eq!(e.line_height, None);
        let foreign = InitialStyle(document, svg_input, Some(&parent), false);
        assert_eq!(foreign.display, Display::kInline);
        assert_eq!(foreign.extended.as_ref().unwrap().font_size, 24.0);
        assert!(foreign.paint.transform_origin.is_some());
    }

    #[test]
    fn math_depth_tracks_element_siblings_in_the_current_arena() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let root = document.Root();
        let math = element(document, root, DOMNamespace::kMathML, "math");
        attribute(document, math, "display", " \tBLOCK\r\n");
        let fraction = element(document, math, DOMNamespace::kMathML, "mfrac");
        let comment = document.CreateComment("ignored sibling".into());
        document.AppendChild(fraction, comment);
        document.AppendText(fraction, " ");
        let numerator = element(document, fraction, DOMNamespace::kMathML, "mi");
        let denominator = element(document, fraction, DOMNamespace::kMathML, "mi");
        let math_style = InitialStyle(document, math, None, false);
        assert_eq!(math_style.display, Display::kBlockMath);
        assert_eq!(
            math_style.extended.as_ref().unwrap().math_style,
            MathStyle::kNormal
        );
        let fraction_style = InitialStyle(document, fraction, Some(&math_style), false);
        let top = InitialStyle(document, numerator, Some(&fraction_style), false);
        let bottom = InitialStyle(document, denominator, Some(&fraction_style), false);
        let e = top.extended.as_ref().unwrap();
        assert_eq!(e.math_depth, 0);
        assert_eq!(e.math_style, MathStyle::kCompact);
        assert_eq!(e.math_shift, MathShift::kNormal);
        assert_eq!(e.text_transform, TextTransform::kMathAuto);
        assert_eq!(
            bottom.extended.as_ref().unwrap().math_shift,
            MathShift::kCompact
        );
        document.InsertBefore(fraction, denominator, numerator);
        assert_eq!(
            InitialStyle(document, numerator, Some(&fraction_style), false)
                .extended
                .as_ref()
                .unwrap()
                .math_shift,
            MathShift::kCompact
        );
        attribute(document, math, "display", "inline");
        let inline_math = InitialStyle(document, math, None, false);
        let inline_fraction = InitialStyle(document, fraction, Some(&inline_math), false);
        assert_eq!(
            InitialStyle(document, numerator, Some(&inline_fraction), false)
                .extended
                .as_ref()
                .unwrap()
                .math_depth,
            1
        );
        let radical = element(document, math, DOMNamespace::kMathML, "mroot");
        let base = element(document, radical, DOMNamespace::kMathML, "mi");
        let degree = element(document, radical, DOMNamespace::kMathML, "mi");
        let radical_style = InitialStyle(document, radical, Some(&math_style), false);
        assert_eq!(
            InitialStyle(document, base, Some(&radical_style), false)
                .extended
                .as_ref()
                .unwrap()
                .math_depth,
            0
        );
        assert_eq!(
            InitialStyle(document, degree, Some(&radical_style), false)
                .extended
                .as_ref()
                .unwrap()
                .math_depth,
            2
        );
    }

    #[test]
    fn html_parser_and_static_adapter_share_initial_style_logic() {
        let markup = "<html><body><figure id='figure'><ruby id='ruby'>x<rt id='rt'>y</rt></ruby></figure><table id='table'><tfoot id='footer'><tr><th id='cell'>z</th></tr></tfoot></table><svg id='svg'><g id='group'></g></svg><input id='control'></body></html>";
        let owner = crate::test_html::html_parser::ParseHTML(markup);
        let persistent = owner.GetDocument();
        let parsed = crate::test_html::Parse(markup);
        let mut persistent_styles: Vec<Option<ComputedStyle>> = vec![None; persistent.NodeCount()];
        fn walk(
            document: &Document,
            index: usize,
            parent: Option<&ComputedStyle>,
            styles: &mut [Option<ComputedStyle>],
        ) {
            let style = (document.Node(index).Type() == DOMNodeType::kElement)
                .then(|| InitialStyle(document, index, parent, false));
            for &child in document.Node(index).Children() {
                walk(document, child, style.as_ref().or(parent), styles);
            }
            styles[index] = style;
        }
        walk(persistent, persistent.Root(), None, &mut persistent_styles);
        let mut static_styles = Vec::<ComputedStyle>::new();
        for (index, element) in parsed.elements.iter().enumerate() {
            let style = StaticInitialStyle(
                &parsed,
                index,
                element.parent.map(|p| &static_styles[p]),
                false,
            );
            if let Some(id) = &element.id {
                let live_index = (0..persistent.NodeCount())
                    .find(|&i| {
                        persistent
                            .Node(i)
                            .FindAttribute("id")
                            .is_some_and(|a| &a.value == id)
                    })
                    .unwrap();
                let live = persistent_styles[live_index].as_ref().unwrap();
                assert_eq!(style.display, live.display, "{id}");
                assert_eq!(style.margin, live.margin, "{id}");
                assert_eq!(style.border_styles, live.border_styles, "{id}");
                let (static_extra, live_extra) = (style.extended.as_ref(), live.extended.as_ref());
                assert_eq!(
                    static_extra.map(|e| e.font_size),
                    live_extra.map(|e| e.font_size),
                    "{id}"
                );
                assert_eq!(
                    static_extra.map(|e| e.vertical_align),
                    live_extra.map(|e| e.vertical_align),
                    "{id}"
                );
                assert_eq!(
                    static_extra.map(|e| e.text_align),
                    live_extra.map(|e| e.text_align),
                    "{id}"
                );
            }
            static_styles.push(style);
        }
    }

    #[test]
    fn persistent_initial_values_match_current_cpp_reference() {
        let markup = include_str!("../../../../artifacts/cpp-reference/initial-style.html");
        let expected =
            include_str!("../../../../artifacts/cpp-reference/initial-style-results.tsv");
        let owner = crate::test_html::html_parser::ParseHTML(markup);
        let document = owner.GetDocument();
        let mut values = std::collections::HashMap::new();
        fn walk(
            document: &Document,
            index: usize,
            parent: Option<&ComputedStyle>,
            values: &mut std::collections::HashMap<String, Vec<f64>>,
        ) {
            let mut style = (document.Node(index).Type() == DOMNodeType::kElement)
                .then(|| InitialStyle(document, index, parent, false));
            if let Some(s) = style.as_mut() {
                if let Some(id) = document.Node(index).FindAttribute("id") {
                    let display = s.display as i32 as f64;
                    let direction = s.direction as i32 as f64;
                    let margin = [s.margin.top, s.margin.right, s.margin.bottom, s.margin.left];
                    let padding = [
                        s.padding.top,
                        s.padding.right,
                        s.padding.bottom,
                        s.padding.left,
                    ];
                    let e = Extra(s);
                    let mut fields = vec![
                        display,
                        e.font_size,
                        e.font_weight,
                        e.font_italic as u8 as f64,
                        e.vertical_align as i32 as f64,
                        e.text_align as i32 as f64,
                    ];
                    fields.extend(margin);
                    fields.extend(padding);
                    fields.extend([
                        e.math_depth as f64,
                        e.math_style as i32 as f64,
                        e.math_shift as i32 as f64,
                        e.text_indent,
                        e.font_size_math as u8 as f64,
                        direction,
                    ]);
                    values.insert(id.value.clone(), fields);
                }
            }
            for &child in document.Node(index).Children() {
                walk(document, child, style.as_ref().or(parent), values);
            }
        }
        walk(document, document.Root(), None, &mut values);
        for line in expected.lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let expected: Vec<f64> = fields[1..].iter().map(|n| n.parse().unwrap()).collect();
            let actual = values
                .remove(fields[0])
                .expect("reference node exists in Rust parser arena");
            assert_eq!(actual, expected, "{}", fields[0]);
        }
        assert!(values.is_empty());
    }
}
