//! Native-authorized provider operations share the Host-owned worker. This
//! actor owns transient operation state and credentials, never installation DBs.
use crate::{
    broker::BrokerHandle,
    broker_generated as broker_wire, provider_generated as wire,
    tushare_oauth::{self, AuthControl, AuthRun, Failure, Phase},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::Notify, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Auth,
    Forget,
    Probe,
}
struct Operation {
    original: wire::ProviderPayload,
    kind: Kind,
    status: Arc<Mutex<wire::ProviderStatus>>,
    cancelled: CancellationToken,
    auth: Arc<Mutex<Option<AuthControl>>>,
    task: Option<JoinHandle<()>>,
    finished: Arc<AtomicBool>,
    completed: Arc<Notify>,
    backend: Arc<Mutex<Option<crate::broker::ProviderBackend>>>,
}
struct Activation {
    broker: BrokerHandle,
    payload: wire::ProviderPayload,
    backend: Arc<Mutex<Option<crate::broker::ProviderBackend>>>,
}
struct Completion {
    finished: Arc<AtomicBool>,
    completed: Arc<Notify>,
}
impl Drop for Completion {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::Release);
        self.completed.notify_waiters();
    }
}
struct BindingRecord {
    scope: wire::ScopeBinding,
    binding: wire::ProviderBinding,
}
pub struct Providers {
    enabled: bool,
    host_id: Option<String>,
    operations: HashMap<String, Operation>,
    bindings: HashMap<String, BindingRecord>,
    broker: BrokerHandle,
    stopped: bool,
}
fn failure_code(error: Failure) -> wire::ErrorCode {
    match error {
        Failure::Cancelled => wire::ErrorCode::AuthorizationRejected,
        Failure::Expired => wire::ErrorCode::AuthorizationTimeout,
        Failure::VaultUnavailable => wire::ErrorCode::KeyringUnavailable,
        Failure::InvalidMetadata => wire::ErrorCode::MetadataUnavailable,
        Failure::NotAuthorized => wire::ErrorCode::AuthorizationRejected,
        Failure::CleanupPending => wire::ErrorCode::CleanupPending,
        Failure::Unavailable => wire::ErrorCode::TemporarilyUnavailable,
    }
}
fn scope_identity(left: &wire::ScopeBinding, right: &wire::ScopeBinding) -> bool {
    left.owner_user_id == right.owner_user_id
        && left.tenant_id == right.tenant_id
        && left.native_process_epoch == right.native_process_epoch
}
fn stable_key(payload: &wire::ProviderPayload) -> String {
    serde_json::to_string(&(
        &payload.scope.owner_user_id,
        &payload.scope.tenant_id,
        &payload.binding.reference.installation_id,
    ))
    .expect("fixed identity")
}
fn alias(payload: &wire::ProviderPayload) -> String {
    tushare_oauth::credential_alias(
        &payload.scope.owner_user_id,
        &payload.scope.tenant_id,
        &payload.binding.reference.installation_id,
        payload.binding.reference.generation,
        &payload.binding.credential_ref,
    )
}
fn deadline(payload: &wire::ProviderPayload) -> Result<Instant, wire::ErrorCode> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| wire::ErrorCode::TemporarilyUnavailable)?
        .as_millis() as i64;
    let ttl = payload
        .scope
        .authorization_expires_at_unix_ms
        .saturating_sub(now)
        .min(wire::MAX_OAUTH_TTL_MS as i64);
    if ttl <= 0 {
        return Err(wire::ErrorCode::ScopeExpired);
    }
    Ok(Instant::now() + Duration::from_millis(ttl as u64))
}
fn initial(payload: &wire::ProviderPayload, kind: Kind) -> wire::ProviderStatus {
    wire::ProviderStatus {
        operation_id: payload.operation_id.clone(),
        host_instance_id: payload.host_instance_id.clone(),
        scope: payload.scope.clone(),
        binding: payload.binding.clone(),
        operation_state: wire::OperationState::Starting,
        authorization_status: if kind == Kind::Auth {
            wire::AuthorizationStatus::Authorizing
        } else {
            wire::AuthorizationStatus::Unknown
        },
        connection_status: if kind == Kind::Probe {
            wire::ConnectionStatus::Connecting
        } else {
            wire::ConnectionStatus::Disconnected
        },
        qualification: wire::Qualification::NotQualified,
        execution_available: false,
        authorization_url: None,
        error_code: None,
    }
}
fn failed(status: &Arc<Mutex<wire::ProviderStatus>>, code: wire::ErrorCode) {
    let mut state = status.lock().expect("provider operation");
    if state.operation_state == wire::OperationState::Cancelled {
        return;
    }
    state.operation_state = wire::OperationState::Failed;
    state.authorization_url = None;
    state.error_code = Some(code);
    // A completed OAuth commit remains a fact when a later connection check
    // fails, expires or cannot finish its normal cleanup.
    if state.authorization_status != wire::AuthorizationStatus::Authorized {
        state.authorization_status = wire::AuthorizationStatus::Error;
    }
    state.connection_status = wire::ConnectionStatus::Error;
    state.execution_available = false;
    state.qualification = wire::Qualification::NotQualified;
}
fn publish_auth(
    status: &Arc<Mutex<wire::ProviderStatus>>,
    observation: tushare_oauth::Observation,
) {
    let mut state = status.lock().expect("provider operation");
    if state.operation_state == wire::OperationState::Cancelled {
        return;
    }
    state.operation_state = match observation.phase {
        Phase::Starting => wire::OperationState::Starting,
        Phase::AwaitingUser => wire::OperationState::AwaitingUser,
        // Authorization completion starts read-only connection verification in
        // the same operation; it is not yet the operation's terminal receipt.
        Phase::Succeeded => wire::OperationState::Starting,
        Phase::Failed => wire::OperationState::Failed,
        Phase::Cancelled => wire::OperationState::Cancelled,
    };
    state.authorization_url = observation.authorization_url;
    state.authorization_status = match observation.phase {
        Phase::Succeeded => wire::AuthorizationStatus::Authorized,
        Phase::Failed => wire::AuthorizationStatus::Error,
        Phase::Cancelled => wire::AuthorizationStatus::Unauthorized,
        _ => wire::AuthorizationStatus::Authorizing,
    };
    state.error_code = observation.failure.map(failure_code);
    if observation.phase == Phase::Succeeded {
        state.connection_status = wire::ConnectionStatus::Connecting;
        state.qualification = wire::Qualification::NotQualified;
        state.execution_available = false;
    }
}
impl Providers {
    pub fn new(broker: BrokerHandle, enabled: bool) -> Self {
        Self {
            enabled,
            host_id: None,
            operations: HashMap::new(),
            bindings: HashMap::new(),
            broker,
            stopped: false,
        }
    }
    fn validate_authority(
        &mut self,
        payload: &wire::ProviderPayload,
    ) -> Result<(), wire::ErrorCode> {
        if self.stopped {
            return Err(wire::ErrorCode::TemporarilyUnavailable);
        }
        deadline(payload)?;
        let broker_host = self
            .broker
            .state
            .lock()
            .expect("broker lock")
            .host_instance_id()
            .map(str::to_owned);
        if self
            .host_id
            .as_ref()
            .is_some_and(|id| id != &payload.host_instance_id)
            || broker_host
                .as_ref()
                .is_some_and(|id| id != &payload.host_instance_id)
        {
            return Err(wire::ErrorCode::AuthorityMismatch);
        }
        if !self.enabled {
            return Err(wire::ErrorCode::UnsupportedAuth);
        }
        if crate::provider_registry::oauth(&payload.binding.service_id).is_none()
            && !crate::credentials::supports(&payload.binding.service_id)
        {
            return Err(wire::ErrorCode::UnsupportedAuth);
        }
        self.host_id = Some(payload.host_instance_id.clone());
        Ok(())
    }
    pub fn broker_initialize_error(&self, bytes: &[u8]) -> Option<Value> {
        let request: broker_wire::InitializeRequest = serde_json::from_slice(bytes).ok()?;
        self.host_id
            .as_ref()
            .filter(|host| *host != &request.payload.process.host_instance_id)
            .map(|_| {
                crate::broker::safe_error(
                    Some(request.request_id),
                    broker_wire::ErrorCode::BindingMismatch,
                )
            })
    }
    fn fence_binding(&mut self, payload: &wire::ProviderPayload) -> Result<(), wire::ErrorCode> {
        let key = stable_key(payload);
        if let Some(old) = self.bindings.get(&key) {
            if old.binding.service_id != payload.binding.service_id
                || payload.binding.reference.generation < old.binding.reference.generation
                || (payload.binding.reference.generation == old.binding.reference.generation
                    && payload.binding.reference.revision < old.binding.reference.revision)
                || (payload.binding.reference.generation == old.binding.reference.generation
                    && payload.binding.credential_ref != old.binding.credential_ref)
                || (scope_identity(&old.scope, &payload.scope)
                    && payload.scope.authorization_revision < old.scope.authorization_revision)
            {
                return Err(wire::ErrorCode::BindingMismatch);
            }
            if old.binding != payload.binding
                || !scope_identity(&old.scope, &payload.scope)
                || old.scope.authorization_revision != payload.scope.authorization_revision
            {
                self.broker
                    .state
                    .lock()
                    .expect("broker lock")
                    .invalidate_installation(
                        &payload.scope.owner_user_id,
                        &payload.scope.tenant_id,
                        &payload.binding.reference.installation_id,
                    );
                self.broker.changed.notify_waiters();
                for operation in self.operations.values() {
                    if stable_key(&operation.original) == key {
                        Self::cancel_operation(operation);
                    }
                }
            }
        }
        self.bindings.insert(
            key,
            BindingRecord {
                scope: payload.scope.clone(),
                binding: payload.binding.clone(),
            },
        );
        Ok(())
    }
    fn cancel_operation(operation: &Operation) -> wire::ProviderStatus {
        // Ask the owned auth handle synchronously before acknowledging cancel;
        // its commit lock prevents a detached late Keyring write.
        let auth = operation.auth.lock().expect("auth slot").clone();
        let authorization_committed =
            auth.is_some_and(|auth| auth.cancel().phase == Phase::Succeeded);
        operation.cancelled.cancel();
        let mut state = operation.status.lock().expect("provider operation");
        if let Some(backend) = operation.backend.lock().expect("backend slot").as_ref() {
            backend.revoke();
        }
        if authorization_committed {
            // OAuth has already committed. Cancellation can stop the metadata
            // step but must not falsely claim that the owned credential vanished.
            state.authorization_status = wire::AuthorizationStatus::Authorized;
        }
        if matches!(
            state.operation_state,
            wire::OperationState::Starting | wire::OperationState::AwaitingUser
        ) {
            state.operation_state = wire::OperationState::Cancelled;
            state.authorization_url = None;
            if state.authorization_status != wire::AuthorizationStatus::Authorized {
                state.authorization_status = wire::AuthorizationStatus::Unauthorized;
            }
            state.connection_status = wire::ConnectionStatus::Disconnected;
            state.execution_available = false;
        }
        state.clone()
    }
    fn existing(
        &self,
        payload: &wire::ProviderPayload,
        kind: Option<Kind>,
    ) -> Result<Option<wire::ProviderStatus>, wire::ErrorCode> {
        let Some(operation) = self.operations.get(&payload.operation_id) else {
            return Ok(None);
        };
        if kind.is_some_and(|kind| kind != operation.kind)
            || operation.original.host_instance_id != payload.host_instance_id
            || !scope_identity(&operation.original.scope, &payload.scope)
            || payload.scope.authorization_revision
                < operation.original.scope.authorization_revision
            || operation.original.binding != payload.binding
        {
            return Err(wire::ErrorCode::RequestConflict);
        }
        let mut result = operation.status.lock().expect("provider operation").clone();
        if operation.kind == Kind::Probe
            && result.operation_state == wire::OperationState::Succeeded
        {
            let live = self
                .broker
                .state
                .lock()
                .expect("broker lock")
                .daily_status(payload);
            result.execution_available = live;
            result.qualification = if live {
                wire::Qualification::Qualified
            } else {
                wire::Qualification::NotQualified
            };
            if !live {
                result.connection_status = wire::ConnectionStatus::Disconnected;
            }
        }
        result.scope = payload.scope.clone();
        Ok(Some(result))
    }
    fn begin(
        &mut self,
        payload: wire::ProviderPayload,
        kind: Kind,
    ) -> Result<wire::ProviderStatus, wire::ErrorCode> {
        self.validate_authority(&payload)?;
        if let Some(status) = self.existing(&payload, Some(kind))? {
            return Ok(status);
        }
        if self.operations.len() >= wire::MAX_OPERATIONS {
            return Err(wire::ErrorCode::CapacityExceeded);
        }
        if kind != Kind::Forget
            && self.operations.values().any(|operation| {
                stable_key(&operation.original) == stable_key(&payload)
                    && !operation.finished.load(Ordering::Acquire)
            })
        {
            return Err(wire::ErrorCode::TemporarilyUnavailable);
        }
        self.fence_binding(&payload)?;
        let end = deadline(&payload)?;
        let waiters: Vec<_> = self
            .operations
            .values()
            .filter(|operation| {
                stable_key(&operation.original) == stable_key(&payload)
                    && !operation.finished.load(Ordering::Acquire)
            })
            .map(|operation| (operation.finished.clone(), operation.completed.clone()))
            .collect();
        if kind == Kind::Forget {
            self.broker
                .state
                .lock()
                .expect("broker lock")
                .invalidate_installation(
                    &payload.scope.owner_user_id,
                    &payload.scope.tenant_id,
                    &payload.binding.reference.installation_id,
                );
            self.broker.changed.notify_waiters();
            for operation in self.operations.values() {
                if stable_key(&operation.original) == stable_key(&payload) {
                    Self::cancel_operation(operation);
                }
            }
        }
        let state = initial(&payload, kind);
        let status = Arc::new(Mutex::new(state.clone()));
        let cancelled = CancellationToken::new();
        let auth = Arc::new(Mutex::new(None));
        let state_ref = status.clone();
        let cancel_ref = cancelled.clone();
        let auth_ref = auth.clone();
        let credential_alias = alias(&payload);
        let backend = Arc::new(Mutex::new(None));
        let activation = Activation {
            broker: self.broker.clone(),
            payload: payload.clone(),
            backend: backend.clone(),
        };
        let prior_backends: Vec<_> = self
            .operations
            .values()
            .filter(|op| stable_key(&op.original) == stable_key(&payload))
            .map(|op| op.backend.clone())
            .collect();
        let finished = Arc::new(AtomicBool::new(false));
        let completed = Arc::new(Notify::new());
        let completion = Completion {
            finished: finished.clone(),
            completed: completed.clone(),
        };
        let task = tokio::spawn(async move {
            let _completion = completion;
            match kind {
                Kind::Auth => {
                    // Re-authorization can reuse the same generation/alias.
                    // Finish its former client refresh/writeback owner first.
                    for slot in prior_backends {
                        let backend = slot.lock().expect("backend slot").clone();
                        if let Some(backend) = backend {
                            backend.close().await;
                        }
                    }
                    run_auth(&credential_alias, end, state_ref, cancel_ref, auth_ref).await
                }
                Kind::Forget => {
                    // Wait for cancelled refresh/callback owners to finish before
                    // deletion, so an old client cannot repopulate the vault.
                    let cleanup = async {
                        for (done, notify) in waiters {
                            loop {
                                let wake = notify.notified();
                                if done.load(Ordering::Acquire) {
                                    break;
                                }
                                wake.await;
                            }
                        }
                        for slot in prior_backends {
                            let backend = slot.lock().expect("backend slot").clone();
                            if let Some(backend) = backend {
                                backend.close().await;
                            }
                        }
                    };
                    if tokio::time::timeout(Duration::from_secs(5), cleanup)
                        .await
                        .is_err()
                    {
                        failed(&state_ref, wire::ErrorCode::CleanupPending);
                        return;
                    }
                    let mut state = state_ref.lock().expect("provider operation");
                    if cancel_ref.is_cancelled()
                        || state.operation_state == wire::OperationState::Cancelled
                    {
                        return;
                    }
                    if Instant::now() >= end {
                        state.operation_state = wire::OperationState::Failed;
                        state.error_code = Some(wire::ErrorCode::ScopeExpired);
                        return;
                    }
                    match tushare_oauth::forget_for_service(
                        &credential_alias,
                        &state.binding.service_id,
                    ) {
                        Ok(()) => {
                            state.operation_state = wire::OperationState::Succeeded;
                            state.authorization_status = wire::AuthorizationStatus::Unauthorized;
                        }
                        Err(_) => {
                            state.operation_state = wire::OperationState::Failed;
                            state.error_code = Some(wire::ErrorCode::CleanupPending);
                            state.authorization_status = wire::AuthorizationStatus::Error;
                        }
                    }
                }
                Kind::Probe => {
                    for slot in prior_backends {
                        let backend = slot.lock().expect("backend slot").clone();
                        if let Some(backend) = backend {
                            backend.close().await;
                        }
                    }
                    run_probe(&credential_alias, end, state_ref, cancel_ref, activation).await
                }
            }
        });
        self.operations.insert(
            payload.operation_id.clone(),
            Operation {
                original: payload,
                kind,
                status,
                cancelled,
                auth,
                task: Some(task),
                finished,
                completed,
                backend,
            },
        );
        Ok(state)
    }
    fn poll(
        &mut self,
        payload: wire::ProviderPayload,
        cancel: bool,
    ) -> Result<wire::ProviderStatus, wire::ErrorCode> {
        self.validate_authority(&payload)?;
        let result = self
            .existing(&payload, None)?
            .ok_or(wire::ErrorCode::NotFound)?;
        if cancel {
            let operation = &self.operations[&payload.operation_id];
            if operation.kind == Kind::Forget {
                return Ok(result);
            }
            let mut result = Self::cancel_operation(operation);
            result.scope = payload.scope;
            Ok(result)
        } else {
            Ok(result)
        }
    }
    pub fn handle(&mut self, bytes: &[u8]) -> Option<Value> {
        let value: Value = serde_json::from_slice(bytes).ok()?;
        let method = value.get("method")?.as_str()?;
        if !method.starts_with("provider_") {
            return None;
        }
        macro_rules! action {
            ($req:ty,$response:ident,$kind:expr) => {{
                let request: $req = match serde_json::from_value(value) {
                    Ok(v) => v,
                    Err(_) => return Some(error(None, wire::ErrorCode::InvalidRequest)),
                };
                let id = request.request_id;
                match self.begin(request.payload, $kind) {
                    Ok(data) => serde_json::to_value(wire::$response {
                        schema_version: 1,
                        request_id: id,
                        data,
                    })
                    .expect("generated provider response"),
                    Err(code) => error(Some(id), code),
                }
            }};
        }
        macro_rules! poll {
            ($req:ty,$response:ident,$cancel:expr) => {{
                let request: $req = match serde_json::from_value(value) {
                    Ok(v) => v,
                    Err(_) => return Some(error(None, wire::ErrorCode::InvalidRequest)),
                };
                let id = request.request_id;
                match self.poll(request.payload, $cancel) {
                    Ok(data) => serde_json::to_value(wire::$response {
                        schema_version: 1,
                        request_id: id,
                        data,
                    })
                    .expect("generated provider response"),
                    Err(code) => error(Some(id), code),
                }
            }};
        }
        Some(match method {
            "provider_auth_begin" => action!(wire::AuthBeginRequest, AuthBeginResponse, Kind::Auth),
            "provider_forget" => action!(wire::ForgetRequest, ForgetResponse, Kind::Forget),
            "provider_probe" => action!(wire::ProbeRequest, ProbeResponse, Kind::Probe),
            "provider_auth_poll" => poll!(wire::AuthPollRequest, AuthPollResponse, false),
            "provider_auth_cancel" => poll!(wire::AuthCancelRequest, AuthCancelResponse, true),
            _ => error(None, wire::ErrorCode::InvalidRequest),
        })
    }
    pub async fn close(&mut self) -> Result<(), Failure> {
        self.stopped = true;
        for operation in self.operations.values() {
            Self::cancel_operation(operation);
        }
        for operation in self.operations.values_mut() {
            if let Some(mut task) = operation.task.take() {
                match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => return Err(Failure::Unavailable),
                    Err(_) => {
                        operation.task = Some(task);
                        return Err(Failure::Unavailable);
                    }
                }
            }
            let backend = operation.backend.lock().expect("backend slot").clone();
            if let Some(backend) = backend {
                backend.close().await;
            }
        }
        Ok(())
    }
}
fn error(request_id: Option<String>, code: wire::ErrorCode) -> Value {
    serde_json::to_value(wire::Error {
        schema_version: 1,
        request_id,
        code,
        retryable: false,
    })
    .expect("generated provider error")
}
async fn run_auth(
    alias: &str,
    end: Instant,
    status: Arc<Mutex<wire::ProviderStatus>>,
    cancelled: CancellationToken,
    slot: Arc<Mutex<Option<AuthControl>>>,
) {
    if cancelled.is_cancelled() {
        return;
    }
    if Instant::now() >= end {
        failed(&status, wire::ErrorCode::ScopeExpired);
        return;
    }
    let service = status
        .lock()
        .expect("provider operation")
        .binding
        .service_id
        .clone();
    let diagnostic = tushare_oauth::Diagnostic::for_operation(
        &status.lock().expect("provider operation").operation_id,
    );
    diagnostic.emit(
        tushare_oauth::Stage::KeyringPreflight,
        tushare_oauth::Event::Started,
        None,
        None,
    );
    let readiness = tokio::select! {biased;_=cancelled.cancelled()=>return, result=tushare_oauth::keyring_readiness_for(alias,&service)=>result};
    if cancelled.is_cancelled() {
        return;
    }
    if Instant::now() >= end {
        failed(&status, wire::ErrorCode::ScopeExpired);
        return;
    }
    diagnostic.emit(
        tushare_oauth::Stage::KeyringPreflight,
        if readiness.is_ok() {
            tushare_oauth::Event::Succeeded
        } else {
            tushare_oauth::Event::Failed
        },
        None,
        readiness.as_ref().err().copied(),
    );
    match readiness {
        Ok(true) => {
            publish_auth(
                &status,
                tushare_oauth::Observation {
                    phase: Phase::Succeeded,
                    authorization_url: None,
                    failure: None,
                },
            );
            run_probe_with_diagnostic(alias, end, status, cancelled, diagnostic, None).await;
            return;
        }
        Err(error) => {
            failed(&status, failure_code(error));
            return;
        }
        Ok(false) => {}
    }
    let mut run = match AuthRun::start_for_service(
        alias.into(),
        &service,
        end.saturating_duration_since(Instant::now()),
        diagnostic.clone(),
    ) {
        Ok(run) => run,
        Err(error) => {
            failed(&status, failure_code(error));
            return;
        }
    };
    let control = run.control();
    *slot.lock().expect("auth slot") = Some(control.clone());
    if cancelled.is_cancelled() {
        control.cancel();
    }
    let authorized = loop {
        let observation = run.observe();
        let authorized = observation.phase == Phase::Succeeded;
        let done = matches!(
            observation.phase,
            Phase::Succeeded | Phase::Failed | Phase::Cancelled
        );
        publish_auth(&status, observation);
        if done {
            break authorized;
        }
        tokio::select! {biased;_=cancelled.cancelled()=>{control.cancel();},_=tokio::time::sleep(Duration::from_millis(20))=>{}}
    };
    let mut cleanup_pending = false;
    while run.close().await.is_err() {
        // Keep owning the callback join fact. Later normal cleanup may finish;
        // forgetting credentials waits for this actor to actually complete.
        cleanup_pending = true;
        failed(&status, wire::ErrorCode::CleanupPending);
        tokio::task::yield_now().await;
    }
    if authorized && !cleanup_pending && !cancelled.is_cancelled() {
        // Keep the original scope deadline. No second DCR, authorization intent,
        // enable intent or executable tool is introduced by this metadata step.
        run_probe_with_diagnostic(alias, end, status, cancelled, diagnostic, None).await;
    }
}
async fn run_probe(
    alias: &str,
    end: Instant,
    status: Arc<Mutex<wire::ProviderStatus>>,
    cancelled: CancellationToken,
    activation: Activation,
) {
    let diagnostic = tushare_oauth::Diagnostic::for_operation(
        &status.lock().expect("provider operation").operation_id,
    );
    run_probe_with_diagnostic(alias, end, status, cancelled, diagnostic, Some(activation)).await;
}
async fn run_probe_with_diagnostic(
    alias: &str,
    end: Instant,
    status: Arc<Mutex<wire::ProviderStatus>>,
    cancelled: CancellationToken,
    diagnostic: tushare_oauth::Diagnostic,
    activation: Option<Activation>,
) {
    if cancelled.is_cancelled() {
        return;
    }
    if Instant::now() >= end {
        failed(&status, wire::ErrorCode::ScopeExpired);
        return;
    }
    diagnostic.emit(
        tushare_oauth::Stage::MetadataInitialize,
        tushare_oauth::Event::Started,
        None,
        None,
    );
    let service = status
        .lock()
        .expect("provider operation")
        .binding
        .service_id
        .clone();
    let permit = Arc::new(crate::daily::HttpPermit::default());
    let connect = tokio::select! {biased;_=cancelled.cancelled()=>return,_=tokio::time::sleep_until(end.into())=>{failed(&status,wire::ErrorCode::ScopeExpired);return;},result=tushare_oauth::connect_for_service(alias,&service,activation.as_ref().map(|_|permit.clone()))=>result};
    diagnostic.emit(
        tushare_oauth::Stage::MetadataInitialize,
        if connect.is_ok() {
            tushare_oauth::Event::Succeeded
        } else {
            tushare_oauth::Event::Failed
        },
        None,
        connect.as_ref().err().copied(),
    );
    let client = match connect {
        Ok(client) => client,
        Err(error) => {
            failed(&status, failure_code(error));
            return;
        }
    };
    if cancelled.is_cancelled() || Instant::now() >= end {
        client.shutdown().await;
        if !cancelled.is_cancelled() {
            failed(&status, wire::ErrorCode::ScopeExpired);
        }
        return;
    }
    // Discovery only. No tool name, schema, annotations, or authorization success
    // can silently qualify a financial call before its explicit provider review.
    diagnostic.emit(
        tushare_oauth::Stage::MetadataToolsList,
        tushare_oauth::Event::Started,
        None,
        None,
    );
    let outcome = tokio::select! {biased;_=cancelled.cancelled()=>Err(wire::ErrorCode::AuthorizationRejected),_=tokio::time::sleep_until(end.into())=>Err(wire::ErrorCode::ScopeExpired),result=list_tools(&client)=>result};
    diagnostic.emit(
        tushare_oauth::Stage::MetadataToolsList,
        if outcome.is_ok() {
            tushare_oauth::Event::Succeeded
        } else {
            tushare_oauth::Event::Failed
        },
        None,
        None,
    );
    let definitions = if service != "tushareMcp" {
        outcome
            .as_ref()
            .ok()
            .filter(|tools| {
                serde_json::to_value(&tools.tools).is_ok_and(|value| permit.1.safe(&value))
            })
            .and_then(|tools| {
                let tools = if service == crate::google_calendar::SERVICE {
                    let config = crate::google_calendar::load(alias).ok()??;
                    crate::google_calendar::tools_for_scopes(&config, tools.tools.clone())
                } else {
                    tools.tools.clone()
                };
                crate::generic::definitions(&service, tools).ok()
            })
    } else {
        None
    };
    let qualified = outcome
        .as_ref()
        .is_ok_and(|tools| crate::daily::qualified(&tools.tools));
    let backend = if activation.is_some() && service == "tushareMcp" && qualified {
        Some(crate::broker::ProviderBackend::Daily(
            crate::daily::Backend::new(client, permit),
        ))
    } else if activation.is_some() && definitions.is_some() {
        Some(crate::broker::ProviderBackend::Generic(
            crate::generic::Backend::new(
                service.clone(),
                definitions.expect("checked schema"),
                client,
                permit,
            ),
        ))
    } else {
        client.shutdown().await;
        None
    };
    if cancelled.is_cancelled() {
        if let Some(backend) = backend {
            backend.close().await;
        }
        return;
    }
    if Instant::now() >= end {
        if let Some(backend) = backend {
            backend.close().await;
        }
        failed(&status, wire::ErrorCode::ScopeExpired);
        return;
    }
    match outcome {
        Ok(tools)
            if !tools.tools.is_empty()
                && tools.tools.len() <= 1024
                && tools.next_cursor.is_none() =>
        {
            let mut state = status.lock().expect("provider operation");
            if state.operation_state == wire::OperationState::Cancelled || cancelled.is_cancelled()
            {
                if let Some(backend) = backend {
                    backend.revoke();
                    if let Some(activation) = &activation {
                        *activation.backend.lock().expect("backend slot") = Some(backend);
                    }
                }
                return;
            }
            let saved = if service == "tushareMcp" {
                save_probe_evidence(&state.operation_id, &tools.tools)
            } else {
                Ok(())
            };
            diagnostic.emit(
                tushare_oauth::Stage::MetadataArtifact,
                if saved.is_ok() {
                    tushare_oauth::Event::Succeeded
                } else {
                    tushare_oauth::Event::Failed
                },
                None,
                None,
            );
            if saved.is_err() {
                if let Some(backend) = backend {
                    backend.revoke();
                    if let Some(activation) = &activation {
                        *activation.backend.lock().expect("backend slot") = Some(backend);
                    }
                }
                state.operation_state = wire::OperationState::Failed;
                state.error_code = Some(wire::ErrorCode::MetadataUnavailable);
                state.connection_status = wire::ConnectionStatus::Error;
                return;
            }
            state.operation_state = wire::OperationState::Succeeded;
            state.authorization_status = wire::AuthorizationStatus::Authorized;
            state.connection_status = wire::ConnectionStatus::Connected;
            state.qualification = wire::Qualification::NotQualified;
            state.execution_available = false;
            if let (Some(activation), Some(backend)) = (activation, backend) {
                *activation.backend.lock().expect("backend slot") = Some(backend.clone());
                if activation
                    .broker
                    .state
                    .lock()
                    .expect("broker lock")
                    .register_provider(activation.payload, backend.clone())
                {
                    state.qualification = wire::Qualification::Qualified;
                    state.execution_available = true;
                } else {
                    backend.revoke();
                }
            }
        }
        Ok(_) => failed(&status, wire::ErrorCode::MetadataUnavailable),
        Err(code) => failed(&status, code),
    }
}

