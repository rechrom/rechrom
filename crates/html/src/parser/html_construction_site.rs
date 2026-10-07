#![allow(non_snake_case, non_upper_case_globals)]

use super::html_token::{HTMLToken, TokenType};
use crate::html_parser_host::{HTMLParserHost, ParserElementPhase};
use dom::error::{invalid_argument, logic_error};
use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMNodeType, PersistentDocument};

// cpp: html/parser/html_construction_site.cc:18-21
#[derive(Clone, Copy)]
struct CaseAdjustment {
    lowered: &'static str,
    adjusted: &'static str,
}

// cpp: html/parser/html_construction_site.cc:26
const kSVGTagAdjustments: &[CaseAdjustment] = &[
    // cpp: html/parser/html_construction_site.cc:27
    CaseAdjustment {
        lowered: "altglyph",
        adjusted: "altGlyph",
    },
    // cpp: html/parser/html_construction_site.cc:28
    CaseAdjustment {
        lowered: "altglyphdef",
        adjusted: "altGlyphDef",
    },
    // cpp: html/parser/html_construction_site.cc:29
    CaseAdjustment {
        lowered: "altglyphitem",
        adjusted: "altGlyphItem",
    },
    // cpp: html/parser/html_construction_site.cc:30
    CaseAdjustment {
        lowered: "animatecolor",
        adjusted: "animateColor",
    },
    // cpp: html/parser/html_construction_site.cc:31
    CaseAdjustment {
        lowered: "animatemotion",
        adjusted: "animateMotion",
    },
    // cpp: html/parser/html_construction_site.cc:32
    CaseAdjustment {
        lowered: "animatetransform",
        adjusted: "animateTransform",
    },
    // cpp: html/parser/html_construction_site.cc:33
    CaseAdjustment {
        lowered: "clippath",
        adjusted: "clipPath",
    },
    // cpp: html/parser/html_construction_site.cc:34
    CaseAdjustment {
        lowered: "feblend",
        adjusted: "feBlend",
    },
    // cpp: html/parser/html_construction_site.cc:35
    CaseAdjustment {
        lowered: "fecolormatrix",
        adjusted: "feColorMatrix",
    },
    // cpp: html/parser/html_construction_site.cc:36
    CaseAdjustment {
        lowered: "fecomponenttransfer",
        adjusted: "feComponentTransfer",
    },
    // cpp: html/parser/html_construction_site.cc:37
    CaseAdjustment {
        lowered: "fecomposite",
        adjusted: "feComposite",
    },
    // cpp: html/parser/html_construction_site.cc:38
    CaseAdjustment {
        lowered: "feconvolvematrix",
        adjusted: "feConvolveMatrix",
    },
    // cpp: html/parser/html_construction_site.cc:39
    CaseAdjustment {
        lowered: "fediffuselighting",
        adjusted: "feDiffuseLighting",
    },
    // cpp: html/parser/html_construction_site.cc:40
    CaseAdjustment {
        lowered: "fedisplacementmap",
        adjusted: "feDisplacementMap",
    },
    // cpp: html/parser/html_construction_site.cc:41
    CaseAdjustment {
        lowered: "fedistantlight",
        adjusted: "feDistantLight",
    },
    // cpp: html/parser/html_construction_site.cc:42
    CaseAdjustment {
        lowered: "fedropshadow",
        adjusted: "feDropShadow",
    },
    // cpp: html/parser/html_construction_site.cc:43
    CaseAdjustment {
        lowered: "feflood",
        adjusted: "feFlood",
    },
    // cpp: html/parser/html_construction_site.cc:44
    CaseAdjustment {
        lowered: "fefunca",
        adjusted: "feFuncA",
    },
    // cpp: html/parser/html_construction_site.cc:45
    CaseAdjustment {
        lowered: "fefuncb",
        adjusted: "feFuncB",
    },
    // cpp: html/parser/html_construction_site.cc:46
    CaseAdjustment {
        lowered: "fefuncg",
        adjusted: "feFuncG",
    },
    // cpp: html/parser/html_construction_site.cc:47
    CaseAdjustment {
        lowered: "fefuncr",
        adjusted: "feFuncR",
    },
    // cpp: html/parser/html_construction_site.cc:48
    CaseAdjustment {
        lowered: "fegaussianblur",
        adjusted: "feGaussianBlur",
    },
    // cpp: html/parser/html_construction_site.cc:49
    CaseAdjustment {
        lowered: "feimage",
        adjusted: "feImage",
    },
    // cpp: html/parser/html_construction_site.cc:50
    CaseAdjustment {
        lowered: "femerge",
        adjusted: "feMerge",
    },
    // cpp: html/parser/html_construction_site.cc:51
    CaseAdjustment {
        lowered: "femergenode",
        adjusted: "feMergeNode",
    },
    // cpp: html/parser/html_construction_site.cc:52
    CaseAdjustment {
        lowered: "femorphology",
        adjusted: "feMorphology",
    },
    // cpp: html/parser/html_construction_site.cc:53
    CaseAdjustment {
        lowered: "feoffset",
        adjusted: "feOffset",
    },
    // cpp: html/parser/html_construction_site.cc:54
    CaseAdjustment {
        lowered: "fepointlight",
        adjusted: "fePointLight",
    },
    // cpp: html/parser/html_construction_site.cc:55
    CaseAdjustment {
        lowered: "fespecularlighting",
        adjusted: "feSpecularLighting",
    },
    // cpp: html/parser/html_construction_site.cc:56
    CaseAdjustment {
        lowered: "fespotlight",
        adjusted: "feSpotLight",
    },
    // cpp: html/parser/html_construction_site.cc:57
    CaseAdjustment {
        lowered: "fetile",
        adjusted: "feTile",
    },
    // cpp: html/parser/html_construction_site.cc:58
    CaseAdjustment {
        lowered: "feturbulence",
        adjusted: "feTurbulence",
    },
    // cpp: html/parser/html_construction_site.cc:59
    CaseAdjustment {
        lowered: "foreignobject",
        adjusted: "foreignObject",
    },
    // cpp: html/parser/html_construction_site.cc:60
    CaseAdjustment {
        lowered: "glyphref",
        adjusted: "glyphRef",
    },
    // cpp: html/parser/html_construction_site.cc:61
    CaseAdjustment {
        lowered: "lineargradient",
        adjusted: "linearGradient",
    },
    // cpp: html/parser/html_construction_site.cc:62
    CaseAdjustment {
        lowered: "radialgradient",
        adjusted: "radialGradient",
    },
    // cpp: html/parser/html_construction_site.cc:63
    CaseAdjustment {
        lowered: "textpath",
        adjusted: "textPath",
    },
];

