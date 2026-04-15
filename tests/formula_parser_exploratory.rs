//! Exploratory tests for `Formula::parse` — parser behaviour, edge cases,
//! ambiguous inputs, normalization stability, and adversarial strings.
//!
//! Design goals:
//!   1. Catch silent semantic loss when the parser misfires.
//!   2. Pin the exact output for all documented surface forms.
//!   3. Prove determinism: same input → same output, always.
//!   4. Prove no-panic: no input produces a runtime failure.

use stf_sir::model::Formula;

// ---------------------------------------------------------------------------
// Helper: assert parse equals an expected variant

fn is_atom(f: &Formula) -> bool {
    matches!(f, Formula::Atom(_))
}
fn is_not(f: &Formula) -> bool {
    matches!(f, Formula::Not(_))
}
fn is_implies(f: &Formula) -> bool {
    matches!(f, Formula::Implies(_, _))
}
fn atom_text(f: &Formula) -> &str {
    match f { Formula::Atom(s) => s.as_str(), _ => panic!("not an atom: {f:?}") }
}

// ---------------------------------------------------------------------------
// 1. Canonical forms

#[test]
fn parse_plain_atom() {
    let f = Formula::parse("A").unwrap();
    assert!(is_atom(&f));
    assert_eq!(atom_text(&f), "A");
}

#[test]
fn parse_atom_is_uppercased() {
    // Parser normalises to uppercase; "a" and "A" must produce the same atom.
    let lower = Formula::parse("hello").unwrap();
    let upper = Formula::parse("HELLO").unwrap();
    assert_eq!(lower, upper, "case must be normalised");
    assert_eq!(atom_text(&lower), "HELLO");
}

#[test]
fn parse_negation() {
    let f = Formula::parse("NOT A").unwrap();
    assert!(is_not(&f), "must be Not, got {f:?}");
    let Formula::Not(inner) = &f else { unreachable!() };
    assert_eq!(atom_text(inner), "A");
}

#[test]
fn parse_implication() {
    let f = Formula::parse("A -> B").unwrap();
    assert!(is_implies(&f), "must be Implies, got {f:?}");
    let Formula::Implies(p, q) = &f else { unreachable!() };
    assert_eq!(atom_text(p), "A");
    assert_eq!(atom_text(q), "B");
}

#[test]
fn parse_double_negation_is_not_collapsed() {
    // parse() does NOT apply DNE — only negate() does.
    // "NOT NOT A" → Not(Not(Atom("A"))), not Atom("A").
    let f = Formula::parse("NOT NOT A").unwrap();
    let Formula::Not(inner) = &f else { panic!("outer must be Not, got {f:?}") };
    assert!(is_not(inner), "inner must also be Not, got {inner:?}");
}

#[test]
fn parse_lowercased_connectives() {
    // "not a" and "a -> b" must be recognised regardless of case.
    let neg = Formula::parse("not a").unwrap();
    assert!(is_not(&neg));
    let imp = Formula::parse("a -> b").unwrap();
    assert!(is_implies(&imp));
}

#[test]
fn parse_mixed_case_connectives() {
    let f = Formula::parse("Not X -> Y").unwrap();
    // The parser checks NOT prefix before "->".
    // "Not X -> Y" uppercases to "NOT X -> Y" → strip "NOT " → parse("X -> Y")
    // Result: Not(Implies(Atom("X"), Atom("Y")))
    assert!(is_not(&f));
    let Formula::Not(inner) = &f else { unreachable!() };
    assert!(is_implies(inner));
}

// ---------------------------------------------------------------------------
// 2. Whitespace robustness

#[test]
fn extra_whitespace_around_arrow_produces_same_formula() {
    // All these must parse identically to "A -> B".
    let canonical = Formula::parse("A -> B").unwrap();
    for variant in &["A  ->  B", "A   ->   B", "  A -> B  ", "  A  ->  B  "] {
        let f = Formula::parse(variant)
            .unwrap_or_else(|| panic!("must parse: {variant:?}"));
        assert_eq!(f, canonical, "whitespace variant {variant:?} must equal canonical");
    }
}

#[test]
fn leading_trailing_whitespace_around_atom() {
    let f = Formula::parse("   A   ").unwrap();
    assert_eq!(f, Formula::atom("A"));
}

#[test]
fn whitespace_only_returns_none() {
    assert_eq!(Formula::parse("   "), None);
    assert_eq!(Formula::parse("\t\n"), None);
}

// ---------------------------------------------------------------------------
// 3. Empty and minimal inputs

#[test]
fn empty_string_returns_none() {
    assert_eq!(Formula::parse(""), None);
}

#[test]
fn single_char_is_atom() {
    let f = Formula::parse("X").unwrap();
    assert_eq!(f, Formula::atom("X"));
}

// ---------------------------------------------------------------------------
// 4. Malformed / adversarial connective placement

#[test]
fn arrow_without_lhs_becomes_atom() {
    // "-> B" has no space before "->" so " -> " is not found → Atom("-> B").
    let f = Formula::parse("-> B").unwrap();
    assert!(is_atom(&f), "headless arrow must degrade to Atom, got {f:?}");
}

#[test]
fn arrow_without_rhs_becomes_atom() {
    // "A ->" has no trailing space after ">", so " -> " is not a match → Atom.
    let f = Formula::parse("A ->").unwrap();
    assert!(is_atom(&f), "tailless arrow must degrade to Atom, got {f:?}");
}

#[test]
fn not_without_body_is_an_atom() {
    // "NOT" alone: no "NOT " prefix match (needs trailing space) → Atom("NOT").
    let f = Formula::parse("NOT").unwrap();
    assert!(is_atom(&f));
    assert_eq!(atom_text(&f), "NOT");
}

