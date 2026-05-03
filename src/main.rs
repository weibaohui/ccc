use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CLAUDE_DIR: &str = ".claude";
const SETTINGS_BASE: &str = "settings.json";

#[derive(Parser, Debug)]
#[command(name = "ccc")]
#[command(about = "Claude settings switcher", long_about = None)]
enum Cli {
    /// List all available settings profiles
    List,
    /// View a settings profile (default: current)
    View {
        /// Profile suffix to view (e.g., "zai", "minimax"). If omitted, view current settings.
        suffix: Option<String>,
    },
    /// Apply a settings profile (backup current + replace)
    Apply {
        /// Profile suffix to apply (e.g., "zai", "minimax")
        suffix: String,
    },
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

fn settings_path(suffix: Option<&str>) -> PathBuf {
    let dir = claude_dir();
    match suffix {
        Some(s) => dir.join(format!("{}.{}", SETTINGS_BASE, s)),
        None => dir.join(SETTINGS_BASE),
    }
}

/// Parse settings.json into a structured view
#[derive(Deserialize, Debug)]
struct Settings {
    env: Option<HashMap<String, serde_json::Value>>,
    attribution: Option<serde_json::Value>,
    verbose: Option<bool>,
    enabled_plugins: Option<HashMap<String, bool>>,
    extra_known_marketplaces: Option<serde_json::Value>,
    always_thinking_enabled: Option<bool>,
    include_co_authored_by: Option<bool>,
    skip_dangerous_mode_permission_prompt: Option<bool>,
    claude_code_disable_nonessential_traffic: Option<serde_json::Value>,
    // Catch-all for unknown fields
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl Settings {
    fn from_file(path: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))
    }

    fn print_summary(&self, label: &str) {
        println!("{}", label);
        println!("{}", "=".repeat(50));

        if let Some(env) = &self.env {
            println!("  env:");
            if let Some(token) = env.get("ANTHROPIC_AUTH_TOKEN") {
                if let Some(s) = token.as_str() {
                    // Mask token for privacy
                    let masked = if s.len() > 12 {
                        format!("{}...{}", &s[..4], &s[s.len() - 4..])
                    } else {
                        s.to_string()
                    };
                    println!("    ANTHROPIC_AUTH_TOKEN: {}", masked);
                }
            }
            if let Some(url) = env.get("ANTHROPIC_BASE_URL") {
                if let Some(s) = url.as_str() {
                    println!("    ANTHROPIC_BASE_URL: {}", s);
                }
            }
            if let Some(model) = env.get("ANTHROPIC_MODEL") {
                if let Some(s) = model.as_str() {
                    println!("    ANTHROPIC_MODEL: {}", s);
                }
            }
            if let Some(model) = env.get("ANTHROPIC_REASONING_MODEL") {
                if let Some(s) = model.as_str() {
                    println!("    ANTHROPIC_REASONING_MODEL: {}", s);
                }
            }
        }

        if let Some(v) = self.verbose {
            println!("  verbose: {}", v);
        }
        if let Some(v) = self.always_thinking_enabled {
            println!("  alwaysThinkingEnabled: {}", v);
        }
        if let Some(v) = self.include_co_authored_by {
            println!("  includeCoAuthoredBy: {}", v);
        }
        if let Some(v) = self.skip_dangerous_mode_permission_prompt {
            println!("  skipDangerousModePermissionPrompt: {}", v);
        }
        if let Some(plugins) = &self.enabled_plugins {
            println!("  enabledPlugins: {}", plugins.len());
            for (k, v) in plugins {
                println!("    {}: {}", k, v);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::List => {
            let profiles = list_profile_files();
            if profiles.is_empty() {
                println!("No profile files found.");
            } else {
                for p in profiles {
                    println!("{}", p);
                }
            }
        }
        Cli::View { suffix } => {
            let path = settings_path(suffix.as_deref());

            if !path.exists() {
                eprintln!("ERROR: {} does not exist", path.display());
                std::process::exit(1);
            }

            match Settings::from_file(&path) {
                Ok(settings) => {
                    let label = match &suffix {
                        Some(s) => format!("Profile: settings.json.{}", s),
                        None => "Current settings.json".to_string(),
                    };
                    settings.print_summary(&label);
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to parse settings: {}", e);
                    // Fall back to raw content
                    if let Ok(content) = fs::read_to_string(&path) {
                        println!("Raw content:\n{}", content);
                    }
                }
            }
        }
        Cli::Apply { suffix } => {
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
