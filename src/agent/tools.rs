//! Agent tool definitions compatible with OpenAI function-calling and Anthropic tool_use.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// A single agent tool with its JSON-Schema parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    /// Stable tool name (snake_case, semver contract).
    pub name: String,

    /// Human-readable description shown to the language model.
    pub description: String,

    /// JSON Schema object describing the tool's parameters.
    pub parameters: serde_json::Value,
}

/// A collection of agent tools that can be passed directly to an LLM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolSchema {
    pub tools: Vec<AgentTool>,
}

/// An inbound tool-call request from an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub parameters: serde_json::Value,
}

/// The result returned to the AI agent after executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub tool_name: String,
    pub result: serde_json::Value,
    /// `None` on success; error message on failure.
    pub error: Option<String>,
}

/// Return the standard STF-SIR agent tool schema (3 tools: query, diff, retention).
///
/// The returned schema is valid for both OpenAI tools format and the Anthropic
/// `tool_use` format.
pub fn stf_sir_tools() -> AgentToolSchema {
    AgentToolSchema {
        tools: vec![
            AgentTool {
                name: "stf_sir_query".to_string(),
                description: "Query a compiled ZMD artifact by token type, gloss pattern, \
                               semantic concept, or node depth. Returns matching ZTokens \
                               with full provenance."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "artifact_path": {
                            "type": "string",
                            "description": "Path to the compiled .zmd artifact file."
                        },
                        "node_type": {
                            "type": "string",
                            "description": "Filter by syntactic node type (e.g. 'heading', 'paragraph', 'list_item')."
                        },
                        "gloss_pattern": {
                            "type": "string",
                            "description": "Substring or regex pattern to match against Σ.gloss."
                        },
                        "concept": {
                            "type": "string",
                            "description": "Filter tokens containing this concept in Σ.concepts."
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return.",
                            "default": 20
                        }
                    },
                    "required": ["artifact_path"]
                }),
            },
            AgentTool {
                name: "stf_sir_diff".to_string(),
                description: "Compute a semantic diff between two compiled ZMD artifacts. \
                               Returns added, removed, and modified ZTokens and relations."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "artifact_path_a": {
                            "type": "string",
                            "description": "Path to the baseline .zmd artifact (version A)."
                        },
                        "artifact_path_b": {
                            "type": "string",
                            "description": "Path to the revised .zmd artifact (version B)."
                        },
                        "include_unchanged": {
                            "type": "boolean",
                            "description": "If true, include unchanged tokens in the result.",
                            "default": false
                        }
                    },
                    "required": ["artifact_path_a", "artifact_path_b"]
                }),
            },
            AgentTool {
                name: "stf_sir_retention".to_string(),
                description: "Compute the retention score (ρ_v2) for a compiled ZMD artifact. \
                               Returns the full RetentionBaseline vector including semantic, \
                               structural, and logical component scores."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "artifact_path": {
                            "type": "string",
                            "description": "Path to the compiled .zmd artifact file."
                        },
                        "include_breakdown": {
                            "type": "boolean",
                            "description": "If true, include per-token retention scores.",
                            "default": false
                        }
                    },
                    "required": ["artifact_path"]
                }),
            },
        ],
    }
}
