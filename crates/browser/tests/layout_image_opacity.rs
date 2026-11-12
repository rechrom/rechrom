//! LayoutImage foreground-opacity dispatch through DOM, Layout and Paint state.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use foundation::{DynamicTo, RespectImageOrientationEnum};
use layoutng_assembly::internal::{
    hit_test_phase::HitTestPhase,
    layout_box::LayoutBox,
    layout_input::FontFace,
    layout_object::{LayoutObject, StyleChangeContext},
    layout_pass_scope::LayoutObjectFactoryScope,
    layout_replaced::{LayoutInputReplaced, LayoutReplaced},
    loader::fetch::resource_priority::VisibilityStatus,
    selection_state::SelectionState,
};
use layoutng_style::style::{
    computed_style::ComputedStyleBuilder, style_difference::StyleDifference,
};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    html: Vec<u8>,
    image: Vec<u8>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let (mime_type, body) = if request.url.ends_with("fixture.png") {
            ("image/png", self.image.clone())
        } else {
            ("text/html", self.html.clone())
        };
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: mime_type.into(),
            body,
            ..Default::default()
        }))))
    }
}

fn fixture_with_attributes(image: &[u8], extra_style: &str, attributes: &str) -> Page {
    let html = format!(
        "<!doctype html><style>body{{margin:0}}img{{display:block;width:80px;height:80px;background:red;{extra_style}}}</style><img id=target src=fixture.png {attributes}>"
    );
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(160, 120);
    constraints.fonts = vec![FontFace {
        family: "LayoutImageOpacity".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let svg = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: html.into_bytes(),
            image: image.to_vec(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(svg)),
        constraints,
        None,
        None,
    );
    page.Open("https://layout-image-opacity.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn fixture(image: &[u8], extra_style: &str) -> Page {
    fixture_with_attributes(image, extra_style, "")
}

fn rounded_box_fixture() -> Page {
    let html = br#"<!doctype html><style>
      body{margin:0}
      #target{width:100px;height:100px;overflow:hidden;border:1px solid black;border-radius:8px;background:red}
      #child{width:100%;height:100%;background:black}
      </style><div id=target><div id=child></div></div>"#;
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(160, 120);
    constraints.fonts = vec![FontFace {
        family: "LayoutBoxBackground".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let svg = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: html.to_vec(),
            image: Vec::new(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(svg)),
        constraints,
        None,
        None,
    );
    page.Open("https://layout-box-background.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn layout_box_by_id<'a>(page: &'a Page, id: &str) -> &'a LayoutBox {
    let owner = page.Document();
    let document = owner.GetDocument();
    let wanted = (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == id)
                .map(|_| document.Node(index).Id())
        })
        .expect("wanted node");
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            assert!(current.IsBox());
            return unsafe { &*object.cast::<LayoutBox>() };
        }
        object = current.NextInPreOrder(root);
    }
    panic!("LayoutBox for {id} missing")
}

fn image_box_ptr(page: &Page) -> *mut LayoutReplaced {
    let owner = page.Document();
    let document = owner.GetDocument();
    let wanted = (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == "target")
                .map(|_| document.Node(index).Id())
        })
        .expect("target node");
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            assert!(current.IsLayoutImage());
            let replaced = DynamicTo::<LayoutReplaced>(object.cast_mut());
            assert!(!replaced.is_null());
            return replaced;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("target LayoutImage missing")
}

fn image_box(page: &Page) -> &LayoutReplaced {
    unsafe { &*image_box_ptr(page) }
}

fn image_background_is_recorded(page: &Page) -> bool {
    let node = image_box(page).GetNode();
    assert!(!node.is_null());
    let node_id = unsafe { &*node }.InputId();
    page.CurrentFrame()
        .unwrap()
        .display_items
        .items
        .iter()
        .any(|item| {
            item.node_id == node_id
                && item.r#type == paint::paint_engine::DisplayItemType::kDrawRect
                && item.color.red == 1.0
                && item.color.green == 0.0
                && item.color.blue == 0.0
                && item.color.alpha == 1.0
        })
}

fn assert_replaced_rect(page: &Page, expected: [f64; 4]) {
    let rect = image_box(page).ReplacedContentRect();
    let actual = [
        rect.X().ToDouble(),
        rect.Y().ToDouble(),
        rect.Width().ToDouble(),
        rect.Height().ToDouble(),
    ];
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.06,
            "replaced rect mismatch: actual={actual}, expected={expected}"
        );
    }
}

