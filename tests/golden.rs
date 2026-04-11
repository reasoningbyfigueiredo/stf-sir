use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::tempdir;

#[test]
fn frozen_golden_fixture_matches_compiler_output_byte_for_byte() -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_md = repo_root.join("tests/golden/sample.md");
    let golden_zmd = repo_root.join("tests/golden/sample.zmd");
    let temp = tempdir()?;
    let output_path = temp.path().join("sample.zmd");

    let status = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
        .current_dir(repo_root)
        .arg("compile")
        .arg("tests/golden/sample.md")
        .arg("-o")
        .arg(&output_path)
        .status()
        .context("failed to execute stf-sir binary")?;

    assert!(status.success(), "compiler exited with non-zero status");

    let produced = fs::read_to_string(&output_path)?;
    let expected = fs::read_to_string(&golden_zmd).with_context(|| {
        format!(
            "golden {} is missing — regenerate with `cargo run -- compile {} -o {}`",
            golden_zmd.display(),
            golden_md.display(),
            golden_zmd.display()
        )
    })?;

    assert_eq!(
        produced, expected,
        "tests/golden/sample.zmd drifted — regenerate with \
         `cargo run -- compile tests/golden/sample.md -o tests/golden/sample.zmd`"
    );

    Ok(())
}
