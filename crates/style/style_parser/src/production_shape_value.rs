//! Source css_shape_value.h:28-203 and css_shape_value.cc:20-168.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValueRandom, CSSValueSubclass},
    css_value_keywords::CSSValueID,
    production_css_value::Value,
};
use foundation::{svg_path_byte_stream::SvgPathCommand, String, WindRule};
use std::rc::Rc;
#[derive(PartialEq)]
pub enum CSSShapeEndPoint {
    Pair([Rc<Value>; 2]),
    Axis(Rc<Value>),
}
impl CSSShapeEndPoint {
    fn CSSText(&self) -> std::string::String {
        match self {
            Self::Pair(p) => format!("{} {}", p[0].CssText().Utf8(), p[1].CssText().Utf8()),
            Self::Axis(p) => p.CssText().Utf8(),
        }
    }
    fn HasRandomFunctions(&self) -> bool {
        match self {
            Self::Pair(p) => p.iter().any(|p| p.HasRandomFunctions()),
            Self::Axis(p) => p.HasRandomFunctions(),
        }
    }
}
#[derive(PartialEq)]
pub struct CSSShapeControlPoint {
    pub origin: CSSValueID,
    pub point: [Rc<Value>; 2],
}
#[derive(PartialEq)]
pub struct CSSShapeArc {
    pub angle: Rc<Value>,
    pub radius: [Rc<Value>; 2],
    pub direction_agnostic: bool,
    pub large: bool,
    pub sweep: bool,
}
#[derive(PartialEq)]
pub struct CSSShapeCommand {
    pub command: SvgPathCommand,
    pub end_point: Option<CSSShapeEndPoint>,
    pub controls: Vec<CSSShapeControlPoint>,
    pub arc: Option<CSSShapeArc>,
}
impl CSSShapeCommand {
    pub fn new(command: SvgPathCommand, end_point: Option<CSSShapeEndPoint>) -> Self {
        Self {
            command,
            end_point,
            controls: Vec::new(),
            arc: None,
        }
    }
    fn CSSText(&self) -> std::string::String {
        use SvgPathCommand::*;
        let name = match self.command {
            Close => return "close".into(),
            MoveAbs | MoveRel => "move",
            LineAbs | LineRel => "line",
            HLineAbs | HLineRel => "hline",
            VLineAbs | VLineRel => "vline",
            CubicAbs | CubicRel | QuadAbs | QuadRel => "curve",
            SmoothCubicAbs | SmoothCubicRel | SmoothQuadAbs | SmoothQuadRel => "smooth",
            ArcAbs | ArcRel => "arc",
        };
        let mut text = format!(
            "{name} {} {}",
            if self.command.IsRelative() {
                "by"
            } else {
                "to"
            },
            self.end_point
                .as_ref()
                .expect("shape command end point")
                .CSSText()
        );
        for (i, control) in self.controls.iter().enumerate() {
            text.push_str(if i == 0 { " with " } else { " / " });
            text.push_str(&format!(
                "{} {}",
                control.point[0].CssText().Utf8(),
                control.point[1].CssText().Utf8()
            ));
            let default = if self.command.IsRelative() {
                CSSValueID::kStart
            } else {
                CSSValueID::kOrigin
            };
            if control.origin != default {
                text.push_str(match control.origin {
                    CSSValueID::kStart => " from start",
                    CSSValueID::kEnd => " from end",
                    CSSValueID::kOrigin => " from origin",
                    _ => unreachable!("shape control origin"),
                });
            }
        }
        if let Some(arc) = &self.arc {
            text.push_str(" of ");
            text.push_str(&arc.radius[0].CssText().Utf8());
            if !arc.direction_agnostic {
                text.push(' ');
                text.push_str(&arc.radius[1].CssText().Utf8());
            }
            if arc.sweep {
                text.push_str(" cw");
            }
            if arc.large {
                text.push_str(" large");
            }
            let zero = matches!(arc.angle.Payload(), crate::css_value::CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() == 0.);
            if !zero {
                text.push_str(" rotate ");
                text.push_str(&arc.angle.CssText().Utf8());
            }
        }
        text
    }
    fn HasRandomFunctions(&self) -> bool {
        self.end_point
            .as_ref()
            .is_some_and(CSSShapeEndPoint::HasRandomFunctions)
            || self
                .controls
                .iter()
                .any(|c| c.point.iter().any(|p| p.HasRandomFunctions()))
            || self.arc.as_ref().is_some_and(|a| {
                a.angle.HasRandomFunctions() || a.radius.iter().any(|r| r.HasRandomFunctions())
            })
    }
}
pub struct CSSShapeValue {
    pub wind_rule: WindRule,
    pub origin: [Rc<Value>; 2],
    pub commands: Vec<CSSShapeCommand>,
}
impl CSSValueSubclass for CSSShapeValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            format!(
                "shape({}from {} {}, {})",
                if self.wind_rule == WindRule::RULE_EVENODD {
                    "evenodd "
                } else {
                    ""
                },
                self.origin[0].CssText().Utf8(),
                self.origin[1].CssText().Utf8(),
                self.commands
                    .iter()
                    .map(CSSShapeCommand::CSSText)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.wind_rule == other.wind_rule
            && self.origin == other.origin
            && self.commands == other.commands
    }
}
impl CSSValueRandom for CSSShapeValue {
    fn HasRandomFunctions(&self) -> bool {
        self.origin.iter().any(|p| p.HasRandomFunctions())
            || self
                .commands
                .iter()
                .any(CSSShapeCommand::HasRandomFunctions)
    }
}
