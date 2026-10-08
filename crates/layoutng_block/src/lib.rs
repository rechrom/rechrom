#![allow(non_snake_case)]

// The block package sees both original C++ package namespaces through the
// one compilation unit that resolves their mutually recursive types.
extern crate layoutng_assembly as layoutng;
extern crate layoutng_assembly as layoutng_fragment_tree;
#[cfg(feature = "inline_extension")]
extern crate layoutng_assembly as layoutng_inline;

pub mod assembly;
pub mod block_child_iterator;
#[cfg(feature = "inline_extension")]
pub mod block_inline_layout;
pub mod block_layout_algorithm;
pub mod block_layout_utils;
pub mod unpositioned_list_marker;

#[cfg(feature = "block_only_host")]
mod excluded_block_boundaries;

#[cfg(all(test, feature = "constructed_fixture"))]
mod block_fixture_test {
    use super::assembly::InstallBlockAlgorithm;
    use layoutng::internal::boundary::assembly::InstallLayoutBoundary;
    use layoutng::internal::layout_input::{
        ComputedStyle, ConstraintSpace, Edges, FontFace, Offset, Size,
    };
    use layoutng::internal::layout_input_types::Color;
    use layoutng::internal::layout_object_builder::LayoutObjectTree;
    use layoutng::layout_assembly::LayoutAssembly;
    use layoutng::layout_engine::{LayoutEngine, LayoutMutation};

    #[test]
    fn constructed_block_tree_matches_reference_fragments() {
        let mut assembly = LayoutAssembly::default();
        InstallBlockAlgorithm(&mut assembly);
        InstallLayoutBoundary(&mut assembly);

        let mut space = ConstraintSpace::default();
        space.available_size = Size {
            width: 400.0,
            height: 320.0,
        };
        space.fonts.push(FontFace {
            family: "Roboto".into(),
            bytes: include_bytes!(
                "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
            )
            .as_slice()
            .into(),
            ..FontFace::default()
        });
        let mut tree = LayoutObjectTree::new_with_factories(&space, &assembly.objects);
        let html = tree.CreateRootDefault(3, &ComputedStyle::default()) as *mut _;
        let mut body_style = ComputedStyle::default();
        body_style.paint.background_color = Color {
            red: 245.0 / 255.0,
            green: 247.0 / 255.0,
            blue: 250.0 / 255.0,
            alpha: 1.0,
        };
        let body = tree.AddBoxDefault(unsafe { &mut *html }, 7, &body_style) as *mut _;
        let mut frame_style = ComputedStyle {
            width: Some(320.0),
            height: Some(240.0),
            margin: Edges {
                top: 24.0,
                right: 24.0,
                bottom: 24.0,
                left: 24.0,
            },
            padding: Edges {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0,
            },
            border: Edges {
                top: 4.0,
                right: 4.0,
                bottom: 4.0,
                left: 4.0,
            },
            ..ComputedStyle::default()
        };
        frame_style.paint.background_color = Color {
            red: 227.0 / 255.0,
            green: 235.0 / 255.0,
            blue: 245.0 / 255.0,
            alpha: 1.0,
        };
        frame_style.paint.border_colors = [Color {
            red: 38.0 / 255.0,
            green: 56.0 / 255.0,
            blue: 77.0 / 255.0,
            alpha: 1.0,
        }; 4];
        let frame = tree.AddBoxDefault(unsafe { &mut *body }, 8, &frame_style) as *mut _;
        let mut top_style = ComputedStyle {
            width: Some(140.0),
            height: Some(56.0),
            margin: Edges {
                bottom: 12.0,
                ..Edges::default()
            },
            ..ComputedStyle::default()
        };
        top_style.paint.background_color = Color {
            red: 48.0 / 255.0,
            green: 107.0 / 255.0,
            blue: 179.0 / 255.0,
            alpha: 1.0,
        };
        tree.AddBoxDefault(unsafe { &mut *frame }, 9, &top_style);
        let mut bottom_style = ComputedStyle {
            width: Some(220.0),
            height: Some(80.0),
            margin: Edges {
                left: 24.0,
                ..Edges::default()
            },
            ..ComputedStyle::default()
        };
        bottom_style.paint.background_color = Color {
            red: 233.0 / 255.0,
            green: 161.0 / 255.0,
            blue: 68.0 / 255.0,
            alpha: 1.0,
        };
        tree.AddBoxDefault(unsafe { &mut *frame }, 10, &bottom_style);

        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(tree));
        engine.Layout();
        let result = engine
            .TakeLayoutResult()
            .expect("successful layout publishes fragments");
        assert_eq!(result.node_id, 3);
        assert_eq!(result.offset, Offset { x: 0.0, y: 0.0 });
        assert_eq!(
            result.size,
            Size {
                width: 400.0,
                height: 320.0
            }
        );
        let body = &result.children[0];
        assert_eq!(
            (body.node_id, body.offset, body.size),
            (
                7,
                Offset { x: 0.0, y: 24.0 },
                Size {
                    width: 400.0,
                    height: 280.0
                }
            )
        );
        let frame = &body.children[0];
        assert_eq!(
            (frame.node_id, frame.offset, frame.size),
            (
                8,
                Offset { x: 24.0, y: 0.0 },
                Size {
                    width: 360.0,
                    height: 280.0
                }
            )
        );
        assert_eq!(frame.children.len(), 2);
        assert_eq!(
            (
                frame.children[0].node_id,
                frame.children[0].offset,
                frame.children[0].size
            ),
            (
                9,
                Offset { x: 20.0, y: 20.0 },
                Size {
                    width: 140.0,
                    height: 56.0
                }
            )
        );
        assert_eq!(
            (
                frame.children[1].node_id,
                frame.children[1].offset,
                frame.children[1].size
            ),
            (
                10,
                Offset { x: 44.0, y: 88.0 },
                Size {
                    width: 220.0,
                    height: 80.0
                }
            )
        );

        let display_items = paint::Paint(&result);
        let rgba = raster::RasterizeDisplayItemList(&display_items, 400, 320);
        let pixel =
            |x: usize, y: usize| -> &[u8] { &rgba[(y * 400 + x) * 4..(y * 400 + x) * 4 + 4] };
        assert_eq!(pixel(0, 0), [255, 255, 255, 255]);
        assert_eq!(pixel(10, 30), [245, 247, 250, 255]);
        assert_eq!(pixel(24, 24), [38, 56, 77, 255]);
        assert_eq!(pixel(28, 28), [227, 235, 245, 255]);
        assert_eq!(pixel(44, 44), [48, 107, 179, 255]);
        if let Ok(path) = std::env::var("LAYOUTNG_BLOCK_PNG") {
            raster::WriteDisplayItemListPng(&display_items, 400, 320, std::path::Path::new(&path))
                .unwrap();
        }
    }
}
