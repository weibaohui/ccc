use clap::Parser;
use std::fs;
use std::path::PathBuf;

const CLAUDE_DIR: &str = ".claude";
const SETTINGS_BASE: &str = "settings.json";

#[derive(Parser, Debug)]
#[command(name = "ccc")]
#[command(about = "Claude settings switcher", long_about = None)]
enum Commands {
    /// List all available settings profiles
    List,
    /// Apply a settings profile (backup current + replace)
    Apply { suffix: String },
}

fn claude_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot find home directory")
        .join(CLAUDE_DIR)
}

fn list_profile_files() -> Vec<String> {
    let dir = claude_dir();
    let mut profiles = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Match settings.json.* but NOT settings.json itself or backup files
            if name_str.starts_with("settings.json.") {
                let suffix = &name_str["settings.json.".len()..];
                // Skip backup files (settings.json.bak-*)
                if !suffix.starts_with("bak-") {
                    profiles.push(suffix.to_string());
                }
            }
        }
    }

    profiles.sort();
    profiles
}

fn main() {
    let cmd = Commands::parse();

    match cmd {
        Commands::List => {
            let profiles = list_profile_files();
            if profiles.is_empty() {
                println!("No profile files found.");
            } else {
                for p in profiles {
                    println!("{}", p);
                }
            }
        }
        Commands::Apply { suffix } => {
            let dir = claude_dir();
            let current = dir.join(SETTINGS_BASE);
            let target = dir.join(format!("{}.{}", SETTINGS_BASE, suffix));

            // Check current exists
            if !current.exists() {
                eprintln!("ERROR: {} does not exist", current.display());
                std::process::exit(1);
            }

            // Check target exists
            if !target.exists() {
                eprintln!("ERROR: {} does not exist", target.display());
                std::process::exit(1);
            }

            // Create backup name with timestamp
            let now = chrono::Local::now();
            let bak_name = format!("settings.json.bak-{}", now.format("%Y%m%d%H%M%S"));
            let bak_path = dir.join(&bak_name);

            // Backup current settings.json
            println!("Backing up current settings to {}...", bak_name);
            if let Err(e) = fs::copy(&current, &bak_path) {
                eprintln!("ERROR: Failed to backup: {}", e);
                std::process::exit(1);
            }

            // Copy target to settings.json
            println!("Applying profile '{}'...", suffix);
            if let Err(e) = fs::copy(&target, &current) {
                eprintln!("ERROR: Failed to apply profile: {}", e);
                eprintln!("NOTE: Backup remains at {}", bak_name);
                std::process::exit(1);
            }

            println!(
                "Done! Backup: {}, Applied: settings.json.{}",
                bak_name, suffix
            );
        }
    }
}
