use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CLAUDE_DIR: &str = ".claude";
const SETTINGS_BASE: &str = "settings.json";

#[derive(Parser, Debug)]
#[command(name = "ccc")]
#[command(about = "Claude settings switcher for AI agents", long_about = None)]
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
    /// Run a command with a specific settings profile (does not modify global settings.json)
    Run {
        /// Profile suffix to use (e.g., "zai", "minimax")
        suffix: String,
        /// Command to execute (pass through to claude)
        #[clap(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Verify if a settings profile is valid by making a real API call
    Verify {
        /// Profile suffix to verify (e.g., "zai", "minimax")
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
        Cli::Run { suffix, command } => {
            let dir = claude_dir();
            let profile_path = dir.join(format!("{}.{}", SETTINGS_BASE, suffix));

            if !profile_path.exists() {
                eprintln!("ERROR: {} does not exist", profile_path.display());
                std::process::exit(1);
            }

            // Build claude command: claude --settings <path> [command...]
            let mut cmd = Command::new("claude");
            cmd.arg("--settings").arg(&profile_path);

            if command.is_empty() {
                // Interactive mode if no command given
            } else {
                // Join all args as a single prompt string for -p mode
                // If first arg is -p or --print, pass through as-is
                if command[0] == "-p" || command[0] == "--print" || command[0] == "-c" || command[0] == "--continue" {
                    for arg in &command {
                        cmd.arg(arg);
                    }
                } else {
                    // Wrap as a prompt
                    cmd.arg("-p");
                    cmd.arg(command.join(" "));
                }
            }

            // Inherit stdin/stdout/stderr for interactive use
            let status = cmd.status();

            match status {
                Ok(exit_status) => {
                    std::process::exit(exit_status.code().unwrap_or(1));
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to execute claude: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Cli::Verify { suffix } => {
            let dir = claude_dir();
            let profile_path = dir.join(format!("{}.{}", SETTINGS_BASE, suffix));

            if !profile_path.exists() {
                eprintln!("ERROR: {} does not exist", profile_path.display());
                std::process::exit(1);
            }

            // Try to parse the JSON first
            match Settings::from_file(&profile_path) {
                Ok(_settings) => {
                    println!("[1/2] JSON parsing: OK");
                }
                Err(e) => {
                    eprintln!("[1/2] JSON parsing: FAILED - {}", e);
                    std::process::exit(1);
                }
            }

            // Check required env fields
            let content = fs::read_to_string(&profile_path).unwrap();
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    let env = json.get("env").and_then(|v| v.as_object());
                    let has_token = env
                        .and_then(|m| m.get("ANTHROPIC_AUTH_TOKEN"))
                        .map(|v| v.is_string())
                        .unwrap_or(false);
                    let has_url = env
                        .and_then(|m| m.get("ANTHROPIC_BASE_URL"))
                        .map(|v| v.is_string())
                        .unwrap_or(false);
                    let model = env
                        .and_then(|m| m.get("ANTHROPIC_MODEL"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");

                    if !has_token {
                        eprintln!("[2/2] Required field ANTHROPIC_AUTH_TOKEN: MISSING");
                        std::process::exit(1);
                    }
                    if !has_url {
                        eprintln!("[2/2] Required field ANTHROPIC_BASE_URL: MISSING");
                        std::process::exit(1);
                    }
                    println!("[2/2] Required fields: OK (model={})", model);
                }
                Err(e) => {
                    eprintln!("[2/2] JSON structure: FAILED - {}", e);
                    std::process::exit(1);
                }
            }

            // Make a real API call to verify credentials
            println!("[3/3] Making API call to verify credentials...");
            let output = Command::new("claude")
                .arg("--settings")
                .arg(&profile_path)
                .arg("-p")
                .arg("Reply with exactly one word: OK")
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);

                    if out.status.success() && stdout.trim().contains("OK") {
                        println!("[3/3] API call: OK");
                        println!("");
                        println!("Verification PASSED ✅");
                        println!("Profile 'settings.json.{}' is valid and usable.", suffix);
                    } else {
                        eprintln!("[3/3] API call: FAILED");
                        if !stdout.is_empty() {
                            eprintln!("stdout: {}", stdout.trim());
                        }
                        if !stderr.is_empty() {
                            eprintln!("stderr: {}", stderr.trim());
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("[3/3] API call: FAILED - {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
