#![allow(non_snake_case)]

use super::border_radius::{Length, SplitTopLevel};
use super::linear_gradient::ParseLinearGradient;
use super::shadow::SplitWhitespace;
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundBox, BackgroundImageLayer, BackgroundRepeatRule, BackgroundSizeMode, PaintBlendMode,
    PaintStyleData,
};
use std::sync::Arc;

// cpp: style_resolver/style_resolver.cc:3008-3023
fn ParseImageURL(input: &str) -> Option<String> {
    let mut value = input.trim();
    if !value
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("url("))
        || !value.ends_with(')')
    {
        return None;
    }
    value = value[4..value.len() - 1].trim();
    if value.len() >= 2
        && ((value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"')))
    {
        value = &value[1..value.len() - 1];
    }
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| matches!(byte, b'\r' | b'\n' | 0x0c))
    {
        return None;
    }
    Some(value.to_owned())
}

// cpp: style_resolver/style_resolver.cc:3025-3052
fn ParseBackgroundImages(input: &str, font_size: f64) -> Option<Vec<BackgroundImageLayer>> {
    if input.trim().eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let mut layers = Vec::new();
    for image in SplitTopLevel(input, b',') {
        if let Some(shader) = ParseLinearGradient(&image, font_size) {
            layers.push(BackgroundImageLayer {
                shader: Some(Arc::new(shader)),
                ..BackgroundImageLayer::default()
            });
        } else {
            let source_url = ParseImageURL(&image)?;
            layers.push(BackgroundImageLayer {
                source_url,
                ..BackgroundImageLayer::default()
            });
        }
    }
    Some(layers)
}

// cpp: style_resolver/style_resolver.cc:6064-6076
pub(crate) fn ApplyMaskImages(style: &mut ComputedStyle, value: &str) {
    let font_size = style
        .extended
        .as_ref()
        .map_or(16.0, |extra| extra.font_size);
    if let Some(images) = ParseBackgroundImages(value, font_size) {
        style.paint.mask_images = images
            .into_iter()
            .map(|mut image| {
                image.origin = BackgroundBox::kBorderBox;
                image.clip = BackgroundBox::kBorderBox;
                layoutng_assembly::internal::paint_input::PaintMaskLayer {
                    image,
                    ..Default::default()
                }
            })
            .collect();
    }
}

// cpp: style_resolver/style_resolver.cc:3140-3180
fn ParseBackgroundRepeatRule(input: &str) -> Option<BackgroundRepeatRule> {
    match input.trim().to_ascii_lowercase().as_str() {
        "repeat" => Some(BackgroundRepeatRule::kRepeat),
        "no-repeat" => Some(BackgroundRepeatRule::kNoRepeat),
        "round" => Some(BackgroundRepeatRule::kRound),
        "space" => Some(BackgroundRepeatRule::kSpace),
        _ => None,
    }
}

fn ParseBackgroundRepeats(
    input: &str,
) -> Option<Vec<(BackgroundRepeatRule, BackgroundRepeatRule)>> {
    let mut values = Vec::new();
    for layer in SplitTopLevel(input, b',') {
        let parts: Vec<String> = layer
            .split_ascii_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        let axes = match parts.as_slice() {
            [one] if one == "repeat-x" => (
                BackgroundRepeatRule::kRepeat,
                BackgroundRepeatRule::kNoRepeat,
            ),
            [one] if one == "repeat-y" => (
                BackgroundRepeatRule::kNoRepeat,
                BackgroundRepeatRule::kRepeat,
            ),
            [one] => {
                let rule = ParseBackgroundRepeatRule(one)?;
                (rule, rule)
            }
            [x, y] => (ParseBackgroundRepeatRule(x)?, ParseBackgroundRepeatRule(y)?),
            _ => return None,
        };
        values.push(axes);
    }
    (!values.is_empty()).then_some(values)
}

// cpp: style_resolver/style_resolver.cc:3056-3062
// cpp: style_resolver/style_resolver.cc:3173-3216
#[derive(Clone, Default)]
pub(crate) struct BackgroundSizeValue {
    mode: Option<BackgroundSizeMode>,
    width: Option<f64>,
    height: Option<f64>,
    width_percentage: Option<f64>,
    height_percentage: Option<f64>,
}

