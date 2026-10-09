// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_map_test.cc
use super::super::cascade_origin::CascadeOrigin;
use super::*;

// cpp: cascade_map_test.cc:38-47
fn AuthorPriority(rule_index: WtfSizeT, declaration_index: WtfSizeT) -> CascadePriority {
    CascadePriority::FromParts(
        CascadeOrigin::kAuthor,
        false,
        0,
        false,
        false,
        false,
        0,
        rule_index as u16,
        declaration_index as u16,
    )
}

fn UaPriority(rule_index: WtfSizeT, declaration_index: WtfSizeT) -> CascadePriority {
    CascadePriority::FromParts(
        CascadeOrigin::kUserAgent,
        false,
        0,
        false,
        false,
        false,
        0,
        rule_index as u16,
        declaration_index as u16,
    )
}

fn UserPriority(rule_index: WtfSizeT, declaration_index: WtfSizeT) -> CascadePriority {
    CascadePriority::FromParts(
        CascadeOrigin::kUser,
        false,
        0,
        false,
        false,
        false,
        0,
        rule_index as u16,
        declaration_index as u16,
    )
}

fn AddTo(map: &mut CascadeMap, name: &CSSPropertyName, priority: CascadePriority) -> bool {
    let before = map.At(name);
    if name.IsCustomProperty() {
        map.AddCustom(name.ToAtomicString().clone(), priority);
    } else {
        map.Add(name.Id(), priority);
    }
    before != map.At(name)
}

// cpp: cascade_map_test.cc:77-105
#[test]
fn EmptyAndAddCustom() {
    let mut map = CascadeMap::new();
    let x = CSSPropertyName::custom(AtomicString::from_str("--x"));
    let y = CSSPropertyName::custom(AtomicString::from_str("--y"));
    assert!(map.Find(&x).is_none());
    assert!(map.Find(&y).is_none());

    let user = CascadePriority::FromOrigin(CascadeOrigin::kUser);
    let author = CascadePriority::FromOrigin(CascadeOrigin::kAuthor);
    assert!(AddTo(&mut map, &x, user));
    assert!(AddTo(&mut map, &x, author));
    assert!(!AddTo(&mut map, &x, author));
    assert_eq!(map.Find(&x), Some(&author));
    assert!(AddTo(&mut map, &y, user));
    assert_eq!(map.Find(&x), Some(&author));
    assert_eq!(map.Find(&y), Some(&user));
}

// cpp: cascade_map_test.cc:107-135,137-172
#[test]
fn AddNativeAndMutateWinner() {
    let mut map = CascadeMap::new();
    let color = CSSPropertyName::new(CSSPropertyID::kColor);
    let display = CSSPropertyName::new(CSSPropertyID::kDisplay);
    let user = CascadePriority::FromOrigin(CascadeOrigin::kUser);
    let author = CascadePriority::FromOrigin(CascadeOrigin::kAuthor);

    assert!(AddTo(&mut map, &color, user));
    assert!(AddTo(&mut map, &color, author));
    assert!(!AddTo(&mut map, &color, author));
    assert!(AddTo(&mut map, &display, user));
    assert_eq!(map.Find(&color), Some(&author));
    assert_eq!(map.Find(&display), Some(&user));

    *map.FindMut(&color).unwrap() = user;
    assert_eq!(map.Find(&color), Some(&user));
}

// cpp: cascade_map_test.cc:201-246,248-271
#[test]
fn HighPriorityAndReset() {
    let mut map = CascadeMap::new();
    let author = CascadePriority::FromOrigin(CascadeOrigin::kAuthor);
    map.Add(CSSPropertyID::kFontSize, author);
    map.Add(CSSPropertyID::kColor, author);
    assert_eq!(
        map.HighPriorityBits(),
        (1u64 << CSSPropertyID::kFontSize as usize) | (1u64 << CSSPropertyID::kColor as usize)
    );
    map.AddCustom(AtomicString::from_str("--x"), author);
    map.Reset();
    assert_eq!(map.HighPriorityBits(), 0);
    assert!(map
        .Find(&CSSPropertyName::new(CSSPropertyID::kColor))
        .is_none());
    assert!(map
        .Find(&CSSPropertyName::custom(AtomicString::from_str("--x")))
        .is_none());
}

