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
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

// External Libraries
use anyhow::{Context, Result};
use crate::fetch_secrets::get_secret_from_key_vault;
use odbc_api::{buffers::TextRowSet, ConnectionOptions, Cursor, Environment, ResultSetMetadata};


// ----------------------------
// Data Structures
// ----------------------------

struct DefinitionFabricDbCredentials
{
    /// Type: Struct.
    /// Input:
    /// - Secret values resolved from Azure Key Vault.
    /// Output:
    /// - In-memory Fabric SQL credentials bundle.
    /// Exceptions:
    /// - None.

    fabric_sql_endpoint: String,
    fabric_service_principal_client_id: String,
    fabric_service_principal_password: String,
}

struct DefinitionAzureDbCredentials
{
    /// Type: Struct.
    /// Input:
    /// - Secret values resolved from Azure Key Vault.
    /// Output:
    /// - In-memory Azure SQL credentials bundle.
    /// Exceptions:
    /// - None.

    azure_sql_endpoint: String,
    azure_service_principal_client_id: String,
    azure_service_principal_password: String,
}

pub struct FabricDefinitionConfig<'a>
{
    /// Type: Struct.
    /// Input:
    /// - Values provided by runtime configuration in `main.rs` for Fabric SQL.
    /// Output:
    /// - Settings required for definition lookup and formatting.
    /// Exceptions:
    /// - None.

    pub repo_root: &'a Path,
    pub akv_base_url: &'a str,
    pub definition_fabric_database: &'a str,
    pub akv_secret_definition_fabric_endpoint: &'a str,
    pub akv_secret_definition_fabric_client_id: &'a str,
    pub akv_secret_definition_fabric_password: &'a str,
    pub odbc_batch_size: usize,
    pub odbc_max_byte_size: usize,
}

pub struct AzureDefinitionConfig<'a>
{
    /// Type: Struct.
    /// Input:
    /// - Values provided by runtime configuration in `main.rs` for Azure SQL.
    /// Output:
    /// - Settings required for Azure definition lookup and formatting.
    /// Exceptions:
    /// - None.

    pub repo_root: &'a Path,
    pub akv_base_url: &'a str,
    pub definition_azure_database: &'a str,
    pub akv_secret_definition_azure_endpoint: &'a str,
    pub akv_secret_definition_azure_client_id: &'a str,
    pub akv_secret_definition_azure_password: &'a str,
    pub odbc_batch_size: usize,
    pub odbc_max_byte_size: usize,
}


// ----------------------------
// Fabric SQL Helper Functions
// ----------------------------

fn find_fabric_sql_path(repo_root: &Path) -> PathBuf
{
    /// Type: Function.
    /// Input:
    /// - `repo_root`: Repository root path.
    /// Output:
    /// - `PathBuf`: `sql/fetch_fabric_definitions.sql`.
    /// Exceptions:
    /// - None.

    repo_root.join("sql").join("fetch_fabric_definitions.sql")
}

fn get_fabric_definition_db_credentials(config: &FabricDefinitionConfig) -> DefinitionFabricDbCredentials
{
    /// Type: Function.
    /// Input:
    /// - `config`: Fabric definition module runtime settings.
    /// Output:
    /// - `DefinitionFabricDbCredentials`: Fabric SQL endpoint/client/password.
    /// Exceptions:
    /// - Panics if required secrets are missing or empty.

    let fabric_sql_endpoint = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_fabric_endpoint,
    );
    let fabric_service_principal_client_id = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_fabric_client_id,
    );
    let fabric_service_principal_password = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_fabric_password,
    );

    if fabric_sql_endpoint.trim().is_empty()
    {
        panic!("[INF] - Fabric Definition DB endpoint secret was empty.");
    }
    if fabric_service_principal_client_id.trim().is_empty()
    {
        panic!("[INF] - Fabric Definition DB client id secret was empty.");
    }
    if fabric_service_principal_password.trim().is_empty()
    {
        panic!("[INF] - Fabric Definition DB password secret was empty.");
    }

    DefinitionFabricDbCredentials
    {
        fabric_sql_endpoint: fabric_sql_endpoint.trim().to_string(),
        fabric_service_principal_client_id: fabric_service_principal_client_id.trim().to_string(),
        fabric_service_principal_password: fabric_service_principal_password.trim().to_string(),
    }
}

