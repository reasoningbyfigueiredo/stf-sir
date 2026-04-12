mod common;

use std::collections::BTreeMap;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
struct ValidFixtureSpec {
    token_count: usize,
    relation_count: usize,
    root_types: &'static [&'static str],
}

fn valid_specs() -> BTreeMap<&'static str, ValidFixtureSpec> {
    BTreeMap::from([
        (
            "blockquote.md",
            ValidFixtureSpec {
                token_count: 2,
                relation_count: 1,
                root_types: &["blockquote"],
            },
        ),
        (
            "code_block.md",
            ValidFixtureSpec {
                token_count: 4,
                relation_count: 3,
                root_types: &["heading", "paragraph", "code_block", "paragraph"],
            },
        ),
        (
            "crlf.md",
            ValidFixtureSpec {
                token_count: 5,
                relation_count: 5,
                root_types: &["heading", "paragraph", "list"],
            },
        ),
        (
            "empty.md",
            ValidFixtureSpec {
                token_count: 0,
                relation_count: 0,
                root_types: &[],
            },
        ),
        (
            "heading.md",
            ValidFixtureSpec {
                token_count: 1,
                relation_count: 0,
                root_types: &["heading"],
            },
        ),
        (
            "heading_paragraph.md",
            ValidFixtureSpec {
                token_count: 2,
                relation_count: 1,
                root_types: &["heading", "paragraph"],
            },
        ),
        (
            "many_siblings.md",
            ValidFixtureSpec {
                token_count: 12,
                relation_count: 11,
                root_types: &[
                    "heading", "heading", "heading", "heading", "heading", "heading", "heading",
                    "heading", "heading", "heading", "heading", "heading",
                ],
            },
        ),
        (
            "multiline.md",
            ValidFixtureSpec {
                token_count: 3,
                relation_count: 2,
                root_types: &["heading", "paragraph", "paragraph"],
            },
        ),
        (
            "nested_list.md",
            ValidFixtureSpec {
                token_count: 8,
                relation_count: 8,
                root_types: &["heading", "list"],
            },
        ),
        (
            "paragraph.md",
            ValidFixtureSpec {
                token_count: 1,
                relation_count: 0,
                root_types: &["paragraph"],
            },
        ),
        (
            "unicode_nfkc.md",
            ValidFixtureSpec {
                token_count: 2,
                relation_count: 1,
                root_types: &["heading", "paragraph"],
            },
        ),
        (
            "whitespace.md",
            ValidFixtureSpec {
                token_count: 0,
                relation_count: 0,
                root_types: &[],
            },
        ),
        (
            "zero_width.md",
            ValidFixtureSpec {
                token_count: 2,
                relation_count: 1,
                root_types: &["heading", "paragraph"],
            },
        ),
    ])
}

#[test]
fn valid_fixture_matrix_compile_validate_graph_retention_and_determinism() -> Result<()> {
    let valid_dir = common::repo_root().join("tests/fixtures/valid");
    let fixtures = common::sorted_files_with_extension(&valid_dir, "md")?;
    let specs = valid_specs();

    assert_eq!(
        fixtures.len(),
        specs.len(),
        "fixture spec table drifted from tests/fixtures/valid"
    );

    for path in fixtures {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("fixture name should be valid UTF-8")?;
        let spec = specs
            .get(file_name)
            .with_context(|| format!("missing fixture spec for {file_name}"))?;
        let relative = path
            .strip_prefix(common::repo_root())
            .expect("fixture lives under repo root");
        let relative = relative.to_string_lossy().into_owned();

        let (artifact_first, yaml_first) = common::compile_and_serialize_fixture(&relative)?;
        let (_artifact_second, yaml_second) = common::compile_and_serialize_fixture(&relative)?;

        assert_eq!(
            yaml_first, yaml_second,
            "fixture {} did not compile deterministically",
            relative
        );

        let errors = common::validate_artifact_with_fixture_source(&artifact_first, &relative)?;
        assert!(
            errors.is_empty(),
            "fixture {} failed validation: {errors:#?}",
            relative
        );

        assert_eq!(
            artifact_first.document.token_count,
            artifact_first.ztokens.len()
        );
        assert_eq!(
            artifact_first.document.relation_count,
            artifact_first.relations.len()
        );
        assert_eq!(artifact_first.document.token_count, spec.token_count);
        assert_eq!(artifact_first.document.relation_count, spec.relation_count);

        common::assert_artifact_id_uniqueness(&artifact_first);
        common::assert_retention_unit_interval(&artifact_first);

        let root_types = artifact_first
            .ztokens
            .iter()
            .filter(|token| token.syntactic.parent_id.is_none())
            .map(|token| token.syntactic.node_type.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            root_types, spec.root_types,
            "fixture {} had unexpected root node types",
            relative
        );

        let graph = artifact_first.as_sir_graph();
        common::assert_graph_indexes_consistent(&graph);
        assert_eq!(graph.nodes.len(), artifact_first.document.token_count);
        assert_eq!(graph.edges.len(), artifact_first.document.relation_count);

        for edge in &graph.edges {
            assert!(
                graph.node(&edge.source).is_some(),
                "fixture {} emitted edge {} with missing source {}",
                relative,
                edge.id,
                edge.source
            );
            if edge.target.starts_with('z') {
                assert!(
                    graph.node(&edge.target).is_some(),
                    "fixture {} emitted edge {} with missing target {}",
                    relative,
                    edge.id,
                    edge.target
                );
            }
        }
    }

    Ok(())
}
