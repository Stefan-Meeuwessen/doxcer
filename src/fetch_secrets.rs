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

// External Libraries
use azure_identity::DeveloperToolsCredential;
use azure_security_keyvault_secrets::{SecretClient, SecretClientOptions};


// ----------------------------
// Data Structures
// ----------------------------

pub fn get_secret_from_key_vault(vault_url: &str, secret_name: &str) -> String
{
    /// Type: Function.
    /// Input:
    /// - `vault_url`: Azure Key Vault base URL.
    /// - `secret_name`: Secret name to retrieve.
    /// Output:
    /// - `String`: Trimmed secret value.
    /// Exceptions:
    /// - Panics if runtime creation, client creation, or secret retrieval fails.

    let rt = tokio::runtime::Runtime::new()
        .expect("[ERR] - Failed to create Tokio runtime");

    rt.block_on(
        async
        {
            let credential = DeveloperToolsCredential::new(None)
                .expect("[ERR] - Failed to create DeveloperToolsCredential");
            let client = SecretClient::new(
                vault_url,
                credential.clone(),
                None::<SecretClientOptions>
            ).expect("[ERR] - Failed to create SecretClient");
            
            let secret = client
                .get_secret(secret_name, None)
                .await
                .expect("[ERR] - Failed to fetch secret")
                .into_model()
                .expect("[ERR] - Failed to deserialize secret model");

            secret.value.expect("[WRN] - Secret has no value").trim().to_string()
        }
    )
}
