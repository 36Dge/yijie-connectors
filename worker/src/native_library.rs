//! Fixed Codex library boundary. No provider is qualified in this foundation.
//! The control router exposes only auth_status, never these future IO methods.
use codex_config::types::{AuthKeyringBackendKind, OAuthCredentialsStoreMode};
use codex_exec_server::ReqwestHttpClient;
use codex_rmcp_client::{OauthLoginHandle, RmcpClient};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::generated::WorkerLibraryPolicy;

pub fn policy() -> WorkerLibraryPolicy {
    // Compile against the actual fixed enum instead of duplicating a storage
    // mode. Status does not invoke the Keychain or load any credential.
    assert_eq!(
        serde_json::to_value(OAuthCredentialsStoreMode::Keyring).unwrap(),
        "keyring"
    );
    WorkerLibraryPolicy {
        mcp_library: "codex-rmcp-client".into(),
        oauth_store: "keyring_only".into(),
        credential_boundary: "connectors_only".into(),
        stdio_shutdown: "eof_only".into(),
        external_calls_enabled: false,
    }
}

// Opaque account aliases separate this worker's Keychain ownership from Codex
// and from another local scope/connection; no raw account identity is stored.
#[allow(dead_code)]
pub fn account_alias(scope: &str, connection: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"yijie-market-credential-v1\0");
    hasher.update((scope.len() as u64).to_be_bytes());
    hasher.update(scope.as_bytes());
    hasher.update((connection.len() as u64).to_be_bytes());
    hasher.update(connection.as_bytes());
    format!("yijie_market_{:x}", hasher.finalize())
}

// No constructor is provided until accepted provider evidence, caller scope,
// callback authority and execution capability contracts are implemented. Source
// reference URLs cannot become an ApprovedHttpTarget through deserialization.
#[allow(dead_code)]
struct ApprovedHttpTarget {
    alias: String,
    endpoint: String,
    scopes: Vec<String>,
    client_id: Option<String>,
    resource: Option<String>,
}

// These small adapters compile-check the public upstream entrypoints. They are
// unreachable from the foundation's status-only protocol and make no runtime
// network calls or credential reads in current operation/tests.
#[allow(dead_code)]
impl ApprovedHttpTarget {
    async fn http_client(&self) -> Result<RmcpClient, &'static str> {
        RmcpClient::new_streamable_http_client(
            &self.alias,
            &self.endpoint,
            None,
            None,
            None,
            OAuthCredentialsStoreMode::Keyring,
            AuthKeyringBackendKind::Direct,
            Arc::new(ReqwestHttpClient),
            None,
        )
        .await
        .map_err(|_| "provider transport unavailable")
    }

    async fn begin_oauth(&self) -> Result<OauthLoginHandle, &'static str> {
        codex_rmcp_client::perform_oauth_login_return_url_with_http_client(
            &self.alias,
            &self.endpoint,
            OAuthCredentialsStoreMode::Keyring,
            AuthKeyringBackendKind::Direct,
            None,
            None,
            &self.scopes,
            self.client_id.as_deref(),
            self.resource.as_deref(),
            Some(300),
            None,
            None,
            Arc::new(ReqwestHttpClient),
        )
        .await
        .map_err(|_| "provider authorization unavailable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_alias_isolated_by_both_scope_and_connection() {
        let a = account_alias("local-scope-a", "connection-a");
        assert_eq!(a, account_alias("local-scope-a", "connection-a"));
        assert_ne!(a, account_alias("local-scope-b", "connection-a"));
        assert_ne!(a, account_alias("local-scope-a", "connection-b"));
        assert!(a.starts_with("yijie_market_"));
        assert!(!a.contains("local-scope"));
    }
}
