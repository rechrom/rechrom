use foundation::StringBuilder;

#[test]
fn empty_builder_returns_non_null_empty_string() {
    let mut builder = StringBuilder::new();
    let value = builder.ToString();
    assert!(!value.IsNull());
    assert!(value.empty());

    let released = builder.ReleaseString();
    assert!(!released.IsNull());
    assert!(released.empty());
}
