use super::*;
use crate::{
    image_decoder::{ImageDecodeInput, ImageDecoder},
    skia_image_decoder::SkiaImageDecoder,
};
use crabby_avif::{
    image::{Plane, ALL_PLANES},
    reformat::rgb,
    utils::pixels::Pixels,
};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/avif-source")
            .join(name),
    )
    .unwrap()
}
fn raw_decoder(name: &str, threads: u32) -> Decoder {
    let mut d = Decoder::default();
    d.settings.max_threads = threads;
    d.settings.codec_choice = CodecChoice::Rav1d;
    d.set_io_vec(fixture(name));
    d.parse().unwrap();
    d
}
fn planes(image: &Image) -> Vec<u16> {
    let mut result = Vec::new();
    for plane in ALL_PLANES {
        if !image.has_plane(plane) {
            continue;
        }
        for y in 0..image.height(plane) as u32 {
            if image.depth == 8 {
                result.extend(
                    image.row(plane, y).unwrap()[..image.width(plane)]
                        .iter()
                        .map(|x| *x as u16),
                );
            } else {
                result.extend_from_slice(&image.row16(plane, y).unwrap()[..image.width(plane)]);
            }
        }
    }
    result
}
fn rgba(image: &Image) -> Vec<u8> {
    let mut rgb = rgb::Image::create_from_yuv(image);
    rgb.depth = 8;
    rgb.allocate().unwrap();
    rgb.convert_from_yuv(image).unwrap();
    (0..image.height)
        .flat_map(|y| rgb.row(y).unwrap()[..image.width as usize * 4].to_vec())
        .collect()
}

#[test]
fn production_dispatch_uses_signature_and_preserves_typed_unsupported() {
    let bytes = fixture("white_2x2.avif");
    for mime_type in ["image/avif", "image/png", ""] {
        let frame = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: &bytes,
                mime_type,
            })
            .unwrap();
        assert_eq!((frame.width, frame.height), (2, 2));
        assert_eq!(frame.rgba8, decode(&bytes).unwrap().rgba8);
    }
    let bytes = fixture("paris_10bpc.avif");
    let error = SkiaImageDecoder
        .Decode(&ImageDecodeInput {
            bytes: &bytes,
            mime_type: "image/avif",
        })
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
}

#[test]
fn blink_configuration_and_all_orientation_combinations() {
    let d = new_decoder(&fixture("white_2x2.avif"), Limits::default()).unwrap();
    assert_eq!(d.settings.max_threads, 2);
    assert_eq!(d.settings.codec_choice, CodecChoice::Rav1d);
    assert_eq!(d.settings.source, Source::PrimaryItem);
    assert!(d.settings.ignore_exif && d.settings.ignore_xmp);
    assert!(!d.settings.allow_progressive && !d.settings.allow_incremental);
    for (axis, expected) in [
        (None, [1, 8, 3, 6]),
        (Some(0), [4, 5, 2, 7]),
        (Some(1), [2, 7, 4, 5]),
    ] {
        for angle in 0..4 {
            assert_eq!(
                orientation(Some(angle), axis).unwrap().exif(),
                expected[angle as usize]
            );
        }
    }
    assert!(orientation(Some(4), None).is_err());
    assert!(orientation(None, Some(2)).is_err());
}

#[test]
fn official_lossless_identity_pixels_equal_png_in_every_channel() {
    // Upstream tests/lossless.rs identifies this exact AVIF/PNG pair as lossless.
    // This validates AV1 reconstruction and identity conversion, not libyuv parity.
    let mut d = raw_decoder("paris_identity.avif", 2);
    d.next_image().unwrap();
    let i = d.image().unwrap();
    let expected = image::load_from_memory_with_format(
        &fixture("paris_icc_exif_xmp.png"),
        image::ImageFormat::Png,
    )
    .unwrap()
    .into_rgba8();
    assert_eq!((i.width, i.height), expected.dimensions());
    let expected = expected.into_raw();
    assert_eq!(rgba(i), expected);
    assert_eq!(super::super::avif_rgb8::convert(i).unwrap(), expected);
}

