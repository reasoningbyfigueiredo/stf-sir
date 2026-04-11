use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use stf_sir::compiler;
use stf_sir::model::ZToken;

#[test]
fn nfkc_fullwidth_fixture_normalizes_compatibility_forms() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/cjk_fullwidth.md")?;
    let paragraph = token(&artifact.ztokens, "z2")?;

    assert_eq!(
        paragraph.lexical.normalized_text,
        "Text with ABC fullwidth letters and 日本語 CJK characters."
    );
    assert_eq!(
        paragraph.semantic.gloss,
        "Text with ABC fullwidth letters and 日本語 CJK characters."
    );
    assert_eq!(paragraph.lexical.span.start_byte, 17);
    assert_eq!(paragraph.lexical.span.end_byte, 84);
    assert_eq!(paragraph.lexical.span.start_line, 3);
    assert_eq!(paragraph.lexical.span.end_line, 3);

    Ok(())
}

#[test]
fn ligature_fixture_unfolds_fb03_and_preserves_spans() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/nfkc_fb03.md")?;
    let paragraph = token(&artifact.ztokens, "z2")?;

    assert_eq!(
        paragraph.lexical.normalized_text,
        "The word offfice contains a ffi ligature that NFKC unfolds to three letters."
    );
    assert_eq!(
        paragraph.semantic.gloss,
        "The word offfice contains a ffi ligature that NFKC unfolds to three letters."
    );
    assert_eq!(paragraph.lexical.span.start_byte, 17);
    assert_eq!(paragraph.lexical.span.end_byte, 93);
    assert_eq!(paragraph.lexical.span.start_line, 3);
    assert_eq!(paragraph.lexical.span.end_line, 3);

    Ok(())
}

#[test]
fn zero_width_fixture_preserves_character_and_correct_span_coordinates() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/nfkc_zwsp.md")?;
    let paragraph = token(&artifact.ztokens, "z2")?;

    assert_eq!(
        paragraph.lexical.plain_text,
        "he\u{200B}llo contains a zero-width space."
    );
    assert_eq!(
        paragraph.lexical.normalized_text,
        "he\u{200B}llo contains a zero-width space."
    );
    assert_eq!(
        paragraph.semantic.gloss,
        "he\u{200B}llo contains a zero-width space."
    );
    assert_eq!(paragraph.lexical.span.start_byte, 14);
    assert_eq!(paragraph.lexical.span.end_byte, 51);
    assert_eq!(paragraph.lexical.span.start_line, 3);
    assert_eq!(paragraph.lexical.span.end_line, 3);

    Ok(())
}

#[test]
fn multiline_paragraph_fixture_tracks_soft_break_lines_and_collapsed_normalization() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/multiline_paragraph.md")?;
    let paragraph = token(&artifact.ztokens, "z2")?;

    assert_eq!(
        paragraph.lexical.plain_text,
        "This paragraph spans\nmultiple source lines\nwith soft breaks only."
    );
    assert_eq!(
        paragraph.lexical.normalized_text,
        "This paragraph spans multiple source lines with soft breaks only."
    );
    assert_eq!(
        paragraph.semantic.gloss,
        "This paragraph spans multiple source lines with soft breaks only."
    );
    assert_eq!(paragraph.lexical.span.start_byte, 15);
    assert_eq!(paragraph.lexical.span.end_byte, 80);
    assert_eq!(paragraph.lexical.span.start_line, 3);
    assert_eq!(paragraph.lexical.span.end_line, 5);

    Ok(())
}

#[test]
fn crlf_fixture_preserves_byte_offsets_and_one_based_line_numbers() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/crlf.md")?;
    let heading = token(&artifact.ztokens, "z1")?;
    let paragraph = token(&artifact.ztokens, "z2")?;
    let list = token(&artifact.ztokens, "z3")?;
    let first_item = token(&artifact.ztokens, "z4")?;
    let second_item = token(&artifact.ztokens, "z5")?;

    assert_eq!(heading.lexical.span.start_byte, 0);
    assert_eq!(heading.lexical.span.end_byte, 7);
    assert_eq!(heading.lexical.span.start_line, 1);
    assert_eq!(heading.lexical.span.end_line, 1);

    assert_eq!(paragraph.lexical.span.start_byte, 11);
    assert_eq!(paragraph.lexical.span.end_byte, 39);
    assert_eq!(paragraph.lexical.span.start_line, 3);
    assert_eq!(paragraph.lexical.span.end_line, 3);

    assert_eq!(list.lexical.normalized_text, "one two");
    assert_eq!(list.semantic.gloss, "one two");
    assert_eq!(list.lexical.span.start_byte, 43);
    assert_eq!(list.lexical.span.end_byte, 55);
    assert_eq!(list.lexical.span.start_line, 5);
    assert_eq!(list.lexical.span.end_line, 6);

    assert_eq!(first_item.lexical.span.start_byte, 43);
    assert_eq!(first_item.lexical.span.end_byte, 48);
    assert_eq!(first_item.lexical.span.start_line, 5);
    assert_eq!(first_item.lexical.span.end_line, 5);

    assert_eq!(second_item.lexical.span.start_byte, 50);
    assert_eq!(second_item.lexical.span.end_byte, 55);
    assert_eq!(second_item.lexical.span.start_line, 6);
    assert_eq!(second_item.lexical.span.end_line, 6);

    Ok(())
}

#[test]
fn empty_document_fixture_emits_no_tokens_and_zero_length_source() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/empty.md")?;

    assert_eq!(artifact.source.length_bytes, 0);
    assert!(artifact.ztokens.is_empty());
    assert!(artifact.relations.is_empty());
    assert_eq!(artifact.document.token_count, 0);
    assert_eq!(artifact.document.relation_count, 0);

    Ok(())
}

#[test]
fn whitespace_only_fixture_emits_no_tokens_but_preserves_source_length() -> Result<()> {
    let artifact = compile_fixture("tests/conformance/valid/whitespace_only.md")?;

    assert_eq!(artifact.source.length_bytes, 11);
    assert!(artifact.ztokens.is_empty());
    assert!(artifact.relations.is_empty());
    assert_eq!(artifact.document.token_count, 0);
    assert_eq!(artifact.document.relation_count, 0);

    Ok(())
}

fn compile_fixture(relative_path: &str) -> Result<stf_sir::model::Artifact> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = repo_root.join(relative_path);
    let source = fs::read_to_string(&path)
        .with_context(|| format!("failed to read fixture {}", path.display()))?;
    compiler::compile_markdown(&source, Some(Path::new(relative_path))).map_err(Into::into)
}

fn token<'a>(ztokens: &'a [ZToken], id: &str) -> Result<&'a ZToken> {
    ztokens
        .iter()
        .find(|token| token.id == id)
        .with_context(|| format!("missing ztoken {id}"))
}
