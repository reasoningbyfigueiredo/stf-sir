pub mod report;
pub mod semantic;
pub mod structural;

pub use report::{diff_artifacts, DiffReport, DiffSummary};
pub use semantic::{semantic_diff, ConceptChange, GlossChange, SemanticDiff};
pub use structural::{structural_diff, NodeTypeChange, StructuralDiff};