pub fn fetch_definitions_from_fabric(
    table_prefix: &str,
    config: &FabricDefinitionConfig,
) -> Result<(Vec<String>, Vec<Vec<String>>)>
{
    /// Type: Function.
    /// Input:
    /// - `table_prefix`: Prefix used for SQL `LIKE` filtering.
    /// - `config`: Fabric definition module runtime settings.
    /// Output:
    /// - `Result<(Vec<String>, Vec<Vec<String>>)>`: Column names and rows as text.
    /// Exceptions:
    /// - Returns `Err(...)` for ODBC/connect/query/read failures.
    /// - Panics if the `LIKE` pattern contains an interior null byte.

    let fabric_definition_db_credentials = get_fabric_definition_db_credentials(config);
    let fabric_odbc_environment = Environment::new().context("[ERR] - Failed to create ODBC environment")?;

    let fabric_conn_str = format!(
        "Driver={{ODBC Driver 18 for SQL Server}};\
        Server=tcp:{host},1433;\
        Database={db};\
        Encrypt=yes;\
        TrustServerCertificate=yes;\
        Authentication=ActiveDirectoryServicePrincipal;\
        UID={uid};\
        PWD={pwd};",
        host = fabric_definition_db_credentials.fabric_sql_endpoint.trim(),
        db = config.definition_fabric_database,
        uid = fabric_definition_db_credentials.fabric_service_principal_client_id.trim(),
        pwd = fabric_definition_db_credentials.fabric_service_principal_password.trim()
    );

    let fabric_odbc_connection = fabric_odbc_environment
        .connect_with_connection_string(&fabric_conn_str, ConnectionOptions::default())
        .context("[ERR] - ODBC connect failed")?;

    let fabric_sql_query = fs::read_to_string(find_fabric_sql_path(config.repo_root))
        .context("[ERR] - Failed to read SQL file for definitions")?;

    let fabric_table_like_pattern = format!("{}%", table_prefix);
    let fabric_table_like_pattern_c = CString::new(fabric_table_like_pattern)
        .expect("[ERR] - LIKE pattern contained an interior null byte");

    let fabric_maybe_cursor = fabric_odbc_connection
        .execute(&fabric_sql_query, &fabric_table_like_pattern_c, None)
        .context("[ERR] - Query execution failed")?;

    let mut fabric_cursor = match fabric_maybe_cursor
    {
        Some(c) => c,
        None => return Ok((Vec::new(), Vec::new())),
    };

    let fabric_column_names: Vec<String> = fabric_cursor
        .column_names()
        .context("[ERR] - Failed to read column names")?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let mut fabric_text_row_set = TextRowSet::for_cursor(
        config.odbc_batch_size,
        &mut fabric_cursor,
        Some(config.odbc_max_byte_size),
    )?;
    let mut fabric_row_set_cursor = fabric_cursor.bind_buffer(&mut fabric_text_row_set)?;

    let mut fabric_all_rows: Vec<Vec<String>> = Vec::new();
    while let Some(batch) = fabric_row_set_cursor.fetch()?
    {
        for row_index in 0..batch.num_rows()
        {
            let mut fields = Vec::with_capacity(batch.num_cols());
            for col_index in 0..batch.num_cols()
            {
                let bytes = batch.at(col_index, row_index).unwrap_or(&[]);
                fields.push(String::from_utf8_lossy(bytes).to_string());
            }
            fabric_all_rows.push(fields);
        }
    }

    Ok((fabric_column_names, fabric_all_rows))
}


// ----------------------------
// Azure SQL Helper Functions
// ----------------------------