#[test]
fn opaque_odd_edges_and_output_lifetime() {
    for (name, size) in [("white_1x1.avif", (1, 1)), ("white_2x2.avif", (2, 2))] {
        let bytes = fixture(name);
        let result = decode(&bytes).unwrap();
        assert_eq!((result.width, result.height), size);
        assert_eq!(result.rgba8.len(), size.0 as usize * size.1 as usize * 4);
        assert!(result.rgba8.chunks_exact(4).all(|x| x[3] == 255));
        let saved = result.clone();
        drop(bytes);
        for _ in 0..8 {
            let _ = decode(&fixture("alpha.avif"));
        }
        assert_eq!(result, saved); // Owns bytes after codec, input and other decodes die.
    }
}

#[test]
fn straight_alpha_matches_retained_alpha_plane_without_double_multiplication() {
    let mut d = raw_decoder("alpha.avif", 2);
    d.next_image().unwrap();
    let image = d.image().unwrap();
    assert!(image.alpha_present && !image.alpha_premultiplied);
    let out = super::super::avif_rgb8::convert(image).unwrap();
    assert_eq!(decode(&fixture("alpha.avif")).unwrap().rgba8, out);
    let alpha: Vec<u8> = (0..image.height)
        .flat_map(|y| image.row(Plane::A, y).unwrap()[..image.width as usize].to_vec())
        .collect();
    assert_eq!(out.chunks_exact(4).map(|p| p[3]).collect::<Vec<_>>(), alpha);
    assert!(alpha.iter().any(|a| *a > 0 && *a < 255));
    let mut multiplied = rgb::Image::create_from_yuv(image);
    multiplied.premultiply_alpha = true;
    multiplied.allocate().unwrap();
    multiplied.convert_from_yuv(image).unwrap();
    let mut different_color = false;
    for y in 0..image.height {
        for x in 0..image.width as usize {
            let i = (y as usize * image.width as usize + x) * 4;
            if out[i + 3] > 0
                && out[i + 3] < 255
                && out[i..i + 3] != multiplied.row(y).unwrap()[x * 4..x * 4 + 3]
            {
                different_color = true;
            }
        }
    }
    assert!(different_color);
}

#[test]
fn raw_8_10_12_bit_planes_and_two_thread_results_are_identical() {
    for (name, depth) in [
        ("alpha.avif", 8),
        ("paris_10bpc.avif", 10),
        ("colors-animated-12bpc-keyframes-0-2-3.avif", 12),
    ] {
        let mut one = raw_decoder(name, 1);
        let mut two = raw_decoder(name, 2);
        one.next_image().unwrap();
        two.next_image().unwrap();
        let a = one.image().unwrap();
        let b = two.image().unwrap();
        assert_eq!(a.depth, depth);
        assert_eq!(b.depth, depth);
        assert_eq!((a.width, a.height), (b.width, b.height));
        assert!(a.width > 0 && a.height > 0);
        let values = planes(a);
        assert!(!values.is_empty());
        assert!(values.iter().all(|v| *v < (1u16 << depth)));
        assert_eq!(values, planes(b));
        if depth > 8 {
            assert!(values.iter().any(|v| *v > 255));
        }
    }
}

#[test]
fn grid_assembly_owns_pixels_and_matches_single_thread() {
    let mut one = raw_decoder("sofa_grid1x5_420.avif", 1);
    let mut two = raw_decoder("sofa_grid1x5_420.avif", 2);
    one.next_image().unwrap();
    two.next_image().unwrap();
    let a = one.image().unwrap();
    let b = two.image().unwrap();
    assert!(matches!(&b.planes[0], Some(Pixels::Buffer(_))));
    assert_eq!(planes(a), planes(b));
    assert!(raw_decoder_parse("sofa_grid1x5_420_dimg_repeat.avif").is_err());
}
fn raw_decoder_parse(name: &str) -> Result<(), AvifError> {
    let mut d = Decoder::default();
    d.set_io_vec(fixture(name));
    d.parse()
}