fn ParseBackgroundSizeComponent(input: &str, font_size: f64) -> Option<(Option<f64>, Option<f64>)> {
    if input == "auto" {
        return Some((None, None));
    }
    if let Some(percent) = input
        .strip_suffix('%')
        .and_then(|value| value.parse::<f64>().ok())
    {
        return (percent >= 0.0).then_some((None, Some(percent / 100.0)));
    }
    Length(input, font_size)
        .filter(|value| *value >= 0.0)
        .map(|value| (Some(value), None))
}

fn ParseBackgroundSizes(input: &str, font_size: f64) -> Option<Vec<BackgroundSizeValue>> {
    let mut values = Vec::new();
    for layer in SplitTopLevel(input, b',') {
        let parts: Vec<String> = layer
            .split_ascii_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        if parts.is_empty() || parts.len() > 2 {
            return None;
        }
        let mut size = BackgroundSizeValue::default();
        if parts.len() == 1 && (parts[0] == "contain" || parts[0] == "cover") {
            size.mode = Some(if parts[0] == "contain" {
                BackgroundSizeMode::kContain
            } else {
                BackgroundSizeMode::kCover
            });
        } else {
            size.mode = Some(BackgroundSizeMode::kExplicit);
            (size.width, size.width_percentage) =
                ParseBackgroundSizeComponent(&parts[0], font_size)?;
            if parts.len() == 2 {
                (size.height, size.height_percentage) =
                    ParseBackgroundSizeComponent(&parts[1], font_size)?;
            }
        }
        values.push(size);
    }
    (!values.is_empty()).then_some(values)
}

// cpp: style_resolver/style_resolver.cc:3366-3384
fn ParseBackgroundBoxes(input: &str) -> Option<Vec<BackgroundBox>> {
    let mut boxes = Vec::new();
    for layer in SplitTopLevel(input, b',') {
        boxes.push(match layer.trim().to_ascii_lowercase().as_str() {
            "border-box" => BackgroundBox::kBorderBox,
            "padding-box" => BackgroundBox::kPaddingBox,
            "content-box" => BackgroundBox::kContentBox,
            _ => return None,
        });
    }
    (!boxes.is_empty()).then_some(boxes)
}

// cpp: style_resolver/style_resolver.cc:3218-3224
#[derive(Clone, Copy, PartialEq, Eq)]
enum PositionAxis {
    Horizontal,
    Vertical,
    Either,
}

fn BackgroundPositionAxis(value: &str) -> PositionAxis {
    match value {
        "left" | "right" => PositionAxis::Horizontal,
        "top" | "bottom" => PositionAxis::Vertical,
        _ => PositionAxis::Either,
    }
}

// cpp: style_resolver/style_resolver.cc:3226-3252
fn ParseBackgroundPositionComponent(
    input: &str,
    font_size: f64,
    percentage: &mut f64,
    offset: &mut f64,
) -> bool {
    match input {
        "left" | "top" => {
            *percentage = 0.0;
            true
        }
        "center" => {
            *percentage = 0.5;
            true
        }
        "right" | "bottom" => {
            *percentage = 1.0;
            true
        }
        _ => {
            if let Some(parsed) = input
                .strip_suffix('%')
                .and_then(|part| part.parse::<f64>().ok())
            {
                *percentage = parsed / 100.0;
                true
            } else if let Some(parsed) = Length(input, font_size) {
                *offset = parsed;
                true
            } else {
                false
            }
        }
    }
}

