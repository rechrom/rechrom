#[path = "../src/test_thread.rs"]
mod test_thread;

use layoutng_assembly::{
    internal::{
        layout_input::{ComputedStyle, Display, ElementData, NodeKind},
        layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree,
        layout_replaced::LayoutInputReplaced,
        layout_view::LayoutView,
    },
    layout_engine::{LayoutEngine, LayoutMutation, LayoutTreeUpdate},
};

fn project_image(
    engine: &mut LayoutEngine,
    natural_width: f64,
    flex_parent: bool,
    fixed_size: bool,
    image_device_pixel_ratio: f64,
    image_resource_maybe_animated: bool,
    image_resource_error: bool,
) {
    engine.ApplyMutation(LayoutMutation::TreeUpdate(
        &mut |tree: &mut LayoutTreeUpdate<'_>| {
            let root_style = ComputedStyle {
                width: Some(320.0),
                height: Some(240.0),
                ..Default::default()
            };
            let parent_style = ComputedStyle {
                display: if flex_parent {
                    Display::kFlex
                } else {
                    Display::kBlock
                },
                width: Some(200.0),
                height: Some(160.0),
                ..Default::default()
            };
            let image_style = ComputedStyle {
                width: fixed_size.then_some(80.0),
                height: fixed_size.then_some(60.0),
                ..Default::default()
            };
            let root = tree.CreateRootDefault(1, &root_style) as *mut LayoutObject;
            let parent =
                tree.AddBoxDefault(unsafe { &mut *root }, 2, &parent_style) as *mut LayoutObject;
            tree.AddReplaced(
                unsafe { &mut *parent },
                3,
                &image_style,
                Some(ElementData {
                    html_image: true,
                    natural_width: Some(natural_width),
                    natural_height: Some(30.0),
                    natural_aspect_ratio: (natural_width > 0.0).then_some(natural_width / 30.0),
                    image_resource_is_loaded: !image_resource_error,
                    image_resource_error,
                    image_resource_known_to_be_opaque: !image_resource_error,
                    image_resource_maybe_animated: !image_resource_error
                        && image_resource_maybe_animated,
                    image_device_pixel_ratio,
                    ..Default::default()
                }),
                "img".into(),
                NodeKind::kReplaced,
            );
        },
    ));
}

#[test]
fn fixed_layout_image_natural_size_change_is_paint_only() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(
            LayoutObjectTree::new_with_factories(&space, &assembly.objects),
        ));

        project_image(&mut engine, 40.0, false, true, 1.0, false, false);
        engine.Layout();
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let image = unsafe { &mut *parent.SlowFirstChild() };
        image.ClearPaintInvalidationFlags();
        project_image(&mut engine, 120.0, false, true, 1.0, false, false);

        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let image = unsafe { &*parent.SlowFirstChild() };
        assert!(image.IsLayoutImage());
        let image = unsafe { &*(image as *const LayoutObject).cast::<LayoutInputReplaced>() };
        assert_eq!(image.GetNaturalDimensions().size.width.ToDouble(), 120.0);
        assert!(!image.NeedsLayout());
        assert_eq!(
            image.PaintInvalidationReasonForPrePaint(),
            foundation::PaintInvalidationReason::kImage
        );
    });
}

#[test]
fn flex_or_auto_sized_layout_image_natural_size_change_requires_layout() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (flex_parent, fixed_size) in [(true, true), (false, false)] {
            let space = browser::CreateBrowserConstraints(320, 240);
            let assembly = browser::CreateLayoutAssembly();
            let mut engine = LayoutEngine::new(&assembly);
            engine.ApplyMutation(LayoutMutation::Constraints(&space));
            engine.ApplyMutation(LayoutMutation::ReplaceTree(
                LayoutObjectTree::new_with_factories(&space, &assembly.objects),
            ));

            project_image(
                &mut engine,
                40.0,
                flex_parent,
                fixed_size,
                1.0,
                false,
                false,
            );
            engine.Layout();
            let root = engine.GetLayoutTree().unwrap().Root();
            let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
            unsafe { &mut *parent.SlowFirstChild() }.ClearPaintInvalidationFlags();
            project_image(
                &mut engine,
                120.0,
                flex_parent,
                fixed_size,
                1.0,
                false,
                false,
            );

            let root = engine.GetLayoutTree().unwrap().Root();
            let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
            let image = unsafe { &*parent.SlowFirstChild() };
            assert!(image.IsLayoutImage());
            assert!(image.NeedsLayout(), "flex={flex_parent} fixed={fixed_size}");
            assert_eq!(
                image.PaintInvalidationReasonForPrePaint(),
                foundation::PaintInvalidationReason::kLayout,
                "flex={flex_parent} fixed={fixed_size}"
            );
        }
    });
}

