// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/invalidation/invalidation_set_test.cc:22-277
// The ten original Backing tests are mapped. DOM matching, enclosing set
// mutation, and singleton-combination tests remain pending their real types.
use super::*;

// cpp: invalidation_set_test.cc:22-32
fn HasAny<const TYPE: u8>(backing: &Backing<TYPE>, flags: &BackingFlags, args: &[&str]) -> bool {
    for string in args {
        if backing.Contains(flags, &AtomicString::from_str(string)) {
            return true;
        }
    }
    false
}
// cpp: invalidation_set_test.cc:34-44
fn HasAll<const TYPE: u8>(backing: &Backing<TYPE>, flags: &BackingFlags, args: &[&str]) -> bool {
    for string in args {
        if !backing.Contains(flags, &AtomicString::from_str(string)) {
            return false;
        }
    }
    true
}

// cpp: invalidation_set_test.cc:46-51
#[test]
fn Backing_Create() {
    let flags = BackingFlags::default();
    let backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    assert!(!(backing.IsHashSet(&flags)));
}

// cpp: invalidation_set_test.cc:53-63
#[test]
fn Backing_Add() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    assert!(!(backing.IsHashSet(&flags)));
    backing.Add(&mut flags, &AtomicString::from_str("test1"));
    assert!(!(backing.IsHashSet(&flags)));
    backing.Add(&mut flags, &AtomicString::from_str("test2"));
    assert!(backing.IsHashSet(&flags));
    backing.Clear(&mut flags);
}

// cpp: invalidation_set_test.cc:65-76
#[test]
fn Backing_AddSame() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    assert!(!(backing.IsHashSet(&flags)));
    backing.Add(&mut flags, &AtomicString::from_str("test1"));
    assert!(!(backing.IsHashSet(&flags)));
    backing.Add(&mut flags, &AtomicString::from_str("test1"));
    // No need to upgrade to HashSet if we're adding the item we already have.
    assert!(!(backing.IsHashSet(&flags)));
    backing.Clear(&mut flags);
}

// cpp: invalidation_set_test.cc:78-142
#[test]
fn Backing_Independence() {
    let mut flags = BackingFlags::default();

    let mut classes = Backing::<{ BackingType::kClasses as u8 }>::default();
    let mut ids = Backing::<{ BackingType::kIds as u8 }>::default();
    let mut tag_names = Backing::<{ BackingType::kTagNames as u8 }>::default();
    let mut attributes = Backing::<{ BackingType::kAttributes as u8 }>::default();

    classes.Add(&mut flags, &AtomicString::from_str("test1"));
    ids.Add(&mut flags, &AtomicString::from_str("test2"));
    tag_names.Add(&mut flags, &AtomicString::from_str("test3"));
    attributes.Add(&mut flags, &AtomicString::from_str("test4"));

    // Adding to set does not affect other backings:
    assert!(classes.Contains(&flags, &AtomicString::from_str("test1")));
    assert!(!(HasAny(&classes, &flags, &["test2", "test3", "test4"])));

    assert!(ids.Contains(&flags, &AtomicString::from_str("test2")));
    assert!(!(HasAny(&ids, &flags, &["test1", "test3", "test4"])));

    assert!(tag_names.Contains(&flags, &AtomicString::from_str("test3")));
    assert!(!(HasAny(&tag_names, &flags, &["test1", "test2", "test4"])));

    assert!(attributes.Contains(&flags, &AtomicString::from_str("test4")));
    assert!(!(HasAny(&attributes, &flags, &["test1", "test2", "test3"])));

    // Adding additional items to one set does not affect others:
    classes.Add(&mut flags, &AtomicString::from_str("test5"));
    tag_names.Add(&mut flags, &AtomicString::from_str("test6"));

    assert!(HasAll(&classes, &flags, &["test1", "test5"]));
    assert!(!(HasAny(&classes, &flags, &["test2", "test3", "test4", "test6"])));

    assert!(ids.Contains(&flags, &AtomicString::from_str("test2")));
    assert!(!(HasAny(&ids, &flags, &["test1", "test3", "test4", "test5", "test6"])));

    assert!(HasAll(&tag_names, &flags, &["test3", "test6"]));
    assert!(!(HasAny(&tag_names, &flags, &["test1", "test2", "test4", "test5"])));

    assert!(attributes.Contains(&flags, &AtomicString::from_str("test4")));
    assert!(!(HasAny(&attributes, &flags, &["test1", "test2", "test3"])));

    // Clearing one set does not clear others:

    classes.Clear(&mut flags);
    ids.Clear(&mut flags);
    attributes.Clear(&mut flags);

    let all_test_strings = &["test1", "test2", "test3", "test4", "test5", "test6"];

    assert!(!(HasAny(&classes, &flags, all_test_strings)));
    assert!(!(HasAny(&ids, &flags, all_test_strings)));
    assert!(!(HasAny(&attributes, &flags, all_test_strings)));

    assert!(!(classes.IsHashSet(&flags)));
    assert!(!(ids.IsHashSet(&flags)));
    assert!(!(attributes.IsHashSet(&flags)));

    assert!(tag_names.IsHashSet(&flags));
    assert!(HasAll(&tag_names, &flags, &["test3", "test6"]));
    assert!(!(HasAny(&tag_names, &flags, &["test1", "test2", "test4", "test5"])));
    tag_names.Clear(&mut flags);
}