// cpp: cascade_map_test.cc:273-326
#[test]
fn FindForOrigin() {
    let mut map = CascadeMap::new();
    let color = CSSPropertyName::new(CSSPropertyID::kColor);
    map.Add(color.Id(), UaPriority(0, 1));
    map.Add(color.Id(), UserPriority(0, 10));
    map.Add(color.Id(), AuthorPriority(0, 20));
    assert_eq!(map.Find(&color), Some(&AuthorPriority(0, 20)));
    assert_eq!(
        map.FindForOrigin(&color, CascadeOrigin::kUser),
        Some(&UserPriority(0, 10))
    );
    assert_eq!(
        map.FindForOrigin(&color, CascadeOrigin::kUserAgent),
        Some(&UaPriority(0, 1))
    );
}

// cpp: cascade_map_test.cc:328-373
#[test]
fn FindRevertRuleSkipsDuplicateRule() {
    let mut map = CascadeMap::new();
    let color = CSSPropertyName::new(CSSPropertyID::kColor);
    let p1 = AuthorPriority(0, 0);
    let p2 = AuthorPriority(1, 0);
    map.Add(color.Id(), p1);
    map.Add(color.Id(), p2);
    map.Add(color.Id(), p2);
    assert_eq!(map.FindRevertRule(&color, p2), Some(&p1));
    assert!(map.FindRevertRule(&color, p1).is_none());
}

// cpp: cascade_map.cc:155-175,220-224
#[test]
fn ImportantSetAndClearAppliedFlags() {
    let mut map = CascadeMap::new();
    let important = CascadePriority::FromOriginImportance(CascadeOrigin::kAuthor, true);
    map.Add(
        CSSPropertyID::kColor,
        CascadePriority::WithAlreadyApplied(important, true),
    );
    assert!(map
        .Find(&CSSPropertyName::new(CSSPropertyID::kColor))
        .unwrap()
        .IsAlreadyApplied());
    map.ClearAppliedFlags();
    assert!(!map
        .Find(&CSSPropertyName::new(CSSPropertyID::kColor))
        .unwrap()
        .IsAlreadyApplied());
    let important_set = map.ReleaseImportantSet().unwrap();
    assert!(important_set.Has(CSSPropertyID::kColor));
}

// cpp: cascade_map_test.cc:62-71
fn ToCascadePriorityVector(
    list: &CascadePriorityList,
    backing_vector: &CascadePriorityBackingVector,
) -> Vec<CascadePriority> {
    let mut v = Vec::new();
    let mut i = list.Begin(backing_vector);
    while i != list.End(backing_vector) {
        v.push(*i);
        i.Advance();
    }
    v
}

// cpp: cascade_map_test.cc:375-386
#[test]
fn InsertIntoEmptyList() {
    let p1 = AuthorPriority(0, 1);

    let mut backing_vector = CascadePriorityBackingVector::new();
    let mut list = CascadePriorityList::default();
    assert!(list.IsEmpty());

    list.InsertKeepingSorted(&mut backing_vector, p1);

    assert_eq!(vec![p1], ToCascadePriorityVector(&list, &backing_vector));
}

// cpp: cascade_map_test.cc:388-406
#[test]
fn InsertStronger() {
    let p1 = AuthorPriority(0, 1);
    let p2 = AuthorPriority(0, 2);
    let p3 = AuthorPriority(0, 3);

    let mut backing_vector = CascadePriorityBackingVector::new();
    let mut list = CascadePriorityList::default();
    assert!(list.IsEmpty());

    list.InsertKeepingSorted(&mut backing_vector, p1);
    assert_eq!(vec![p1], ToCascadePriorityVector(&list, &backing_vector));
    list.InsertKeepingSorted(&mut backing_vector, p2);
    assert_eq!(
        vec![p2, p1],
        ToCascadePriorityVector(&list, &backing_vector)
    );
    list.InsertKeepingSorted(&mut backing_vector, p3);
    assert_eq!(
        vec![p3, p2, p1],
        ToCascadePriorityVector(&list, &backing_vector)
    );
}

