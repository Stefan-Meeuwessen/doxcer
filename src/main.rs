//////////////////////////////////////////////////////////
// AUTHOR   : Stefan B. J. Meeuwessen
// CREATION : 2025-11-05
// VERSION  : 3.0.1
//////////////////////////////////////////////////////////


//! Doxcer — Markdown documentation generator.
//!
//! CLI tool that reads a Notebook
//! strips platform metadata, builds a structured prompt and calls Azure AI Foundry
//! to generate Markdown documentation.
//!
//! Optionally, column definitions are pulled from a central "definitions" table and included in the prompt.


// ----------------------------
// Compiler Directives
// ----------------------------

// #![allow(unused)]
// #![allow(unused_doc_comments)]


// ----------------------------
// Imports
// ----------------------------

// Standard Libraries
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, Instant};

// External Libraries
use chrono::Utc;
use chrono_tz::Europe::Amsterdam;
use dotenvy;
use fetch_definitions::FabricDefinitionConfig;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

// Unit Tests
#[cfg(test)]
mod unit_tests;
mod fetch_definitions;
mod fetch_secrets;


// ----------------------------
// Data Structures
// ----------------------------

#[derive(Serialize)]
struct ChatRequest
{
    /// Type: Struct.
    /// Input:
    /// - Values assigned by caller before serialization.
    /// Output:
    /// - JSON payload for chat completion requests.
    /// Exceptions:
    /// - None.

    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message
{
    /// Type: Struct.
    /// Input:
    /// - Values assigned by caller before serialization.
    /// Output:
    /// - JSON message object in `ChatRequest`.
    /// Exceptions:
    /// - None.

    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse
{
    /// Type: Struct.
    /// Input:
    /// - JSON response payload from Azure OpenAI.
    /// Output:
    /// - Deserialized response subset used by this application.
    /// Exceptions:
    /// - None.

    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice
{
    /// Type: Struct.
    /// Input:
    /// - JSON `choices[]` entry from API response.
    /// Output:
    /// - Deserialized choice containing one message.
    /// Exceptions:
    /// - None.

    message: ChoiceMessage,
}

#[derive(Deserialize, Debug)]
struct ChoiceMessage
{
    /// Type: Struct.
    /// Input:
    /// - JSON message object from API response.
    /// Output:
    /// - Deserialized assistant content text.
    /// Exceptions:
    /// - None.

    role: Option<String>,
    content: String,
}

#[derive(Debug, Eq, PartialEq)]
struct CliArgs
{
    /// Type: Struct.
    /// Input:
    /// - Parsed CLI tokens.
    /// Output:
    /// - Runtime CLI argument object.
    /// Exceptions:
    /// - None.

    file_path: String,
    profile: PromptProfile,
}

struct EnvParameters
{
    /// Type: Struct.
    /// Input:
    /// - Environment variables loaded from split env files.
    /// Output:
    /// - Strongly-typed runtime configuration.
    /// Exceptions:
    /// - None.

    // AI Model
    ai_enabled: bool,
    ai_base_url: String,
    ai_model: String,
    ai_version: String,
    ai_task: String,

    // Azure Key Vault
    akv_enabled: bool,
    akv_base_url: String,
    akv_secret_ai: String,

    // Definition DB
    definition_database_enabled: bool,
    
    // Definition DB Fabric
    definition_fabric_database_enabled: bool,
    definition_fabric_database: String,
    akv_secret_definition_fabric_endpoint: String,
    akv_secret_definition_fabric_client_id: String,
    akv_secret_definition_fabric_password: String,

    // Definition DB Azure
    definition_azure_database_enabled: bool,
    definition_azure_database: String,
    akv_secret_definition_azure_endpoint: String,
    akv_secret_definition_azure_client_id: String,
    akv_secret_definition_azure_password: String,

    // ODBC
    odbc_batch_size: usize,
    odbc_max_byte_size: usize,
}

struct PromptProfileSpec
{
    /// Type: Struct.
    /// Input:
    /// - Compile-time profile metadata values.
    /// Output:
    /// - Single source of truth for profile names, selectors, and template stems.
    /// Exceptions:
    /// - None.

    profile: PromptProfile,
    name: &'static str,
    selector_flags: &'static [&'static str],
    template_stem: &'static str,
}


// ----------------------------
// Data Enumerations
// ----------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PromptProfile
{
    /// Type: Enum.
    /// Input:
    /// - Parsed selector flag from CLI.
    /// Output:
    /// - Selected prompt profile variant.
    /// Exceptions:
    /// - None.

    Default,
    Fabric,
    Synapse,
    Databricks,
    PowerBi,
    Aws,
    DataFactory,
    Pipeline,
}


// ----------------------------
// Static Selector Flags
// ----------------------------

static PROMPT_PROFILE_SPECS: &[PromptProfileSpec] = &[
    PromptProfileSpec
    {
        profile: PromptProfile::Default,
        name: "default",
        selector_flags: &[],
        template_stem: "default",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::Fabric,
        name: "fabric",
        selector_flags: &["-fabric"],
        template_stem: "fabric",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::Synapse,
        name: "synapse",
        selector_flags: &["-synapse"],
        template_stem: "synapse",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::Databricks,
        name: "databricks",
        selector_flags: &["-databricks"],
        template_stem: "databricks",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::PowerBi,
        name: "powerbi",
        selector_flags: &["-powerbi"],
        template_stem: "powerbi",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::Aws,
        name: "aws",
        selector_flags: &["-aws"],
        template_stem: "aws",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::DataFactory,
        name: "datafactory",
        selector_flags: &["-datafactory"],
        template_stem: "datafactory",
    },
    PromptProfileSpec
    {
        profile: PromptProfile::Pipeline,
        name: "pipeline",
        selector_flags: &["-pipeline"],
        template_stem: "pipeline",
    },
];

const USAGE_TEXT_TEMPLATE: &str = "[INF] - Usage:
[INF] -   doxcer <path/to/notebook.py>
[INF] -   doxcer [selector] <path/to/notebook.py>
[INF] -   doxcer --help
[INF] - Selectors:
[INF] -   {selectors}
[INF] - The path and selector can be provided in any order.";


// ----------------------------
// .ENV CONFIG
// ----------------------------

static ENVCONFIG: Lazy<EnvParameters> = Lazy::new(||
{
    load_env();
    EnvParameters
    {
        // Azure AI Foundry model configuration
        ai_enabled: env::var("AI_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",
        ai_base_url: env::var("AI_BASE_URL").expect("[WRN] - Missing AI_BASE_URL"),
        ai_model: env::var("AI_MODEL").expect("[WRN] - Missing AI_MODEL"),
        ai_version: env::var("AI_VERSION").expect("[WRN] - Missing AI_VERSION"),
        ai_task: env::var("AI_TASK").expect("[WRN] - Missing AI_TASK"),

        // Azure Key Vault Secrets
        akv_enabled: env::var("AKV_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",
        akv_base_url: env::var("AKV_BASE_URL").expect("[WRN] - Missing AKV_BASE_URL"),
        akv_secret_ai: env::var("AKV_SECRET_AI").expect("[WRN] - Missing AKV_SECRET_AI"),

        // Definition database
        definition_database_enabled: env::var("DEFINITION_DATABASE_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",

        // Fabric SQL Definition database Azure Key Vault
        definition_fabric_database_enabled: env::var("DEFINITION_FABRIC_DATABASE_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",
        definition_fabric_database: env::var("DEFINITION_FABRIC_DATABASE").expect("[WRN] - Missing DEFINITION_FABRIC_DATABASE"),
        akv_secret_definition_fabric_endpoint: env::var("AKV_SECRET_DEFINITION_FABRIC_ENDPOINT").expect("[WRN] - Missing AKV_SECRET_DEFINITION_FABRIC_ENDPOINT"),
        akv_secret_definition_fabric_client_id: env::var("AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_CLIENT").expect("[WRN] - Missing AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_CLIENT"),
        akv_secret_definition_fabric_password: env::var("AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_PASSWORD").expect("[WRN] - Missing AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_PASSWORD"),

        // Azure SQL Definition database Azure Key Vault
        definition_azure_database_enabled: env::var("DEFINITION_AZURE_DATABASE_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true",
        definition_azure_database: env::var("DEFINITION_AZURE_DATABASE").expect("Missing DEFINITION_AZURE_DATABASE"),
        akv_secret_definition_azure_endpoint: env::var("AKV_SECRET_DEFINITION_AZURE_ENDPOINT").expect("[WRN] - Missing AKV_SECRET_DEFINITION_AZURE_ENDPOINT"),
        akv_secret_definition_azure_client_id: env::var("AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_CLIENT").expect("[WRN] - Missing AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_CLIENT"),
        akv_secret_definition_azure_password: env::var("AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_PASSWORD").expect("[WRN] - Missing AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_PASSWORD"),

        // ODBC Database connection configuration
        odbc_batch_size: env::var("ODBC_BATCH_SIZE").unwrap_or_else(|_| "200".to_string()).parse().expect("[WRN] - Invalid ODBC_BATCH_SIZE"),
        odbc_max_byte_size: env::var("ODBC_MAX_BYTE_SIZE").unwrap_or_else(|_| "4096".to_string()).parse().expect("[WRN] - Invalid ODBC_MAX_BYTE_SIZE"),
    }
});


// ----------------------------
// Helper Functions
// ----------------------------

fn load_env()
{
    //! Loads required `.env` files into process environment variables.
    //!
    //! # Inputs
    //! - None. Uses [`find_env_paths`] internally.
    //!
    //! # Side Effects
    //! - Sets process environment variables from each required env file.
    //!
    //! # Panics
    //! - If a required env file is missing.
    //! - If loading/parsing a required env file fails.

    for env_path in find_env_paths()
    {
        if !env_path.exists()
        {
            panic!(
                "[ERR] - Missing env file '{}'. Run 'set-up-doxcer.ps1' to generate runtime configuration.",
                env_path.display()
            );
        }

        println!("[INF] - Loading environment from {}", env_path.display());

        // system.env is intentionally not a dotenv file; it contains only path mapping metadata.
        let is_system_env = env_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("system.env"))
            .unwrap_or(false);

        if is_system_env
        {
            if parse_system_env_absolute_path(&env_path).is_none()
            {
                panic!(
                    "[ERR] - Invalid system env file '{}': missing ABSOLUTE_DOXCER_PATH.",
                    env_path.display()
                );
            }
            continue;
        }

        dotenvy::from_path(&env_path).unwrap_or_else(|err|
        {
            panic!(
                "[ERR] - Failed to load env file '{}': {}",
                env_path.display(),
                err
            )
        });
    }
}

fn has_repo_markers(path: &Path) -> bool
{
    //! Checks whether a path looks like the project repository root.
    //!
    //! # Inputs
    //! - `path`: Candidate directory path.
    //!
    //! # Returns
    //! - `true` when required repository markers exist (`Cargo.toml`, `config/`, `templates/`).

    path.join("Cargo.toml").is_file()
        && path.join("config").is_dir()
        && path.join("templates").is_dir()
}

fn parse_system_env_absolute_path(system_env_path: &Path) -> Option<PathBuf>
{
    //! Extracts `ABSOLUTE_DOXCER_PATH` from `config/system.env`.
    //!
    //! # Inputs
    //! - `system_env_path`: Path to the system env mapping file.
    //!
    //! # Returns
    //! - `Some(PathBuf)` when a non-empty `ABSOLUTE_DOXCER_PATH=` value is found.
    //! - `None` when the file cannot be read or the key/value is missing or empty.

    let content = fs::read_to_string(system_env_path).ok()?;
    for line in content.lines()
    {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#')
        {
            continue;
        }

        let Some(value) = trimmed.strip_prefix("ABSOLUTE_DOXCER_PATH=") else
        {
            continue;
        };

        let normalized = value.trim().trim_matches('"').trim_matches('\'');
        if normalized.is_empty()
        {
            continue;
        }

        return Some(PathBuf::from(normalized));
    }

    None
}

fn find_repo_root_in_ancestors(start: &Path) -> Option<PathBuf>
{
    //! Walks ancestor directories to locate a valid repository root.
    //!
    //! # Inputs
    //! - `start`: Starting path used for ancestor traversal.
    //!
    //! # Returns
    //! - `Some(PathBuf)` when a directory with repository markers is found directly.
    //! - `Some(PathBuf)` when a valid mapped root is found via `config/system.env`.
    //! - `None` when no valid root can be resolved.

    for ancestor in start.ancestors()
    {
        if has_repo_markers(ancestor)
        {
            return Some(ancestor.to_path_buf());
        }

        let system_env = ancestor.join("config").join("system.env");
        if let Some(repo_root) = parse_system_env_absolute_path(&system_env)
        {
            if has_repo_markers(&repo_root)
            {
                return Some(repo_root);
            }
        }
    }

    None
}

fn find_repo_root_path() -> PathBuf
{
    //! Resolves the repository root path for runtime resources.
    //!
    //! # Inputs
    //! - None.
    //!
    //! # Returns
    //! - Repository root path that contains expected project markers.
    //!
    //! # Panics
    //! - If no valid repository root can be discovered.

    if let Ok(value) = env::var("ABSOLUTE_DOXCER_PATH")
    {
        let candidate = value.trim().trim_matches('"').trim_matches('\'');
        if !candidate.is_empty()
        {
            let candidate_path = PathBuf::from(candidate);
            if has_repo_markers(&candidate_path)
            {
                return candidate_path;
            }

            eprintln!(
                "[WRN] - ABSOLUTE_DOXCER_PATH is set but invalid: {}",
                candidate_path.display()
            );
        }
    }

    if let Ok(current_dir) = env::current_dir()
    {
        if let Some(repo_root) = find_repo_root_in_ancestors(&current_dir)
        {
            return repo_root;
        }
    }

    if let Ok(exe_path) = env::current_exe()
    {
        if let Some(repo_root) = find_repo_root_in_ancestors(&exe_path)
        {
            return repo_root;
        }
    }

    panic!("[ERR] - Failed to locate repository root. Run 'set-up-doxcer.ps1' first.");
}

fn find_env_paths() -> Vec<PathBuf>
{
    //! Returns required environment file paths in load order.
    //!
    //! # Inputs
    //! - None.
    //!
    //! # Returns
    //! - Paths, in order:
    //!   - `config/system.env`
    //!   - `config/definitions.env`
    //!   - `config/azure_key_vault.env`
    //!   - `config/ai_model.env`
    //!
    //! # Panics
    //! - If repository root discovery fails.
    
    let repo = find_repo_root_path();
    let config_dir = repo.join("config");

    vec![
        config_dir.join("system.env"),
        config_dir.join("definitions.env"),
        config_dir.join("azure_key_vault.env"),
        config_dir.join("ai_model.env"),
    ]
}

fn print_usage()
{
    //! Prints CLI usage instructions to stderr.

    eprintln!("{}", usage_text());
}

fn usage_text() -> String
{
    //! Builds CLI usage instructions with the current selector list.

    let selector_display = supported_selector_list().replace(", ", " | ");
    USAGE_TEXT_TEMPLATE.replace("{selectors}", &selector_display)
}

fn is_help_requested(args: &[String]) -> bool
{
    //! Returns `true` when CLI input is exactly `doxcer --help`.

    args.len() == 2 && args[1] == "--help"
}

fn prompt_profile_spec(profile: PromptProfile) -> &'static PromptProfileSpec
{
    //! Returns the metadata entry for a prompt profile.
    //!
    //! # Inputs
    //! - `profile`: Prompt profile variant.
    //!
    //! # Returns
    //! - Profile metadata entry from the static registry.
    //!
    //! # Panics
    //! - If metadata for the given profile is missing.

    PROMPT_PROFILE_SPECS
        .iter()
        .find(|spec| spec.profile == profile)
        .expect("[ERR] - Missing prompt profile specification")
}

fn parse_profile_selector(arg: &str) -> Option<PromptProfile>
{
    //! Parses a CLI token as a prompt selector flag.
    //!
    //! # Inputs
    //! - `arg`: Raw CLI token.
    //!
    //! # Returns
    //! - `Some(profile)` when the token matches a known selector.
    //! - `None` when the token is not a known selector.

    for spec in PROMPT_PROFILE_SPECS
    {
        if spec.selector_flags.iter().any(|selector| *selector == arg)
        {
            return Some(spec.profile);
        }
    }

    None
}

fn profile_selector_name(profile: PromptProfile) -> &'static str
{
    //! Returns the canonical selector name for a prompt profile.
    //!
    //! # Inputs
    //! - `profile`: Prompt profile variant.
    //!
    //! # Returns
    //! - Canonical selector name without a leading `-`.

    prompt_profile_spec(profile).name
}

fn supported_selector_list() -> String
{
    //! Builds a comma-separated list of supported selector flags.
    //!
    //! # Inputs
    //! - None.
    //!
    //! # Returns
    //! - Comma-separated selector list in registry order.

    PROMPT_PROFILE_SPECS
        .iter()
        .filter_map(|spec| spec.selector_flags.first().copied())
        .collect::<Vec<&str>>()
        .join(", ")
}

fn parse_cli_args(args: &[String]) -> std::result::Result<CliArgs, String>
{
    //! Parses and validates CLI arguments.
    //!
    //! # Inputs
    //! - `args`: Raw process arguments including executable name.
    //!
    //! # Returns
    //! - `Ok(CliArgs)` when the path/selectors are valid.
    //! - `Err(String)` with a user-facing validation message.

    if args.is_empty()
    {
        return Err("[ERR] - Missing executable name.".to_string());
    }

    let mut selector_profile: Option<PromptProfile> = None;
    let mut file_path: Option<String> = None;

    for arg in args.iter().skip(1)
    {
        if let Some(parsed_selector) = parse_profile_selector(arg)
        {
            if let Some(existing_selector) = selector_profile
            {
                if existing_selector != parsed_selector
                {
                    return Err(format!(
                        "[ERR] - Conflicting selectors: both '{}' and '{}' were provided.",
                        profile_selector_name(existing_selector),
                        profile_selector_name(parsed_selector)
                    ));
                }
            }
            else
            {
                selector_profile = Some(parsed_selector);
            }
            continue;
        }

        match arg.as_str()
        {
            _ if arg.starts_with('-') =>
            {
                return Err(format!(
                    "[ERR] - Unknown selector '{}'. Supported selectors: {}.",
                    arg,
                    supported_selector_list()
                ));
            }
            _ =>
            {
                if let Some(existing_path) = &file_path
                {
                    return Err(format!(
                        "[ERR] - Multiple input paths were provided: '{}' and '{}'.",
                        existing_path, arg
                    ));
                }
                file_path = Some(arg.to_string());
            }
        }
    }

    let profile = selector_profile.unwrap_or(PromptProfile::Default);

    let file_path = file_path
        .ok_or_else(|| "[ERR] - Missing required notebook path argument.".to_string())?;

    Ok(CliArgs
    {
        file_path,
        profile,
    })
}

fn find_prompt_path(profile: &PromptProfile) -> PathBuf
{
    //! Resolves the prompt template path for the selected profile.
    //!
    //! # Inputs
    //! - `profile`: Prompt profile selector.
    //!
    //! # Returns
    //! - Profile-specific template path when it exists.
    //! - Default template path when profile template is missing.
    //!
    //! # Panics
    //! - If repository root discovery fails.

    let prompt_file_stem = prompt_profile_spec(*profile).template_stem;

    let repo = find_repo_root_path();
    let template_dir = repo.join("templates");
    let selected_template = template_dir.join(format!("{}_prompt.md", prompt_file_stem));

    if selected_template.exists()
    {
        selected_template
    }
    else
    {
        template_dir.join("default_prompt.md")
    }
}

fn find_context_path() -> PathBuf
{
    //! Returns the path to `templates/context.md`.
    //!
    //! # Panics
    //! - If repository root discovery fails.
    
    let repo = find_repo_root_path();
    repo.join("templates")
        .join("context.md")
}

fn find_docs_path() -> PathBuf
{
    //! Returns the output directory for generated Markdown files.
    //!
    //! # Panics
    //! - If repository root discovery fails.

    let repo = find_repo_root_path();
    repo.join("docs")
        .join("newly-documented")
}

fn is_metadata_line(line: &str) -> bool
{
    //! Checks whether a source line is notebook metadata.
    //!
    //! # Inputs
    //! - `line`: Single notebook source line.
    //!
    //! # Returns
    //! - `true` when line starts with `# METADATA`, `# META`, or `# CELL`.

    let trimmed = line.trim_start();
    trimmed.starts_with("# METADATA")
        || trimmed.starts_with("# META")
        || trimmed.starts_with("# CELL")
}

fn strip_notebook_metadata(source: &str) -> String
{
    //! Removes metadata lines from notebook source text.
    //!
    //! # Inputs
    //! - `source`: Raw notebook source text.
    //!
    //! # Returns
    //! - Source text without metadata lines.

    let mut cleaned_lines: Vec<&str> = Vec::new();

    for line in source.lines()
    {
        if !is_metadata_line(line)
        {
            cleaned_lines.push(line);
        }
    }

    cleaned_lines.join("\n")
}

fn collapse_blank_lines(source: &str) -> String
{
    //! Collapses consecutive blank lines into a single blank line.
    //!
    //! # Inputs
    //! - `source`: Multi-line text.
    //!
    //! # Returns
    //! - Text with consecutive blank lines collapsed.

    let mut result: Vec<&str> = Vec::new();
    let mut previous_was_blank = false;

    for line in source.lines()
    {
        let is_blank = line.trim().is_empty();

        if is_blank
        {
            if !previous_was_blank
            {
                result.push(line);
                previous_was_blank = true;
            }
        }
        else
        {
            result.push(line);
            previous_was_blank = false;
        }
    }

    result.join("\n")
}

fn is_assistant_role(role: Option<&str>) -> bool
{
    //! Returns `true` when a response role is missing or explicitly assistant.
    //!
    //! # Inputs
    //! - `role`: Optional response role value from the API payload.
    //!
    //! # Returns
    //! - `true` for `None` to preserve compatibility with payloads that omit role.
    //! - `true` for case-insensitive `"assistant"`.
    //! - `false` for any other role.

    role
        .map(|value| value.eq_ignore_ascii_case("assistant"))
        .unwrap_or(true)
}

fn looks_like_internal_prompt(content: &str) -> bool
{
    //! Heuristically detects when model output appears to be a full prompt echo.
    //!
    //! # Inputs
    //! - `content`: Assistant text content.
    //!
    //! # Returns
    //! - `true` when content contains all internal prompt envelope markers.

    content.contains("Current date time:")
        && content.contains("Notebook filename:")
        && content.contains("Documentation template:")
        && content.contains("Code:")
}

fn env_flag_enabled(name: &str) -> bool
{
    //! Reads a boolean-like environment flag.
    //!
    //! # Inputs
    //! - `name`: Environment variable name.
    //!
    //! # Returns
    //! - `true` for values: `1`, `true`, `yes`, `on` (case-insensitive).
    //! - `false` otherwise.

    match env::var(name)
    {
        Ok(value) =>
        {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        }
        Err(_) => false,
    }
}

fn determine_output_names(input_path: &Path) -> (String, String)
{
    //! Derives output names from the input notebook path.
    //!
    //! # Inputs
    //! - `input_path`: Input notebook path.
    //!
    //! # Returns
    //! - `(output_file_name, output_file_name_ext)`.
    //! - Uses parent directory name when file is `notebook-content.py`.
    //!
    //! # Panics
    //! - If `input_path` has no filename.

    let filename_os = input_path
        .file_name()
        .expect("[ERR] - Input file has no filename");

    let filename = filename_os.to_string_lossy();

    if filename != "notebook-content.py"
    {
        let output_file_name = input_path
            .file_stem()
            .unwrap_or(filename_os)
            .to_string_lossy()
            .to_string();

        let output_file_name_ext = filename.to_string();

        return (output_file_name, output_file_name_ext);
    }

    // Special case: notebook-content.py
    let parent_dir_name = input_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|os| os.to_string_lossy().to_string())
        .unwrap_or_else(|| "notebook-content".to_string());

    let output_file_name = parent_dir_name
        .trim_end_matches(".Notebook")
        .to_string();

    let output_file_name_ext = format!("{}.py", output_file_name);

    (output_file_name, output_file_name_ext)
}


// ----------------------------
// Runtime
// ----------------------------

fn main()
{
    //! Application entry point for generating notebook documentation.
    //!
    //! # Inputs
    //! - CLI args: `doxcer [selector] <path/to/notebook.py>`.
    //! - Environment variables from the split env files.
    //!
    //! # Side Effects
    //! - Calls the Azure AI Foundry chat endpoint.
    //! - Prints generated Markdown to stdout.
    //! - Writes output Markdown to `docs/newly-documented`.
    //!
    //! # Process Exit
    //! - Exits with status code `1` for invalid CLI arguments.
    //!
    //! # Panics
    //! - On unrecoverable runtime or configuration errors.

    // CLI args
    let args: Vec<String> = env::args().collect();
    if is_help_requested(&args)
    {
        print_usage();
        process::exit(0);
    }

    let cli_args = match parse_cli_args(&args)
    {
        Ok(parsed) => parsed,
        Err(err) =>
        {
            eprintln!("{}", err);
            print_usage();
            process::exit(1);
        }
    };

    let file_path = &cli_args.file_path;
    println!(
        "[INF] - Processing notebook '{}' with '{}' profile.",
        file_path,
        profile_selector_name(cli_args.profile)
    );

    // Validate AI & Key Vault config
    if !ENVCONFIG.ai_enabled == true
        || ENVCONFIG.ai_base_url.trim().is_empty()
        || ENVCONFIG.ai_version.trim().is_empty()
        || ENVCONFIG.ai_task.trim().is_empty()
        || ENVCONFIG.ai_model.trim().is_empty()
    {
        eprintln!("[ERR] - AI Model configuration missing in env files");
        return;
    }

    if !ENVCONFIG.akv_enabled == true
        || ENVCONFIG.akv_base_url.trim().is_empty()
        || ENVCONFIG.akv_secret_ai.trim().is_empty()
    {
        eprintln!("[ERR] - Azure Key Vault configuration missing in env files");
        return;
    }

    // Determine notebook output names
    let input_path = Path::new(file_path);
    let (output_file_name, output_file_name_ext) = determine_output_names(input_path);

    // Fetch notebook content & clean
    println!("[INF] - Loading notebook from {}", file_path);
    let notebook_load_started_at = Instant::now();
    let notebook_content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("[ERR] - Failed to read file {}", file_path));
    let cleaned_notebook = collapse_blank_lines(&strip_notebook_metadata(&notebook_content));
    println!(
        "[SUC] - Notebook loaded and cleaned in {:.1}s.",
        notebook_load_started_at.elapsed().as_secs_f64()
    );

    // Load prompt & context templates
    println!("[INF] - Resolving prompt and context templates");
    let prompt_path = find_prompt_path(&cli_args.profile);
    println!("[INF] - Using prompt template {}", prompt_path.display());
    let prompt_content = fs::read_to_string(&prompt_path)
        .unwrap_or_else(|_| panic!("[ERR] - Failed to read prompt template {}", prompt_path.display()));
    let context_content = fs::read_to_string(find_context_path())
        .expect("[ERR] - Failed to read context template");
    println!("[SUC] - Prompt and context templates loaded");

    // Determine definitions
    let fabric_definitions = if ENVCONFIG.definition_database_enabled == true
    {
        println!("[INF] - Definition table enabled");

        if ENVCONFIG.definition_fabric_database_enabled == true
            && !ENVCONFIG.akv_secret_definition_fabric_endpoint.trim().is_empty()
            && !ENVCONFIG.akv_secret_definition_fabric_client_id.trim().is_empty()
            && !ENVCONFIG.akv_secret_definition_fabric_password.trim().is_empty()
            && !ENVCONFIG.definition_fabric_database.trim().is_empty()
        {
            println!("[SUC] - Fabric Definition DB config found");

            let repo_root = find_repo_root_path();
            let fabric_definition_config = FabricDefinitionConfig
            {
                repo_root: repo_root.as_path(),
                akv_base_url: &ENVCONFIG.akv_base_url,
                definition_fabric_database: &ENVCONFIG.definition_fabric_database,
                akv_secret_definition_fabric_endpoint: &ENVCONFIG.akv_secret_definition_fabric_endpoint,
                akv_secret_definition_fabric_client_id: &ENVCONFIG.akv_secret_definition_fabric_client_id,
                akv_secret_definition_fabric_password: &ENVCONFIG.akv_secret_definition_fabric_password,
                odbc_batch_size: ENVCONFIG.odbc_batch_size,
                odbc_max_byte_size: ENVCONFIG.odbc_max_byte_size,
            };

            // Fetch from Fabric SQL
            println!(
                "[INF] - Querying Fabric definitions for notebook '{}'",
                output_file_name
            );
            let definition_query_started_at = Instant::now();
            match fetch_definitions::fetch_definitions_from_fabric(
                &output_file_name,
                &fabric_definition_config,
            )
            {
                Ok((cols, rows)) if !cols.is_empty() && !rows.is_empty() =>
                {
                    println!(
                        "[SUC] - Definitions query completed in {:.1}s.",
                        definition_query_started_at.elapsed().as_secs_f64()
                    );
                    println!("[SUC] - Definitions found: {} row(s).", rows.len());
                    fetch_definitions::format_definitions_as_markdown_table(&cols, &rows)
                }
                Ok(_) =>
                {
                    println!(
                        "[INF] - Definitions query completed in {:.1}s.",
                        definition_query_started_at.elapsed().as_secs_f64()
                    );
                    println!("[INF] - No definitions found for this notebook.");
                    "[INF] - No definitions loaded (query returned no rows).".to_string()
                }
                Err(e) =>
                {
                    eprintln!(
                        "[WRN] - Definitions query failed after {:.1}s: {e}",
                        definition_query_started_at.elapsed().as_secs_f64()
                    );
                    "[INF] - No definitions loaded (query failed).".to_string()
                }
            }
        }
        else if ENVCONFIG.definition_azure_database_enabled == true
        {
            println!("[SUC] - Azure Definition DB config found");
            "[INF] - Azure SQL definitions not implemented yet.".to_string()
        }
        else
        {
            println!("[ERR] - No supported definition DB configured");
            return;
        }
    }
    else
    {
        println!("[INF] - Definition database disabled");
        "[INF] - Definition database disabled.".to_string()
    };

    // Build prompt
    println!("[INF] - Building prompt payload");
    let current_datetime = Utc::now().with_timezone(&Amsterdam)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let prompt = format!(
        "Current date time: {}\n\nNotebook filename: {}\n\nDefinitions: {}\n\nDocumentation template: {}\n\nCode: {}",
        current_datetime,
        output_file_name_ext,
        fabric_definitions,
        prompt_content,
        cleaned_notebook
    );

    // Call API
    println!("[INF] - Resolving API key from Azure Key Vault");
    let key_vault_lookup_started_at = Instant::now();
    let api_key = fetch_secrets::get_secret_from_key_vault(&ENVCONFIG.akv_base_url, &ENVCONFIG.akv_secret_ai);
    println!(
        "[SUC] - API key resolved in {:.1}s.",
        key_vault_lookup_started_at.elapsed().as_secs_f64()
    );
    let api_url = format!(
        "{base}/models/chat/{task}?api-version={version}",
        base = ENVCONFIG.ai_base_url,
        task = ENVCONFIG.ai_task,
        version = ENVCONFIG.ai_version
    );

    let request = ChatRequest
    {
        model: ENVCONFIG.ai_model.clone(),
        messages: vec![
            Message { role: "system".to_string(), content: context_content },
            Message { role: "user".to_string(), content: prompt },
        ],
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .expect("Failed to build HTTP client");

    // Handle response
    println!(
        "[INF] - Submitting API request for task '{}' (model '{}')",
        ENVCONFIG.ai_task,
        ENVCONFIG.ai_model
    );
    println!("[INF] - Waiting for API response (timeout: 300s)");
    let api_request_started_at = Instant::now();
    match client.post(&api_url)
        .header("Content-Type", "application/json")
        .header("api-key", api_key)
        .json(&request)
        .send()
    {
        Ok(res) if res.status().is_success() =>
        {
            println!(
                "[SUC] - API request completed in {:.1}s with status {}.",
                api_request_started_at.elapsed().as_secs_f64(),
                res.status()
            );
            let body_text = res.text().unwrap_or_default();
            println!("[INF] - Parsing API response");
            match serde_json::from_str::<ChatResponse>(&body_text)
            {
                Ok(chat_response) =>
                {
                    println!(
                        "[SUC] - API response parsed ({} choice(s)).",
                        chat_response.choices.len()
                    );
                    if let Some(first_choice) = chat_response
                        .choices
                        .iter()
                        .find(|choice| is_assistant_role(choice.message.role.as_deref()))
                    {
                        let content = &first_choice.message.content;
                        if content.trim().is_empty()
                        {
                            println!("[INF] - API response was empty.");
                            return;
                        }

                        if looks_like_internal_prompt(content)
                        {
                            eprintln!("[WRN] - API response appears to echo the internal prompt. Output suppressed.");
                            return;
                        }

                        // Save to wiki
                        let mut output_path = find_docs_path();
                        output_path.push(format!("{}.md", output_file_name));

                        if let Some(parent) = output_path.parent()
                        {
                            if let Err(e) = fs::create_dir_all(parent)
                            {
                                eprintln!("[WRN] - Failed to create wiki directory {}: {}", parent.display(), e);
                            }
                        }

                        if let Err(e) = fs::write(&output_path, content)
                        {
                            eprintln!("[WRN] - Failed to save documentation to {}: {}", output_path.display(), e);
                        }
                        else
                        {
                            println!("[SUC] - Saved documentation to: {}", output_path.display());
                        }
                    }
                    else
                    {
                        println!("[INF] - No assistant 'choices' found in response.");
                    }
                }
                Err(e) =>
                {
                    if env_flag_enabled("DOXCER_DEBUG_API")
                    {
                        eprintln!("[ERR] - Failed to deserialize response: {e}\n[INF] - Raw response: {body_text}");
                    }
                    else
                    {
                        eprintln!("[ERR] - Failed to deserialize response: {e}");
                        eprintln!("[INF] - Set DOXCER_DEBUG_API=true to print the raw API response body.");
                    }
                }
            }
        }
        Ok(res) =>
        {
            let status = res.status();
            let body_text = res.text().unwrap_or_default();
            let elapsed = api_request_started_at.elapsed().as_secs_f64();

            if env_flag_enabled("DOXCER_DEBUG_API")
            {
                eprintln!("[ERR] - API request failed after {elapsed:.1}s ({status}): {body_text}");
            }
            else
            {
                eprintln!("[ERR] - API request failed after {elapsed:.1}s with status: {status}");
                eprintln!("[INF] - Set DOXCER_DEBUG_API=true to print the raw API error body.");
            }
        }
        Err(e) => eprintln!(
            "[ERR] - Request error after {:.1}s: {}",
            api_request_started_at.elapsed().as_secs_f64(),
            e
        ),
    }
}
