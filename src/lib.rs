//! # STF-SIR — Semantic Compilation and Theory-Aware Reasoning Infrastructure
//!
//! STF-SIR is an auditable semantic compilation system. It transforms source
//! documents into structured `.zmd` artefacts and evaluates those artefacts for
//! logical, computational, and operational coherence.
//!
//! ## Canonical entry points
//!
//! | Task | Function / Type |
//! |---|---|
//! | Compile a Markdown file | [`compiler::compile_markdown`] / [`compiler::compile_path`] |
//! | Convert artefact to theory | [`model::artifact_to_theory`] ← **primary bridge** |
//! | Evaluate coherence | [`compiler::recommended_engine`] + [`compiler::EvaluationResult`] |
//! | Project to SIR graph | [`sir::SirGraph::from_artifact`] |
//! | Compute retention score | [`retention::RetentionBaseline`] |
//!
//! ## Architecture (ADR-SEM-001 Rule 3.4 — layer separation)
//!
//! ```text
//! Artefact layer  →  SIR layer  →  Theory layer  →  Coherence layer
//! (model::Artifact)  (sir::SirGraph)  (model::Theory)  (compiler::StfEngine)
//! ```
//!
//! Dependencies flow in one direction only: Coherence depends on Theory,
//! Theory is derived from the SIR layer via the bridge function β
//! ([`model::artifact_to_theory`]), and the SIR layer is projected from the
//! Artefact.

pub mod agent;
pub mod benchmark;
pub mod cli;
pub mod compiler;
pub mod diff;
pub mod error;
pub mod model;
pub mod plugin;
pub mod rag;
pub mod retention;
pub mod sir;

pub use compiler::{
    // Recommended engine — prefer in all new code (ADR-SEM-001 I-6)
    recommended_engine, recommended_engine_with_budget, recommended_engine_with_sir,
    RecommendedEngine, RECOMMENDED_STEP_BUDGET,
    // Shared types
    EvaluationResult, FormulaEngine, formula_engine_with_budget,
    // Grounding
    GroundingChecker, SirGroundingChecker,
};
#[allow(deprecated)]
pub use compiler::{default_engine, DefaultEngine};
pub use error::{CoherenceError, CompileError, CompileResult, ErrorKind, Severity};
pub use model::{artifact_to_theory, CoherenceVector, Formula, InsertionOutcome, SemanticDimensions, Statement, Theory, TrustLevel, TruthValue};
pub use retention::{
    CoherenceRetention, PipelineScore, RetentionBaseline, RetentionScore,
    RetentionVector, UnifiedRetentionVector,
};