// cpp: style_resolver/style_resolver.cc:3254-3351
pub(crate) fn ParseBackgroundPositionLayer(
    input: &str,
    font_size: f64,
) -> Option<(Offset, Offset)> {
    let parts: Vec<String> = input
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let mut percentage = Offset::default();
    let mut offset = Offset::default();
    let mut has_x = false;
    let mut has_y = false;
    if parts.len() >= 3 {
        let mentions_horizontal_edge = parts
            .iter()
            .any(|part| BackgroundPositionAxis(part) == PositionAxis::Horizontal);
        let mentions_vertical_edge = parts
            .iter()
            .any(|part| BackgroundPositionAxis(part) == PositionAxis::Vertical);
        let centers = parts.iter().filter(|part| *part == "center").count();
        if centers > 1 || (centers == 1 && mentions_horizontal_edge == mentions_vertical_edge) {
            return None;
        }
        let mut index = 0;
        while index < parts.len() {
            let axis = BackgroundPositionAxis(&parts[index]);
            if parts[index] == "center" {
                let horizontal = mentions_vertical_edge;
                if (horizontal && has_x) || (!horizontal && has_y) {
                    return None;
                }
                if horizontal {
                    percentage.x = 0.5;
                    has_x = true;
                } else {
                    percentage.y = 0.5;
                    has_y = true;
                }
                index += 1;
                continue;
            }
            if axis == PositionAxis::Either {
                return None;
            }
            let horizontal = axis == PositionAxis::Horizontal;
            if (horizontal && has_x) || (!horizontal && has_y) {
                return None;
            }
            let mut percent = if matches!(parts[index].as_str(), "right" | "bottom") {
                1.0
            } else {
                0.0
            };
            let mut length = 0.0;
            let from_end = percent == 1.0;
            index += 1;
            if index < parts.len()
                && BackgroundPositionAxis(&parts[index]) == PositionAxis::Either
                && parts[index] != "center"
            {
                let mut parsed_percent = 0.0;
                let mut parsed_length = 0.0;
                if !ParseBackgroundPositionComponent(
                    &parts[index],
                    font_size,
                    &mut parsed_percent,
                    &mut parsed_length,
                ) {
                    return None;
                }
                if parts[index].ends_with('%') {
                    percent = if from_end {
                        1.0 - parsed_percent
                    } else {
                        parsed_percent
                    };
                } else {
                    length = if from_end {
                        -parsed_length
                    } else {
                        parsed_length
                    };
                }
                index += 1;
            }
            if horizontal {
                percentage.x = percent;
                offset.x = length;
                has_x = true;
            } else {
                percentage.y = percent;
                offset.y = length;
                has_y = true;
            }
        }
        return (has_x && has_y).then_some((percentage, offset));
    }
    for part in &parts {
        let axis = BackgroundPositionAxis(part);
        let horizontal = if axis == PositionAxis::Either {
            !has_x
        } else {
            axis == PositionAxis::Horizontal
        };
        if (horizontal && has_x) || (!horizontal && has_y) {
            return None;
        }
        let (percent, length) = if horizontal {
            (&mut percentage.x, &mut offset.x)
        } else {
            (&mut percentage.y, &mut offset.y)
        };
        if !ParseBackgroundPositionComponent(part, font_size, percent, length) {
            return None;
        }
        if horizontal {
            has_x = true;
        } else {
            has_y = true;
        }
    }
    if !has_x {
        percentage.x = 0.5;
    }
    if !has_y {
        percentage.y = 0.5;
    }
    Some((percentage, offset))
}

// cpp: style_resolver/style_resolver.cc:3353-3365
fn ParseBackgroundPositions(input: &str, font_size: f64) -> Option<Vec<(Offset, Offset)>> {
    let mut values = Vec::new();
    for layer in SplitTopLevel(input, b',') {
        values.push(ParseBackgroundPositionLayer(&layer, font_size)?);
    }
    (!values.is_empty()).then_some(values)
}

// Declaration layer values remain separate until export, as in the source.
// cpp: style_resolver/style_resolver.cc:3070-3136
pub struct BackgroundCascadeState {
    pub(crate) images: Vec<BackgroundImageLayer>,
    pub(crate) repeats: Vec<(BackgroundRepeatRule, BackgroundRepeatRule)>,
    pub(crate) positions: Vec<(Offset, Offset)>,
    pub(crate) sizes: Vec<BackgroundSizeValue>,
    pub(crate) origins: Vec<BackgroundBox>,
    pub(crate) clips: Vec<BackgroundBox>,
    pub(crate) blend_modes: Vec<PaintBlendMode>,
}

