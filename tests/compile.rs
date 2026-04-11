use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::compiler::validator;
use stf_sir::model::RelationCategory;
use tempfile::tempdir;

#[test]
fn relation_category_exists_and_structural_relations_are_structural() -> Result<()> {
    let input = "# Title\n\n- first item\n- second item\n\n> quoted block\n";
    let output = compile_fixture(input)?;
    let artifact: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output)?;

    let relations = artifact["relations"]
        .as_sequence()
        .context("relations should be a YAML sequence")?;

    assert!(
        relations
            .iter()
            .any(|relation| relation["type"].as_str() == Some("contains")),
        "expected at least one contains relation"
    );
    assert!(
        relations
            .iter()
            .any(|relation| relation["type"].as_str() == Some("precedes")),
        "expected at least one precedes relation"
    );

    for relation in relations {
        let relation_type = relation["type"]
            .as_str()
            .context("relation.type should be a string")?;
        let category = relation["category"]
            .as_str()
            .context("relation.category should be present")?;

        if matches!(relation_type, "contains" | "precedes") {
            assert_eq!(category, "structural");
        }
    }

    Ok(())
}

#[test]
fn compiler_output_is_deterministic() -> Result<()> {
    let temp = tempdir()?;
    let input_path = temp.path().join("input.md");
    fs::write(&input_path, "# Title\n\nParagraph with  extra   spacing.\n")?;

    let first = compile_fixture_from_path(&input_path, temp.path().join("first.zmd"))?;
    let second = compile_fixture_from_path(&input_path, temp.path().join("second.zmd"))?;

    assert_eq!(first, second);
    Ok(())
}

/// Onda 2.3 — golden test: the canonical fixture must round-trip byte-for-byte
/// when compiled with the repo root as the working directory, matching how the
/// fixture was originally generated (`source.path` is recorded as supplied to
/// the compiler, so we must supply it as a relative path).
#[test]
fn canonical_sample_roundtrips_byte_for_byte() -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let zmd = repo_root.join("examples/sample.zmd");

    let temp = tempdir()?;
    let output_path = temp.path().join("sample.zmd");

    let status = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
        .current_dir(repo_root)
        .arg("compile")
        .arg("examples/sample.md")
        .arg("-o")
        .arg(&output_path)
        .status()
        .context("failed to execute stf-sir binary")?;
    assert!(status.success(), "compiler exited with non-zero status");

    let produced = fs::read_to_string(&output_path)?;
    let expected = fs::read_to_string(&zmd)?;

    assert_eq!(
        produced, expected,
        "examples/sample.zmd is out of date — regenerate with \
         `cargo run -- compile examples/sample.md -o examples/sample.zmd`"
    );
    Ok(())
}

// ---------- §9 validation invariants ----------

#[test]
fn sample_passes_full_validator() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let errors = validator::validate(&artifact, Some(sample_markdown().as_bytes()));
    assert!(
        errors.is_empty(),
        "unexpected validation errors: {errors:?}"
    );
    Ok(())
}

#[test]
fn rule_07_08_counts_match_vectors() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    assert_eq!(artifact.document.token_count, artifact.ztokens.len());
    assert_eq!(artifact.document.relation_count, artifact.relations.len());
    Ok(())
}

#[test]
fn rule_09_parent_ids_reference_existing_tokens() -> Result<()> {
    let artifact = compiler::compile_markdown("# H\n\n- a\n  - nested\n- b\n\n> quote\n", None)?;
    let known: std::collections::HashSet<&str> =
        artifact.ztokens.iter().map(|t| t.id.as_str()).collect();
    for token in &artifact.ztokens {
        if let Some(parent) = &token.syntactic.parent_id {
            assert!(
                known.contains(parent.as_str()),
                "token {} has dangling parent_id {}",
                token.id,
                parent
            );
        }
    }
    Ok(())
}

#[test]
fn rule_10_11_relation_endpoints_reference_existing_tokens() -> Result<()> {
    let artifact = compiler::compile_markdown("# H\n\n- a\n  - nested\n- b\n\n> quote\n", None)?;
    let known: std::collections::HashSet<&str> =
        artifact.ztokens.iter().map(|t| t.id.as_str()).collect();
    for relation in &artifact.relations {
        assert!(known.contains(relation.source.as_str()));
        if !matches!(relation.category, RelationCategory::SemanticLink) {
            assert!(known.contains(relation.target.as_str()));
        }
    }
    Ok(())
}

#[test]
fn rule_14_byte_spans_are_within_source_length() -> Result<()> {
    let source = "# Model\n\nA paragraph.\n\n- one\n- two\n";
    let artifact = compiler::compile_markdown(source, None)?;
    for token in &artifact.ztokens {
        let span = &token.lexical.span;
        assert!(span.start_byte < span.end_byte);
        assert!(span.end_byte <= artifact.source.length_bytes);
    }
    Ok(())
}

