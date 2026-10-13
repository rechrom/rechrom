# Typed fit-content grid tracks

The production Style parser consumes `fit-content(<length-percentage>)` into
one typed CSSFunctionValue. Its existing nonnegative length/math consumer owns
validation, document-mode unitless quirks, relative units and percentages. The
function accepts exactly one argument and uses the source RestoringBlockGuard
semantics on failure. Grid breadth keywords, flex units, negative literals,
extra arguments and nested sizing functions are not valid arguments.

The production grid converter sends the resolved Length through the existing
GridTrackSizeInput::FitContent constructor. Native ComputedStyle retains
GridTrackSize(kFitContentTrackSizing), with intrinsic auto/auto fields and the
separate fit-content limit. Native LayoutNG owns minimum contributions,
max-content growth, percentage resolution and the fit-content cap. Neither
Page nor Layout parses CSS or reconstructs the track from text.

This completes an existing translated native mechanism. There is no grid
solver, new public interface, resource cache, host service, site branch or
paint adjustment in this change. Mutation and media invalidation continue
through the existing Style/Layout lifecycle and immutable publication.

Chromium revision: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

- css_parsing_utils.cc ConsumeFitContent and ConsumeGridTrackSize
- style_builder_converter.cc ConvertGridTrackSize
- grid_track_size.h intrinsic fields and separate fit-content limit

Regression coverage lives in browser/tests/responsive_grid_tracks.rs and
Style's document_style_engine_test.rs. It checks native data, caps and intrinsic
minimums, percentage/calc/zero arguments, both axes, shorthand, shadow/media
rules, viewport changes, mutation, inheritance, malformed arguments and old
published snapshots.

Repeat/subgrid parsing and other pre-existing unsupported grammar remain their
own mechanisms. A separate known mixed native/compatibility rule-order issue
is not repaired here; supported fit-content rules now stay on the canonical
native path rather than entering that fallback. Named font-feature-values
resolution remains separate and is not compensated by grid geometry.
