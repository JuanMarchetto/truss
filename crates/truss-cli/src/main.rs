use clap::{Parser, Subcommand};
use std::fs;
use std::io;
use std::path::Path;
use truss_core::TrussEngine;

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
    /// Validate a YAML file
    Validate {
        /// Path to the YAML file to validate
        path: String,
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
    
    let engine = TrussEngine::new();
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

fn analyze_file(path: &str) -> Result<(), TrussError> {
    let content = read_file(path)?;
    
    let engine = TrussEngine::new();
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
        Some(Commands::Validate { path }) => {
            if let Err(e) = validate_file(&path) {
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

