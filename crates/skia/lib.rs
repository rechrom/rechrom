//! Source-aligned CPU Skia translation under construction.
//! Registered modules may still contain only partial translated source slices.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
// Native incomplete GrDirectContext declarations retain an unsized extern type.
// This source translation currently requires nightly for that language mapping.
#![feature(extern_types)]
#![feature(explicit_tail_calls)]
#![feature(repr_simd, core_intrinsics, simd_ffi, abi_vectorcall)]
#![feature(c_variadic)]
#![feature(const_eval_select)]
#![feature(generic_const_exprs)]
#![feature(arbitrary_self_types)]
// Actual native static thread-local guard/lock state; no thread container owner.
#![feature(thread_local)]
#![feature(const_trait_impl, const_cmp)]
#![feature(const_default)]
#![feature(inherent_associated_types)]
#![feature(specialization, const_ops)]
#![allow(incomplete_features)]
// Native LSX/LASX branches use the actual unstable stdarch intrinsics.
// LoongArch builds therefore require a nightly toolchain.
#![cfg_attr(target_arch = "loongarch64", feature(stdarch_loongarch))]

// Host-buffer boundary for the browser CPU replay migration. No legacy backend.
pub mod raster_target;

// Actual Chromium SK_REF_CNT_MIXIN_INCLUDE source translations. These are
// embedding definitions, separate from the pinned Skia source-line total.
pub mod skia {
    pub mod ext;
    pub mod config {
        pub mod sk_ref_cnt_ext_debug;
        pub mod sk_ref_cnt_ext_release;
    }
}

