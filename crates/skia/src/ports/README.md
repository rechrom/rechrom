# 字形与 CPU 绘制参考（迁移记录）

这些实现已从 renderer 移到本地 skia。下文保留历史对照范围；其中提到的
 tiny-skia 路径现在是本 crate 中直接迁入的实现，已经没有外部 tiny-skia 依赖。
 最新目录和来源映射以 translation_map.json 为准。

# Skia 对应关系

文字掩码沿用 macOS Skia 的系统字体边界：Rust 调用 CoreText/CoreGraphics，
Rust 执行位置量化、覆盖率转换和 A8 合成；图像、渐变和圆角使用下述
Skia 对应阶段，其它绘制仍使用 tiny-skia。
生产模块不调用 C++ Skia。原生 Skia 仅用于 `source_replay` 对照测试。

| Rust 文件 / 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `../core/SkGlyph.rs::position_transform` | `SkGlyph.cpp::SkGlyphPositionRoundingSpec`、`SkGlyph.h::SkPackedGlyphID`、`SkGlyphRunPainter.cpp` 的 CPU mask 分支 | 设备空间的位置量化：沿水平基线 1/4 像素，垂直方向整像素；旋转 / 斜切对应轴规则 |
| `SkScalerContext_mac_ct.rs::SkScalerContextMac` | `SkScalerContext_mac_ct.cpp::generateMetrics`、`Offscreen::getCG`、`generateImage` | 字体 / 变体、掩码边界、系统栅格、关闭系统重复量化、字形定位 |
| `SkScalerContext_mac_ct.rs::smooth_behavior` | `SkCTFont.cpp::SkCTFontGetSmoothBehavior` | 同一个小型探测字体检测系统是否启用灰度或子像素平滑；字体许可证见 `SKIA_LICENSE` |
| `../core/SkMaskGamma.rs::mac_smoothing_lut` | `SkTypeface_mac_ct.cpp::onFilterRec`、`SkScalerContext.cpp::PreprocessRec`、`SkMaskGamma.cpp::SkTMaskGamma_build_correcting_lut` | 三位颜色规范化、macOS 平滑颜色调整、A8 亮度、sRGB 覆盖率 LUT；当前软件截图的 contrast=0 |
| `../core/SkBlitter_ARGB32.rs::blend_mask` | `SkBlitMask_opts.h::blit_mask_d32_a8_*` 的 ARM NEON 路径 | 预乘 Alpha、覆盖率和整数合成取整；文字与阴影共用 |
| `../core/SkCanvas.rs::Canvas::draw_glyphs` | 项目 `skia_renderer.cc::DrawGlyphs`、Skia CPU glyph run painter | 共享 DisplayItem 输入；选择系统掩码或轮廓回放，应用画布裁剪 |

渐变阶段另外位于 `../core/SkRasterPipeline.rs`：对应
`SkLinearGradient.cpp::pts_to_unit_matrix`、
`SkGradientBaseShader.cpp::init_stop_evenly` 和
`SkRasterPipeline_opts.h` 的 matrix / gradient / dither / src-over / byte-store
阶段。源码截图分块中的设备坐标决定 8×8 抖动相位。当前对不透明、均匀
色标、Pad 和整数矩形平移路径启用；其它配置仍走 tiny-skia。
`tests/gradient_replay.rs` 对照 96 组场景，包括实际百度渐变的非零起点。

图像阶段位于 `../core/SkBitmapProcState.rs`，保留 Skia 中两条不同的采样路径：

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `build_mip_image` / `downsample` | 项目 `DrawImage` 的 mip 缓存、`SkMipmapHQDownSampler.cpp`、`SkMipmapAccessor.cpp` | 奇偶尺寸的箱式 / 三角形滤波，选择 floor 尺寸层级，生成项目使用的 ceil 尺寸缓存 |
| `sample_bilinear_lowp` | `SkRasterPipeline_opts.h::LOWP_STAGE_GP(bilerp_clamp_8888)` | `scalePixels` 的 16.16 坐标与 Q15 双线性插值 |
| `draw_image_bitmap` / `sample_bilinear_bitmap` | `SkBitmapProcStateAutoMapper`、`filter_scale`、`S32_alpha_D32_filter_DX`、N32 src-over | 最终图片绘制的 32.32 起点 / 步长、四位小数权重、截图分块坐标和整数合成 |

