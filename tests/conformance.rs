//! Data-driven conformance suite for STF-SIR v1.
//!
//! Every `.md` fixture under `tests/conformance/valid/` is:
//!
//! 1. compiled from scratch with the repo root as CWD (so `source.path`
//!    matches how the golden was originally generated);
//! 2. compared byte-for-byte against the checked-in `.zmd` golden;
//! 3. validated through the full pipeline (schema + semantic) with its
//!    Markdown source attached so rule 16 (source_text == slice) fires.
//!
//! The fixture set covers span-and-Unicode hardening targets: empty
//! documents, whitespace-only input, CRLF line endings, NFKC compatibility
//! forms (ﬃ U+FB03), zero-width spaces, CJK fullwidth, nested lists,
//! multi-line paragraphs, fenced code blocks, and heading depth.
//!
//! Invalid fixtures live in `tests/conformance/invalid_schema/` and
//! `tests/conformance/invalid_semantic/` and are exercised by
//! `invalid_cases_report_expected_rules`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use stf_sir::compiler::validator;

#[test]
fn every_valid_fixture_roundtrips_and_validates() -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let valid_dir = repo_root.join("tests/conformance/valid");
    let fixtures = collect_md_fixtures(&valid_dir)?;
    assert!(
        !fixtures.is_empty(),
        "conformance valid suite is empty at {}",
        valid_dir.display()
    );

    let temp_root = tempfile::tempdir()?;

    for md_path in fixtures {
        let rel = md_path
            .strip_prefix(repo_root)
            .expect("md fixture lives under repo root");
        let golden = md_path.with_extension("zmd");
        let golden_rel = golden
            .strip_prefix(repo_root)
            .expect("golden lives under repo root");

        let output_path = temp_root.path().join(rel.with_extension("zmd"));
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Step 1: compile from the repo root so source.path is recorded as a
        // repo-relative path and matches the golden.
        let status = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
            .current_dir(repo_root)
            .arg("compile")
            .arg(rel)
            .arg("-o")
            .arg(&output_path)
            .status()
            .with_context(|| format!("failed to compile {}", rel.display()))?;
        assert!(
            status.success(),
            "compiler exited non-zero on {}",
            rel.display()
        );

        // Step 2: byte-for-byte golden diff.
        let produced = fs::read_to_string(&output_path)?;
        let expected = fs::read_to_string(&golden).with_context(|| {
            format!(
                "golden {} is missing — regenerate with `cargo run -- compile {} -o {}`",
                golden_rel.display(),
                rel.display(),
                golden_rel.display()
            )
        })?;
        assert_eq!(
            produced,
            expected,
            "conformance golden drift detected for {}",
            rel.display()
        );

        // Step 3: schema + semantic validation against the checked-in golden,
        // with source bytes so rule 16 (L.source_text == source slice) fires.
        let source_bytes = fs::read(&md_path)?;
        let errors = validator::validate_yaml_str(&expected, Some(&source_bytes));
        assert!(
            errors.is_empty(),
            "golden {} failed validation: {errors:#?}",
            golden_rel.display()
        );
    }

    Ok(())
}

#[test]
fn invalid_cases_report_expected_rules() -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let roots = [
        repo_root.join("tests/conformance/invalid_schema"),
        repo_root.join("tests/conformance/invalid_semantic"),
    ];

    let mut visited = 0usize;
    for root in &roots {
        if !root.exists() {
            continue;
        }
        for entry in fs::read_dir(root)? {
            let case = entry?.path();
            if !case.is_dir() {
                continue;
            }
            let input = case.join("input.zmd");
            let expected_path = case.join("expected.txt");
            if !input.exists() || !expected_path.exists() {
                continue;
            }

            let yaml = fs::read_to_string(&input)?;
            let errors = validator::validate_yaml_str(&yaml, None);
            let rendered = errors
                .iter()
                .map(|e| format!("{}: {}", e.rule, e.message))
                .collect::<Vec<_>>()
                .join("\n");

            let expected = fs::read_to_string(&expected_path)?;
            for wanted in expected.lines().map(str::trim).filter(|s| !s.is_empty()) {
                assert!(
                    rendered.contains(wanted),
                    "case {}: missing expected token {:?} in validator output:\n{}",
                    case.display(),
                    wanted,
                    rendered
                );
            }
            visited += 1;
        }
    }

    assert!(
        visited > 0,
        "no invalid conformance cases were exercised — check tests/conformance/invalid_*"
    );
    Ok(())
}

fn collect_md_fixtures(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