// cpp: html/parser/html_construction_site.cc:69
const kSVGAttributeAdjustments: &[CaseAdjustment] = &[
    // cpp: html/parser/html_construction_site.cc:70
    CaseAdjustment {
        lowered: "attributename",
        adjusted: "attributeName",
    },
    // cpp: html/parser/html_construction_site.cc:71
    CaseAdjustment {
        lowered: "attributetype",
        adjusted: "attributeType",
    },
    // cpp: html/parser/html_construction_site.cc:72
    CaseAdjustment {
        lowered: "basefrequency",
        adjusted: "baseFrequency",
    },
    // cpp: html/parser/html_construction_site.cc:73
    CaseAdjustment {
        lowered: "baseprofile",
        adjusted: "baseProfile",
    },
    // cpp: html/parser/html_construction_site.cc:74
    CaseAdjustment {
        lowered: "calcmode",
        adjusted: "calcMode",
    },
    // cpp: html/parser/html_construction_site.cc:75
    CaseAdjustment {
        lowered: "clippathunits",
        adjusted: "clipPathUnits",
    },
    // cpp: html/parser/html_construction_site.cc:76
    CaseAdjustment {
        lowered: "diffuseconstant",
        adjusted: "diffuseConstant",
    },
    // cpp: html/parser/html_construction_site.cc:77
    CaseAdjustment {
        lowered: "edgemode",
        adjusted: "edgeMode",
    },
    // cpp: html/parser/html_construction_site.cc:78
    CaseAdjustment {
        lowered: "filterunits",
        adjusted: "filterUnits",
    },
    // cpp: html/parser/html_construction_site.cc:79
    CaseAdjustment {
        lowered: "glyphref",
        adjusted: "glyphRef",
    },
    // cpp: html/parser/html_construction_site.cc:80
    CaseAdjustment {
        lowered: "gradienttransform",
        adjusted: "gradientTransform",
    },
    // cpp: html/parser/html_construction_site.cc:81
    CaseAdjustment {
        lowered: "gradientunits",
        adjusted: "gradientUnits",
    },
    // cpp: html/parser/html_construction_site.cc:82
    CaseAdjustment {
        lowered: "kernelmatrix",
        adjusted: "kernelMatrix",
    },
    // cpp: html/parser/html_construction_site.cc:83
    CaseAdjustment {
        lowered: "kernelunitlength",
        adjusted: "kernelUnitLength",
    },
    // cpp: html/parser/html_construction_site.cc:84
    CaseAdjustment {
        lowered: "keypoints",
        adjusted: "keyPoints",
    },
    // cpp: html/parser/html_construction_site.cc:85
    CaseAdjustment {
        lowered: "keysplines",
        adjusted: "keySplines",
    },
    // cpp: html/parser/html_construction_site.cc:86
    CaseAdjustment {
        lowered: "keytimes",
        adjusted: "keyTimes",
    },
    // cpp: html/parser/html_construction_site.cc:87
    CaseAdjustment {
        lowered: "lengthadjust",
        adjusted: "lengthAdjust",
    },
    // cpp: html/parser/html_construction_site.cc:88
    CaseAdjustment {
        lowered: "limitingconeangle",
        adjusted: "limitingConeAngle",
    },
    // cpp: html/parser/html_construction_site.cc:89
    CaseAdjustment {
        lowered: "markerheight",
        adjusted: "markerHeight",
    },
    // cpp: html/parser/html_construction_site.cc:90
    CaseAdjustment {
        lowered: "markerunits",
        adjusted: "markerUnits",
    },
    // cpp: html/parser/html_construction_site.cc:91
    CaseAdjustment {
        lowered: "markerwidth",
        adjusted: "markerWidth",
    },
    // cpp: html/parser/html_construction_site.cc:92
    CaseAdjustment {
        lowered: "maskcontentunits",
        adjusted: "maskContentUnits",
    },
    // cpp: html/parser/html_construction_site.cc:93
    CaseAdjustment {
        lowered: "maskunits",
        adjusted: "maskUnits",
    },
    // cpp: html/parser/html_construction_site.cc:94
    CaseAdjustment {
        lowered: "numoctaves",
        adjusted: "numOctaves",
    },
    // cpp: html/parser/html_construction_site.cc:95
    CaseAdjustment {
        lowered: "pathlength",
        adjusted: "pathLength",
    },
    // cpp: html/parser/html_construction_site.cc:96
    CaseAdjustment {
        lowered: "patterncontentunits",
        adjusted: "patternContentUnits",
    },
    // cpp: html/parser/html_construction_site.cc:97
    CaseAdjustment {
        lowered: "patterntransform",
        adjusted: "patternTransform",
    },
    // cpp: html/parser/html_construction_site.cc:98
    CaseAdjustment {
        lowered: "patternunits",
        adjusted: "patternUnits",
    },
    // cpp: html/parser/html_construction_site.cc:99
    CaseAdjustment {
        lowered: "pointsatx",
        adjusted: "pointsAtX",
    },
    // cpp: html/parser/html_construction_site.cc:100
    CaseAdjustment {
        lowered: "pointsaty",
        adjusted: "pointsAtY",
    },
    // cpp: html/parser/html_construction_site.cc:101
    CaseAdjustment {
        lowered: "pointsatz",
        adjusted: "pointsAtZ",
    },
    // cpp: html/parser/html_construction_site.cc:102
    CaseAdjustment {
        lowered: "preservealpha",
        adjusted: "preserveAlpha",
    },
    // cpp: html/parser/html_construction_site.cc:103
    CaseAdjustment {
        lowered: "preserveaspectratio",
        adjusted: "preserveAspectRatio",
    },
    // cpp: html/parser/html_construction_site.cc:104
    CaseAdjustment {
        lowered: "primitiveunits",
        adjusted: "primitiveUnits",
    },
    // cpp: html/parser/html_construction_site.cc:105
    CaseAdjustment {
        lowered: "refx",
        adjusted: "refX",
    },
    // cpp: html/parser/html_construction_site.cc:106
    CaseAdjustment {
        lowered: "refy",
        adjusted: "refY",
    },
    // cpp: html/parser/html_construction_site.cc:107
    CaseAdjustment {
        lowered: "repeatcount",
        adjusted: "repeatCount",
    },
    // cpp: html/parser/html_construction_site.cc:108
    CaseAdjustment {
        lowered: "repeatdur",
        adjusted: "repeatDur",
    },
    // cpp: html/parser/html_construction_site.cc:109
    CaseAdjustment {
        lowered: "requiredextensions",
        adjusted: "requiredExtensions",
    },
    // cpp: html/parser/html_construction_site.cc:110
    CaseAdjustment {
        lowered: "requiredfeatures",
        adjusted: "requiredFeatures",
    },
    // cpp: html/parser/html_construction_site.cc:111
    CaseAdjustment {
        lowered: "specularconstant",
        adjusted: "specularConstant",
    },
    // cpp: html/parser/html_construction_site.cc:112
    CaseAdjustment {
        lowered: "specularexponent",
        adjusted: "specularExponent",
    },
    // cpp: html/parser/html_construction_site.cc:113
    CaseAdjustment {
        lowered: "spreadmethod",
        adjusted: "spreadMethod",
    },
    // cpp: html/parser/html_construction_site.cc:114
    CaseAdjustment {
        lowered: "startoffset",
        adjusted: "startOffset",
    },
    // cpp: html/parser/html_construction_site.cc:115
    CaseAdjustment {
        lowered: "stddeviation",
        adjusted: "stdDeviation",
    },
    // cpp: html/parser/html_construction_site.cc:116
    CaseAdjustment {
        lowered: "stitchtiles",
        adjusted: "stitchTiles",
    },
    // cpp: html/parser/html_construction_site.cc:117
    CaseAdjustment {
        lowered: "surfacescale",
        adjusted: "surfaceScale",
    },
    // cpp: html/parser/html_construction_site.cc:118
    CaseAdjustment {
        lowered: "systemlanguage",
        adjusted: "systemLanguage",
    },
    // cpp: html/parser/html_construction_site.cc:119
    CaseAdjustment {
        lowered: "tablevalues",
        adjusted: "tableValues",
    },
    // cpp: html/parser/html_construction_site.cc:120
    CaseAdjustment {
        lowered: "targetx",
        adjusted: "targetX",
    },
    // cpp: html/parser/html_construction_site.cc:121
    CaseAdjustment {
        lowered: "targety",
        adjusted: "targetY",
    },
    // cpp: html/parser/html_construction_site.cc:122
    CaseAdjustment {
        lowered: "textlength",
        adjusted: "textLength",
    },
    // cpp: html/parser/html_construction_site.cc:123
    CaseAdjustment {
        lowered: "viewbox",
        adjusted: "viewBox",
    },
    // cpp: html/parser/html_construction_site.cc:124
    CaseAdjustment {
        lowered: "viewtarget",
        adjusted: "viewTarget",
    },
    // cpp: html/parser/html_construction_site.cc:125
    CaseAdjustment {
        lowered: "xchannelselector",
        adjusted: "xChannelSelector",
    },
    // cpp: html/parser/html_construction_site.cc:126
    CaseAdjustment {
        lowered: "ychannelselector",
        adjusted: "yChannelSelector",
    },
    // cpp: html/parser/html_construction_site.cc:127
    CaseAdjustment {
        lowered: "zoomandpan",
        adjusted: "zoomAndPan",
    },
];

