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

fn validate_file(path: &str, quiet: bool) -> Result<(), TrussError> {
    let content = read_file(path)?;
    
    let mut engine = TrussEngine::new();
    let result = engine.analyze(&content);
    
    if result.is_ok() {
        if !quiet {
            println!("✓ Valid: {}", path);
            
            for diagnostic in &result.diagnostics {
                if diagnostic.severity != truss_core::Severity::Error {
                    println!("  {}", diagnostic);
                }
            }
        }
        Ok(())
    } else {
        if !quiet {
            for diagnostic in &result.diagnostics {
                eprintln!("  {}", diagnostic);
            }
        }
        Err(TrussError::ValidationFailed)
    }
}

fn validate_files(paths: Vec<String>, quiet: bool) -> Result<(), TrussError> {
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
            let result = validate_file(path, quiet);
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
                if !quiet {
                    eprintln!("Error validating {}: {}", path, e);
                }
            }
        }
    }

    // Print summary if multiple files and not quiet
    if !quiet && paths.len() > 1 {
        println!("\nSummary: {} passed, {} failed", success_count, error_count);
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
        Commands::Validate { paths, quiet } => {
            if let Err(e) = validate_files(paths, quiet) {
                if !quiet {
                    eprintln!("Error: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
}

