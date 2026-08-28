use mdpath::{resolve, MdPathError};
use std::path::Path;

const ACCEPTED_URI: &str = "md://fixtures/proof/corpus.md#proof-surface:figure:accepted-diagram";
const REJECTED_URI: &str = "md://fixtures/proof/corpus.md#proof-surface:figure:duplicate";

fn normalize(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_owned()
}

#[test]
fn proof_fixtures_record_accepted_resolution_and_structured_failure() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let accepted_uri = mdpath::parse(ACCEPTED_URI).expect("accepted URI parses");
    let accepted = resolve(&accepted_uri, root).expect("accepted URI resolves");
    let accepted_report = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"mdpath.proof.v1\",\n",
            "  \"status\": \"accepted\",\n",
            "  \"uri\": \"{}\",\n",
            "  \"element_type\": \"{}\",\n",
            "  \"label\": \"{}\",\n",
            "  \"line_start\": {},\n",
            "  \"line_end\": {}\n",
            "}}"
        ),
        accepted.uri,
        format!("{:?}", accepted.element_type).to_ascii_lowercase(),
        accepted
            .label
            .as_deref()
            .expect("accepted figure has a label"),
        accepted.line_start,
        accepted.line_end
    );
    assert_eq!(
        normalize(&accepted_report),
        normalize(include_str!("../fixtures/proof/accepted.json"))
    );

    let rejected_uri = mdpath::parse(REJECTED_URI).expect("rejected URI parses");
    let error = resolve(&rejected_uri, root).expect_err("ambiguous URI is rejected");
    let (label, count) = match error {
        MdPathError::LabelAmbiguous(label, count) => (label, count),
        other => panic!("expected LabelAmbiguous, got {other}"),
    };
    let rejected_report = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"mdpath.proof.v1\",\n",
            "  \"status\": \"rejected\",\n",
            "  \"uri\": \"{}\",\n",
            "  \"error\": {{\n",
            "    \"kind\": \"label-ambiguous\",\n",
            "    \"label\": \"{}\",\n",
            "    \"count\": {}\n",
            "  }}\n",
            "}}"
        ),
        rejected_uri.to_uri_string(),
        label,
        count
    );
    assert_eq!(
        normalize(&rejected_report),
        normalize(include_str!("../fixtures/proof/rejected.json"))
    );
}