// cpp: html/parser/html_construction_site.cc:130-135
const kXLinkNamespace: &str = "http://www.w3.org/1999/xlink";
const kXMLNamespace: &str = "http://www.w3.org/XML/1998/namespace";
const kXMLNSNamespace: &str = "http://www.w3.org/2000/xmlns/";

// cpp: html/parser/html_construction_site.cc:137-144
fn AdjustCase(name: String, adjustments: &[CaseAdjustment]) -> String {
    for adjustment in adjustments {
        if name == adjustment.lowered {
            return adjustment.adjusted.to_owned();
        }
    }
    name
}

// cpp: html/parser/html_construction_site.cc:146-150
pub(crate) fn AdjustTagName(name: String, node_namespace: DOMNamespace) -> String {
    if node_namespace == DOMNamespace::kSVG {
        return AdjustCase(name, kSVGTagAdjustments);
    }
    name
}

// cpp: html/parser/html_construction_site.cc:152-185
pub(crate) fn AdjustAttribute(
    mut name: String,
    value: String,
    node_namespace: DOMNamespace,
) -> DOMAttribute {
    if node_namespace == DOMNamespace::kSVG {
        name = AdjustCase(name, kSVGAttributeAdjustments);
    } else if node_namespace == DOMNamespace::kMathML && name == "definitionurl" {
        name = "definitionURL".to_owned();
    }

    let mut attribute = DOMAttribute {
        local_name: name,
        value,
        ..DOMAttribute::default()
    };
    let qualified_name = attribute.local_name.clone();
    if let Some(local_name) = qualified_name.strip_prefix("xlink:") {
        const kXLinkAttributes: [&str; 7] = [
            "actuate", "arcrole", "href", "role", "show", "title", "type",
        ];
        if kXLinkAttributes.contains(&local_name) {
            attribute.prefix = "xlink".to_owned();
            attribute.local_name = local_name.to_owned();
            attribute.namespace_uri = kXLinkNamespace.to_owned();
        }
    } else if qualified_name == "xml:lang" || qualified_name == "xml:space" {
        attribute.prefix = "xml".to_owned();
        attribute.local_name = qualified_name[4..].to_owned();
        attribute.namespace_uri = kXMLNamespace.to_owned();
    } else if qualified_name == "xmlns" {
        attribute.namespace_uri = kXMLNSNamespace.to_owned();
    } else if qualified_name == "xmlns:xlink" {
        attribute.prefix = "xmlns".to_owned();
        attribute.local_name = "xlink".to_owned();
        attribute.namespace_uri = kXMLNSNamespace.to_owned();
    }
    attribute
}