fn assert_rect(actual: foundation::PhysicalRect, expected: [f64; 4]) {
    let actual = [
        actual.X().ToDouble(),
        actual.Y().ToDouble(),
        actual.Width().ToDouble(),
        actual.Height().ToDouble(),
    ];
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.06,
            "rect mismatch: actual={actual}, expected={expected}"
        );
    }
}

#[test]
fn layout_image_opaque_foreground_matches_loaded_pixels_and_style_coverage() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let opaque =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/paris_icc_exif_xmp.png");
        let alpha =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/alpha-libyuv.png");

        let page = fixture(opaque, "");
        assert!(image_box(&page).ComputeBackgroundIsKnownToBeObscured());

        let page = fixture(alpha, "");
        assert!(!image_box(&page).ComputeBackgroundIsKnownToBeObscured());

        let page = fixture(opaque, "padding:1px");
        assert!(!image_box(&page).ComputeBackgroundIsKnownToBeObscured());

        let page = fixture(opaque, "object-fit:contain");
        assert!(!image_box(&page).ComputeBackgroundIsKnownToBeObscured());

        let page = fixture(opaque, "border:1px solid black");
        let box_ = image_box(&page);
        assert!(box_.ForegroundIsKnownToBeOpaqueInRect(&box_.PhysicalContentBoxRect(), 0));

        // BorderEdge::EffectiveStyle converts a too-thin double border to
        // solid, while a real double border and a dashed border leave gaps.
        let page = fixture(opaque, "border:1px double black");
        let box_ = image_box(&page);
        assert!(box_.ForegroundIsKnownToBeOpaqueInRect(&box_.PhysicalContentBoxRect(), 0));

        let page = fixture(opaque, "border:3px double black");
        let box_ = image_box(&page);
        assert!(!box_.ForegroundIsKnownToBeOpaqueInRect(&box_.PhysicalContentBoxRect(), 0));

        let page = fixture(opaque, "border:1px dashed black");
        let box_ = image_box(&page);
        assert!(!box_.ForegroundIsKnownToBeOpaqueInRect(&box_.PhysicalContentBoxRect(), 0));
    });
}

#[test]
fn paint_skips_only_a_layout_image_background_known_to_be_obscured() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let opaque =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/paris_icc_exif_xmp.png");
        let alpha =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/alpha-libyuv.png");

        let opaque_page = fixture(opaque, "");
        assert!(image_box(&opaque_page).ComputeBackgroundIsKnownToBeObscured());
        assert!(!image_background_is_recorded(&opaque_page));

        let alpha_page = fixture(alpha, "");
        assert!(!image_box(&alpha_page).ComputeBackgroundIsKnownToBeObscured());
        assert!(image_background_is_recorded(&alpha_page));
    });
}

#[test]
fn rounded_box_background_is_skipped_when_opaque_child_obscures_it() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = rounded_box_fixture();
        let target = layout_box_by_id(&page, "target");
        assert!(target.ComputeBackgroundIsKnownToBeObscured());

        let node = target.GetNode();
        assert!(!node.is_null());
        let node_id = unsafe { &*node }.InputId();
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| {
                item.node_id == node_id
                    && item.color.red == 1.0
                    && item.color.green == 0.0
                    && item.color.blue == 0.0
                    && item.color.alpha == 1.0
            }));
    });
}