#[test]
fn retained_pictures_seeking_final_frame_and_alpha_lifetime() {
    for name in [
        "colors-animated-12bpc-keyframes-0-2-3.avif",
        "colors-animated-8bpc-alpha-exif-xmp.avif",
    ] {
        let mut sequential = raw_decoder(name, 2);
        let mut seek = raw_decoder(name, 2);
        let last = sequential.image_count() - 1;
        for _ in 0..=last {
            sequential.next_image().unwrap();
        }
        let expected = planes(sequential.image().unwrap());
        seek.nth_image(last).unwrap(); // Exercises the upstream get_last_image path.
        assert_eq!(planes(seek.image().unwrap()), expected);
        seek.nth_image(0).unwrap();
        seek.nth_image(last).unwrap();
        assert_eq!(planes(seek.image().unwrap()), expected);
    }
}

#[test]
fn orientation_is_metadata_without_rotating_storage() {
    for (name, expected) in [
        ("white_iden_irot.avif", 8),
        ("white_iden_chain_with_imir_primary.avif", 4),
    ] {
        let out = decode(&fixture(name)).unwrap();
        assert_eq!(out.orientation.exif(), expected);
        assert_eq!((out.width, out.height), (2, 2));
        assert_eq!(out.rgba8, decode(&fixture("white_2x2.avif")).unwrap().rgba8);
    }
}

#[test]
fn rejects_deferred_semantics_before_publishing_pixels() {
    for name in [
        "paris_10bpc.avif",
        "paris_identity.avif",
        "alpha_premultiplied.avif",
        "progressive/tiger_3layer_1res.avif",
        "seine_hdr_srgb.avif",
        "colors-animated-8bpc-alpha-exif-xmp.avif",
    ] {
        let err = decode(&fixture(name)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported, "{name}: {err}");
    }
}

#[test]
fn rejects_empty_truncated_malformed_and_color_alpha_mismatch() {
    let bytes = fixture("white_2x2.avif");
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end]).is_err(), "accepted truncation {end}");
    }
    for bytes in [
        vec![0; 128],
        b"not an image".to_vec(),
        fixture("invalid_color10x10_alpha5x5.avif"),
    ] {
        assert!(decode(&bytes).is_err());
    }
    // This upstream fixture is intentionally invalid, not a valid unsupported
    // crop: its required transform properties are not marked essential.
    let invalid_transform = decode(&fixture("clap_irot_imir_non_essential.avif")).unwrap_err();
    assert_eq!(invalid_transform.kind(), io::ErrorKind::InvalidData);
    assert!(invalid_transform
        .to_string()
        .contains("required essential property"));
    // A failure does not poison the next independent decode or its retained planes.
    assert!(decode(&fixture("white_2x2.avif")).is_ok());
}

#[test]
fn enforces_encoded_area_and_dimension_budgets() {
    let bytes = fixture("white_2x2.avif");
    for limits in [
        Limits {
            input_bytes: bytes.len() - 1,
            ..Limits::default()
        },
        Limits {
            pixels: NonZeroU32::new(3).unwrap(),
            ..Limits::default()
        },
        Limits {
            dimension: NonZeroU32::new(1).unwrap(),
            ..Limits::default()
        },
    ] {
        assert!(decode_with_limits(&bytes, limits).is_err());
    }
    assert!(decode_with_limits(
        &bytes,
        Limits {
            pixels: NonZeroU32::new(4).unwrap(),
            dimension: NonZeroU32::new(2).unwrap(),
            ..Limits::default()
        }
    )
    .is_ok());
    // Lower the codec budget after parse: this independently exercises rav1d's
    // frame_size_limit, rather than only the container's ispe validation.
    let mut d = raw_decoder("white_2x2.avif", 2);
    d.settings.image_size_limit = NonZeroU32::new(3);
    assert!(d.next_image().is_err());
}

#[test]
fn optional_real_asset_backend_probe() {
    let Ok(path) = std::env::var("RECHROM_AVIF_REAL_ASSET") else {
        return;
    };
    let bytes = std::fs::read(path).unwrap();
    let mut d = Decoder::default();
    d.settings.max_threads = 2;
    d.settings.codec_choice = CodecChoice::Rav1d;
    d.set_io_vec(bytes.clone());
    d.parse().unwrap();
    d.next_image().unwrap();
    let i = d.image().unwrap();
    let output = rgba(i);
    println!(
        "real_asset raw={}x{} depth={} alpha={} format={:?} CICP={:?}/{:?}/{:?} rgba_bytes={}",
        i.width,
        i.height,
        i.depth,
        i.alpha_present,
        i.yuv_format,
        i.color_primaries,
        i.transfer_characteristics,
        i.matrix_coefficients,
        output.len()
    );
    assert!(i.width > 0 && i.height > 0);
    println!(
        "real_asset candidate={:?}",
        decode(&bytes).map(|x| (x.width, x.height, x.rgba8.len()))
    );
}

