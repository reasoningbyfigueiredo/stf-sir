mod common;

use std::fs;

use anyhow::{Context, Result};
use tempfile::tempdir;

#[test]
fn golden_fixtures_match_compiler_output_byte_for_byte_and_validate() -> Result<()> {
    let golden_dir = common::repo_root().join("tests/golden");
    let fixtures = common::sorted_files_with_extension(&golden_dir, "md")?;
    assert!(
        fixtures.len() >= 6,
        "expected expanded golden suite in {}, found {} fixtures",
        golden_dir.display(),
        fixtures.len()
    );

    let temp = tempdir()?;

    for md_path in fixtures {
        let relative = md_path
            .strip_prefix(common::repo_root())
            .expect("golden fixture lives under repo root");
        let expected_path = md_path.with_extension("zmd");
        let output_path = temp.path().join(relative.with_extension("zmd"));
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        common::compile_cli_relative(relative, &output_path)?;

        let produced = fs::read_to_string(&output_path)?;
        let expected = fs::read_to_string(&expected_path).with_context(|| {
            format!(
                "golden {} is missing — regenerate with `cargo run -- compile {} -o {}`",
                expected_path.display(),
                relative.display(),
                expected_path.display()
            )
        })?;

        assert_eq!(
            produced,
            expected,
            "golden drift detected for {}",
            relative.display()
        );

        let source_bytes = fs::read(&md_path)?;
        let errors =
            stf_sir::compiler::validator::validate_yaml_str(&produced, Some(&source_bytes));
        assert!(
            errors.is_empty(),
            "golden {} failed validation: {errors:#?}",
            expected_path.display()
        );
    }

    Ok(())
}