// cpp: html/parser/html_construction_site.cc:187-203
fn AttributesFromToken(token: &HTMLToken, node_namespace: DOMNamespace) -> Vec<DOMAttribute> {
    let mut output = Vec::with_capacity(token.Attributes().len());
    for token_attribute in token.Attributes() {
        let attribute = AdjustAttribute(
            token_attribute.GetName().Utf8(),
            token_attribute.Value().Utf8(),
            node_namespace,
        );
        let duplicate = output.iter().find(|existing: &&DOMAttribute| {
            existing.local_name == attribute.local_name
                && existing.namespace_uri == attribute.namespace_uri
        });
        if duplicate.is_none() {
            output.push(attribute);
        }
    }
    output
}

// cpp: html/parser/html_construction_site.cc:205-211
fn LowerASCII(value: &str) -> String {
    value.to_ascii_lowercase()
}

// cpp: html/parser/html_construction_site.h:53-56
#[derive(Clone, Copy)]
struct FosterLocation {
    parent: usize,
    before: Option<usize>,
}

// cpp: html/parser/html_construction_site.h:17-62
// Stable DOM arena indices replace source DOMNode pointers. The exclusive
// document borrow also prevents the callback from outliving its DOM owner.
pub struct HTMLConstructionSite<'a> {
    document_: &'a mut PersistentDocument,
    host_: Option<&'a mut dyn HTMLParserHost>,
    open_elements_: Vec<usize>,
}

