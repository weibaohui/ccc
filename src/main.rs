use clap::Parser;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write as IoWrite};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use zip::ZipArchive;

const CLAUDE_DIR: &str = ".claude";
const SETTINGS_BASE: &str = "settings.json";
const SKILL_NAME: &str = "ccc";

// Embedded skill zip (generated at compile time by build.rs)
static SKILL_ZIP: &[u8] = include_bytes!("ccc_skill.bin");

// Global result store shared across threads
static RESULTS: Lazy<Arc<Mutex<Vec<VerifyResult>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Parser, Debug)]
#[command(name = "ccc")]
#[command(about = "Claude settings changer for AI agents", long_about = None)]
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
    /// Batch verify all available profiles with real-time progress
    Batch {
        /// Profile suffixes to verify (if omitted, verify all profiles)
        #[clap(trailing_var_arg = true)]
        profiles: Vec<String>,
    },
    /// Manage the embedded CCC skill (install to ~/.claude/skills/)
    Skill {
        /// Skill action: install
        #[clap(subcommand)]
        action: SkillAction,
    },
}

#[derive(Parser, Debug)]
enum SkillAction {
    /// Install the embedded CCC skill to ~/.claude/skills/ccc/
    Install,
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

// ---------- Batch verify types ----------

#[derive(Clone)]
struct VerifyResult {
    suffix: String,
    profile_path: PathBuf,
    status: Arc<Mutex<Option<ProfileStatus>>>,
}

enum ProfileStatus {
    Running,
    Passed { model: String },
    Failed { step: &'static str, message: String },
}

fn verify_single_profile(suffix: &str, profile_path: &PathBuf) -> VerifyResult {
    let result = VerifyResult {
        suffix: suffix.to_string(),
        profile_path: profile_path.clone(),
        status: Arc::new(Mutex::new(None)),
    };
    let status = result.status.clone();
    let suffix = suffix.to_string();
    let profile_path = profile_path.clone();

    // Mark as running
    *status.lock().unwrap() = Some(ProfileStatus::Running);

    // Step 1: JSON parsing
    let content = match fs::read_to_string(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            *status.lock().unwrap() = Some(ProfileStatus::Failed {
                step: "read",
                message: e.to_string(),
            });
            return result;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            *status.lock().unwrap() = Some(ProfileStatus::Failed {
                step: "json",
                message: e.to_string(),
            });
            return result;
        }
    };

    // Step 2: required fields
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
        .unwrap_or("N/A")
        .to_string();

    if !has_token || !has_url {
        *status.lock().unwrap() = Some(ProfileStatus::Failed {
            step: "fields",
            message: format!(
                "token={}, url={}",
                if has_token { "ok" } else { "missing" },
                if has_url { "ok" } else { "missing" }
            ),
        });
        return result;
    }

    // Step 3: real API call
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
                *status.lock().unwrap() = Some(ProfileStatus::Passed { model });
            } else {
                let msg = if !stdout.is_empty() {
                    format!("API error: {}", stdout.trim())
                } else if !stderr.is_empty() {
                    format!("stderr: {}", stderr.trim().chars().take(120).collect::<String>())
                } else {
                    format!("exit code: {:?}", out.status.code())
                };
                *status.lock().unwrap() = Some(ProfileStatus::Failed {
                    step: "api",
                    message: msg,
                });
            }
        }
        Err(e) => {
            *status.lock().unwrap() = Some(ProfileStatus::Failed {
                step: "api",
                message: e.to_string(),
            });
        }
    }

    result
}

fn print_batch_progress(profiles: &[String], start: Instant) {
    let total = profiles.len();
    loop {
        thread::sleep(Duration::from_millis(300));
        let results = RESULTS.lock().unwrap();
        let done = results.len();
        let passed = results
            .iter()
            .filter(|r| {
                matches!(
                    *r.status.lock().unwrap(),
                    Some(ProfileStatus::Passed { .. })
                )
            })
            .count();
        let failed = results
            .iter()
            .filter(|r| {
                matches!(
                    *r.status.lock().unwrap(),
                    Some(ProfileStatus::Failed { .. })
                )
            })
            .count();

        // Print progress line (overwrite with \r)
        let elapsed = start.elapsed().as_secs();
        print!(
            "\r  [{}s] Progress: {}/{} | ✅ {} | ❌ {} | ⏳ {}   ",
            elapsed, done, total, passed, failed, total - done
        );
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Check if all done
        if done >= total {
            println!(); // newline after progress bar
            break;
        }
    }
}

