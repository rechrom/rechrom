#![allow(non_snake_case)]
use crate::style_resolver::{
    cascade::{CascadedDeclaration, ResolveDeclarationVariables},
    css_wide::{CSSWideKeywordSource, CSSWideSource},
    CustomProperties,
};
use layoutng_assembly::internal::layout_input::ComputedStyle;

// Blink: css/resolver/style_builder_converter.cc:ConvertZoom and
// css/resolver/style_resolver_state.cc:StyleResolverState::SetZoom.
fn ParseZoom(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value == "normal" {
        return Some(1.0);
    }
    let (number, divisor) = value
        .strip_suffix('%')
        .map_or((value.as_str(), 1.0), |v| (v, 100.0));
    let number = crate::style_resolver::number::Number(number)? / divisor;
    if !number.is_finite() || number < 0.0 || number > f32::MAX as f64 {
        return None;
    }
    Some(if number == 0.0 { 1.0 } else { number as f32 })
}
pub(crate) fn ApplyZoom(style: &mut ComputedStyle, value: &str) {
    if let Some(zoom) = ParseZoom(value) {
        let e = style.extended.get_or_insert_with(Default::default);
        e.effective_zoom = (e.effective_zoom / e.zoom * zoom).clamp(1e-6, 1e6);
        e.zoom = zoom;
    }
}
pub(crate) fn ResolveZoom(
    cascade: &[CascadedDeclaration],
    custom: &CustomProperties,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
) -> f32 {
    let mut zoom = 1.0;
    for item in cascade {
        if !matches!(item.declaration.property.as_str(), "zoom" | "all") {
            continue;
        }
        let declaration =
            ResolveDeclarationVariables(&item.declaration, custom).unwrap_or_else(|| {
                cssom::CSSDeclaration {
                    property: item.declaration.property.clone(),
                    value: "unset".into(),
                    ..Default::default()
                }
            });
        // Zoom is non-inherited. CSS-wide unset therefore uses the initial own zoom.
        let value = declaration.value.trim().to_ascii_lowercase();
        if value == "unset" {
            zoom = 1.0;
        } else if let Some(source) =
            CSSWideKeywordSource(&value, initial, inherited, unset, reverted)
        {
            zoom = source.style.extended.as_ref().map_or(1.0, |e| e.zoom);
        } else if declaration.property == "zoom" {
            if let Some(parsed) = ParseZoom(&value) {
                zoom = parsed;
            }
        }
    }
    zoom
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_normal_percent_and_invalid_values_follow_blink() {
        for (value, expected) in [
            ("0", Some(1.0)),
            ("0%", Some(1.0)),
            ("normal", Some(1.0)),
            ("80%", Some(0.8)),
            ("2", Some(2.0)),
            ("-1", None),
            ("1px", None),
            ("NaN", None),
        ] {
            assert_eq!(ParseZoom(value), expected, "{value}");
        }
    }
}