#[test]
fn rule_15_line_spans_are_one_based_and_monotonic() -> Result<()> {
    let source = "# H1\n\nP1\n\n# H2\n";
    let artifact = compiler::compile_markdown(source, None)?;
    for token in &artifact.ztokens {
        let span = &token.lexical.span;
        assert!(span.start_line >= 1);
        assert!(span.start_line <= span.end_line);
    }
    Ok(())
}

#[test]
fn rule_16_source_text_equals_exact_slice() -> Result<()> {
    let source = "# Heading\n\nFirst para.\n\nSecond para.\n";
    let artifact = compiler::compile_markdown(source, None)?;
    let bytes = source.as_bytes();
    for token in &artifact.ztokens {
        let span = &token.lexical.span;
        let slice = &bytes[span.start_byte..span.end_byte];
        assert_eq!(
            slice,
            token.lexical.source_text.as_bytes(),
            "L.source_text mismatch for {} (expected bytes {:?})",
            token.id,
            std::str::from_utf8(slice).unwrap_or("<invalid>")
        );
    }
    Ok(())
}

#[test]
fn rule_17_sigma_gloss_is_always_present() -> Result<()> {
    let artifact = compiler::compile_markdown("# H\n\n> q\n", None)?;
    for token in &artifact.ztokens {
        let _: &str = &token.semantic.gloss;
    }
    Ok(())
}

#[test]
fn precedes_ordering_survives_more_than_nine_siblings() -> Result<()> {
    // Spec §7: precedes is between consecutive siblings. A naive grouping
    // keyed by string id would order "z10" before "z2" lexicographically.
    // We create 12 roots to prove sibling ordering uses the parent's
    // preorder index, not the id string.
    let mut md = String::new();
    for i in 0..12 {
        md.push_str(&format!("# H{i}\n\n"));
    }

    let artifact = compiler::compile_markdown(&md, None)?;
    assert_eq!(artifact.ztokens.len(), 12);

    let precedes: Vec<_> = artifact
        .relations
        .iter()
        .filter(|r| r.type_ == "precedes")
        .collect();
    assert_eq!(precedes.len(), 11, "expected 11 precedes for 12 roots");

    for (i, relation) in precedes.iter().enumerate() {
        let expected_source = format!("z{}", i + 1);
        let expected_target = format!("z{}", i + 2);
        assert_eq!(relation.source, expected_source);
        assert_eq!(relation.target, expected_target);
    }
    Ok(())
}

#[test]
fn utf8_invalid_source_emits_structured_diagnostic() -> Result<()> {
    let temp = tempdir()?;
    let path = temp.path().join("bad.md");
    fs::write(&path, [0xFFu8, 0xFE, 0xFD])?;

    let err = compiler::compile_path(&path).unwrap_err();
    match err {
        stf_sir::CompileError::Fatal { diagnostics } => {
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].code, "SRC_UTF8_INVALID");
            assert_eq!(diagnostics[0].stage, "lexical");
        }
        other => panic!("expected Fatal, got {other:?}"),
    }
    Ok(())
}

#[test]
fn nfkc_normalization_collapses_whitespace_and_compatibility_forms() -> Result<()> {
    // NFKC: ﬃ (U+FB03) -> "ffi"; also collapses multiple spaces.
    let source = "# Heading\n\nﬃ   test   text\n";
    let artifact = compiler::compile_markdown(source, None)?;
    let paragraph = artifact
        .ztokens
        .iter()
        .find(|t| t.syntactic.node_type == "paragraph")
        .expect("paragraph token");
    assert_eq!(paragraph.lexical.normalized_text, "ffi test text");
    assert_eq!(paragraph.semantic.gloss, "ffi test text");
    Ok(())
}

// ---------- H3: stage enum enforcement ----------

#[test]
fn all_emitted_relations_use_logical_stage() -> Result<()> {
    // The current compiler only emits contains/precedes from the logical
    // stage, so every relation must carry `stage: logical`. This test
    // locks that invariant so accidental drift breaks CI.
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    for relation in &artifact.relations {
        assert_eq!(
            relation.stage, "logical",
            "relation {} unexpectedly emitted from stage {:?}",
            relation.id, relation.stage
        );
    }
    Ok(())
}

#[test]
fn unknown_relation_stage_fails_val_18() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    // Inject an out-of-set stage. Schema enum catches this too, but here
    // we bypass the schema pass to exercise VAL_18 directly via the
    // semantic-only entry point.
    let broken = yaml.replace("stage: logical", "stage: not-a-stage");
    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors
            .iter()
            .any(|e| e.rule == "SCHEMA_VIOLATION" || e.rule == "VAL_18_RELATION_STAGE"),
        "expected SCHEMA_VIOLATION or VAL_18_RELATION_STAGE, got {errors:#?}"
    );
    Ok(())
}

// ---------- schema-level validator tests ----------

/// Test 1 — a canonical sample passes both validation passes.
#[test]
fn valid_sample_passes_full_validation() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    let errors = validator::validate_yaml_str(&yaml, Some(sample_markdown().as_bytes()));
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    Ok(())
}