pub mod include {
    pub mod codec {
        pub mod SkCodecAnimation;
        pub mod SkEncodedImageFormat;
        pub mod SkEncodedOrigin;
        pub mod SkPngChunkReader;
        pub use crate::src::codec::SkCodec;
        pub use crate::src::codec::SkPixmapUtils;
    }
    pub mod effects {
        pub mod SkGradient;
        pub mod SkImageFilters;
        pub mod SkOverdrawColorFilter;
        pub mod SkPerlinNoiseShader;
        pub mod SkShaderMaskFilter;
    }
    pub mod sksl {
        pub mod SkSLDebugTrace;
        pub mod SkSLVersion;
    }
    pub mod utils {
        pub mod SkNoDrawCanvas;
        pub use crate::src::utils::SkCustomTypeface;
        pub use crate::src::utils::SkLogHandler;
    }
    pub mod core {
        pub mod SkCapabilities;
        pub mod SkOpenTypeSVGDecoder;
        pub mod SkTraceMemoryDump;
        pub use crate::src::core::SkBBHFactory;
        pub mod SkAlphaType;
        pub mod SkArc;
        pub mod SkBlender;
        pub mod SkBlurTypes;
        pub mod SkClipOp;
        pub mod SkColorType;
        pub mod SkFontArguments;
        pub mod SkFontMetrics;
        pub mod SkFontParameters;
        pub mod SkFontStyle;
        pub mod SkFontTypes;
        pub mod SkPathTypes;
        pub mod SkRecorder;
        pub mod SkRefCnt;
        pub mod SkSamplingOptions;
        pub mod SkSerialProcs;
        pub use crate::src::core::SkScalar;
        pub mod SkCanvasVirtualEnforcer;
        pub mod SkRasterHandleAllocator;
        pub mod SkSize;
        pub mod SkSurfaceProps;
        pub mod SkTextureCompressionType;
        pub mod SkTileMode;
        pub mod SkTypes;
    }
    pub mod private {
        pub mod SkTFitsIn;
        pub mod SkTo;
        pub use crate::src::codec::SkEncodedInfo;
        pub use crate::src::codec::SkGainmapInfo;
        pub use crate::src::codec::SkHdrMetadata;
        pub mod SkAssert;
        pub mod SkFeatures;
        pub mod SkLoadUserConfig;
        pub mod SkLog;
        pub mod SkLogPriority;
        pub mod SkMutex;
        pub mod SkOnce;
        pub mod SkWeakRefCnt;
        pub use crate::src::core::SkDebug;
        pub use crate::src::core::SkSemaphore;
        pub use crate::src::core::SkThreadID;
        pub mod chromium {
            pub mod SkCodecsICCProfileChromium;
            pub mod SkDiscardableMemory;
            pub mod Slug;
        }
        pub mod SkFixed;
        pub mod SkMath;
        pub mod SkPathRef;
        pub mod SkTArray;
        pub mod SkTPin;
        pub mod SkTemplates;
    }
}
pub mod modules {
    pub mod skcms {
        pub mod skcms;
        pub mod src {
            pub mod Transform_inl;
            pub mod skcms_Transform;
            pub mod skcms_TransformBaseline;
            pub mod skcms_TransformHsw;
            pub mod skcms_TransformSkx;
            pub mod skcms_internals;
            pub mod skcms_public;
        }
    }
}
pub mod src {
    pub mod codec {
        pub mod SkCodec;
        pub mod SkCodecColorProfile;
        #[cfg(SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST)]
        pub mod SkCodecColorProfileRust;
        pub mod SkCodecImageGenerator;
        pub mod SkCodecPriv;
        pub mod SkColorPalette;
        pub mod SkEncodedInfo;
        pub mod SkFrameHolder;
        pub mod SkGainmapInfo;
        pub mod SkHdrAgtm;
        pub mod SkHdrAgtmParse;
        pub mod SkHdrAgtmPriv;
        pub mod SkHdrMetadata;
        pub mod SkImageGenerator_FromEncoded;
        pub mod SkPixmapUtils;
        pub mod SkPixmapUtilsPriv;
        pub mod SkSampler;
    }
    pub mod ports {
        pub mod SkGlobalInitialization_default;
        pub mod SkLog_stdio;
        pub mod SkMemory_malloc;
        #[cfg(SK_BUILD_FOR_IOS)]
        pub mod SkOSFile_ios;
        #[cfg(unix)]
        pub mod SkOSFile_posix;
        #[cfg(unix)]
        pub mod SkOSFile_stdio;
        #[cfg(target_vendor = "apple")]
        pub mod SkScalerContext_mac_ct;
        #[cfg(target_vendor = "apple")]
        pub mod SkTypeface_mac_ct;
    }
    pub mod sksl {
        pub mod codegen {
            pub mod SkSLRasterPipelineBuilder;
            pub mod SkSLRasterPipelineCodeGenerator;
        }
        pub mod SkSLAnalysis;
        pub mod SkSLBuiltinTypes;
        pub mod SkSLCompiler;
        pub mod SkSLConstantFolder;
        pub mod SkSLContext;
        pub mod SkSLDefines;
        pub mod SkSLErrorReporter;
        pub mod SkSLGLSL;
        pub mod SkSLGraphiteModules;
        pub mod SkSLInliner;
        pub mod SkSLIntrinsicList;
        pub mod SkSLLexer;
        pub mod SkSLMangler;
        pub mod SkSLMemoryPool;
        pub mod SkSLModule;
        pub mod SkSLModuleDataDefault;
        pub mod SkSLModuleLoader;
        pub mod SkSLOperator;
        pub mod SkSLOutputStream;
        pub mod SkSLParser;
        pub mod SkSLPool;
        pub mod SkSLPosition;
        pub mod SkSLProgramKind;
        pub mod SkSLProgramSettings;
        pub mod SkSLSampleUsage;
        pub mod SkSLString;
        pub mod SkSLStringStream;
        pub mod SkSLUtil;
        pub mod transform {
            pub mod SkSLAddConstToVarModifiers;
            pub mod SkSLEliminateDeadFunctions;
            pub mod SkSLEliminateDeadGlobalVariables;
            pub mod SkSLEliminateDeadLocalVariables;
            pub mod SkSLEliminateUnreachableCode;
            pub mod SkSLFindAndDeclareBuiltinFunctions;
            pub mod SkSLFindAndDeclareBuiltinStructs;
            pub mod SkSLFindAndDeclareBuiltinVariables;
            pub mod SkSLRewriteIndexedSwizzle;

            pub mod SkSLEliminateEmptyStatements;
            pub mod SkSLEliminateUnnecessaryBraces;
            pub mod SkSLHoistSwitchVarDeclarationsAtTopLevel;
            pub mod SkSLProgramWriter;
            pub mod SkSLRenamePrivateSymbols;
            pub mod SkSLReplaceConstVarsWithLiterals;
            pub mod SkSLReplaceSplatCastsWithSwizzles;
        }
        pub mod analysis {
            pub mod SkSLCheckProgramStructure;
            pub mod SkSLCheckSymbolTableCorrectness;
            pub mod SkSLFinalizationChecks;
            pub mod SkSLGetLoopControlFlowInfo;
            pub mod SkSLGetReturnComplexity;
            pub mod SkSLIsDynamicallyUniformExpression;
            pub mod SkSLReturnsInputAlpha;
            pub mod SkSLSymbolTableStackBuilder;

