use clap::{Parser, Subcommand, ValueEnum};
use glob::glob;
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;
use truss_core::TrussEngine;

/// Exit code: one or more files had validation errors.
const EXIT_VALIDATION_FAILED: i32 = 1;
/// Exit code: usage error (bad arguments, no files provided).
const EXIT_USAGE: i32 = 2;
/// Exit code: I/O error (file not found, permission denied, etc.).
const EXIT_IO: i32 = 3;

#[derive(Parser)]
#[command(name = "truss")]
#[command(version)]
#[command(about = "Truss - CI/CD pipeline validation tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate YAML file(s)
    Validate {
        /// Path(s), directories, or glob patterns to validate. Use `-` for stdin.
        #[arg(num_args = 1..)]
        paths: Vec<String>,

        /// Suppress output (only exit code indicates success/failure)
        #[arg(short, long)]
        quiet: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Minimum severity level to display and fail on
        #[arg(long, value_enum)]
        severity: Option<SeverityFilter>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SeverityFilter {
    /// Show only errors
    Error,
    /// Show errors and warnings
    Warning,
    /// Show everything (default)
    Info,
}

impl SeverityFilter {
    fn includes(self, severity: truss_core::Severity) -> bool {
        match self {
            SeverityFilter::Error => severity == truss_core::Severity::Error,
            SeverityFilter::Warning => {
                severity == truss_core::Severity::Error || severity == truss_core::Severity::Warning
            }
            SeverityFilter::Info => true,
        }
    }
}

#[derive(Debug)]
enum TrussError {
    Io(io::Error),
    Usage(String),
    ValidationFailed,
}

impl std::fmt::Display for TrussError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrussError::Io(e) => write!(f, "I/O error: {}", e),
            TrussError::Usage(msg) => write!(f, "{}", msg),
            TrussError::ValidationFailed => write!(f, "Validation failed"),
        }
    }
}

impl std::error::Error for TrussError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TrussError::Io(e) => Some(e),
            TrussError::Usage(_) => None,
            TrussError::ValidationFailed => None,
        }
    }
}

impl From<io::Error> for TrussError {
    fn from(err: io::Error) -> Self {
        TrussError::Io(err)
    }
}

impl TrussError {
    fn exit_code(&self) -> i32 {
        match self {
            TrussError::Io(_) => EXIT_IO,
            TrussError::Usage(_) => EXIT_USAGE,
            TrussError::ValidationFailed => EXIT_VALIDATION_FAILED,
        }
    }
}

fn read_source(path: &str) -> Result<String, TrussError> {
    if path == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(TrussError::Io)?;
        Ok(buf)
    } else {
        fs::read_to_string(path).map_err(TrussError::Io)
    }
}

/// Expand a user-provided path into concrete file paths.
///
/// - `-` is returned as-is (stdin marker).
/// - If the path is a directory, recursively find all `*.yml` and `*.yaml` files.
/// - If the path contains glob characters, expand via `glob::glob()`.
/// - Otherwise, return the path as-is.
fn expand_paths(raw_paths: &[String]) -> Result<Vec<String>, TrussError> {
    let mut expanded = Vec::new();

    for raw in raw_paths {
        if raw == "-" {
            expanded.push("-".to_string());
            continue;
        }

        let path = Path::new(raw);

        if path.is_dir() {
            for ext in &["/**/*.yml", "/**/*.yaml"] {
                let pattern = format!("{}{}", raw.trim_end_matches('/'), ext);
                match glob(&pattern) {
                    Ok(entries) => {
                        for entry in entries.flatten() {
                            expanded.push(entry.display().to_string());
                        }
                    }
                    Err(e) => {
                        return Err(TrussError::Io(io::Error::other(format!(
                            "Invalid glob pattern '{}': {}",
                            pattern, e
                        ))));
                    }
                }
            }
        } else if raw.contains('*') || raw.contains('?') || raw.contains('[') {
            match glob(raw) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        expanded.push(entry.display().to_string());
                    }
                }
                Err(e) => {
                    return Err(TrussError::Io(io::Error::other(format!(
                        "Invalid glob pattern '{}': {}",
                        raw, e
                    ))));
                }
            }
        } else {
            expanded.push(raw.clone());
        }
    }

    Ok(expanded)
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

