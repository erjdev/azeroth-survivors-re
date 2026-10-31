//! Guard the scripts shipped by this checkout's standalone browser launcher.
//! The realm service is a separate deployment, not a workspace crate here.
//! Source modules must be copied into dist; only wasm-bindgen glue is generated.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every `./something.js` in a page: `import … from './x.js'`, `import('./x.js')`, and the
/// import map's `"./wasi_stubs.js"`. One scan for all three — the quoting is what they share.
fn referenced_scripts(page: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for quote in ['\'', '"'] {
        let needle = format!("{quote}./");
        let mut rest = page;
        while let Some(start) = rest.find(&needle) {
            rest = &rest[start + needle.len()..];
            if let Some(end) = rest.find(quote) {
                let name = &rest[..end];
                if name.ends_with(".js") && !name.contains('/') {
                    found.insert(name.to_string());
                }
            }
        }
    }
    found
}

#[test]
fn every_page_script_that_lives_in_web_is_copied_into_dist() {
    let build = read("scripts/web-build.sh");
    let cp = build
        .lines()
        .find(|l| l.trim_start().starts_with("cp web/"))
        .expect("web-build.sh has the cp line that populates web/dist");
    let page = "web/index.html";
    for name in referenced_scripts(&read(page)) {
        // wasm-bindgen derives this module name from the input wasm basename.
        // Every other import must exist as source; a typo must fail the gate.
        if name == "wenilla.js" {
            assert!(build.contains("/wenilla.wasm\""));
            assert!(build
                .contains("wasm-bindgen \"${bindgen_args[@]}\" --out-dir \"${DIST}\" \"${WASM}\""));
            continue;
        }
        assert!(
            root().join("web").join(&name).is_file(),
            "{page}: missing source module {name}"
        );
        assert!(
            cp.contains(&format!("web/{name}")),
            "{page} imports ./{name}, but web-build.sh does not copy it into dist: {cp}"
        );
    }
}

#[test]
fn platform_storage_is_optional_and_off_the_boot_path() {
    // The standalone launcher uses platform.js for durable storage. It has no
    // separate HTML fullscreen button; failure of this helper must not block boot.
    let text = read("web/index.html");
    assert!(
        text.contains("import('./platform.js')"),
        "platform.js must remain a dynamic import"
    );
    assert!(
        text.contains(".then(({ requestPersistentStorage }) => requestPersistentStorage())")
            && text.contains(".catch((e) => console.warn('platform.js:', e))"),
        "request persistence and handle an optional platform module failure"
    );
}