#[test]
fn layout_image_replaced_content_matches_object_fit_position_and_clip_contract() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
        let image =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/paris_icc_exif_xmp.png");

        let page = fixture(image, "");
        assert_replaced_rect(&page, [0.0, 0.0, 80.0, 80.0]);
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        assert_eq!(input.GetName(), "LayoutImage");
        let object: &LayoutObject = input;
        assert_eq!(
            DynamicTo::<LayoutInputReplaced>(object as *const LayoutObject).cast_const(),
            input as *const LayoutInputReplaced
        );
        assert_eq!(object.GetName(), "LayoutImage");
        assert_eq!(input.ImageDevicePixelRatio(), 1.0);
        assert!(!input.IsGeneratedContent());
        input.SetIsGeneratedContent(true);
        assert!(input.IsGeneratedContent());
        input.SetIsGeneratedContent(false);
        assert!(!input.DrawsBackgroundOntoContentLayer());
        assert!(input.IsInSelfHitTestingPhase(HitTestPhase::kForeground));
        assert!(input.IsInSelfHitTestingPhase(HitTestPhase::kSelfBlockBackground));
        assert!(!input.IsInSelfHitTestingPhase(HitTestPhase::kFloat));
        assert_eq!(input.CachedResourcePriority(), None);
        let priority = input.ComputeResourcePriority();
        assert_eq!(priority.visibility, VisibilityStatus::kVisible);
        assert_eq!(priority.intra_priority_value, 80 * 80);
        assert_eq!(input.CachedResourcePriority(), Some(priority));
        assert_eq!(
            input.CachedSpeculativeDecodeSize(),
            foundation::gfx::Size::default()
        );
        assert_eq!(
            input.ComputeSpeculativeDecodeSize(),
            foundation::gfx::Size::new(80, 80)
        );
        assert_eq!(
            input.CachedSpeculativeDecodeSize(),
            foundation::gfx::Size::new(80, 80)
        );
        let quality = input.ComputeSpeculativeDecodeQuality();
        assert_eq!(quality, input.StyleRef().GetInterpolationQuality());
        assert_eq!(input.CachedSpeculativeDecodeQuality(), quality);
        assert!(!input.IsUnsizedImage());
        assert!(image_box(&page).RespectsCSSOverflow());
        assert!(image_box(&page).ClipsToContentBox());
        let replaced = unsafe { &mut *image_box_ptr(&page) };
        assert_rect(replaced.LocalSelectionVisualRect(), [0.0, 0.0, 0.0, 0.0]);
        replaced.SetSelectionState(SelectionState::kStart);
        assert_rect(replaced.LocalSelectionVisualRect(), [0.0, 0.0, 80.0, 80.0]);

        let page = fixture(image, "display:inline");
        let replaced = unsafe { &mut *image_box_ptr(&page) };
        replaced.SetSelectionState(SelectionState::kStart);
        let inline_selection = replaced.LocalSelectionVisualRect();
        assert_eq!(inline_selection.Width().ToDouble(), 80.0);
        assert!(inline_selection.Height().ToDouble() >= 80.0);

        let page = fixture(image, "margin-left:200px");
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        let priority = input.ComputeResourcePriority();
        assert_eq!(priority.visibility, VisibilityStatus::kNotVisible);
        assert_eq!(priority.intra_priority_value, 0);

        let page = fixture(image, "width:0;height:0");
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        let priority = input.ComputeResourcePriority();
        assert_eq!(priority.visibility, VisibilityStatus::kVisible);
        assert_eq!(priority.intra_priority_value, 0);

        let page = fixture(image, "width:auto;height:auto");
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        assert!(input.IsUnsizedImage());

        let page = fixture(image, "width:80px;height:auto;aspect-ratio:2/1");
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        assert!(!input.IsUnsizedImage());

        let page = fixture(
            image,
            "width:300px;height:300px;object-fit:scale-down;object-position:0 0",
        );
        assert_replaced_rect(&page, [0.0, 0.0, 300.0, 300.0 * 302.0 / 403.0]);
        let input = unsafe { &*image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        input.SetImageDevicePixelRatio(2.0);
        assert_replaced_rect(&page, [0.0, 0.0, 403.0 / 2.0, 302.0 / 2.0]);

        let page = fixture(image, "overflow:visible");
        assert!(!image_box(&page).ClipsToContentBox());

        let page = fixture(image, "overflow:hidden");
        assert!(image_box(&page).ClipsToContentBox());

        let contained_height = 80.0 * 302.0 / 403.0;
        let page = fixture(image, "object-fit:contain;object-position:0 0");
        assert_replaced_rect(&page, [0.0, 0.0, 80.0, contained_height]);

        // HTML width/height attributes participate in presentational sizing,
        // but a decoded LayoutImage still uses ImageResource dimensions for
        // object-fit.  The 1:1 attributes must not erase the 403:302 ratio.
        let page = fixture_with_attributes(
            image,
            "object-fit:contain;object-position:50% 50%",
            "width=200 height=200",
        );
        assert_replaced_rect(
            &page,
            [0.0, (80.0 - contained_height) * 0.5, 80.0, contained_height],
        );

        let page = fixture(image, "object-fit:contain;object-position:25% 75%");
        assert_replaced_rect(
            &page,
            [
                0.0,
                (80.0 - contained_height) * 0.75,
                80.0,
                contained_height,
            ],
        );

        let page = fixture(image, "object-fit:cover;object-position:0 0");
        assert_replaced_rect(&page, [0.0, 0.0, 80.0 * 403.0 / 302.0, 80.0]);

        let page = fixture(image, "object-fit:none;object-position:0 0");
        assert_replaced_rect(&page, [0.0, 0.0, 403.0, 302.0]);

        let page = fixture(image, "object-fit:scale-down;object-position:0 0");
        assert_replaced_rect(&page, [0.0, 0.0, 80.0, contained_height]);

        for view_box in [
            "inset(10px 20px)",
            "rect(10px 383px 292px 20px)",
            "xywh(20px 10px 363px 282px)",
        ] {
            let page = fixture(image, &format!("object-view-box:{view_box}"));
            assert_replaced_rect(
                &page,
                [
                    -20.0 * 80.0 / 363.0,
                    -10.0 * 80.0 / 282.0,
                    403.0 * 80.0 / 363.0,
                    302.0 * 80.0 / 282.0,
                ],
            );
        }
    });
}