            pub mod SkSLCanExitWithoutReturningValue;
            pub mod SkSLGetLoopUnrollInfo;
            pub mod SkSLHasSideEffects;
            pub mod SkSLIsConstantExpression;
            pub mod SkSLIsSameExpressionTree;
            pub mod SkSLIsTrivialExpression;
            pub mod SkSLNoOpErrorReporter;
            pub mod SkSLProgramUsage;
            pub mod SkSLProgramVisitor;
            pub mod SkSLSwitchCaseContainsExit;
        }
        pub mod ir {
            pub mod SkSLEmptyExpression;

            pub mod SkSLBinaryExpression;
            pub mod SkSLBlock;
            pub mod SkSLBreakStatement;
            pub mod SkSLChildCall;
            pub mod SkSLConstructor;
            pub mod SkSLConstructorArray;
            pub mod SkSLConstructorArrayCast;
            pub mod SkSLConstructorCompound;
            pub mod SkSLConstructorCompoundCast;
            pub mod SkSLConstructorDiagonalMatrix;
            pub mod SkSLConstructorMatrixResize;
            pub mod SkSLConstructorScalarCast;
            pub mod SkSLConstructorSplat;
            pub mod SkSLConstructorStruct;
            pub mod SkSLContinueStatement;
            pub mod SkSLDiscardStatement;
            pub mod SkSLDoStatement;
            pub mod SkSLExpression;
            pub mod SkSLExpressionStatement;
            pub mod SkSLExtension;
            pub mod SkSLFieldAccess;
            pub mod SkSLFieldSymbol;
            pub mod SkSLForStatement;
            pub mod SkSLFunctionCall;
            pub mod SkSLFunctionDeclaration;
            pub mod SkSLFunctionDefinition;
            pub mod SkSLFunctionPrototype;
            pub mod SkSLFunctionReference;
            pub mod SkSLIRHelpers;
            pub mod SkSLIRNode;
            pub mod SkSLIfStatement;
            pub mod SkSLIndexExpression;
            pub mod SkSLInterfaceBlock;
            pub mod SkSLLayout;
            pub mod SkSLLiteral;
            pub mod SkSLMethodReference;
            pub mod SkSLModifierFlags;
            pub mod SkSLModifiers;
            pub mod SkSLModifiersDeclaration;
            pub mod SkSLNop;
            pub mod SkSLPoison;
            pub mod SkSLPostfixExpression;
            pub mod SkSLPrefixExpression;
            pub mod SkSLProgram;
            pub mod SkSLProgramElement;
            pub mod SkSLReturnStatement;
            pub mod SkSLSetting;
            pub mod SkSLStatement;
            pub mod SkSLStructDefinition;
            pub mod SkSLSwitchCase;
            pub mod SkSLSwitchStatement;
            pub mod SkSLSwizzle;
            pub mod SkSLSymbol;
            pub mod SkSLSymbolTable;
            pub mod SkSLTernaryExpression;
            pub mod SkSLType;
            pub mod SkSLTypeReference;
            pub mod SkSLVarDeclarations;
            pub mod SkSLVariable;
            pub mod SkSLVariableReference;
        }
        pub mod tracing {
            pub mod SkSLDebugTracePriv;
            pub mod SkSLTraceHook;
        }
    }
    pub mod text {
        pub mod GlyphRun;
        pub mod StrikeForGPU;
    }
    pub mod core {
        pub mod Sk4px;
        pub mod SkAAClip;
        pub mod SkAdvancedTypefaceMetrics;
        pub mod SkAlphaRuns;
        pub mod SkAnalyticEdge;
        pub mod SkArenaAlloc;
        pub mod SkAutoBlitterChoose;
        pub mod SkAutoMalloc;
        pub mod SkAutoPixmapStorage;
        pub mod SkBBHFactory;
        pub mod SkBezierCurves;
        pub mod SkBigPicture;
        pub mod SkBitmap;
        pub mod SkBitmapCache;
        pub mod SkBitmapDevice;
        pub mod SkBitmapProcState;
        pub mod SkBitmapProcState_matrixProcs;
        pub mod SkBitmapProcState_opts;
        pub mod SkBitmapProcState_opts_lasx;
        pub mod SkBitmapProcState_opts_ssse3;
        pub mod SkBlendMode;
        pub mod SkBlendModeBlender;
        pub mod SkBlendModePriv;
        pub mod SkBlenderBase;
        pub mod SkBlitBWMaskTemplate;
        pub mod SkBlitMask;
        pub mod SkBlitMask_opts;
        pub mod SkBlitRow;
        pub mod SkBlitRow_D32;
        pub mod SkBlitRow_opts;
        pub mod SkBlitRow_opts_lasx;
        pub mod SkBlitRow_opts_ml3;
        pub mod SkBlitter;
        pub mod SkBlitter_A8;
        pub mod SkBlitter_ARGB32;
        pub mod SkBlitter_Sprite;
        pub mod SkBlockAllocator;
        pub mod SkBlurEngine;
        pub mod SkBlurMask;
        pub mod SkBlurMaskFilterImpl;
        pub mod SkBuffer;
        pub mod SkCPUContext;
        pub mod SkCPUContextImpl;
        pub mod SkCPURecorder;
        pub mod SkCPURecorderImpl;
        pub mod SkCachedData;
        pub mod SkCanvas;
        pub mod SkCanvasPriv;
        pub mod SkCanvas_Raster;
        pub mod SkCapabilities;
        pub mod SkChecksum;
        pub mod SkClipStack;
        pub mod SkClipStackDevice;
        pub mod SkColor;
        pub mod SkColorData;
        pub mod SkColorFilter;
        pub mod SkColorFilterPriv;
        pub mod SkColorPriv;
        pub mod SkColorSpace;
        pub mod SkColorSpacePriv;
        pub mod SkColorSpaceXformSteps;
        pub mod SkColorTable;
        pub mod SkCompressedDataUtils;
        pub mod SkContainers;
        pub mod SkContourMeasure;
        pub mod SkConvertPixels;
        pub mod SkCoreBlitters;
        pub mod SkCpu;
        pub mod SkCubicClipper;
        pub mod SkCubics;
        pub mod SkData;
        pub mod SkDebug;
        pub mod SkDeque;
        pub mod SkDescriptor;
        pub mod SkDevice;
        pub mod SkDraw;
        pub mod SkDrawProcs;
        pub mod SkDrawShadowInfo;
        pub mod SkDrawTypes;
        pub mod SkDraw_atlas;
        pub mod SkDraw_text;
        pub mod SkDraw_vertices;
        pub mod SkDrawable;
        pub mod SkEdge;
        pub mod SkEdgeBuilder;
        pub mod SkEdgeClipper;
        pub mod SkEffectPriv;
        pub mod SkFDot6;
        pub mod SkFixedArray;
        pub mod SkFlattenable;
        pub mod SkFloatBits;
        pub mod SkFloatingPoint;
        pub mod SkFont;
        pub mod SkFontDescriptor;
        pub mod SkFontMgr;
        pub mod SkFontPriv;
        pub mod SkFont_serial;
        pub mod SkGaussFilter;
        pub mod SkGeometry;
        pub mod SkGlobalInitialization_core;
        pub mod SkGlyph;
        pub mod SkGlyphRunPainter;
        pub mod SkGraphics;
        pub mod SkHalf;
        pub mod SkIDChangeListener;
        pub mod SkImageFilter;
        pub mod SkImageFilterCache;
        pub mod SkImageFilterTypes;
        pub mod SkImageFilter_Base;
        pub mod SkImageGenerator;
        pub mod SkImageInfo;
        pub mod SkImageInfoPriv;
        pub mod SkKnownRuntimeEffects;
        pub mod SkLRUCache;
        pub mod SkLatticeIter;
        pub mod SkLineClipper;
        pub mod SkLocalMatrixImageFilter;
        pub mod SkLog;
        pub mod SkM44;
        pub mod SkMallocPixelRef;
        pub mod SkMask;
        pub mod SkMaskBlurFilter;
        pub mod SkMaskCache;
        pub mod SkMaskFilter;
        pub mod SkMaskFilterBase;
        pub mod SkMaskGamma;
        pub mod SkMathPriv;
        pub mod SkMatrix;
        pub mod SkMatrixInvert;
        pub mod SkMatrixPriv;
        pub mod SkMemset;
        pub mod SkMemset_opts;
        pub mod SkMemset_opts_avx;
        pub mod SkMemset_opts_erms;
        pub mod SkMesh;
        pub mod SkMeshPriv;
        pub mod SkMessageBus;
        pub mod SkMipmap;
        pub mod SkMipmapAccessor;
        pub mod SkMipmapBuilder;
        #[cfg(SK_USE_DRAWING_MIPMAP_DOWNSAMPLER)]
        pub mod SkMipmapDrawDownSampler;
        #[cfg(not(SK_USE_DRAWING_MIPMAP_DOWNSAMPLER))]
        pub mod SkMipmapHQDownSampler;
        pub mod SkNextID;
        pub mod SkNoDestructor;
        #[cfg(all(unix, not(target_os = "ios")))]
        pub mod SkOSFile;
        pub mod SkOpts;
        pub mod SkPaint;
        pub mod SkPaintDefaults;
        pub mod SkPaintPriv;
        pub mod SkPath;
        pub mod SkPathBuilder;
        pub mod SkPathData;
        pub mod SkPathDump;
        pub mod SkPathEffect;
        pub mod SkPathEffectBase;
        pub mod SkPathEnums;
        pub mod SkPathIter;
        pub mod SkPathMakers;
        pub mod SkPathMeasure;
        pub mod SkPathMeasurePriv;
        pub mod SkPathPriv;
        pub mod SkPathRaw;
        pub mod SkPathRawShapes;
        pub mod SkPathUtils;
        pub mod SkPath_interpolate;
        pub mod SkPath_serial;
        pub mod SkPicture;
        pub mod SkPictureData;
        pub mod SkPictureFlat;
        pub mod SkPicturePlayback;
        pub mod SkPicturePriv;
        pub mod SkPictureRecord;
        pub mod SkPictureRecorder;
        pub mod SkPixelRef;
        pub mod SkPixelStorage;
        pub mod SkPixmap;
        pub mod SkPixmapDraw;
        pub mod SkPoint;
        pub mod SkPoint3;
        pub mod SkPointPriv;
        pub mod SkPtrRecorder;
        pub mod SkQuadClipper;
        pub mod SkQuads;
        pub mod SkRRect;
        pub mod SkRRectPriv;
        pub mod SkRSXform;
        pub mod SkRTree;
        pub mod SkRandom;
        pub mod SkRasterClip;
        pub mod SkRasterClipStack;
        pub mod SkRasterPipeline;
        pub mod SkRasterPipelineBlitter;
        pub mod SkRasterPipelineContextUtils;
        pub mod SkRasterPipelineOpContexts;
        pub mod SkRasterPipelineOpList;
        pub mod SkReadBuffer;
        pub mod SkReadPixelsRec;
        pub mod SkRecord;
        pub mod SkRecordCanvas;
        pub mod SkRecordDraw;
        pub mod SkRecordOpts;
        pub mod SkRecordPattern;
        pub mod SkRecordedDrawable;
        pub mod SkRecords;
        pub mod SkRect;
        pub mod SkRectMemcpy;
        pub mod SkRectPriv;
        pub mod SkRegion;
        pub mod SkRegionPriv;
        pub mod SkRegion_path;
        pub mod SkResourceCache;
        pub mod SkRuntimeBlender;
        pub mod SkRuntimeEffect;
        pub mod SkRuntimeEffectPriv;
        pub mod SkSLTypeShared;
        pub mod SkSafeMath;
        pub mod SkSafeRange;
        pub mod SkSamplingPriv;
        pub mod SkScalar;
        pub mod SkScaleToSides;
        pub mod SkScalerContext;
        pub mod SkScan;
        pub mod SkScanPriv;
        pub mod SkScan_AAAPath;
        pub mod SkScan_AntiPath;
        pub mod SkScan_Antihair;
        pub mod SkScan_Hairline;
        pub mod SkScan_Path;
        pub mod SkSemaphore;
        pub mod SkSpecialImage;
        pub mod SkSpinlock;
        pub mod SkSpriteBlitter;
        pub mod SkSpriteBlitter_ARGB32;
        pub mod SkStream;
        pub mod SkStreamPriv;
        pub mod SkStrike;
        pub mod SkStrikeCache;
        pub mod SkStrikeRef;
        pub mod SkStrikeSpec;
        pub mod SkString;
        pub mod SkStringUtils;
        pub mod SkStroke;
        pub mod SkStrokeRec;
        pub mod SkStrokerPriv;
        pub mod SkSurfacePriv;
        pub mod SkSwizzlePriv;
        pub mod SkSwizzler_opts;
        pub mod SkSynchronizedResourceCache;
        pub mod SkTBlockList;
        pub mod SkTDArray;
        pub mod SkTDPQueue;
        pub mod SkTDynamicHash;
        pub mod SkTHash;
        pub mod SkTInternalLList;
        pub mod SkTLazy;
        pub mod SkTSearch;
        pub mod SkTSort;
        pub mod SkTextBlob;
        pub mod SkTextBlobPriv;
        pub mod SkTextFormatParams;
        pub mod SkThreadID;
        pub mod SkTypeface;
        pub mod SkTypefaceCache;
        pub mod SkUTF;
        pub mod SkUnPreMultiply;
        pub mod SkValidationUtils;
        pub mod SkVertState;
        pub mod SkVertices;
        pub mod SkVerticesPriv;
        pub mod SkVx;
        pub mod SkWriteBuffer;
        pub mod SkWritePixelsRec;
        pub mod SkWriter32;
        pub mod SkYUVAInfo;
        pub mod SkYUVAInfoLocation;
        pub mod SkYUVAPixmaps;
        pub mod SkYUVMath;
        pub mod SkYUVPlanesCache;
        pub mod SkZip;
    }
    pub mod opts {
        pub mod SkBitmapProcState_opts;
        pub mod SkBlitMask_opts;
        pub mod SkBlitRow_opts;
        pub mod SkMemset_opts;
        pub mod SkOpts_lasx;
        pub mod SkOpts_ml3;
        pub mod SkOpts_ml4;
        pub mod SkRasterPipeline_opts;
        pub mod SkSwizzler_opts;
    }
    pub mod shaders {
        pub mod SkBitmapProcShader;
        pub mod gradients {
            pub mod SkConicalGradient;
            pub mod SkGradientBaseShader;
            pub mod SkLinearGradient;
            pub mod SkRadialGradient;
            pub mod SkSweepGradient;
        }
        pub mod SkBlendShader;
        pub mod SkColorFilterShader;
        pub mod SkColorShader;
        pub mod SkCoordClampShader;
        pub mod SkEmptyShader;
        pub mod SkImageShader;
        pub mod SkLocalMatrixShader;
        pub mod SkPerlinNoiseShaderImpl;
        pub mod SkPerlinNoiseShaderType;
        pub mod SkPictureShader;
        pub mod SkRuntimeShader;
        pub mod SkShader;
        pub mod SkShaderBase;
        pub mod SkTransformShader;
        pub mod SkTriColorShader;
        pub mod SkWorkingColorSpaceShader;
    }
    pub mod image {
        pub mod SkImage;
        pub mod SkImageGeneratorPriv;
        pub mod SkImage_Base;
        pub mod SkImage_Lazy;
        pub mod SkImage_LazyFactories;
        pub mod SkImage_Picture;
        pub mod SkImage_Raster;
        pub mod SkImage_RasterFactories;
        pub mod SkPictureImageGenerator;
        pub mod SkRescaleAndReadPixels;
        pub mod SkSurface;
        pub mod SkSurface_Base;
        pub mod SkSurface_Null;
        pub mod SkSurface_Raster;
    }
    pub mod effects {
        pub mod Sk1DPathEffect;
        pub mod Sk2DPathEffect;
        pub mod SkBlenders;
        pub mod SkColorMatrix;
        pub mod SkColorMatrixFilter;
        pub mod SkCornerPathEffect;
        pub mod SkDashImpl;
        pub mod SkDashPathEffect;
        pub mod SkDiscretePathEffect;
        pub mod SkEmbossMask;
        pub mod SkEmbossMaskFilter;
        pub mod SkShaderMaskFilterImpl;
        pub mod SkTableMaskFilter;
        pub mod SkTrimPE;
        pub mod SkTrimPathEffect;
        pub mod imagefilters {
            pub mod SkBlendImageFilter;
            pub mod SkBlurImageFilter;
            pub mod SkColorFilterImageFilter;
            pub mod SkComposeImageFilter;
            pub mod SkCropImageFilter;
            pub mod SkDisplacementMapImageFilter;
            pub mod SkDropShadowImageFilter;
            pub mod SkImageImageFilter;
            pub mod SkLightingImageFilter;
            pub mod SkMagnifierImageFilter;
            pub mod SkMatrixConvolutionImageFilter;
            pub mod SkMatrixTransformImageFilter;
            pub mod SkMergeImageFilter;
            pub mod SkMorphologyImageFilter;
            pub mod SkPictureImageFilter;
            pub mod SkRuntimeImageFilter;
            pub mod SkShaderImageFilter;
        }
        pub mod colorfilters {
            pub mod SkBlendModeColorFilter;
            pub mod SkColorFilterBase;
            pub mod SkColorSpaceXformColorFilter;
            pub mod SkComposeColorFilter;
            pub mod SkGaussianColorFilter;
            pub mod SkMatrixColorFilter;
            pub mod SkRuntimeColorFilter;
            pub mod SkTableColorFilter;
            pub mod SkWorkingFormatColorFilter;
        }
    }
    pub mod sfnt {
        pub mod SkIBMFamilyClass;
        pub mod SkOTTableTypes;
        pub mod SkOTTable_OS_2;
        pub mod SkOTTable_OS_2_V0;
        pub mod SkOTTable_OS_2_V1;
        pub mod SkOTTable_OS_2_V2;
        pub mod SkOTTable_OS_2_V3;
        pub mod SkOTTable_OS_2_V4;
        pub mod SkOTTable_OS_2_VA;
        pub mod SkOTTable_head;
        pub mod SkOTTable_name;
        pub mod SkOTUtils;
        pub mod SkPanose;
        pub mod SkSFNTHeader;
    }
    pub mod utils {
        pub mod SkCustomTypeface;
        pub mod SkLogHandler;
        pub mod SkPatchUtils;
        pub mod SkPolyUtils;
        pub mod SkShadowTessellator;
        pub mod SkShadowUtils;
        #[cfg(target_vendor = "apple")]
        pub mod mac {
            pub mod SkCGGeometry;
            pub mod SkCTFont;
            pub mod SkCTFontCreateExactCopy;
            pub mod SkUniqueCFRef;
        }
        pub mod SkDashPath;
        pub mod SkDashPathPriv;
        pub mod SkFloatUtils;
        pub mod SkMatrix22;
    }
}

