#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::ffi::c_char;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use cssom::{CSSDeclaration, CSSKeyframesRule};
use dom::persistent_document::{DOMNamespace, DOMNode, PersistentDocument, DOM};
use html::html_parser::ParseHTMLBytes;
use image_resource::{
    ContainerKey, CreatedDocumentImage, DocumentImage, DocumentImageEffect, DocumentImageFrame,
    DocumentImageMutation, ImageId, IntrinsicSize, MutationCause,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use style::media_queries::media_query_backend::ParseMediaQuerySet;
use style::media_queries::media_query_evaluator::MediaQueryEvaluator;
use style::media_queries::{MediaValuesCached, MediaValuesCachedData};
use style::PreferredColorScheme;

unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
}

fn PreferredScheme(value: u8) -> PreferredColorScheme {
    // ContainerKey uses the renderer-neutral image encoding: 0 is light and
    // 1 is dark. Blink's mojom enum orders dark before light.
    if value == 1 {
        PreferredColorScheme::kDark
    } else {
        PreferredColorScheme::kLight
    }
}

// cpp: image_decoder/svg_image_decoder.h:8-20
pub struct SVGImageDecoder {
    assembly_: LayoutAssembly,
    host_constraints_: layoutng_assembly::internal::layout_input::ConstraintSpace,
}

#[derive(Clone)]
struct TimedKeyframe {
    offset: f64,
    declarations: Vec<CSSDeclaration>,
}

#[derive(Clone)]
struct CSSImageAnimation {
    node_id: u64,
    effect_id: u64,
    duration: f64,
    delay: f64,
    iterations: f64,
    easing: String,
    keyframes: Vec<TimedKeyframe>,
}

struct SVGDocumentAnimation {
    resource_id: ImageId,
    document: DOM,
    style_engine: style::StyleEngine,
    engine: layoutng_assembly::layout_engine::LayoutEngine,
    paint_engine: paint::paint_engine::PaintEngine,
    constraints: layoutng_assembly::internal::layout_input::ConstraintSpace,
    width: u32,
    height: u32,
    container: ContainerKey,
    revision: u64,
    animations: Vec<CSSImageAnimation>,
    last_sample: Option<AnimationSampleBatch>,
}

#[derive(Clone, Debug, PartialEq)]
struct AnimationSample {
    node_id: u64,
    effect_id: u64,
    declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Debug, PartialEq)]
struct AnimationSampleBatch {
    timeline_time: Duration,
    samples: Vec<AnimationSample>,
}

impl SVGImageDecoder {
    // cpp: image_decoder/svg_image_decoder.h:12-13
    pub fn new(assembly: &LayoutAssembly) -> Self {
        Self {
            assembly_: *assembly,
            host_constraints_: Default::default(),
        }
    }

    pub fn new_with_constraints(
        assembly: &LayoutAssembly,
        constraints: &layoutng_assembly::internal::layout_input::ConstraintSpace,
    ) -> Self {
        Self {
            assembly_: *assembly,
            host_constraints_: constraints.clone(),
        }
    }

    // cpp: image_decoder/svg_image_decoder.h:15-15
    // cpp: image_decoder/svg_image_decoder.cc:68-81
    pub fn CanDecode(&self, bytes: &[u8], mime_type: &str) -> bool {
        if SVGLowerASCII(mime_type).contains("image/svg+xml") {
            return true;
        }
        let mut offset = 0;
        while offset < bytes.len() && bytes[offset].is_ascii_whitespace() {
            offset += 1;
        }
        const SVG: &[u8] = b"<svg";
        bytes.len() - offset >= SVG.len()
            && SVG
                .iter()
                .zip(&bytes[offset..])
                .all(|(expected, actual)| *expected == actual.to_ascii_lowercase())
    }

