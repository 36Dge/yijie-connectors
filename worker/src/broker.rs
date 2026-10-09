//! Owner-only control state. This is transient admission state, not an
//! installation database or a provider credential store.
use crate::broker_generated as wire;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;
use uuid::Uuid;

pub const CAPABILITY_ENV: &str = "YIJIE_MARKET_RUNTIME_CAPABILITY";
#[cfg(feature = "broker-qualification")]
pub const QUALIFICATION_INSTALLATION: &str = "11111111-1111-4111-8111-111111111111";
const MAX_LEASES: usize = wire::MAX_RETAINED_LEASES;
const MAX_CALLS: usize = wire::MAX_PENDING_CALLS;
const MAX_RECEIPTS: usize = wire::MAX_MUTATION_RECEIPTS;
fn daily_key(scope: &wire::ScopeBinding, installation: &str) -> String {
    serde_json::to_string(&(&scope.owner_user_id, &scope.tenant_id, installation))
        .expect("daily scoped key")
}
fn same_authority(left: &wire::ScopeBinding, right: &wire::ScopeBinding) -> bool {
    left.owner_user_id == right.owner_user_id
        && left.tenant_id == right.tenant_id
        && left.native_process_epoch == right.native_process_epoch
        && left.authorization_revision == right.authorization_revision
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Result<(u64, u64), wire::ErrorCode>;
}
struct SystemClock(Instant);
impl Clock for SystemClock {
    fn now(&self) -> Result<(u64, u64), wire::ErrorCode> {
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| wire::ErrorCode::TemporarilyUnavailable)?
            .as_millis();
        let wall = u64::try_from(wall).map_err(|_| wire::ErrorCode::TemporarilyUnavailable)?;
        Ok((wall, self.0.elapsed().as_millis() as u64))
    }
}
#[derive(Clone, Copy)]
pub enum ProviderMode {
    #[cfg_attr(feature = "broker-qualification", allow(dead_code))]
    Product,
    #[cfg(feature = "broker-qualification")]
    #[allow(
        dead_code,
        reason = "Constructed only by the separate qualification entrypoint and its tests"
    )]
    Qualification,
}
impl ProviderMode {
    fn permits(&self, refs: &[crate::generated::SelectionRef]) -> bool {
        if refs.is_empty() {
            return true;
        }
        match self {
            Self::Product => false,
            #[cfg(feature = "broker-qualification")]
            Self::Qualification => {
                refs.len() == 1
                    && refs[0].installation_id == QUALIFICATION_INSTALLATION
                    && refs[0].revision == 1
                    && refs[0].generation == 1
            }
        }
    }
}
struct LeaseRecord {
    view: wire::Lease,
    deadline: u64,
    cancelled: tokio_util::sync::CancellationToken,
    providers: Vec<BoundProvider>,
}
struct DailyCapability {
    payload: crate::provider_generated::ProviderPayload,
    backend: ProviderBackend,
}
#[derive(Clone)]
struct BoundProvider {
    binding: crate::provider_generated::ProviderBinding,
    backend: ProviderBackend,
}
#[derive(Clone)]
pub enum ProviderBackend {
    Daily(Arc<crate::daily::Backend>),
    Generic(Arc<crate::generic::Backend>),
}
impl ProviderBackend {
    pub fn available(&self) -> bool {
        match self {
            Self::Daily(b) => b.available(),
            Self::Generic(b) => b.available(),
        }
    }
    pub fn revoke(&self) {
        match self {
            Self::Daily(b) => b.revoke(),
            Self::Generic(b) => b.revoke(),
        }
    }
    pub async fn close(&self) {
        match self {
            Self::Daily(b) => b.close().await,
            Self::Generic(b) => b.close().await,
        }
    }
    fn service(&self) -> &str {
        match self {
            Self::Daily(_) => "tushareMcp",
            Self::Generic(b) => &b.service,
        }
    }
    fn tools(&self) -> Vec<rmcp::model::Tool> {
        match self {
            Self::Daily(_) => vec![crate::daily::descriptor()],
            Self::Generic(b) => b.definitions.iter().map(|d| d.tool.clone()).collect(),
        }
    }
    fn contains(&self, tool: &str) -> bool {
        match self {
            Self::Daily(_) => tool == crate::provider_generated::TUSHARE_DAILY_TOOL_NAME,
            Self::Generic(b) => b.definition(tool).is_some(),
        }
    }
}
pub enum Execution {
    Synthetic(Value),
    Daily {
        backend: Arc<crate::daily::Backend>,
        arguments: crate::provider_generated::TushareDailyArguments,
        lease: tokio_util::sync::CancellationToken,
        deadline: Instant,
    },
    Generic {
        backend: Arc<crate::generic::Backend>,
        tool: String,
        arguments: Map<String, Value>,
        lease: tokio_util::sync::CancellationToken,
        deadline: Instant,
    },
}
struct CallRecord {
    view: wire::PendingCall,
    deadline: u64,
    // This exact parsed snapshot is used after admission; no re-parse/defaults.
    arguments: Option<Map<String, Value>>,
}
struct Receipt {
    intent: Value,
    result: Value,
    prepare_ref: Option<String>,
}
pub struct Broker {
    clock: Arc<dyn Clock>,
    mode: ProviderMode,
    worker_id: String,
    gateway_url: String,
    process: Option<wire::ProcessBinding>,
    stopped: bool,
    leases: HashMap<String, LeaseRecord>,
    calls: HashMap<String, CallRecord>,
    receipts: HashMap<String, Receipt>,
    turn_reservations: HashMap<String, String>,
    daily: HashMap<String, DailyCapability>,
}
pub type SharedBroker = Arc<Mutex<Broker>>;
#[derive(Clone)]
pub struct BrokerHandle {
    pub state: SharedBroker,
    pub changed: Arc<Notify>,
}
impl BrokerHandle {
    pub fn new(mode: ProviderMode, gateway_url: String) -> Self {
        Self {
            state: Arc::new(Mutex::new(Broker::new(
                mode,
                gateway_url,
                Arc::new(SystemClock(Instant::now())),
            ))),
            changed: Arc::new(Notify::new()),
        }
    }
    pub fn stop(&self) {
        self.state.lock().expect("broker lock").stop();
        self.changed.notify_waiters();
    }
    pub fn handle(&self, bytes: &[u8]) -> Value {
        let value = self.state.lock().expect("broker lock").handle(bytes);
        self.changed.notify_waiters();
        value
    }
}
impl Broker {
    fn new(mode: ProviderMode, gateway_url: String, clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            mode,
            worker_id: Uuid::new_v4().to_string(),
            gateway_url,
            process: None,
            stopped: false,
            leases: HashMap::new(),
            calls: HashMap::new(),
            receipts: HashMap::new(),
            turn_reservations: HashMap::new(),
            daily: HashMap::new(),
        }
    }
    fn now(&self) -> Result<(u64, u64), wire::ErrorCode> {
        self.clock.now()
    }
    fn refresh(&mut self) -> Result<u64, wire::ErrorCode> {
        let (_, now) = self.now()?;
        for record in self.leases.values_mut() {
            if now >= record.deadline
                && matches!(
                    record.view.state,
                    wire::LeaseState::Prepared | wire::LeaseState::Bound
                )
            {
                record.view.state = wire::LeaseState::Expired;
                record.view.revision += 1;
            }
            record.view.remaining_ttl_ms = record.deadline.saturating_sub(now) as i64;
            if matches!(
                record.view.state,
                wire::LeaseState::Expired | wire::LeaseState::Revoked
            ) {
                record.view.remaining_ttl_ms = 0;
                record.cancelled.cancel();
            }
        }
        for record in self.calls.values_mut() {
            if matches!(
                record.view.state,
                wire::CallState::Pending | wire::CallState::Approved
            ) {
                let lease = &self.leases[&record.view.identity.binding.capability_ref];
                let next = if matches!(lease.view.state, wire::LeaseState::Revoked) {
                    Some(wire::CallState::Revoked)
                } else if now >= record.deadline
                    || matches!(lease.view.state, wire::LeaseState::Expired)
                {
                    Some(wire::CallState::Expired)
                } else {
                    None
                };
                if let Some(next) = next {
                    record.view.state = next;
                    record.view.revision += 1;
                }
            }
            record.view.remaining_ttl_ms = record.deadline.saturating_sub(now) as i64;
            if !matches!(
                record.view.state,
                wire::CallState::Pending | wire::CallState::Approved
            ) {
                record.view.remaining_ttl_ms = 0;
                record.arguments = None;
            }
        }
        Ok(now)
    }
    fn require_running(&self) -> Result<(), wire::ErrorCode> {
        if self.stopped {
            Err(wire::ErrorCode::Stopping)
        } else if self.process.is_none() {
            Err(wire::ErrorCode::NotInitialized)
        } else {
            Ok(())
        }
    }
    fn require_process(&self, process: &wire::ProcessBinding) -> Result<(), wire::ErrorCode> {
        if self.process.as_ref() == Some(process) {
            Ok(())
        } else {
            Err(wire::ErrorCode::BindingMismatch)
        }
    }
    fn exact_lease(
        &self,
        binding: &wire::CapabilityBinding,
    ) -> Result<&LeaseRecord, wire::ErrorCode> {
        self.require_process(&binding.context.process)?;
        let record = self
            .leases
            .get(&binding.capability_ref)
            .ok_or(wire::ErrorCode::NotFound)?;
        if &record.view.binding != binding {
            return Err(wire::ErrorCode::BindingMismatch);
        }
        Ok(record)
    }
    fn live_lease(
        &self,
        binding: &wire::CapabilityBinding,
    ) -> Result<&LeaseRecord, wire::ErrorCode> {
        let record = self.exact_lease(binding)?;
        if record
            .providers
            .iter()
            .any(|provider| !provider.backend.available())
        {
            return Err(wire::ErrorCode::NotQualified);
        }
        match record.view.state {
            wire::LeaseState::Expired => Err(wire::ErrorCode::LeaseExpired),
            wire::LeaseState::Revoked => Err(wire::ErrorCode::LeaseRevoked),
            _ => Ok(record),
        }
    }
    fn replay<T: Serialize>(
        &mut self,
        operation: &str,
        method: &str,
        payload: &T,
    ) -> Result<Option<Value>, wire::ErrorCode> {
        self.refresh()?;
        let intent = serde_json::json!({"method":method,"payload":payload});
        if let Some(receipt) = self.receipts.get(operation) {
            if receipt.intent != intent {
                return Err(wire::ErrorCode::RequestConflict);
            }
            if let Some(reference) = &receipt.prepare_ref {
                let lease = &self.leases[reference];
                self.live_lease(&lease.view.binding)?;
                return Ok(Some(
                    serde_json::to_value(&lease.view).expect("generated lease"),
                ));
            }
            return Ok(Some(receipt.result.clone()));
        }
        self.require_running()?;
        if self.receipts.len() >= MAX_RECEIPTS {
            return Err(wire::ErrorCode::CapacityExceeded);
        }
        Ok(None)
    }
    fn remember<T: Serialize, R: Serialize>(
        &mut self,
        operation: &str,
        method: &str,
        payload: &T,
        result: &R,
        prepare_ref: Option<String>,
    ) {
        self.receipts.insert(
            operation.into(),
            Receipt {
                intent: serde_json::json!({"method":method,"payload":payload}),
                result: serde_json::to_value(result).expect("generated receipt"),
                prepare_ref,
            },
        );
    }
    fn initialize(
        &mut self,
        payload: wire::InitializePayload,
    ) -> Result<wire::InitializeResult, wire::ErrorCode> {
        let intent = serde_json::json!({"method":"initialize","payload":payload});
        if let Some(receipt) = self.receipts.get(&payload.operation_id) {
            if receipt.intent != intent {
                return Err(wire::ErrorCode::RequestConflict);
            }
            return serde_json::from_value(receipt.result.clone())
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        if self.stopped {
            return Err(wire::ErrorCode::Stopping);
        }
        if self.process.is_some() {
            return Err(wire::ErrorCode::AlreadyInitialized);
        }
        self.process = Some(payload.process.clone());
        let result = wire::InitializeResult {
            worker_instance_id: self.worker_id.clone(),
            process: payload.process.clone(),
            gateway_url: self.gateway_url.clone(),
            external_calls_enabled: matches!(self.mode, ProviderMode::Product),
        };
        self.remember(&payload.operation_id, "initialize", &payload, &result, None);
        Ok(result)
    }
    fn prepare(&mut self, payload: wire::PreparePayload) -> Result<wire::Lease, wire::ErrorCode> {
        if let Some(value) = self.replay(&payload.operation_id, "prepare", &payload)? {
            return serde_json::from_value(value)
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        self.require_process(&payload.context.process)?;
        let providers = self.providers_for(&payload.context, &payload.snapshot.selection);
        if !self.mode.permits(&payload.snapshot.selection) && providers.is_none() {
            return Err(wire::ErrorCode::NotQualified);
        }
        let (wall, now) = self.now()?;
        let remaining =
            (payload.context.scope.authorization_expires_at_unix_ms as u64).saturating_sub(wall);
        let ttl = remaining
            .min(payload.ttl_ms as u64)
            .min(wire::MAX_LEASE_TTL_MS as u64);
        if ttl == 0 {
            return Err(wire::ErrorCode::ScopeExpired);
        }
        // Reserve stable authority plus session/turn intent, independent of
        // operationId and requested TTL. A new mutation ID cannot resurrect it.
        let key = serde_json::to_string(&serde_json::json!({"scopeOwner":payload.context.scope.owner_user_id,
            "tenant":payload.context.scope.tenant_id,"epoch":payload.context.scope.native_process_epoch,
            "session":payload.context.agent_session_id,
            "operation":payload.snapshot.turn_operation_id})).expect("scope key");
        if let Some(reference) = self.turn_reservations.get(&key) {
            let old = &self.leases[reference];
            self.live_lease(&old.view.binding)?;
            if old.view.snapshot != payload.snapshot || old.view.binding.context != payload.context
            {
                return Err(wire::ErrorCode::RequestConflict);
            }
            let result = old.view.clone();
            self.remember(
                &payload.operation_id,
                "prepare",
                &payload,
                &result,
                Some(result.binding.capability_ref.clone()),
            );
            return Ok(result);
        }
        if self.leases.len() >= MAX_LEASES {
            return Err(wire::ErrorCode::CapacityExceeded);
        }
        // Only one live turn capability per native thread. Preparing another
        // turn before explicit terminal revoke is an owner sequencing error.
        if self.leases.values().any(|old| {
            payload
                .context
                .native_thread_id
                .as_ref()
                .is_some_and(|thread| {
                    old.view.bound_native_thread_id.as_ref() == Some(thread)
                        || old.view.binding.context.native_thread_id.as_ref() == Some(thread)
                })
                && matches!(
                    old.view.state,
                    wire::LeaseState::Prepared | wire::LeaseState::Bound
                )
        }) {
            return Err(wire::ErrorCode::RequestConflict);
        }
        let reference = Uuid::new_v4().to_string();
        let view = wire::Lease {
            binding: wire::CapabilityBinding {
                context: payload.context.clone(),
                capability_ref: reference.clone(),
                turn_operation_id: payload.snapshot.turn_operation_id.clone(),
                selection_digest: payload.snapshot.selection_digest.clone(),
            },
            snapshot: payload.snapshot.clone(),
            revision: 1,
            state: wire::LeaseState::Prepared,
            remaining_ttl_ms: ttl as i64,
            native_turn_id: None,
            bound_native_thread_id: None,
        };
        self.leases.insert(
            reference.clone(),
            LeaseRecord {
                view: view.clone(),
                deadline: now + ttl,
                cancelled: tokio_util::sync::CancellationToken::new(),
                providers: providers.unwrap_or_default(),
            },
        );
        self.turn_reservations.insert(key, reference.clone());
        self.remember(
            &payload.operation_id,
            "prepare",
            &payload,
            &view,
            Some(reference),
        );
        Ok(view)
    }
    fn bind_turn(
        &mut self,
        payload: wire::BindTurnPayload,
    ) -> Result<wire::Lease, wire::ErrorCode> {
        if let Some(value) = self.replay(&payload.operation_id, "bind_turn", &payload)? {
            self.live_lease(&payload.binding)?;
            return serde_json::from_value(value)
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        let old = self.live_lease(&payload.binding)?;
        if old.view.revision != payload.expected_revision {
            return Err(wire::ErrorCode::RevisionConflict);
        }
        if old.view.native_turn_id.is_some()
            || old
                .view
                .binding
                .context
                .native_thread_id
                .as_ref()
                .is_some_and(|thread| thread != &payload.native_thread_id)
        {
            return Err(wire::ErrorCode::TurnMismatch);
        }
        if self.leases.values().any(|other| {
            other.view.binding.capability_ref != payload.binding.capability_ref
                && other.view.bound_native_thread_id.as_ref() == Some(&payload.native_thread_id)
                && matches!(other.view.state, wire::LeaseState::Bound)
        }) {
            return Err(wire::ErrorCode::RequestConflict);
        }
        let old = self
            .leases
            .get_mut(&payload.binding.capability_ref)
            .expect("checked lease");
        old.view.state = wire::LeaseState::Bound;
        old.view.native_turn_id = Some(payload.native_turn_id.clone());
        old.view.bound_native_thread_id = Some(payload.native_thread_id.clone());
        old.view.revision += 1;
        let result = old.view.clone();
        self.remember(&payload.operation_id, "bind_turn", &payload, &result, None);
        Ok(result)
    }
    fn revoke(&mut self, payload: wire::RevokePayload) -> Result<wire::Lease, wire::ErrorCode> {
        if let Some(value) = self.replay(&payload.operation_id, "revoke", &payload)? {
            return serde_json::from_value(value)
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        let old = self.exact_lease(&payload.binding)?;
        if old.view.revision != payload.expected_revision {
            return Err(wire::ErrorCode::RevisionConflict);
        }
        let old = self
            .leases
            .get_mut(&payload.binding.capability_ref)
            .expect("checked lease");
        old.view.state = wire::LeaseState::Revoked;
        old.view.remaining_ttl_ms = 0;
        old.view.revision += 1;
        let result = old.view.clone();
        self.refresh()?;
        self.remember(&payload.operation_id, "revoke", &payload, &result, None);
        Ok(result)
    }
    fn status(&mut self, payload: wire::StatusPayload) -> Result<wire::Lease, wire::ErrorCode> {
        self.refresh()?;
        Ok(self.exact_lease(&payload.binding)?.view.clone())
    }
    fn pending_call(
        &mut self,
        payload: wire::PendingCallPayload,
    ) -> Result<wire::PendingCall, wire::ErrorCode> {
        self.refresh()?;
        self.exact_lease(&payload.binding)?;
        let call = self
            .calls
            .get(&payload.call_ref)
            .ok_or(wire::ErrorCode::NotFound)?;
        if call.view.identity.binding != payload.binding
            || call.view.identity.native_thread_id != payload.native_thread_id
            || call.view.identity.native_turn_id != payload.native_turn_id
        {
            return Err(wire::ErrorCode::BindingMismatch);
        }
        Ok(call.view.clone())
    }
    fn decide_call(
        &mut self,
        payload: wire::DecideCallPayload,
    ) -> Result<wire::PendingCall, wire::ErrorCode> {
        if let Some(value) = self.replay(&payload.operation_id, "decide_call", &payload)? {
            return serde_json::from_value(value)
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        let lease = self.live_lease(&payload.identity.binding)?;
        if lease.view.native_turn_id.as_ref() != Some(&payload.identity.native_turn_id)
            || lease.view.bound_native_thread_id.as_ref()
                != Some(&payload.identity.native_thread_id)
        {
            return Err(wire::ErrorCode::TurnMismatch);
        }
        let call = self
            .calls
            .get_mut(&payload.identity.call_ref)
            .ok_or(wire::ErrorCode::NotFound)?;
        if call.view.identity != payload.identity {
            return Err(wire::ErrorCode::BindingMismatch);
        }
        if call.view.revision != payload.expected_revision {
            return Err(wire::ErrorCode::RevisionConflict);
        }
        if !matches!(call.view.state, wire::CallState::Pending) {
            return Err(wire::ErrorCode::CallConsumed);
        }
        call.view.decision_id = Some(payload.decision_id.clone());
        call.view.decision = Some(payload.decision);
        call.view.state = match payload.decision {
            wire::Decision::ApproveOnce => wire::CallState::Approved,
            wire::Decision::Reject => wire::CallState::Rejected,
            wire::Decision::Cancel => wire::CallState::Cancelled,
        };
        call.view.revision += 1;
        if !matches!(call.view.state, wire::CallState::Approved) {
            call.arguments = None;
            call.view.remaining_ttl_ms = 0;
        }
        let result = call.view.clone();
        self.remember(
            &payload.operation_id,
            "decide_call",
            &payload,
            &result,
            None,
        );
        Ok(result)
    }
    fn shutdown(
        &mut self,
        payload: wire::ShutdownPayload,
    ) -> Result<wire::ShutdownResult, wire::ErrorCode> {
        if let Some(value) = self.replay(&payload.operation_id, "shutdown", &payload)? {
            return serde_json::from_value(value)
                .map_err(|_| wire::ErrorCode::TemporarilyUnavailable);
        }
        self.require_process(&payload.process)?;
        self.stop();
        let result = wire::ShutdownResult {
            state: "stopping".into(),
            stopped_admissions: true,
            external_outcomes: "not_asserted".into(),
        };
        self.remember(&payload.operation_id, "shutdown", &payload, &result, None);
        Ok(result)
    }
    fn stop(&mut self) {
        self.stopped = true;
        for capability in self.daily.values() {
            capability.backend.revoke();
        }
        for lease in self.leases.values_mut() {
            lease.cancelled.cancel();
            if matches!(
                lease.view.state,
                wire::LeaseState::Prepared | wire::LeaseState::Bound
            ) {
                lease.view.state = wire::LeaseState::Revoked;
                lease.view.revision += 1;
                lease.view.remaining_ttl_ms = 0;
            }
        }
        for call in self.calls.values_mut() {
            if matches!(
                call.view.state,
                wire::CallState::Pending | wire::CallState::Approved
            ) {
                call.view.state = wire::CallState::Revoked;
                call.view.revision += 1;
                call.view.remaining_ttl_ms = 0;
                call.arguments = None;
            }
        }
    }
    pub fn host_instance_id(&self) -> Option<&str> {
        self.process
            .as_ref()
            .map(|process| process.host_instance_id.as_str())
    }
    #[cfg(test)]
    pub fn register_daily(
        &mut self,
        payload: crate::provider_generated::ProviderPayload,
        backend: Arc<crate::daily::Backend>,
    ) -> bool {
        self.register_provider(payload, ProviderBackend::Daily(backend))
    }
    pub fn register_provider(
        &mut self,
        payload: crate::provider_generated::ProviderPayload,
        backend: ProviderBackend,
    ) -> bool {
        if payload.binding.service_id != backend.service() {
            return false;
        }
        if self.stopped || !matches!(self.mode, ProviderMode::Product) || !backend.available() {
            return false;
        }
        let Ok((wall, _)) = self.now() else {
            return false;
        };
        let remaining = (payload.scope.authorization_expires_at_unix_ms as u64)
            .saturating_sub(wall)
            .min(wire::MAX_LEASE_TTL_MS as u64);
        if remaining == 0 {
            return false;
        }
        let key = daily_key(&payload.scope, &payload.binding.reference.installation_id);
        if let Some(old) = self.daily.get(&key) {
            old.backend.revoke();
        }
        self.daily.insert(key, DailyCapability { payload, backend });
        true
    }
    pub fn daily_status(&mut self, payload: &crate::provider_generated::ProviderPayload) -> bool {
        let Ok((wall, _)) = self.now() else {
            return false;
        };
        let Some(cap) = self.daily.get_mut(&daily_key(
            &payload.scope,
            &payload.binding.reference.installation_id,
        )) else {
            return false;
        };
        if self.stopped
            || cap.payload.operation_id != payload.operation_id
            || cap.payload.binding != payload.binding
            || cap.payload.host_instance_id != payload.host_instance_id
            || !same_authority(&cap.payload.scope, &payload.scope)
            || !cap.backend.available()
        {
            return false;
        }
        let remaining = (payload.scope.authorization_expires_at_unix_ms as u64)
            .saturating_sub(wall)
            .min(wire::MAX_LEASE_TTL_MS as u64);
        if remaining == 0 {
            return false;
        }
        // A fresh Native permission lease can maintain current capability;
        // immutable turn leases and their original deadlines are never changed.
        cap.payload.scope = payload.scope.clone();
        true
    }
    fn providers_for(
        &self,
        context: &wire::ContextBinding,
        selection: &[crate::generated::SelectionRef],
    ) -> Option<Vec<BoundProvider>> {
        if !matches!(self.mode, ProviderMode::Product) {
            return None;
        }
        let mut result = Vec::new();
        let mut names = std::collections::HashSet::new();
        let mut bytes = 0usize;
        for reference in selection {
            let cap = self
                .daily
                .get(&daily_key(&context.scope, &reference.installation_id))?;
            if cap.payload.binding.reference != *reference
                || cap.payload.host_instance_id != context.process.host_instance_id
                || !same_authority(&cap.payload.scope, &context.scope)
                || !cap.backend.available()
            {
                return None;
            }
            for tool in cap.backend.tools() {
                bytes = bytes.checked_add(serde_json::to_vec(&tool).ok()?.len())?;
                if !names.insert(tool.name.to_string())
                    || names.len() > wire::GENERIC_MAX_TOOLS_PER_SELECTION
                    || bytes > wire::GENERIC_MAX_SCHEMAS_BYTES
                {
                    return None;
                }
            }
            result.push(BoundProvider {
                binding: cap.payload.binding.clone(),
                backend: cap.backend.clone(),
            });
        }
        Some(result)
    }
    pub fn invalidate_installation(&mut self, owner: &str, tenant: &str, installation: &str) {
        for cap in self.daily.values() {
            if cap.payload.binding.reference.installation_id == installation
                && cap.payload.scope.owner_user_id == owner
                && cap.payload.scope.tenant_id == tenant
            {
                cap.backend.revoke();
            }
        }
        for record in self.leases.values_mut() {
            if record.view.binding.context.scope.owner_user_id == owner
                && record.view.binding.context.scope.tenant_id == tenant
                && record
                    .view
                    .snapshot
                    .selection
                    .iter()
                    .any(|reference| reference.installation_id == installation)
                && matches!(
                    record.view.state,
                    wire::LeaseState::Prepared | wire::LeaseState::Bound
                )
            {
                record.view.state = wire::LeaseState::Revoked;
                record.view.revision += 1;
                record.view.remaining_ttl_ms = 0;
                record.cancelled.cancel();
            }
        }
        let _ = self.refresh();
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
    pub fn tools_visible(&mut self, reference: &str) -> Result<bool, wire::ErrorCode> {
        self.require_running()?;
        self.refresh()?;
        let lease = self
            .leases
            .get(reference)
            .ok_or(wire::ErrorCode::NotFound)?;
        self.live_lease(&lease.view.binding)?;
        Ok(!lease.view.snapshot.selection.is_empty()
            && (self.mode.permits(&lease.view.snapshot.selection) || !lease.providers.is_empty()))
    }
    pub fn visible_tools(
        &mut self,
        reference: &str,
    ) -> Result<Option<Vec<rmcp::model::Tool>>, wire::ErrorCode> {
        if !self.tools_visible(reference)? {
            return Ok(Some(vec![]));
        }
        let lease = &self.leases[reference];
        if lease.providers.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            lease
                .providers
                .iter()
                .flat_map(|p| p.backend.tools())
                .collect(),
        ))
    }
    pub fn register_call(
        &mut self,
        reference: &str,
        thread: &str,
        turn: &str,
        tool: &str,
        arguments: Option<Map<String, Value>>,
    ) -> Result<wire::PendingCall, wire::ErrorCode> {
        self.require_running()?;
        let now = self.refresh()?;
        let lease = self
            .leases
            .get(reference)
            .ok_or(wire::ErrorCode::NotFound)?;
        self.live_lease(&lease.view.binding)?;
        if lease.view.native_turn_id.is_none() {
            return Err(wire::ErrorCode::NotBound);
        }
        if lease.view.bound_native_thread_id.as_deref() != Some(thread)
            || lease.view.native_turn_id.as_deref() != Some(turn)
        {
            return Err(wire::ErrorCode::TurnMismatch);
        }
        let args = arguments.as_ref().ok_or(wire::ErrorCode::InvalidRequest)?;
        let (service, selected, review) = if let Some(provider) =
            lease.providers.iter().find(|p| p.backend.contains(tool))
        {
            let review = match &provider.backend {
                ProviderBackend::Daily(_) => {
                    let args: crate::provider_generated::TushareDailyArguments =
                        serde_json::from_value(Value::Object(args.clone()))
                            .map_err(|_| wire::ErrorCode::InvalidRequest)?;
                    wire::ReviewProjection {
                        title: "Tushare 日线行情".into(),
                        summary: format!(
                            "只读查询 {} 在 {} 的公开未复权日线行情，最多返回 1 行。本次仅执行一次外部请求。",
                            args.ts_code, args.trade_date
                        ),
                        risk: "read".into(),
                        arguments_json: None,
                        schema_digest: None,
                    }
                }
                ProviderBackend::Generic(backend) => backend
                    .review(tool, args)
                    .map_err(|_| wire::ErrorCode::InvalidRequest)?,
            };
            (
                provider.binding.service_id.clone(),
                provider.binding.reference.clone(),
                review,
            )
        } else {
            if !lease.providers.is_empty()
                || !self.mode.permits(&lease.view.snapshot.selection)
                || lease.view.snapshot.selection.is_empty()
                || tool != "lookup"
            {
                return Err(wire::ErrorCode::NotQualified);
            }
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or(wire::ErrorCode::InvalidRequest)?;
            if args.len() != 1 || query.is_empty() || query.chars().count() > 256 {
                return Err(wire::ErrorCode::InvalidRequest);
            }
            ("market-qualification".into(),lease.view.snapshot.selection[0].clone(),wire::ReviewProjection{title:"Local synthetic lookup".into(),summary:"Read the public value from the in-process qualification service. No external account or provider is contacted.".into(),risk:"read".into(),arguments_json:None,schema_digest:None})
        };
        review
            .validate()
            .map_err(|_| wire::ErrorCode::InvalidRequest)?;
        let encoded =
            serde_json::to_vec(&arguments).map_err(|_| wire::ErrorCode::InvalidRequest)?;
        if encoded.len() > wire::MAX_ARGUMENTS_BYTES {
            return Err(wire::ErrorCode::InvalidRequest);
        }
        if self.calls.len() >= MAX_CALLS {
            return Err(wire::ErrorCode::CapacityExceeded);
        }
        let mut digest = Sha256::new();
        digest.update(b"yijie.market-arguments/worker-json-v1\n");
        digest.update(encoded);
        let call_ref = Uuid::new_v4().to_string();
        let deadline = lease.deadline.min(now + wire::MAX_APPROVAL_TTL_MS as u64);
        let call = wire::PendingCall {
            identity: wire::CallIdentity {
                binding: lease.view.binding.clone(),
                native_thread_id: thread.into(),
                native_turn_id: turn.into(),
                call_ref: call_ref.clone(),
                service_id: service,
                reference: selected,
                tool_name: tool.into(),
                args_digest: format!("{:x}", digest.finalize()),
                args_encoding: "worker-json-v1".into(),
            },
            revision: 1,
            state: wire::CallState::Pending,
            remaining_ttl_ms: (deadline - now) as i64,
            review,
            decision_id: None,
            decision: None,
        };
        self.calls.insert(
            call_ref,
            CallRecord {
                view: call.clone(),
                deadline,
                arguments,
            },
        );
        Ok(call)
    }
    pub fn pending_call_view(&mut self, reference: &str) -> Option<wire::CallState> {
        if self.refresh().is_err() {
            return None;
        }
        self.calls.get(reference).map(|call| call.view.state)
    }
    pub fn consume_execution(
        &mut self,
        call_ref: &str,
        native_accepted: bool,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> Result<Execution, wire::ErrorCode> {
        self.require_running()?;
        let now = self.refresh()?;
        let call = self.calls.get(call_ref).ok_or(wire::ErrorCode::NotFound)?;
        self.live_lease(&call.view.identity.binding)?;
        if !native_accepted
            || cancellation.is_cancelled()
            || !matches!(call.view.state, wire::CallState::Approved)
        {
            if matches!(
                call.view.state,
                wire::CallState::Pending | wire::CallState::Approved
            ) {
                let call = self.calls.get_mut(call_ref).expect("checked call");
                call.view.state = wire::CallState::Cancelled;
                call.view.revision += 1;
                call.view.remaining_ttl_ms = 0;
                call.arguments = None;
            }
            return Err(wire::ErrorCode::ApprovalInvalid);
        }
        let lease = &self.leases[&call.view.identity.binding.capability_ref];
        let provider = lease
            .providers
            .iter()
            .find(|p| {
                p.binding.reference == call.view.identity.reference
                    && p.binding.service_id == call.view.identity.service_id
                    && p.backend.contains(&call.view.identity.tool_name)
            })
            .map(|p| p.backend.clone());
        if provider.is_none() && matches!(self.mode, ProviderMode::Product) {
            return Err(wire::ErrorCode::NotQualified);
        }
        let tool = call.view.identity.tool_name.clone();
        let lease_cancel = lease.cancelled.clone();
        let deadline = Instant::now() + Duration::from_millis(call.deadline.saturating_sub(now));
        let call = self.calls.get_mut(call_ref).expect("checked call");
        call.view.state = wire::CallState::Consumed;
        call.view.revision += 1;
        call.view.remaining_ttl_ms = 0;
        let arguments = call.arguments.take().expect("frozen qualified arguments");
        if let Some(ProviderBackend::Daily(backend)) = provider {
            let arguments = serde_json::from_value(Value::Object(arguments))
                .map_err(|_| wire::ErrorCode::InvalidRequest)?;
            Ok(Execution::Daily {
                backend,
                arguments,
                lease: lease_cancel,
                deadline,
            })
        } else if let Some(ProviderBackend::Generic(backend)) = provider {
            Ok(Execution::Generic {
                backend,
                tool,
                arguments,
                lease: lease_cancel,
                deadline,
            })
        } else {
            Ok(Execution::Synthetic(
                serde_json::json!({"value":"public-synthetic-value","query":arguments["query"]}),
            ))
        }
    }
    #[cfg(all(test, feature = "broker-qualification"))]
    fn consume_call(
        &mut self,
        call_ref: &str,
        accepted: bool,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> Result<Value, wire::ErrorCode> {
        match self.consume_execution(call_ref, accepted, cancellation)? {
            Execution::Synthetic(value) => Ok(value),
            Execution::Daily { .. } | Execution::Generic { .. } => {
                Err(wire::ErrorCode::NotQualified)
            }
        }
    }
    fn handle(&mut self, bytes: &[u8]) -> Value {
        let value: Value = match serde_json::from_slice(bytes) {
            Ok(v) => v,
            Err(_) => return safe_error(None, wire::ErrorCode::InvalidRequest),
        };
        macro_rules! route {
            ($request:ty,$response:ident,$method:ident) => {{
                let request: $request = match serde_json::from_value(value) {
                    Ok(r) => r,
                    Err(_) => return safe_error(None, wire::ErrorCode::InvalidRequest),
                };
                let request_id = request.request_id;
                match self.$method(request.payload) {
                    Ok(data) => serde_json::to_value(wire::$response {
                        schema_version: 1,
                        request_id,
                        data,
                    })
                    .expect("generated response"),
                    Err(code) => safe_error(Some(request_id), code),
                }
            }};
        }
        match value.get("method").and_then(Value::as_str) {
            Some("initialize") => route!(wire::InitializeRequest, InitializeResponse, initialize),
            Some("prepare") => route!(wire::PrepareRequest, PrepareResponse, prepare),
            Some("bind_turn") => route!(wire::BindTurnRequest, BindTurnResponse, bind_turn),
            Some("revoke") => route!(wire::RevokeRequest, RevokeResponse, revoke),
            Some("status") => route!(wire::StatusRequest, StatusResponse, status),
            Some("pending_call") => {
                route!(wire::PendingCallRequest, PendingCallResponse, pending_call)
            }
            Some("decide_call") => route!(wire::DecideCallRequest, DecideCallResponse, decide_call),
            Some("shutdown") => route!(wire::ShutdownRequest, ShutdownResponse, shutdown),
            _ => safe_error(None, wire::ErrorCode::InvalidRequest),
        }
    }
}
pub fn safe_error(request_id: Option<String>, code: wire::ErrorCode) -> Value {
    serde_json::to_value(wire::Error {
        schema_version: 1,
        request_id,
        code,
        retryable: false,
    })
    .expect("generated error")
}
pub const UNBOUND_WAIT: Duration = Duration::from_millis(wire::MAX_UNBOUND_WAIT_MS as u64);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    struct TestClock {
        wall: AtomicU64,
        tick: AtomicU64,
    }
    impl Clock for TestClock {
        fn now(&self) -> Result<(u64, u64), wire::ErrorCode> {
            Ok((
                self.wall.load(Ordering::SeqCst),
                self.tick.load(Ordering::SeqCst),
            ))
        }
    }
    fn fixture(mode: ProviderMode) -> (Broker, Arc<TestClock>, wire::ContextBinding) {
        let clock = Arc::new(TestClock {
            wall: AtomicU64::new(1_800_000_000_000),
            tick: AtomicU64::new(0),
        });
        let process = wire::ProcessBinding {
            host_instance_id: Uuid::new_v4().to_string(),
            runtime_generation: Uuid::new_v4().to_string(),
        };
        let mut broker = Broker::new(mode, "http://127.0.0.1:12345/mcp".into(), clock.clone());
        broker
            .initialize(wire::InitializePayload {
                operation_id: Uuid::new_v4().to_string(),
                process: process.clone(),
            })
            .unwrap();
        let context = wire::ContextBinding {
            process,
            scope: wire::ScopeBinding {
                owner_user_id: Uuid::new_v4().to_string(),
                tenant_id: Uuid::new_v4().to_string(),
                native_process_epoch: Uuid::new_v4().to_string(),
                authorization_revision: 1,
                authorization_expires_at_unix_ms: 1_800_000_300_000,
            },
            agent_session_id: Uuid::new_v4().to_string(),
            native_thread_id: Some("native-thread-1".into()),
        };
        (broker, clock, context)
    }
    fn prepare(context: wire::ContextBinding, selected: bool) -> wire::PreparePayload {
        let refs = if selected {
            vec![crate::generated::SelectionRef {
                installation_id: "11111111-1111-4111-8111-111111111111".into(),
                revision: 1,
                generation: 1,
            }]
        } else {
            vec![]
        };
        wire::PreparePayload {
            operation_id: Uuid::new_v4().to_string(),
            context,
            snapshot: crate::selection_generated::freeze_selection(
                Uuid::new_v4().to_string(),
                refs,
            )
            .unwrap(),
            ttl_ms: 120_000,
        }
    }
    fn bind(broker: &mut Broker, lease: &wire::Lease) -> wire::Lease {
        broker
            .bind_turn(wire::BindTurnPayload {
                operation_id: Uuid::new_v4().to_string(),
                binding: lease.binding.clone(),
                expected_revision: lease.revision,
                native_thread_id: "native-thread-1".into(),
                native_turn_id: "native-turn-1".into(),
            })
            .unwrap()
    }
    #[test]
    fn ordinary_empty_selection_lease_replay_preserves_deadline_and_digest() {
        let (mut broker, clock, context) = fixture(ProviderMode::Product);
        let request = prepare(context, false);
        let lease = broker.prepare(request.clone()).unwrap();
        clock.tick.store(10_000, Ordering::SeqCst);
        let replay = broker.prepare(request).unwrap();
        assert_eq!(lease.binding, replay.binding);
        assert_eq!(replay.remaining_ttl_ms, 110_000);
        assert!(!broker.tools_visible(&lease.binding.capability_ref).unwrap());
        let bound = bind(&mut broker, &lease);
        assert_eq!(bound.native_turn_id.as_deref(), Some("native-turn-1"));
    }
    #[test]
    fn first_thread_can_be_prepared_before_actual_native_ids_exist() {
        let (mut broker, _, mut context) = fixture(ProviderMode::Product);
        context.native_thread_id = None;
        let request = prepare(context, false);
        let lease = broker.prepare(request.clone()).unwrap();
        assert!(lease.bound_native_thread_id.is_none());
        assert!(lease.native_turn_id.is_none());
        let bound = bind(&mut broker, &lease);
        assert_eq!(bound.binding, lease.binding);
        assert!(bound.binding.context.native_thread_id.is_none());
        assert_eq!(
            bound.bound_native_thread_id.as_deref(),
            Some("native-thread-1")
        );
        assert_eq!(bound.native_turn_id.as_deref(), Some("native-turn-1"));
        let replay = broker.prepare(request).unwrap();
        assert_eq!(replay.binding, lease.binding);
    }
    #[test]
    fn existing_thread_constraint_and_single_live_bound_turn_are_enforced() {
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        let lease = broker.prepare(prepare(context.clone(), false)).unwrap();
        assert!(matches!(
            broker.bind_turn(wire::BindTurnPayload {
                operation_id: Uuid::new_v4().to_string(),
                binding: lease.binding.clone(),
                expected_revision: lease.revision,
                native_thread_id: "another-native-thread".into(),
                native_turn_id: "native-turn-1".into(),
            }),
            Err(wire::ErrorCode::TurnMismatch)
        ));
        bind(&mut broker, &lease);
        let mut new_context = context;
        new_context.native_thread_id = None;
        let new_lease = broker.prepare(prepare(new_context, false)).unwrap();
        assert!(matches!(
            broker.bind_turn(wire::BindTurnPayload {
                operation_id: Uuid::new_v4().to_string(),
                binding: new_lease.binding,
                expected_revision: new_lease.revision,
                native_thread_id: "native-thread-1".into(),
                native_turn_id: "native-turn-2".into(),
            }),
            Err(wire::ErrorCode::RequestConflict)
        ));
    }
    #[tokio::test]
    async fn ordinary_daily_binding_requires_both_decisions_and_keeps_original_lease() {
        let (backend, calls, factory) = crate::daily::tests::local_backend().await;
        let (mut broker, clock, context) = fixture(ProviderMode::Product);
        let request = prepare(context.clone(), true);
        let provider = crate::provider_generated::ProviderPayload {
            operation_id: Uuid::new_v4().to_string(),
            host_instance_id: context.process.host_instance_id.clone(),
            scope: context.scope.clone(),
            binding: crate::provider_generated::ProviderBinding {
                reference: request.snapshot.selection[0].clone(),
                service_id: "tushareMcp".into(),
                credential_ref: Uuid::new_v4().to_string(),
            },
        };
        // This internal ordinary fixture proves state transitions, not a real
        // provider's schema qualification or external account readiness.
        assert!(broker.register_daily(provider.clone(), backend.clone()));
        let lease = broker.prepare(request).unwrap();
        let lease = bind(&mut broker, &lease);
        let mut renewed = provider.clone();
        renewed.scope.authorization_expires_at_unix_ms += 30_000;
        assert!(broker.daily_status(&renewed));
        assert_eq!(broker.leases[&lease.binding.capability_ref].view, lease);
        let args = Some(
            serde_json::json!({"ts_code":"000001.SZ","trade_date":"20260105"})
                .as_object()
                .unwrap()
                .clone(),
        );
        let call = broker
            .register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                crate::provider_generated::TUSHARE_DAILY_TOOL_NAME,
                args.clone(),
            )
            .unwrap();
        assert!(matches!(
            broker.consume_execution(
                &call.identity.call_ref,
                true,
                &tokio_util::sync::CancellationToken::new()
            ),
            Err(wire::ErrorCode::ApprovalInvalid)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let rejected = broker
            .register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                crate::provider_generated::TUSHARE_DAILY_TOOL_NAME,
                args.clone(),
            )
            .unwrap();
        broker
            .decide_call(wire::DecideCallPayload {
                operation_id: Uuid::new_v4().to_string(),
                identity: rejected.identity.clone(),
                expected_revision: 1,
                approval_ref: Uuid::new_v4().to_string(),
                decision_id: Uuid::new_v4().to_string(),
                decision: wire::Decision::Reject,
            })
            .unwrap();
        assert!(
            broker
                .consume_execution(
                    &rejected.identity.call_ref,
                    true,
                    &tokio_util::sync::CancellationToken::new()
                )
                .is_err()
        );
        assert_eq!(
            broker.pending_call_view(&rejected.identity.call_ref),
            Some(wire::CallState::Rejected)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let call = broker
            .register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                crate::provider_generated::TUSHARE_DAILY_TOOL_NAME,
                args,
            )
            .unwrap();
        broker
            .decide_call(wire::DecideCallPayload {
                operation_id: Uuid::new_v4().to_string(),
                identity: call.identity.clone(),
                expected_revision: 1,
                approval_ref: Uuid::new_v4().to_string(),
                decision_id: Uuid::new_v4().to_string(),
                decision: wire::Decision::ApproveOnce,
            })
            .unwrap();
        let execution = broker
            .consume_execution(
                &call.identity.call_ref,
                true,
                &tokio_util::sync::CancellationToken::new(),
            )
            .unwrap();
        let Execution::Daily {
            backend: executor,
            arguments,
            lease: cancel,
            deadline,
        } = execution
        else {
            panic!("expected daily")
        };
        let result = executor
            .execute(
                arguments,
                tokio_util::sync::CancellationToken::new(),
                cancel,
                deadline,
                crate::tushare_oauth::Diagnostic::default(),
            )
            .await
            .unwrap();
        assert_eq!(result["rows"][0]["ts_code"], "000001.SZ");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            broker.consume_execution(
                &call.identity.call_ref,
                true,
                &tokio_util::sync::CancellationToken::new()
            ),
            Err(wire::ErrorCode::ApprovalInvalid)
        ));
        clock.tick.store(310_000, Ordering::SeqCst);
        clock.wall.store(1_800_000_310_000, Ordering::SeqCst);
        renewed.scope.authorization_expires_at_unix_ms = 1_800_000_610_000;
        assert!(broker.daily_status(&renewed));
        assert!(
            !broker
                .tools_visible(&lease.binding.capability_ref)
                .unwrap_or(false)
        );
        broker.invalidate_installation(
            &context.scope.owner_user_id,
            &context.scope.tenant_id,
            &provider.binding.reference.installation_id,
        );
        assert!(!broker.daily_status(&renewed));
        backend.close().await;
        factory.join().await;
    }
    #[tokio::test]
    async fn sorftime_market_call_binds_exact_ref_and_requires_a_fresh_decision() {
        let (a, a_calls, a_factory) = crate::generic::tests::local_backend("lingxing").await;
        let (b, b_calls, b_factory) = crate::generic::tests::local_backend("sorftime").await;
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        let mut request = prepare(context.clone(), true);
        let mut refs = request.snapshot.selection.clone();
        let mut second = refs[0].clone();
        second.installation_id = Uuid::new_v4().to_string();
        refs.push(second.clone());
        request.snapshot =
            crate::selection_generated::freeze_selection(Uuid::new_v4().to_string(), refs.clone())
                .unwrap();
        for (backend, reference) in [(a.clone(), refs[0].clone()), (b.clone(), second.clone())] {
            assert!(broker.register_provider(
                crate::provider_generated::ProviderPayload {
                    operation_id: Uuid::new_v4().to_string(),
                    host_instance_id: context.process.host_instance_id.clone(),
                    scope: context.scope.clone(),
                    binding: crate::provider_generated::ProviderBinding {
                        reference,
                        service_id: backend.service.clone(),
                        credential_ref: Uuid::new_v4().to_string()
                    },
                },
                ProviderBackend::Generic(backend)
            ));
        }
        let prepared = broker.prepare(request).unwrap();
        let lease = bind(&mut broker, &prepared);
        let tools = broker
            .visible_tools(&lease.binding.capability_ref)
            .unwrap()
            .unwrap();
        assert_eq!(tools.len(), 2);
        assert_ne!(tools[0].name, tools[1].name);
        let tool = b.definitions[0].tool.name.to_string();
        let args = serde_json::json!({"query":"普通本地查询"})
            .as_object()
            .unwrap()
            .clone();
        let cancellation = tokio_util::sync::CancellationToken::new();
        for decision in [wire::Decision::Reject, wire::Decision::ApproveOnce] {
            let call = broker
                .register_call(
                    &lease.binding.capability_ref,
                    "native-thread-1",
                    "native-turn-1",
                    &tool,
                    Some(args.clone()),
                )
                .unwrap();
            assert_eq!(call.identity.reference, second);
            assert_eq!(call.identity.service_id, "sorftime");
            assert_eq!(call.review.risk, "write");
            assert_eq!(
                call.review.arguments_json.as_deref(),
                Some(serde_json::to_string(&args).unwrap().as_str())
            );
            assert!(call.review.schema_digest.is_some());
            broker
                .decide_call(wire::DecideCallPayload {
                    operation_id: Uuid::new_v4().to_string(),
                    identity: call.identity.clone(),
                    expected_revision: 1,
                    approval_ref: Uuid::new_v4().to_string(),
                    decision_id: Uuid::new_v4().to_string(),
                    decision,
                })
                .unwrap();
            let execution = broker.consume_execution(&call.identity.call_ref, true, &cancellation);
            if let Ok(Execution::Generic {
                backend,
                tool,
                arguments,
                lease,
                deadline,
            }) = execution
            {
                let result = backend
                    .execute(
                        &tool,
                        arguments,
                        crate::daily::Permit {
                            tool: String::new(),
                            arguments: Value::Null,
                            request: cancellation.clone(),
                            lease,
                            backend: tokio_util::sync::CancellationToken::new(),
                            deadline,
                        },
                        crate::tushare_oauth::Diagnostic::default(),
                    )
                    .await
                    .unwrap();
                assert_eq!(result.structured_content.unwrap()["count"], 1);
                assert_eq!(b_calls.load(Ordering::SeqCst), 1);
            } else {
                assert_eq!(b_calls.load(Ordering::SeqCst), 0);
            }
            assert!(
                broker
                    .consume_execution(&call.identity.call_ref, true, &cancellation)
                    .is_err()
            );
        }
        assert_eq!(a_calls.load(Ordering::SeqCst), 0);
        assert_eq!(b_calls.load(Ordering::SeqCst), 1);
        a.close().await;
        b.close().await;
        a_factory.join().await;
        b_factory.join().await;
    }
    #[test]
    fn real_provider_refs_remain_unqualified() {
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        assert!(matches!(
            broker.prepare(prepare(context, true)),
            Err(wire::ErrorCode::NotQualified)
        ));
        assert!(broker.leases.is_empty());
    }
    #[test]
    fn authorization_renewal_does_not_recreate_expired_original_turn() {
        let (mut broker, clock, context) = fixture(ProviderMode::Product);
        let mut request = prepare(context, false);
        request.ttl_ms = 1000;
        broker.prepare(request.clone()).unwrap();
        clock.tick.store(1001, Ordering::SeqCst);
        request.operation_id = Uuid::new_v4().to_string();
        request.context.scope.authorization_revision += 1;
        request.context.scope.authorization_expires_at_unix_ms += 300_000;
        assert!(matches!(
            broker.prepare(request),
            Err(wire::ErrorCode::LeaseExpired)
        ));
        assert_eq!(broker.leases.len(), 1);
    }
    #[test]
    fn authorization_expiry_clamps_monotonic_deadline() {
        let (mut broker, clock, mut context) = fixture(ProviderMode::Product);
        context.scope.authorization_expires_at_unix_ms = 1_800_000_000_020;
        let lease = broker.prepare(prepare(context, false)).unwrap();
        assert_eq!(lease.remaining_ttl_ms, 20);
        clock.wall.store(1_700_000_000_000, Ordering::SeqCst);
        clock.tick.store(21, Ordering::SeqCst);
        let status = broker
            .status(wire::StatusPayload {
                binding: lease.binding,
            })
            .unwrap();
        assert!(matches!(status.state, wire::LeaseState::Expired));
    }
    #[test]
    fn same_mutation_changed_intent_conflicts_and_new_turn_requires_revoke() {
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        let request = prepare(context.clone(), false);
        broker.prepare(request.clone()).unwrap();
        let mut changed = request;
        changed.ttl_ms += 1;
        assert!(matches!(
            broker.prepare(changed),
            Err(wire::ErrorCode::RequestConflict)
        ));
        assert!(matches!(
            broker.prepare(prepare(context, false)),
            Err(wire::ErrorCode::RequestConflict)
        ));
    }
    #[test]
    fn owner_stop_revokes_admission_and_never_respawns() {
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        let request = prepare(context, false);
        let lease = broker.prepare(request.clone()).unwrap();
        broker.stop();
        assert!(matches!(
            broker.prepare(request),
            Err(wire::ErrorCode::LeaseRevoked)
        ));
        assert!(matches!(
            broker
                .status(wire::StatusPayload {
                    binding: lease.binding
                })
                .unwrap()
                .state,
            wire::LeaseState::Revoked
        ));
        assert!(matches!(
            broker.initialize(wire::InitializePayload {
                operation_id: Uuid::new_v4().to_string(),
                process: broker.process.clone().unwrap()
            }),
            Err(wire::ErrorCode::Stopping)
        ));
    }
    #[test]
    fn read_only_query_cannot_create_pending_call() {
        let (mut broker, _, context) = fixture(ProviderMode::Product);
        let lease = broker.prepare(prepare(context, false)).unwrap();
        let lease = bind(&mut broker, &lease);
        assert!(matches!(
            broker.pending_call(wire::PendingCallPayload {
                binding: lease.binding,
                native_thread_id: "native-thread-1".into(),
                native_turn_id: "native-turn-1".into(),
                call_ref: Uuid::new_v4().to_string()
            }),
            Err(wire::ErrorCode::NotFound)
        ));
        assert!(broker.calls.is_empty());
    }
    #[cfg(feature = "broker-qualification")]
    fn register(broker: &mut Broker, lease: &wire::Lease) -> wire::PendingCall {
        broker
            .register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                "lookup",
                Some(
                    serde_json::json!({"query":"public-synthetic-record"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .unwrap()
    }
    #[cfg(feature = "broker-qualification")]
    fn approve(broker: &mut Broker, call: &wire::PendingCall) -> wire::DecideCallPayload {
        let payload = wire::DecideCallPayload {
            operation_id: Uuid::new_v4().to_string(),
            identity: call.identity.clone(),
            expected_revision: call.revision,
            approval_ref: Uuid::new_v4().to_string(),
            decision_id: Uuid::new_v4().to_string(),
            decision: wire::Decision::ApproveOnce,
        };
        broker.decide_call(payload.clone()).unwrap();
        payload
    }
    #[test]
    #[cfg(feature = "broker-qualification")]
    fn actual_registered_call_requires_both_decisions_and_is_consumed_once() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let lease = bind(&mut broker, &lease);
        let call = register(&mut broker, &lease);
        let payload = approve(&mut broker, &call);
        assert_eq!(
            broker
                .consume_call(
                    &call.identity.call_ref,
                    true,
                    &tokio_util::sync::CancellationToken::new()
                )
                .unwrap()["value"],
            "public-synthetic-value"
        );
        assert!(broker.calls[&call.identity.call_ref].arguments.is_none());
        assert!(matches!(
            broker.consume_call(
                &call.identity.call_ref,
                true,
                &tokio_util::sync::CancellationToken::new()
            ),
            Err(wire::ErrorCode::ApprovalInvalid)
        ));
        let receipt = broker.decide_call(payload).unwrap();
        assert!(matches!(receipt.state, wire::CallState::Approved));
        assert!(matches!(
            broker.calls[&call.identity.call_ref].view.state,
            wire::CallState::Consumed
        ));
    }
    #[test]
    #[cfg(feature = "broker-qualification")]
    fn native_decline_cancels_owner_approval_and_releases_arguments() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let lease = bind(&mut broker, &lease);
        let call = register(&mut broker, &lease);
        approve(&mut broker, &call);
        assert!(
            broker
                .consume_call(
                    &call.identity.call_ref,
                    false,
                    &tokio_util::sync::CancellationToken::new()
                )
                .is_err()
        );
        assert!(matches!(
            broker.calls[&call.identity.call_ref].view.state,
            wire::CallState::Cancelled
        ));
        assert!(broker.calls[&call.identity.call_ref].arguments.is_none());
    }
    #[test]
    #[cfg(feature = "broker-qualification")]
    fn revoke_after_approval_prevents_admission_and_retains_call_identity() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let lease = bind(&mut broker, &lease);
        let call = register(&mut broker, &lease);
        approve(&mut broker, &call);
        broker
            .revoke(wire::RevokePayload {
                operation_id: Uuid::new_v4().to_string(),
                binding: lease.binding,
                expected_revision: lease.revision,
                reason: wire::RevokeReason::TurnTerminal,
            })
            .unwrap();
        assert!(matches!(
            broker.consume_call(
                &call.identity.call_ref,
                true,
                &tokio_util::sync::CancellationToken::new()
            ),
            Err(wire::ErrorCode::LeaseRevoked)
        ));
        assert!(broker.calls[&call.identity.call_ref].arguments.is_none());
        assert_eq!(
            broker.calls[&call.identity.call_ref].view.identity,
            call.identity
        );
    }
    #[tokio::test]
    #[cfg(feature = "broker-qualification")]
    async fn ordinary_precancelled_gateway_request_does_not_register() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let lease = bind(&mut broker, &lease);
        let handle = BrokerHandle {
            state: Arc::new(Mutex::new(broker)),
            changed: Arc::new(Notify::new()),
        };
        let cancellation = tokio_util::sync::CancellationToken::new();
        cancellation.cancel();
        let request = rmcp::model::CallToolRequestParams::new("lookup").with_arguments(
            serde_json::json!({"query":"public-synthetic-record"})
                .as_object()
                .unwrap()
                .clone(),
        );
        assert!(
            crate::gateway::wait_for_bound_call(
                &handle,
                &cancellation,
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                &request
            )
            .await
            .is_err()
        );
        assert!(handle.state.lock().unwrap().calls.is_empty());
    }
    #[tokio::test]
    #[cfg(feature = "broker-qualification")]
    async fn ordinary_cancellation_ends_unbound_wait_before_late_bind() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let handle = BrokerHandle {
            state: Arc::new(Mutex::new(broker)),
            changed: Arc::new(Notify::new()),
        };
        let cancellation = tokio_util::sync::CancellationToken::new();
        let request = rmcp::model::CallToolRequestParams::new("lookup").with_arguments(
            serde_json::json!({"query":"public-synthetic-record"})
                .as_object()
                .unwrap()
                .clone(),
        );
        let pending = crate::gateway::wait_for_bound_call(
            &handle,
            &cancellation,
            &lease.binding.capability_ref,
            "native-thread-1",
            "native-turn-1",
            &request,
        );
        tokio::pin!(pending);
        tokio::select! {
            biased;
            _ = &mut pending => panic!("prepared request must wait for owner bind"),
            _ = tokio::task::yield_now() => {},
        }
        cancellation.cancel();
        assert!(pending.await.is_err());
        let mut state = handle.state.lock().unwrap();
        bind(&mut state, &lease);
        assert!(state.calls.is_empty());
    }
    #[test]
    #[cfg(feature = "broker-qualification")]
    fn cancellation_ready_with_accept_does_not_spend_approval() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        let lease = bind(&mut broker, &lease);
        let call = register(&mut broker, &lease);
        approve(&mut broker, &call);
        let cancellation = tokio_util::sync::CancellationToken::new();
        cancellation.cancel();
        assert!(
            broker
                .consume_call(&call.identity.call_ref, true, &cancellation)
                .is_err()
        );
        let record = &broker.calls[&call.identity.call_ref];
        assert_eq!(record.view.state, wire::CallState::Cancelled);
        assert_eq!(record.view.remaining_ttl_ms, 0);
        assert!(record.arguments.is_none());
    }
    #[test]
    #[cfg(feature = "broker-qualification")]
    fn call_waits_for_owner_bind_and_unknown_tool_allocates_nothing() {
        let (mut broker, _, context) = fixture(ProviderMode::Qualification);
        let lease = broker.prepare(prepare(context, true)).unwrap();
        assert!(matches!(
            broker.register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                "lookup",
                None
            ),
            Err(wire::ErrorCode::NotBound)
        ));
        let lease = bind(&mut broker, &lease);
        assert!(matches!(
            broker.register_call(
                &lease.binding.capability_ref,
                "native-thread-1",
                "native-turn-1",
                "unlisted",
                None
            ),
            Err(wire::ErrorCode::NotQualified)
        ));
        assert!(broker.calls.is_empty());
    }
}
