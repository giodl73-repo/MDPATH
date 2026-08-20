use mdpath::document::ParsedDocument;
use mdpath::heading::normalize_heading;
use mdpath::{resolve, resolve_with_classifier, DefaultClassifier, ElementType, MdPathError};
use std::path::Path;

const CORPUS: &str = include_str!("../fixtures/consumers/corpus.md");

fn normalize(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_owned()
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn retained_consumers_match_their_public_surfaces() {
    let document: ParsedDocument = mdpath::parser::parse_document(CORPUS);
    let amaze = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"mdpath.consumer-proof.v1\",\n",
            "  \"consumer\": \"AMAZE\",\n",
            "  \"status\": \"accepted\",\n",
            "  \"surface\": \"document-parse-and-heading-normalization\",\n",
            "  \"heading_count\": {},\n",
            "  \"normalized_heading\": \"{}\"\n",
            "}}"
        ),
        document.headings.len(),
        normalize_heading("Input/Output Handling")
    );
    assert_eq!(
        normalize(&amaze),
        normalize(include_str!("../fixtures/consumers/amaze.json"))
    );

    let mdcrop_uri =
        mdpath::parse("md://fixtures/consumers/corpus.md#consumer-contract/inputoutput-handling")
            .expect("MDCROP section URI should parse");
    let mdcrop = resolve(&mdcrop_uri, root()).expect("MDCROP section URI should resolve");
    let mdcrop_report = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"mdpath.consumer-proof.v1\",\n",
            "  \"consumer\": \"MDCROP\",\n",
            "  \"status\": \"accepted\",\n",
            "  \"surface\": \"canonical-section-resolution\",\n",
            "  \"uri\": \"{}\",\n",
            "  \"label\": \"{}\",\n",
            "  \"element_type\": \"{}\",\n",
            "  \"line_start\": {},\n",
            "  \"line_end\": {}\n",
            "}}"
        ),
        mdcrop.uri,
        mdcrop
            .label
            .as_deref()
            .expect("section should have a label"),
        format!("{:?}", mdcrop.element_type).to_ascii_lowercase(),
        mdcrop.line_start,
        mdcrop.line_end
    );
    assert_eq!(
        normalize(&mdcrop_report),
        normalize(include_str!("../fixtures/consumers/mdcrop.json"))
    );

    let mdloom_uri =
        mdpath::parse("md://fixtures/consumers/corpus.md#consumer-contract:math:energy-balance")
            .expect("MDLOOM math URI should parse");
    let mdloom = resolve_with_classifier(&mdloom_uri, root(), &DefaultClassifier)
        .expect("MDLOOM math URI should resolve");
    let mdloom_report = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"mdpath.consumer-proof.v1\",\n",
            "  \"consumer\": \"MDLOOM\",\n",
            "  \"status\": \"accepted\",\n",
            "  \"surface\": \"typed-fence-resolution\",\n",
            "  \"uri\": \"{}\",\n",
            "  \"label\": \"{}\",\n",
            "  \"element_type\": \"{}\",\n",
            "  \"line_start\": {},\n",
            "  \"line_end\": {}\n",
            "}}"
        ),
        mdloom.uri,
        mdloom
            .label
            .as_deref()
            .expect("math block should have a label"),
        format!("{:?}", mdloom.element_type).to_ascii_lowercase(),
        mdloom.line_start,
        mdloom.line_end
    );
    assert_eq!(mdloom.element_type, ElementType::Math);
    assert_eq!(
        normalize(&mdloom_report),
        normalize(include_str!("../fixtures/consumers/mdloom.json"))
    );
}

#[test]
fn retained_consumers_keep_structured_failures() {
    let parse_error =
        mdpath::parse("https://example.invalid/corpus.md").expect_err("non-md URI must fail");
    assert!(matches!(parse_error, MdPathError::ParseError(_)));

    let missing_uri = mdpath::parse("md://fixtures/consumers/corpus.md#missing")
        .expect("missing-section URI should parse");
    let section_error = resolve(&missing_uri, root()).expect_err("missing section must fail");
    assert!(matches!(
        section_error,
        MdPathError::SectionNotFound(ref segment) if segment == "missing"
    ));

    let report = concat!(
        "{\n",
        "  \"schema\": \"mdpath.consumer-proof.v1\",\n",
        "  \"status\": \"rejected\",\n",
        "  \"failures\": [\n",
        "    {\n",
        "      \"surface\": \"uri-parse\",\n",
        "      \"kind\": \"parse-error\",\n",
        "      \"input\": \"https://example.invalid/corpus.md\"\n",
        "    },\n",
        "    {\n",
        "      \"surface\": \"section-resolution\",\n",
        "      \"kind\": \"section-not-found\",\n",
        "      \"segment\": \"missing\"\n",
        "    }\n",
        "  ]\n",
        "}"
    );
    assert_eq!(
        normalize(report),
        normalize(include_str!("../fixtures/consumers/failures.json"))
    );
}