    // cpp: image_decoder/svg_image_decoder.cc:83-110
    // The first half of Decode, shared with the source document pipeline below.
    fn PrepareDocument(&self, bytes: &[u8], mime_type: &str) -> io::Result<(DOM, i32, i32)> {
        if !self.CanDecode(bytes, mime_type) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input is not an SVG image",
            ));
        }
        let mut document = ParseHTMLBytes(bytes);
        let svg =
            FindSVG(document.GetDocument(), document.GetDocument().Root()).ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "SVG image has no root svg element")
            })?;
        let svg = document.GetDocument().Node(svg);
        let mut width = NumberAttribute(svg, "width");
        let mut height = NumberAttribute(svg, "height");
        let view_box = ViewBoxSize(svg);
        if width.is_none() {
            width = view_box.map(|view_box| view_box.0);
        }
        if height.is_none() {
            height = view_box.map(|view_box| view_box.1);
        }
        let (width, height) = width.zip(height).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "SVG image has no intrinsic dimensions",
            )
        })?;
        const MAXIMUM_DIMENSION: f64 = 8192.0;
        if width > MAXIMUM_DIMENSION || height > MAXIMUM_DIMENSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "SVG image dimensions exceed the raster limit",
            ));
        }
        let pixel_width = (width.ceil() as i32).max(1);
        let pixel_height = (height.ceil() as i32).max(1);
        // The inert parser intentionally does not run Page's stylesheet
        // scheduler.  SVGImage owns an isolated document, so admit its inline
        // author sheets here before resolving the nested document.
        let style_nodes: Vec<_> = (0..document.GetDocument().NodeCount())
            .filter(|&index| {
                document
                    .GetDocument()
                    .Node(index)
                    .IsElement("style", DOMNamespace::kSVG)
                    || document.GetDocument().Node(index).IsHTMLElement("style")
            })
            .collect();
        for index in style_nodes {
            fn text(document: &PersistentDocument, index: usize, output: &mut String) {
                let node = document.Node(index);
                if node.Type() == dom::persistent_document::DOMNodeType::kText {
                    output.push_str(node.Data());
                }
                for &child in node.Children() {
                    text(document, child, output);
                }
            }
            let mut source = String::new();
            text(document.GetDocument(), index, &mut source);
            let mut sheet = style::ParseCSS(&source);
            sheet.owner_node_id = document.GetDocument().Node(index).Id();
            document.GetDocumentMut().AppendStyleSheet(sheet);
        }
        document
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(concat!(
                "html,body{margin:0;padding:0;overflow:hidden;background:transparent;}",
                "html,body{width:100%;height:100%;}svg{display:block;}"
            )));
        Ok((document, pixel_width, pixel_height))
    }

    fn PrepareDocumentImage(
        &self,
        resource_id: ImageId,
        bytes: &[u8],
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<SVGDocumentAnimation> {
        use layoutng_assembly::internal::layout_input::{Size, ViewportGeometry};
        use layoutng_assembly::internal::layout_input_types::IntSize;

        let (mut document, width, height) = self.PrepareDocument(bytes, mime_type)?;
        let mut container = container.clone();
        if container.width == 0 {
            container.width = width as u32;
        }
        if container.height == 0 {
            container.height = height as u32;
        }
        let media = ImageMediaData(&container);
        let mut style_engine = style::StyleEngine::new(&document);
        style_engine
            .Update(&mut document, &media, &[])
            .map_err(|error| io::Error::other(error.to_string()))?;
        let environment = MediaValuesCached::new(&media);
        let animations = CollectAnimations(document.GetDocument(), &environment)?;
        let mut constraints = self.host_constraints_.clone();
        constraints.available_size = Size {
            width: f64::from(container.width),
            height: f64::from(container.height),
        };
        constraints.viewport = Some(ViewportGeometry {
            size: IntSize {
                width: container.width as i32,
                height: container.height as i32,
            },
            ..Default::default()
        });
        Ok(SVGDocumentAnimation {
            resource_id,
            document,
            style_engine,
            engine: layoutng_assembly::layout_engine::LayoutEngine::new(&self.assembly_),
            paint_engine: paint::paint_engine::PaintEngine::new(),
            constraints,
            width: width as u32,
            height: height as u32,
            container,
            revision: 0,
            animations,
            last_sample: None,
        })
    }
}

