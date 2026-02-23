use clap::{Parser, Subcommand};
use std::fs;
use std::io;
use std::time::Instant;
use truss_core::TrussEngine;
use rayon::prelude::*;
use serde_json;

#[derive(Parser)]
#[command(name = "truss")]
#[command(about = "Truss - CI/CD pipeline validation tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate YAML file(s)
    Validate {
        /// Path(s) to the YAML file(s) to validate
        #[arg(num_args = 1..)]
        paths: Vec<String>,
        
        /// Suppress output (only exit code indicates success/failure)
        #[arg(short, long)]
        quiet: bool,
        
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug)]
enum TrussError {
    Io(io::Error),
    ValidationFailed,
}

impl std::fmt::Display for TrussError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrussError::Io(e) => write!(f, "I/O error: {}", e),
            TrussError::ValidationFailed => write!(f, "Validation failed"),
        }
    }
}

impl std::error::Error for TrussError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TrussError::Io(e) => Some(e),
            TrussError::ValidationFailed => None,
        }
    }
}

impl From<io::Error> for TrussError {
    fn from(err: io::Error) -> Self {
        TrussError::Io(err)
    }
}

fn read_file(path: &str) -> Result<String, TrussError> {
    fs::read_to_string(path).map_err(TrussError::Io)
}

#[derive(serde::Serialize)]
struct FileResult {
    file: String,
    valid: bool,
    diagnostics: Vec<truss_core::Diagnostic>,
    duration_ms: f64,
    metadata: FileMetadata,
}

#[derive(serde::Serialize)]
struct FileMetadata {
    file_size: u64,
    lines: usize,
}

fn validate_file(path: &str, quiet: bool, json: bool) -> Result<FileResult, TrussError> {
    let content = read_file(path)?;
    let file_size = content.len() as u64;
    let lines = content.lines().count();
    
    let start = Instant::now();
    let mut engine = TrussEngine::new();
    let result = engine.analyze(&content);
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    
    let valid = result.is_ok();
    
    if json {
        let file_result = FileResult {
            file: path.to_string(),
            valid,
            diagnostics: result.diagnostics,
            duration_ms,
            metadata: FileMetadata {
                file_size,
                lines,
            },
        };
        return Ok(file_result);
    }
    
    if valid {
        if !quiet {
            println!("✓ Valid: {}", path);
            
            for diagnostic in &result.diagnostics {
                if diagnostic.severity != truss_core::Severity::Error {
                    println!("  {}", diagnostic);
                }
            }
        }
    } else {
        if !quiet {
            for diagnostic in &result.diagnostics {
                eprintln!("  {}", diagnostic);
            }
        }
    }
    
    Ok(FileResult {
        file: path.to_string(),
        valid,
        diagnostics: result.diagnostics,
        duration_ms,
        metadata: FileMetadata {
            file_size,
            lines,
        },
    })
}

fn validate_files(paths: Vec<String>, quiet: bool, json: bool) -> Result<(), TrussError> {
    if paths.is_empty() {
        return Err(TrussError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No files provided",
        )));
    }

    // Process files in parallel
    // Note: Each file gets its own engine instance to avoid mutable borrow conflicts
    let results: Vec<(String, Result<FileResult, TrussError>)> = paths
        .par_iter()
        .map(|path| {
            let result = validate_file(path, quiet, json);
            (path.clone(), result)
        })
        .collect();

    // Aggregate results
    let mut has_errors = false;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut file_results = Vec::new();

    for (path, result) in results {
        match result {
            Ok(file_result) => {
                if !file_result.valid {
                    error_count += 1;
                    has_errors = true;
                } else {
                    success_count += 1;
                }
                file_results.push(file_result);
            }
            Err(e) => {
                error_count += 1;
                has_errors = true;
                if !quiet && !json {
                    eprintln!("Error validating {}: {}", path, e);
                }
            }
        }
    }

    if json {
        let json_output = serde_json::to_string_pretty(&file_results)
            .map_err(|e| TrussError::Io(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize JSON: {}", e),
            )))?;
        println!("{}", json_output);
    } else {
        // Print summary if multiple files and not quiet
        if !quiet && paths.len() > 1 {
            println!("\nSummary: {} passed, {} failed", success_count, error_count);
        }
    }

    if has_errors {
        Err(TrussError::ValidationFailed)
    } else {
        Ok(())
    }
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { paths, quiet, json } => {
            if let Err(e) = validate_files(paths, quiet, json) {
                if !quiet && !json {
                    eprintln!("Error: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
}