#[test]
fn malformed_itut_t35_obu_returns_error_without_process_abort() {
    use rav1d::include::dav1d::{data::Dav1dData, dav1d::Dav1dSettings, picture::Dav1dPicture};
    use rav1d::src::lib::*;
    use std::{mem::MaybeUninit, ptr::NonNull};
    // Minimal official upstream issue #1476 reproduction. Every pointer below
    // refers to live initialized storage; the decoder and data are unref'd once.
    unsafe {
        let mut s = MaybeUninit::<Dav1dSettings>::uninit();
        dav1d_default_settings(NonNull::new(s.as_mut_ptr()).unwrap());
        let mut s = s.assume_init();
        s.n_threads = 2;
        s.max_frame_delay = 1;
        let mut ctx = None;
        assert_eq!(
            dav1d_open(Some(NonNull::from(&mut ctx)), Some(NonNull::from(&mut s))).0,
            0
        );
        let bytes = [47u8, 47, 5, 4, 128, 0, 0, 0];
        let mut data = Dav1dData::default();
        let dest = dav1d_data_create(Some(NonNull::from(&mut data)), bytes.len());
        assert!(!dest.is_null());
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
        let send = dav1d_send_data(ctx, Some(NonNull::from(&mut data))).0;
        let mut pic = Dav1dPicture::default();
        let get = dav1d_get_picture(ctx, Some(NonNull::from(&mut pic))).0;
        assert!(send < 0 || get < 0);
        dav1d_picture_unref(Some(NonNull::from(&mut pic)));
        dav1d_data_unref(Some(NonNull::from(&mut data)));
        dav1d_close(Some(NonNull::from(&mut ctx)));
        assert!(ctx.is_none());
    }
}

#[test]
fn progressive_same_resolution_layers_match_complete_decode() {
    // Upstream decoder.rs uses three 1216x832 layers. Resolution-changing
    // variants need libyuv scaling and remain outside this integration slice.
    let name = "progressive/tiger_3layer_1res.avif";
    let mut complete = raw_decoder(name, 2);
    complete.next_image().unwrap();
    let expected = planes(complete.image().unwrap());
    let mut layers = Decoder::default();
    layers.settings.max_threads = 2;
    layers.settings.codec_choice = CodecChoice::Rav1d;
    layers.settings.allow_progressive = true;
    layers.set_io_vec(fixture(name));
    layers.parse().unwrap();
    assert_eq!(layers.image_count(), 3);
    for _ in 0..3 {
        layers.next_image().unwrap();
        assert_eq!(
            (
                layers.image().unwrap().width,
                layers.image().unwrap().height
            ),
            (1216, 832)
        );
    }
    assert_eq!(planes(layers.image().unwrap()), expected);
}

