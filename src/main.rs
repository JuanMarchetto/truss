use clap::{Parser, Subcommand};
use std::fs;

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

fn validate_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let _doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    println!("✓ Valid YAML: {}", path);
    Ok(())
}

fn parse_and_display(path: &str) {
    let content = fs::read_to_string(path).expect("failed to read file");

    let doc: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("failed to parse YAML");

    println!("Parsed workflow: {}", path);

    if let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) {
        println!("Jobs: {}", jobs.len());

        for (name, job) in jobs {
            let steps = job
                .get("steps")
                .and_then(|s| s.as_sequence())
                .map(|s| s.len())
                .unwrap_or(0);

            println!("  - {:?}: {} steps", name, steps);
        }
    }
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
                parse_and_display(&path);
            } else {
                eprintln!("Error: No command or path provided");
                eprintln!("Use 'truss validate <path>' or 'truss <path>'");
                std::process::exit(1);
            }
        }
    }
}
