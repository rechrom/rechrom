//! Exercise fill-layer finalization through production StyleEngine, rather than
//! manually invoking the already-translated FillLayer normalization helpers.
use dom::{persistent_document::DOMAttribute, DOM};
use foundation::{BlendMode, Persistent};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_constants::{BackgroundEdgeOrigin, EFillAttachment, EFillBox, EFillRepeat},
    fill_layer::FillLayer,
};
use style::{media_queries::MediaValuesCachedData, StyleEngine};

fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: "screen".into(),
        viewport_width: 320.0,
        viewport_height: 240.0,
        large_viewport_width: 320.0,
        large_viewport_height: 240.0,
        ..Default::default()
    }
}
fn node(owner: &DOM, name: &str) -> usize {
    let doc = owner.GetDocument();
    (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == name)
        })
        .unwrap()
}
fn native(owner: &DOM, name: &str) -> Persistent<ComputedStyle> {
    owner
        .GetDocument()
        .ResolvedStyleFor(node(owner, name))
        .unwrap()
        .native_style
        .clone()
}
fn layers(first: &FillLayer) -> Vec<&FillLayer> {
    let mut result = vec![first];
    while let Some(next) = unsafe { result.last().unwrap().Next().as_ref() } {
        result.push(next);
    }
    result
}
fn set_style(owner: &mut DOM, name: &str, css: &str) {
    let index = node(owner, name);
    owner.GetDocumentMut().SetAttribute(
        index,
        DOMAttribute {
            local_name: "style".into(),
            value: css.into(),
            ..Default::default()
        },
    );
}
const IMAGES: &str = "linear-gradient(transparent,transparent),url(sprite.svg),none";

#[test]
fn background_lists_cycle_all_native_fill_fields_in_production() {
    let _heap = foundation::LayoutHeapScope::new();
    let css = format!(
        "background-image:{IMAGES};background-position:right 2px bottom 3px,left 7px top 8px;\
        background-size:20px 40px,30px 60px;background-repeat:no-repeat,repeat-x;\
        background-origin:content-box,padding-box;background-clip:padding-box,content-box;\
        background-attachment:local,fixed;background-blend-mode:multiply,screen"
    );
    let mut owner = html::html_parser::ParseHTML(&format!(
        "<!doctype html><div id=a style='{css}'></div><div id=b style='{css}'></div>"
    ));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    for id in ["a", "b"] {
        let style = native(&owner, id);
        let items = layers(unsafe { &*style.Get() }.BackgroundLayers());
        assert_eq!(items.len(), 3);
        for (i, item) in items.iter().enumerate() {
            let even = i % 2 == 0;
            assert_eq!(
                item.PositionX().Pixels(),
                if even { 2.0 } else { 7.0 },
                "{id} layer {i} x"
            );
            assert_eq!(
                item.PositionY().Pixels(),
                if even { 3.0 } else { 8.0 },
                "{id} layer {i} y"
            );
            assert_eq!(
                item.BackgroundXOrigin(),
                if even {
                    BackgroundEdgeOrigin::kRight
                } else {
                    BackgroundEdgeOrigin::kLeft
                }
            );
            assert_eq!(
                item.BackgroundYOrigin(),
                if even {
                    BackgroundEdgeOrigin::kBottom
                } else {
                    BackgroundEdgeOrigin::kTop
                }
            );
            assert_eq!(
                item.SizeLength().Width().Pixels(),
                if even { 20.0 } else { 30.0 }
            );
            assert_eq!(
                item.SizeLength().Height().Pixels(),
                if even { 40.0 } else { 60.0 }
            );
            assert_eq!(
                item.Repeat().x,
                if even {
                    EFillRepeat::kNoRepeatFill
                } else {
                    EFillRepeat::kRepeatFill
                }
            );
            assert_eq!(item.Repeat().y, EFillRepeat::kNoRepeatFill);
            assert_eq!(
                item.Origin(),
                if even {
                    EFillBox::kContent
                } else {
                    EFillBox::kPadding
                }
            );
            assert_eq!(
                item.Clip(),
                if even {
                    EFillBox::kPadding
                } else {
                    EFillBox::kContent
                }
            );
            assert_eq!(
                item.Attachment(),
                if even {
                    EFillAttachment::kLocal
                } else {
                    EFillAttachment::kFixed
                }
            );
            assert_eq!(
                item.GetBlendMode(),
                if even {
                    BlendMode::kMultiply
                } else {
                    BlendMode::kScreen
                }
            );
            assert_eq!(
                item.IsPositionXSet(),
                i < 2,
                "repetition preserves authored set flags"
            );
            assert!(
                item.IsImageSet(),
                "explicit none still establishes an image layer"
            );
        }
    }
}