impl<'a> HTMLConstructionSite<'a> {
    pub(crate) fn IntoOpenElements(self) -> Vec<usize> {
        self.open_elements_
    }
    pub(crate) fn Resume(
        document: &'a mut PersistentDocument,
        host: &'a mut dyn HTMLParserHost,
        open_elements: Vec<usize>,
    ) -> Self {
        Self {
            document_: document,
            host_: Some(host),
            open_elements_: open_elements,
        }
    }

    // cpp: html/parser/html_construction_site.h:21
    // cpp: html/parser/html_construction_site.cc:215-217
    pub fn new(
        document: &'a mut PersistentDocument,
        host: Option<&'a mut dyn HTMLParserHost>,
    ) -> Self {
        Self {
            document_: document,
            host_: host,
            open_elements_: Vec::new(),
        }
    }

    // cpp: html/parser/html_construction_site.h:23
    pub fn OwnerDocument(&self) -> &PersistentDocument {
        &*self.document_
    }

    // Rust tree-builder adapter for the C++ builder's second Document&.
    // The site already holds the exclusive borrow, so all writes use it.
    pub fn OwnerDocumentMut(&mut self) -> &mut PersistentDocument {
        self.document_
    }

    // cpp: html/parser/html_construction_site.h:24
    // cpp: html/parser/html_construction_site.cc:219-226
    pub fn CurrentNode(&self) -> usize {
        let Some(&current) = self.open_elements_.last() else {
            return self.document_.Root();
        };
        self.document_.TemplateContents(current).unwrap_or(current)
    }

    // cpp: html/parser/html_construction_site.h:25
    // cpp: html/parser/html_construction_site.cc:228-230
    pub fn CurrentElement(&self) -> Option<usize> {
        self.open_elements_.last().copied()
    }