最终图片专用路径限完整源图、整数目标矩形与平移变换。绘制区域内的
裁剪必须为 0 / 255 覆盖率；区域外的圆角抗锯齿不会改变采样路径。
分数边缘、部分源图、非平移变换和穿过图片的软裁剪仍走 tiny-skia。
`tests/fixtures/image_mipmap.rgba` 的 3,610 个 C++ 用例比较完整预乘像素；
`tests/image_replay.rs` 比较 432 组最终缩放、透明度、硬裁剪、区域外软裁剪
与跨分块场景。实际在线图片诊断见 `artifacts/live-baidu-fidelity`。

上层保持现有 `DisplayItemList`，不暴露 CoreText 或 tiny-skia 类型。
默认 `GlyphRasterMode::Platform` 在 macOS 对普通、无描边的水平文字使用
CoreText 掩码。合成粗斜体、描边和非平移画布变换仍走已有轮廓实现，尚未
证明这些路径与 Skia 像素一致。彩色字体掩码也尚未完成。
`GlyphRasterMode::Outlines` 显式选择完全由 Rust 解析和栅格化的轮廓路径。
其它平台目前沿用轮廓路径，不能据 macOS 的结果声称跨平台像素一致。

验证入口：

```sh
cargo test --offline -p renderer --no-default-features \
  --features pure_replay,source_replay --test glyph_replay
cargo test --offline -p skia device_positions_match_skia_cpu_mask_oracle
```

`glyph_replay` 使用相同绘制指令分别经过原生 Skia 和 Rust 后端，比较完整
RGBA 字节，覆盖 216 组中英文 / 字号 / 平滑 / 透明度 / 亚像素位置 / 裁剪组合。
位置测试的 6,144 组参考由 `tests/oracles/glyph_position.cc` 生成。
阴影也通过 `../core/SkBlitter_ARGB32.rs::blend_premultiplied` 直接合成 A8 掩码，
避免中间 RGBA 图像的浮点 src-over 取整差异。`tests/shadow_replay.rs`
分别对照透明阴影叠加和搜索框两层模糊阴影的完整 RGBA；24 组单层位置/透明度变体与 4 组输入/挖空 mask 也达到 0。

页面诊断记录位于 `artifacts/glyph-raster-diagnosis`；文字专用截图比较与整页
比较分别记录，整页仍受脚本和其它绘制功能影响。

圆角阶段位于 `../core/SkScan_AAAPath.rs`，生产代码不调用原生 Skia：

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `AnalyticEdge` / `Quad` | `SkAnalyticEdge.cpp` 的 line / quadratic / updateQuadratic | 16.16 边数据、1/4 像素 Y 量化、前向差分与零高度段处理 |
| `conic_quads` / `add_conic` | `SkGeometry.cpp` 的 `SkConic::computeQuadPOW2` / `chop` / `subdivide` | 0.25 像素容差，保留参考库的 legacy conic chop 顺序 |
| `chop_quad_axis` / `clipped_quad` / `add_line` | `SkChopQuadAt{Y,X}Extrema`、`SkEdgeClipper::clipMonoQuad`、`SkFindUnitQuadRoots`、`SkAnalyticEdgeBuilder::combineVertical` | 先裁剪再建边，合并相邻竖直线段 |
| `convex_walk` / `Coverage` | `SkScan_AAAPath.cpp::aaa_walk_convex_edges`、`blit_trapezoid_row` | 梯形覆盖率、小图形 mask 与 RLE 吸附规则 |
| `rounded_rect_mask` | 项目 `skia_renderer.cc::tiles_for_axis` | 256 像素分块、1 像素边界、局部设备坐标 |
| `../core/SkBlitter_ARGB32.rs::blend_anti_h2` | `SkARGB32_*_Blitter::blitAntiH2` | 普通 / 透明 / 黑色的不同整数合成规则，裁剪包装器回退到 `blitAntiH` |

当前接入平移变换下的圆角填充与抗锯齿圆角裁剪；旋转、缩放以及超出
当前固定点范围的图形保留 tiny-skia 路径。48 组原始覆盖率检查和 108 组
RGBA 场景均与原生 Skia 一致，后者覆盖黑色 / 彩色 / 透明颜色、硬裁剪、
圆角软裁剪、小数坐标和跨分块。任意路径、圆角描边和阴影仍未完全对齐。

