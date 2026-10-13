//! CSSOM View's Window branch uses a Page-supplied viewport snapshot. Bindings
//! own conversion and source scroll semantics; Page owns layout and mutation.
use super::*;
use foundation::gfx;
use layoutng_assembly::internal::layout_input::Offset;

/// Immutable values from the current layout viewport, in LayoutZoomFactor
/// coordinates. This does not expose a LayoutObject, scroll area or Page.
#[derive(Clone, Copy, Debug)]
pub struct ViewportScrollState {
    pub target_node_id: HostObjectId,
    pub offset: Offset,
    pub minimum: Offset,
    pub maximum: Offset,
    pub layout_zoom_factor: f64,
    pub smooth_behavior: bool,
    pub snap: bool,
}

impl DOMJavaScriptBindings {
    pub(super) fn CallViewportScroll(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostResult> {
        if !matches!(member, "viewportScrollOffset" | "viewportScroll") {
            return None;
        }
        if receiver != 0 {
            return Some(Self::type_error("Illegal invocation"));
        }
        let Some(read) = self.services.host.read_viewport_scroll.as_mut() else {
            return Some(if member == "viewportScrollOffset" {
                Self::value(HostValue::Number(0.0))
            } else {
                Self::value(HostValue::String(
                    "Viewport scrolling requires a connected host".into(),
                ))
            });
        };
        // LocalDOMWindow updates style/layout before reading the layout
        // viewport. The embedder returns current DOM-owned offsets, so writes
        // in the same JavaScript turn compose before paint publication.
        let Some(state) = read() else {
            return Some(Self::value(if member == "viewportScrollOffset" {
                HostValue::Number(0.0)
            } else {
                HostValue::String(String::new())
            }));
        };
        let zoom = state.layout_zoom_factor as f32;
        if member == "viewportScrollOffset" {
            let rounded = gfx::ToRoundedVector2d(&gfx::Vector2dF::new(
                state.offset.x as f32,
                state.offset.y as f32,
            ));
            let axis_x = matches!(arguments.first(), Some(HostValue::String(axis)) if axis == "x");
            return Some(Self::value(HostValue::Number(
                (if axis_x {
                    rounded.x() as f32
                } else {
                    rounded.y() as f32
                } / zoom) as f64,
            )));
        }
        let [HostValue::Boolean(relative), x, y, HostValue::String(behavior)] = arguments else {
            return Some(Self::type_error("Invalid viewport scroll request"));
        };
        let unsupported = if state.snap {
            Some("Viewport scroll snapping is not implemented")
        } else if behavior == "smooth" || (behavior == "auto" && state.smooth_behavior) {
            Some("Smooth viewport scrolling is not implemented")
        } else {
            None
        };
        if let Some(message) = unsupported {
            return Some(Self::value(HostValue::String(message.into())));
        }
        // LocalDOMWindow casts the unrestricted double to float before
        // ScrollableArea::NormalizeNonFiniteScroll. Float overflow also maps
        // to zero, not to the edge of the document.
        fn coordinate(value: &HostValue) -> Option<f32> {
            match value {
                HostValue::Number(value) => {
                    Some(if value.is_finite() && value.abs() <= f32::MAX as f64 {
                        *value as f32
                    } else {
                        0.0
                    })
                }
                _ => None,
            }
        }
        // PaintLayerScrollableArea exposes minimum = -ScrollOrigin. Keep
        // source's offset -> position -> offset f32 operations: cancelling the
        // origin algebraically changes rounding near half pixels in RTL.
        let origin = gfx::Vector2dF::new(-state.minimum.x as f32, -state.minimum.y as f32);
        let axis = |value: &HostValue, current: f64, origin: f32| -> f32 {
            if *relative {
                let position = current as f32 + origin;
                let delta = coordinate(value).unwrap_or(0.0) * zoom;
                (position + delta) - origin
            } else {
                coordinate(value).map_or(current as f32, |value| value * zoom)
            }
        };
        let mut offset = gfx::Vector2dF::new(
            axis(x, state.offset.x, origin.x()),
            axis(y, state.offset.y, origin.y()),
        );
        // The absolute Window branch rounds before its origin round trip.
        if !*relative {
            let rounded = gfx::ToRoundedVector2d(&offset);
            offset = gfx::Vector2dF::new(
                (rounded.x() as f32 + origin.x()) - origin.x(),
                (rounded.y() as f32 + origin.y()) - origin.y(),
            );
        }
        let clamped = Offset {
            x: (offset.x() as f64).clamp(state.minimum.x, state.maximum.x),
            y: (offset.y() as f64).clamp(state.minimum.y, state.maximum.y),
        };
        // ScrollableArea::ScrollOffsetChanged rounds storage with Chromium
        // default FractionalScrollOffsets disabled, after clamping.
        let rounded =
            gfx::ToRoundedVector2d(&gfx::Vector2dF::new(clamped.x as f32, clamped.y as f32));
        let offset = Offset {
            x: rounded.x() as f64,
            y: rounded.y() as f64,
        };
        let Some(write) = self.services.host.write_scroll.as_mut() else {
            return Some(Self::value(HostValue::String(
                "Viewport scrolling requires a connected host".into(),
            )));
        };
        // This synchronous host call changes the native offset. Its existing
        // owner handles invalidation, coalesced scroll events and presentation.
        write(state.target_node_id, offset);
        Some(Self::value(HostValue::String(String::new())))
    }
}