// cpp: invalidation_set_test.cc:144-178
#[test]
fn Backing_ClearContains() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    let test1 = AtomicString::from_str("test1");
    let test2 = AtomicString::from_str("test2");

    // Clearing an empty set:
    assert!(!(backing.Contains(&flags, &test1)));
    assert!(!(backing.IsHashSet(&flags)));
    backing.Clear(&mut flags);
    assert!(!(backing.IsHashSet(&flags)));

    // Add one element to the set, and clear it:
    backing.Add(&mut flags, &test1);
    assert!(!(backing.IsHashSet(&flags)));
    assert!(backing.Contains(&flags, &test1));
    backing.Clear(&mut flags);
    assert!(!(backing.Contains(&flags, &test1)));
    assert!(!(backing.IsHashSet(&flags)));

    // Add two elements to the set, and clear them:
    backing.Add(&mut flags, &test1);
    assert!(!(backing.IsHashSet(&flags)));
    assert!(backing.Contains(&flags, &test1));
    assert!(!(backing.Contains(&flags, &test2)));
    backing.Add(&mut flags, &test2);
    assert!(backing.IsHashSet(&flags));
    assert!(backing.Contains(&flags, &test1));
    assert!(backing.Contains(&flags, &test2));
    backing.Clear(&mut flags);
    assert!(!(backing.Contains(&flags, &test1)));
    assert!(!(backing.Contains(&flags, &test2)));
    assert!(!(backing.IsHashSet(&flags)));
}

// cpp: invalidation_set_test.cc:180-190
#[test]
fn Backing_BackingIsEmpty() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    assert!(backing.IsEmpty(&flags));
    backing.Add(&mut flags, &AtomicString::from_str("test1"));
    assert!(!(backing.IsEmpty(&flags)));
    backing.Add(&mut flags, &AtomicString::from_str("test2"));
    backing.Clear(&mut flags);
    assert!(backing.IsEmpty(&flags));
}

// cpp: invalidation_set_test.cc:192-203
#[test]
fn Backing_IsEmpty() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

    assert!(backing.IsEmpty(&flags));

    backing.Add(&mut flags, &AtomicString::from_str("test1"));
    assert!(!(backing.IsEmpty(&flags)));

    backing.Clear(&mut flags);
    assert!(backing.IsEmpty(&flags));
}

// cpp: invalidation_set_test.cc:205-254
#[test]
fn Backing_Iterator() {
    let test1 = AtomicString::from_str("test1");
    let test2 = AtomicString::from_str("test2");
    let test3 = AtomicString::from_str("test3");
    // Iterate over empty set.
    {
        let flags = BackingFlags::default();
        let backing = Backing::<{ BackingType::kClasses as u8 }>::default();

        let mut strings = Vec::new();
        for string in backing.Items(&flags) {
            strings.push(string.clone());
        }
        assert_eq!(0, strings.len());
    }

    // Iterate over set with one item.
    {
        let mut flags = BackingFlags::default();
        let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

        backing.Add(&mut flags, &test1);
        let mut strings = Vec::new();
        for string in backing.Items(&flags) {
            strings.push(string.clone());
        }
        assert_eq!(1, strings.len());
        assert!(strings.contains(&test1));
        backing.Clear(&mut flags);
    }

    // Iterate over set with multiple items.
    {
        let mut flags = BackingFlags::default();
        let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();

        backing.Add(&mut flags, &test1);
        backing.Add(&mut flags, &test2);
        backing.Add(&mut flags, &test3);
        let mut strings = Vec::new();
        for string in backing.Items(&flags) {
            strings.push(string.clone());
        }
        assert_eq!(3, strings.len());
        assert!(strings.contains(&test1));
        assert!(strings.contains(&test2));
        assert!(strings.contains(&test3));
        backing.Clear(&mut flags);
    }
}

// cpp: invalidation_set_test.cc:256-266
#[test]
fn Backing_GetString() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();
    assert!(backing.GetString(&flags).is_some());
    assert!(backing.GetString(&flags).unwrap().IsNull());
    backing.Add(&mut flags, &AtomicString::from_str("a"));
    assert_eq!(
        AtomicString::from_str("a"),
        *backing.GetString(&flags).unwrap()
    );
    backing.Add(&mut flags, &AtomicString::from_str("b"));
    assert!(backing.GetString(&flags).is_none());
    backing.Clear(&mut flags);
}

// cpp: invalidation_set_test.cc:268-277
#[test]
fn Backing_GetHashSet() {
    let mut flags = BackingFlags::default();
    let mut backing = Backing::<{ BackingType::kClasses as u8 }>::default();
    assert!(!(backing.GetHashSet(&flags).is_some()));
    backing.Add(&mut flags, &AtomicString::from_str("a"));
    assert!(!(backing.GetHashSet(&flags).is_some()));
    backing.Add(&mut flags, &AtomicString::from_str("b"));
    assert!(backing.GetHashSet(&flags).is_some());
    backing.Clear(&mut flags);
}

// Additional coverage of invalidation_set.h:614-624 and
// invalidation_set.cc:50-66, preserving their exact null-slot size behavior.
#[test]
fn BackingSizeAndEqualFollowOriginalNullSlotLogic() {
    let a = Backing::<{ BackingType::kClasses as u8 }>::default();
    let mut b = Backing::<{ BackingType::kClasses as u8 }>::default();
    let a_flags = BackingFlags::default();
    let mut b_flags = BackingFlags::default();
    assert!(a.IsEmpty(&a_flags));
    assert_eq!(a.Size(&a_flags), 1);
    b.Add(&mut b_flags, &AtomicString::from_str("one"));
    assert!(BackingEqual(&a_flags, &a, &b_flags, &b));
    assert!(!BackingEqual(&b_flags, &b, &a_flags, &a));
    b.Add(&mut b_flags, &AtomicString::from_str("two"));
    assert_eq!(b.Size(&b_flags), 2);
    assert!(!BackingEqual(&a_flags, &a, &b_flags, &b));
    b.Clear(&mut b_flags);
}