// cpp: style_resolver/style_resolver.cc:3400-3555
fn ParseBackgroundShorthand(
    input: &str,
    font_size: f64,
) -> Option<(BackgroundCascadeState, Color)> {
    let layers = SplitTopLevel(input, b',');
    if layers.is_empty() {
        return None;
    }
    let mut result = BackgroundCascadeState {
        images: Vec::new(),
        repeats: Vec::new(),
        positions: Vec::new(),
        sizes: Vec::new(),
        origins: Vec::new(),
        clips: Vec::new(),
        blend_modes: vec![PaintBlendMode::kNormal],
    };
    let mut color = Color::default();
    let mut has_color = false;
    for (layer_index, layer) in layers.iter().enumerate() {
        let slash_parts = SplitTopLevel(layer, b'/');
        if slash_parts.is_empty() || slash_parts.len() > 2 {
            return None;
        }
        let mut image = None;
        let mut explicit_none = false;
        let mut position_parts = Vec::new();
        let mut size_parts = Vec::new();
        let mut repeat_parts = Vec::new();
        let mut boxes = Vec::new();
        let mut parse_tokens = |text: &str, after_slash: bool| -> Option<()> {
            for token in SplitWhitespace(text) {
                let lower = token.to_ascii_lowercase();
                if lower == "none" {
                    if after_slash || image.is_some() || explicit_none {
                        return None;
                    }
                    explicit_none = true;
                    continue;
                }
                if let Some(mut parsed) = ParseBackgroundImages(&token, font_size) {
                    if after_slash || image.is_some() || explicit_none || parsed.len() != 1 {
                        return None;
                    }
                    image = parsed.pop();
                    continue;
                }
                if let Some(background_box) = match lower.as_str() {
                    "border-box" => Some(BackgroundBox::kBorderBox),
                    "padding-box" => Some(BackgroundBox::kPaddingBox),
                    "content-box" => Some(BackgroundBox::kContentBox),
                    _ => None,
                } {
                    if boxes.len() == 2 {
                        return None;
                    }
                    boxes.push(background_box);
                    continue;
                }
                if matches!(lower.as_str(), "repeat-x" | "repeat-y")
                    || ParseBackgroundRepeatRule(&lower).is_some()
                {
                    if repeat_parts.len() == 2 {
                        return None;
                    }
                    repeat_parts.push(lower);
                    continue;
                }
                if lower == "scroll" {
                    continue;
                }
                if matches!(lower.as_str(), "fixed" | "local") {
                    return None;
                }
                if let Some(parsed_color) =
                    layoutng_assembly::css_color_parser::ParseCSSColor(&token)
                {
                    if layer_index + 1 != layers.len() || has_color {
                        return None;
                    }
                    color = parsed_color;
                    has_color = true;
                    continue;
                }
                (if after_slash {
                    &mut size_parts
                } else {
                    &mut position_parts
                })
                .push(token);
            }
            Some(())
        };
        parse_tokens(&slash_parts[0], false)?;
        if slash_parts.len() == 2 {
            parse_tokens(&slash_parts[1], true)?;
            if size_parts.is_empty() {
                return None;
            }
        }
        let repeat = if repeat_parts.is_empty() {
            (BackgroundRepeatRule::kRepeat, BackgroundRepeatRule::kRepeat)
        } else {
            ParseBackgroundRepeats(&repeat_parts.join(" "))?
                .into_iter()
                .next()?
        };
        let size = if size_parts.is_empty() {
            BackgroundSizeValue::default()
        } else {
            ParseBackgroundSizes(&size_parts.join(" "), font_size)?
                .into_iter()
                .next()?
        };
        let position = if position_parts.is_empty() {
            (Offset::default(), Offset::default())
        } else {
            ParseBackgroundPositionLayer(&position_parts.join(" "), font_size)?
        };
        let origin = boxes.first().copied().unwrap_or(BackgroundBox::kPaddingBox);
        let clip = boxes.last().copied().unwrap_or(BackgroundBox::kBorderBox);
        if explicit_none && layers.len() != 1 {
            return None;
        }
        if let Some(image) = image {
            result.images.push(image);
            result.repeats.push(repeat);
            result.sizes.push(size);
            result.positions.push(position);
            result.origins.push(origin);
            result.clips.push(clip);
        } else {
            if layers.len() != 1 {
                return None;
            }
            result.origins.push(origin);
            result.clips.push(clip);
        }
    }
    if result.repeats.is_empty() {
        result
            .repeats
            .push((BackgroundRepeatRule::kRepeat, BackgroundRepeatRule::kRepeat));
        result.sizes.push(BackgroundSizeValue::default());
        result
            .positions
            .push((Offset::default(), Offset::default()));
    }
    Some((result, color))
}

