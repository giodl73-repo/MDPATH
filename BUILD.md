# Build and verify the address workbench

The unpublished standalone `web/adapter` workspace uses MDPATH's public
`parse_document` and `resolve_in_doc` APIs with the default classifier. It does
not call filesystem resolution, fetch URI paths, implement a second grammar,
render Markdown as HTML or claim that names survive arbitrary renaming.

Install Rust 1.95.0 with rustfmt, Clippy and wasm32-unknown-unknown, Python 3.12,
Node 22 and wasm-bindgen-cli 0.2.127. Then:

```sh
cargo +1.95.0 test --locked
cargo +1.95.0 test --locked --manifest-path web/adapter/Cargo.toml --all-features
cargo +1.95.0 clippy --locked --manifest-path web/adapter/Cargo.toml --all-features --all-targets -- -D warnings
npm ci
npx playwright install chromium
python -B tools/build-pages.py
npm run test:pages
python -B tools/serve-pages.py
```

Open http://127.0.0.1:4177/MDPATH/. The builder explicitly selects Rust 1.95.0;
CI pins and checksums the bindings archive and uses the npm lockfile. The native
library's manifest and dependencies are unchanged. A newly retained root lockfile
pins qualification; the companion has its own lockfile and workspace. The prior
tracked numeric URI guard is merged with master, retaining master's lint fixes.
No new registry publication or external consumer adoption is claimed.

One request contains `markdown`, `path` and `uri` strings. `path` is a logical
binding, not a real filesystem root. It must be relative with forward slashes
and no empty, dot or parent segments, backslashes, colon or NUL. The parsed URI
path must equal it exactly. No other document is read. Core paths remain literal,
not percent-decoded. Default classification is distinct from PROOF's extensions.

The companion bounds Markdown to 128 KiB UTF-8 and 8,192 lines, URI to 4,096
bytes, logical path to 1,024 bytes and request JSON to 1 MiB. Decimal runs in
address fragments must parse as u32. This explicit admission rule avoids native
usize/wasm32 overflow differences, including the core's numeric-selector fallback
to zero. It also bounds numeric runs embedded in labels. It is not a change to
MDPATH grammar. Bounds are not an allocation quota. Startup/compute worker
deadlines are 12/10 seconds; local file reads have a five-second recovery deadline.
Edits/reset/retry fence stale file/worker completions.

Input downloads preserve opened valid UTF-8 and CRLF until editing, successful
replacement or choosing/restoring an example. Failed reads keep the previous
draft. Current-request downloads serialize current editor text. Native reports
retain Rust's response string, including typed errors. Resolved content stays
inert text. Line ranges are core values and may include fences; table projection
is transformed content. Section-only requests currently resolve the heading
alone. The page reports actual behavior rather than expanding it to match prose.

The builder generates native reference responses and a clean commit-labelled
source ZIP, source views, compiled WASM, exact dependency and Rust notices,
provenance and a per-file SHA-256 inventory. Dirty previews are labelled and use
staged source, not release evidence. Pages deploys only the browser-tested master
artifact. Hosted artifacts retain native references, browser results and review
screenshots; TRACKER records subsequent exact deployed-file and live-browser
custody. Existing native doctest ignores retain their original scope.