    // cpp: html/parser/html_construction_site.h:26
    pub fn OpenElements(&self) -> &[usize] {
        &self.open_elements_
    }

    fn Notify(&mut self, element: usize, phase: ParserElementPhase) {
        if let Some(host) = self.host_.as_deref_mut() {
            host.HandleParserElementInDocument(self.document_, element, phase);
        }
    }

    // cpp: html/parser/html_construction_site.h:28-29
    // cpp: html/parser/html_construction_site.cc:232-239
    pub fn InsertElementFromToken(
        &mut self,
        token: &HTMLToken,
        node_namespace: DOMNamespace,
    ) -> usize {
        if token.GetType() != TokenType::kStartTag {
            invalid_argument("element insertion requires a start tag");
        }
        self.InsertElement(
            AdjustTagName(token.GetName().AsString().Utf8(), node_namespace),
            AttributesFromToken(token, node_namespace),
            node_namespace,
        )
    }

    // cpp: html/parser/html_construction_site.h:30-32
    // cpp: html/parser/html_construction_site.cc:241-256
    pub fn InsertElement(
        &mut self,
        name: String,
        attributes: Vec<DOMAttribute>,
        node_namespace: DOMNamespace,
    ) -> usize {
        let element = self
            .document_
            .CreateElement(node_namespace, name, attributes);
        let parent = self.CurrentNode();
        self.document_.AppendChild(parent, element);
        if self.document_.Node(element).IsHTMLElement("template") {
            self.document_.EnsureTemplateContents(element);
        }
        self.open_elements_.push(element);
        self.Notify(element, ParserElementPhase::kInserted);
        element
    }

    // cpp: html/parser/html_construction_site.h:33
    // cpp: html/parser/html_construction_site.cc:258-262
    pub fn InsertComment(&mut self, data: String) -> usize {
        let comment = self.document_.CreateComment(data);
        let parent = self.CurrentNode();
        self.document_.AppendChild(parent, comment);
        comment
    }

    // cpp: html/parser/html_construction_site.h:34
    // cpp: html/parser/html_construction_site.cc:264-266
    pub fn InsertText(&mut self, data: &str) -> usize {
        let parent = self.CurrentNode();
        self.document_.AppendText(parent, data)
    }

    // cpp: html/parser/html_construction_site.h:57
    // cpp: html/parser/html_construction_site.cc:268-294
    fn FindFosterLocation(&self) -> FosterLocation {
        let mut last_template = None;
        let mut last_table = None;
        for (index, &element) in self.open_elements_.iter().enumerate() {
            if self.document_.Node(element).IsHTMLElement("template") {
                last_template = Some((element, index));
            }
            if self.document_.Node(element).IsHTMLElement("table") {
                last_table = Some((element, index));
            }
        }
        if let Some((template, template_index)) = last_template {
            if last_table.is_none_or(|(_, table_index)| template_index > table_index) {
                return FosterLocation {
                    parent: self
                        .document_
                        .TemplateContents(template)
                        .expect("HTML template contents must exist"),
                    before: None,
                };
            }
        }
        let Some((table, table_index)) = last_table else {
            return FosterLocation {
                parent: self
                    .open_elements_
                    .first()
                    .copied()
                    .unwrap_or_else(|| self.document_.Root()),
                before: None,
            };
        };
        if let Some(parent) = self.document_.Node(table).Parent() {
            return FosterLocation {
                parent,
                before: Some(table),
            };
        }
        if table_index > 0 {
            return FosterLocation {
                parent: self.open_elements_[table_index - 1],
                before: None,
            };
        }
        FosterLocation {
            parent: self.open_elements_[0],
            before: None,
        }
    }

    // cpp: html/parser/html_construction_site.h:35
    // cpp: html/parser/html_construction_site.cc:296-303
    pub fn InsertAtFosterParent(&mut self, child: usize) -> usize {
        let location = self.FindFosterLocation();
        if let Some(before) = location.before {
            self.document_.InsertBefore(location.parent, child, before);
        } else {
            self.document_.AppendChild(location.parent, child);
        }
        child
    }

    // cpp: html/parser/html_construction_site.h:36
    // cpp: html/parser/html_construction_site.cc:305-310
    pub fn InsertTextAtFosterParent(&mut self, data: &str) -> usize {
        let location = self.FindFosterLocation();
        if let Some(before) = location.before {
            return self
                .document_
                .InsertTextBefore(location.parent, before, data);
        }
        self.document_.AppendText(location.parent, data)
    }

