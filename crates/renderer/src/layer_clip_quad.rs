//! SoftwareRenderer's direct rounded clip for a single disjoint tile layer.
//! The caller proves the clip is a leaf scope and its owned quad rectangles
//! do not overlap. The real synthesized A8 coverage remains the clip shader;
//! source coverage is applied in the original N32 clip/blend order.
use super::*;

fn sample_fractional_row<const OPAQUE: bool, const COMPOSITOR: bool>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    fractional_row_signed::<OPAQUE, COMPOSITOR>(output, pixels, adjacent, sy, sx, sampler, format)
}

fn compose_covered_run(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: Option<&PreparedFractionalSampler>,
    solid: Option<u32>,
    alpha: u8,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    // SkAAClipBlitter::blitH / mergeT (SkAAClip.cpp): zero runs do not draw,
    // FF runs use the original blitter, partial runs retain the N32 coverage
    // operation. A mixed edge row never sends its FF interior through scratch.
    if alpha == 0 {
        return None;
    }
    if alpha == 255 {
        if let Some(mut color) = solid {
            if format != skia::PixelFormat::Rgba8888 {
                color = (color & 0xff00_ff00) | ((color & 0xff) << 16) | ((color >> 16) & 0xff);
            }
            if format == skia::PixelFormat::Bgrx8888 {
                color &= 0x00ff_ffff;
            }
            output.fill(color);
            return Some((0, output.len()));
        }
        if let Some(sampler) = sampler {
            // The same sampler computes the same N32 source. SrcOver onto
            // zero in the old temporary surface was the identity operation.
            return sample_fractional_row::<false, true>(
                output, pixels, adjacent, sy, sx, sampler, format,
            );
        }
        let (sy, sx) = (sy as usize, sx as usize);
        let extent = pixels.row_support.rows[sy].extent;
        let first = sx.max(extent.0);
        let last = (sx + output.len()).min(extent.1);
        if first >= last {
            return None;
        }
        for &(left, right) in pixels.row_support.spans(sy) {
            let left = first.max(left);
            let right = last.min(right);
            if left >= right {
                continue;
            }
            let start = (sy * pixels.pixel_size.0 as usize + left) * 4;
            let input = &pixels.rgba[start..start + (right - left) * 4];
            let destination = &mut output[left - sx..right - sx];
            if pixels.white_backing
                || pixels.row_support.opaque
                || pixels.row_support.rows[sy].opaque
            {
                copy_opaque_rgba_row(destination, input, format);
            } else {
                // SAFETY: exclusive initialized complete N32 destination.
                let bytes = unsafe {
                    std::slice::from_raw_parts_mut(
                        destination.as_mut_ptr().cast::<u8>(),
                        std::mem::size_of_val(destination),
                    )
                };
                skia::blend_premultiplied_rgba_row(bytes, input, format);
            }
        }
        return Some((first - sx, last - sx));
    }
    // Match the existing N32 isolated-source boundary before clip coverage.
    // Sparse source skips must remain RGBA zero; never reuse previous rows.
    let mut sampled = [0u32; TILE_SIZE as usize + 1];
    let source = &mut sampled[..output.len()];
    let span = if let Some(color) = solid {
        source.fill(color);
        Some((0, source.len()))
    } else if let Some(sampler) = sampler {
        sample_fractional_row::<false, false>(
            source,
            pixels,
            adjacent,
            sy,
            sx,
            sampler,
            skia::PixelFormat::Rgba8888,
        )
    } else {
        let (sy, sx) = (sy as usize, sx as usize);
        let extent = pixels.row_support.rows[sy].extent;
        let first = sx.max(extent.0);
        let last = (sx + source.len()).min(extent.1);
        if first >= last {
            None
        } else {
            let start = sy * pixels.pixel_size.0 as usize * 4 + first * 4;
            let input = &pixels.rgba[start..start + (last - first) * 4];
            for (dst, bytes) in source[first - sx..last - sx]
                .iter_mut()
                .zip(input.chunks_exact(4))
            {
                *dst = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            }
            Some((first - sx, last - sx))
        }
    };
    let (first, last) = span?;
    for pixel in &mut source[first..last] {
        *pixel = scale_mask_word(*pixel, alpha);
    }
    blend_compositor_row(&mut output[first..last], &source[first..last], format);
    Some((first, last))
}

