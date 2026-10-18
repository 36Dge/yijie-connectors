//! Public metadata captured without credentials on 2026-10-16. No remote calls.
//! The observed mismatch is retained as evidence, never normalized to bypass validation.
use super::*;

#[test]
fn feat158_tiktok_public_metadata_keeps_strict_issuer_validation_and_no_registration() {
    let spec = crate::provider_registry::oauth("chuhaijiang").unwrap();
    let resource = spec.resource.clone().unwrap();
    assert_eq!(resource, "https://mcp.gateway.chuhaijiang.com/mcp/oauth");
    assert_eq!(spec.oauth_scopes, ["mcp.full"]);
    let policy = DiscoveryPolicy::new(resource).unwrap();
    let resource_metadata =
        "https://mcp.gateway.chuhaijiang.com/.well-known/oauth-protected-resource/mcp/oauth";
    assert!(policy.permits("GET", resource_metadata));
    policy
        .observe(
            resource_metadata,
            include_bytes!("../tests/fixtures/chuhaijiang/protected-resource.json"),
        )
        .unwrap();
    let issuer_metadata =
        "https://openapi.gateway.chuhaijiang.com/.well-known/oauth-authorization-server";
    assert!(policy.permits("GET", issuer_metadata));
    let published: AuthorizationMetadata = serde_json::from_slice(include_bytes!(
        "../tests/fixtures/chuhaijiang/authorization-server.json"
    ))
    .unwrap();
    assert_eq!(
        published.issuer.as_deref(),
        Some("https://openapi.gateway.chuhaijiang.com")
    );
    assert!(matches!(
        policy.validate_metadata(&published),
        Err(Failure::InvalidMetadata)
    ));
    assert!(!policy.permits(
        "POST",
        "https://openapi.gateway.chuhaijiang.com/oauth/register"
    ));
    assert!(!policy.permits(
        "POST",
        "https://openapi.gateway.chuhaijiang.com/oauth/token"
    ));
}

#[test]
fn feat158_tiktok_standard_metadata_shape_uses_existing_discovery_with_exact_issuer() {
    // Ordinary synthetic provider, not a rewritten production response or live compatibility claim.
    let policy = DiscoveryPolicy::new("https://shop.example/mcp/oauth".into()).unwrap();
    policy.observe("https://shop.example/.well-known/oauth-protected-resource/mcp/oauth", br#"{"resource":"https://shop.example/mcp/oauth","authorization_servers":["https://auth.example/"]}"#).unwrap();
    let metadata: AuthorizationMetadata = serde_json::from_value(serde_json::json!({
        "issuer":"https://auth.example/",
        "authorization_endpoint":"https://auth.example/oauth/authorize",
        "token_endpoint":"https://auth.example/oauth/token",
        "registration_endpoint":"https://auth.example/oauth/register",
        "response_types_supported":["code"],
        "code_challenge_methods_supported":["S256"],
        "token_endpoint_auth_methods_supported":["none"],
        "scopes_supported":["mcp.full"]
    }))
    .unwrap();
    policy.validate_metadata(&metadata).unwrap();
    assert!(policy.permits("POST", "https://auth.example/oauth/register"));
    assert!(policy.permits("POST", "https://auth.example/oauth/token"));
}
