//////////////////////////////////////////////////////////
// AUTHOR   : Stefan B. J. Meeuwessen
// CREATION : 2026-02-11
// VERSION  : 0.0.1
//////////////////////////////////////////////////////////


// ----------------------------
// Compiler Directives
// ----------------------------

// #![allow(unused)]
#![allow(unused_doc_comments)]


// ----------------------------
// Imports
// ----------------------------

// Standard Libraries
use super::*;
use std::path::Path;


// ----------------------------
// Shared Test Helpers
// ----------------------------

fn make_args(raw: &[&str]) -> Vec<String>
{
    /// Type: Helper function.
    /// Input:
    /// - `raw`: Slice of CLI argument tokens.
    /// Output:
    /// - `Vec<String>` for `parse_cli_args`.
    /// Exceptions:
    /// - None.

    raw.iter().map(|v| v.to_string()).collect()
}


// ----------------------------
// main.rs
// ----------------------------

#[test]
fn parse_no_flag_selects_default_prompt()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when no selector flag resolves to `PromptProfile::Default`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Default);
}

#[test]
fn parse_fabric_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-fabric` resolves to `PromptProfile::Fabric`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-fabric", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Fabric);
}

#[test]
fn parse_synapse_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-synapse` resolves to `PromptProfile::Synapse`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-synapse", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Synapse);
}

#[test]
fn parse_databricks_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-databricks` resolves to `PromptProfile::Databricks`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-databricks", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Databricks);
}

#[test]
fn parse_powerbi_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-powerbi` resolves to `PromptProfile::PowerBi`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-powerbi", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::PowerBi);
}