impl SVGDocumentAnimation {
    fn UpdateContainerEnvironment(&mut self) {
        use layoutng_assembly::internal::layout_input::{Size, ViewportGeometry};
        use layoutng_assembly::internal::layout_input_types::IntSize;
        self.constraints.available_size = Size {
            width: f64::from(self.container.width),
            height: f64::from(self.container.height),
        };
        self.constraints.viewport = Some(ViewportGeometry {
            size: IntSize {
                width: self.container.width as i32,
                height: self.container.height as i32,
            },
            ..Default::default()
        });
    }

    fn Sample(&self, elapsed: Duration) -> AnimationSampleBatch {
        let samples = self
            .animations
            .iter()
            .map(|animation| AnimationSample {
                node_id: animation.node_id,
                effect_id: animation.effect_id,
                declarations: SampleAnimation(animation, elapsed.as_secs_f64()),
            })
            .collect();
        AnimationSampleBatch {
            timeline_time: elapsed,
            samples,
        }
    }

    fn RenderSample(
        &mut self,
        sample: AnimationSampleBatch,
    ) -> io::Result<Arc<DocumentImageFrame>> {
        for animation in &sample.samples {
            self.document.GetDocumentMut().SetAnimationStyle(
                animation.node_id,
                animation.effect_id,
                animation.declarations.clone(),
            );
        }
        self.style_engine
            .Update(&mut self.document, &ImageMediaData(&self.container), &[])
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.document
            .EmitConstraints(&self.constraints, |mutation| {
                self.engine.ApplyMutation(mutation);
            });
        self.document
            .EmitLayoutMutations(&dom::UserInteractionState::default(), |mutation| {
                self.engine.ApplyMutation(mutation);
            });
        self.engine.Layout();
        let fragments = self
            .engine
            .TakeLayoutResult()
            .ok_or_else(|| io::Error::other("animated SVG layout produced no fragments"))?;
        self.paint_engine.Paint(&fragments, None);
        let artifact = self
            .paint_engine
            .GetPaintResult()
            .ok_or_else(|| io::Error::other("animated SVG paint produced no artifact"))?
            .clone();
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| io::Error::other("document image revision exhausted"))?;
        self.last_sample = Some(sample);
        let record = Arc::new(paint::paint_engine::DocumentPaintArtifactRecord {
            artifact,
            intrinsic_size: IntrinsicSize {
                width: self.width,
                height: self.height,
            },
            record_size: IntrinsicSize {
                width: self.container.width,
                height: self.container.height,
            },
            container_key: self.container.clone(),
        });
        Ok(Arc::new(DocumentImageFrame {
            resource_id: self.resource_id,
            revision: self.revision,
            intrinsic_size: IntrinsicSize {
                width: self.width,
                height: self.height,
            },
            container_key: self.container.clone(),
            record,
        }))
    }

    fn RenderAt(&mut self, elapsed: Duration) -> io::Result<Arc<DocumentImageFrame>> {
        let sample = self.Sample(elapsed);
        self.RenderSample(sample)
    }
}

