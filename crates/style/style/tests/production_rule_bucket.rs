use foundation::AtomicString;
use style::{production_style_sheet::ProductionRuleBucketTable, rule_set::RuleBucketTable};

#[test]
fn atomic_rule_bucket_uses_interned_keys_without_changing_extent_semantics() {
    let key = AtomicString::from_str("octicon-repo");
    let same_interned_key = AtomicString::from_str("octicon-repo");
    let other_key = AtomicString::from_str("octicon-issue-opened");
    let mut buckets = ProductionRuleBucketTable::New(8);

    assert!(buckets.Find(&key).is_none());
    let extent = buckets.Insert(&key).expect("resident key insertion");
    extent.index = 17;
    extent.length = 3;

    let found = buckets
        .Find(&same_interned_key)
        .expect("re-interned AtomicString must address the same rule bucket");
    assert_eq!((found.index, found.length), (17, 3));

    buckets
        .Insert(&same_interned_key)
        .expect("existing resident key")
        .length += 2;
    buckets
        .Insert(&other_key)
        .expect("second resident key")
        .index = 29;

    assert_eq!(buckets.Find(&key).unwrap().length, 5);
    assert_eq!(buckets.Find(&other_key).unwrap().index, 29);
    assert_eq!(buckets.Entries().len(), 2);
}