#[test]
fn arrow_without_spaces_is_atom() {
    // "A->B" (no surrounding spaces) is not split → Atom.
    let f = Formula::parse("A->B").unwrap();
    assert!(is_atom(&f));
}

#[test]
fn right_associative_implication() {
    // "A -> B -> C" splits at first " -> ": lhs="A", rhs="B -> C".
    // So → Implies(Atom("A"), Implies(Atom("B"), Atom("C"))).
    let f = Formula::parse("A -> B -> C").unwrap();
    let Formula::Implies(p, q) = &f else { panic!("must be Implies, got {f:?}") };
    assert_eq!(atom_text(p), "A");
    assert!(is_implies(q), "RHS of A->B->C must itself be Implies");
}

// ---------------------------------------------------------------------------
// 5. Unicode identifiers

#[test]
fn unicode_atom_is_parseable() {
    // Non-ASCII identifiers must not panic; they become Atom(uppercase).
    let f = Formula::parse("átomo");
    assert!(f.is_some(), "unicode atom must parse without panic");
    let inner = f.unwrap();
    assert!(is_atom(&inner), "unicode input without connectives must be Atom");
}

#[test]
fn unicode_negation() {
    // "NOT ψ" → Not(Atom("Ψ")) (unicode uppercase of ψ).
    let f = Formula::parse("NOT ψ").unwrap();
    assert!(is_not(&f));
}

// ---------------------------------------------------------------------------
// 6. Normalization stability (roundtrip)

#[test]
fn display_roundtrip_atom() {
    let f = Formula::atom("ALPHA");
    assert_eq!(Formula::parse(&f.to_string()), Some(f));
}

#[test]
fn display_roundtrip_not() {
    let f = Formula::not(Formula::atom("P"));
    assert_eq!(Formula::parse(&f.to_string()), Some(f));
}

#[test]
fn display_roundtrip_implies() {
    let f = Formula::implies(Formula::atom("P"), Formula::atom("Q"));
    assert_eq!(Formula::parse(&f.to_string()), Some(f));
}

#[test]
fn display_roundtrip_nested_implies() {
    // Implies(Not(P), ...) displays as "NOT P -> Q -> R".
    // The parser sees the NOT prefix first and produces Not(Implies(P, Q->R)).
    // This is a known parser precedence limitation: NOT binds tighter than ->.
    // Round-trip works for Implies(Atom, Implies(Atom, Atom)):
    let f = Formula::implies(
        Formula::atom("P"),
        Formula::implies(Formula::atom("Q"), Formula::atom("R")),
    );
    assert_eq!(Formula::parse(&f.to_string()), Some(f));

    // Verify the known precedence behaviour explicitly:
    let not_p_implies = Formula::implies(
        Formula::not(Formula::atom("P")),
        Formula::atom("Q"),
    );
    let s = not_p_implies.to_string(); // "NOT P -> Q"
    // Parser: "NOT P -> Q" → NOT prefix → Not(parse("P -> Q")) = Not(Implies(P, Q))
    let parsed = Formula::parse(&s).unwrap();
    assert!(matches!(parsed, Formula::Not(_)),
        "parser gives NOT higher precedence than ->; got {:?}", parsed);
}

// ---------------------------------------------------------------------------
// 7. Determinism

#[test]
fn parse_is_deterministic() {
    for input in &[
        "A",
        "NOT A",
        "A -> B",
        "NOT NOT A",
        "",
        "-> B",
        "A ->",
        "  A  ->  B  ",
    ] {
        let r1 = Formula::parse(input);
        let r2 = Formula::parse(input);
        assert_eq!(r1, r2, "parse must be deterministic for input {input:?}");
    }
}

// ---------------------------------------------------------------------------
// 8. Structural contradiction checks

#[test]
fn atom_contradicts_its_negation() {
    let a = Formula::atom("A");
    let not_a = Formula::not(Formula::atom("A"));
    assert!(a.contradicts(&not_a));
    assert!(not_a.contradicts(&a));
}

#[test]
fn atom_does_not_contradict_different_atom() {
    let a = Formula::atom("A");
    let b = Formula::atom("B");
    assert!(!a.contradicts(&b));
}

#[test]
fn implication_does_not_contradict_atom() {
    let a = Formula::atom("A");
    let a_b = Formula::implies(Formula::atom("A"), Formula::atom("B"));
    assert!(!a.contradicts(&a_b));
    assert!(!a_b.contradicts(&a));
}

#[test]
fn negation_of_implication_contradicts_implication() {
    let imp = Formula::implies(Formula::atom("A"), Formula::atom("B"));
    let not_imp = Formula::not(imp.clone());
    assert!(imp.contradicts(&not_imp));
    assert!(not_imp.contradicts(&imp));
}

// ---------------------------------------------------------------------------
// 9. No-panic sweep on diverse inputs

#[test]
fn no_panic_on_pathological_inputs() {
    let inputs = [
        "A -> B -> C -> D",
        "NOT NOT NOT A",
        "-> -> ->",
        "NOT NOT NOT NOT",
        "   ->   ",
        "   NOT   ->   NOT   ",
        "A -> NOT B",
        "NOT A -> B",
        "NOT (A -> B)",   // parens not supported; becomes complex Atom
        "∀x.P(x)",        // first-order syntax, not supported; becomes Atom
        "⊥",
        "TRUE",
        "FALSE",
    ];
    for s in &inputs {
        // Must not panic; result can be None or Some.
        let _ = Formula::parse(s);
    }
}
