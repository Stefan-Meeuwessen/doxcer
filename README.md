# Doxcer

Doxcer is a Rust CLI tool that generates Markdown documentation from Notebook files.

It:
- Parses and cleans notebook source
- Builds an LLM prompt from templates
- Optionally enriches documentation with definitions from a SQL definitions table
- Writes output to `docs/newly-documented/`


## Current Status
- Fabric definitions fetch is implemented.
- Azure definitions fetch is scaffolded but not implemented yet.
- Unit tests are centralized in `src/unit_tests.rs`.


## CLI Usage
```bash
(default)   doxcer <path/to/notebook.py>
(optional)  doxcer [-selector] <path/to/notebook.py>
(help)      doxcer --help
```

Notes:
- Path and selector can be passed in any order.
- Current implemented selectors are; ``-fabric``, ``-synapse``, ``-databricks``, ``-datafactory``, ``-aws`` and ``-powerbi``.


## Runtime Flow
At runtime, Doxcer:
1. Parses CLI args and selects prompt profile.
2. Loads env config from:
   - `config/system.env`
   - `config/definitions.env`
   - `config/azure_key_vault.env`
   - `config/ai_model.env`
3. Reads notebook file and removes metadata lines:
   - `# METADATA`
   - `# META`
   - `# CELL`
4. Loads prompt template from `templates/*_prompt.md` and context from `templates/context.md`.
5. Optionally fetches definitions from Fabric SQL (via ODBC).
6. Calls the configured AI endpoint.
7. Writes markdown to `docs/newly-documented/<name>.md`.


## Configuration
Configuration is read from the env files under `config/` (see `config/examples/`).

### `config/system.env`
- `ABSOLUTE_DOXCER_PATH`

### `config/ai_model.env`
- `AI_ENABLED`
- `AI_BASE_URL`
- `AI_MODEL`
- `AI_VERSION`
- `AI_TASK`

### `config/azure_key_vault.env`
- `AKV_ENABLED`
- `AKV_BASE_URL`
- `AKV_SECRET_AI`

### `config/definitions.env`
- `DEFINITION_DATABASE_ENABLED`
- `ODBC_BATCH_SIZE`
- `ODBC_MAX_BYTE_SIZE`

Fabric section:
- `DEFINITION_FABRIC_DATABASE_ENABLED`
- `DEFINITION_FABRIC_DATABASE`
- `AKV_SECRET_DEFINITION_FABRIC_ENDPOINT`
- `AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_CLIENT`
- `AKV_SECRET_DEFINITION_FABRIC_SERVICE_PRINCIPAL_PASSWORD`

Azure section:
- `DEFINITION_AZURE_DATABASE_ENABLED`
- `DEFINITION_AZURE_DATABASE`
- `AKV_SECRET_DEFINITION_AZURE_ENDPOINT`
- `AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_CLIENT`
- `AKV_SECRET_DEFINITION_AZURE_SERVICE_PRINCIPAL_PASSWORD`


## Prerequisites Scripts
First use:
```Bash
./build.sh
```

Then use:
```Powershell
.\set-up-doxcer.ps1
```

The Bash script:
- Builds a release version of Doxcer
- Copies this version of Doxcer into Dist/ so that the Powershell script can work some magic

The Powershell script:
- ensures ODBC Driver 18 and Azure CLI are available
- requires admin rights only when installing ODBC/Azure CLI
- checks Azure CLI login
- selects newest dist release by folder name:
  - `doxcer-windows-x64-yyyy-MM-dd_HH-mm-ss`
- generates `config/system.env` with `ABSOLUTE_DOXCER_PATH=<repo-root>`
- copies `doxcer.exe` into `%LOCALAPPDATA%\doxcer\bin\doxcer.exe`
- sets `ABSOLUTE_DOXCER_PATH` as a user environment variable
- in admin mode, normalizes user PATH to use `%LOCALAPPDATA%\doxcer\bin`


## Build and Distribution
Build script:
```bash
./build.sh
```

It runs:
- `cargo clean`
- `cargo check`
- `cargo build`
- `cargo build --tests`
- `cargo test`
- `cargo build --release`

Then it packages:
- `target/release/doxcer.exe`
- into `dist/doxcer-windows-x64-<timestamp>/`


## SQL Scripts
Definition DDL and fetch queries are split by provider:
- `sql/create_fabric_definitiions.sql`
  - Fabric-compatible table creation
  - includes stored procedures for timestamp behavior due Fabric constraints
- `sql/create_azure_definitions.sql`
  - Azure SQL table creation with identity/default/trigger-based timestamp behavior
- `sql/fetch_fabric_definitions.sql`
  - parameterized fetch query for Fabric
- `sql/fetch_azure_definitions.sql`
  - parameterized fetch query for Azure


## Testing
Run:
```bash
cargo test
```

Unit tests are maintained in:
- `src/unit_tests.rs`


## Project Structure
```text
doxcer/
├── Cargo.toml
├── README.md
├── build.sh
├── set-up-doxcer.ps1
├── config/
│   ├── system.env
│   ├── ai_model.env
│   ├── azure_key_vault.env
│   ├── definitions.env
│   └── examples/
├── sql/
│   ├── create_fabric_definitiions.sql
│   ├── create_azure_definitions.sql
│   ├── fetch_fabric_definitions.sql
│   └── fetch_azure_definitions.sql
├── src/
│   ├── main.rs
│   ├── fetch_definitions.rs
│   ├── fetch_secrets.rs
│   └── unit_tests.rs
├── templates/
│   ├── default_prompt.md
│   ├── fabric_prompt.md
│   ├── synapse_prompt.md
│   ├── databricks_prompt.md
│   ├── powerbi_prompt.md
│   ├── aws_prompt.md
│   ├── datafactory_prompt.md
│   └── context.md
└── docs/
    └── newly-documented/
```


## Version
- Cargo package version: `0.1.4`
- Project version: 3.0.0