fn validate_source(
    engine: &mut TrussEngine,
    label: &str,
    content: &str,
    quiet: bool,
    json: bool,
    severity_filter: SeverityFilter,
) -> Result<FileResult, TrussError> {
    let file_size = content.len() as u64;
    let lines = content.lines().count();

    let start = Instant::now();
    let result = engine.analyze(content);
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Filter diagnostics by severity
    let filtered: Vec<truss_core::Diagnostic> = result
        .diagnostics
        .into_iter()
        .filter(|d| severity_filter.includes(d.severity))
        .collect();

    let valid = !filtered
        .iter()
        .any(|d| d.severity == truss_core::Severity::Error);

    if json {
        return Ok(FileResult {
            file: label.to_string(),
            valid,
            diagnostics: filtered,
            duration_ms,
            metadata: FileMetadata { file_size, lines },
        });
    }

    if valid {
        if !quiet {
            println!("✓ Valid: {}", label);
            for diagnostic in &filtered {
                println!("  {}", diagnostic);
            }
        }
    } else if !quiet {
        for diagnostic in &filtered {
            eprintln!("  {}", diagnostic);
        }
    }

    Ok(FileResult {
        file: label.to_string(),
        valid,
        diagnostics: filtered,
        duration_ms,
        metadata: FileMetadata { file_size, lines },
    })
}

fn validate_file(
    engine: &mut TrussEngine,
    path: &str,
    quiet: bool,
    json: bool,
    severity_filter: SeverityFilter,
) -> Result<FileResult, TrussError> {
    let content = read_source(path)?;
    let label = if path == "-" { "<stdin>" } else { path };
    validate_source(engine, label, &content, quiet, json, severity_filter)
}

fn validate_files(
    paths: Vec<String>,
    quiet: bool,
    json: bool,
    severity_filter: SeverityFilter,
) -> Result<(), TrussError> {
    let expanded = expand_paths(&paths)?;

    if expanded.is_empty() {
        return Err(TrussError::Usage(
            "No files found. Run 'truss validate --help' for usage.".to_string(),
        ));
    }

    // Separate stdin from file paths (stdin can't be parallelized)
    let (stdin_paths, file_paths): (Vec<_>, Vec<_>) =
        expanded.iter().partition(|p| p.as_str() == "-");

    let mut all_results: Vec<(String, Result<FileResult, TrussError>)> = Vec::new();

    // Process stdin first (sequential, reuse one engine)
    let mut engine = TrussEngine::new();
    for path in &stdin_paths {
        let result = validate_file(&mut engine, path, quiet, json, severity_filter);
        all_results.push((path.to_string(), result));
    }

    // For a single file, sequential is faster (avoids rayon thread pool overhead).
    // For multiple files, parallel processing pays off.
    let file_results: Vec<(String, Result<FileResult, TrussError>)> = if file_paths.len() <= 1 {
        file_paths
            .iter()
            .map(|path| {
                let result = validate_file(&mut engine, path, quiet, json, severity_filter);
                (path.to_string(), result)
            })
            .collect()
    } else {
        file_paths
            .par_iter()
            .map(|path| {
                let mut engine = TrussEngine::new();
                let result = validate_file(&mut engine, path, quiet, json, severity_filter);
                (path.to_string(), result)
            })
            .collect()
    };

    all_results.extend(file_results);

    // Aggregate results
    let mut has_errors = false;
    let mut has_io_error = false;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut file_results = Vec::new();

    for (path, result) in &all_results {
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
                if matches!(e, TrussError::Io(_)) {
                    has_io_error = true;
                }
                if !quiet && !json {
                    eprintln!("Error validating {}: {}", path, e);
                }
            }
        }
    }

    if json {
        let json_output = serde_json::to_string_pretty(&file_results).map_err(|e| {
            TrussError::Io(io::Error::other(format!("Failed to serialize JSON: {}", e)))
        })?;
        println!("{}", json_output);
    } else if !quiet && expanded.len() > 1 {
        println!(
            "\nSummary: {} passed, {} failed",
            success_count, error_count
        );
    }

    if has_io_error {
        Err(TrussError::Io(io::Error::other("One or more files failed")))
    } else if has_errors {
        Err(TrussError::ValidationFailed)
    } else {
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            paths,
            quiet,
            json,
            severity,
        } => {
            let severity_filter = severity.unwrap_or(SeverityFilter::Info);

            if paths.is_empty() {
                if !quiet && !json {
                    eprintln!("Error: No files provided. Run 'truss validate --help' for usage.");
                }
                std::process::exit(EXIT_USAGE);
            }

            if let Err(e) = validate_files(paths, quiet, json, severity_filter) {
                if !quiet && !json {
                    eprintln!("Error: {}", e);
                }
                std::process::exit(e.exit_code());
            }
        }
    }
}