fn print_summary_table(profiles: &[String]) {
    let results = RESULTS.lock().unwrap();
    let total = profiles.len();
    let passed = results
        .iter()
        .filter(|r| {
            matches!(
                *r.status.lock().unwrap(),
                Some(ProfileStatus::Passed { .. })
            )
        })
        .count();
    let failed = results
        .iter()
        .filter(|r| {
            matches!(
                *r.status.lock().unwrap(),
                Some(ProfileStatus::Failed { .. })
            )
        })
        .count();

    println!();
    println!("{}", "═".repeat(60));
    println!("  BATCH VERIFY RESULTS  ({} profiles)", total);
    println!("{}", "─".repeat(60));
    println!(
        "  {:<20} {:<15} {}",
        "Profile", "Status", "Detail"
    );
    println!("{}", "─".repeat(60));

    for result in results.iter() {
        let status_guard = result.status.lock().unwrap();
        let (icon, status_str, detail) = match status_guard.as_ref() {
            Some(ProfileStatus::Passed { model }) => {
                ("✅ PASS".to_string(), "passed".to_string(), format!("model={}", model))
            }
            Some(ProfileStatus::Failed { step, message }) => {
                ("❌ FAIL".to_string(), format!("failed@{}", step), message.chars().take(35).collect())
            }
            Some(ProfileStatus::Running) => ("⏳ RUN".to_string(), "running".to_string(), "".to_string()),
            None => ("⚪ PEND".to_string(), "pending".to_string(), "".to_string()),
        };
        println!("  {:<20} {:<15} {}", result.suffix, icon, detail);
    }

    println!("{}", "─".repeat(60));
    println!(
        "  Total: {} | ✅ Passed: {} | ❌ Failed: {}",
        total, passed, failed
    );
    println!("{}", "═".repeat(60));

    if failed == 0 && passed > 0 {
        println!("  🎉 All profiles passed!");
    } else if failed > 0 {
        println!(
            "  ⚠️  {} profile(s) need attention.",
            failed
        );
    }
}

// ---------- Main ----------

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

            if !current.exists() {
                eprintln!("ERROR: {} does not exist", current.display());
                std::process::exit(1);
            }
            if !target.exists() {
                eprintln!("ERROR: {} does not exist", target.display());
                std::process::exit(1);
            }

            let now = chrono::Local::now();
            let bak_name = format!("settings.json.bak-{}", now.format("%Y%m%d%H%M%S"));
            let bak_path = dir.join(&bak_name);

            println!("Backing up current settings to {}...", bak_name);
            if let Err(e) = fs::copy(&current, &bak_path) {
                eprintln!("ERROR: Failed to backup: {}", e);
                std::process::exit(1);
            }

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

            let mut cmd = Command::new("claude");
            cmd.arg("--settings").arg(&profile_path);

            if command.is_empty() {
                // interactive mode
            } else if command[0] == "-p"
                || command[0] == "--print"
                || command[0] == "-c"
                || command[0] == "--continue"
            {
                for arg in &command {
                    cmd.arg(arg);
                }
            } else {
                cmd.arg("-p");
                cmd.arg(command.join(" "));
            }

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

            match Settings::from_file(&profile_path) {
                Ok(_settings) => {
                    println!("[1/2] JSON parsing: OK");
                }
                Err(e) => {
                    eprintln!("[1/2] JSON parsing: FAILED - {}", e);
                    std::process::exit(1);
                }
            }

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
                        println!();
                        println!("Verification PASSED ✅");
                        println!(
                            "Profile 'settings.json.{}' is valid and usable.",
                            suffix
                        );
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
        Cli::Batch { profiles } => {
            let all_profiles = if profiles.is_empty() {
                list_profile_files()
            } else {
                profiles
            };

            if all_profiles.is_empty() {
                println!("No profiles found to verify.");
                std::process::exit(0);
            }

            let total = all_profiles.len();
            println!("Batch verify: {} profile(s)", total);
            println!("{}", "─".repeat(40));

            // Spawn monitoring thread for progress display
            let start = Instant::now();
            let all_profiles_for_monitor = all_profiles.clone();
            let monitor = thread::spawn(move || {
                print_batch_progress(&all_profiles_for_monitor, start);
            });

            // Launch verification threads
            let mut handles = Vec::new();
            for suffix in &all_profiles {
                let dir = claude_dir();
                let profile_path = dir.join(format!("{}.{}", SETTINGS_BASE, suffix));
                let suffix_clone = suffix.clone();
                let path_clone = profile_path.clone();

                let handle = thread::spawn(move || {
                    let result = verify_single_profile(&suffix_clone, &path_clone);
                    let mut res = RESULTS.lock().unwrap();
                    res.push(result);
                });
                handles.push(handle);
            }

            // Wait for all verify threads
            for h in handles {
                h.join().ok();
            }

            // Wait for monitor to finish
            monitor.join().ok();

            // Print summary table
            print_summary_table(&all_profiles);

            // Exit with error if any failed
            let results = RESULTS.lock().unwrap();
            let any_failed = results
                .iter()
                .any(|r| matches!(*r.status.lock().unwrap(), Some(ProfileStatus::Failed { .. })));
            if any_failed {
                std::process::exit(1);
            }
        }
        Cli::Skill { action } => match action {
            SkillAction::Install => {
                if let Err(e) = skill_install() {
                    eprintln!("Skill install failed: {}", e);
                    std::process::exit(1);
                }
            }
        },
    }
}

fn skill_install() -> Result<(), String> {
    let skill_bytes = SKILL_ZIP;
    if skill_bytes.is_empty() {
        return Err("No embedded skill found. The skill was not compiled into the binary.".to_string());
    }

    let cursor = std::io::Cursor::new(skill_bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Failed to read skill zip: {}", e))?;

    // Target directory: ~/.claude/skills/ccc/
    let skill_dir = claude_dir().join("skills").join(SKILL_NAME);
    fs::create_dir_all(&skill_dir).map_err(|e| format!("Failed to create skill directory: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let outpath = skill_dir.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create dir: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file {:?}: {}", outpath, e))?;
        }
    }

    println!("✅ Skill '{}' installed to:", SKILL_NAME);
    println!("   {}", skill_dir.display());
    Ok(())
}