impl DocumentImage for SVGDocumentAnimation {
    fn apply_mutation(
        &mut self,
        mutation: DocumentImageMutation,
    ) -> io::Result<Vec<DocumentImageEffect>> {
        // The isolated SVG document owns a complete Style/Layout/Paint
        // lifecycle. Its typed mutation boundary is therefore also its cppgc
        // allocation boundary; the application thread that hosts this object
        // does not need to know which mutation will allocate native style.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        match mutation {
            DocumentImageMutation::AdvanceTimeline {
                frame_time,
                begin_frame_sequence,
            } => {
                let sample = self.Sample(frame_time);
                if self
                    .last_sample
                    .as_ref()
                    .is_some_and(|previous| previous.samples == sample.samples)
                {
                    return Ok(self
                        .has_active_animation()
                        .then_some(DocumentImageEffect::RequestBeginFrame)
                        .into_iter()
                        .collect());
                }
                let frame = self.RenderSample(sample)?;
                let mut effects = vec![DocumentImageEffect::FrameChanged {
                    frame,
                    cause: MutationCause {
                        begin_frame_sequence,
                    },
                }];
                if self.has_active_animation() {
                    effects.push(DocumentImageEffect::RequestBeginFrame);
                }
                Ok(effects)
            }
            DocumentImageMutation::SetContainer { container } => {
                if self.container == container {
                    return Ok(Vec::new());
                }
                self.container = container;
                if self.container.width == 0 {
                    self.container.width = self.width;
                }
                if self.container.height == 0 {
                    self.container.height = self.height;
                }
                self.UpdateContainerEnvironment();
                let frame = self.RenderAt(
                    self.last_sample
                        .as_ref()
                        .map_or(Duration::ZERO, |s| s.timeline_time),
                )?;
                Ok(vec![DocumentImageEffect::FrameChanged {
                    frame,
                    cause: MutationCause {
                        begin_frame_sequence: 0,
                    },
                }])
            }
            DocumentImageMutation::SetPreferredColorScheme { scheme } => {
                if self.container.preferred_color_scheme == scheme {
                    return Ok(Vec::new());
                }
                self.container.preferred_color_scheme = scheme;
                let frame = self.RenderAt(
                    self.last_sample
                        .as_ref()
                        .map_or(Duration::ZERO, |s| s.timeline_time),
                )?;
                Ok(vec![DocumentImageEffect::FrameChanged {
                    frame,
                    cause: MutationCause {
                        begin_frame_sequence: 0,
                    },
                }])
            }
        }
    }

    fn has_active_animation(&self) -> bool {
        !self.animations.is_empty()
    }
}

impl image_resource::DocumentImageDecoder for SVGImageDecoder {
    fn can_decode(&self, bytes: &[u8], mime_type: &str) -> bool {
        self.CanDecode(bytes, mime_type)
    }

    fn create(
        &mut self,
        resource_id: ImageId,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<CreatedDocumentImage> {
        // Creation performs the same isolated Style/Layout/Paint lifecycle as
        // later animation mutations and may allocate native ComputedStyle.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        let mut image = self.PrepareDocumentImage(resource_id, &bytes, mime_type, container)?;
        let initial_frame = image.RenderAt(Duration::ZERO)?;
        let effects = image
            .has_active_animation()
            .then_some(DocumentImageEffect::RequestBeginFrame)
            .into_iter()
            .collect();
        Ok(CreatedDocumentImage {
            initial_frame,
            image: Box::new(image),
            effects,
        })
    }
}

#[derive(Default)]
struct AnimationShorthand {
    name: String,
    duration: f64,
    delay: f64,
    iterations: f64,
    easing: String,
}

fn Seconds(value: &str) -> Option<f64> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.trim().parse::<f64>().ok().map(|value| value / 1000.0);
    }
    value
        .strip_suffix('s')
        .and_then(|seconds| seconds.trim().parse::<f64>().ok())
}

fn ParseAnimationShorthand(value: &str) -> Option<AnimationShorthand> {
    let value = value.split(',').next()?.trim();
    if value.eq_ignore_ascii_case("none") {
        return None;
    }
    let mut result = AnimationShorthand {
        duration: 0.0,
        delay: 0.0,
        iterations: 1.0,
        easing: "ease".into(),
        ..Default::default()
    };
    let mut time_count = 0;
    for token in value.split_ascii_whitespace() {
        if let Some(seconds) = Seconds(token) {
            if time_count == 0 {
                result.duration = seconds.max(0.0);
            } else if time_count == 1 {
                result.delay = seconds;
            }
            time_count += 1;
        } else if token.eq_ignore_ascii_case("infinite") {
            result.iterations = f64::INFINITY;
        } else if let Ok(iterations) = token.parse::<f64>() {
            result.iterations = iterations.max(0.0);
        } else if matches!(
            token.to_ascii_lowercase().as_str(),
            "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
        ) || token.to_ascii_lowercase().starts_with("cubic-bezier(")
            || token.to_ascii_lowercase().starts_with("steps(")
        {
            result.easing = token.to_ascii_lowercase();
        } else if !matches!(
            token.to_ascii_lowercase().as_str(),
            "normal"
                | "reverse"
                | "alternate"
                | "alternate-reverse"
                | "none"
                | "forwards"
                | "backwards"
                | "both"
                | "running"
                | "paused"
        ) {
            result.name = token.to_owned();
        }
    }
    (!result.name.is_empty() && result.duration > 0.0 && result.iterations > 0.0).then_some(result)
}