impl Default for BackgroundCascadeState {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            repeats: vec![(BackgroundRepeatRule::kRepeat, BackgroundRepeatRule::kRepeat)],
            positions: vec![(Offset::default(), Offset::default())],
            sizes: vec![BackgroundSizeValue::default()],
            origins: vec![BackgroundBox::kPaddingBox],
            clips: vec![BackgroundBox::kBorderBox],
            blend_modes: vec![PaintBlendMode::kNormal],
        }
    }
}

impl BackgroundCascadeState {
    // cpp: style_resolver/style_resolver.cc:3073-3081
    pub(crate) fn ResetLayers(&mut self) {
        *self = Self::default();
    }
    // cpp: style_resolver/style_resolver.cc:3083-3108
    pub(crate) fn Import(&mut self, paint: &PaintStyleData) {
        self.images = paint.background_images.clone();
        self.repeats.clear();
        self.sizes.clear();
        self.positions.clear();
        self.origins.clear();
        self.clips.clear();
        self.blend_modes.clear();
        for layer in &paint.background_images {
            self.repeats.push((
                layer.repeat_rule_x.unwrap_or(BackgroundRepeatRule::kRepeat),
                layer.repeat_rule_y.unwrap_or(BackgroundRepeatRule::kRepeat),
            ));
            self.sizes.push(BackgroundSizeValue {
                mode: Some(layer.size_mode),
                width: layer.width,
                height: layer.height,
                width_percentage: layer.width_percentage,
                height_percentage: layer.height_percentage,
            });
            self.positions.push((layer.position, layer.position_offset));
            self.origins.push(layer.origin);
            self.clips.push(layer.clip);
            self.blend_modes.push(layer.blend_mode);
        }
        if self.repeats.is_empty() {
            self.repeats
                .push((BackgroundRepeatRule::kRepeat, BackgroundRepeatRule::kRepeat));
        }
        if self.sizes.is_empty() {
            self.sizes.push(BackgroundSizeValue::default());
        }
        if self.positions.is_empty() {
            self.positions.push((Offset::default(), Offset::default()));
        }
        if self.origins.is_empty() {
            self.origins.push(BackgroundBox::kPaddingBox);
        }
        if self.clips.is_empty() {
            self.clips.push(paint.background_clip);
        }
        if self.blend_modes.is_empty() {
            self.blend_modes.push(PaintBlendMode::kNormal);
        }
    }
    pub fn Apply(&mut self, style: &mut ComputedStyle, property: &str, value: &str) -> bool {
        let font_size = style
            .extended
            .as_ref()
            .map_or(16.0, |extended| extended.font_size);
        match property {
            "background-position" => {
                if let Some(positions) = ParseBackgroundPositions(value, font_size) {
                    self.positions = positions;
                }
            }
            "background-repeat" => {
                if let Some(repeats) = ParseBackgroundRepeats(value) {
                    self.repeats = repeats;
                }
            }
            "background-size" => {
                if let Some(sizes) = ParseBackgroundSizes(value, font_size) {
                    self.sizes = sizes;
                }
            }
            "background-origin" => {
                if let Some(origins) = ParseBackgroundBoxes(value) {
                    self.origins = origins;
                }
            }
            "background-clip" => {
                if let Some(clips) = ParseBackgroundBoxes(value) {
                    self.clips = clips;
                }
            }
            "background-image" => {
                if let Some(images) = ParseBackgroundImages(value, font_size) {
                    self.images = images;
                }
            }
            "background-blend-mode" => {
                if let Some(values) = ParseBackgroundBlendModes(value) {
                    self.blend_modes = values;
                }
            }
            "background-color" => {
                if let Some(color) = layoutng_assembly::css_color_parser::ParseCSSColor(value) {
                    style.paint.background_color = color;
                }
            }
            "background" => {
                if let Some((background, color)) = ParseBackgroundShorthand(value, font_size) {
                    *self = background;
                    style.paint.background_color = color;
                }
            }
            _ => return false,
        }
        true
    }

