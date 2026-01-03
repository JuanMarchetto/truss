use clap::{Parser, Subcommand};
use std::fs;
use std::io;
use std::path::Path;
use truss_core::TrussEngine;
use rayon::prelude::*;

#[derive(Parser)]
#[command(name = "truss")]
#[command(about = "Truss - CI/CD pipeline validation tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Path to YAML file (legacy: direct path argument)
    path: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate YAML file(s)
    Validate {
        /// Path(s) to the YAML file(s) to validate
        #[arg(num_args = 1..)]
        paths: Vec<String>,
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
    if !Path::new(path).exists() {
        return Err(TrussError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", path),
        )));
    }
    fs::read_to_string(path).map_err(TrussError::Io)
}

fn validate_file(path: &str) -> Result<(), TrussError> {
    let content = read_file(path)?;
    
    let mut engine = TrussEngine::new();
    let result = engine.analyze(&content);
    
    if result.is_ok() {
        println!("✓ Valid: {}", path);
        
        // Print any warnings or info messages
        for diagnostic in &result.diagnostics {
            if diagnostic.severity != truss_core::Severity::Error {
                println!("  {}", diagnostic);
            }
        }
        Ok(())
    } else {
        // Print all diagnostics
        for diagnostic in &result.diagnostics {
            eprintln!("  {}", diagnostic);
        }
        Err(TrussError::ValidationFailed)
    }
}

fn validate_files(paths: Vec<String>) -> Result<(), TrussError> {
    if paths.is_empty() {
        return Err(TrussError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No files provided",
        )));
    }

    // Process files in parallel
    // Note: Each file gets its own engine instance to avoid mutable borrow conflicts
    let results: Vec<(String, Result<(), TrussError>)> = paths
        .par_iter()
        .map(|path| {
            let result = validate_file(path);
            (path.clone(), result)
        })
        .collect();

    // Aggregate results
    let mut has_errors = false;
    let mut success_count = 0;
    let mut error_count = 0;

    for (path, result) in results {
        match result {
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                error_count += 1;
                has_errors = true;
                eprintln!("Error validating {}: {}", path, e);
            }
        }
    }

    // Print summary if multiple files
    if paths.len() > 1 {
        println!("\nSummary: {} passed, {} failed", success_count, error_count);
    }

    if has_errors {
        Err(TrussError::ValidationFailed)
    } else {
        Ok(())
    }
}

fn analyze_file(path: &str) -> Result<(), TrussError> {
    let content = read_file(path)?;
    
    let mut engine = TrussEngine::new();
    let result = engine.analyze(&content);
    
    println!("Analysis for: {}", path);
    
    if result.diagnostics.is_empty() {
        println!("  No issues found");
    } else {
        for diagnostic in &result.diagnostics {
            println!("  {}", diagnostic);
        }
    }
    
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { paths }) => {
            if let Err(e) = validate_files(paths) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Legacy behavior: if a path is provided as a positional argument, use it
            if let Some(path) = cli.path {
                if let Err(e) = analyze_file(&path) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: No command or path provided");
                eprintln!("Use 'truss validate <path>' or 'truss <path>'");
                std::process::exit(1);
            }
        }
    }
}