fn KeyframeOffset(value: &str) -> Option<f64> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "from" => Some(0.0),
        "to" => Some(1.0),
        _ => value
            .strip_suffix('%')
            .and_then(|percent| percent.trim().parse::<f64>().ok())
            .map(|percent| (percent / 100.0).clamp(0.0, 1.0)),
    }
}

fn ExpandKeyframes(rule: &CSSKeyframesRule) -> Vec<TimedKeyframe> {
    let mut result = Vec::new();
    for keyframe in &rule.keyframes {
        for key in keyframe.key_text.split(',') {
            if let Some(offset) = KeyframeOffset(key) {
                result.push(TimedKeyframe {
                    offset,
                    declarations: keyframe.declarations.clone(),
                });
            }
        }
    }
    result.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    result
}

// Chromium SVGImageForContainer supplies the container viewport, DPR and color scheme.
fn ImageMediaData(container: &ContainerKey) -> MediaValuesCachedData {
    let width = container.width as f64;
    let height = container.height as f64;
    MediaValuesCachedData {
        viewport_width: width,
        viewport_height: height,
        small_viewport_width: width,
        small_viewport_height: height,
        large_viewport_width: width,
        large_viewport_height: height,
        dynamic_viewport_width: width,
        dynamic_viewport_height: height,
        device_width: container.width as i32,
        device_height: container.height as i32,
        device_pixel_ratio: container.device_pixel_ratio() as f32,
        media_type: foundation::String::from("screen"),
        preferred_color_scheme: PreferredScheme(container.preferred_color_scheme),
        scripting: style::media_queries::scripting::Scripting::kNone,
        ..Default::default()
    }
}

fn CollectAnimations(
    document: &PersistentDocument,
    environment: &MediaValuesCached,
) -> io::Result<Vec<CSSImageAnimation>> {
    let mut definitions = HashMap::new();
    let mut keyframe_rule_count = 0;
    for sheet in document.ActiveStyleSheets() {
        for rule in &sheet.keyframes {
            keyframe_rule_count += 1;
            if rule.media_conditions.iter().all(|condition| {
                MediaQueryEvaluator::ForMediaValues(environment)
                    .Eval(&ParseMediaQuerySet(condition))
            }) {
                definitions.insert(rule.name.clone(), ExpandKeyframes(rule));
            }
        }
    }
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() && keyframe_rule_count != 0 {
        eprintln!(
            "svg-animation-profile keyframes={} definitions={}",
            keyframe_rule_count,
            definitions.len()
        );
    }
    if definitions.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut selector_service = style::persistent_selector::PersistentSelectorService::default();
    let mut effect_id = 1;
    for index in 0..document.NodeCount() {
        if document.Node(index).Type() != dom::persistent_document::DOMNodeType::kElement {
            continue;
        }
        let mut shorthand = None;
        for sheet in document.ActiveStyleSheets() {
            for rule in &sheet.rules {
                if !rule.media_conditions.iter().all(|condition| {
                    MediaQueryEvaluator::ForMediaValues(environment)
                        .Eval(&ParseMediaQuerySet(condition))
                }) || !selector_service
                    .Matches(document, index, &rule.selector_text)
                    .map_err(|error| {
                        io::Error::new(
                            io::ErrorKind::Unsupported,
                            format!("SVG animation selector: {error:?}"),
                        )
                    })?
                {
                    continue;
                }
                for declaration in &rule.declarations {
                    if declaration.property.eq_ignore_ascii_case("animation")
                        || declaration
                            .property
                            .eq_ignore_ascii_case("-webkit-animation")
                    {
                        shorthand = ParseAnimationShorthand(&declaration.value);
                    }
                }
            }
        }
        let Some(animation) = shorthand else { continue };
        let Some(keyframes) = definitions
            .get(&animation.name)
            .filter(|frames| !frames.is_empty())
        else {
            continue;
        };
        result.push(CSSImageAnimation {
            node_id: document.Node(index).Id(),
            effect_id,
            duration: animation.duration,
            delay: animation.delay,
            iterations: animation.iterations,
            easing: animation.easing,
            keyframes: keyframes.clone(),
        });
        effect_id += 1;
    }
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() && keyframe_rule_count != 0 {
        eprintln!("svg-animation-profile targets={}", result.len());
    }
    Ok(result)
}