async fn list_tools(
    client: &codex_rmcp_client::RmcpClient,
) -> Result<rmcp::model::ListToolsResult, wire::ErrorCode> {
    let mut tools = Vec::new();
    let mut cursor = None;
    let mut seen = std::collections::HashSet::new();
    loop {
        let page = client
            .list_tools(cursor.clone(), Some(Duration::from_secs(15)))
            .await
            .map_err(|_| wire::ErrorCode::MetadataUnavailable)?;
        tools.extend(page.tools);
        if tools.len() > 1024 || seen.len() >= 32 {
            return Err(wire::ErrorCode::MetadataUnavailable);
        }
        let Some(next) = page.next_cursor else {
            return Ok(rmcp::model::ListToolsResult {
                tools,
                ..Default::default()
            });
        };
        if !seen.insert(next.clone()) {
            return Err(wire::ErrorCode::MetadataUnavailable);
        }
        let mut params = rmcp::model::PaginatedRequestParams::default();
        params.cursor = Some(next);
        cursor = Some(params);
    }
}

// Private operational qualification evidence, not a second installation store.
// Preserve only expected daily schema structure and digests; discard arbitrary
// supplier descriptions, annotations, defaults, examples, enum/const values.
fn safe_daily_shape(schema: &Value) -> Value {
    let known = [
        "ts_code",
        "trade_date",
        "start_date",
        "end_date",
        "fields",
        "offset",
        "limit",
    ];
    fn types(schema: &Value) -> Vec<String> {
        let names = [
            "object", "array", "string", "number", "integer", "boolean", "null",
        ];
        let mut out = Vec::new();
        if let Some(name) = schema
            .get("type")
            .and_then(Value::as_str)
            .filter(|v| names.contains(v))
        {
            out.push(name.to_string());
        }
        for branch in ["anyOf", "oneOf"] {
            if let Some(items) = schema.get(branch).and_then(Value::as_array) {
                for item in items.iter().take(8) {
                    if let Some(name) = item
                        .get("type")
                        .and_then(Value::as_str)
                        .filter(|v| names.contains(v))
                    {
                        out.push(name.to_string());
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    let mut projection = serde_json::Map::new();
    for name in known {
        if let Some(property) = properties.and_then(|v| v.get(name)) {
            projection.insert(name.into(),serde_json::json!({"types":types(property),"constraintReviewRequired":property.as_object().is_none_or(|fields|fields.keys().any(|key|!matches!(key.as_str(),"type"|"description"|"title"|"default"|"examples"|"anyOf"|"oneOf")))}));
        }
    }
    let required = schema.get("required").and_then(Value::as_array);
    let required_known: Vec<_> = required
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|v| known.contains(v))
        .collect();
    serde_json::json!({"types":types(schema),"properties":projection,"requiredKnownFields":required_known,
        "unknownPropertyCount":properties.map(|v|v.keys().filter(|name|!known.contains(&name.as_str())).count()).unwrap_or(0),
        "unknownRequiredCount":required.map(|v|v.iter().filter(|name|name.as_str().is_none_or(|name|!known.contains(&name))).count()).unwrap_or(0),
        "topLevelConstraintReviewRequired":schema.as_object().is_none_or(|fields|fields.keys().any(|key|!matches!(key.as_str(),"type"|"properties"|"required"|"additionalProperties"|"description"|"title"|"$schema")))})
}
fn save_probe_evidence(operation_id: &str, tools: &[rmcp::model::Tool]) -> Result<(), ()> {
    let home = tushare_oauth::validate_library_home().map_err(|_| ())?;
    let serialized = serde_json::to_vec(tools).map_err(|_| ())?;
    if serialized.len() > 4 * 1024 * 1024 {
        return Err(());
    }
    let daily: Vec<_> = tools.iter().filter(|tool| tool.name == "daily").collect();
    let candidate=daily.first().map(|tool| {
        let schema=serde_json::to_value(&tool.input_schema).expect("MCP schema");
        serde_json::json!({"name":"daily","inputSchemaSha256":format!("{:x}",Sha256::digest(serde_json::to_vec(&schema).expect("schema JSON"))),"structure":safe_daily_shape(&schema)})
    });
    let report = serde_json::json!({"schemaVersion":1,"serviceId":"tushareMcp","providerProfile":wire::TUSHARE_DAILY_PROFILE,"operationId":operation_id,
        "toolCount":tools.len(),"toolCatalogSha256":format!("{:x}",Sha256::digest(serialized)),"dailyNameMatches":daily.len(),"dailyCandidate":candidate,
        "financialToolCalls":0,"qualification":"not_qualified","executionAvailable":false,"dailyPolicyReviewPassed":crate::daily::qualified(tools),"supplierTextOmitted":true});
    let directory = home.join("qualification").join("tushare");
    std::fs::create_dir_all(&directory).map_err(|_| ())?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|_| ())?;
    let path = directory.join(format!("{operation_id}.json"));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => file.write_all(&bytes).map_err(|_| ()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if std::fs::read(path).map_err(|_| ())? == bytes {
                Ok(())
            } else {
                Err(())
            }
        }
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authorization_commit_waits_for_metadata_and_connection_failure_preserves_it() {
        let status = Arc::new(Mutex::new(initial(&payload(), Kind::Auth)));
        publish_auth(
            &status,
            tushare_oauth::Observation {
                phase: Phase::Succeeded,
                authorization_url: None,
                failure: None,
            },
        );
        {
            let state = status.lock().unwrap();
            assert_eq!(state.operation_state, wire::OperationState::Starting);
            assert_eq!(
                state.authorization_status,
                wire::AuthorizationStatus::Authorized
            );
            assert_eq!(state.connection_status, wire::ConnectionStatus::Connecting);
            assert!(!state.execution_available);
            state.validate().unwrap();
        }
        failed(&status, wire::ErrorCode::MetadataUnavailable);
        let state = status.lock().unwrap();
        assert_eq!(state.operation_state, wire::OperationState::Failed);
        assert_eq!(
            state.authorization_status,
            wire::AuthorizationStatus::Authorized
        );
        assert_eq!(state.connection_status, wire::ConnectionStatus::Error);
        assert_eq!(state.qualification, wire::Qualification::NotQualified);
        state.validate().unwrap();
    }
    #[test]
    fn cancelling_connection_check_preserves_completed_authorization() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Auth,
            wire::OperationState::Starting,
        );
        publish_auth(
            &actor.operations[&request.operation_id].status,
            tushare_oauth::Observation {
                phase: Phase::Succeeded,
                authorization_url: None,
                failure: None,
            },
        );
        let cancelled = actor.poll(request.clone(), true).unwrap();
        assert_eq!(cancelled.operation_state, wire::OperationState::Cancelled);
        assert_eq!(
            cancelled.authorization_status,
            wire::AuthorizationStatus::Authorized
        );
        assert_eq!(
            cancelled.connection_status,
            wire::ConnectionStatus::Disconnected
        );
        assert!(!cancelled.execution_available);
        assert!(
            actor.operations[&request.operation_id]
                .cancelled
                .is_cancelled()
        );
    }
    #[tokio::test]
    async fn expired_metadata_step_never_erases_committed_authorization() {
        let status = Arc::new(Mutex::new(initial(&payload(), Kind::Auth)));
        publish_auth(
            &status,
            tushare_oauth::Observation {
                phase: Phase::Succeeded,
                authorization_url: None,
                failure: None,
            },
        );
        run_probe_with_diagnostic(
            "unused-alias",
            Instant::now(),
            status.clone(),
            CancellationToken::new(),
            tushare_oauth::Diagnostic::default(),
            None,
        )
        .await;
        let state = status.lock().unwrap();
        assert_eq!(state.operation_state, wire::OperationState::Failed);
        assert_eq!(
            state.authorization_status,
            wire::AuthorizationStatus::Authorized
        );
        assert_eq!(state.error_code, Some(wire::ErrorCode::ScopeExpired));
        assert!(!state.execution_available);
    }
    #[test]
    fn qualification_projection_retains_structure_without_supplier_text() {
        let shape = safe_daily_shape(
            &serde_json::json!({"type":"object","description":"ordinary provider description","properties":{"ts_code":{"type":"string","description":"ordinary sample stock","default":"000001.SZ"},"trade_date":{"type":"string"},"page":{"type":"integer"}},"required":["ts_code","page"]}),
        );
        assert_eq!(
            shape["properties"]["ts_code"]["types"],
            serde_json::json!(["string"])
        );
        assert_eq!(shape["unknownPropertyCount"], 1);
        assert_eq!(shape["unknownRequiredCount"], 1);
        let serialized = shape.to_string();
        assert!(!serialized.contains("ordinary provider"));
        assert!(!serialized.contains("000001.SZ"));
    }
    fn payload() -> wire::ProviderPayload {
        let id = || uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        wire::ProviderPayload {
            operation_id: id(),
            host_instance_id: id(),
            scope: wire::ScopeBinding {
                owner_user_id: id(),
                tenant_id: id(),
                native_process_epoch: id(),
                authorization_revision: 1,
                authorization_expires_at_unix_ms: now + 60_000,
            },
            binding: wire::ProviderBinding {
                reference: crate::generated::SelectionRef {
                    installation_id: id(),
                    revision: 1,
                    generation: 1,
                },
                service_id: "tushareMcp".into(),
                credential_ref: id(),
            },
        }
    }
    fn actor(enabled: bool) -> Providers {
        Providers::new(
            BrokerHandle::new(
                crate::broker::ProviderMode::Product,
                "http://127.0.0.1:12345/mcp".into(),
            ),
            enabled,
        )
    }
    fn seed(
        actor: &mut Providers,
        payload: &wire::ProviderPayload,
        kind: Kind,
        state: wire::OperationState,
    ) {
        let mut status = initial(payload, kind);
        status.operation_state = state;
        if state == wire::OperationState::Succeeded {
            status.authorization_status = wire::AuthorizationStatus::Authorized;
        }
        actor.operations.insert(
            payload.operation_id.clone(),
            Operation {
                original: payload.clone(),
                kind,
                status: Arc::new(Mutex::new(status)),
                cancelled: CancellationToken::new(),
                auth: Arc::new(Mutex::new(None)),
                task: None,
                finished: Arc::new(AtomicBool::new(false)),
                completed: Arc::new(Notify::new()),
                backend: Arc::new(Mutex::new(None)),
            },
        );
    }
    #[test]
    fn provider_status_queries_do_not_require_or_create_runtime_identity() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Auth,
            wire::OperationState::Starting,
        );
        let status = actor.poll(request, false).unwrap();
        assert_eq!(status.operation_state, wire::OperationState::Starting);
        assert!(
            actor
                .broker
                .state
                .lock()
                .unwrap()
                .host_instance_id()
                .is_none()
        );
    }
    #[test]
    fn credential_rotation_requires_a_new_installation_generation() {
        let mut actor = actor(true);
        let original = payload();
        actor.fence_binding(&original).unwrap();
        let mut rotated = original.clone();
        rotated.binding.reference.revision += 1;
        rotated.binding.credential_ref = uuid::Uuid::new_v4().to_string();
        assert_eq!(
            actor.fence_binding(&rotated),
            Err(wire::ErrorCode::BindingMismatch)
        );
        assert_eq!(
            actor.bindings[&stable_key(&original)].binding,
            original.binding
        );
        rotated.binding.reference.generation += 1;
        actor.fence_binding(&rotated).unwrap();
    }
    #[test]
    fn qualification_profile_cannot_start_real_oauth() {
        let mut actor = actor(false);
        assert!(matches!(
            actor.begin(payload(), Kind::Auth),
            Err(wire::ErrorCode::UnsupportedAuth)
        ));
        assert!(actor.operations.is_empty());
    }
    #[test]
    fn removed_providers_do_not_start_authentication_or_discovery() {
        for service in ["taobao-flash-sale-retail", "doukou-doctor"] {
            let mut actor = actor(true);
            let mut request = payload();
            request.binding.service_id = service.into();
            for kind in [Kind::Auth, Kind::Probe] {
                assert!(matches!(
                    actor.begin(request.clone(), kind),
                    Err(wire::ErrorCode::UnsupportedAuth)
                ));
            }
            assert!(actor.operations.is_empty());
            assert!(actor.bindings.is_empty());
        }
    }
    #[test]
    fn normal_scope_renewal_reads_same_operation_without_restarting_it() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Auth,
            wire::OperationState::AwaitingUser,
        );
        let original_expiry = request.scope.authorization_expires_at_unix_ms;
        let mut renewed = request.clone();
        renewed.scope.authorization_revision += 1;
        renewed.scope.authorization_expires_at_unix_ms += 30_000;
        let status = actor.poll(renewed.clone(), false).unwrap();
        assert_eq!(status.operation_id, request.operation_id);
        assert_eq!(status.scope, renewed.scope);
        assert_eq!(
            actor.operations[&request.operation_id]
                .original
                .scope
                .authorization_expires_at_unix_ms,
            original_expiry
        );
        assert_eq!(actor.operations.len(), 1);
    }
    #[test]
    fn ordinary_cancel_before_oauth_callback_owner_exists_closes_admission() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Auth,
            wire::OperationState::Starting,
        );
        let status = actor.poll(request.clone(), true).unwrap();
        assert_eq!(status.operation_state, wire::OperationState::Cancelled);
        assert!(
            actor.operations[&request.operation_id]
                .cancelled
                .is_cancelled()
        );
        assert!(!status.execution_available);
    }
    #[test]
    fn forget_cancel_only_reads_original_fact() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Forget,
            wire::OperationState::Starting,
        );
        let status = actor.poll(request.clone(), true).unwrap();
        assert_eq!(status.operation_state, wire::OperationState::Starting);
        assert!(
            !actor.operations[&request.operation_id]
                .cancelled
                .is_cancelled()
        );
    }
    #[test]
    fn successful_auth_is_never_a_qualified_tool_or_enabled_installation() {
        let mut actor = actor(true);
        let request = payload();
        seed(
            &mut actor,
            &request,
            Kind::Auth,
            wire::OperationState::Succeeded,
        );
        let status = actor.poll(request, true).unwrap();
        assert_eq!(status.operation_state, wire::OperationState::Succeeded);
        assert_eq!(
            status.authorization_status,
            wire::AuthorizationStatus::Authorized
        );
        assert_eq!(status.qualification, wire::Qualification::NotQualified);
        assert!(!status.execution_available);
    }
}