#[test]
fn parse_aws_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-aws` resolves to `PromptProfile::Aws`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-aws", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Aws);
}

#[test]
fn parse_datafactory_flag()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when `-datafactory` resolves to `PromptProfile::DataFactory`.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "-datafactory", "test/example.py"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::DataFactory);
}

#[test]
fn parse_accepts_any_argument_order()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when path and selector order are both accepted.
    /// Exceptions:
    /// - Panics if assertions fail.

    let parsed = parse_cli_args(&make_args(&["doxcer", "test/example.py", "-fabric"])).unwrap();
    assert_eq!(parsed.file_path, "test/example.py");
    assert_eq!(parsed.profile, PromptProfile::Fabric);
}

#[test]
fn parse_conflicting_selectors_fail()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when conflicting selectors return an error.
    /// Exceptions:
    /// - Panics if assertions fail.

    let err = parse_cli_args(&make_args(&["doxcer", "-fabric", "test/example.py", "-synapse"]))
        .unwrap_err();
    assert!(err.contains("Conflicting selectors"));
}

#[test]
fn parse_unknown_selector_fails()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when unknown selector flags return an error.
    /// Exceptions:
    /// - Panics if assertions fail.

    let err = parse_cli_args(&make_args(&["doxcer", "-unknown", "test/example.py"]))
        .unwrap_err();
    assert!(err.contains("Unknown selector"));
}

#[test]
fn parse_double_dash_selector_fails()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when GNU-style profile selectors remain unsupported.
    /// Exceptions:
    /// - Panics if assertions fail.

    let err = parse_cli_args(&make_args(&["doxcer", "--fabric", "test/example.py"]))
        .unwrap_err();
    assert!(err.contains("Unknown selector"));
}

#[test]
fn supported_selector_list_uses_canonical_only()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when selector list uses canonical selector names only.
    /// Exceptions:
    /// - Panics if assertions fail.

    let supported = supported_selector_list();
    assert_eq!(
        supported,
        "-fabric, -synapse, -databricks, -powerbi, -aws, -datafactory"
    );
}

#[test]
fn parse_missing_path_fails()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when missing path input returns an error.
    /// Exceptions:
    /// - Panics if assertions fail.

    let err = parse_cli_args(&make_args(&["doxcer", "-fabric"])).unwrap_err();
    assert!(err.contains("Missing required notebook path argument"));
}

#[test]
fn parse_multiple_paths_fail()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when multiple path arguments return an error.
    /// Exceptions:
    /// - Panics if assertions fail.

    let err = parse_cli_args(&make_args(&["doxcer", "test/a.py", "test/b.py"])).unwrap_err();
    assert!(err.contains("Multiple input paths"));
}

#[test]
fn parse_profile_selector_accepts_known_values()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when known selector values resolve to expected profiles.
    /// Exceptions:
    /// - Panics if assertions fail.

    assert_eq!(parse_profile_selector("-fabric"), Some(PromptProfile::Fabric));
    assert_eq!(parse_profile_selector("-datafactory"), Some(PromptProfile::DataFactory));
}

#[test]
fn parse_profile_selector_returns_none_for_unknown_value()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when unknown selector values return `None`.
    /// Exceptions:
    /// - Panics if assertions fail.

    assert_eq!(parse_profile_selector("-not-a-selector"), None);
}

#[test]
fn profile_selector_name_maps_to_canonical_name()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when profile names map to canonical selector names.
    /// Exceptions:
    /// - Panics if assertions fail.

    assert_eq!(profile_selector_name(PromptProfile::Default), "default");
    assert_eq!(profile_selector_name(PromptProfile::Synapse), "synapse");
    assert_eq!(profile_selector_name(PromptProfile::DataFactory), "datafactory");
}

#[test]
fn prompt_profile_spec_returns_expected_datafactory_metadata()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when DataFactory profile metadata is correct.
    /// Exceptions:
    /// - Panics if assertions fail.

    let spec = prompt_profile_spec(PromptProfile::DataFactory);
    assert_eq!(spec.template_stem, "datafactory");
    assert!(spec.selector_flags.contains(&"-datafactory"));
}

#[test]
fn is_metadata_line_detects_supported_prefixes()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when metadata prefixes are detected and non-metadata lines are ignored.
    /// Exceptions:
    /// - Panics if assertions fail.

    assert!(is_metadata_line("# METADATA x"));
    assert!(is_metadata_line("  # META y"));
    assert!(is_metadata_line("\t# CELL 2"));
    assert!(!is_metadata_line("print('hello')"));
    assert!(!is_metadata_line("#METADATA"));
}

#[test]
fn strip_notebook_metadata_removes_only_metadata_lines()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when metadata lines are removed and normal lines remain.
    /// Exceptions:
    /// - Panics if assertions fail.

    let source = "# METADATA a\nprint('x')\n# META b\n# CELL c\nprint('y')";
    let cleaned = strip_notebook_metadata(source);
    assert_eq!(cleaned, "print('x')\nprint('y')");
}

#[test]
fn collapse_blank_lines_reduces_consecutive_blank_runs()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when multiple consecutive blank lines collapse to one.
    /// Exceptions:
    /// - Panics if assertions fail.

    let source = "line1\n\n\nline2\n   \n\t\nline3";
    let collapsed = collapse_blank_lines(source);
    assert_eq!(collapsed, "line1\n\nline2\n   \nline3");
}

#[test]
fn determine_output_names_for_standard_file()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when standard files resolve to stem + original filename.
    /// Exceptions:
    /// - Panics if assertions fail.

    let (name, ext_name) = determine_output_names(Path::new("x/y/pipeline.py"));
    assert_eq!(name, "pipeline");
    assert_eq!(ext_name, "pipeline.py");
}

#[test]
fn determine_output_names_for_notebook_content_uses_parent_folder_name()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when notebook-content.py uses parent folder name.
    /// Exceptions:
    /// - Panics if assertions fail.

    let (name, ext_name) = determine_output_names(Path::new("Sales.Notebook/notebook-content.py"));
    assert_eq!(name, "Sales");
    assert_eq!(ext_name, "Sales.py");
}

#[test]
fn determine_output_names_for_root_notebook_content_uses_fallback()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when root notebook-content.py uses fallback output names.
    /// Exceptions:
    /// - Panics if assertions fail.

    let (name, ext_name) = determine_output_names(Path::new("notebook-content.py"));
    assert_eq!(name, "notebook-content");
    assert_eq!(ext_name, "notebook-content.py");
}

#[test]
fn find_repo_root_path_contains_project_markers()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when resolved repo root contains expected marker files.
    /// Exceptions:
    /// - Panics if assertions fail.

    let root = find_repo_root_path();
    assert!(has_repo_markers(&root));
}

#[test]
fn find_env_paths_returns_expected_files_in_order()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when env paths are returned in the expected order.
    /// Exceptions:
    /// - Panics if assertions fail.

    let env_paths = find_env_paths();
    let names: Vec<String> = env_paths
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert_eq!(
        names,
        vec![
            "system.env".to_string(),
            "definitions.env".to_string(),
            "azure_key_vault.env".to_string(),
            "ai_model.env".to_string()
        ]
    );
}

#[test]
fn find_prompt_path_finds_existing_profile_prompt()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when a profile with a template resolves to that template.
    /// Exceptions:
    /// - Panics if assertions fail.

    let path = find_prompt_path(&PromptProfile::Fabric);
    assert_eq!(path.file_name().unwrap().to_string_lossy(), "fabric_prompt.md");
    assert!(path.exists());
}

#[test]
fn find_context_and_docs_paths_point_to_expected_locations()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when context and docs paths resolve to expected names.
    /// Exceptions:
    /// - Panics if assertions fail.

    let context_path = find_context_path();
    assert_eq!(context_path.file_name().unwrap().to_string_lossy(), "context.md");
    assert!(context_path.exists());

    let docs_path = find_docs_path();
    assert_eq!(docs_path.file_name().unwrap().to_string_lossy(), "newly-documented");
    assert_eq!(
        docs_path.parent().and_then(|p| p.file_name()).unwrap().to_string_lossy(),
        "docs"
    );
}


// ----------------------------
// fetch_definitions.rs
// ----------------------------

#[test]
fn definitions_markdown_table_returns_info_for_empty_columns()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when empty columns return the no-definitions info message.
    /// Exceptions:
    /// - Panics if assertions fail.

    let result = crate::fetch_definitions::format_definitions_as_markdown_table(&[], &[]);
    assert_eq!(result, "[INF] - No definition rows returned.");
}

#[test]
fn definitions_markdown_table_renders_header_and_separator()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when only header/separator are rendered for empty rows.
    /// Exceptions:
    /// - Panics if assertions fail.

    let columns = vec!["col_a".to_string(), "col_b".to_string()];
    let rows: Vec<Vec<String>> = Vec::new();

    let result = crate::fetch_definitions::format_definitions_as_markdown_table(&columns, &rows);
    let expected = concat!("| col_a | col_b |\n", "| --- | --- |\n");

    assert_eq!(result, expected);
}

#[test]
fn definitions_markdown_table_escapes_pipe_and_newline_characters()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when cell values are escaped/sanitized for Markdown.
    /// Exceptions:
    /// - Panics if assertions fail.

    let columns = vec!["name|raw".to_string(), "description".to_string()];
    let rows = vec![vec![
        "left|value".to_string(),
        "line1\nline2\rline3".to_string(),
    ]];

    let result = crate::fetch_definitions::format_definitions_as_markdown_table(&columns, &rows);
    let expected = concat!(
        "| name\\|raw | description |\n",
        "| --- | --- |\n",
        "| left\\|value | line1 line2 line3 |\n"
    );

    assert_eq!(result, expected);
}

#[test]
fn definitions_markdown_table_pads_missing_cells_and_ignores_extra_cells()
{
    /// Type: Unit test.
    /// Input:
    /// - None.
    /// Output:
    /// - Passes when short rows are padded and extra row columns are ignored.
    /// Exceptions:
    /// - Panics if assertions fail.

    let columns = vec!["a".to_string(), "b".to_string()];
    let rows = vec![
        vec!["only-a".to_string()],
        vec!["a2".to_string(), "b2".to_string(), "extra".to_string()],
    ];

    let result = crate::fetch_definitions::format_definitions_as_markdown_table(&columns, &rows);
    let expected = concat!(
        "| a | b |\n",
        "| --- | --- |\n",
        "| only-a |  |\n",
        "| a2 | b2 |\n"
    );

    assert_eq!(result, expected);
}


// ----------------------------
// fetch_secrets.rs
// ----------------------------

// No deterministic unit-test surface is currently exposed without adding seams or mocks.
