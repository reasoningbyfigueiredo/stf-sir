use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::{CompileError, CompileResult};
use crate::model::{Artifact, CompilerInfo, Diagnostic, DocumentInfo, SourceInfo, ZToken};

pub mod coherence;
pub mod diagnostics;
pub mod domain;
pub mod engine;
pub mod enricher;
pub mod frontend;
pub mod grounding;
pub mod inference;
pub mod lang;
pub mod lexical;
pub mod logical;
pub mod profile;
pub mod schema;
pub mod semantic;
pub mod serializer;
pub mod syntactic;
pub mod validator;

pub use coherence::{FormulaCoherenceChecker, LogicalCoherenceChecker, SimpleLogicChecker};
pub use engine::{
    // Recommended (formula-AST) engine — prefer these in new code
    recommended_engine, recommended_engine_with_budget, recommended_engine_with_sir,
    RecommendedEngine, RECOMMENDED_STEP_BUDGET,
    // Formula engine (explicit alias, same backends as RecommendedEngine)
    formula_engine_with_budget, FormulaEngine,
    // Shared result type
    EvaluationResult,
};
// Legacy — deprecated since 1.1.0, kept for backwards compatibility
#[allow(deprecated)]
pub use engine::{default_engine, DefaultEngine};
pub use grounding::{GroundingChecker, GroundingResult, ProvenanceGroundingChecker, SirGroundingChecker};
pub use inference::{FormulaInferenceEngine, InferenceEngine, RuleBasedInferenceEngine};

const FORMAT: &str = "stf-sir.zmd";
const VERSION: u32 = 1;
const COMPILER_NAME: &str = "stf-sir-ref";

pub fn compile_to_file(input: &Path, output: &Path) -> CompileResult<Artifact> {
    let artifact = compile_path(input)?;
    let yaml = serializer::to_yaml_string(&artifact)?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| CompileError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }

    fs::write(output, yaml).map_err(|source| CompileError::Write {
        path: output.to_path_buf(),
        source,
    })?;

    Ok(artifact)
}

pub fn compile_path(input: &Path) -> CompileResult<Artifact> {
    let lexical_document = lexical::load_from_path(input)?;
    compile_lexical_document(lexical_document)
}

pub fn compile_markdown(source: &str, source_path: Option<&Path>) -> CompileResult<Artifact> {
    let lexical_document = lexical::load_from_string(
        source.to_owned(),
        source_path.map(|path| path.to_string_lossy().into_owned()),
    )?;
    compile_lexical_document(lexical_document)
}

fn compile_lexical_document(lexical_document: lexical::LexicalDocument) -> CompileResult<Artifact> {
    let compiler_info = CompilerInfo {
        name: COMPILER_NAME.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        config_hash: lexical::sha256_prefixed(config_string().as_bytes()),
        profile: Some("stf-sir-spec-v1-mvp".to_string()),
    };

    let source_info = SourceInfo {
        path: lexical_document.path.clone(),
        media_type: "text/markdown".to_string(),
        encoding: "utf-8".to_string(),
        length_bytes: lexical_document.length_bytes,
        sha256: lexical_document.sha256.clone(),
    };

    let syntactic_output = syntactic::parse_markdown(&lexical_document)
        .map_err(|err| CompileError::Syntactic(err.to_string()))?;
    let mut ztokens = syntactic_output.ztokens;
    let mut diagnostics = syntactic_output.diagnostics;

    semantic::apply(&mut ztokens);
    let relations = logical::build_relations(&mut ztokens);

    let root_token_ids = ztokens
        .iter()
        .filter(|token| token.syntactic.parent_id.is_none())
        .map(|token| token.id.clone())
        .collect::<Vec<_>>();

    let document_info = DocumentInfo {
        language: "und".to_string(),
        token_count: ztokens.len(),
        relation_count: relations.len(),
        root_token_ids,
    };

    diagnostics.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then(left.stage.cmp(&right.stage))
            .then(left.message.cmp(&right.message))
    });

    Ok(Artifact {
        format: FORMAT.to_string(),
        version: VERSION,
        source: source_info,
        compiler: compiler_info,
        document: document_info,
        ztokens,
        relations,
        diagnostics,
        extensions: BTreeMap::new(),
    })
}

fn config_string() -> String {
    [
        "format=stf-sir.zmd",
        "version=1",
        "media_type=text/markdown",
        "parser=pulldown-cmark",
        "parser_options=tables,footnotes,strikethrough",
        "node_types=heading,paragraph,blockquote,list,list_item,code_block,table,footnote_definition",
        "relations=contains,precedes",
        "relation_categories=structural,logical,semantic-link",
        "relation_stages=lexical,syntactic,semantic,logical",
        "diagnostic_stages=lexical,syntactic,semantic,logical,validation",
        "semantic_fallback=normalized_text_or_empty",
        "normalization=nfkc_trim_collapse_whitespace",
        "serialization=serde_yaml_ng_stable_struct_order",
        "sibling_ordering=parent_token_index",
        "encoding=utf-8",
    ]
    .join(";")
}

#[allow(dead_code)]
fn _assert_send_sync()
where
    Artifact: Send + Sync,
    CompilerInfo: Send + Sync,
    Diagnostic: Send + Sync,
    DocumentInfo: Send + Sync,
    SourceInfo: Send + Sync,
    ZToken: Send + Sync,
{
}