#[test]
fn mask_lists_cycle_and_excess_layers_are_culled_in_production() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(&format!("<!doctype html><div id=a style='mask-image:{IMAGES};mask-position:4px 9px;mask-size:20px 40px;mask-repeat:no-repeat;mask-origin:content-box;mask-clip:padding-box'></div><div id=b style='background-image:url(sprite.svg);background-position:1px 2px,3px 4px,5px 6px;mask-image:url(sprite.svg);mask-position:1px 2px,3px 4px'></div>"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let a = native(&owner, "a");
    let masks = layers(unsafe { &*a.Get() }.MaskLayers());
    assert_eq!(masks.len(), 3);
    for item in masks {
        assert_eq!(item.PositionX().Pixels(), 4.0);
        assert_eq!(item.PositionY().Pixels(), 9.0);
        assert_eq!(item.SizeLength().Width().Pixels(), 20.0);
        assert_eq!(item.SizeLength().Height().Pixels(), 40.0);
        assert_eq!(item.Repeat().x, EFillRepeat::kNoRepeatFill);
        assert_eq!(item.Repeat().y, EFillRepeat::kNoRepeatFill);
        assert_eq!(item.Origin(), EFillBox::kContent);
        assert_eq!(item.Clip(), EFillBox::kPadding);
    }
    let b = native(&owner, "b");
    assert_eq!(layers(unsafe { &*b.Get() }.BackgroundLayers()).len(), 1);
    assert_eq!(layers(unsafe { &*b.Get() }.MaskLayers()).len(), 1);
}

#[test]
fn fill_layer_mutation_preserves_old_native_snapshots_and_cached_siblings() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = "background-image:linear-gradient(transparent,transparent),url(sprite.svg);background-position:0 -20px;background-repeat:no-repeat";
    let mut owner = html::html_parser::ParseHTML(&format!(
        "<!doctype html><div id=a style='{initial}'></div><div id=b style='{initial}'></div>"
    ));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let old = native(&owner, "a");
    let old_second = unsafe { &*old.Get() }.BackgroundLayers().Next();
    assert_eq!(unsafe { &*old_second }.PositionY().Pixels(), -20.0);
    set_style(
        &mut owner,
        "a",
        &format!("{initial};background-position:0 -40px"),
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let changed = native(&owner, "a");
    for layer in layers(unsafe { &*changed.Get() }.BackgroundLayers()) {
        assert_eq!(layer.PositionY().Pixels(), -40.0);
    }
    assert_eq!(unsafe { &*old_second }.PositionY().Pixels(), -20.0);
    let sibling = native(&owner, "b");
    for layer in layers(unsafe { &*sibling.Get() }.BackgroundLayers()) {
        assert_eq!(layer.PositionY().Pixels(), -20.0);
    }
    set_style(
        &mut owner,
        "a",
        "background-image:url(sprite.svg);background-position:0 -60px,0 -80px,0 -100px",
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let culled = native(&owner, "a");
    assert_eq!(
        layers(unsafe { &*culled.Get() }.BackgroundLayers()).len(),
        1
    );
    assert_eq!(
        layers(unsafe { &*changed.Get() }.BackgroundLayers()).len(),
        2
    );
}

#[test]
fn five_image_layers_repeat_independent_numeric_patterns() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=a style='background-image:url(a.svg),none,url(b.svg),none,url(c.svg);background-position:2px 3px,7px 8px;background-size:20px 40px;background-repeat:no-repeat'></div>");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let style = native(&owner, "a");
    let items = layers(unsafe { &*style.Get() }.BackgroundLayers());
    assert_eq!(items.len(), 5);
    for (i, item) in items.iter().enumerate() {
        assert_eq!(
            item.PositionX().Pixels(),
            if i % 2 == 0 { 2.0 } else { 7.0 }
        );
        assert_eq!(
            item.PositionY().Pixels(),
            if i % 2 == 0 { 3.0 } else { 8.0 }
        );
        assert_eq!(item.SizeLength().Width().Pixels(), 20.0);
        assert_eq!(item.Repeat().x, EFillRepeat::kNoRepeatFill);
        assert_eq!(item.Repeat().y, EFillRepeat::kNoRepeatFill);
    }
}
