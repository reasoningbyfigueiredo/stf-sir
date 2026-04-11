use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use crate::compiler;
use crate::compiler::validator;
use crate::error::CompileError;
use crate::model::DiagnosticSeverity;

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
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => match compiler::compile_to_file(&input, &output) {
            Ok(_) => Ok(()),
            Err(err) => Err(format_compile_error(err)),
        },
        Commands::Validate { artifact, source } => run_validate(&artifact, source.as_deref()),
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
