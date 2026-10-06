// Shared by `build.rs` (through `include!`) and the binary, so the identity embedded at build time
// and the identity recomputed at review time cannot use different algorithms.

/// Source roots, relative to the reviewer engine directory, whose bytes define one reviewer build.
pub const BUILD_SOURCE_ROOTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "src",
    "../../workflow/engine/Cargo.toml",
    "../../workflow/engine/src",
];

/// Hex SHA-256 over every source file under `BUILD_SOURCE_ROOTS`, in sorted relative-path order.
pub fn build_source_sha256(engine_dir: &std::path::Path) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};

    fn collect(
        dir: &std::path::Path,
        rel: &str,
        out: &mut Vec<(String, std::path::PathBuf)>,
    ) -> std::io::Result<()> {
        let path = dir.join(rel);
        if path.is_dir() {
            for entry in std::fs::read_dir(&path)? {
                let name = entry?.file_name().to_string_lossy().into_owned();
                collect(dir, &format!("{rel}/{name}"), out)?;
            }
        } else {
            out.push((rel.to_string(), path));
        }
        Ok(())
    }

    let mut files = Vec::new();
    for root in BUILD_SOURCE_ROOTS {
        collect(engine_dir, root, &mut files)?;
    }
    files.sort();
    let mut hasher = Sha256::new();
    for (rel, path) in files {
        hasher.update(b"\0FILE\0");
        hasher.update(rel.as_bytes());
        hasher.update(b"\0");
        hasher.update(std::fs::read(path)?);
    }
    Ok(hex::encode(hasher.finalize()))
}