直线与一般曲线填充继续复用相同的 Canvas/A8 合成边界：

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `../core/SkAnalyticEdge.rs::Cubic` | `SkAnalyticCubicEdge::setCubicWithoutUpdate/updateCubic/keepContinuous` | 三次曲线前向差分、零高度段、单调 Y 与连续 X |
| `chop_y_extrema` | `SkChopCubicAtYExtrema`、`SkFindCubicExtrema` | 单位二次根、一次/两次并行 De Casteljau 切分、极值处控制点拉平 |
| `../core/SkScan_AAAPath.rs::general_walk` | `SkScan_AAAPath.cpp::aaa_walk_edges` | 绕组/奇偶区间、曲线更新、活跃边与待进入边的插入/相交检查、禁止直接 blit 的重叠场景 |
| `path_mask` | `SkAnalyticEdgeBuilder`、凸/一般 AAA 路径分派 | 单轮廓保守凸性判断，多轮廓一般扫描，源分块坐标与 mask/RLE |
| `../../core/SkCanvas.rs::draw_axis_stroke` | `SkStroke` 的直线与端帽、`blitFatAntiRect` | 宽度大于 1px 的水平/竖直描边，平头/方头/圆头；矩形面积截断与端点吸附 |

`line_stroke_replay` 的 216 组 RGBA 对照全部一致。`cubic_path_replay`
的 96 组凸轮廓、72 组双轮廓圆环和 24 组实际在线图标几何对照全部一致。
专用路径限平移、有限定点范围、允许三次和二次曲线跨块/裁剪边界。新增 48 组实际 SVG 的硬/AA 裁剪及跨分块用例全部一致。
二次路径命令现在先按源实现切分 Y 极值；conic 路径命令和复杂单轮廓
尚未接入此分派。一般曲线描边的其它配置继续使用 tiny-skia；不能据局部测试声称
完整 Skia 或整页行为已完成。

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `../analytic_aa/cubic_clip.rs` | `SkEdgeClipper::clipCubic/clipMonoCubic`、`SkLineClipper::ClipLine` | Y/X 极值细分、双精度边界求交、边界投影与右侧剔除 |
| `../analytic_aa/polynomial_roots.rs` | `SkCubics`、`SkQuads`、`SkGeometry::first_axis_intersection` | 同阶降级、判别式 FMA、Cardano、单位根修正与区间回退 |
| `../../core/SkCanvas.rs::CanvasState` | `SkRasterClip::op/updateCache` | 近整数 AA 矩形转硬裁剪、AA 矩形覆盖率与矩形 mask 缓存 |
| `../../core/SkCanvas.rs::draw_box_shadow` | `SkDraw::DrawToMask`、`SkEdgeClipper`、AAA 解析覆盖率 | 圆角模糊输入与内部挖空 mask，复用整数模糊和 A8 合成 |

细描边阶段位于 `../hairline.rs`，对应 `SkDrawTreatAsHairline`、
`SkScan_Hairline::hair_quad/hairconic/hair_cubic` 和
`SkScan_AntiHair::do_anti_hairline`。保留 16.6 坐标、端点覆盖率、
逐段颜色合成和 `SkRegion::Cliperator` 的线段整数交集范围；H2/V2
经过裁剪包装器时回退 AntiH，不能把所有段先相加成单张 mask。

`uniform_border_path` 对应源 `CanStrokeDoubleRoundedRect` 与 RRect inset，
接入 <=1px、圆角半径可直接 inset 的均匀边框。`hairline_replay` 的
72 组圆角边框和 72 组线/二次/圆锥/三次曲线完整 RGBA 对照全部为 0。
当前仅平移、AA、butt cap；三次曲线须通过 Skia 的 quick niceness 检查，
非 nice 曲线的最大曲率切分、其它端帽及一般宽曲线描边仍保留原路径。

渐变对照扩展到 144 组非恒定色标（加入圆角软裁剪）和 8 组恒定色标，
均为 0。恒定渐变先前回退 tiny-skia，如今复用源浮点渐变/抖动阶段；
它仍是渐变 shader，保留 Skia 的 shader 抖动与合成顺序。

旋转/翻转字形继续使用同一个 `SkScalerContextMac`，对应 Mac port
`MatrixToCGAffineTransform`、`generateMetrics`、`Offscreen::getCG`：
CoreGraphics 的 Y 轴符号转换、线性变换后的字形边界、文本矩阵及逆矩阵
映射的绘制位置均由 Rust 处理。`glyph_position::mask_transform` 对应
`SkScalerContext::MakeRecAndEffects::sk_relax`，先把 mask 的矩阵量化到
1/1024；基线位置仍由原始 CTM 映射，再按量化矩阵确定亚像素轴。
CSS 180° 旋转的微小正弦项因此归零，与源 glyph cache 行为一致。