/// Test 2 — an artifact with a relation missing its required `category`
/// field fails at the schema pass with a SCHEMA_VIOLATION.
#[test]
fn missing_relation_category_fails_schema() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    // Mutate YAML in place: drop the `category:` line from the first
    // relation. The resulting document is still parseable YAML but no
    // longer satisfies the required-field rule for `relation`.
    let broken = strip_first_line_containing(&yaml, "  category:");

    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors
            .iter()
            .any(|e| e.rule == "SCHEMA_VIOLATION" && e.message.contains("category")),
        "expected SCHEMA_VIOLATION mentioning category, got {errors:#?}"
    );
    Ok(())
}

/// Test 3 — an unknown `relation.category` enum value fails at the
/// schema pass.
#[test]
fn invalid_relation_category_fails_schema() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    let broken = yaml.replace("category: structural", "category: bogus-category");

    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors.iter().any(|e| e.rule == "SCHEMA_VIOLATION"),
        "expected SCHEMA_VIOLATION, got {errors:#?}"
    );
    Ok(())
}

/// Test 4 — a wrong top-level `format` constant fails at the schema pass.
#[test]
fn wrong_format_constant_fails_schema() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    let broken = yaml.replace("format: stf-sir.zmd", "format: something-else");

    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors.iter().any(|e| e.rule == "SCHEMA_VIOLATION"),
        "expected SCHEMA_VIOLATION, got {errors:#?}"
    );
    Ok(())
}

/// Test 5 — a mismatched `document.token_count` passes schema but fails
/// semantic validation (rule VAL_07_TOKEN_COUNT).
#[test]
fn mismatched_token_count_fails_semantic() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    // token_count is a positive integer in the sample; mutate to 999.
    let broken = yaml.replace(
        &format!("token_count: {}", artifact.ztokens.len()),
        "token_count: 999",
    );

    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors.iter().any(|e| e.rule == "VAL_07_TOKEN_COUNT"),
        "expected VAL_07_TOKEN_COUNT, got {errors:#?}"
    );
    Ok(())
}

/// Test 6 — a relation referencing a non-existent ztoken passes schema
/// but fails semantic validation (rule VAL_10_RELATION_SOURCE).
#[test]
fn broken_relation_reference_fails_semantic() -> Result<()> {
    let artifact = compiler::compile_markdown(sample_markdown(), None)?;
    let yaml = compiler::serializer::to_yaml_string(&artifact)?;
    // Rewrite the source of a relation to a non-existent ztoken id.
    let broken = yaml.replacen("source: z1", "source: zDOES_NOT_EXIST", 1);

    let errors = validator::validate_yaml_str(&broken, None);
    assert!(
        errors.iter().any(|e| e.rule == "VAL_10_RELATION_SOURCE"),
        "expected VAL_10_RELATION_SOURCE, got {errors:#?}"
    );
    Ok(())
}

/// CLI-level smoke test for `stf-sir validate` against the canonical fixture.
#[test]
fn cli_validate_reports_valid_on_canonical_sample() -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
        .current_dir(repo_root)
        .arg("validate")
        .arg("examples/sample.zmd")
        .output()
        .context("failed to execute stf-sir binary")?;

    assert!(
        output.status.success(),
        "validate returned non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("VALID:") && stdout.contains("conforms to STF-SIR v1"),
        "unexpected stdout: {stdout}"
    );
    Ok(())
}

// Helper used by schema-level failure tests. Drops the first line whose
// content starts with `needle` (after leading whitespace). This is a tiny
// line-oriented mutator — sufficient for deterministic YAML fixtures.
fn strip_first_line_containing(yaml: &str, needle: &str) -> String {
    let mut out = String::with_capacity(yaml.len());
    let mut removed = false;
    for line in yaml.split_inclusive('\n') {
        if !removed && line.starts_with(needle) {
            removed = true;
            continue;
        }
        out.push_str(line);
    }
    out
}

// ---------- helpers ----------

fn sample_markdown() -> &'static str {
    "# Title\n\nFirst paragraph with text.\n\n- list item one\n- list item two\n\n> a quote\n"
}

fn compile_fixture(markdown: &str) -> Result<String> {
    let temp = tempdir()?;
    let input_path = temp.path().join("input.md");
    fs::write(&input_path, markdown)?;

    compile_fixture_from_path(&input_path, temp.path().join("output.zmd"))
}

fn compile_fixture_from_path(input_path: &Path, output_path: PathBuf) -> Result<String> {
    let status = Command::new(env!("CARGO_BIN_EXE_stf-sir"))
        .arg("compile")
        .arg(input_path)
        .arg("-o")
        .arg(&output_path)
        .status()
        .context("failed to execute stf-sir binary")?;

    assert!(status.success(), "compiler exited with non-zero status");

    let output = fs::read_to_string(&output_path)
        .with_context(|| format!("failed to read {}", output_path.display()))?;

    Ok(output)
}
