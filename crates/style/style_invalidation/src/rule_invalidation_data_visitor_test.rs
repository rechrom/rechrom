// Additional verification for the mapped local types in
// rule_invalidation_data_visitor.h:94-329 and .cc:287-341.
use super::*;

fn atom(value: &str) -> AtomicString {
    AtomicString::from_str(value)
}

#[test]
fn emitted_tags_are_routed_without_merging_subject_depth() {
    let mut features = InvalidationSetFeatures::default();
    features.tag_names.push(atom("span"));
    features.descendant_features_depth = 2;
    features.max_direct_adjacent_selectors = 1;
    let mut other = InvalidationSetFeatures::default();
    other.classes.push(atom("a"));
    other.attributes.push(atom("title"));
    other.ids.push(atom("b"));
    other.custom_pseudo_names.push(atom("pseudo"));
    other.tag_names.push(atom("li"));
    other.emitted_tag_names.push(atom("ol"));
    other.has_features_for_rule_set_invalidation = true;
    other.descendant_features_depth = 9;
    other.max_direct_adjacent_selectors = 3;
    other.invalidation_flags.SetInvalidatesParts(true);
    other.content_pseudo_crossing = true;
    other.has_nth_pseudo = true;
    features.Merge(&other);
    assert_eq!(features.tag_names, vec![atom("span")]);
    assert_eq!(features.emitted_tag_names, vec![atom("li"), atom("ol")]);
    assert_eq!(features.Size(), 7);
    assert_eq!(features.descendant_features_depth, 2);
    assert_eq!(features.max_direct_adjacent_selectors, 3);
    assert!(!features.has_features_for_rule_set_invalidation);
    assert!(features.invalidation_flags.InvalidatesParts());
    assert!(features.content_pseudo_crossing && features.has_nth_pseudo);
    assert!(features.HasFeatures() && features.HasIdClassOrAttribute());
    features.ClearFeatures();
    assert_eq!(features.Size(), 0);
    assert!(features.HasFeatures()); // Parts invalidation counts as a feature.
    assert!(!features.HasIdClassOrAttribute());
    assert_eq!(features.descendant_features_depth, 2);
}

#[test]
fn narrowing_keeps_the_source_precedence_and_metadata() {
    let mut features = InvalidationSetFeatures::default();
    features.descendant_features_depth = 2;
    features.NarrowToTag(&atom("div"));
    features.NarrowToCustomPseudo(&atom("nonempty"));
    assert_eq!(features.tag_names, vec![atom("div")]);
    features.NarrowToCustomPseudo(&atom(""));
    assert_eq!(features.custom_pseudo_names, vec![atom("")]);
    features.NarrowToAttribute(&atom("title"));
    assert_eq!(features.attributes, vec![atom("title")]);
    features.NarrowToClass(&atom("a"));
    assert_eq!(features.classes, vec![atom("a")]);
    features.NarrowToAttribute(&atom("hidden"));
    features.NarrowToTag(&atom("span"));
    assert_eq!(features.classes, vec![atom("a")]);
    features.NarrowToId(&atom("b"));
    features.NarrowToClass(&atom("c"));
    assert_eq!(features.ids, vec![atom("b")]);
    let mut other = InvalidationSetFeatures::default();
    other.tag_names.push(atom("p"));
    features.NarrowToFeatures(&other); // Equal-size alternatives do not replace.
    assert_eq!(features.ids, vec![atom("b")]);
    features.classes.push(atom("c"));
    features.NarrowToFeatures(&other);
    assert_eq!(features.tag_names, vec![atom("p")]);
    assert!(features.ids.is_empty() && features.classes.is_empty());
    assert_eq!(features.descendant_features_depth, 2);
    features.NarrowToFeatures(&InvalidationSetFeatures::default());
    assert_eq!(features.Size(), 1); // Empty other does not replace nonempty self.
    features.ClearFeatures();
    features.NarrowToFeatures(&other);
    assert_eq!(features.tag_names, vec![atom("p")]);
}

#[test]
fn guards_restore_only_their_own_state_including_during_unwind() {
    let mut features = InvalidationSetFeatures::default();
    features.max_direct_adjacent_selectors = 3;
    features.descendant_features_depth = 2;
    {
        let mut guard = AutoRestoreMaxDirectAdjacentSelectors::new(Some(&mut features));
        let f = guard.Features().unwrap();
        f.max_direct_adjacent_selectors = 7;
        f.descendant_features_depth = 4;
    }
    assert_eq!(features.max_direct_adjacent_selectors, 3);
    assert_eq!(features.descendant_features_depth, 4);
    {
        let mut guard = AutoRestoreDescendantFeaturesDepth::new(Some(&mut features));
        guard.Features().unwrap().descendant_features_depth = 9;
    }
    assert_eq!(features.descendant_features_depth, 4);
    assert!(AutoRestoreMaxDirectAdjacentSelectors::new(None)
        .Features()
        .is_none());
    assert!(AutoRestoreDescendantFeaturesDepth::new(None)
        .Features()
        .is_none());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut guard = AutoRestoreWholeSubtreeInvalid::new(&mut features);
        guard.invalidation_flags.SetWholeSubtreeInvalid(true);
        guard.invalidation_flags.SetTreeBoundaryCrossing(true);
        panic!("exercise guard drop during unwinding");
    }));
    assert!(result.is_err());
    assert!(!features.invalidation_flags.WholeSubtreeInvalid());
    assert!(features.invalidation_flags.TreeBoundaryCrossing());
    {
        let mut guard = AutoRestoreTreeBoundaryCrossingFlag::new(&mut features);
        guard.invalidation_flags.SetTreeBoundaryCrossing(false);
    }
    assert!(features.invalidation_flags.TreeBoundaryCrossing());
    {
        let mut guard = AutoRestoreInsertionPointCrossingFlag::new(&mut features);
        guard.invalidation_flags.SetInsertionPointCrossing(true);
    }
    assert!(!features.invalidation_flags.InsertionPointCrossing());
}
