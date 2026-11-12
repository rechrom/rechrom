# Shared scroll snapping

The pinned Chromium cc scroll-snap data, selection strategies and scroll-step
helpers have one owner here. Layout and Compositor keep independent state while
using these source algorithms. Existing PaintModel/Compositor value paths
re-export the same geometry and ElementId types; this crate has no engine
or platform dependencies.
