//! First-party single-document workbench. URI and resolution semantics stay in MDPATH.
use mdpath::{parser::parse_document, resolver::resolve_in_doc, MdPathError};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

pub const MAX_REQUEST: usize = 1024 * 1024;
pub const MAX_MARKDOWN: usize = 128 * 1024;
const SCHEMA: &str = "mdpath.workbench.v1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    markdown: String,
    path: String,
    uri: String,
}
fn fail(phase: &str, code: &str, message: &str) -> Value {
    json!({"schema":SCHEMA,"ok":false,"phase":phase,"diagnostic":{"code":code,"message":message}})
}
fn error(phase: &str, e: MdPathError) -> Value {
    let code = match &e {
        MdPathError::ParseError(_) => "ParseError",
        MdPathError::FileNotFound(_) => "FileNotFound",
        MdPathError::SectionNotFound(_) => "SectionNotFound",
        MdPathError::SectionAmbiguous(_) => "SectionAmbiguous",
        MdPathError::ElementNotFound(..) => "ElementNotFound",
        MdPathError::LabelNotFound(_) => "LabelNotFound",
        MdPathError::LabelAmbiguous(..) => "LabelAmbiguous",
        MdPathError::SubKeyNotFound(_) => "SubKeyNotFound",
        MdPathError::InvalidSubSelector(..) => "InvalidSubSelector",
        MdPathError::InvalidQueryOnType(..) => "InvalidQueryOnType",
        MdPathError::InvariantViolated(..) => "InvariantViolated",
        MdPathError::TemplateNotFound(_) => "TemplateNotFound",
        MdPathError::NumericUriStale(_) => "NumericUriStale",
        MdPathError::Io(..) => "Io",
    };
    let mut result = fail(phase, code, &e.to_string());
    if let MdPathError::NumericUriStale(label) = e {
        result["diagnostic"]["stable_label"] = json!(label);
    }
    result
}
fn response(input: &str) -> Value {
    if input.len() > MAX_REQUEST {
        return fail("request", "RequestLimit", "Request exceeds 1 MiB UTF-8.");
    }
    let req: Request = match serde_json::from_str(input) {
        Ok(req) => req,
        Err(e) => return fail("request", "InvalidRequest", &e.to_string()),
    };
    if req.markdown.len() > MAX_MARKDOWN || req.markdown.lines().count() > 8192 {
        return fail(
            "document",
            "DocumentLimit",
            "Markdown exceeds 128 KiB UTF-8 or 8,192 lines.",
        );
    }
    if req.uri.len() > 4096 || req.path.len() > 1024 {
        return fail(
            "request",
            "AddressLimit",
            "URI exceeds 4,096 bytes or logical path exceeds 1,024 bytes.",
        );
    }
    if req.path.is_empty()
        || req.path.contains(['\\', ':', '\0'])
        || req.path.split('/').any(|s| matches!(s, "" | "." | ".."))
    {
        return fail("document","InvalidLogicalPath","Use a relative logical path with forward slashes and no empty, dot or parent segments.");
    }
    if req.uri.split_once('#').is_some_and(|(_, fragment)| {
        fragment
            .split(|c: char| !c.is_ascii_digit())
            .any(|part| !part.is_empty() && part.parse::<u32>().is_err())
    }) {
        return fail("uri","PortableNumberLimit","Decimal runs in an address fragment must fit unsigned 32-bit values in this workbench.");
    }
    let uri = match mdpath::parse(&req.uri) {
        Ok(uri) => uri,
        Err(e) => return error("uri", e),
    };
    if uri.path != req.path {
        return fail(
            "binding",
            "DocumentMismatch",
            "URI path must exactly match this tab's logical document path. No other file is read.",
        );
    }
    let doc = parse_document(&req.markdown);
    let resolved = match resolve_in_doc(&uri, &doc, Path::new(&req.path)) {
        Ok(result) => result,
        Err(e) => return error("resolution", e),
    };
    json!({"schema":SCHEMA,"ok":true,
      "address":{"canonical":uri.to_uri_string(),"path":uri.path,"heading_path":uri.heading_path,"element_type":uri.element_type.map(|t|format!("{t:?}")),"kind":uri.kind,"selector":format!("{:?}",uri.selector),"sub_selectors":format!("{:?}",uri.sub_selectors),"query":format!("{:?}",uri.query)},
      "document":{"headings":doc.headings.len(),"elements":doc.elements.len()},
      "resolved":{"uri":resolved.uri,"file":resolved.file.to_string_lossy(),"line_start":resolved.line_start,"line_end":resolved.line_end,"content":resolved.content,"label":resolved.label,"section_heading":resolved.section_heading,"element_type":format!("{:?}",resolved.element_type),"kind":resolved.kind}})
}
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn inspect(input: &str) -> String {
    response(input).to_string()
}