    // cpp: html/parser/html_construction_site.h:38
    // cpp: html/parser/html_construction_site.cc:312-321
    pub fn Pop(&mut self) {
        let Some(&node) = self.open_elements_.last() else {
            logic_error("cannot pop an empty open-element stack");
        };
        self.Notify(node, ParserElementPhase::kChildrenFinished);
        self.open_elements_.pop();
    }

    // cpp: html/parser/html_construction_site.h:39
    // cpp: html/parser/html_construction_site.cc:323-327
    pub fn PushAlreadyParsed(&mut self, node: usize) {
        if self.document_.Node(node).Type() != DOMNodeType::kElement {
            invalid_argument("open-element stack requires an element");
        }
        self.open_elements_.push(node);
    }

    // cpp: html/parser/html_construction_site.h:40
    // cpp: html/parser/html_construction_site.cc:329-345
    pub fn PopUntil(&mut self, html_name: &str) {
        while let Some(&node) = self.open_elements_.last() {
            self.Notify(node, ParserElementPhase::kChildrenFinished);
            self.open_elements_.pop();
            let element = self.document_.Node(node);
            if element.Type() != DOMNodeType::kElement {
                continue;
            }
            if element.Namespace() == DOMNamespace::kHTML {
                if element.Name() == html_name {
                    return;
                }
            } else if LowerASCII(element.Name()) == html_name {
                return;
            }
        }
    }

    // cpp: html/parser/html_construction_site.h:41
    // cpp: html/parser/html_construction_site.cc:347-351
    pub fn RemoveFromOpenElements(&mut self, node: usize) {
        if let Some(index) = self
            .open_elements_
            .iter()
            .position(|&element| element == node)
        {
            self.open_elements_.remove(index);
        }
    }

    // cpp: html/parser/html_construction_site.h:42
    // cpp: html/parser/html_construction_site.cc:353-359
    pub fn ReplaceInOpenElements(&mut self, old_node: usize, new_node: usize) {
        let Some(index) = self
            .open_elements_
            .iter()
            .position(|&element| element == old_node)
        else {
            logic_error("open element replacement target is absent");
        };
        self.open_elements_[index] = new_node;
    }

    // cpp: html/parser/html_construction_site.h:43
    // cpp: html/parser/html_construction_site.cc:361-367
    pub fn InsertInOpenElementsAfter(&mut self, position: usize, node: usize) {
        let Some(index) = self
            .open_elements_
            .iter()
            .position(|&element| element == position)
        else {
            logic_error("open element insertion target is absent");
        };
        self.open_elements_.insert(index + 1, node);
    }

    // cpp: html/parser/html_construction_site.h:44
    // cpp: html/parser/html_construction_site.cc:369-374
    pub fn OpenElementBefore(&self, node: usize) -> Option<usize> {
        let index = self
            .open_elements_
            .iter()
            .position(|&element| element == node)?;
        index
            .checked_sub(1)
            .map(|before| self.open_elements_[before])
    }

    // cpp: html/parser/html_construction_site.h:45
    // cpp: html/parser/html_construction_site.cc:376-380
    pub fn HasInOpenElements(&self, html_name: &str) -> bool {
        self.open_elements_
            .iter()
            .any(|&node| self.document_.Node(node).IsHTMLElement(html_name))
    }

    // cpp: html/parser/html_construction_site.h:46
    // cpp: html/parser/html_construction_site.cc:382-384
    pub fn HasNodeInOpenElements(&self, node: usize) -> bool {
        self.open_elements_.contains(&node)
    }

    // cpp: html/parser/html_construction_site.h:48
    // cpp: html/parser/html_construction_site.cc:386-388
    pub fn Reparent(&mut self, new_parent: usize, child: usize) {
        self.document_.AppendChild(new_parent, child);
    }

    // cpp: html/parser/html_construction_site.h:49
    // cpp: html/parser/html_construction_site.cc:390-393
    pub fn InsertAlreadyParsedChild(&mut self, new_parent: usize, child: usize) {
        self.document_.AppendChild(new_parent, child);
    }

    // cpp: html/parser/html_construction_site.h:50
    // cpp: html/parser/html_construction_site.cc:395-398
    pub fn TakeAllChildren(&mut self, source: usize, destination: usize) {
        self.document_.TakeAllChildren(source, destination);
    }
}
