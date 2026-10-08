//! Fixed provider configuration owned by Connectors; never read endpoints from IPC.
use serde::Deserialize;
use std::sync::OnceLock;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Service {
    pub service_id: String,
    pub display_name: String,
    pub auth_mode: String,
    pub transport: String,
    pub resource: Option<String>,
    pub credential_header: Option<String>,
    #[serde(default)]
    pub credential_query: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    profile: String,
    services: Vec<Service>,
}
pub fn services() -> &'static [Service] {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    &REGISTRY
        .get_or_init(|| {
            let registry: Registry =
                serde_json::from_str(include_str!("../../catalog/provider-registry.v1.json"))
                    .expect("packaged registry");
            assert_eq!(registry.schema_version, 1);
            assert_eq!(registry.profile, "generic-mcp-v1");
            assert_eq!(registry.services.len(), 49);
            let mut seen = std::collections::HashSet::new();
            for service in &registry.services {
                assert!(seen.insert(&service.service_id));
            }
            registry
        })
        .services
}
pub fn get(service: &str) -> Option<&'static Service> {
    services().iter().find(|v| v.service_id == service)
}
pub fn oauth(service: &str) -> Option<&'static Service> {
    get(service).filter(|s| s.auth_mode == "oauth" && s.transport == "http")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_49_entries_match_catalog_without_network_or_credentials() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/market-catalog.v1.json")).unwrap();
        for entry in catalog["catalog"].as_array().unwrap() {
            let spec = get(entry["serviceId"].as_str().unwrap()).unwrap();
            assert_eq!(spec.display_name, entry["displayName"].as_str().unwrap());
            assert_eq!(spec.auth_mode, entry["authMode"].as_str().unwrap());
            assert_eq!(spec.transport, entry["transport"].as_str().unwrap());
            assert_ne!(entry["availability"], "blocked");
            if let Some(resource) = &spec.resource {
                let u = url::Url::parse(resource).unwrap();
                assert_eq!(u.scheme(), "https");
                assert!(u.username().is_empty() && u.password().is_none());
            }
        }
        assert!(get("taobao-flash-sale-retail").is_none());
        assert!(get("doukou-doctor").is_none());
        assert_eq!(
            services()
                .iter()
                .filter(|s| oauth(&s.service_id).is_some())
                .count(),
            45
        );
        assert_eq!(
            get("FTShare").unwrap().credential_header.as_deref(),
            Some("FTSHARE_API_KEY")
        );
    }
}
