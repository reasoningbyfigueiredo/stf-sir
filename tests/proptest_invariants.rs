//! Property tests for STF-SIR compiler invariants.
//!
//! These tests generate small Markdown documents from a restricted grammar
//! (headings, paragraphs, and bullet lists composed of ASCII lowercase
//! words) and assert the four load-bearing invariants from spec §9 that
//! span-and-UTF-8 bugs are most likely to break:
//!
//! * **rule 5** — every ztoken id is unique,
//! * **rule 7/8** — `document.token_count` and `relation_count` match the
//!   length of the serialized vectors,
//! * **rule 14** — every byte span satisfies
//!   `0 ≤ start_byte < end_byte ≤ source.length_bytes`,
//! * **rule 16** — `L.source_text` is byte-equal to the source slice it
//!   identifies.
//!
//! The generator is intentionally modest: it is meant to exercise the hot
//! paths (heading/paragraph/list combinations with varying whitespace) and
//! hand the deeper edge cases to the curated fixtures in
//! `tests/conformance/valid/`. Fuzzing proper is out of scope for v1.

use std::collections::HashSet;

use proptest::prelude::*;
use stf_sir::compiler;
use stf_sir::compiler::validator;

/// Produce a single ASCII word of 1..6 lowercase characters.
fn word() -> impl Strategy<Value = String> {
    "[a-z]{1,6}".prop_map(String::from)
}

/// Produce a block: heading, paragraph, or short bullet list.
fn block() -> impl Strategy<Value = String> {
    let heading = (1usize..=3, prop::collection::vec(word(), 1..4))
        .prop_map(|(level, words)| format!("{} {}", "#".repeat(level), words.join(" ")));
    let paragraph = prop::collection::vec(word(), 1..8).prop_map(|words| words.join(" "));
    let bullet_list = prop::collection::vec(
        prop::collection::vec(word(), 1..4).prop_map(|ws| format!("- {}", ws.join(" "))),
        1..4,
    )
    .prop_map(|items| items.join("\n"));

    prop_oneof![heading, paragraph, bullet_list]
}

/// Assemble 1..5 blocks separated by blank lines and terminated by `\n`.
fn document() -> impl Strategy<Value = String> {
    prop::collection::vec(block(), 1..5).prop_map(|blocks| {
        let mut out = blocks.join("\n\n");
        out.push('\n');
        out
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        // Keep the suite fast in CI; the curated fixtures handle breadth.
        cases: 128,
        .. ProptestConfig::default()
    })]

    #[test]
    fn compiled_artifacts_satisfy_core_invariants(src in document()) {
        let artifact = compiler::compile_markdown(&src, None)
            .expect("curated markdown must compile");

        // Rule 5 — unique ztoken ids.
        let mut ids: HashSet<&str> = HashSet::new();
        for token in &artifact.ztokens {
            prop_assert!(
                ids.insert(token.id.as_str()),
                "duplicate ztoken id {:?}",
                token.id
            );
        }

        // Rules 7 and 8 — counts in sync with the serialized vectors.
        prop_assert_eq!(artifact.document.token_count, artifact.ztokens.len());
        prop_assert_eq!(artifact.document.relation_count, artifact.relations.len());

        // Rule 14 — byte span bounds.
        for token in &artifact.ztokens {
            let span = &token.lexical.span;
            prop_assert!(span.start_byte < span.end_byte);
            prop_assert!(span.end_byte <= artifact.source.length_bytes);
        }

        // Rule 16 — L.source_text is byte-equal to the source slice.
        let bytes = src.as_bytes();
        for token in &artifact.ztokens {
            let span = &token.lexical.span;
            prop_assert!(span.end_byte <= bytes.len());
            let slice = &bytes[span.start_byte..span.end_byte];
            prop_assert_eq!(
                slice,
                token.lexical.source_text.as_bytes(),
                "L.source_text mismatch for ztoken {}",
                token.id
            );
        }

        // Full pipeline must accept the artifact as well.
        let yaml = compiler::serializer::to_yaml_string(&artifact)
            .expect("serialization is infallible for owned model");
        let errors = validator::validate_yaml_str(&yaml, Some(bytes));
        prop_assert!(
            errors.is_empty(),
            "validator rejected a generated artifact: {:?}",
            errors
        );
    }
}
