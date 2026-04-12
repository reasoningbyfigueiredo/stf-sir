mod common;

use anyhow::{Context, Result};
use stf_sir::compiler::validator;

#[test]
fn invalid_fixture_matrix_reports_expected_stable_error_codes() -> Result<()> {
    let invalid_dir = common::repo_root().join("tests/fixtures/invalid");
    let fixtures = common::sorted_files_with_extension(&invalid_dir, "zmd")?;

    assert!(
        !fixtures.is_empty(),
        "invalid fixture suite is empty at {}",
        invalid_dir.display()
    );

    for path in fixtures {
        let yaml = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let expected_path = path.with_extension("expected");
        let expected_code = std::fs::read_to_string(&expected_path)
            .with_context(|| format!("failed to read {}", expected_path.display()))?
            .trim()
            .to_string();

        let errors = validator::validate_yaml_str(&yaml, None);
        assert!(
            !errors.is_empty(),
            "invalid fixture {} unexpectedly validated",
            path.display()
        );

        let rules = errors.iter().map(|error| error.rule).collect::<Vec<_>>();
        assert!(
            rules.contains(&expected_code.as_str()),
            "fixture {} expected rule {}, got {:?}",
            path.display(),
            expected_code,
            rules
        );
    }

    Ok(())
}
