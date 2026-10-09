use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Manifest {
    release_id: String,
    binaries: std::collections::BTreeMap<String, Binary>,
}
#[derive(Deserialize)]
struct Binary {
    sha256: String,
    build_id: String,
}

pub fn verify_release_binary(name: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    let embedded_release = env!("CODEOS_RELEASE_ID");
    let embedded_build = env!("CODEOS_RELEASE_SOURCE_SHA256");
    anyhow::ensure!(
        embedded_release != "UNQUALIFIED" && embedded_build != "UNQUALIFIED",
        "unqualified release binary; use build-qualified-release.sh"
    );
    let executable = std::env::current_exe()?;
    let manifest_path = std::env::var_os("CODEOS_RELEASE_MANIFEST")
        .map(PathBuf::from)
        .or_else(|| {
            executable
                .ancestors()
                .map(|v| v.join("maintenance/release-manifest.json"))
                .find(|v| v.is_file())
        })
        .context("active CodeOS release manifest not found")?;
    verify(
        &manifest_path,
        &executable,
        name,
        embedded_release,
        embedded_build,
    )
}

fn verify(
    manifest_path: &Path,
    executable: &Path,
    name: &str,
    embedded_release: &str,
    embedded_build: &str,
) -> Result<()> {
    let manifest: Manifest = serde_json::from_slice(&std::fs::read(manifest_path)?)?;
    anyhow::ensure!(
        manifest.release_id == embedded_release,
        "release identity mismatch"
    );
    let expected = manifest
        .binaries
        .get(name)
        .with_context(|| format!("manifest omits {name}"))?;
    anyhow::ensure!(
        expected.build_id == embedded_build,
        "embedded build identity mismatch"
    );
    let actual = hex::encode(Sha256::digest(std::fs::read(executable)?));
    anyhow::ensure!(
        actual == expected.sha256,
        "executable does not match active CodeOS release manifest"
    );
    Ok(())
}