    // cpp: style_resolver/style_resolver.cc:3110-3136
    pub fn Export(self, style: &mut ComputedStyle) {
        let last = self.images.len().saturating_sub(1);
        style.paint.background_clip = self.clips[last % self.clips.len()];
        style.paint.background_images = self.images;
        for (index, layer) in style.paint.background_images.iter_mut().enumerate() {
            let repeat = self.repeats[index % self.repeats.len()];
            let position = self.positions[index % self.positions.len()];
            let size = &self.sizes[index % self.sizes.len()];
            layer.repeat_rule_x = Some(repeat.0);
            layer.repeat_rule_y = Some(repeat.1);
            layer.position = position.0;
            layer.position_offset = position.1;
            layer.size_mode = size.mode.unwrap_or(BackgroundSizeMode::kAuto);
            layer.width = size.width;
            layer.height = size.height;
            layer.width_percentage = size.width_percentage;
            layer.height_percentage = size.height_percentage;
            layer.origin = self.origins[index % self.origins.len()];
            layer.clip = self.clips[index % self.clips.len()];
            layer.blend_mode = self.blend_modes[index % self.blend_modes.len()];
        }
    }
}

// cpp: style_resolver/style_resolver.cc:2535-2554
pub(crate) fn ParseBlendMode(value: &str) -> Option<PaintBlendMode> {
    use PaintBlendMode::*;
    Some(match value {
        "normal" => kNormal,
        "multiply" => kMultiply,
        "screen" => kScreen,
        "overlay" => kOverlay,
        "darken" => kDarken,
        "lighten" => kLighten,
        "color-dodge" => kColorDodge,
        "color-burn" => kColorBurn,
        "hard-light" => kHardLight,
        "soft-light" => kSoftLight,
        "difference" => kDifference,
        "exclusion" => kExclusion,
        "hue" => kHue,
        "saturation" => kSaturation,
        "color" => kColor,
        "luminosity" => kLuminosity,
        "plus-lighter" => kPlusLighter,
        _ => return None,
    })
}
// cpp: style_resolver/style_resolver.cc:3386-3396
fn ParseBackgroundBlendModes(input: &str) -> Option<Vec<PaintBlendMode>> {
    let values = SplitTopLevel(input, b',')
        .iter()
        .map(|layer| {
            ParseBlendMode(
                &layer
                    .trim_matches(|c: char| {
                        crate::style_resolver::selector::SourceSpace::source_space(c)
                    })
                    .to_ascii_lowercase(),
            )
        })
        .collect::<Option<Vec<_>>>()?;
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layered_linear_gradient_shorthand_replaces_prior_solid_background() {
        let (state, color) = ParseBackgroundShorthand(
            "linear-gradient(white, white) padding-box, linear-gradient(160deg, #3377fe, #4c6fff, #8370ff, #ba59ff) border-box",
            16.0,
        )
        .unwrap();
        assert_eq!(color.alpha, 0.0);
        assert_eq!(state.images.len(), 2);
        assert_eq!(
            state.clips,
            vec![BackgroundBox::kPaddingBox, BackgroundBox::kBorderBox]
        );
        assert_eq!(state.images[0].shader.as_ref().unwrap().stops.len(), 2);
        let second = state.images[1].shader.as_ref().unwrap();
        assert_eq!(second.linear_angle, Some(160.0));
        assert_eq!(second.stops.len(), 4);
    }
}