fn Cubic(t: f64, a: f64, b: f64) -> f64 {
    3.0 * (1.0 - t) * (1.0 - t) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
}

fn Ease(value: f64, easing: &str) -> f64 {
    let points = match easing {
        "ease" => Some((0.25, 0.1, 0.25, 1.0)),
        "ease-in" => Some((0.42, 0.0, 1.0, 1.0)),
        "ease-out" => Some((0.0, 0.0, 0.58, 1.0)),
        "ease-in-out" => Some((0.42, 0.0, 0.58, 1.0)),
        "step-start" => return 1.0,
        "step-end" => return if value < 1.0 { 0.0 } else { 1.0 },
        _ => None,
    };
    let Some((x1, y1, x2, y2)) = points else {
        return value;
    };
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..24 {
        let middle = (low + high) * 0.5;
        if Cubic(middle, x1, x2) < value {
            low = middle;
        } else {
            high = middle;
        }
    }
    Cubic((low + high) * 0.5, y1, y2)
}

fn NumericParts(value: &str) -> (Vec<f64>, Vec<String>) {
    let bytes = value.as_bytes();
    let mut numbers = Vec::new();
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut text = 0;
    while cursor < bytes.len() {
        let start = cursor;
        if matches!(bytes[cursor], b'+' | b'-') {
            cursor += 1;
        }
        let digits = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'.' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
        }
        if cursor > digits && value[start..cursor].parse::<f64>().is_ok() {
            parts.push(value[text..start].to_owned());
            numbers.push(value[start..cursor].parse().unwrap());
            text = cursor;
        } else {
            cursor = start + 1;
        }
    }
    parts.push(value[text..].to_owned());
    (numbers, parts)
}

fn CanonicalPart(value: &str) -> String {
    value.replace("px", "").replace("deg", "")
}

fn Interpolate(a: &str, b: &str, progress: f64) -> String {
    if a == b {
        return a.to_owned();
    }
    let (an, ap) = NumericParts(a);
    let (bn, bp) = NumericParts(b);
    if an.len() != bn.len()
        || an.is_empty()
        || ap.len() != bp.len()
        || !ap
            .iter()
            .zip(&bp)
            .all(|(a, b)| CanonicalPart(a) == CanonicalPart(b))
    {
        return if progress < 0.5 { a } else { b }.to_owned();
    }
    let mut result = String::new();
    for index in 0..an.len() {
        result.push_str(&bp[index]);
        result.push_str(&format!(
            "{:.6}",
            an[index] + (bn[index] - an[index]) * progress
        ));
    }
    result.push_str(bp.last().unwrap());
    result
}

