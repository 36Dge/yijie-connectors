//! Explicit isolated mode of the canonical worker, never a qualification binary.
use serde_json::Value;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};
static ACTIVE: AtomicBool = AtomicBool::new(false);
pub fn profile() -> &'static Value {
    static PROFILE: OnceLock<Value> = OnceLock::new();
    PROFILE.get_or_init(|| {
        serde_json::from_str(include_str!("../../catalog/market-local-acceptance.json"))
            .expect("canonical local acceptance profile")
    })
}
pub fn enabled() -> bool {
    ACTIVE.load(Ordering::Acquire)
}
pub fn service_id() -> &'static str {
    profile()["provider"]["serviceId"]
        .as_str()
        .expect("service")
}
pub fn activate(label: &str) -> std::io::Result<()> {
    let fail = || std::io::Error::other("isolated local acceptance profile required");
    if label.is_empty()
        || label.len() > 48
        || !label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || label.starts_with('-')
    {
        return Err(fail());
    }
    let home =
        std::path::PathBuf::from(std::env::var("YIJIE_MARKET_WORKER_HOME").map_err(|_| fail())?);
    let suffix = std::path::Path::new(".local/market-local-acceptance")
        .join(label)
        .join("host-home/market-worker/library-home");
    if !home.is_absolute()
        || !home.ends_with(suffix)
        || home.canonicalize()? != home
        || std::env::var("CODEX_HOME").ok().as_deref() != home.to_str()
    {
        return Err(fail());
    }
    ACTIVE.store(true, Ordering::Release);
    Ok(())
}
pub fn services() -> &'static [crate::provider_registry::Service] {
    static SERVICES: OnceLock<Vec<crate::provider_registry::Service>> = OnceLock::new();
    SERVICES.get_or_init(|| {
        vec![serde_json::from_value(profile()["provider"].clone()).expect("canonical provider")]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_worker_keeps_local_provider_unavailable() {
        assert!(!enabled());
        assert_eq!(crate::provider_registry::services().len(), 61);
        assert!(crate::provider_registry::get(service_id()).is_none());
        assert!(!crate::provider_registry::keyless(service_id()));
        let local = &services()[0];
        assert_eq!(
            local.resource.as_deref(),
            Some("http://127.0.0.1:18085/mcp")
        );
        assert_eq!(local.auth_mode, "none");
        assert_eq!(local.service_id, "market-local-verification");
        let entry: crate::generated::CatalogEntry =
            serde_json::from_value(profile()["catalogEntry"].clone()).unwrap();
        entry.validate().unwrap();
    }
}
