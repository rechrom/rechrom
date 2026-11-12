Chromium revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`
Metric: nonblank non-comment physical source lines

## Full production directory scope

Blink Paint production sources that feed the rendering pipeline; tests, performance tests, and core/paint/timing are reported separately.

| Chromium area | Files | Audited files | Unassigned files | Code | Translated | Unsupported | N/A | Remaining |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `core/paint` | 195 | 195 | 0 | 43077 | 24776 | 13160 | 5141 | 0 |
| `core/paint/compositing` | 2 | 2 | 0 | 471 | 216 | 210 | 45 | 0 |
| `platform/graphics/paint` | 78 | 78 | 0 | 9719 | 7370 | 793 | 1556 | 0 |
| **Total** | **275** | **275** | **0** | **53267** | **32362** | **14163** | **6742** | **0** |

Audited files have a manually verified source-range entry. Unassigned files count entirely as Remaining; a verified package subtotal therefore describes only its listed backbone files, not every Chromium file that may ultimately belong to that package.
`core/paint/timing` is a separate performance-observation subsystem and is not included in the rendering-pipeline total above.
Supporting sources outside the two Paint directories remain in the backbone detail and outside the full-directory total: `third_party/blink/renderer/core/css/basic_shape_functions.cc`, `third_party/blink/renderer/core/css/css_basic_shape_values.cc`, `third_party/blink/renderer/core/css/css_basic_shape_values.h`, `third_party/blink/renderer/core/css/css_gradient_value.cc`, `third_party/blink/renderer/core/css/css_gradient_value.h`, `third_party/blink/renderer/core/css/css_image_generator_value.cc`, `third_party/blink/renderer/core/css/css_image_generator_value.h`, `third_party/blink/renderer/core/css/css_to_length_conversion_data.cc`, `third_party/blink/renderer/core/css/css_to_length_conversion_data.h`, `third_party/blink/renderer/core/css/css_value.cc`, `third_party/blink/renderer/core/css/properties/css_parsing_utils.cc`, `third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc`, `third_party/blink/renderer/core/css/resolver/style_builder_converter.cc`, `third_party/blink/renderer/core/dom/element.cc`, `third_party/blink/renderer/core/editing/layout_selection.cc`, `third_party/blink/renderer/core/highlight/highlight_style_utils.cc`, `third_party/blink/renderer/core/layout/ink_overflow.cc`, `third_party/blink/renderer/core/layout/inline/caret_rect.cc`, `third_party/blink/renderer/core/layout/inline/caret_rect.h`, `third_party/blink/renderer/core/layout/inline/fragment_item.cc`, `third_party/blink/renderer/core/layout/inline/inline_cursor.cc`, `third_party/blink/renderer/core/layout/inline/used_font.cc`, `third_party/blink/renderer/core/layout/inline/used_font.h`, `third_party/blink/renderer/core/layout/layout_block.cc`, `third_party/blink/renderer/core/layout/layout_block.h`, `third_party/blink/renderer/core/layout/layout_block_flow.cc`, `third_party/blink/renderer/core/layout/layout_block_flow.h`, `third_party/blink/renderer/core/layout/layout_box.cc`, `third_party/blink/renderer/core/layout/layout_box_model_object.cc`, `third_party/blink/renderer/core/layout/layout_box_model_object.h`, `third_party/blink/renderer/core/layout/layout_image.cc`, `third_party/blink/renderer/core/layout/layout_image.h`, `third_party/blink/renderer/core/layout/layout_inline.cc`, `third_party/blink/renderer/core/layout/layout_inline.h`, `third_party/blink/renderer/core/layout/layout_object.cc`, `third_party/blink/renderer/core/layout/layout_replaced.cc`, `third_party/blink/renderer/core/layout/layout_replaced.h`, `third_party/blink/renderer/core/layout/layout_theme.cc`, `third_party/blink/renderer/core/layout/layout_theme.h`, `third_party/blink/renderer/core/layout/layout_theme_ios.mm`, `third_party/blink/renderer/core/layout/layout_theme_mac.mm`, `third_party/blink/renderer/core/layout/out_of_flow_layout_part.cc`, `third_party/blink/renderer/core/layout/outline_rect_collector.cc`, `third_party/blink/renderer/core/layout/outline_rect_collector.h`, `third_party/blink/renderer/core/layout/physical_box_fragment.cc`, `third_party/blink/renderer/core/layout/svg/layout_svg_resource_container.cc`, `third_party/blink/renderer/core/layout/text_decoration_offset.cc`, `third_party/blink/renderer/core/layout/text_decoration_offset.h`, `third_party/blink/renderer/core/style/border_edge.cc`, `third_party/blink/renderer/core/style/computed_style.cc`, `third_party/blink/renderer/core/style/fill_layer.cc`, `third_party/blink/renderer/core/style/fill_layer.h`, `third_party/blink/renderer/core/style/filter_operation.cc`, `third_party/blink/renderer/core/style/filter_operations.cc`, `third_party/blink/renderer/core/style/shadow_data.cc`, `third_party/blink/renderer/core/style/shadow_list.cc`, `third_party/blink/renderer/core/style/style_fetched_image.cc`, `third_party/blink/renderer/core/style/style_fetched_image.h`, `third_party/blink/renderer/core/style/style_generated_image.cc`, `third_party/blink/renderer/core/style/style_generated_image.h`, `third_party/blink/renderer/core/style/style_image.cc`, `third_party/blink/renderer/core/style/style_image.h`, `third_party/blink/renderer/core/style/style_mask_source_image.cc`, `third_party/blink/renderer/core/style/style_mask_source_image.h`, `third_party/blink/renderer/core/svg/graphics/filters/svg_fe_image.cc`, `third_party/blink/renderer/core/svg/graphics/filters/svg_fe_image.h`, `third_party/blink/renderer/core/svg/graphics/filters/svg_filter_builder.cc`, `third_party/blink/renderer/core/svg/properties/svg_animated_property.h`, `third_party/blink/renderer/core/svg/svg_animated_length.cc`, `third_party/blink/renderer/core/svg/svg_component_transfer_function_element.cc`, `third_party/blink/renderer/core/svg/svg_enumeration.cc`, `third_party/blink/renderer/core/svg/svg_fe_blend_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_color_matrix_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_component_transfer_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_composite_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_convolve_matrix_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_diffuse_lighting_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_displacement_map_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_distant_light_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_drop_shadow_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_flood_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_func_a_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_func_b_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_func_g_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_func_r_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_gaussian_blur_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_image_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_light_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_merge_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_merge_node_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_morphology_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_offset_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_point_light_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_specular_lighting_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_spot_light_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_tile_element.cc`, `third_party/blink/renderer/core/svg/svg_fe_turbulence_element.cc`, `third_party/blink/renderer/core/svg/svg_filter_element.cc`, `third_party/blink/renderer/core/svg/svg_filter_primitive_standard_attributes.cc`, `third_party/blink/renderer/core/svg/svg_integer.cc`, `third_party/blink/renderer/core/svg/svg_integer_optional_integer.cc`, `third_party/blink/renderer/core/svg/svg_length.cc`, `third_party/blink/renderer/core/svg/svg_number.cc`, `third_party/blink/renderer/core/svg/svg_number_list.cc`, `third_party/blink/renderer/core/svg/svg_number_optional_number.cc`, `third_party/blink/renderer/core/svg/svg_parser_utilities.cc`, `third_party/blink/renderer/core/svg/svg_preserve_aspect_ratio.cc`, `third_party/blink/renderer/core/svg/svg_string.h`, `third_party/blink/renderer/platform/geometry/stroke_data.cc`, `third_party/blink/renderer/platform/geometry/stroke_data.h`, `third_party/blink/renderer/platform/graphics/box_reflection.cc`, `third_party/blink/renderer/platform/graphics/box_reflection.h`, `third_party/blink/renderer/platform/graphics/canvas_child_paint_state.cc`, `third_party/blink/renderer/platform/graphics/canvas_child_paint_state.h`, `third_party/blink/renderer/platform/graphics/compositing_reasons.cc`, `third_party/blink/renderer/platform/graphics/compositing_reasons.h`, `third_party/blink/renderer/platform/graphics/filters/fe_blend.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_box_reflect.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_box_reflect.h`, `third_party/blink/renderer/platform/graphics/filters/fe_color_matrix.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_component_transfer.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_composite.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_convolve_matrix.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_diffuse_lighting.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_displacement_map.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_drop_shadow.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_flood.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_flood.h`, `third_party/blink/renderer/platform/graphics/filters/fe_gaussian_blur.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_lighting.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_merge.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_morphology.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_offset.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_specular_lighting.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_tile.cc`, `third_party/blink/renderer/platform/graphics/filters/fe_turbulence.cc`, `third_party/blink/renderer/platform/graphics/filters/filter.cc`, `third_party/blink/renderer/platform/graphics/filters/filter.h`, `third_party/blink/renderer/platform/graphics/filters/filter_effect.cc`, `third_party/blink/renderer/platform/graphics/filters/paint_filter_builder.cc`, `third_party/blink/renderer/platform/graphics/filters/paint_filter_builder.h`, `third_party/blink/renderer/platform/graphics/overlay_scrollbar_clip_behavior.h`, `third_party/blink/renderer/platform/graphics/styled_stroke_data.cc`, `third_party/blink/renderer/platform/graphics/styled_stroke_data.h`.

## Rechrom Paint package responsibility map

These are responsibility boundaries, not claims that every corresponding Chromium source file has been audited.

| Rechrom package | Responsibility |
|---|---|
| `layout_fragment/prepaint` | exports source Layout fragments, style facts, scroll facts, and retained revisions |
| `layoutng_foundation/paint_model` | owns shared paint identities and Chromium compositing-reason constants |
| `paint_model` | owns property-tree, geometry-mapping, cull-rect, clip, and filter value types |
| `prepaint` | builds paint properties, updates cull rects, and computes paint invalidation |
| `paint_walk` | walks paint layers/fragments in paint order and selects paint phases |
| `paint_record` | records display items and drawing operations through PaintController and painters |
| `paint_chunk` | groups display items by property state and carries hit-test/capture metadata |
| `paint_reuse` | compares retained pre-paint state and admits reusable paint output |
| `paint_commit` | assembles the PaintArtifact and publishes scroll paint updates |
| `paint` | orchestrates the complete Paint phase across the responsibility packages above |

## Verified backbone by Rechrom package

This table groups only source files with manually assigned Rechrom ownership. It does not assign the still-unmapped Chromium files to a package by filename guess.

| Rechrom package | Files | Code | Translated | Unsupported | N/A | Remaining |
|---|---:|---:|---:|---:|---:|---:|
| `browser/dom/document_loader/image_decoder/image_resource/layoutng/layout_invalidation/layout_fragment/prepaint/paint_record/raster_commands/raster/compositor` | 1 | 89 | 48 | 28 | 13 | 0 |
| `browser/dom/http_transport/resource/document_loader/image_resource/layoutng/layout_invalidation/layout_fragment/prepaint/paint_record/raster_commands/raster/compositor` | 1 | 203 | 146 | 23 | 34 | 0 |
| `browser/dom/layoutng_assembly/layout_invalidation/layout_fragment/prepaint/paint_record` | 1 | 135 | 117 | 5 | 13 | 0 |
| `browser/dom/style/http_transport/url_loader/resource/document_loader/image_resource/layoutng_assembly/layout_invalidation/layout_fragment/prepaint/paint_record` | 1 | 311 | 255 | 30 | 26 | 0 |
| `browser/layout_fragment/prepaint/paint_record/compositor` | 2 | 369 | 0 | 287 | 82 | 0 |
| `browser/layout_invalidation/layout_algo_inline/layout_fragment` | 1 | 746 | 124 | 0 | 0 | 622 |
| `browser/layout_invalidation/layout_fragment` | 1 | 850 | 59 | 0 | 0 | 791 |
| `browser/layout_invalidation/layout_fragment/layout_algo_replaced` | 1 | 3600 | 62 | 0 | 0 | 3538 |
| `browser/layout_invalidation/layoutng/layout_algo_replaced/layout_fragment` | 1 | 942 | 39 | 0 | 0 | 903 |
| `browser/layoutng/layout_fragment/paint/paint_walk/paint_record` | 1 | 59 | 15 | 8 | 36 | 0 |
| `browser/paint` | 1 | 22 | 4 | 1 | 17 | 0 |
| `canvas/layout_fragment/paint_record/raster` | 2 | 94 | 68 | 0 | 26 | 0 |
| `compositor` | 6 | 1004 | 915 | 0 | 89 | 0 |
| `dom` | 9 | 868 | 278 | 181 | 196 | 213 |
| `dom/image_resource/paint_model/raster` | 3 | 420 | 113 | 142 | 165 | 0 |
| `dom/layout_algo_replaced/layout_fragment/paint_record/browser` | 1 | 10978 | 57 | 0 | 0 | 10921 |
| `dom/layout_fragment/paint_model/raster` | 1 | 116 | 34 | 11 | 71 | 0 |
| `dom/layoutng_style/layout_fragment/paint_record/image_resource/raster` | 2 | 155 | 0 | 125 | 30 | 0 |
| `dom/paint_model/raster` | 31 | 2812 | 1106 | 173 | 1533 | 0 |
| `dom/raster` | 1 | 402 | 237 | 92 | 73 | 0 |
| `image_resource/raster` | 1 | 13 | 0 | 0 | 13 | 0 |
| `layout_algo_block/layout_fragment` | 4 | 1247 | 60 | 0 | 0 | 1187 |
| `layout_algo_grid/layout_algo_flex/layout_fragment/paint_record` | 2 | 368 | 332 | 0 | 36 | 0 |
| `layout_algo_inline/layout_fragment` | 1 | 183 | 11 | 0 | 0 | 172 |
| `layout_algo_inline/layout_fragment/paint_record` | 2 | 116 | 23 | 76 | 17 | 0 |
| `layout_algo_out_of_flow/layout_fragment` | 1 | 2184 | 25 | 0 | 0 | 2159 |
| `layout_algo_replaced/layout_fragment/paint_model/paint_record/interaction/browser` | 2 | 573 | 476 | 40 | 57 | 0 |
| `layout_algo_replaced/layout_fragment/prepaint/paint_record/paint_walk/interaction` | 2 | 550 | 360 | 124 | 66 | 0 |
| `layout_fragment` | 4 | 276 | 237 | 0 | 39 | 0 |
| `layout_fragment/layout_algo_inline` | 3 | 799 | 299 | 61 | 14 | 425 |
| `layout_fragment/layout_invalidation/layout_algo_inline/layout_algo_replaced` | 1 | 4437 | 182 | 7 | 0 | 4248 |
| `layout_fragment/paint_record` | 32 | 8038 | 2244 | 2144 | 727 | 2923 |
| `layout_fragment/paint_record/browser` | 2 | 109 | 0 | 82 | 27 | 0 |
| `layout_fragment/paint_record/interaction` | 4 | 680 | 111 | 193 | 59 | 317 |
| `layout_fragment/paint_record/paint_walk/paint_chunk` | 2 | 185 | 69 | 86 | 30 | 0 |
| `layout_fragment/paint_record/raster` | 6 | 1681 | 836 | 735 | 110 | 0 |
| `layout_fragment/paint_record/raster/compositor` | 3 | 235 | 167 | 20 | 48 | 0 |
| `layout_fragment/paint_record/raster_commands/raster/compositor` | 1 | 67 | 58 | 0 | 9 | 0 |
| `layout_fragment/prepaint` | 11 | 5215 | 2973 | 1961 | 281 | 0 |
| `layout_fragment/prepaint/paint_model` | 2 | 371 | 335 | 0 | 36 | 0 |
| `layout_fragment/prepaint/paint_record` | 7 | 1087 | 891 | 90 | 106 | 0 |
| `layout_fragment/prepaint/paint_record/browser/compositor` | 2 | 150 | 0 | 112 | 38 | 0 |
| `layout_fragment/prepaint/paint_record/paint_walk` | 6 | 979 | 802 | 44 | 133 | 0 |
| `layout_fragment/prepaint/paint_record/raster` | 2 | 928 | 168 | 674 | 86 | 0 |
| `layout_fragment/prepaint/paint_walk` | 6 | 3328 | 2124 | 709 | 495 | 0 |
| `layout_invalidation/prepaint` | 1 | 23 | 17 | 0 | 6 | 0 |
| `layoutng` | 6 | 1265 | 135 | 0 | 21 | 1109 |
| `layoutng/layout_fragment` | 3 | 253 | 53 | 0 | 17 | 183 |
| `layoutng/layout_fragment/paint_record/paint_chunk/compositor` | 2 | 164 | 129 | 0 | 35 | 0 |
| `layoutng/paint_record` | 2 | 183 | 15 | 131 | 37 | 0 |
| `layoutng_foundation/compositor` | 1 | 9 | 4 | 0 | 5 | 0 |
| `layoutng_foundation/paint_commit` | 2 | 111 | 42 | 36 | 33 | 0 |
| `layoutng_foundation/paint_model` | 2 | 325 | 304 | 0 | 21 | 0 |
| `layoutng_style` | 3 | 1402 | 101 | 0 | 20 | 1281 |
| `layoutng_style/layout_fragment` | 7 | 2948 | 262 | 30 | 72 | 2584 |
| `layoutng_style/layout_fragment/dom/paint_model/raster` | 1 | 70 | 50 | 0 | 20 | 0 |
| `layoutng_style/layout_fragment/paint_record` | 2 | 787 | 746 | 0 | 41 | 0 |
| `layoutng_style/style/dom/image_resource/paint_record/raster` | 2 | 279 | 168 | 74 | 37 | 0 |
| `layoutng_style/style/layout_fragment/paint_record/document_image` | 2 | 198 | 142 | 34 | 22 | 0 |
| `layoutng_style/style_resolve` | 2 | 746 | 36 | 32 | 3 | 675 |
| `layoutng_style/style_resolve/layout_fragment` | 6 | 2785 | 1143 | 249 | 75 | 1318 |
| `paint` | 2 | 26 | 14 | 0 | 12 | 0 |
| `paint_chunk` | 12 | 1022 | 725 | 120 | 177 | 0 |
| `paint_commit` | 2 | 147 | 112 | 7 | 28 | 0 |
| `paint_model` | 15 | 1710 | 1358 | 38 | 314 | 0 |
| `paint_model/compositor` | 15 | 2693 | 2225 | 74 | 394 | 0 |
| `paint_model/interaction` | 2 | 91 | 22 | 42 | 27 | 0 |
| `paint_model/paint_record` | 2 | 78 | 19 | 37 | 22 | 0 |
| `paint_model/prepaint` | 1 | 369 | 192 | 68 | 109 | 0 |
| `paint_model/prepaint/paint_record` | 2 | 247 | 71 | 140 | 36 | 0 |
| `paint_model/prepaint/paint_record/paint_walk` | 1 | 10 | 5 | 0 | 5 | 0 |
| `paint_model/prepaint/paint_walk` | 2 | 114 | 78 | 0 | 36 | 0 |
| `paint_model/prepaint/raster` | 4 | 568 | 483 | 0 | 85 | 0 |
| `paint_model/raster` | 23 | 2133 | 972 | 8 | 886 | 267 |
| `paint_record` | 50 | 10694 | 7611 | 2000 | 1083 | 0 |
| `paint_record/compositor` | 6 | 664 | 450 | 103 | 111 | 0 |
| `paint_record/paint_chunk` | 1 | 83 | 73 | 0 | 10 | 0 |
| `paint_record/paint_commit/compositor` | 2 | 125 | 100 | 0 | 25 | 0 |
| `paint_record/paint_walk` | 5 | 295 | 217 | 28 | 50 | 0 |
| `paint_record/paint_walk/interaction` | 2 | 2654 | 1892 | 510 | 252 | 0 |
| `paint_record/raster` | 4 | 65 | 0 | 31 | 34 | 0 |
| `paint_walk` | 2 | 658 | 351 | 243 | 64 | 0 |
| `paint_walk/interaction` | 2 | 211 | 148 | 35 | 28 | 0 |
| `prepaint` | 24 | 7762 | 3993 | 2782 | 987 | 0 |
| `prepaint/paint_record` | 6 | 1398 | 875 | 404 | 119 | 0 |
| `prepaint/paint_walk/interaction` | 2 | 34 | 10 | 0 | 24 | 0 |
| `raster` | 1 | 9 | 0 | 0 | 9 | 0 |
| `style/layout_fragment/prepaint/paint_record/raster` | 2 | 215 | 176 | 11 | 28 | 0 |
| `style_parser/style_resolve` | 2 | 678 | 92 | 0 | 0 | 586 |
| `style_parser/style_resolve/layoutng_style/layout_algo_replaced/browser` | 3 | 20708 | 163 | 0 | 0 | 20545 |
| `style_resolve/layoutng_style/layout_algo_replaced` | 1 | 3752 | 9 | 0 | 0 | 3743 |

## Verified backbone detail

| Entity | Rechrom package | Chromium source | Code | Translated | Unsupported | N/A | Remaining |
|---|---|---|---:|---:|---:|---:|---:|
| BackgroundImageGeometry | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/background_image_geometry.cc` | 585 | 543 | 31 | 11 | 0 |
| BackgroundImageGeometry | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/background_image_geometry.h` | 95 | 66 | 10 | 19 | 0 |
| BlockPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/block_paint_invalidator.cc` | 17 | 15 | 2 | 0 | 0 |
| BlockPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/block_paint_invalidator.h` | 17 | 17 | 0 | 0 | 0 |
| BorderShapePainter | `paint_record` | `third_party/blink/renderer/core/paint/border_shape_painter.cc` | 290 | 0 | 267 | 23 | 0 |
| BorderShapePainter | `paint_record` | `third_party/blink/renderer/core/paint/border_shape_painter.h` | 46 | 0 | 35 | 11 | 0 |
| BorderShapeUtils | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/border_shape_utils.cc` | 64 | 1 | 57 | 6 | 0 |
| BorderShapeUtils | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/border_shape_utils.h` | 22 | 0 | 12 | 10 | 0 |
| BoxBackgroundPaintContext | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/box_background_paint_context.cc` | 341 | 267 | 50 | 24 | 0 |
| BoxBackgroundPaintContext | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/box_background_paint_context.h` | 88 | 60 | 3 | 25 | 0 |
| BoxBorderPainter | `paint_record` | `third_party/blink/renderer/core/paint/box_border_painter.cc` | 1965 | 1106 | 803 | 56 | 0 |
| BoxBorderPainter | `paint_record` | `third_party/blink/renderer/core/paint/box_border_painter.h` | 156 | 134 | 0 | 22 | 0 |
| BoxDecorationData | `paint_record` | `third_party/blink/renderer/core/paint/box_decoration_data.cc` | 45 | 41 | 0 | 4 | 0 |
| BoxDecorationData | `paint_record` | `third_party/blink/renderer/core/paint/box_decoration_data.h` | 132 | 114 | 0 | 18 | 0 |
| BoxFragmentPainter | `paint_record/paint_walk/interaction` | `third_party/blink/renderer/core/paint/box_fragment_painter.cc` | 2354 | 1723 | 416 | 215 | 0 |
| BoxFragmentPainter | `paint_record/paint_walk/interaction` | `third_party/blink/renderer/core/paint/box_fragment_painter.h` | 300 | 169 | 94 | 37 | 0 |
| BoxPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/box_painter.cc` | 99 | 62 | 16 | 21 | 0 |
| BoxPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/box_painter.h` | 44 | 28 | 0 | 16 | 0 |
| BoxPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/box_paint_invalidator.cc` | 386 | 376 | 10 | 0 | 0 |
| BoxPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/box_paint_invalidator.h` | 42 | 42 | 0 | 0 | 0 |
| BoxPainterBase | `paint_record` | `third_party/blink/renderer/core/paint/box_painter_base.cc` | 1314 | 926 | 333 | 55 | 0 |
| BoxPainterBase | `paint_record` | `third_party/blink/renderer/core/paint/box_painter_base.h` | 166 | 117 | 8 | 41 | 0 |
| ClipPathClipper | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/clip_path_clipper.cc` | 606 | 226 | 339 | 41 | 0 |
| ClipPathClipper | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/clip_path_clipper.h` | 62 | 17 | 24 | 21 | 0 |
| ClipRect | `paint_model/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/clip_rect.cc` | 61 | 47 | 0 | 14 | 0 |
| ClipRect | `paint_model/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/clip_rect.h` | 53 | 31 | 0 | 22 | 0 |
| CompositingReasonFinder | `prepaint` | `third_party/blink/renderer/core/paint/compositing/compositing_reason_finder.cc` | 447 | 210 | 206 | 31 | 0 |
| CompositingReasonFinder | `prepaint` | `third_party/blink/renderer/core/paint/compositing/compositing_reason_finder.h` | 24 | 6 | 4 | 14 | 0 |
| ContouredBorderGeometry | `paint_model/prepaint/paint_record` | `third_party/blink/renderer/core/paint/contoured_border_geometry.cc` | 212 | 65 | 127 | 20 | 0 |
| ContouredBorderGeometry | `paint_model/prepaint/paint_record` | `third_party/blink/renderer/core/paint/contoured_border_geometry.h` | 35 | 6 | 13 | 16 | 0 |
| CullRectUpdater | `prepaint` | `third_party/blink/renderer/core/paint/cull_rect_updater.cc` | 528 | 378 | 113 | 37 | 0 |
| CullRectUpdater | `prepaint` | `third_party/blink/renderer/core/paint/cull_rect_updater.h` | 74 | 29 | 22 | 23 | 0 |
| FieldsetPaintInfo | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/fieldset_paint_info.cc` | 41 | 37 | 0 | 4 | 0 |
| FieldsetPaintInfo | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/fieldset_paint_info.h` | 18 | 9 | 0 | 9 | 0 |
| FieldsetPainter | `paint_record` | `third_party/blink/renderer/core/paint/fieldset_painter.cc` | 131 | 105 | 8 | 18 | 0 |
| FieldsetPainter | `paint_record` | `third_party/blink/renderer/core/paint/fieldset_painter.h` | 25 | 11 | 0 | 14 | 0 |
| FrameSetPainter | `paint_record` | `third_party/blink/renderer/core/paint/frame_set_painter.cc` | 121 | 111 | 0 | 10 | 0 |
| FrameSetPainter | `paint_record` | `third_party/blink/renderer/core/paint/frame_set_painter.h` | 38 | 23 | 0 | 15 | 0 |
| GeometryBoxUtils | `prepaint` | `third_party/blink/renderer/core/paint/geometry_box_utils.cc` | 28 | 23 | 0 | 5 | 0 |
| GeometryBoxUtils | `prepaint` | `third_party/blink/renderer/core/paint/geometry_box_utils.h` | 13 | 3 | 0 | 10 | 0 |
| HTMLCanvasPainter | `canvas/layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/html_canvas_painter.cc` | 76 | 61 | 0 | 15 | 0 |
| HTMLCanvasPainterContract | `canvas/layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/html_canvas_painter.h` | 18 | 7 | 0 | 11 | 0 |
| ImagePainter | `browser/dom/http_transport/resource/document_loader/image_resource/layoutng/layout_invalidation/layout_fragment/prepaint/paint_record/raster_commands/raster/compositor` | `third_party/blink/renderer/core/paint/image_painter.cc` | 203 | 146 | 23 | 34 | 0 |
| ImagePainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/image_painter.h` | 24 | 9 | 0 | 15 | 0 |
| InlineBoxFragmentPainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/inline_box_fragment_painter.cc` | 509 | 433 | 44 | 32 | 0 |
| InlineBoxFragmentPainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/inline_box_fragment_painter.h` | 206 | 155 | 0 | 51 | 0 |
| ObjectPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/object_paint_invalidator.cc` | 85 | 66 | 19 | 0 | 0 |
| ObjectPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/object_paint_invalidator.h` | 43 | 40 | 3 | 0 | 0 |
| PaintFlags | `paint_model` | `third_party/blink/renderer/core/paint/paint_flags.h` | 18 | 9 | 0 | 9 | 0 |
| PaintInfo | `paint_record` | `third_party/blink/renderer/core/paint/paint_info.h` | 133 | 73 | 27 | 33 | 0 |
| PaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/paint_invalidator.cc` | 218 | 95 | 123 | 0 | 0 |
| PaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/paint_invalidator.h` | 67 | 56 | 11 | 0 | 0 |
| OutOfFlowStaticPositionOnPaintLayer | `layout_algo_out_of_flow/layout_fragment` | `third_party/blink/renderer/core/layout/out_of_flow_layout_part.cc` | 2184 | 25 | 0 | 0 | 2159 |
| FilterEffectBuilderCanonicalOperationsAndViewport | `paint_model/prepaint/raster` | `third_party/blink/renderer/core/paint/filter_effect_builder.cc` | 480 | 439 | 0 | 41 | 0 |
| FilterEffectBuilderCanonicalOperationsAndViewport | `paint_model/prepaint/raster` | `third_party/blink/renderer/core/paint/filter_effect_builder.h` | 48 | 26 | 0 | 22 | 0 |
| PaintLayer | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer.cc` | 2207 | 1348 | 557 | 302 | 0 |
| PaintLayer | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer.h` | 532 | 298 | 121 | 113 | 0 |
| PaintLayerClipper | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_clipper.cc` | 158 | 112 | 31 | 15 | 0 |
| PaintLayerClipper | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_clipper.h` | 62 | 49 | 0 | 13 | 0 |
| PaintLayerPaintOrderIterator | `paint_walk/interaction` | `third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.cc` | 120 | 97 | 17 | 6 | 0 |
| PaintLayerPaintOrderIterator | `paint_walk/interaction` | `third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.h` | 91 | 51 | 18 | 22 | 0 |
| PaintLayerPainter | `paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_painter.cc` | 612 | 335 | 235 | 42 | 0 |
| PaintLayerPainter | `paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_painter.h` | 46 | 16 | 8 | 22 | 0 |
| PaintResult | `paint_model/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/paint_result.h` | 10 | 5 | 0 | 5 | 0 |
| PaintLayerScrollableArea | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/paint_layer_scrollable_area.cc` | 2908 | 1349 | 1468 | 91 | 0 |
| PaintLayerScrollableArea | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/paint_layer_scrollable_area.h` | 571 | 259 | 248 | 64 | 0 |
| PaintLayerStackingNode | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_stacking_node.cc` | 313 | 278 | 0 | 35 | 0 |
| PaintLayerStackingNode | `layout_fragment/prepaint/paint_walk` | `third_party/blink/renderer/core/paint/paint_layer_stacking_node.h` | 56 | 39 | 0 | 17 | 0 |
| PaintPhase | `paint_model` | `third_party/blink/renderer/core/paint/paint_phase.cc` | 7 | 3 | 0 | 4 | 0 |
| PaintPhase | `paint_model` | `third_party/blink/renderer/core/paint/paint_phase.h` | 36 | 31 | 0 | 5 | 0 |
| PaintPropertyTreeBuilder | `prepaint` | `third_party/blink/renderer/core/paint/paint_property_tree_builder.cc` | 3847 | 1770 | 1773 | 304 | 0 |
| PaintPropertyTreeBuilder | `prepaint` | `third_party/blink/renderer/core/paint/paint_property_tree_builder.h` | 211 | 118 | 67 | 26 | 0 |
| PrePaintTreeWalk | `prepaint` | `third_party/blink/renderer/core/paint/pre_paint_tree_walk.cc` | 1090 | 591 | 396 | 103 | 0 |
| PrePaintTreeWalk | `prepaint` | `third_party/blink/renderer/core/paint/pre_paint_tree_walk.h` | 165 | 93 | 33 | 39 | 0 |
| RoundedInnerRectClipper | `paint_model/paint_record` | `third_party/blink/renderer/core/paint/rounded_inner_rect_clipper.cc` | 59 | 15 | 37 | 7 | 0 |
| RoundedInnerRectClipper | `paint_model/paint_record` | `third_party/blink/renderer/core/paint/rounded_inner_rect_clipper.h` | 19 | 4 | 0 | 15 | 0 |
| ScopedSVGPaintState | `paint_record` | `third_party/blink/renderer/core/paint/scoped_svg_paint_state.cc` | 140 | 91 | 27 | 22 | 0 |
| ScopedSVGPaintState | `paint_record` | `third_party/blink/renderer/core/paint/scoped_svg_paint_state.h` | 60 | 21 | 10 | 29 | 0 |
| ScrollableAreaPainter | `paint_record` | `third_party/blink/renderer/core/paint/scrollable_area_painter.cc` | 341 | 281 | 32 | 28 | 0 |
| ScrollableAreaPainter | `paint_record` | `third_party/blink/renderer/core/paint/scrollable_area_painter.h` | 47 | 26 | 0 | 21 | 0 |
| SVGBackgroundPaintContext | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/svg_background_paint_context.cc` | 28 | 18 | 0 | 10 | 0 |
| SVGBackgroundPaintContext | `prepaint/paint_record` | `third_party/blink/renderer/core/paint/svg_background_paint_context.h` | 22 | 5 | 0 | 17 | 0 |
| SVGContainerPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_container_painter.cc` | 118 | 79 | 16 | 23 | 0 |
| SVGContainerPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_container_painter.h` | 18 | 5 | 0 | 13 | 0 |
| SVGMaskPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_mask_painter.cc` | 284 | 251 | 6 | 27 | 0 |
| SVGMaskPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_mask_painter.h` | 36 | 17 | 0 | 19 | 0 |
| SVGModelObjectPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_model_object_painter.cc` | 55 | 47 | 0 | 8 | 0 |
| SVGModelObjectPainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_model_object_painter.h` | 46 | 30 | 0 | 16 | 0 |
| SVGShapePainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_shape_painter.cc` | 274 | 244 | 0 | 30 | 0 |
| SVGShapePainter | `paint_record` | `third_party/blink/renderer/core/paint/svg_shape_painter.h` | 30 | 17 | 0 | 13 | 0 |
| TableCellPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/table_cell_paint_invalidator.cc` | 45 | 45 | 0 | 0 | 0 |
| TableCellPaintInvalidator | `prepaint` | `third_party/blink/renderer/core/paint/table_cell_paint_invalidator.h` | 20 | 20 | 0 | 0 | 0 |
| ScopedPaintState | `paint_record/paint_walk` | `third_party/blink/renderer/core/paint/scoped_paint_state.cc` | 141 | 105 | 26 | 10 | 0 |
| ScopedPaintState | `paint_record/paint_walk` | `third_party/blink/renderer/core/paint/scoped_paint_state.h` | 83 | 64 | 2 | 17 | 0 |
| CaretRect | `layout_fragment/paint_record/interaction` | `third_party/blink/renderer/core/layout/inline/caret_rect.cc` | 398 | 61 | 0 | 20 | 317 |
| CaretRect | `layout_fragment/paint_record/interaction` | `third_party/blink/renderer/core/layout/inline/caret_rect.h` | 18 | 12 | 0 | 6 | 0 |
| CompositingReasons | `layoutng_foundation/paint_model` | `third_party/blink/renderer/platform/graphics/compositing_reasons.h` | 151 | 139 | 0 | 12 | 0 |
| CompositingReasons | `layoutng_foundation/paint_model` | `third_party/blink/renderer/platform/graphics/compositing_reasons.cc` | 174 | 165 | 0 | 9 | 0 |
| PaintPropertyNode | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/paint_property_node.cc` | 136 | 68 | 3 | 65 | 0 |
| PaintPropertyNode | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/paint_property_node.h` | 190 | 100 | 22 | 68 | 0 |
| PropertyTreeState | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/property_tree_state.cc` | 156 | 139 | 0 | 17 | 0 |
| PropertyTreeState | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/property_tree_state.h` | 261 | 172 | 17 | 72 | 0 |
| TransformPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/transform_paint_property_node.cc` | 216 | 190 | 11 | 15 | 0 |
| TransformPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/transform_paint_property_node.h` | 343 | 273 | 13 | 57 | 0 |
| ScrollPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/scroll_paint_property_node.cc` | 124 | 99 | 13 | 12 | 0 |
| ScrollPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/scroll_paint_property_node.h` | 156 | 106 | 4 | 46 | 0 |
| ClipPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/clip_paint_property_node.cc` | 101 | 84 | 5 | 12 | 0 |
| ClipPaintPropertyNode | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/clip_paint_property_node.h` | 179 | 118 | 11 | 50 | 0 |
| CanvasChildPaintState | `paint_model` | `third_party/blink/renderer/platform/graphics/canvas_child_paint_state.cc` | 59 | 54 | 0 | 5 | 0 |
| CanvasChildPaintState | `paint_model` | `third_party/blink/renderer/platform/graphics/canvas_child_paint_state.h` | 36 | 21 | 0 | 15 | 0 |
| EffectPaintPropertyNode | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/effect_paint_property_node.cc` | 244 | 224 | 0 | 20 | 0 |
| EffectPaintPropertyNode | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/effect_paint_property_node.h` | 311 | 252 | 13 | 46 | 0 |
| CullRect | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/cull_rect.cc` | 391 | 374 | 0 | 17 | 0 |
| CullRect | `paint_model` | `third_party/blink/renderer/platform/graphics/paint/cull_rect.h` | 65 | 39 | 0 | 26 | 0 |
| DisplayItemClientTypes | `layout_fragment/paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item_client_types.h` | 13 | 7 | 0 | 6 | 0 |
| DisplayItemCacheSkipper | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item_cache_skipper.h` | 23 | 15 | 0 | 8 | 0 |
| DisplayItem | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item.cc` | 236 | 181 | 23 | 32 | 0 |
| DisplayItem | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item.h` | 242 | 201 | 2 | 39 | 0 |
| DisplayItemClient | `layoutng_foundation/paint_commit` | `third_party/blink/renderer/platform/graphics/paint/display_item_client.cc` | 23 | 0 | 15 | 8 | 0 |
| DisplayItemClient | `layoutng_foundation/paint_commit` | `third_party/blink/renderer/platform/graphics/paint/display_item_client.h` | 88 | 42 | 21 | 25 | 0 |
| DisplayItemList | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item_list.cc` | 41 | 27 | 6 | 8 | 0 |
| DisplayItemList | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/display_item_list.h` | 208 | 166 | 4 | 38 | 0 |
| DrawingRecorder | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/drawing_recorder.cc` | 38 | 17 | 14 | 7 | 0 |
| DrawingRecorder | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/drawing_recorder.h` | 77 | 32 | 25 | 20 | 0 |
| GeometryMapper | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc` | 496 | 485 | 0 | 11 | 0 |
| GeometryMapper | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h` | 151 | 137 | 0 | 14 | 0 |
| OverlayScrollbarClipBehavior | `layoutng_foundation/compositor` | `third_party/blink/renderer/platform/graphics/overlay_scrollbar_clip_behavior.h` | 9 | 4 | 0 | 5 | 0 |
| FloatClipRect | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h` | 108 | 94 | 0 | 14 | 0 |
| GeometryMapperClipCache | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc` | 50 | 36 | 0 | 14 | 0 |
| GeometryMapperClipCache | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h` | 70 | 41 | 0 | 29 | 0 |
| GeometryMapperTransformCache | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc` | 122 | 117 | 0 | 5 | 0 |
| GeometryMapperTransformCache | `paint_model/compositor` | `third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h` | 160 | 134 | 0 | 26 | 0 |
| HitTestData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/hit_test_data.cc` | 89 | 80 | 0 | 9 | 0 |
| HitTestData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/hit_test_data.h` | 39 | 26 | 1 | 12 | 0 |
| IgnorePaintTimingScope | `paint_record/paint_walk` | `third_party/blink/renderer/platform/graphics/paint/ignore_paint_timing_scope.cc` | 5 | 2 | 0 | 3 | 0 |
| IgnorePaintTimingScope | `paint_record/paint_walk` | `third_party/blink/renderer/platform/graphics/paint/ignore_paint_timing_scope.h` | 34 | 26 | 0 | 8 | 0 |
| LayerSelectionData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/layer_selection_data.h` | 16 | 5 | 0 | 11 | 0 |
| PaintArtifact | `paint_commit` | `third_party/blink/renderer/platform/graphics/paint/paint_artifact.cc` | 88 | 76 | 5 | 7 | 0 |
| PaintArtifact | `paint_commit` | `third_party/blink/renderer/platform/graphics/paint/paint_artifact.h` | 59 | 36 | 2 | 21 | 0 |
| PaintChunk | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/paint_chunk.cc` | 103 | 75 | 0 | 28 | 0 |
| PaintChunk | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/paint_chunk.h` | 137 | 74 | 32 | 31 | 0 |
| PaintChunkSubset | `paint_record/paint_commit/compositor` | `third_party/blink/renderer/platform/graphics/paint/paint_chunk_subset.cc` | 19 | 15 | 0 | 4 | 0 |
| PaintChunkSubset | `paint_record/paint_commit/compositor` | `third_party/blink/renderer/platform/graphics/paint/paint_chunk_subset.h` | 106 | 85 | 0 | 21 | 0 |
| PaintChunker | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/paint_chunker.cc` | 316 | 240 | 68 | 8 | 0 |
| PaintChunker | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/paint_chunker.h` | 104 | 62 | 19 | 23 | 0 |
| PaintRecordBuilder | `paint_record/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_record_builder.cc` | 19 | 0 | 16 | 3 | 0 |
| PaintRecordBuilder | `paint_record/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_record_builder.h` | 31 | 0 | 15 | 16 | 0 |
| PaintBackendNamespaceAliases | `raster` | `third_party/blink/renderer/platform/graphics/paint/paint_canvas.h` | 9 | 0 | 0 | 9 | 0 |
| PaintBackendNamespaceAliases | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_filter.h` | 29 | 0 | 0 | 29 | 0 |
| PaintBackendNamespaceAliases | `image_resource/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_image.h` | 13 | 0 | 0 | 13 | 0 |
| PaintBackendNamespaceAliases | `paint_record/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_record.h` | 8 | 0 | 0 | 8 | 0 |
| PaintBackendNamespaceAliases | `paint_record/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_recorder.h` | 7 | 0 | 0 | 7 | 0 |
| PaintBackendNamespaceAliases | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/paint/paint_shader.h` | 7 | 0 | 0 | 7 | 0 |
| PaintControllerDebugData | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/paint_controller_debug_data.cc` | 107 | 96 | 0 | 11 | 0 |
| PaintController | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/paint_controller.cc` | 726 | 432 | 244 | 50 | 0 |
| PaintController | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/paint_controller.h` | 318 | 191 | 58 | 69 | 0 |
| PaintedSelectionBound | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/painted_selection_bound.h` | 14 | 4 | 0 | 10 | 0 |
| RegionCaptureData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/region_capture_data.cc` | 21 | 16 | 0 | 5 | 0 |
| RegionCaptureData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/region_capture_data.h` | 19 | 5 | 0 | 14 | 0 |
| ScopedDisplayItemFragment | `layout_fragment/paint_record` | `third_party/blink/renderer/platform/graphics/paint/scoped_display_item_fragment.h` | 26 | 18 | 0 | 8 | 0 |
| ScopedEffectivelyInvisible | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/scoped_effectively_invisible.h` | 27 | 20 | 0 | 7 | 0 |
| ScopedPaintChunkProperties | `paint_record/paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/scoped_paint_chunk_properties.h` | 83 | 73 | 0 | 10 | 0 |
| TrackedElementData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/tracked_element_data.cc` | 62 | 57 | 0 | 5 | 0 |
| TrackedElementData | `paint_chunk` | `third_party/blink/renderer/platform/graphics/paint/tracked_element_data.h` | 102 | 81 | 0 | 21 | 0 |
| LayoutObjectOverflowAndStyleSupport | `layout_fragment/layout_invalidation/layout_algo_inline/layout_algo_replaced` | `third_party/blink/renderer/core/layout/layout_object.cc` | 4437 | 182 | 7 | 0 | 4248 |
| LayoutBlockOutlineAndVisualOverflow | `layout_algo_block/layout_fragment` | `third_party/blink/renderer/core/layout/layout_block.cc` | 486 | 35 | 0 | 0 | 451 |
| PhysicalBoxFragmentInkOverflow | `layout_fragment/prepaint` | `third_party/blink/renderer/core/layout/physical_box_fragment.cc` | 1490 | 1248 | 199 | 43 | 0 |
| FragmentItemInkOverflow | `layout_fragment/paint_record` | `third_party/blink/renderer/core/layout/inline/fragment_item.cc` | 1189 | 165 | 0 | 0 | 1024 |
| ComputedStyleDecorationOutsetsAndBorderObscuration | `layoutng_style/layout_fragment` | `third_party/blink/renderer/core/style/computed_style.cc` | 2584 | 51 | 0 | 0 | 2533 |
| FillLayer | `layoutng_style/layout_fragment/paint_record` | `third_party/blink/renderer/core/style/fill_layer.cc` | 457 | 437 | 0 | 20 | 0 |
| FillLayer | `layoutng_style/layout_fragment/paint_record` | `third_party/blink/renderer/core/style/fill_layer.h` | 330 | 309 | 0 | 21 | 0 |
| StyleImage | `layoutng_style` | `third_party/blink/renderer/core/style/style_image.cc` | 26 | 22 | 0 | 4 | 0 |
| StyleImage | `layoutng_style` | `third_party/blink/renderer/core/style/style_image.h` | 90 | 74 | 0 | 16 | 0 |
| CSSValueImageAndGradientHash | `layoutng_style` | `third_party/blink/renderer/core/css/css_value.cc` | 1286 | 5 | 0 | 0 | 1281 |
| CSSGradientValue | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/css/css_gradient_value.cc` | 1978 | 775 | 159 | 6 | 1038 |
| CSSGradientValue | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/css/css_gradient_value.h` | 391 | 66 | 45 | 0 | 280 |
| CSSImageGeneratorValue | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/css/css_image_generator_value.cc` | 196 | 142 | 30 | 24 | 0 |
| CSSImageGeneratorValue | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/css/css_image_generator_value.h` | 76 | 50 | 2 | 24 | 0 |
| StyleGeneratedImage | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/style/style_generated_image.cc` | 88 | 69 | 11 | 8 | 0 |
| StyleGeneratedImage | `layoutng_style/style_resolve/layout_fragment` | `third_party/blink/renderer/core/style/style_generated_image.h` | 56 | 41 | 2 | 13 | 0 |
| CSSToLengthConversionDataContainerSizes | `layoutng_style/style_resolve` | `third_party/blink/renderer/core/css/css_to_length_conversion_data.cc` | 426 | 26 | 23 | 0 | 377 |
| CSSToLengthConversionDataContainerSizes | `layoutng_style/style_resolve` | `third_party/blink/renderer/core/css/css_to_length_conversion_data.h` | 320 | 10 | 9 | 3 | 298 |
| StyleFetchedImage | `layoutng_style/style/dom/image_resource/paint_record/raster` | `third_party/blink/renderer/core/style/style_fetched_image.cc` | 208 | 123 | 65 | 20 | 0 |
| StyleFetchedImage | `layoutng_style/style/dom/image_resource/paint_record/raster` | `third_party/blink/renderer/core/style/style_fetched_image.h` | 71 | 45 | 9 | 17 | 0 |
| StyleMaskSourceImage | `layoutng_style/style/layout_fragment/paint_record/document_image` | `third_party/blink/renderer/core/style/style_mask_source_image.cc` | 138 | 101 | 27 | 10 | 0 |
| StyleMaskSourceImage | `layoutng_style/style/layout_fragment/paint_record/document_image` | `third_party/blink/renderer/core/style/style_mask_source_image.h` | 60 | 41 | 7 | 12 | 0 |
| BorderEdgeOpacity | `layoutng_style/layout_fragment` | `third_party/blink/renderer/core/style/border_edge.cc` | 68 | 17 | 0 | 0 | 51 |
| LayoutReplacedContentGeometryAndOverflow | `layout_algo_replaced/layout_fragment/paint_model/paint_record/interaction/browser` | `third_party/blink/renderer/core/layout/layout_replaced.cc` | 466 | 383 | 40 | 43 | 0 |
| LayoutReplacedContentGeometryContract | `layout_algo_replaced/layout_fragment/paint_model/paint_record/interaction/browser` | `third_party/blink/renderer/core/layout/layout_replaced.h` | 107 | 93 | 0 | 14 | 0 |
| ElementPaintMetadataAndLayoutReplacedCSSOverflowAdmission | `dom/layout_algo_replaced/layout_fragment/paint_record/browser` | `third_party/blink/renderer/core/dom/element.cc` | 10978 | 57 | 0 | 0 | 10921 |
| LayoutReplacedObjectViewBoxProperty | `style_parser/style_resolve/layoutng_style/layout_algo_replaced/browser` | `third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc` | 11234 | 18 | 0 | 0 | 11216 |
| LayoutReplacedObjectViewBoxShapeParsing | `style_parser/style_resolve/layoutng_style/layout_algo_replaced/browser` | `third_party/blink/renderer/core/css/properties/css_parsing_utils.cc` | 8826 | 97 | 0 | 0 | 8729 |
| LayoutReplacedObjectViewBoxComputedShape | `style_parser/style_resolve/layoutng_style/layout_algo_replaced/browser` | `third_party/blink/renderer/core/css/basic_shape_functions.cc` | 648 | 48 | 0 | 0 | 600 |
| LayoutReplacedObjectViewBoxConverter | `style_resolve/layoutng_style/layout_algo_replaced` | `third_party/blink/renderer/core/css/resolver/style_builder_converter.cc` | 3752 | 9 | 0 | 0 | 3743 |
| ObjectViewBoxXYWHValue | `style_parser/style_resolve` | `third_party/blink/renderer/core/css/css_basic_shape_values.h` | 298 | 54 | 0 | 0 | 244 |
| ObjectViewBoxXYWHValue | `style_parser/style_resolve` | `third_party/blink/renderer/core/css/css_basic_shape_values.cc` | 380 | 38 | 0 | 0 | 342 |
| OutlineRectCollector | `layout_fragment` | `third_party/blink/renderer/core/layout/outline_rect_collector.cc` | 67 | 59 | 0 | 8 | 0 |
| OutlineRectCollector | `layout_fragment` | `third_party/blink/renderer/core/layout/outline_rect_collector.h` | 66 | 54 | 0 | 12 | 0 |
| DecoratingBox | `paint_record` | `third_party/blink/renderer/core/paint/decorating_box.h` | 56 | 45 | 0 | 11 | 0 |
| InlinePaintContext | `paint_record` | `third_party/blink/renderer/core/paint/inline_paint_context.h` | 106 | 95 | 0 | 11 | 0 |
| InlinePaintContext | `paint_record` | `third_party/blink/renderer/core/paint/inline_paint_context.cc` | 267 | 258 | 0 | 9 | 0 |
| StyledStrokeData | `paint_model` | `third_party/blink/renderer/platform/graphics/styled_stroke_data.h` | 39 | 28 | 0 | 11 | 0 |
| StyledStrokeData | `paint_model` | `third_party/blink/renderer/platform/graphics/styled_stroke_data.cc` | 116 | 108 | 0 | 8 | 0 |
| StrokeData | `paint_model` | `third_party/blink/renderer/platform/geometry/stroke_data.h` | 34 | 23 | 0 | 11 | 0 |
| StrokeData | `paint_model` | `third_party/blink/renderer/platform/geometry/stroke_data.cc` | 28 | 24 | 0 | 4 | 0 |
| DecorationLinePainterGeometry | `paint_record` | `third_party/blink/renderer/core/paint/decoration_line_painter.h` | 53 | 39 | 0 | 14 | 0 |
| DecorationLinePainterGeometry | `paint_record` | `third_party/blink/renderer/core/paint/decoration_line_painter.cc` | 347 | 333 | 0 | 14 | 0 |
| TextPainterSvgDecorationAndSelection | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/text_painter.cc` | 662 | 462 | 166 | 34 | 0 |
| TextFragmentPainterSelectionAndTextFitOrder | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/text_fragment_painter.cc` | 725 | 312 | 328 | 85 | 0 |
| TextDecorationPainterPasses | `paint_record` | `third_party/blink/renderer/core/paint/text_decoration_painter.cc` | 267 | 229 | 26 | 12 | 0 |
| TextDecorationPainterPasses | `paint_record` | `third_party/blink/renderer/core/paint/text_decoration_painter.h` | 74 | 45 | 5 | 24 | 0 |
| LineRelativeOffsetAndRect | `paint_record` | `third_party/blink/renderer/core/paint/line_relative_rect.h` | 87 | 73 | 0 | 14 | 0 |
| LineRelativeRect | `paint_record` | `third_party/blink/renderer/core/paint/line_relative_rect.cc` | 89 | 85 | 0 | 4 | 0 |
| TextDecorationInfo | `paint_record` | `third_party/blink/renderer/core/paint/text_decoration_info.h` | 163 | 134 | 0 | 29 | 0 |
| TextDecorationInfo | `paint_record` | `third_party/blink/renderer/core/paint/text_decoration_info.cc` | 557 | 539 | 0 | 18 | 0 |
| LayoutThemeMarkerColors | `layoutng` | `third_party/blink/renderer/core/layout/layout_theme.h` | 173 | 2 | 0 | 0 | 171 |
| LayoutThemeMarkerColors | `layoutng` | `third_party/blink/renderer/core/layout/layout_theme.cc` | 817 | 6 | 0 | 0 | 811 |
| LayoutThemeMarkerColors | `layoutng` | `third_party/blink/renderer/core/layout/layout_theme_mac.mm` | 105 | 6 | 0 | 0 | 99 |
| LayoutThemeMarkerColors | `layoutng` | `third_party/blink/renderer/core/layout/layout_theme_ios.mm` | 34 | 6 | 0 | 0 | 28 |
| TextDecorationOffset | `layoutng` | `third_party/blink/renderer/core/layout/text_decoration_offset.h` | 36 | 23 | 0 | 13 | 0 |
| TextDecorationOffset | `layoutng` | `third_party/blink/renderer/core/layout/text_decoration_offset.cc` | 100 | 92 | 0 | 8 | 0 |
| UsedFontPaintScaling | `layout_fragment/layout_algo_inline` | `third_party/blink/renderer/core/layout/inline/used_font.h` | 45 | 34 | 0 | 11 | 0 |
| UsedFontPaintScaling | `layout_fragment/layout_algo_inline` | `third_party/blink/renderer/core/layout/inline/used_font.cc` | 34 | 31 | 0 | 3 | 0 |
| HighlightSelectionDecoration | `layout_fragment/paint_record` | `third_party/blink/renderer/core/highlight/highlight_style_utils.cc` | 524 | 45 | 0 | 0 | 479 |
| HighlightSelectionPaintState | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/highlight_painter.cc` | 1042 | 307 | 690 | 45 | 0 |
| InlineCursorSelectionRect | `layout_fragment/paint_record` | `third_party/blink/renderer/core/layout/inline/inline_cursor.cc` | 1487 | 67 | 0 | 0 | 1420 |
| LayoutSelectionPaintInvalidation | `browser/layout_invalidation/layoutng/layout_algo_replaced/layout_fragment` | `third_party/blink/renderer/core/editing/layout_selection.cc` | 942 | 39 | 0 | 0 | 903 |
| ShadowListSelectionOutsets | `layout_fragment/paint_record` | `third_party/blink/renderer/core/style/shadow_list.cc` | 19 | 13 | 0 | 6 | 0 |
| ShadowDataSelectionOutsets | `layout_fragment/paint_record` | `third_party/blink/renderer/core/style/shadow_data.cc` | 15 | 12 | 0 | 3 | 0 |
| LayoutBoxModelObjectOutlineAndImageInvalidation | `browser/layout_invalidation/layout_fragment` | `third_party/blink/renderer/core/layout/layout_box_model_object.cc` | 850 | 59 | 0 | 0 | 791 |
| DocumentImageClientInvalidation | `browser/layout_invalidation/layout_fragment/layout_algo_replaced` | `third_party/blink/renderer/core/layout/layout_box.cc` | 3600 | 62 | 0 | 0 | 3538 |
| LayoutImageNaturalSizeInvalidationPaintedRectAndOpacity | `browser/dom/style/http_transport/url_loader/resource/document_loader/image_resource/layoutng_assembly/layout_invalidation/layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/layout/layout_image.cc` | 311 | 255 | 30 | 26 | 0 |
| LayoutImagePaintIdentityAndMutableState | `browser/dom/layoutng_assembly/layout_invalidation/layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/layout/layout_image.h` | 135 | 117 | 5 | 13 | 0 |
| LayoutInlineGeometryOutlineAndImageInvalidation | `browser/layout_invalidation/layout_algo_inline/layout_fragment` | `third_party/blink/renderer/core/layout/layout_inline.cc` | 746 | 124 | 0 | 0 | 622 |
| LayoutBlockFlowOutline | `layout_algo_block/layout_fragment` | `third_party/blink/renderer/core/layout/layout_block_flow.cc` | 549 | 17 | 0 | 0 | 532 |
| LayoutBoxModelObjectVisualOverflowAndOutlineContract | `layoutng/layout_fragment` | `third_party/blink/renderer/core/layout/layout_box_model_object.h` | 191 | 8 | 0 | 0 | 183 |
| LayoutBlockOutlineContract | `layout_algo_block/layout_fragment` | `third_party/blink/renderer/core/layout/layout_block.h` | 120 | 4 | 0 | 0 | 116 |
| LayoutBlockFlowOutlineContract | `layout_algo_block/layout_fragment` | `third_party/blink/renderer/core/layout/layout_block_flow.h` | 92 | 4 | 0 | 0 | 88 |
| LayoutInlineVisualOverflowAndOutlineContract | `layout_algo_inline/layout_fragment` | `third_party/blink/renderer/core/layout/layout_inline.h` | 183 | 11 | 0 | 0 | 172 |
| PaintLayerFragment | `prepaint/paint_walk/interaction` | `third_party/blink/renderer/core/paint/paint_layer_fragment.cc` | 9 | 0 | 0 | 9 | 0 |
| PaintLayerFragment | `prepaint/paint_walk/interaction` | `third_party/blink/renderer/core/paint/paint_layer_fragment.h` | 25 | 10 | 0 | 15 | 0 |
| PrePaintSubtreeWalkReasons | `layout_invalidation/prepaint` | `third_party/blink/renderer/core/paint/pre_paint_subtree_walk_reasons.h` | 23 | 17 | 0 | 6 | 0 |
| PrePaintDisableSideEffectsScope | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/pre_paint_disable_side_effects_scope.cc` | 4 | 0 | 0 | 4 | 0 |
| PrePaintDisableSideEffectsScope | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/pre_paint_disable_side_effects_scope.h` | 19 | 0 | 0 | 19 | 0 |
| PaintLayerBoxReflectionGeometry | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/box_reflection_utils.cc` | 66 | 56 | 0 | 10 | 0 |
| PaintLayerBoxReflectionGeometry | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/box_reflection_utils.h` | 10 | 2 | 0 | 8 | 0 |
| StyleFilterOperationsGeometry | `layoutng_style/layout_fragment` | `third_party/blink/renderer/core/style/filter_operations.cc` | 80 | 61 | 14 | 5 | 0 |
| StyleFilterOperationGeometry | `layoutng_style/layout_fragment` | `third_party/blink/renderer/core/style/filter_operation.cc` | 72 | 31 | 16 | 25 | 0 |
| GaussianBlurBounds | `layoutng_style/layout_fragment` | `third_party/blink/renderer/platform/graphics/filters/fe_gaussian_blur.cc` | 61 | 43 | 0 | 18 | 0 |
| DropShadowBounds | `layoutng_style/layout_fragment/dom/paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_drop_shadow.cc` | 70 | 50 | 0 | 20 | 0 |
| BoxReflectionBounds | `layoutng_style/layout_fragment` | `third_party/blink/renderer/platform/graphics/box_reflection.cc` | 41 | 33 | 0 | 8 | 0 |
| BoxReflectionBounds | `layoutng_style/layout_fragment` | `third_party/blink/renderer/platform/graphics/box_reflection.h` | 42 | 26 | 0 | 16 | 0 |
| FEBoxReflect | `paint_model/prepaint/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_box_reflect.cc` | 21 | 10 | 0 | 11 | 0 |
| FEBoxReflect | `paint_model/prepaint/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_box_reflect.h` | 19 | 8 | 0 | 11 | 0 |
| PaintFilterBuilderAndBoxReflectFilter | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/paint_filter_builder.cc` | 117 | 82 | 8 | 27 | 0 |
| PaintFilterBuilderAndBoxReflectFilter | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/paint_filter_builder.h` | 27 | 14 | 0 | 13 | 0 |
| ConvolveMatrixFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_convolve_matrix.cc` | 156 | 82 | 0 | 74 | 0 |
| SvgFeConvolveMatrixElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_convolve_matrix_element.cc` | 227 | 104 | 0 | 123 | 0 |
| SvgNumberParsing | `dom` | `third_party/blink/renderer/core/svg/svg_parser_utilities.cc` | 157 | 126 | 0 | 31 | 0 |
| SvgNumberListValue | `dom` | `third_party/blink/renderer/core/svg/svg_number_list.cc` | 84 | 31 | 44 | 9 | 0 |
| SvgNumberValue | `dom` | `third_party/blink/renderer/core/svg/svg_number.cc` | 97 | 22 | 26 | 49 | 0 |
| SvgNumberOptionalNumberValue | `dom` | `third_party/blink/renderer/core/svg/svg_number_optional_number.cc` | 75 | 18 | 34 | 23 | 0 |
| SvgEnumerationValue | `dom` | `third_party/blink/renderer/core/svg/svg_enumeration.cc` | 47 | 9 | 17 | 21 | 0 |
| SvgIntegerValue | `dom` | `third_party/blink/renderer/core/svg/svg_integer.cc` | 48 | 9 | 26 | 13 | 0 |
| SvgIntegerOptionalIntegerValue | `dom` | `third_party/blink/renderer/core/svg/svg_integer_optional_integer.cc` | 76 | 19 | 34 | 23 | 0 |
| SvgAnimatedPropertyInvalidReset | `dom` | `third_party/blink/renderer/core/svg/properties/svg_animated_property.h` | 240 | 27 | 0 | 0 | 213 |
| SvgAnimatedLengthValue | `dom` | `third_party/blink/renderer/core/svg/svg_animated_length.cc` | 44 | 17 | 0 | 27 | 0 |
| SvgStringValue | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_string.h` | 45 | 10 | 11 | 24 | 0 |
| SvgLengthValue | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_length.cc` | 296 | 73 | 157 | 66 | 0 |
| SvgFilterElementRegion | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_filter_element.cc` | 128 | 33 | 5 | 90 | 0 |
| SvgPreserveAspectRatioValue | `dom/raster` | `third_party/blink/renderer/core/svg/svg_preserve_aspect_ratio.cc` | 402 | 237 | 92 | 73 | 0 |
| SvgComponentTransferFunctionElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_component_transfer_function_element.cc` | 116 | 53 | 0 | 63 | 0 |
| SvgFeFuncRElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_func_r_element.cc` | 5 | 2 | 0 | 3 | 0 |
| SvgFeFuncGElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_func_g_element.cc` | 5 | 2 | 0 | 3 | 0 |
| SvgFeFuncBElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_func_b_element.cc` | 5 | 2 | 0 | 3 | 0 |
| SvgFeFuncAElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_func_a_element.cc` | 5 | 2 | 0 | 3 | 0 |
| SvgFeComponentTransferElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_component_transfer_element.cc` | 73 | 31 | 0 | 42 | 0 |
| ComponentTransferFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_component_transfer.cc` | 175 | 114 | 0 | 61 | 0 |
| SvgFeColorMatrixElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_color_matrix_element.cc` | 95 | 34 | 0 | 61 | 0 |
| ColorMatrixFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_color_matrix.cc` | 176 | 90 | 0 | 86 | 0 |
| SvgFeMergeNodeElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_merge_node_element.cc` | 34 | 3 | 0 | 31 | 0 |
| SvgFeMergeElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_merge_element.cc` | 29 | 16 | 0 | 13 | 0 |
| MergeFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_merge.cc` | 31 | 12 | 0 | 19 | 0 |
| SvgFeOffsetElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_offset_element.cc` | 66 | 20 | 0 | 46 | 0 |
| OffsetFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_offset.cc` | 46 | 18 | 0 | 28 | 0 |
| SvgFeCompositeElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_composite_element.cc` | 138 | 56 | 0 | 82 | 0 |
| CompositeFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_composite.cc` | 175 | 79 | 0 | 96 | 0 |
| SvgFilterPrimitiveStandardAttributes | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_filter_primitive_standard_attributes.cc` | 165 | 76 | 0 | 89 | 0 |
| SvgFeTileElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_tile_element.cc` | 50 | 13 | 0 | 37 | 0 |
| TileFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_tile.cc` | 39 | 23 | 0 | 16 | 0 |
| SvgFeGaussianBlurElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_gaussian_blur_element.cc` | 75 | 32 | 0 | 43 | 0 |
| SvgFeBlendElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_blend_element.cc` | 132 | 74 | 0 | 58 | 0 |
| BlendFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_blend.cc` | 35 | 13 | 0 | 22 | 0 |
| SvgFeMorphologyElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_morphology_element.cc` | 109 | 42 | 0 | 67 | 0 |
| MorphologyFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_morphology.cc` | 90 | 29 | 0 | 61 | 0 |
| SvgFilterBuilderGraphAndTaint | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/graphics/filters/svg_filter_builder.cc` | 172 | 121 | 0 | 51 | 0 |
| FilterEffectGraphValue | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/filter_effect.cc` | 134 | 77 | 0 | 57 | 0 |
| SvgFilterCoordinateContext | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/filter.h` | 42 | 25 | 0 | 17 | 0 |
| SvgFilterCoordinateResolution | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/filter.cc` | 52 | 42 | 0 | 10 | 0 |
| DisplacementMapFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_displacement_map.cc` | 118 | 48 | 0 | 70 | 0 |
| SvgFeDisplacementMapElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_displacement_map_element.cc` | 124 | 48 | 0 | 76 | 0 |
| SvgFeDropShadowElement | `dom/layout_fragment/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_drop_shadow_element.cc` | 116 | 34 | 11 | 71 | 0 |
| SvgFeFloodElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_flood_element.cc` | 42 | 17 | 0 | 25 | 0 |
| FloodFilterEffectContract | `dom/paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_flood.h` | 22 | 8 | 0 | 14 | 0 |
| FloodFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_flood.cc` | 50 | 16 | 0 | 34 | 0 |
| SvgFeImageElement | `dom/image_resource/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_image_element.cc` | 157 | 38 | 20 | 99 | 0 |
| SvgFeImageFilterEffectContract | `dom/image_resource/paint_model/raster` | `third_party/blink/renderer/core/svg/graphics/filters/svg_fe_image.h` | 39 | 9 | 11 | 19 | 0 |
| SvgFeTurbulenceElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_turbulence_element.cc` | 138 | 50 | 0 | 88 | 0 |
| SvgFeDiffuseLightingElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_diffuse_lighting_element.cc` | 140 | 38 | 0 | 102 | 0 |
| SvgFeSpecularLightingElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_specular_lighting_element.cc` | 152 | 43 | 0 | 109 | 0 |
| SvgFeDistantLightElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_distant_light_element.cc` | 13 | 7 | 0 | 6 | 0 |
| SvgFePointLightElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_point_light_element.cc` | 12 | 6 | 0 | 6 | 0 |
| SvgFeSpotLightElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_spot_light_element.cc` | 16 | 9 | 0 | 7 | 0 |
| SvgFeLightElement | `dom/paint_model/raster` | `third_party/blink/renderer/core/svg/svg_fe_light_element.cc` | 183 | 81 | 0 | 102 | 0 |
| TurbulenceFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_turbulence.cc` | 126 | 37 | 0 | 89 | 0 |
| LightingFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_lighting.cc` | 110 | 83 | 0 | 27 | 0 |
| DiffuseLightingFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_diffuse_lighting.cc` | 40 | 20 | 0 | 20 | 0 |
| SpecularLightingFilterEffect | `paint_model/raster` | `third_party/blink/renderer/platform/graphics/filters/fe_specular_lighting.cc` | 54 | 31 | 0 | 23 | 0 |
| SvgFeImageFilterEffect | `dom/image_resource/paint_model/raster` | `third_party/blink/renderer/core/svg/graphics/filters/svg_fe_image.cc` | 224 | 66 | 111 | 47 | 0 |
| SVGFilterRectangleViewportResolution | `paint_model/raster` | `third_party/blink/renderer/core/layout/svg/layout_svg_resource_container.cc` | 304 | 37 | 0 | 0 | 267 |
| InkOverflowTextEffects | `layout_fragment/layout_algo_inline` | `third_party/blink/renderer/core/layout/ink_overflow.cc` | 720 | 234 | 61 | 0 | 425 |
| ScrollbarDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/scrollbar_display_item.cc` | 122 | 79 | 27 | 16 | 0 |
| ScrollbarDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/scrollbar_display_item.h` | 80 | 51 | 2 | 27 | 0 |
| DrawingDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/drawing_display_item.cc` | 265 | 170 | 73 | 22 | 0 |
| DrawingDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/drawing_display_item.h` | 94 | 71 | 1 | 22 | 0 |
| ForeignLayerDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/foreign_layer_display_item.cc` | 55 | 46 | 0 | 9 | 0 |
| ForeignLayerDisplayItem | `paint_record/compositor` | `third_party/blink/renderer/platform/graphics/paint/foreign_layer_display_item.h` | 48 | 33 | 0 | 15 | 0 |
| DisplayItemRasterInvalidator | `compositor` | `third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc` | 203 | 199 | 0 | 4 | 0 |
| DisplayItemRasterInvalidator | `compositor` | `third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.h` | 52 | 44 | 0 | 8 | 0 |
| RasterInvalidator | `compositor` | `third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc` | 332 | 315 | 0 | 17 | 0 |
| RasterInvalidator | `compositor` | `third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h` | 138 | 114 | 0 | 24 | 0 |
| RasterInvalidationTracking | `compositor` | `third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.cc` | 204 | 189 | 0 | 15 | 0 |
| RasterInvalidationTracking | `compositor` | `third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.h` | 75 | 54 | 0 | 21 | 0 |
| PaintUnderInvalidationChecker | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/paint_under_invalidation_checker.cc` | 213 | 189 | 8 | 16 | 0 |
| PaintUnderInvalidationChecker | `paint_record` | `third_party/blink/renderer/platform/graphics/paint/paint_under_invalidation_checker.h` | 44 | 31 | 0 | 13 | 0 |
| VideoPainter | `browser/dom/document_loader/image_decoder/image_resource/layoutng/layout_invalidation/layout_fragment/prepaint/paint_record/raster_commands/raster/compositor` | `third_party/blink/renderer/core/paint/video_painter.cc` | 89 | 48 | 28 | 13 | 0 |
| VideoPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/video_painter.h` | 18 | 9 | 0 | 9 | 0 |
| ViewPainter | `paint_record` | `third_party/blink/renderer/core/paint/view_painter.cc` | 317 | 268 | 13 | 36 | 0 |
| SubsequenceRecorder | `paint_record/paint_walk` | `third_party/blink/renderer/platform/graphics/paint/subsequence_recorder.h` | 32 | 20 | 0 | 12 | 0 |
| MathMLPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/mathml_painter.cc` | 155 | 143 | 0 | 12 | 0 |
| MathMLPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/mathml_painter.h` | 24 | 14 | 0 | 10 | 0 |
| NinePieceImagePainter | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/nine_piece_image_painter.cc` | 175 | 159 | 0 | 16 | 0 |
| NinePieceImagePainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/nine_piece_image_painter.h` | 28 | 14 | 0 | 14 | 0 |
| NinePieceImageGrid | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/nine_piece_image_grid.cc` | 318 | 310 | 0 | 8 | 0 |
| NinePieceImageGrid | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/nine_piece_image_grid.h` | 78 | 63 | 0 | 15 | 0 |
| BoxModelObjectPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/box_model_object_painter.cc` | 55 | 31 | 11 | 13 | 0 |
| BoxModelObjectPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/box_model_object_painter.h` | 34 | 21 | 1 | 12 | 0 |
| CSSMaskPainter | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/css_mask_painter.cc` | 73 | 40 | 20 | 13 | 0 |
| CSSMaskPainter | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/css_mask_painter.h` | 18 | 7 | 0 | 11 | 0 |
| ObjectPainter | `layout_fragment/paint_record/paint_walk/paint_chunk` | `third_party/blink/renderer/core/paint/object_painter.cc` | 153 | 57 | 81 | 15 | 0 |
| ObjectPainter | `layout_fragment/paint_record/paint_walk/paint_chunk` | `third_party/blink/renderer/core/paint/object_painter.h` | 32 | 12 | 5 | 15 | 0 |
| FragmentPainter | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/fragment_painter.cc` | 61 | 16 | 36 | 9 | 0 |
| FragmentPainter | `layout_fragment/prepaint/paint_record` | `third_party/blink/renderer/core/paint/fragment_painter.h` | 26 | 16 | 1 | 9 | 0 |
| TextShadowPainter | `layout_fragment/paint_record/raster/compositor` | `third_party/blink/renderer/core/paint/text_shadow_painter.cc` | 60 | 51 | 0 | 9 | 0 |
| TextShadowPainter | `layout_fragment/paint_record/raster/compositor` | `third_party/blink/renderer/core/paint/text_shadow_painter.h` | 45 | 35 | 0 | 10 | 0 |
| TextCombinePainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/text_combine_painter.cc` | 124 | 110 | 0 | 14 | 0 |
| TextCombinePainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/text_combine_painter.h` | 31 | 22 | 0 | 9 | 0 |
| SVGRootPainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/svg_root_painter.cc` | 84 | 70 | 0 | 14 | 0 |
| SVGRootPainter | `layout_fragment/prepaint/paint_record/paint_walk` | `third_party/blink/renderer/core/paint/svg_root_painter.h` | 25 | 12 | 0 | 13 | 0 |
| TextOverflowPostLayoutSnapshot | `layoutng/layout_fragment` | `third_party/blink/renderer/core/paint/text_overflow_post_layout_snapshot.cc` | 40 | 31 | 0 | 9 | 0 |
| TextOverflowPostLayoutSnapshot | `layoutng/layout_fragment` | `third_party/blink/renderer/core/paint/text_overflow_post_layout_snapshot.h` | 22 | 14 | 0 | 8 | 0 |
| SelectionBoundsRecorder | `layoutng/layout_fragment/paint_record/paint_chunk/compositor` | `third_party/blink/renderer/core/paint/selection_bounds_recorder.cc` | 135 | 112 | 0 | 23 | 0 |
| SelectionBoundsRecorder | `layoutng/layout_fragment/paint_record/paint_chunk/compositor` | `third_party/blink/renderer/core/paint/selection_bounds_recorder.h` | 29 | 17 | 0 | 12 | 0 |
| TextFragmentPainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/text_fragment_painter.h` | 37 | 21 | 0 | 16 | 0 |
| TextPainter | `layout_fragment/paint_record/raster/compositor` | `third_party/blink/renderer/core/paint/text_painter.h` | 130 | 81 | 20 | 29 | 0 |
| FragmentDataIterator | `layout_fragment` | `third_party/blink/renderer/core/paint/fragment_data_iterator.cc` | 60 | 53 | 0 | 7 | 0 |
| FragmentDataIterator | `layout_fragment` | `third_party/blink/renderer/core/paint/fragment_data_iterator.h` | 83 | 71 | 0 | 12 | 0 |
| FragmentData | `layout_fragment/prepaint/paint_model` | `third_party/blink/renderer/core/paint/fragment_data.cc` | 170 | 154 | 0 | 16 | 0 |
| FragmentData | `layout_fragment/prepaint/paint_model` | `third_party/blink/renderer/core/paint/fragment_data.h` | 201 | 181 | 0 | 20 | 0 |
| PaintEvent | `paint` | `third_party/blink/renderer/core/paint/paint_event.h` | 11 | 6 | 0 | 5 | 0 |
| FramePainter | `browser/layoutng/layout_fragment/paint/paint_walk/paint_record` | `third_party/blink/renderer/core/paint/frame_painter.cc` | 59 | 15 | 8 | 36 | 0 |
| FramePainter | `browser/paint` | `third_party/blink/renderer/core/paint/frame_painter.h` | 22 | 4 | 1 | 17 | 0 |
| LayoutObjectCounter | `paint` | `third_party/blink/renderer/core/paint/layout_object_counter.h` | 15 | 8 | 0 | 7 | 0 |
| PaintAutoDarkMode | `layout_fragment/paint_record/raster_commands/raster/compositor` | `third_party/blink/renderer/core/paint/paint_auto_dark_mode.cc` | 67 | 58 | 0 | 9 | 0 |
| PaintAutoDarkMode | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/paint_auto_dark_mode.h` | 39 | 28 | 0 | 11 | 0 |
| URLMetadataUtils | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/url_metadata_utils.cc` | 20 | 0 | 13 | 7 | 0 |
| URLMetadataUtils | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/url_metadata_utils.h` | 12 | 0 | 4 | 8 | 0 |
| EmbeddedObjectPainter | `layout_fragment/paint_record/browser` | `third_party/blink/renderer/core/paint/embedded_object_painter.cc` | 91 | 0 | 73 | 18 | 0 |
| EmbeddedObjectPainter | `layout_fragment/paint_record/browser` | `third_party/blink/renderer/core/paint/embedded_object_painter.h` | 18 | 0 | 9 | 9 | 0 |
| PaintLayerResourceInfo | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/paint_layer_resource_info.cc` | 18 | 0 | 14 | 4 | 0 |
| PaintLayerResourceInfo | `layout_fragment/prepaint` | `third_party/blink/renderer/core/paint/paint_layer_resource_info.h` | 38 | 12 | 12 | 14 | 0 |
| EmbeddedContentPainter | `layout_fragment/prepaint/paint_record/browser/compositor` | `third_party/blink/renderer/core/paint/embedded_content_painter.cc` | 128 | 0 | 100 | 28 | 0 |
| EmbeddedContentPainter | `layout_fragment/prepaint/paint_record/browser/compositor` | `third_party/blink/renderer/core/paint/embedded_content_painter.h` | 22 | 0 | 12 | 10 | 0 |
| SVGImagePainter | `dom/layoutng_style/layout_fragment/paint_record/image_resource/raster` | `third_party/blink/renderer/core/paint/svg_image_painter.cc` | 135 | 0 | 114 | 21 | 0 |
| SVGImagePainter | `dom/layoutng_style/layout_fragment/paint_record/image_resource/raster` | `third_party/blink/renderer/core/paint/svg_image_painter.h` | 20 | 0 | 11 | 9 | 0 |
| SVGObjectPainter | `style/layout_fragment/prepaint/paint_record/raster` | `third_party/blink/renderer/core/paint/svg_object_painter.cc` | 171 | 151 | 9 | 11 | 0 |
| SVGObjectPainter | `style/layout_fragment/prepaint/paint_record/raster` | `third_party/blink/renderer/core/paint/svg_object_painter.h` | 44 | 25 | 2 | 17 | 0 |
| TextPaintStyle | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/text_paint_style.h` | 40 | 20 | 2 | 18 | 0 |
| ReplacedPainter | `layout_algo_replaced/layout_fragment/prepaint/paint_record/paint_walk/interaction` | `third_party/blink/renderer/core/paint/replaced_painter.cc` | 499 | 341 | 118 | 40 | 0 |
| ReplacedPainter | `layout_algo_replaced/layout_fragment/prepaint/paint_record/paint_walk/interaction` | `third_party/blink/renderer/core/paint/replaced_painter.h` | 51 | 19 | 6 | 26 | 0 |
| HighlightPainterContract | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/highlight_painter.h` | 215 | 78 | 93 | 44 | 0 |
| HighlightOverlay | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/highlight_overlay.cc` | 683 | 135 | 384 | 164 | 0 |
| HighlightOverlayContract | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/highlight_overlay.h` | 182 | 9 | 144 | 29 | 0 |
| MarkerRangeMappingContext | `layout_algo_inline/layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/marker_range_mapping_context.cc` | 72 | 21 | 43 | 8 | 0 |
| MarkerRangeMappingContextContract | `layout_algo_inline/layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/marker_range_mapping_context.h` | 44 | 2 | 33 | 9 | 0 |
| StyleableMarkerPainter | `layoutng/paint_record` | `third_party/blink/renderer/core/paint/styleable_marker_painter.cc` | 159 | 15 | 123 | 21 | 0 |
| StyleableMarkerPainterContract | `layoutng/paint_record` | `third_party/blink/renderer/core/paint/styleable_marker_painter.h` | 24 | 0 | 8 | 16 | 0 |
| HitTestingTransformState | `paint_model/interaction` | `third_party/blink/renderer/core/paint/hit_testing_transform_state.cc` | 45 | 18 | 22 | 5 | 0 |
| HitTestingTransformStateContract | `paint_model/interaction` | `third_party/blink/renderer/core/paint/hit_testing_transform_state.h` | 46 | 4 | 20 | 22 | 0 |
| FindPaintOffsetNeedingUpdateScope | `prepaint` | `third_party/blink/renderer/core/paint/find_paint_offset_needing_update.h` | 59 | 0 | 0 | 59 | 0 |
| FindPropertiesNeedingUpdateScope | `prepaint` | `third_party/blink/renderer/core/paint/find_properties_needing_update.h` | 74 | 0 | 0 | 74 | 0 |
| PaintPropertyTreePrinter | `prepaint` | `third_party/blink/renderer/core/paint/paint_property_tree_printer.cc` | 230 | 0 | 0 | 230 | 0 |
| PaintPropertyTreePrinterContract | `prepaint` | `third_party/blink/renderer/core/paint/paint_property_tree_printer.h` | 32 | 0 | 0 | 32 | 0 |
| ViewPainterContract | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/view_painter.h` | 40 | 18 | 0 | 22 | 0 |
| CustomScrollbarTheme | `layout_fragment/paint_record/interaction` | `third_party/blink/renderer/core/paint/custom_scrollbar_theme.cc` | 180 | 23 | 141 | 16 | 0 |
| CustomScrollbarThemeContract | `layout_fragment/paint_record/interaction` | `third_party/blink/renderer/core/paint/custom_scrollbar_theme.h` | 84 | 15 | 52 | 17 | 0 |
| LinkHighlightImpl | `browser/layout_fragment/prepaint/paint_record/compositor` | `third_party/blink/renderer/core/paint/link_highlight_impl.cc` | 285 | 0 | 237 | 48 | 0 |
| LinkHighlightImplContract | `browser/layout_fragment/prepaint/paint_record/compositor` | `third_party/blink/renderer/core/paint/link_highlight_impl.h` | 84 | 0 | 50 | 34 | 0 |
| GapDecorationsPainter | `layout_algo_grid/layout_algo_flex/layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/gap_decorations_painter.cc` | 345 | 326 | 0 | 19 | 0 |
| GapDecorationsPainterContract | `layout_algo_grid/layout_algo_flex/layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/gap_decorations_painter.h` | 23 | 6 | 0 | 17 | 0 |
| OutlinePainter | `layout_fragment/prepaint/paint_record/raster` | `third_party/blink/renderer/core/paint/outline_painter.cc` | 880 | 158 | 668 | 54 | 0 |
| OutlinePainterContract | `layout_fragment/prepaint/paint_record/raster` | `third_party/blink/renderer/core/paint/outline_painter.h` | 48 | 10 | 6 | 32 | 0 |
| ThemePainter | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/theme_painter.cc` | 350 | 65 | 223 | 62 | 0 |
| ThemePainterContract | `layout_fragment/paint_record` | `third_party/blink/renderer/core/paint/theme_painter.h` | 124 | 84 | 17 | 23 | 0 |
| ThemePainterDefault | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/theme_painter_default.cc` | 729 | 44 | 650 | 35 | 0 |
| ThemePainterDefaultContract | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/theme_painter_default.h` | 82 | 58 | 8 | 16 | 0 |
| ObjectPaintProperties | `paint_model/prepaint` | `third_party/blink/renderer/core/paint/object_paint_properties.h` | 369 | 192 | 68 | 109 | 0 |
| TablePainters | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/table_painters.cc` | 751 | 661 | 60 | 30 | 0 |
| TablePaintersContract | `layout_fragment/paint_record/raster` | `third_party/blink/renderer/core/paint/table_painters.h` | 87 | 73 | 0 | 14 | 0 |
| **Selected backbone total** | | | **128735** | **42243** | **15531** | **10251** | **60710** |

Only manually verified source ranges count as translated. Existing Rust code without a verified source-range entry remains in Remaining until audited. This table is the detailed backbone subset; the first table is the authoritative full-directory total.
