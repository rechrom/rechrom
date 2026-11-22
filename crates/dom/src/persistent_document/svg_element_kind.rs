//! SVGElementFactory source svg_tag_names.json5 and SVG*Element inheritance.
//! The discriminator is selected only during canonical DOM allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SVGElementKind {
    A,
    Animate,
    AnimateColor,
    AnimateMotion,
    AnimateTransform,
    Set,
    Circle,
    ClipPath,
    Defs,
    Desc,
    Ellipse,
    FeBlend,
    FeColorMatrix,
    FeComponentTransfer,
    FeComposite,
    FeConvolveMatrix,
    FeDiffuseLighting,
    FeDisplacementMap,
    FeDistantLight,
    FeDropShadow,
    FeFlood,
    FeFuncA,
    FeFuncB,
    FeFuncG,
    FeFuncR,
    FeGaussianBlur,
    FeImage,
    FeMerge,
    FeMergeNode,
    FeMorphology,
    FeOffset,
    FePointLight,
    FeSpecularLighting,
    FeSpotLight,
    FeTile,
    FeTurbulence,
    Filter,
    ForeignObject,
    G,
    Image,
    Line,
    LinearGradient,
    Marker,
    Mask,
    Metadata,
    Mpath,
    Path,
    Pattern,
    Polygon,
    Polyline,
    RadialGradient,
    Rect,
    Script,
    Stop,
    Style,
    Svg,
    Switch,
    Symbol,
    Text,
    TextPath,
    Title,
    Tspan,
    Use,
    View,
    Unknown,
}
impl SVGElementKind {
    pub(super) fn FromFactoryName(name: &str) -> Self {
        match name {
            "a" => Self::A,
            "animate" => Self::Animate,
            "animateColor" => Self::AnimateColor,
            "animateMotion" => Self::AnimateMotion,
            "animateTransform" => Self::AnimateTransform,
            "set" => Self::Set,
            "circle" => Self::Circle,
            "clipPath" => Self::ClipPath,
            "defs" => Self::Defs,
            "desc" => Self::Desc,
            "ellipse" => Self::Ellipse,
            "feBlend" => Self::FeBlend,
            "feColorMatrix" => Self::FeColorMatrix,
            "feComponentTransfer" => Self::FeComponentTransfer,
            "feComposite" => Self::FeComposite,
            "feConvolveMatrix" => Self::FeConvolveMatrix,
            "feDiffuseLighting" => Self::FeDiffuseLighting,
            "feDisplacementMap" => Self::FeDisplacementMap,
            "feDistantLight" => Self::FeDistantLight,
            "feDropShadow" => Self::FeDropShadow,
            "feFlood" => Self::FeFlood,
            "feFuncA" => Self::FeFuncA,
            "feFuncB" => Self::FeFuncB,
            "feFuncG" => Self::FeFuncG,
            "feFuncR" => Self::FeFuncR,
            "feGaussianBlur" => Self::FeGaussianBlur,
            "feImage" => Self::FeImage,
            "feMerge" => Self::FeMerge,
            "feMergeNode" => Self::FeMergeNode,
            "feMorphology" => Self::FeMorphology,
            "feOffset" => Self::FeOffset,
            "fePointLight" => Self::FePointLight,
            "feSpecularLighting" => Self::FeSpecularLighting,
            "feSpotLight" => Self::FeSpotLight,
            "feTile" => Self::FeTile,
            "feTurbulence" => Self::FeTurbulence,
            "filter" => Self::Filter,
            "foreignObject" => Self::ForeignObject,
            "g" => Self::G,
            "image" => Self::Image,
            "line" => Self::Line,
            "linearGradient" => Self::LinearGradient,
            "marker" => Self::Marker,
            "mask" => Self::Mask,
            "metadata" => Self::Metadata,
            "mpath" => Self::Mpath,
            "path" => Self::Path,
            "pattern" => Self::Pattern,
            "polygon" => Self::Polygon,
            "polyline" => Self::Polyline,
            "radialGradient" => Self::RadialGradient,
            "rect" => Self::Rect,
            "script" => Self::Script,
            "stop" => Self::Stop,
            "style" => Self::Style,
            "svg" => Self::Svg,
            "switch" => Self::Switch,
            "symbol" => Self::Symbol,
            "text" => Self::Text,
            "textPath" => Self::TextPath,
            "title" => Self::Title,
            "tspan" => Self::Tspan,
            "use" => Self::Use,
            "view" => Self::View,
            _ => Self::Unknown,
        }
    }
    pub fn IsSVGGraphicsElement(self) -> bool {
        matches!(
            self,
            Self::A
                | Self::Circle
                | Self::Defs
                | Self::Ellipse
                | Self::ForeignObject
                | Self::G
                | Self::Image
                | Self::Line
                | Self::Path
                | Self::Polygon
                | Self::Polyline
                | Self::Rect
                | Self::Svg
                | Self::Switch
                | Self::Symbol
                | Self::Text
                | Self::TextPath
                | Self::Tspan
                | Self::Use
        )
    }
}
impl super::DOMNode {
    pub fn SVGElementKind(&self) -> Option<SVGElementKind> {
        self.svg_element_kind
    }
    pub fn IsSVGGraphicsElement(&self) -> bool {
        self.svg_element_kind
            .is_some_and(SVGElementKind::IsSVGGraphicsElement)
    }
}