pub(super) fn compose_clipped_tile(
    target: &mut [u32],
    width: u32,
    height: u32,
    view: TargetView,
    format: skia::PixelFormat,
    tile: &TilePlacement,
    composition: LayerComposition,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    draw_visible: Option<SurfaceBounds>,
    coverage: &MaskCoverage,
) -> io::Result<Option<SurfaceBounds>> {
    if !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other("invalid clipped tile extent"));
    }
    if tile.pixel_size.0 > TILE_SIZE
        || coverage
            .stride
            .checked_mul(coverage.rows.len())
            .is_none_or(|len| len != coverage.pixels.len())
    {
        return Err(io::Error::other("invalid direct clip coverage extent"));
    }
    let Some((mut bounds, (x, y), phase)) =
        tile_composition_geometry(tile, composition, width, height)?
    else {
        return Ok(None);
    };
    let Some((left, top, right, bottom)) = coverage.bounds else {
        return Ok(None);
    };
    bounds.left = bounds.left.max(left).max(coverage.origin.0);
    bounds.right = bounds
        .right
        .min(right)
        .min(coverage.origin.0 + coverage.stride);
    bounds.top = bounds.top.max(top).max(coverage.origin.1);
    bounds.bottom = bounds
        .bottom
        .min(bottom)
        .min(coverage.origin.1 + coverage.rows.len());
    if let Some(visible) = draw_visible {
        bounds.left = bounds.left.max(visible.left);
        bounds.right = bounds.right.min(visible.right);
        bounds.top = bounds.top.max(visible.top);
        bounds.bottom = bounds.bottom.min(visible.bottom);
    }
    if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
        return Ok(None);
    }
    // SkAAClipBlitter::blitRect (SkAAClip.cpp:1783): an encoded clip that
    // quickContains the complete write rectangle delegates to the underlying
    // blitter. This queries the actual generation's A8 runs once, avoiding a
    // repeated dense full-coverage scan on every interior tile row.
    if coverage.run_clip.quick_contains(
        bounds.left - coverage.origin.0,
        bounds.top - coverage.origin.1,
        bounds.right - coverage.origin.0,
        bounds.bottom - coverage.origin.1,
    ) {
        return compose_tile(
            target,
            width,
            height,
            view,
            format,
            tile,
            composition,
            pixels,
            adjacent,
            Some(bounds),
        );
    }
    // Prepare from this actual ceil-owned tile placement, once for every row
    // and corner span. Nominal layer translation is not a phase substitute.
    let sampler = (phase != (0.0, 0.0)).then(|| PreparedFractionalSampler::new(phase));
    let solid = (bounds.left as i64 >= x && bounds.top as i64 >= y)
        .then(|| layer_solid::color_for_sampling(pixels, adjacent, phase))
        .flatten();
    let mut touched = None;
    for row in bounds.top..bounds.bottom {
        let mask_y = row - coverage.origin.1;
        let extent = coverage.rows[mask_y];
        let left = bounds.left.max(extent.0);
        let right = bounds.right.min(extent.1);
        if left >= right {
            continue;
        }
        let sy = row as i64 - y;
        for (offset, count, alpha) in
            coverage
                .run_clip
                .row_runs(mask_y, left - coverage.origin.0, right - coverage.origin.0)
        {
            if alpha == 0 {
                continue;
            }
            let run_left = left + offset;
            let output = &mut target[view.row_range(row, run_left, run_left + count)];
            let sx = run_left as i64 - x;
            let written = compose_covered_run(
                output,
                pixels,
                adjacent,
                sy,
                sx,
                sampler.as_ref(),
                solid,
                alpha,
                format,
            );
            if let Some((first, last)) = written {
                union_bounds(
                    &mut touched,
                    Some(SurfaceBounds {
                        left: run_left + first,
                        right: run_left + last,
                        top: row,
                        bottom: row + 1,
                    }),
                );
            }
        }
    }
    Ok(touched)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rounded_quad_matches_isolated_n32_source_and_real_a8_clip() {
        let (width, height) = (76u32, 44u32);
        let state = paint::paint_property_tree::PropertyTreeState::default();
        let mut clip = (*state.clip).clone();
        clip.parent = Some(state.clip.clone());
        clip.lifecycle = Default::default();
        clip.id = 1;
        clip.rect = Some(PaintRect {
            x: 2.375,
            y: 1.6875,
            width: 65.25,
            height: 33.5,
        });
        clip.radii = paint::border_shape_utils::UniformCornerRadii(9.75);
        let coverage = ClipMaskCache::default()
            .coverage(&[Arc::new(clip)], width, height, 1.0, None)
            .unwrap();
        assert!(coverage.pixels.iter().any(|&v| v > 0 && v < 255));
        let mut tiles = Vec::new();
        for index in 0..4usize {
            let (column, row) = (index % 2, index / 2);
            let rect = PaintRect {
                x: (column * 32) as f64,
                y: (row * 16) as f64,
                width: 32.0,
                height: 16.0,
            };
            let tile = TilePlacement {
                tile_id: TileId(index as u64 + 1),
                generation: 1,
                tile_index: (column as i32, row as i32),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (32, 16),
                ready: true,
            };
            let rgba: Vec<u8> = (0..16usize)
                .flat_map(|y| {
                    (0..32usize).flat_map(move |x| {
                        if index == 0 {
                            return [45, 83, 127, 255];
                        }
                        let alpha = if index == 3 {
                            255
                        } else if (8..20).contains(&x) || y == 6 {
                            0
                        } else {
                            ((x * 37 + y * 53 + index * 23) % 256) as u8
                        };
                        [alpha / 3, alpha / 2, alpha, alpha]
                    })
                })
                .collect();
            let pixels = CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (32, 16),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, 32, false),
                rgba,
                bgra: Vec::new(),
            };
            tiles.push((tile, pixels));
        }
        let adjacent = |index: usize| {
            [
                (index % 2 == 0).then(|| &tiles[index + 1].1),
                (index < 2).then(|| &tiles[index + 2].1),
                (index == 0).then(|| &tiles[3].1),
            ]
        };
        let source_view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let parent_view = TargetView {
            origin: (1, 1),
            stride: 80,
        };
        let center = SurfaceBounds {
            left: 20,
            top: 14,
            right: 50,
            bottom: 23,
        };
        assert!(coverage.run_clip.quick_contains(
            center.left - coverage.origin.0,
            center.top - coverage.origin.1,
            center.right - coverage.origin.0,
            center.bottom - coverage.origin.1
        ));
        // Four small semantic cases exercise integer/fractional tile ownership
        // and the final BGRX proof. The AA-edge rectangle exercises the covered
        // path; the proven interior rectangle exercises quickContains delegation.
        for visible in [
            Some(SurfaceBounds {
                left: 5,
                top: 4,
                right: 65,
                bottom: 34,
            }),
            Some(center),
        ] {
            for (translation, format) in [
                ((3.0, 2.0), skia::PixelFormat::Rgba8888),
                ((3.0, 2.0), skia::PixelFormat::Bgrx8888),
                ((3.625, 2.6875), skia::PixelFormat::Rgba8888),
                ((3.625, 2.6875), skia::PixelFormat::Bgrx8888),
            ] {
                let composition = LayerComposition {
                    translation,
                    clip: None,
                    content_bounds: None,
                    opaque_bounds: None,
                    white_backing: false,
                    grid_min: None,
                };
                let initial: Vec<u32> = (0..parent_view.stride * (height as usize - 1))
                    .map(|i| {
                        let alpha = ((i * 17 + 61) % 256) as u32;
                        (alpha / 4)
                            | ((alpha / 2) << 8)
                            | ((alpha / 3) << 16)
                            | if format == skia::PixelFormat::Bgrx8888 {
                                0
                            } else {
                                alpha << 24
                            }
                    })
                    .collect();
                let mut expected = initial.clone();
                let mut actual = initial;
                let mut source = vec![0u32; width as usize * height as usize];
                let mut touched = None;
                for (index, (tile, pixels)) in tiles.iter().enumerate() {
                    union_bounds(
                        &mut touched,
                        compose_tile(
                            &mut source,
                            width,
                            height,
                            source_view,
                            skia::PixelFormat::Rgba8888,
                            tile,
                            composition,
                            pixels,
                            adjacent(index),
                            visible,
                        )
                        .unwrap(),
                    );
                }
                composite_effect_surface(
                    &mut expected,
                    &source,
                    touched,
                    parent_view,
                    source_view,
                    format,
                    1.0,
                    Some(&coverage),
                );
                for (index, (tile, pixels)) in tiles.iter().enumerate() {
                    compose_clipped_tile(
                        &mut actual,
                        width,
                        height,
                        parent_view,
                        format,
                        tile,
                        composition,
                        pixels,
                        adjacent(index),
                        visible,
                        &coverage,
                    )
                    .unwrap();
                }
                assert_eq!(
                    actual, expected,
                    "translation={translation:?} format={format:?}"
                );
            }
        }
    }
}