新增 216 组 Arial 单位轴旋转/翻转完整 RGBA 对照全部为 0，覆盖
0/90/180/270°、两轴反射、180° 浮点正弦残值、三种字号/平滑及亚像素位置，
含半透明颜色。原有 216 组普通文字检查继续通过。当前仅接入量化后
单位轴旋转/翻转；一般缩放/斜切仍需 `computeMatrices` 的完整分解，
合成粗体/斜体、文字描边、彩色字体等尚未在此路径完成验证。

闭合平滑曲线描边新增 `../stroke.rs`，逐项对应 `SkStroke.cpp`：

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `QuadConstruct` / `curve_stroke` | `SkQuadConstruct`、`cubicStroke` / `quadStroke` | 原参数区间、端点共享、二次近似与源递归限制 |
| `Curve::ray` / `P::length` | `setRayPts`、`SkEvalCubicAt`、`SkPoint::setLength` | 保存切线向量，f32 曲线求值及 double 长度归一化 |
| `intersect_ray` / `close_enough` | `intersectRay`、`strokeCloseEnough` | 切线求交、射线/二次根、误差和尖角检查 |
| `outline` / `reverse_contour` | `preJoinTo/postJoinTo/finishContour`、`Dot2AngleType` 的 nearly-line 分支 | 两侧轮廓、闭合和反向内轮廓 |
| `../core/SkScan_AAAPath.rs` 的二次命令及 `clipped_quad_with_culling` | `SkEdgeBuilder::build`、`SkEdgeClipper::clipMonoQuad` | Y 极值切分及非凸路径整体在右侧时剔除；保留部分跨界的竖直投影 |

旧 tiny-skia stroker 把切线存为端点再减回向量，并用加权端点计算交点；
当前 Skia 使用直接切线向量和 `start + tangent * t`。大坐标可使旧轮廓
差几个像素，因此新的平滑闭合分支保留当前 Skia 的算术。108 组完整
描边 RGBA 及 108 组二次轮廓填充 RGBA 全部为 0，覆盖三宽度、三位置
（含小数与分块）、不裁剪/硬/AA 裁剪、两透明度。

尖锐连接、开放端帽、可能有 cusp 的交叉控制线、圆锥描边、非平移等
仍由原 stroker 生成轮廓或回退。这里尚未声明完整 SkStroke 已翻译。
一次真实在线百度完整绘制列表（556 指令、18 图片、27 字体）共享回放
达到 0，18 条隔离几何指令也为 0。完整浏览器脚本运行验收尚未完成。

后续新增实际 C++ 原始入口执行在线脚本后的绘制列表对照（623 指令、
123 分组、19 图片、27 字体）。完整导出资源的原生重放与原始 PNG
为 0，Rust 重放也为 0。对应的源结构进一步扩展：

| Rust 入口 | Skia 参考 | 职责 |
| --- | --- | --- |
| `Curve::Conic::eval` / `curve_stroke` | `SkConicCoeff`、`evalTangentAt`、`compareQuadConic` / `conicStroke` | 保留有理曲线求值和切线，用原参数区间构建平行二次轮廓 |
| `cap` / `outline` | `SkStrokerPriv::RoundCapper`、`SkPathStroker::finishContour` | 开放 butt/round 端帽和内轮廓反接；圆端帽用两段 conic |
| `analytic_aa::path_mask` | `SkAnalyticEdgeBuilder` 的 conic 分支 | 设备坐标 0.25px 细分、单调极值和裁剪 |
| `analytic_aa/path_geometry.rs::is_convex` | `SkPathPriv::Convexicator`、`IsConcaveBySign` | 设备坐标变换后的 f32 方向变化，选择凸或普通 winding 扫描 |

162 组 conic/开放圆端点及 36 组微小三次圆点变换场景完整 RGBA 均为 0。
此前的开放端帽、conic 限制对这些已覆盖配置不再适用；锐角连接、
square 端帽、cusp、一般变换和完整 SkStroke 仍未声明完成。
Rust 自己执行脚本并生成相同布局的完整浏览器运行目标尚未验收。