#[test]
fn layout_image_style_changes_follow_replaced_lifecycle() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let image =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/paris_icc_exif_xmp.png");

        let page = fixture(image, "");
        let assembly = browser::CreateLayoutAssembly();
        let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
        let replaced = unsafe { &mut *image_box_ptr(&page) };
        let natural_size =
            unsafe { &*(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() }
                .GetNaturalDimensions()
                .size;
        assert!(!replaced.NeedsPaintPropertyUpdate());
        let style = replaced.StylePtr();
        let mut radius_diff = StyleDifference::default();
        radius_diff.set_border_radius_changed(true);
        replaced.StyleDidChange(
            radius_diff,
            style,
            unsafe { &*style },
            &StyleChangeContext::default(),
        );
        assert!(replaced.NeedsPaintPropertyUpdate());

        replaced.ClearNeedsLayoutWithoutPaintInvalidation();
        replaced.ClearIntrinsicLogicalWidthsDirty();
        let old_style = replaced.StylePtr();
        let mut zoomed = ComputedStyleBuilder::from_style(unsafe { &*old_style });
        assert!(zoomed.SetEffectiveZoom(unsafe { &*old_style }.EffectiveZoom() * 2.0));
        replaced.SetStyle(zoomed.TakeStyle());
        let zoomed_natural_size =
            unsafe { &*(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() }
                .GetNaturalDimensions()
                .size;
        assert_eq!(zoomed_natural_size.width, natural_size.width * 2);
        assert_eq!(zoomed_natural_size.height, natural_size.height * 2);
        assert!(replaced.SelfNeedsFullLayout());
        assert!(replaced.IntrinsicLogicalWidthsDirty());

        let page = fixture(image, "");
        let input = unsafe { &mut *image_box_ptr(&page).cast::<LayoutInputReplaced>() };
        input.ClearNeedsLayoutWithoutPaintInvalidation();
        input.ClearIntrinsicLogicalWidthsDirty();
        let object: &LayoutObject = input;
        object
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        let mut oriented = ComputedStyleBuilder::from_style(input.StyleRef());
        oriented.SetImageOrientation(RespectImageOrientationEnum::kDoNotRespectImageOrientation);
        input.SetStyle(oriented.TakeStyle());
        assert!(input.NeedsPaintPropertyUpdate());
        assert!(input.SelfNeedsFullLayout());
        assert!(input.IntrinsicLogicalWidthsDirty());
    });
}

#[test]
fn layout_image_visual_overflow_matches_replaced_effect_geometry() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let image =
            include_bytes!("../../image_decoder/tests/fixtures/avif-source/paris_icc_exif_xmp.png");
        let assembly = browser::CreateLayoutAssembly();
        let _factories = LayoutObjectFactoryScope::new(&assembly.objects);

        for (style, expected) in [
            ("", [0.0, 0.0, 80.0, 80.0]),
            ("box-shadow:10px 12px 0 3px black", [0.0, 0.0, 93.0, 95.0]),
            (
                "outline:4px solid black;outline-offset:2px",
                [-6.0, -6.0, 92.0, 92.0],
            ),
            (
                "object-fit:none;object-position:0 0;overflow:visible",
                [0.0, 0.0, 403.0, 302.0],
            ),
            (
                "object-fit:none;object-position:0 0;overflow:hidden",
                [0.0, 0.0, 80.0, 80.0],
            ),
        ] {
            let page = fixture(image, style);
            let replaced = image_box_ptr(&page);
            unsafe { &mut *replaced.cast::<LayoutObject>() }.RecalcVisualOverflow();
            assert_rect(unsafe { &*replaced }.VisualOverflowRect(), expected);
        }
    });
}
