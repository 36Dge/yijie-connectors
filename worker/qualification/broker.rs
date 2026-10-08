//! Dedicated, normally built qualification binary. Never shipped as the
//! product worker and never used to assert any provider's readiness.
#[path = "../src/broker.rs"]
mod broker;
#[allow(dead_code)]
#[path = "../src/broker_generated.rs"]
mod broker_generated;
#[path = "../src/credentials.rs"]
mod credentials;
#[path = "../src/daily.rs"]
mod daily;
#[path = "../src/gateway.rs"]
mod gateway;
#[allow(dead_code)]
#[path = "../src/generated.rs"]
mod generated;
#[path = "../src/generic.rs"]
mod generic;
#[path = "../src/google_calendar.rs"]
mod google_calendar;
#[allow(dead_code)]
#[path = "../src/native_library.rs"]
mod native_library;
#[path = "../src/oauth_policy.rs"]
mod oauth_policy;
#[allow(dead_code)]
#[path = "../src/provider_generated.rs"]
mod provider_generated;
#[path = "../src/provider_registry.rs"]
mod provider_registry;
#[path = "../src/providers.rs"]
mod providers;
#[path = "../src/secret_guard.rs"]
mod secret_guard;
#[allow(dead_code)]
#[path = "../src/selection_generated.rs"]
mod selection_generated;
#[allow(dead_code)]
#[path = "../src/tushare_oauth.rs"]
mod tushare_oauth;
fn main() -> std::io::Result<()> {
    if std::env::args().skip(1).collect::<Vec<_>>() != ["--broker-control-stdio"] {
        return Err(std::io::Error::other(
            "dedicated broker qualification mode required",
        ));
    }
    gateway::run(broker::ProviderMode::Qualification)
}
