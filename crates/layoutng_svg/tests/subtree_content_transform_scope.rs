use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use layoutng_svg::subtree_content_transform_scope::SubtreeContentTransformScope;

#[test]
fn nested_scope_restores_the_previous_global_transform() {
    let original = SubtreeContentTransformScope::CurrentContentTransformation();
    let mut translation = AffineTransform::default();
    translation.Translate(3.0, 5.0);
    {
        let _outer = SubtreeContentTransformScope::new(translation);
        assert_eq!(
            SubtreeContentTransformScope::CurrentContentTransformation(),
            translation
        );
        let mut scale = AffineTransform::default();
        scale.Scale(2.0, 2.0);
        let mut expected = translation;
        expected.PostConcat(scale);
        {
            let _inner = SubtreeContentTransformScope::new(scale);
            assert_eq!(
                SubtreeContentTransformScope::CurrentContentTransformation(),
                expected
            );
        }
        assert_eq!(
            SubtreeContentTransformScope::CurrentContentTransformation(),
            translation
        );
    }
    assert_eq!(
        SubtreeContentTransformScope::CurrentContentTransformation(),
        original
    );
}