#[test]
fn valid_crop_and_color_metadata_are_explicit_admission_gates() {
    use crabby_avif::utils::clap::{CleanAperture, CropRect};
    let mut image = Image {
        width: 4,
        height: 4,
        depth: 8,
        yuv_format: PixelFormat::Yuv444,
        ..Image::default()
    };
    assert!(admit_still_image(&image, false).is_ok());
    image.clap = Some(
        CleanAperture::create_from(
            &CropRect {
                x: 0,
                y: 0,
                width: 2,
                height: 2,
            },
            4,
            4,
            PixelFormat::Yuv444,
        )
        .unwrap(),
    );
    assert_eq!(
        admit_still_image(&image, false).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    image.clap = None;
    for transfer in [
        TransferCharacteristics::Pq,
        TransferCharacteristics::Hlg,
        TransferCharacteristics::Linear,
    ] {
        image.transfer_characteristics = transfer;
        assert_eq!(
            admit_still_image(&image, false).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
    image.transfer_characteristics = TransferCharacteristics::Unspecified;
    assert_eq!(
        admit_still_image(&image, true).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    image.icc = vec![1, 2, 3];
    assert_eq!(
        admit_still_image(&image, false).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}

// Independent expected bytes come from the exact pinned libyuv C and SIMD
// implementations, with their own dispatch/filter/constants (not this port).
fn check_oracle(bytes: &[u8]) -> usize {
    use crabby_avif::{image::YuvRange, Category};
    let mut cursor = 0;
    let mut cases = 0;
    while cursor < bytes.len() {
        let mut m = [0u32; 6];
        for value in &mut m {
            *value = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;
        }
        let mut image = Image {
            width: m[0],
            height: m[1],
            depth: 8,
            yuv_format: match m[2] {
                400 => PixelFormat::Yuv400,
                420 => PixelFormat::Yuv420,
                422 => PixelFormat::Yuv422,
                444 => PixelFormat::Yuv444,
                _ => panic!(),
            },
            matrix_coefficients: MatrixCoefficients::from(m[3] as u16),
            yuv_range: if m[4] == 1 {
                YuvRange::Full
            } else {
                YuvRange::Limited
            },
            alpha_present: m[5] == 1,
            ..Image::default()
        };
        image.allocate_planes(Category::Color).unwrap();
        if image.alpha_present {
            image.allocate_planes(Category::Alpha).unwrap();
        }
        for plane in ALL_PLANES {
            if !image.has_plane(plane) {
                continue;
            }
            let width = image.width(plane);
            for y in 0..image.height(plane) as u32 {
                image.row_mut(plane, y).unwrap()[..width]
                    .copy_from_slice(&bytes[cursor..cursor + width]);
                cursor += width;
            }
        }
        let len = image.width as usize * image.height as usize * 4;
        let expected = &bytes[cursor..cursor + len];
        let actual = super::super::avif_rgb8::convert(&image).unwrap();
        let differences = actual
            .chunks_exact(4)
            .zip(expected.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(differences, 0, "libyuv case {cases}: {m:?}");
        cursor += len;
        cases += 1;
    }
    cases
}

#[test]
fn pinned_libyuv_scalar_and_simd_oracles_match_all_admitted_yuv_cases() {
    assert_eq!(check_oracle(&fixture("libyuv-rgb8-oracle.bin")), 832);
}

#[test]
fn official_alpha_avif_matches_pinned_libyuv_not_generic_float_conversion() {
    let oracle = fixture("alpha-libyuv.bin");
    assert_eq!(check_oracle(&oracle), 1);
    let decoded = decode(&fixture("alpha.avif")).unwrap();
    assert_eq!(decoded.rgba8, oracle[oracle.len() - decoded.rgba8.len()..]);
}

#[test]
fn optional_real_asset_matches_pinned_libyuv() {
    let (Ok(asset), Ok(oracle)) = (
        std::env::var("RECHROM_AVIF_REAL_ASSET"),
        std::env::var("RECHROM_AVIF_REAL_ORACLE"),
    ) else {
        return;
    };
    let oracle = std::fs::read(oracle).unwrap();
    let decoded = decode(&std::fs::read(asset).unwrap()).unwrap();
    assert_eq!(decoded.rgba8, oracle[oracle.len() - decoded.rgba8.len()..]);
    println!(
        "real asset pinned libyuv: {} pixels, zero differences",
        decoded.rgba8.len() / 4
    );
}

#[test]
fn chromium_sdr_transfer_aliases_and_identity_gates_are_explicit() {
    let mut image = Image {
        width: 4,
        height: 4,
        depth: 8,
        yuv_format: PixelFormat::Yuv444,
        ..Image::default()
    };
    for transfer in [
        TransferCharacteristics::Unspecified,
        TransferCharacteristics::Srgb,
        TransferCharacteristics::Bt709,
        TransferCharacteristics::Bt601,
        TransferCharacteristics::Bt2020_10bit,
        TransferCharacteristics::Bt2020_12bit,
    ] {
        image.transfer_characteristics = transfer;
        admit_still_image(&image, false).unwrap();
    }
    image.matrix_coefficients = MatrixCoefficients::Identity;
    image.yuv_format = PixelFormat::Yuv420;
    assert_eq!(
        admit_still_image(&image, false).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}
