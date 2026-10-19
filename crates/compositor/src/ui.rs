//! Module assembly for source-relative Chromium translations.
//!
//! A same-stem Chromium `.h` and `.cc` pair maps to one Rust `.rs` file
//! under `ui/<directory>/<stem>.rs`.

pub mod gfx {
    pub mod animation {
        pub mod keyframe {
            pub mod keyframe_effect {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/ui/gfx/animation/keyframe/keyframe_effect.rs"
                ));
            }
            pub mod keyframe_model {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/ui/gfx/animation/keyframe/keyframe_model.rs"
                ));
            }
        }
    }
}
