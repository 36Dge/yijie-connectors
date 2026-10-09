//! Private owner-controlled JSONL worker. Auth status has no network or vault IO.
use std::io::{self, BufRead, Read, Write};

mod broker;
#[allow(dead_code)]
mod broker_generated;
mod credentials;
mod daily;
mod gateway;
#[allow(dead_code)]
mod generated;
mod generic;
mod google_calendar;
mod native_library;
mod oauth_policy;
#[allow(dead_code)]
mod provider_generated;
mod provider_registry;
mod providers;
mod secret_guard;
#[allow(dead_code)]
mod selection_generated;
mod shopify;
#[allow(dead_code)]
mod tushare_oauth;

use generated::{
    CatalogEntry, WorkerAuthStatus, WorkerAuthStatusRequest, WorkerAuthStatusResponse, WorkerError,
    WorkerErrorCode,
};
use serde::Deserialize;

const MAX_REQUEST_BYTES: u64 = 16 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductCatalog {
    catalog_revision: i64,
    catalog: Vec<CatalogEntry>,
}

fn error(request_id: Option<String>, code: WorkerErrorCode) -> serde_json::Value {
    serde_json::to_value(WorkerError {
        schema_version: 1,
        request_id,
        code,
        retryable: false,
    })
    .expect("fixed safe error is serializable")
}

fn handle(line: &[u8], catalog: &ProductCatalog) -> serde_json::Value {
    let request: WorkerAuthStatusRequest = match serde_json::from_slice(line) {
        Ok(value) => value,
        Err(_) => return error(None, WorkerErrorCode::InvalidRequest),
    };
    if !catalog
        .catalog
        .iter()
        .any(|item| item.service_id == request.service_id)
    {
        return error(Some(request.request_id), WorkerErrorCode::UnknownService);
    }
    let response = WorkerAuthStatusResponse {
        schema_version: 1,
        request_id: request.request_id,
        data: WorkerAuthStatus {
            service_id: request.service_id,
            qualification: "not_qualified".into(),
            authorization_status: "unknown".into(),
            connection_status: "disconnected".into(),
            execution_available: false,
            library_policy: native_library::policy(),
        },
    };
    response
        .validate()
        .expect("response follows canonical contract");
    serde_json::to_value(response).expect("status is serializable")
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--broker-control-stdio"] {
        return gateway::run(broker::ProviderMode::Product);
    }
    if !args.is_empty() {
        return Err(io::Error::other("unsupported worker mode"));
    }
    let catalog: ProductCatalog =
        serde_json::from_str(include_str!("../../catalog/market-catalog.v1.json"))
            .map_err(|_| io::Error::other("managed catalog is invalid"))?;
    if catalog.catalog_revision != 7 || catalog.catalog.len() != 58 {
        return Err(io::Error::other("managed catalog version is unsupported"));
    }
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let count = input
            .by_ref()
            .take(MAX_REQUEST_BYTES + 1)
            .read_until(b'\n', &mut line)?;
        if count == 0 {
            return Ok(()); // Normal EOF; no subprocess or forced termination.
        }
        let value = if line.len() as u64 > MAX_REQUEST_BYTES {
            error(None, WorkerErrorCode::InvalidRequest)
        } else {
            handle(&line, &catalog)
        };
        serde_json::to_writer(&mut output, &value)?;
        output.write_all(b"\n")?;
        output.flush()?;
        if line.len() as u64 > MAX_REQUEST_BYTES {
            return Ok(()); // Stop consuming an oversized request; never log it.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> ProductCatalog {
        serde_json::from_str(include_str!("../../catalog/market-catalog.v1.json")).unwrap()
    }

    #[test]
    fn every_known_service_reports_unqualified_without_external_io() {
        let catalog = catalog();
        for item in &catalog.catalog {
            let request = serde_json::json!({"schemaVersion":1,"requestId":"11111111-1111-4111-8111-111111111111","method":"auth_status","serviceId":item.service_id});
            let response: WorkerAuthStatusResponse =
                serde_json::from_value(handle(&serde_json::to_vec(&request).unwrap(), &catalog))
                    .unwrap();
            assert!(!response.data.execution_available);
            assert_eq!(response.data.authorization_status, "unknown");
            assert_eq!(response.data.qualification, "not_qualified");
            assert_eq!(response.data.library_policy.oauth_store, "keyring_only");
        }
    }

    #[test]
    fn ordinary_unknown_service_is_distinct_from_unqualified_service() {
        let request = br#"{"schemaVersion":1,"requestId":"11111111-1111-4111-8111-111111111111","method":"auth_status","serviceId":"unlisted-service"}"#;
        let response: WorkerError = serde_json::from_value(handle(request, &catalog())).unwrap();
        assert_eq!(response.code, WorkerErrorCode::UnknownService);
    }
}
