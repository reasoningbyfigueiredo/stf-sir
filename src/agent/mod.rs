//! Agent tool interface for STF-SIR (EPIC-206).
//!
//! Exposes STF-SIR capabilities as AI agent tools compatible with the
//! OpenAI function-calling and Anthropic tool_use JSON formats.
//!
//! The agent tool schema (`AgentToolSchema`) is always available (no feature gate)
//! because it is a pure data / schema type used for documentation and integration.

pub mod tools;

pub use tools::{stf_sir_tools, AgentTool, AgentToolSchema, ToolCallRequest, ToolCallResponse};
