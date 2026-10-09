fn main() {
    println!("cargo:rerun-if-env-changed=CODEOS_RELEASE_ID");
    println!("cargo:rerun-if-env-changed=CODEOS_RELEASE_SOURCE_SHA256");
    println!(
        "cargo:rustc-env=CODEOS_RELEASE_ID={}",
        std::env::var("CODEOS_RELEASE_ID").unwrap_or_else(|_| "UNQUALIFIED".into())
    );
    println!(
        "cargo:rustc-env=CODEOS_RELEASE_SOURCE_SHA256={}",
        std::env::var("CODEOS_RELEASE_SOURCE_SHA256").unwrap_or_else(|_| "UNQUALIFIED".into())
    );
}
