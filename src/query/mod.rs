//! EPIC-203: Semantic Query Engine — public API.
//!
//! Re-exports the query engine from [`crate::sir::query`].
//!
//! See [`crate::sir::query`] for the full implementation.

pub mod ast;
pub mod executor;
pub mod result;
pub mod traversal;

pub use ast::{Dimension, Query};
pub use executor::QueryExecutor;
pub use result::QueryResult;
