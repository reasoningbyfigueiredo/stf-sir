//! EPIC-203: Semantic Query Engine — submodule of `sir`.
//!
//! Provides a typed, deterministic query engine over [`SirGraph`].
//!
//! # Quick start
//!
//! ```rust,no_run
//! use stf_sir::compiler::compile_markdown;
//! use stf_sir::sir::query::{Query, QueryExecutor};
//!
//! let artifact = compile_markdown("# Hello\n\nWorld.\n", None).unwrap();
//! let graph = artifact.as_sir_graph();
//! let executor = QueryExecutor::new(&graph, &artifact);
//!
//! let result = executor.execute(&Query::ByType { node_type: "heading".to_string() });
//! assert!(!result.token_ids.is_empty());
//! ```

pub mod ast;
pub mod executor;
pub mod result;
pub mod traversal;

pub use ast::{Dimension, Query};
pub use executor::QueryExecutor;
pub use result::QueryResult;
