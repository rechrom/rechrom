//! HTML image-map focus painting through DOM, LayoutImage and ImagePainter.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use dom::UserInteractionState;
use layoutng_assembly::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle},
    internal::{
        layout_box::LayoutBox, layout_input::FontFace, layout_object::LayoutObject,
        layout_pass_scope::LayoutObjectFactoryScope,
    },
    layout_engine::PaintFragmentCursor,
};
use page_mutation::{InteractionStateMutation, PageMutation};
use paint::paint_engine::{DisplayItemType, PaintRect};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader(Vec<u8>);
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.clone(),
            ..Default::default()
        }))))
    }
}

fn fixture() -> Page {
    let html = br##"<!doctype html><style>
        body { margin: 0 }
        img { display:block; width:100px; height:80px; border:2px solid; padding:3px }
        area:focus-visible { outline: 1px auto rgb(10,20,30) }
      </style>
      <img id=image usemap="#map">
      <img id=image2 usemap="#map">
      <canvas id=canvas></canvas>
      <map name=map>
        <area id=area shape=rect coords="10,20,30,40" href="#">
        <area id=empty-area shape=circle coords="10,20,0" href="#">
      </map>"##;
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(180, 140);
    constraints.fonts = vec![FontFace {
        family: "ImageMapFocusRing".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let svg = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader(html.to_vec()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(svg)),
        constraints,
        None,
        None,
    );
    page.Open("https://image-map-focus.test/", 16 * 1024, 4096)
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

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| document.Node(index).Id())
        })
        .unwrap()
}

fn native<'a>(page: &'a Page, name: &str) -> &'a LayoutObject {
    let wanted = id(page, name);
    let assembly = browser::CreateLayoutAssembly();
    let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            return current;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("missing native layout object {name}")
}

fn find_fragment(handle: &LayoutResultHandle, wanted: u64) -> Option<&FragmentPaintData> {
    fn find(node: PaintFragmentCursor<'_>, wanted: u64) -> Option<PaintFragmentCursor<'_>> {
        if node.PaintData().node_id == wanted {
            Some(node)
        } else {
            node.Children()
                .into_iter()
                .find_map(|child| find(child, wanted))
        }
    }
    find(PaintFragmentCursor::FromHandle(handle), wanted).map(PaintFragmentCursor::PaintData)
}

fn set_attribute(page: &mut Page, name: &str, attribute: &str, value: &str) {
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: id(page, name),
        name: attribute.into(),
        value: value.into(),
        ..Default::default()
    }))
    .unwrap();
}