fn find_azure_sql_path(repo_root: &Path) -> PathBuf
{
    /// Type: Function.
    /// Input:
    /// - `repo_root`: Repository root path.
    /// Output:
    /// - `PathBuf`: `sql/fetch_azure_definitions.sql`.
    /// Exceptions:
    /// - None.

    // TODO: Confirm the Azure SQL query file name and path

    repo_root.join("sql").join("fetch_azure_definitions.sql")
}

fn get_azure_definition_db_credentials(config: &AzureDefinitionConfig) -> DefinitionAzureDbCredentials
{
    /// Type: Function.
    /// Input:
    /// - `config`: Azure definition module runtime settings.
    /// Output:
    /// - `DefinitionAzureDbCredentials`: Azure SQL endpoint/client/password.
    /// Exceptions:
    /// - Panics if required secrets are missing or empty.

    // TODO: Implement Azure SQL credentials retrieval for definitions

    let azure_sql_endpoint = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_azure_endpoint,
    );
    let azure_service_principal_client_id = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_azure_client_id,
    );
    let azure_service_principal_password = get_secret_from_key_vault(
        config.akv_base_url,
        config.akv_secret_definition_azure_password,
    );

    if azure_sql_endpoint.trim().is_empty()
    {
        panic!("[INF] - Azure Definition DB endpoint secret was empty.");
    }
    if azure_service_principal_client_id.trim().is_empty()
    {
        panic!("[INF] - Azure Definition DB client id secret was empty.");
    }
    if azure_service_principal_password.trim().is_empty()
    {
        panic!("[INF] - Azure Definition DB password secret was empty.");
    }

    DefinitionAzureDbCredentials
    {
        azure_sql_endpoint: azure_sql_endpoint.trim().to_string(),
        azure_service_principal_client_id: azure_service_principal_client_id.trim().to_string(),
        azure_service_principal_password: azure_service_principal_password.trim().to_string(),
    }
}

pub fn fetch_definitions_from_azure(
    _table_prefix: &str,
    _config: &AzureDefinitionConfig,
) -> Result<(Vec<String>, Vec<Vec<String>>)>
{
    /// Type: Function.
    /// Input:
    /// - `table_prefix`: Prefix used for SQL `LIKE` filtering.
    /// - `config`: Azure definition module runtime settings.
    /// Output:
    /// - `Result<(Vec<String>, Vec<Vec<String>>)>`: Column names and rows as text.

    // TODO: Implement Azure SQL fetch for a definitions table.
    Err(anyhow::anyhow!(
        "[ERR] - Azure SQL definitions fetch is not implemented yet."
    ))
}

pub fn format_definitions_as_markdown_table(col_names: &[String], rows: &[Vec<String>]) -> String
{
    /// Type: Function.
    /// Input:
    /// - `col_names`: Column names used as Markdown headers.
    /// - `rows`: Definition rows.
    /// Output:
    /// - `String`: Markdown table text.
    /// - Returns `[INF] - No definition rows returned.` when `col_names` is empty.
    /// Exceptions:
    /// - None expected under normal execution.

    if col_names.is_empty()
    {
        return "[INF] - No definition rows returned.".to_string();
    }

    fn esc(s: &str) -> String
    {
        s.replace('|', r"\|").replace('\n', " ").replace('\r', " ")
    }

    let mut out = String::new();

    out.push('|');
    for c in col_names
    {
        out.push(' ');
        out.push_str(&esc(c));
        out.push(' ');
        out.push('|');
    }
    out.push('\n');

    out.push('|');
    for _ in col_names
    {
        out.push_str(" --- |");
    }
    out.push('\n');

    for r in rows
    {
        out.push('|');
        for i in 0..col_names.len()
        {
            let v = r.get(i).map(|s| s.as_str()).unwrap_or("");
            out.push(' ');
            out.push_str(&esc(v));
            out.push(' ');
            out.push('|');
        }
        out.push('\n');
    }

    out
}
