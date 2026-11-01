use mdpath::{resolve, MdPathError};
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn numeric_selector_rejects_labelled_fenced_element() {
    let uri = mdpath::parse("md://fixtures/numeric/corpus.md#numeric-fallback:figure:0")
        .expect("numeric URI parses");

    let error = resolve(&uri, root()).expect_err("labelled numeric URI is stale");

    assert!(matches!(
        error,
        MdPathError::NumericUriStale(ref label) if label == "named-diagram"
    ));
}

#[test]
fn numeric_selector_still_resolves_unlabelled_fenced_element() {
    let uri = mdpath::parse("md://fixtures/numeric/corpus.md#numeric-fallback:figure:1")
        .expect("numeric URI parses");

    let resolved = resolve(&uri, root()).expect("unlabelled numeric fallback resolves");

    assert_eq!(resolved.label, None);
    assert_eq!(resolved.element_type, mdpath::ElementType::Figure);
}