#[test]
fn focused_area_records_clipped_image_area_focus_path() {
    test_thread::run(|| {
        let mut page = fixture();
        let area = id(&page, "area");
        let image = id(&page, "image");
        let second_image = id(&page, "image2");
        assert!(native(&page, "image").CanBeSelectionLeafInternal());
        assert!(!native(&page, "canvas").CanBeSelectionLeafInternal());
        let image_box =
            unsafe { &*(native(&page, "image") as *const LayoutObject).cast::<LayoutBox>() };
        let canvas_box =
            unsafe { &*(native(&page, "canvas") as *const LayoutObject).cast::<LayoutBox>() };
        assert!(image_box.BackgroundShouldAlwaysBeClipped());
        assert!(!canvas_box.BackgroundShouldAlwaysBeClipped());
        assert_eq!(
            page.Document().GetDocument().ImageMapImageForAreaId(area),
            Some(image)
        );
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath));

        page.Apply(PageMutation::InteractionStateMutation(
            InteractionStateMutation {
                state: UserInteractionState {
                    focused_node_id: Some(area),
                    focus_visible_node_id: Some(area),
                    ..Default::default()
                },
            },
        ))
        .unwrap();

        let owner = page.Document();
        let document = owner.GetDocument();
        let area_index = document.FindNodeById(area).unwrap();
        let area_style = document
            .ResolvedStyleFor(area_index)
            .expect("focused display:none area must have ensured style");
        let area_source = unsafe { area_style.computed_style_owner.Get().as_ref() }
            .expect("focused area native computed style");
        assert!(area_source.HasOutline());
        assert!(area_source.OutlineStyleIsAuto());
        drop(owner);
        let image_input = unsafe {
            &*native(&page, "image")
                .GetNode()
                .cast::<layoutng_assembly::internal::layout_node_metadata::Element>()
        }
        .InputElementData()
        .as_ref()
        .unwrap();
        assert!(
            image_input.focused_image_map_area.is_some(),
            "focused area must reach LayoutImage input"
        );
        let image_fragment =
            find_fragment(&page.CurrentFrame().unwrap().fragments, image).expect("image fragment");
        assert!(
            image_fragment.paint.focused_image_map_area.is_some(),
            "focused area must reach LayoutImage fragment"
        );

        let items = &page.CurrentFrame().unwrap().display_items.items;
        let stroke = items
            .iter()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath)
            .expect("focused image-map area path");
        assert_eq!(stroke.stroke_width, 3.0);
        assert_eq!(stroke.corner_radius, 3.0);
        assert_eq!(stroke.path.len(), 5);
        assert_eq!(stroke.path[0].point.x, 10.0);
        assert_eq!(stroke.path[0].point.y, 20.0);
        assert_eq!(stroke.path[2].point.x, 30.0);
        assert_eq!(stroke.path[2].point.y, 40.0);
        let stroke_position = items
            .iter()
            .position(|item| std::ptr::eq(item, stroke))
            .unwrap();
        let clip = items[..stroke_position]
            .iter()
            .rev()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kClipRect)
            .expect("content-box focus clip");
        assert_eq!(
            clip.rect,
            PaintRect {
                x: 5.0,
                y: 5.0,
                width: 100.0,
                height: 80.0,
            }
        );
        assert!(!items.iter().any(|item| {
            item.node_id == second_image && item.r#type == DisplayItemType::kStrokePath
        }));

        set_attribute(&mut page, "area", "coords", "12,22,32,42");
        let changed = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath)
            .expect("changed image-map area path");
        assert_eq!(changed.path[0].point.x, 12.0);
        assert_eq!(changed.path[0].point.y, 22.0);
        assert_eq!(changed.path[2].point.x, 32.0);
        assert_eq!(changed.path[2].point.y, 42.0);

        set_attribute(&mut page, "area", "shape", "circle");
        set_attribute(&mut page, "area", "coords", "50,40,10");
        let circle = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath)
            .expect("circle image-map area path");
        assert_eq!(circle.path.len(), 6);
        assert_eq!(circle.path[0].point.x, 60.0);
        assert_eq!(circle.path[0].point.y, 40.0);

        set_attribute(&mut page, "area", "shape", "poly");
        set_attribute(&mut page, "area", "coords", "5,6,15,16,25,6");
        let polygon = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath)
            .expect("polygon image-map area path");
        assert_eq!(polygon.path.len(), 4);
        assert!(polygon.even_odd);
        assert_eq!(polygon.path[2].point.x, 25.0);
        assert_eq!(polygon.path[2].point.y, 6.0);

        set_attribute(&mut page, "area", "shape", "default");
        let default_shape = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath)
            .expect("default image-map area path");
        assert_eq!(default_shape.path.len(), 5);
        assert_eq!(default_shape.path[0].point.x, 0.0);
        assert_eq!(default_shape.path[0].point.y, 0.0);
        assert_eq!(default_shape.path[2].point.x, 110.0);
        assert_eq!(default_shape.path[2].point.y, 90.0);

        set_attribute(
            &mut page,
            "area",
            "style",
            "outline: 1px solid rgb(10,20,30)",
        );
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath));
        set_attribute(
            &mut page,
            "area",
            "style",
            "outline: 1px auto rgb(10,20,30)",
        );
        assert!(page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath));

        set_attribute(&mut page, "image", "usemap", "#other");
        let reassociated = &page.CurrentFrame().unwrap().display_items.items;
        assert!(!reassociated
            .iter()
            .any(|item| { item.node_id == image && item.r#type == DisplayItemType::kStrokePath }));
        assert!(reassociated.iter().any(|item| {
            item.node_id == second_image && item.r#type == DisplayItemType::kStrokePath
        }));
        set_attribute(&mut page, "image", "usemap", "#map");
        let restored = &page.CurrentFrame().unwrap().display_items.items;
        assert!(restored
            .iter()
            .any(|item| { item.node_id == image && item.r#type == DisplayItemType::kStrokePath }));
        assert!(!restored.iter().any(|item| {
            item.node_id == second_image && item.r#type == DisplayItemType::kStrokePath
        }));

        page.Apply(PageMutation::InteractionStateMutation(
            InteractionStateMutation::default(),
        ))
        .unwrap();
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath));

        let empty_area = id(&page, "empty-area");
        page.Apply(PageMutation::InteractionStateMutation(
            InteractionStateMutation {
                state: UserInteractionState {
                    focused_node_id: Some(empty_area),
                    focus_visible_node_id: Some(empty_area),
                    ..Default::default()
                },
            },
        ))
        .unwrap();
        let image_input = unsafe {
            &*native(&page, "image")
                .GetNode()
                .cast::<layoutng_assembly::internal::layout_node_metadata::Element>()
        }
        .InputElementData()
        .as_ref()
        .unwrap();
        assert!(image_input.focused_image_map_area.is_none());
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .any(|item| item.node_id == image && item.r#type == DisplayItemType::kStrokePath));
    });
}