// cpp: cascade_map_test.cc:408-426
#[test]
fn InsertWeaker() {
    let p1 = AuthorPriority(0, 1);
    let p2 = AuthorPriority(0, 2);
    let p3 = AuthorPriority(0, 3);

    let mut backing_vector = CascadePriorityBackingVector::new();
    let mut list = CascadePriorityList::default();
    assert!(list.IsEmpty());

    list.InsertKeepingSorted(&mut backing_vector, p3);
    assert_eq!(vec![p3], ToCascadePriorityVector(&list, &backing_vector));
    list.InsertKeepingSorted(&mut backing_vector, p2);
    assert_eq!(
        vec![p3, p2],
        ToCascadePriorityVector(&list, &backing_vector)
    );
    list.InsertKeepingSorted(&mut backing_vector, p1);
    assert_eq!(
        vec![p3, p2, p1],
        ToCascadePriorityVector(&list, &backing_vector)
    );
}

// cpp: cascade_map_test.cc:428-446
#[test]
fn InsertMiddle() {
    let p1 = AuthorPriority(0, 1);
    let p2 = AuthorPriority(0, 2);
    let p3 = AuthorPriority(0, 3);

    let mut backing_vector = CascadePriorityBackingVector::new();
    let mut list = CascadePriorityList::default();
    assert!(list.IsEmpty());

    list.InsertKeepingSorted(&mut backing_vector, p1);
    assert_eq!(vec![p1], ToCascadePriorityVector(&list, &backing_vector));
    list.InsertKeepingSorted(&mut backing_vector, p3);
    assert_eq!(
        vec![p3, p1],
        ToCascadePriorityVector(&list, &backing_vector)
    );
    list.InsertKeepingSorted(&mut backing_vector, p2);
    assert_eq!(
        vec![p3, p2, p1],
        ToCascadePriorityVector(&list, &backing_vector)
    );
}

// cpp: cascade_map_test.cc:448-481
#[test]
fn InsertTwoListsInterleaved() {
    let p1 = AuthorPriority(0, 1);
    let p2 = AuthorPriority(0, 2);
    let p3 = AuthorPriority(0, 3);
    let p4 = AuthorPriority(0, 4);
    let p5 = AuthorPriority(0, 5);
    let p6 = AuthorPriority(0, 6);

    let mut backing_vector = CascadePriorityBackingVector::new();
    let mut list1 = CascadePriorityList::default();
    let mut list2 = CascadePriorityList::default();

    list1.InsertKeepingSorted(&mut backing_vector, p1);
    list2.InsertKeepingSorted(&mut backing_vector, p2);
    assert_eq!(vec![p1], ToCascadePriorityVector(&list1, &backing_vector));
    assert_eq!(vec![p2], ToCascadePriorityVector(&list2, &backing_vector));

    list1.InsertKeepingSorted(&mut backing_vector, p5);
    list2.InsertKeepingSorted(&mut backing_vector, p6);
    assert_eq!(
        vec![p5, p1],
        ToCascadePriorityVector(&list1, &backing_vector)
    );
    assert_eq!(
        vec![p6, p2],
        ToCascadePriorityVector(&list2, &backing_vector)
    );

    // Inserts in the middle.
    list1.InsertKeepingSorted(&mut backing_vector, p3);
    list2.InsertKeepingSorted(&mut backing_vector, p4);
    assert_eq!(
        vec![p5, p3, p1],
        ToCascadePriorityVector(&list1, &backing_vector)
    );
    assert_eq!(
        vec![p6, p4, p2],
        ToCascadePriorityVector(&list2, &backing_vector)
    );
}

// Additional verification of cascade_map.h:136-140,268-284,246-251.
#[test]
fn SharedBackingVectorAndMutableTop() {
    let mut backing = CascadePriorityBackingVector::new();
    let p1 = AuthorPriority(0, 1);
    let p2 = AuthorPriority(0, 2);
    let mut first = CascadePriorityList::FromPriority(&mut backing, p1);
    let second = CascadePriorityList::FromPriority(&mut backing, p2);
    first.Push(&mut backing, p2);
    assert_eq!(first.Top(&backing), &p2);
    *first.TopMut(&mut backing) = CascadePriority::WithAlreadyApplied(p2, true);
    assert!(first.Top(&backing).IsAlreadyApplied());
    assert!(!second.Top(&backing).IsAlreadyApplied());
    assert_eq!(
        ToCascadePriorityVector(&first, &backing),
        vec![CascadePriority::WithAlreadyApplied(p2, true), p1]
    );
    assert_eq!(ToCascadePriorityVector(&second, &backing), vec![p2]);
    assert_eq!(
        CascadePriorityList::default().Begin(&backing),
        second.End(&backing)
    );
}