pub fn examples() -> Value {
    let markdown = "# Address Lab\n\n## Architecture\n\n```\nrequest-router\n+--------+      +-------+\n| CLIENT | ---> | CACHE |\n+--------+      +-------+\n```\n\n## Measurements\n\n| Component | Budget |\n| --- | --- |\n| Router | 12 |\n| Cache | 8 |\n\n## Notes\n\nThese are synthetic examples, not performance evidence.\n";
    let mut map = serde_json::Map::new();
    for (name, uri) in [
        (
            "named",
            "md://guide.md#address-lab/architecture:figure:request-router",
        ),
        (
            "prefix",
            "md://guide.md#address-lab/architecture:figure:request",
        ),
        (
            "cell",
            "md://guide.md#address-lab/measurements:table:0[row=Cache,col=Budget]",
        ),
        (
            "projection",
            "md://guide.md#address-lab/measurements:table:0?select=Component,Budget&top=1",
        ),
        ("section", "md://guide.md#address-lab/architecture"),
        (
            "numeric-stale",
            "md://guide.md#address-lab/architecture:figure:0",
        ),
    ] {
        map.insert(
            name.into(),
            json!({"markdown":markdown,"path":"guide.md","uri":uri}),
        );
    }
    map.insert("ambiguous".into(),json!({"markdown":markdown.replace("## Measurements","```\nrequest-retry\n+-------+\n| RETRY |\n+-------+\n```\n\n## Measurements"),"path":"guide.md","uri":"md://guide.md#address-lab/architecture:figure:request"}));
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_and_table_cell_use_existing_resolver() {
        let result = response(&examples()["named"].to_string());
        assert_eq!(result["ok"], true);
        assert_eq!(result["resolved"]["label"], "request-router");
        let cell = response(&examples()["cell"].to_string());
        assert_eq!(cell["ok"], true);
        assert!(cell["resolved"]["content"].as_str().unwrap().contains('8'));
    }
    #[test]
    fn ambiguity_and_numeric_staleness_remain_errors() {
        assert_eq!(
            response(&examples()["ambiguous"].to_string())["diagnostic"]["code"],
            "LabelAmbiguous"
        );
        assert_eq!(
            response(&examples()["numeric-stale"].to_string())["diagnostic"]["code"],
            "NumericUriStale"
        );
    }
    #[test]
    fn supplied_document_binding_never_loads_another_path() {
        let mut req = examples()["named"].clone();
        req["path"] = json!("other.md");
        assert_eq!(
            response(&req.to_string())["diagnostic"]["code"],
            "DocumentMismatch"
        );
        for path in ["/absolute.md", "../guide.md", "C:\\guide.md", "a//b.md"] {
            req["path"] = json!(path);
            assert_eq!(
                response(&req.to_string())["diagnostic"]["code"],
                "InvalidLogicalPath"
            );
        }
    }
    #[test]
    fn limits_and_strict_envelope_precede_resolution() {
        assert_eq!(response("{}")["ok"], false);
        assert_eq!(
            response(&" ".repeat(MAX_REQUEST + 1))["diagnostic"]["code"],
            "RequestLimit"
        );
        let mut req = examples()["named"].clone();
        req["markdown"] = json!("x".repeat(MAX_MARKDOWN + 1));
        assert_eq!(
            response(&req.to_string())["diagnostic"]["code"],
            "DocumentLimit"
        );
        req["markdown"] = json!("\n".repeat(8193));
        assert_eq!(
            response(&req.to_string())["diagnostic"]["code"],
            "DocumentLimit"
        );
    }
    #[test]
    fn portable_numbers_are_rejected_before_usize_fallback() {
        for uri in [
            "md://guide.md#address-lab:figure:4294967296",
            "md://guide.md#address-lab:table:0[row=18446744073709551616]",
            "md://guide.md#address-lab:table:0?skip=4294967296",
        ] {
            let mut req = examples()["named"].clone();
            req["uri"] = json!(uri);
            assert_eq!(
                response(&req.to_string())["diagnostic"]["code"],
                "PortableNumberLimit"
            );
        }
        let mut req = examples()["named"].clone();
        req["uri"] = json!("md://guide.md#address-lab:figure:4294967295");
        assert_eq!(
            response(&req.to_string())["diagnostic"]["code"],
            "ElementNotFound"
        );
    }
    #[test]
    fn serialization_is_deterministic_and_content_stays_inert() {
        for (_, req) in examples().as_object().unwrap() {
            assert_eq!(inspect(&req.to_string()), inspect(&req.to_string()));
        }
        let mut req = examples()["named"].clone();
        req["markdown"] = json!("# Notes\n\n<img src=x onerror=alert(1)>\n");
        req["uri"] = json!("md://guide.md#notes:text:0");
        assert_eq!(
            response(&req.to_string())["resolved"]["content"],
            "<img src=x onerror=alert(1)>"
        );
    }
}
