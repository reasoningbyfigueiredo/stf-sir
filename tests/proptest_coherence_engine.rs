//! Property-based tests for the coherence engine, formula layer, grounding,
//! and bridge.
//!
//! All generators are intentionally small and controlled to keep CI fast
//! while covering the hot paths that unit tests cannot exhaustively enumerate.

use std::collections::HashSet;

use proptest::prelude::*;
use stf_sir::compiler::coherence::{FormulaCoherenceChecker, LogicalCoherenceChecker};
use stf_sir::compiler::grounding::{GroundingChecker, ProvenanceGroundingChecker};
use stf_sir::model::{Formula, Statement, Theory, artifact_to_theory};
use stf_sir::retention::CoherenceRetention;

// ---------------------------------------------------------------------------
// Generators

/// Small formula-like string: alphanumeric + space, 0–20 chars.
fn formula_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,20}".prop_map(String::from)
}

/// Short uppercase atom (1–6 letters).
fn atom_name() -> impl Strategy<Value = String> {
    "[A-Z]{1,6}".prop_map(String::from)
}

/// Compile a small markdown document (1–4 word paragraphs, 1–5 blocks).
fn small_markdown() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::collection::vec("[a-z]{1,6}".prop_map(String::from), 1..4usize)
            .prop_map(|ws| ws.join(" ")),
        1..5usize,
    )
    .prop_map(|blocks| blocks.join("\n\n") + "\n")
}

// ---------------------------------------------------------------------------
// P1: Formula::parse never panics on arbitrary short strings.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn formula_parse_never_panics(s in "[\\x20-\\x7E]{0,40}") {
        // Printable ASCII, 0–40 chars. Must not panic.
        let _ = Formula::parse(&s);
    }
}

// ---------------------------------------------------------------------------
// P2: Formula::parse is deterministic (same input → same output, always).

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn formula_parse_is_deterministic(s in formula_string()) {
        let r1 = Formula::parse(&s);
        let r2 = Formula::parse(&s);
        prop_assert_eq!(r1, r2, "parse must be deterministic for {:?}", s);
    }
}

// ---------------------------------------------------------------------------
// P3: Atom contradicts its negation; negation contradicts atom. (Symmetry)

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn contradiction_detection_is_symmetric(atom in atom_name()) {
        let a_text    = atom.clone();
        let not_text  = format!("NOT {atom}");

        let a     = Statement::atomic("s1", &a_text,   "test");
        let not_a = Statement::atomic("s2", &not_text, "test");

        let checker = FormulaCoherenceChecker;

        // Extend {A} with NOT A.
        let mut t_a = Theory::new();  t_a.insert(a.clone());
        let r1 = checker.check_extension(&t_a, &not_a);

        // Extend {NOT A} with A.
        let mut t_na = Theory::new(); t_na.insert(not_a.clone());
        let r2 = checker.check_extension(&t_na, &a);

        prop_assert_eq!(
            r1.is_err(), r2.is_err(),
            "contradiction must be symmetric for atom={:?}", atom
        );
        // Both must detect contradiction.
        prop_assert!(r1.is_err(), "A contradicts NOT A");
        prop_assert!(r2.is_err(), "NOT A contradicts A");
    }
}

// ---------------------------------------------------------------------------
// P4: Atomic statements with no provenance are always ungrounded.

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn atomic_statement_without_provenance_is_ungrounded(
        id   in "[a-z]{1,8}",
        text in "[a-z ]{1,20}",
    ) {
        let stmt = Statement::atomic(&id, &text, "test");
        // Atomic constructor sets formula=None, provenance=Default (all empty).
        let result = ProvenanceGroundingChecker.check_grounding(&stmt);
        prop_assert!(!result.is_grounded,
            "atomic statement without provenance must be ungrounded (id={id:?})");
    }
}

// ---------------------------------------------------------------------------
// P5: Bridge produces unique statement ids for any compiled markdown.

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn bridge_never_produces_duplicate_ids(src in small_markdown()) {
        let Ok(artifact) = stf_sir::compiler::compile_markdown(&src, None) else { return Ok(()); };
        let theory = artifact_to_theory(&artifact);
        let ids: HashSet<&str> = theory.statements.keys().map(String::as_str).collect();
        prop_assert_eq!(ids.len(), theory.statements.len(),
            "all statement ids must be unique");
    }
}

// ---------------------------------------------------------------------------
// P6: CoherenceRetention scalar stays within [0, 1] for all valid inputs.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn coherence_retention_scalar_in_unit_interval(
        rho  in 0.0f32..=1.0,
        lex  in 0.0f32..=1.0,
        struc in 0.0f32..=1.0,
        grd  in 0.0f32..=1.0,
    ) {
        let cr = CoherenceRetention {
            rho,
            lexical_preservation: lex,
            structural_preservation: struc,
            grounding_preservation: grd,
        };
        let scalar = cr.scalar();
        prop_assert!(scalar >= 0.0, "scalar must be ≥ 0, got {scalar}");
        prop_assert!(scalar <= 1.0, "scalar must be ≤ 1, got {scalar}");
    }
}

// ---------------------------------------------------------------------------
// P7: Formula roundtrip: parse(f.to_string()) == Some(f) for non-trivial formulas.

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn formula_display_roundtrip(a in atom_name(), b in atom_name()) {
        for f in [
            Formula::atom(a.clone()),
            Formula::not(Formula::atom(a.clone())),
            Formula::implies(Formula::atom(a.clone()), Formula::atom(b.clone())),
        ] {
            let s = f.to_string();
            let parsed = Formula::parse(&s);
            prop_assert_eq!(parsed, Some(f.clone()),
                "roundtrip failed for {:?} (original: {:?})", s, f);
        }
    }
}

// ---------------------------------------------------------------------------
// P8: Adding a non-contradicting atom to a consistent theory stays consistent.

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn adding_unrelated_atom_preserves_consistency(
        atoms in prop::collection::vec(atom_name(), 1..5usize),
        new_atom in "[A-Z]{7,10}",  // longer name → unlikely to collide with short atoms
    ) {
        let checker = FormulaCoherenceChecker;

        // Build a consistent theory from short distinct atoms.
        let mut theory = Theory::new();
        for (i, a) in atoms.iter().enumerate() {
            theory.insert(Statement::atomic(format!("s{i}"), a, "test"));
        }

        // Base must be consistent (no negations yet).
        prop_assume!(checker.check_consistency(&theory).is_ok());

        // Add a long-named atom that cannot conflict.
        let candidate = Statement::atomic("extra", &new_atom, "test");
        let result = checker.check_extension(&theory, &candidate);
        prop_assert!(result.is_ok(),
            "adding unrelated atom {new_atom:?} must not create contradiction");
    }
}

// ---------------------------------------------------------------------------
// P9: steps_used is non-negative and bounded by theory size.

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn steps_used_bounded_by_theory_size(
        atoms in prop::collection::vec(atom_name(), 1..8usize),
    ) {
        use stf_sir::compiler::engine::formula_engine_with_budget;

        let mut theory = Theory::new();
        for (i, a) in atoms.iter().enumerate() {
            theory.insert(Statement::atomic(format!("s{i}"), a, "test"));
        }
        let n = theory.len();
        let candidate = Statement::atomic("c", "ZZZZZZZ", "test"); // never conflicts

        let engine = formula_engine_with_budget(usize::MAX);
        let result = engine.evaluate_statement(&theory, &candidate);

        prop_assert!(result.steps_used <= n + 1,
            "steps_used ({}) must be ≤ theory.len() + 1 = {}", result.steps_used, n + 1);
    }
}
