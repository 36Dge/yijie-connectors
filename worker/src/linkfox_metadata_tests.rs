//! Ordinary public discovery documents, captured without an account on 2026-10-17.
use super::*;

#[test]
fn feat158_linkfox_public_discovery_reuses_exact_existing_oauth_policy() {
    let spec = crate::provider_registry::oauth("linkfox").unwrap();
    let resource = "https://mcp-tool-gateway.linkfox.com/mcp/any-tool";
    assert_eq!(spec.resource.as_deref(), Some(resource));
    assert_eq!(spec.oauth_scopes, ["mcp:read", "mcp:invoke"]);
    let policy = DiscoveryPolicy::new(resource.into()).unwrap();
    policy.observe_challenge(resource, 401, "Bearer resource_metadata=\"https://mcp-tool-gateway.linkfox.com/.well-known/oauth-protected-resource\", scope=\"mcp:read mcp:invoke\"").unwrap();
    let metadata = "https://mcp-tool-gateway.linkfox.com/.well-known/oauth-protected-resource";
    assert!(policy.permits("GET", metadata));
    policy
        .observe(
            metadata,
            include_bytes!("../tests/fixtures/linkfox/protected-resource.json"),
        )
        .unwrap();
    let issuer_metadata = "https://tool-gateway.linkfox.com/.well-known/oauth-authorization-server";
    assert!(policy.permits("GET", issuer_metadata));
    let body = include_bytes!("../tests/fixtures/linkfox/authorization-server.json");
    let authorization: AuthorizationMetadata = serde_json::from_slice(body).unwrap();
    assert_eq!(
        authorization.issuer.as_deref(),
        Some("https://tool-gateway.linkfox.com")
    );
    policy.observe(issuer_metadata, body).unwrap();
    assert!(policy.permits("POST", "https://tool-gateway.linkfox.com/oauth/register"));
    assert!(policy.permits("POST", "https://tool-gateway.linkfox.com/oauth/token"));
    // Eligibility of the declared endpoints is not a real registration, account or tool discovery.
}
