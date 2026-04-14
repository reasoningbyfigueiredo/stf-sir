use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use crate::compiler;
use crate::compiler::validator;
use crate::error::CompileError;
use crate::model::{artifact_to_theory, DiagnosticSeverity};

#[derive(Debug, Parser)]
#[command(name = "stf-sir")]
#[command(about = "Deterministic STF-SIR reference compiler")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Compile a Markdown source into a deterministic `.zmd` artifact.
    Compile {
        input: PathBuf,
        #[arg(short = 'o', long = "output")]
        output: PathBuf,
    },
    /// Validate an existing `.zmd` artifact against the STF-SIR v1 spec §9.
    Validate {
        /// Path to the `.zmd` artifact to validate.
        artifact: PathBuf,
        /// Optional path to the original source bytes for rule 16
        /// (L.source_text must equal the exact source slice).
        #[arg(long = "source")]
        source: Option<PathBuf>,
    },
    /// Audit the coherence of a compiled `.zmd` artifact.
    ///
    /// Evaluates the coherence triple (C_l, C_c, C_o), detects hallucinations
    /// (locally coherent but ungrounded tokens), contradictions, and
    /// operationally sterile statements.
    ///
    /// Examples:
    ///   stf-sir audit-coherence examples/sample.zmd
    ///   stf-sir audit-coherence examples/sample.zmd --json
    AuditCoherence {
        /// Path to the `.zmd` artifact to audit.
        artifact: PathBuf,
        /// Emit machine-readable JSON output.
        #[arg(long = "json")]
        json: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => match compiler::compile_to_file(&input, &output) {
            Ok(_) => Ok(()),
            Err(err) => Err(format_compile_error(err)),
        },
        Commands::Validate { artifact, source } => run_validate(&artifact, source.as_deref()),
        Commands::AuditCoherence { artifact, json } => run_audit_coherence(&artifact, json),
    }
}

fn run_validate(artifact_path: &Path, source_path: Option<&Path>) -> Result<()> {
    let yaml = fs::read_to_string(artifact_path)
        .with_context(|| format!("failed to read artifact {}", artifact_path.display()))?;

    let source_bytes = match source_path {
        Some(path) => Some(
            fs::read(path).with_context(|| format!("failed to read source {}", path.display()))?,
        ),
        None => None,
    };

    let errors = validator::validate_yaml_str(&yaml, source_bytes.as_deref());

    if errors.is_empty() {
        println!("VALID: {} conforms to STF-SIR v1", artifact_path.display());
        Ok(())
    } else {
        eprintln!("INVALID: {}", artifact_path.display());
        for error in &errors {
            eprintln!("  - {error}");
        }
        Err(anyhow!(
            "artifact failed validation with {} issue(s)",
            errors.len()
        ))
    }
}

fn run_audit_coherence(artifact_path: &Path, json: bool) -> Result<()> {
    let yaml = fs::read_to_string(artifact_path)
        .with_context(|| format!("failed to read artifact {}", artifact_path.display()))?;

    let artifact: crate::model::Artifact = serde_yaml_ng::from_str(&yaml)
        .with_context(|| format!("failed to parse artifact {}", artifact_path.display()))?;

    let theory = artifact_to_theory(&artifact);

    let engine = compiler::default_engine();
    let result = engine.audit_theory(&theory);

    if json {
        println!("{}", serde_json::to_string_pretty(&result.to_json_value())?);
    } else {
        println!(
            "coherence: {}",
            result.coherence
        );
        println!("grounded:          {}", result.grounded);
        println!("useful_information:{}", result.useful_information);
        println!("derived_count:     {}", result.derived_count);

        if result.errors.is_empty() {
            println!("errors: none");
        } else {
            println!("errors ({}):", result.errors.len());
            for e in &result.errors {
                println!("  [{}/{}] {}", e.kind, e.severity, e.message);
            }
        }

        if result.errors.iter().any(|e| {
            matches!(
                e.severity,
                crate::error::Severity::High | crate::error::Severity::Critical
            )
        }) {
            return Err(anyhow!(
                "coherence audit found {} issue(s)",
                result.errors.len()
            ));
        }
    }

    Ok(())
}

fn format_compile_error(err: CompileError) -> anyhow::Error {
    if let CompileError::Fatal { diagnostics } = &err {
        for diag in diagnostics {
            let severity = match diag.severity {
                DiagnosticSeverity::Info => "info",
                DiagnosticSeverity::Warning => "warning",
                DiagnosticSeverity::Error => "error",
            };
            eprintln!("{}: [{}] {}", severity, diag.code, diag.message);
        }
    }
    anyhow!(err)
}
