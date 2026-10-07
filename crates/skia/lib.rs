//! Rust CPU Skia subset. Official ownership paths and explicit Rust adapters.
//! Algorithms include directly migrated tiny-skia sources; see translation_map.json.
#![allow(non_snake_case)]
extern crate alloc;
pub mod path;
pub mod raster;
pub use raster::*;
pub mod cpu;
pub use compat::surface::RasterSurface;
pub mod include {
    pub mod core {
        pub mod SkBlendMode;
        pub mod SkCanvas;
        pub mod SkColor;
        pub mod SkColorType;
        pub mod SkPaint;
        pub mod SkPathTypes;
        pub mod SkPixmap;
        pub mod SkPoint;
        pub mod SkRect;
        pub mod SkScalar;
        pub mod SkShader;
        pub mod SkSize;
        pub mod SkTileMode;
    }
    pub mod private {
        pub mod SkFixed;
        pub mod SkFloatingPoint;
        pub mod SkMath;
    }
}
pub mod src {
    pub mod core {
        pub mod SkAlphaRuns;
        pub mod SkAnalyticEdge;
        pub mod SkBitmap;
        pub mod SkBlendMode;
        pub mod SkBlitRow_D32;
        pub mod SkBlitter;
        pub mod SkBlitter_ARGB32;
        pub mod SkBlurEngine;
        pub mod SkCanvas;
        pub mod SkColor;
        pub(crate) mod SkColorGlyphClip;
        pub mod SkContourMeasure;
        pub mod SkConvertPixels;
        pub mod SkCubicClipper;
        pub mod SkCubics;
        pub mod SkEdge;
        pub mod SkEdgeBuilder;
        pub mod SkEdgeClipper;
        pub mod SkFDot6;
        pub mod SkGeometry;
        pub mod SkLineClipper;
        pub mod SkMask;
        pub mod SkMaskBlurFilter;
        pub mod SkMaskBuilder;
        #[cfg(target_os = "macos")]
        pub mod SkMaskGamma;
        pub mod SkMatrix;
        pub mod SkPath;
        pub mod SkPathBuilder;
        pub mod SkPixelRef;
        pub mod SkPixmap;
        pub mod SkQuads;
        pub mod SkRasterClip;
        pub mod SkRasterPipeline;
        pub mod SkRasterPipelineBlitter;
        pub mod SkRasterPipelineOpContexts;
        pub mod SkRasterPipelineOpList;
        pub mod SkScalerContext;
        pub mod SkScan;
        pub mod SkScan_AAAPath;
        pub mod SkScan_AntiPath;
        pub mod SkScan_Antihair;
        pub mod SkScan_Hairline;
        pub mod SkScan_Path;
        pub mod SkStrike;
        pub mod SkStrikeCache;
        pub mod SkStroke;
        pub mod SkStrokerPriv;
        pub mod SkTSort;
        pub mod SkVx;
    }
    pub mod opts {
        pub mod SkBlitMask_opts;
        pub mod SkBlitRow_opts;
        pub mod SkMemset_opts;
        pub mod SkRasterPipeline_opts;
    }
    pub mod pathops {
        pub mod SkDCubicLineIntersection;
        pub mod SkPathOpsCubic;
        pub mod SkPathOpsPoint;
        pub mod SkPathOpsQuad;
        pub mod SkPathOpsTypes;
    }
    pub mod ports {
        #[cfg(target_os = "macos")]
        pub mod SkScalerContext_mac_ct;
    }
    pub mod shaders {
        pub mod SkImageShader;
        pub mod gradients {
            pub mod SkConicalGradient;
            pub mod SkGradientBaseShader;
            pub mod SkLinearGradient;
            pub mod SkRadialGradient;
            pub mod SkSweepGradient;
        }
    }
    pub mod utils {
        pub mod SkDashPath;
    }
}
pub mod compat {
    pub mod analytic_masks;
    pub mod blend_mode;
    pub mod border;
    pub mod canvas;
    pub mod color;
    pub mod commands;
    pub mod f16;
    pub mod geometry;
    pub(crate) mod glyph_paths;
    pub mod glyph_position;
    pub(crate) mod glyph_rasters;
    pub mod hairline;
    pub mod image_sampling;
    pub mod layer_filters;
    pub mod mask_blitter;
    pub mod pixel_view;
    pub mod pixmap_draw;
    pub mod png;
    pub mod raster_bounds;
    pub mod surface;
}
pub mod extensions {
    pub mod RasterPipelineReplay;
    pub mod ShaderCompatibility;
    pub mod hvgl;
}

pub use src::core::SkColorGlyphClip::{
    RasterClipProductCache, RasterClipProductCacheAccounting, RasterClipProductCacheStats,
};
