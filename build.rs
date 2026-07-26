// build.rs: make the zukan-assets payload (data/ + icons/) available to
// rust-embed, and tell it where via the ASSETS_DIR env var.
//
// Two locations, picked in order:
//   1. In-tree `assets/{data,icons}` — populated by `make download` / `make
//      assets`. Preferred for dev: rust-embed reads it directly, no copy.
//   2. `OUT_DIR/zukan-assets/` — cold-build fallback for when the source tree
//      isn't writable or isn't pre-populated: `cargo install`, `cargo publish`
//      verification, or a fresh clone built without `make download`.
//

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

const RELEASE_URL: &str =
    "https://github.com/lazywalker/zukan-assets/releases/latest/download/zukan-assets-bin.tar.gz";

/// Hard ceiling for the whole download (DNS + connect + body). Avoids hanging
/// forever on a flaky/sandboxed network; fail fast so the user sees an error
/// they can act on (`make download`, retry, fix connectivity).
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

/// The two subdirectories rust-embed actually consumes.
const PAYLOAD_SUBDIRS: &[&str] = &["data", "icons"];

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let in_tree = manifest_dir.join("assets");

    // Prefer the in-tree payload (dev). Fall back to OUT_DIR for cold builds
    // where the source tree isn't writable (cargo install / publish verify).
    let assets_dir = if has_payload(&in_tree) {
        in_tree
    } else {
        let staged = out_dir.join("zukan-assets");
        if !has_payload(&staged) {
            download_and_extract(&staged);
            require_payload(&staged);
        }
        staged
    };

    // Watch the payload so a manual re-download (rm + rebuild, or
    // `git clean -Xdf assets/`) triggers rust-embed to re-read.
    for sub in PAYLOAD_SUBDIRS {
        println!("cargo:rerun-if-changed={}/{}", assets_dir.display(), sub);
    }
    println!("cargo:rustc-env=ASSETS_DIR={}", assets_dir.display());
}

/// True if `dir` contains both `data/` and `icons/` subdirectories.
fn has_payload(dir: &Path) -> bool {
    PAYLOAD_SUBDIRS.iter().all(|sub| dir.join(sub).is_dir())
}

/// Bail out clearly if the directory doesn't actually contain data/ + icons/.
fn require_payload(path: &Path) {
    if !has_payload(path) {
        panic!(
            "assets directory ({}) is missing data/ or icons/ after download; \
             run `make download`, or check your network connection.",
            path.display()
        );
    }
}

/// Download the Release tar.gz and extract it into `dest` in-process.
///
/// Extracts into a `.download-partials` subdir first, then moves data/ and
/// icons/ into place, so a failed download (network error, partial stream)
/// never leaves a half-populated `dest/data` that the next build would trust
/// as complete.
fn download_and_extract(dest: &Path) {
    eprintln!("[zukan] downloading assets from {RELEASE_URL}");
    fs::create_dir_all(dest).unwrap_or_else(|e| panic!("failed to create {}: {e}", dest.display()));

    let tmp = dest.join(".download-partials");
    if tmp.exists() {
        fs::remove_dir_all(&tmp).ok();
    }
    fs::create_dir_all(&tmp).unwrap_or_else(|e| panic!("failed to create {}: {e}", tmp.display()));

    // rustls TLS, gzip-transparent decode so we get the raw tar stream.
    // Capped by DOWNLOAD_TIMEOUT so a blocked port fails fast rather than
    // hanging the build indefinitely.
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .build();
    let agent = config.new_agent();
    let response = agent
        .get(RELEASE_URL)
        .header("accept-encoding", "gzip")
        .call()
        .unwrap_or_else(|e| {
            // Clean up the partial extraction dir so the next build retries.
            fs::remove_dir_all(&tmp).ok();
            panic!(
                "assets download failed ({RELEASE_URL}): {e}\n\
                 hint: run `make download` to fetch via curl instead, \
                 or check your network connection."
            )
        });

    let reader = response.into_body().into_reader();
    let gz = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(&tmp).unwrap_or_else(|e| {
        // Clean up the partial extraction so the next build retries cleanly.
        fs::remove_dir_all(&tmp).ok();
        panic!("failed to unpack assets into {}: {e}", tmp.display());
    });

    // Move the extracted top-level dirs (data/, icons/) into `dest`.
    for sub in PAYLOAD_SUBDIRS {
        let from = tmp.join(sub);
        let to = dest.join(sub);
        if from.is_dir() {
            if to.exists() {
                fs::remove_dir_all(&to).ok();
            }
            fs::rename(&from, &to).unwrap_or_else(|e| {
                panic!("failed to move {} → {}: {e}", from.display(), to.display())
            });
        }
    }
    fs::remove_dir_all(&tmp).ok();
}