fn SampleAnimation(animation: &CSSImageAnimation, elapsed: f64) -> Vec<CSSDeclaration> {
    let active = elapsed - animation.delay;
    if active < 0.0 {
        return Vec::new();
    }
    if animation.iterations.is_finite() && active >= animation.duration * animation.iterations {
        return Vec::new();
    }
    let progress = Ease(
        (active % animation.duration) / animation.duration,
        &animation.easing,
    );
    let mut properties: BTreeMap<String, Vec<(f64, String, bool)>> = BTreeMap::new();
    for frame in &animation.keyframes {
        for declaration in &frame.declarations {
            properties
                .entry(declaration.property.clone())
                .or_default()
                .push((
                    frame.offset,
                    declaration.value.clone(),
                    declaration.important,
                ));
        }
    }
    properties
        .into_iter()
        .filter_map(|(property, mut values)| {
            values.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut left = &values[0];
            let mut right = values.last().unwrap();
            for pair in values.windows(2) {
                if progress >= pair[0].0 && progress <= pair[1].0 {
                    left = &pair[0];
                    right = &pair[1];
                    break;
                }
            }
            let local = if right.0 == left.0 {
                1.0
            } else {
                ((progress - left.0) / (right.0 - left.0)).clamp(0.0, 1.0)
            };
            Some(CSSDeclaration {
                property,
                value: Interpolate(&left.1, &right.1, local),
                important: left.2 || right.2,
            })
        })
        .collect()
}

// cpp: image_decoder/svg_image_decoder.cc:23-29
fn SVGLowerASCII(input: &str) -> String {
    input.to_ascii_lowercase()
}

// cpp: image_decoder/svg_image_decoder.cc:31-36
fn FindSVG(document: &PersistentDocument, node: usize) -> Option<usize> {
    let node_ref = document.Node(node);
    if node_ref.IsElement("svg", DOMNamespace::kSVG) {
        return Some(node);
    }
    for &child in node_ref.Children() {
        if let Some(result) = FindSVG(document, child) {
            return Some(result);
        }
    }
    None
}

// cpp: image_decoder/svg_image_decoder.cc:38-51
fn NumberAttribute(node: &DOMNode, name: &str) -> Option<f64> {
    let attribute = node.FindAttribute(name)?;
    if attribute.value.is_empty() || attribute.value.contains('%') {
        return None;
    }
    let mut bytes = attribute.value.as_bytes().to_vec();
    bytes.push(0);
    let mut end: *mut c_char = std::ptr::null_mut();
    let start = bytes.as_ptr().cast::<c_char>();
    let value = unsafe { strtod(start, &mut end) };
    if end == start.cast_mut() || !value.is_finite() || value <= 0.0 {
        return None;
    }
    while unsafe { *end } != 0 && (unsafe { *end } as u8).is_ascii_whitespace() {
        end = unsafe { end.add(1) };
    }
    if unsafe { *end } != 0 {
        let suffix = unsafe { std::ffi::CStr::from_ptr(end) };
        if suffix.to_bytes() != b"px" {
            return None;
        }
    }
    Some(value)
}

// cpp: image_decoder/svg_image_decoder.cc:53-64
fn ViewBoxSize(node: &DOMNode) -> Option<(f64, f64)> {
    let attribute = node.FindAttribute("viewBox")?;
    let value = attribute.value.replace(',', " ");
    // istringstream's formatted extraction consumes a numeric prefix and
    // leaves the rest for the next extraction; split + parse would reject it.
    let mut bytes = value.into_bytes();
    bytes.push(0);
    let mut cursor = bytes.as_ptr().cast::<c_char>();
    let mut next_number = || {
        while unsafe { *cursor } != 0 && (unsafe { *cursor } as u8).is_ascii_whitespace() {
            cursor = unsafe { cursor.add(1) };
        }
        let mut end = std::ptr::null_mut();
        let number = unsafe { strtod(cursor, &mut end) };
        if end == cursor.cast_mut() {
            return None;
        }
        cursor = end;
        Some(number)
    };
    let _x = next_number()?;
    let _y = next_number()?;
    let width = next_number()?;
    let height = next_number()?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some((width, height))
}