#[test]
fn retained_layout_image_updates_device_pixel_ratio_member() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(
            LayoutObjectTree::new_with_factories(&space, &assembly.objects),
        ));

        project_image(&mut engine, 40.0, false, true, 1.0, false, false);
        engine.Layout();
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let original = parent.SlowFirstChild();
        let image = unsafe { &*original.cast::<LayoutInputReplaced>() };
        assert_eq!(image.ImageDevicePixelRatio(), 1.0);
        assert_eq!(image.GetNaturalDimensions().size.width.ToDouble(), 40.0);

        project_image(&mut engine, 40.0, false, true, 2.0, false, false);
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let retained = parent.SlowFirstChild();
        assert_eq!(retained, original);
        let image = unsafe { &*retained.cast::<LayoutInputReplaced>() };
        assert_eq!(image.ImageDevicePixelRatio(), 2.0);
        assert_eq!(image.GetNaturalDimensions().size.width.ToDouble(), 40.0);
    });
}

#[test]
fn animated_layout_image_change_delays_full_paint_invalidation() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(
            LayoutObjectTree::new_with_factories(&space, &assembly.objects),
        ));

        project_image(&mut engine, 40.0, false, true, 1.0, false, false);
        engine.Layout();
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        unsafe { &mut *parent.SlowFirstChild() }.ClearPaintInvalidationFlags();

        project_image(&mut engine, 40.0, false, true, 1.0, true, false);
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let image = unsafe { &*parent.SlowFirstChild() };
        assert_eq!(
            image.PaintInvalidationReasonForPrePaint(),
            foundation::PaintInvalidationReason::kImage
        );
        assert!(image.ShouldDelayFullPaintInvalidation());
    });
}

#[test]
fn failed_layout_image_retains_last_successful_natural_dimensions() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(
            LayoutObjectTree::new_with_factories(&space, &assembly.objects),
        ));

        project_image(&mut engine, 40.0, false, true, 1.0, false, false);
        engine.Layout();
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        unsafe { &mut *parent.SlowFirstChild() }.ClearPaintInvalidationFlags();

        project_image(&mut engine, 0.0, false, true, 1.0, false, true);
        let root = engine.GetLayoutTree().unwrap().Root();
        let parent = unsafe { &*unsafe { &*root }.SlowFirstChild() };
        let image = unsafe { &*parent.SlowFirstChild().cast::<LayoutInputReplaced>() };
        assert_eq!(image.GetNaturalDimensions().size.width.ToDouble(), 40.0);
        assert!(!image.NeedsLayout());
        assert_eq!(
            image.PaintInvalidationReasonForPrePaint(),
            foundation::PaintInvalidationReason::kImage
        );
    });
}

#[test]
fn layout_image_contributes_visually_non_empty_pixels_only_once() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(&space));
        engine.ApplyMutation(LayoutMutation::ReplaceTree(
            LayoutObjectTree::new_with_factories(&space, &assembly.objects),
        ));

        project_image(&mut engine, 10.0, false, true, 1.0, false, false);
        let root = engine.GetLayoutTree().unwrap().Root();
        let view = unsafe { &*(root as *const LayoutObject).cast::<LayoutView>() };
        assert_eq!(view.VisuallyNonEmptyPixelCount(), 300);
        assert!(!view.IsVisuallyNonEmpty());

        project_image(&mut engine, 100.0, false, true, 1.0, false, false);
        let root = engine.GetLayoutTree().unwrap().Root();
        let view = unsafe { &*(root as *const LayoutObject).cast::<LayoutView>() };
        assert_eq!(view.VisuallyNonEmptyPixelCount(), 300);
        assert!(!view.IsVisuallyNonEmpty());
    });
}
