// Embeds the reviewer source identity so the Review Readiness Gate can refuse a stale binary.
include!("src/build_identity.rs");

fn main() {
    let engine_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    for root in BUILD_SOURCE_ROOTS {
        println!("cargo:rerun-if-changed={root}");
    }
    let sha = build_source_sha256(&engine_dir).expect("hash reviewer build sources");
    println!("cargo:rustc-env=CODEOS_REVIEWER_BUILD_SHA256={sha}");
}