// The translated ICC bridge remains in-tree, but encoded-image decoding is a
// separate package boundary in the browser CPU embedding. Selecting the native
// Rust ICC macro therefore requires a dedicated decoder embedding rather than
// silently changing the color-profile parser.
#[cfg(SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST)]
compile_error!(
    "the Rust ICC codec branch belongs to an encoded-image decoder embedding, not browser_cpu"
);

// Actual Chromium numeric producer files; separate from Skia row accounting.
// Their unresolved native inputs remain explicit compile errors, not defaults.
#[cfg(feature = "chromium_embedding")]
pub mod base;
// Actual C++ partition_alloc namespace retains its mirrored source subtree.
#[cfg(feature = "chromium_embedding")]
pub use base::allocator::partition_allocator::src::partition_alloc;

// Genuine compiler primitive transport; all Skia arithmetic stays in Rust.
pub mod interop {
    #[cfg(sk_native_long_double_ffi)]
    pub mod native_long_double;
    #[cfg(feature = "chromium_embedding")]
    pub mod native_ostream;
}

// Rust integration error boundary, not attributed to upstream source lines.
// Incomplete CPU stage support must remain visible to the renderer caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RasterError {
    MissingStage(crate::src::core::SkRasterPipeline::MissingStage),
    RejectedStages,
}
impl From<crate::src::core::SkRasterPipeline::MissingStage> for RasterError {
    fn from(error: crate::src::core::SkRasterPipeline::MissingStage) -> Self {
        Self::MissingStage(error)
    }
}
impl From<crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError>
    for RasterError
{
    fn from(
        error: crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError,
    ) -> Self {
        use crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError;
        match error {
            ColorFilterEvaluationError::MissingStage(stage) => Self::MissingStage(stage),
            ColorFilterEvaluationError::RejectedStages => Self::RejectedStages,
        }
    }
}
impl core::fmt::Display for RasterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingStage(error) => error.fmt(f),
            Self::RejectedStages => f.write_str("CPU raster stage construction was rejected"),
        }
    }
}
impl std::error::Error for RasterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingStage(error) => Some(error),
            Self::RejectedStages => None,
        }
    }
}
