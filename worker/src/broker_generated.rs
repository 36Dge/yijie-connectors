// Generated from market-broker-control source and declared borrowed authorities; DO NOT EDIT.
use serde::{Deserialize, Serialize};

fn optional_non_null<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
fn required_nullable<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d)
}
pub use crate::generated::CanonicalId;
pub use crate::generated::Revision;
pub use crate::generated::SelectionRef;
pub use crate::generated::ServiceId;
pub use crate::selection_generated::SelectionDigest;
pub use crate::selection_generated::SelectionSnapshot;
pub type Milliseconds = i64;
pub type RequestedTtlMs = i64;
pub type UnixMilliseconds = i64;
pub type NativeId = String;
pub type ToolName = String;
pub type ArgsDigest = String;
pub type GatewayUrl = String;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseState {
    #[serde(rename = "prepared")]
    Prepared,
    #[serde(rename = "bound")]
    Bound,
    #[serde(rename = "revoked")]
    Revoked,
    #[serde(rename = "expired")]
    Expired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallState {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "consumed")]
    Consumed,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "revoked")]
    Revoked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    #[serde(rename = "approve_once")]
    ApproveOnce,
    #[serde(rename = "reject")]
    Reject,
    #[serde(rename = "cancel")]
    Cancel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevokeReason {
    #[serde(rename = "turn_terminal")]
    TurnTerminal,
    #[serde(rename = "user_cancelled")]
    UserCancelled,
    #[serde(rename = "selection_changed")]
    SelectionChanged,
    #[serde(rename = "authorization_changed")]
    AuthorizationChanged,
    #[serde(rename = "generation_closed")]
    GenerationClosed,
    #[serde(rename = "owner_shutdown")]
    OwnerShutdown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownReason {
    #[serde(rename = "owner_shutdown")]
    OwnerShutdown,
    #[serde(rename = "generation_closed")]
    GenerationClosed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "not_initialized")]
    NotInitialized,
    #[serde(rename = "already_initialized")]
    AlreadyInitialized,
    #[serde(rename = "binding_mismatch")]
    BindingMismatch,
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "request_conflict")]
    RequestConflict,
    #[serde(rename = "revision_conflict")]
    RevisionConflict,
    #[serde(rename = "scope_expired")]
    ScopeExpired,
    #[serde(rename = "lease_expired")]
    LeaseExpired,
    #[serde(rename = "lease_revoked")]
    LeaseRevoked,
    #[serde(rename = "turn_mismatch")]
    TurnMismatch,
    #[serde(rename = "not_bound")]
    NotBound,
    #[serde(rename = "not_qualified")]
    NotQualified,
    #[serde(rename = "capacity_exceeded")]
    CapacityExceeded,
    #[serde(rename = "call_expired")]
    CallExpired,
    #[serde(rename = "call_consumed")]
    CallConsumed,
    #[serde(rename = "approval_invalid")]
    ApprovalInvalid,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "temporarily_unavailable")]
    TemporarilyUnavailable,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProcessBindingWire")]
pub struct ProcessBinding {
    pub host_instance_id: CanonicalId,
    pub runtime_generation: CanonicalId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProcessBindingWire {
    pub host_instance_id: CanonicalId,
    pub runtime_generation: CanonicalId,
}
impl TryFrom<ProcessBindingWire> for ProcessBinding {
    type Error = &'static str;
    fn try_from(w: ProcessBindingWire) -> Result<Self, Self::Error> {
        let v = Self {
            host_instance_id: w.host_instance_id,
            runtime_generation: w.runtime_generation,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProcessBinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_host_instance_id = &self.host_instance_id;
            if !canonical_id(value_host_instance_id) {
                return Err("invalid broker UUID");
            }
            if value_host_instance_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_runtime_generation = &self.runtime_generation;
            if !canonical_id(value_runtime_generation) {
                return Err("invalid broker UUID");
            }
            if value_runtime_generation == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProcessBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProcessBinding([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ScopeBindingWire")]
pub struct ScopeBinding {
    pub owner_user_id: CanonicalId,
    pub tenant_id: CanonicalId,
    pub native_process_epoch: CanonicalId,
    pub authorization_revision: Revision,
    pub authorization_expires_at_unix_ms: UnixMilliseconds,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScopeBindingWire {
    pub owner_user_id: CanonicalId,
    pub tenant_id: CanonicalId,
    pub native_process_epoch: CanonicalId,
    pub authorization_revision: Revision,
    pub authorization_expires_at_unix_ms: UnixMilliseconds,
}
impl TryFrom<ScopeBindingWire> for ScopeBinding {
    type Error = &'static str;
    fn try_from(w: ScopeBindingWire) -> Result<Self, Self::Error> {
        let v = Self {
            owner_user_id: w.owner_user_id,
            tenant_id: w.tenant_id,
            native_process_epoch: w.native_process_epoch,
            authorization_revision: w.authorization_revision,
            authorization_expires_at_unix_ms: w.authorization_expires_at_unix_ms,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ScopeBinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_owner_user_id = &self.owner_user_id;
            if !canonical_id(value_owner_user_id) {
                return Err("invalid broker UUID");
            }
            if value_owner_user_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_tenant_id = &self.tenant_id;
            if !canonical_id(value_tenant_id) {
                return Err("invalid broker UUID");
            }
            if value_tenant_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_native_process_epoch = &self.native_process_epoch;
            if !canonical_id(value_native_process_epoch) {
                return Err("invalid broker UUID");
            }
            if value_native_process_epoch == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_authorization_revision = &self.authorization_revision;
            if *value_authorization_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_authorization_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        {
            let value_authorization_expires_at_unix_ms = &self.authorization_expires_at_unix_ms;
            if *value_authorization_expires_at_unix_ms < 1 {
                return Err("invalid broker number");
            }
            if *value_authorization_expires_at_unix_ms > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for ScopeBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScopeBinding([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ContextBindingWire")]
pub struct ContextBinding {
    pub process: ProcessBinding,
    pub scope: ScopeBinding,
    pub agent_session_id: CanonicalId,
    #[serde(deserialize_with = "required_nullable")]
    pub native_thread_id: Option<NativeId>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContextBindingWire {
    pub process: ProcessBinding,
    pub scope: ScopeBinding,
    pub agent_session_id: CanonicalId,
    #[serde(deserialize_with = "required_nullable")]
    pub native_thread_id: Option<NativeId>,
}
impl TryFrom<ContextBindingWire> for ContextBinding {
    type Error = &'static str;
    fn try_from(w: ContextBindingWire) -> Result<Self, Self::Error> {
        let v = Self {
            process: w.process,
            scope: w.scope,
            agent_session_id: w.agent_session_id,
            native_thread_id: w.native_thread_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ContextBinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_process = &self.process;
            value_process.validate()?;
        }
        {
            let value_scope = &self.scope;
            value_scope.validate()?;
        }
        {
            let value_agent_session_id = &self.agent_session_id;
            if !canonical_id(value_agent_session_id) {
                return Err("invalid broker UUID");
            }
            if value_agent_session_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_native_thread_id = &self.native_thread_id;
            value_native_thread_id
                .as_ref()
                .map(|value| -> Result<(), &'static str> {
                    if value.chars().count() < 1 {
                        return Err("invalid broker text");
                    }
                    if value.chars().count() > 256 {
                        return Err("invalid broker text");
                    }
                    Ok(())
                })
                .transpose()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ContextBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextBinding([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "CapabilityBindingWire")]
pub struct CapabilityBinding {
    pub context: ContextBinding,
    pub capability_ref: CanonicalId,
    pub turn_operation_id: CanonicalId,
    pub selection_digest: SelectionDigest,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilityBindingWire {
    pub context: ContextBinding,
    pub capability_ref: CanonicalId,
    pub turn_operation_id: CanonicalId,
    pub selection_digest: SelectionDigest,
}
impl TryFrom<CapabilityBindingWire> for CapabilityBinding {
    type Error = &'static str;
    fn try_from(w: CapabilityBindingWire) -> Result<Self, Self::Error> {
        let v = Self {
            context: w.context,
            capability_ref: w.capability_ref,
            turn_operation_id: w.turn_operation_id,
            selection_digest: w.selection_digest,
        };
        v.validate()?;
        Ok(v)
    }
}
impl CapabilityBinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_context = &self.context;
            value_context.validate()?;
        }
        {
            let value_capability_ref = &self.capability_ref;
            if !canonical_id(value_capability_ref) {
                return Err("invalid broker UUID");
            }
            if value_capability_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_turn_operation_id = &self.turn_operation_id;
            if !canonical_id(value_turn_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_turn_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_selection_digest = &self.selection_digest;
            if value_selection_digest.len() != 64
                || !value_selection_digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("invalid broker digest");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for CapabilityBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CapabilityBinding([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "LeaseWire")]
pub struct Lease {
    pub binding: CapabilityBinding,
    pub snapshot: SelectionSnapshot,
    pub revision: Revision,
    pub state: LeaseState,
    pub remaining_ttl_ms: Milliseconds,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub native_turn_id: Option<NativeId>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub bound_native_thread_id: Option<NativeId>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaseWire {
    pub binding: CapabilityBinding,
    pub snapshot: SelectionSnapshot,
    pub revision: Revision,
    pub state: LeaseState,
    pub remaining_ttl_ms: Milliseconds,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub native_turn_id: Option<NativeId>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub bound_native_thread_id: Option<NativeId>,
}
impl TryFrom<LeaseWire> for Lease {
    type Error = &'static str;
    fn try_from(w: LeaseWire) -> Result<Self, Self::Error> {
        let v = Self {
            binding: w.binding,
            snapshot: w.snapshot,
            revision: w.revision,
            state: w.state,
            remaining_ttl_ms: w.remaining_ttl_ms,
            native_turn_id: w.native_turn_id,
            bound_native_thread_id: w.bound_native_thread_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl Lease {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        {
            let value_snapshot = &self.snapshot;
            value_snapshot.validate()?;
        }
        {
            let value_revision = &self.revision;
            if *value_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        {
            let value_remaining_ttl_ms = &self.remaining_ttl_ms;
            if *value_remaining_ttl_ms < 0 {
                return Err("invalid broker number");
            }
            if *value_remaining_ttl_ms > 300000 {
                return Err("invalid broker number");
            }
        }
        self.native_turn_id
            .as_ref()
            .map(|value_native_turn_id| -> Result<(), &'static str> {
                if value_native_turn_id.chars().count() < 1 {
                    return Err("invalid broker text");
                }
                if value_native_turn_id.chars().count() > 256 {
                    return Err("invalid broker text");
                }
                Ok(())
            })
            .transpose()?;
        self.bound_native_thread_id
            .as_ref()
            .map(|value_bound_native_thread_id| -> Result<(), &'static str> {
                if value_bound_native_thread_id.chars().count() < 1 {
                    return Err("invalid broker text");
                }
                if value_bound_native_thread_id.chars().count() > 256 {
                    return Err("invalid broker text");
                }
                Ok(())
            })
            .transpose()?;
        if self.snapshot.turn_operation_id != self.binding.turn_operation_id
            || self.snapshot.selection_digest != self.binding.selection_digest
        {
            return Err("lease snapshot binding mismatch");
        }
        if self.native_turn_id.is_some() != self.bound_native_thread_id.is_some()
            || (self.state == LeaseState::Prepared && self.native_turn_id.is_some())
            || (self.state == LeaseState::Bound && self.native_turn_id.is_none())
        {
            return Err("invalid lease turn binding");
        }
        if self
            .binding
            .context
            .native_thread_id
            .as_ref()
            .zip(self.bound_native_thread_id.as_ref())
            .is_some_and(|(expected, actual)| expected != actual)
        {
            return Err("lease thread binding mismatch");
        }
        Ok(())
    }
}
impl std::fmt::Debug for Lease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Lease([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ReviewProjectionWire")]
pub struct ReviewProjection {
    pub title: String,
    pub summary: String,
    pub risk: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub arguments_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub schema_digest: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReviewProjectionWire {
    pub title: String,
    pub summary: String,
    pub risk: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub arguments_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub schema_digest: Option<String>,
}
impl TryFrom<ReviewProjectionWire> for ReviewProjection {
    type Error = &'static str;
    fn try_from(w: ReviewProjectionWire) -> Result<Self, Self::Error> {
        let v = Self {
            title: w.title,
            summary: w.summary,
            risk: w.risk,
            arguments_json: w.arguments_json,
            schema_digest: w.schema_digest,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ReviewProjection {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_title = &self.title;
            if value_title.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_title.chars().count() > 160 {
                return Err("invalid broker text");
            }
        }
        {
            let value_summary = &self.summary;
            if value_summary.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_summary.chars().count() > 2048 {
                return Err("invalid broker text");
            }
        }
        {
            let value_risk = &self.risk;
            if !["read", "write"].contains(&value_risk.as_str()) {
                return Err("invalid broker enum");
            }
        }
        self.arguments_json
            .as_ref()
            .map(|value_arguments_json| -> Result<(), &'static str> {
                if value_arguments_json.chars().count() < 2 {
                    return Err("invalid broker text");
                }
                if value_arguments_json.chars().count() > 16384 {
                    return Err("invalid broker text");
                }
                Ok(())
            })
            .transpose()?;
        self.schema_digest
            .as_ref()
            .map(|value_schema_digest| -> Result<(), &'static str> {
                if value_schema_digest.len() != 64
                    || !value_schema_digest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err("invalid broker digest");
                }
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for ReviewProjection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReviewProjection([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "CallIdentityWire")]
pub struct CallIdentity {
    pub binding: CapabilityBinding,
    pub native_turn_id: NativeId,
    pub call_ref: CanonicalId,
    pub service_id: ServiceId,
    pub reference: SelectionRef,
    pub tool_name: ToolName,
    pub args_digest: ArgsDigest,
    pub args_encoding: String,
    pub native_thread_id: NativeId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CallIdentityWire {
    pub binding: CapabilityBinding,
    pub native_turn_id: NativeId,
    pub call_ref: CanonicalId,
    pub service_id: ServiceId,
    pub reference: SelectionRef,
    pub tool_name: ToolName,
    pub args_digest: ArgsDigest,
    pub args_encoding: String,
    pub native_thread_id: NativeId,
}
impl TryFrom<CallIdentityWire> for CallIdentity {
    type Error = &'static str;
    fn try_from(w: CallIdentityWire) -> Result<Self, Self::Error> {
        let v = Self {
            binding: w.binding,
            native_turn_id: w.native_turn_id,
            call_ref: w.call_ref,
            service_id: w.service_id,
            reference: w.reference,
            tool_name: w.tool_name,
            args_digest: w.args_digest,
            args_encoding: w.args_encoding,
            native_thread_id: w.native_thread_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl CallIdentity {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        {
            let value_native_turn_id = &self.native_turn_id;
            if value_native_turn_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_turn_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        {
            let value_call_ref = &self.call_ref;
            if !canonical_id(value_call_ref) {
                return Err("invalid broker UUID");
            }
            if value_call_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid broker text");
            }
            if value_service_id.is_empty()
                || !value_service_id.as_bytes()[0].is_ascii_alphanumeric()
                || !value_service_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                return Err("invalid broker service ID");
            }
        }
        {
            let value_reference = &self.reference;
            value_reference.validate()?;
        }
        {
            let value_tool_name = &self.tool_name;
            if value_tool_name.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_tool_name.chars().count() > 128 {
                return Err("invalid broker text");
            }
        }
        {
            let value_args_digest = &self.args_digest;
            if value_args_digest.len() != 64
                || !value_args_digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("invalid broker digest");
            }
        }
        {
            let value_args_encoding = &self.args_encoding;
            if value_args_encoding != "worker-json-v1" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_native_thread_id = &self.native_thread_id;
            if value_native_thread_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_thread_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for CallIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CallIdentity([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PendingCallWire")]
pub struct PendingCall {
    pub identity: CallIdentity,
    pub revision: Revision,
    pub state: CallState,
    pub remaining_ttl_ms: Milliseconds,
    pub review: ReviewProjection,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub decision_id: Option<CanonicalId>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub decision: Option<Decision>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingCallWire {
    pub identity: CallIdentity,
    pub revision: Revision,
    pub state: CallState,
    pub remaining_ttl_ms: Milliseconds,
    pub review: ReviewProjection,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub decision_id: Option<CanonicalId>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub decision: Option<Decision>,
}
impl TryFrom<PendingCallWire> for PendingCall {
    type Error = &'static str;
    fn try_from(w: PendingCallWire) -> Result<Self, Self::Error> {
        let v = Self {
            identity: w.identity,
            revision: w.revision,
            state: w.state,
            remaining_ttl_ms: w.remaining_ttl_ms,
            review: w.review,
            decision_id: w.decision_id,
            decision: w.decision,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PendingCall {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_identity = &self.identity;
            value_identity.validate()?;
        }
        {
            let value_revision = &self.revision;
            if *value_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        {
            let value_remaining_ttl_ms = &self.remaining_ttl_ms;
            if *value_remaining_ttl_ms < 0 {
                return Err("invalid broker number");
            }
            if *value_remaining_ttl_ms > 300000 {
                return Err("invalid broker number");
            }
        }
        {
            let value_review = &self.review;
            value_review.validate()?;
        }
        self.decision_id
            .as_ref()
            .map(|value_decision_id| -> Result<(), &'static str> {
                if !canonical_id(value_decision_id) {
                    return Err("invalid broker UUID");
                }
                if value_decision_id == "00000000-0000-0000-0000-000000000000" {
                    return Err("invalid broker identity");
                }
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for PendingCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingCall([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InitializePayloadWire")]
pub struct InitializePayload {
    pub operation_id: CanonicalId,
    pub process: ProcessBinding,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InitializePayloadWire {
    pub operation_id: CanonicalId,
    pub process: ProcessBinding,
}
impl TryFrom<InitializePayloadWire> for InitializePayload {
    type Error = &'static str;
    fn try_from(w: InitializePayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            process: w.process,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InitializePayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_process = &self.process;
            value_process.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for InitializePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InitializePayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InitializeRequestWire")]
pub struct InitializeRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: InitializePayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InitializeRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: InitializePayload,
}
impl TryFrom<InitializeRequestWire> for InitializeRequest {
    type Error = &'static str;
    fn try_from(w: InitializeRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InitializeRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "initialize" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for InitializeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InitializeRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InitializeResultWire")]
pub struct InitializeResult {
    pub worker_instance_id: CanonicalId,
    pub process: ProcessBinding,
    pub gateway_url: GatewayUrl,
    pub external_calls_enabled: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeResultWire {
    pub worker_instance_id: CanonicalId,
    pub process: ProcessBinding,
    pub gateway_url: GatewayUrl,
    pub external_calls_enabled: bool,
}
impl TryFrom<InitializeResultWire> for InitializeResult {
    type Error = &'static str;
    fn try_from(w: InitializeResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            worker_instance_id: w.worker_instance_id,
            process: w.process,
            gateway_url: w.gateway_url,
            external_calls_enabled: w.external_calls_enabled,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InitializeResult {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_worker_instance_id = &self.worker_instance_id;
            if !canonical_id(value_worker_instance_id) {
                return Err("invalid broker UUID");
            }
            if value_worker_instance_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_process = &self.process;
            value_process.validate()?;
        }
        {
            let value_gateway_url = &self.gateway_url;
            if value_gateway_url.chars().count() < 22 {
                return Err("invalid broker text");
            }
            if value_gateway_url.chars().count() > 32 {
                return Err("invalid broker text");
            }
            if !gateway_url(value_gateway_url) {
                return Err("invalid broker gateway URL");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for InitializeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InitializeResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InitializeResponseWire")]
pub struct InitializeResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: InitializeResult,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: InitializeResult,
}
impl TryFrom<InitializeResponseWire> for InitializeResponse {
    type Error = &'static str;
    fn try_from(w: InitializeResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InitializeResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for InitializeResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InitializeResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PreparePayloadWire")]
pub struct PreparePayload {
    pub operation_id: CanonicalId,
    pub context: ContextBinding,
    pub snapshot: SelectionSnapshot,
    pub ttl_ms: RequestedTtlMs,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparePayloadWire {
    pub operation_id: CanonicalId,
    pub context: ContextBinding,
    pub snapshot: SelectionSnapshot,
    pub ttl_ms: RequestedTtlMs,
}
impl TryFrom<PreparePayloadWire> for PreparePayload {
    type Error = &'static str;
    fn try_from(w: PreparePayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            context: w.context,
            snapshot: w.snapshot,
            ttl_ms: w.ttl_ms,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PreparePayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_context = &self.context;
            value_context.validate()?;
        }
        {
            let value_snapshot = &self.snapshot;
            value_snapshot.validate()?;
        }
        {
            let value_ttl_ms = &self.ttl_ms;
            if *value_ttl_ms < 1 {
                return Err("invalid broker number");
            }
            if *value_ttl_ms > 300000 {
                return Err("invalid broker number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for PreparePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparePayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PrepareRequestWire")]
pub struct PrepareRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: PreparePayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PrepareRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: PreparePayload,
}
impl TryFrom<PrepareRequestWire> for PrepareRequest {
    type Error = &'static str;
    fn try_from(w: PrepareRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PrepareRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "prepare" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for PrepareRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PrepareRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PrepareResponseWire")]
pub struct PrepareResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrepareResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
impl TryFrom<PrepareResponseWire> for PrepareResponse {
    type Error = &'static str;
    fn try_from(w: PrepareResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PrepareResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for PrepareResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PrepareResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "BindTurnPayloadWire")]
pub struct BindTurnPayload {
    pub operation_id: CanonicalId,
    pub binding: CapabilityBinding,
    pub expected_revision: Revision,
    pub native_turn_id: NativeId,
    pub native_thread_id: NativeId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BindTurnPayloadWire {
    pub operation_id: CanonicalId,
    pub binding: CapabilityBinding,
    pub expected_revision: Revision,
    pub native_turn_id: NativeId,
    pub native_thread_id: NativeId,
}
impl TryFrom<BindTurnPayloadWire> for BindTurnPayload {
    type Error = &'static str;
    fn try_from(w: BindTurnPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            binding: w.binding,
            expected_revision: w.expected_revision,
            native_turn_id: w.native_turn_id,
            native_thread_id: w.native_thread_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl BindTurnPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        {
            let value_native_turn_id = &self.native_turn_id;
            if value_native_turn_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_turn_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        {
            let value_native_thread_id = &self.native_thread_id;
            if value_native_thread_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_thread_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for BindTurnPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BindTurnPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "BindTurnRequestWire")]
pub struct BindTurnRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: BindTurnPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BindTurnRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: BindTurnPayload,
}
impl TryFrom<BindTurnRequestWire> for BindTurnRequest {
    type Error = &'static str;
    fn try_from(w: BindTurnRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl BindTurnRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "bind_turn" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for BindTurnRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BindTurnRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "BindTurnResponseWire")]
pub struct BindTurnResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindTurnResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
impl TryFrom<BindTurnResponseWire> for BindTurnResponse {
    type Error = &'static str;
    fn try_from(w: BindTurnResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl BindTurnResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for BindTurnResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BindTurnResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RevokePayloadWire")]
pub struct RevokePayload {
    pub operation_id: CanonicalId,
    pub binding: CapabilityBinding,
    pub expected_revision: Revision,
    pub reason: RevokeReason,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RevokePayloadWire {
    pub operation_id: CanonicalId,
    pub binding: CapabilityBinding,
    pub expected_revision: Revision,
    pub reason: RevokeReason,
}
impl TryFrom<RevokePayloadWire> for RevokePayload {
    type Error = &'static str;
    fn try_from(w: RevokePayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            binding: w.binding,
            expected_revision: w.expected_revision,
            reason: w.reason,
        };
        v.validate()?;
        Ok(v)
    }
}
impl RevokePayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for RevokePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RevokePayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RevokeRequestWire")]
pub struct RevokeRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: RevokePayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RevokeRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: RevokePayload,
}
impl TryFrom<RevokeRequestWire> for RevokeRequest {
    type Error = &'static str;
    fn try_from(w: RevokeRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl RevokeRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "revoke" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for RevokeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RevokeRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RevokeResponseWire")]
pub struct RevokeResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevokeResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
impl TryFrom<RevokeResponseWire> for RevokeResponse {
    type Error = &'static str;
    fn try_from(w: RevokeResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl RevokeResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for RevokeResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RevokeResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "StatusPayloadWire")]
pub struct StatusPayload {
    pub binding: CapabilityBinding,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StatusPayloadWire {
    pub binding: CapabilityBinding,
}
impl TryFrom<StatusPayloadWire> for StatusPayload {
    type Error = &'static str;
    fn try_from(w: StatusPayloadWire) -> Result<Self, Self::Error> {
        let v = Self { binding: w.binding };
        v.validate()?;
        Ok(v)
    }
}
impl StatusPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for StatusPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StatusPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "StatusRequestWire")]
pub struct StatusRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: StatusPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StatusRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: StatusPayload,
}
impl TryFrom<StatusRequestWire> for StatusRequest {
    type Error = &'static str;
    fn try_from(w: StatusRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl StatusRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "status" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for StatusRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StatusRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "StatusResponseWire")]
pub struct StatusResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: Lease,
}
impl TryFrom<StatusResponseWire> for StatusResponse {
    type Error = &'static str;
    fn try_from(w: StatusResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl StatusResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for StatusResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StatusResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PendingCallPayloadWire")]
pub struct PendingCallPayload {
    pub binding: CapabilityBinding,
    pub native_turn_id: NativeId,
    pub call_ref: CanonicalId,
    pub native_thread_id: NativeId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PendingCallPayloadWire {
    pub binding: CapabilityBinding,
    pub native_turn_id: NativeId,
    pub call_ref: CanonicalId,
    pub native_thread_id: NativeId,
}
impl TryFrom<PendingCallPayloadWire> for PendingCallPayload {
    type Error = &'static str;
    fn try_from(w: PendingCallPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            binding: w.binding,
            native_turn_id: w.native_turn_id,
            call_ref: w.call_ref,
            native_thread_id: w.native_thread_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PendingCallPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        {
            let value_native_turn_id = &self.native_turn_id;
            if value_native_turn_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_turn_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        {
            let value_call_ref = &self.call_ref;
            if !canonical_id(value_call_ref) {
                return Err("invalid broker UUID");
            }
            if value_call_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_native_thread_id = &self.native_thread_id;
            if value_native_thread_id.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_native_thread_id.chars().count() > 256 {
                return Err("invalid broker text");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for PendingCallPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingCallPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PendingCallRequestWire")]
pub struct PendingCallRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: PendingCallPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PendingCallRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: PendingCallPayload,
}
impl TryFrom<PendingCallRequestWire> for PendingCallRequest {
    type Error = &'static str;
    fn try_from(w: PendingCallRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PendingCallRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "pending_call" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for PendingCallRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingCallRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PendingCallResponseWire")]
pub struct PendingCallResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: PendingCall,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingCallResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: PendingCall,
}
impl TryFrom<PendingCallResponseWire> for PendingCallResponse {
    type Error = &'static str;
    fn try_from(w: PendingCallResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl PendingCallResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for PendingCallResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingCallResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "DecideCallPayloadWire")]
pub struct DecideCallPayload {
    pub operation_id: CanonicalId,
    pub identity: CallIdentity,
    pub expected_revision: Revision,
    pub approval_ref: CanonicalId,
    pub decision_id: CanonicalId,
    pub decision: Decision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DecideCallPayloadWire {
    pub operation_id: CanonicalId,
    pub identity: CallIdentity,
    pub expected_revision: Revision,
    pub approval_ref: CanonicalId,
    pub decision_id: CanonicalId,
    pub decision: Decision,
}
impl TryFrom<DecideCallPayloadWire> for DecideCallPayload {
    type Error = &'static str;
    fn try_from(w: DecideCallPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            identity: w.identity,
            expected_revision: w.expected_revision,
            approval_ref: w.approval_ref,
            decision_id: w.decision_id,
            decision: w.decision,
        };
        v.validate()?;
        Ok(v)
    }
}
impl DecideCallPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_identity = &self.identity;
            value_identity.validate()?;
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid broker number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid broker number");
            }
        }
        {
            let value_approval_ref = &self.approval_ref;
            if !canonical_id(value_approval_ref) {
                return Err("invalid broker UUID");
            }
            if value_approval_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_decision_id = &self.decision_id;
            if !canonical_id(value_decision_id) {
                return Err("invalid broker UUID");
            }
            if value_decision_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for DecideCallPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecideCallPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "DecideCallRequestWire")]
pub struct DecideCallRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: DecideCallPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DecideCallRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: DecideCallPayload,
}
impl TryFrom<DecideCallRequestWire> for DecideCallRequest {
    type Error = &'static str;
    fn try_from(w: DecideCallRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl DecideCallRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "decide_call" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for DecideCallRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecideCallRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "DecideCallResponseWire")]
pub struct DecideCallResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: PendingCall,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecideCallResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: PendingCall,
}
impl TryFrom<DecideCallResponseWire> for DecideCallResponse {
    type Error = &'static str;
    fn try_from(w: DecideCallResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl DecideCallResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for DecideCallResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecideCallResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ShutdownPayloadWire")]
pub struct ShutdownPayload {
    pub operation_id: CanonicalId,
    pub process: ProcessBinding,
    pub reason: ShutdownReason,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShutdownPayloadWire {
    pub operation_id: CanonicalId,
    pub process: ProcessBinding,
    pub reason: ShutdownReason,
}
impl TryFrom<ShutdownPayloadWire> for ShutdownPayload {
    type Error = &'static str;
    fn try_from(w: ShutdownPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            process: w.process,
            reason: w.reason,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ShutdownPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_process = &self.process;
            value_process.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ShutdownPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShutdownPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ShutdownRequestWire")]
pub struct ShutdownRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ShutdownPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShutdownRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ShutdownPayload,
}
impl TryFrom<ShutdownRequestWire> for ShutdownRequest {
    type Error = &'static str;
    fn try_from(w: ShutdownRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ShutdownRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "shutdown" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ShutdownRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShutdownRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ShutdownResultWire")]
pub struct ShutdownResult {
    pub state: String,
    pub stopped_admissions: bool,
    pub external_outcomes: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShutdownResultWire {
    pub state: String,
    pub stopped_admissions: bool,
    pub external_outcomes: String,
}
impl TryFrom<ShutdownResultWire> for ShutdownResult {
    type Error = &'static str;
    fn try_from(w: ShutdownResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            state: w.state,
            stopped_admissions: w.stopped_admissions,
            external_outcomes: w.external_outcomes,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ShutdownResult {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_state = &self.state;
            if value_state != "stopping" {
                return Err("invalid broker constant");
            }
        }
        {
            let value_stopped_admissions = &self.stopped_admissions;
            if !*value_stopped_admissions {
                return Err("invalid broker constant");
            }
        }
        {
            let value_external_outcomes = &self.external_outcomes;
            if value_external_outcomes != "not_asserted" {
                return Err("invalid broker constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for ShutdownResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShutdownResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ShutdownResponseWire")]
pub struct ShutdownResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ShutdownResult,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShutdownResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ShutdownResult,
}
impl TryFrom<ShutdownResponseWire> for ShutdownResponse {
    type Error = &'static str;
    fn try_from(w: ShutdownResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ShutdownResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid broker UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ShutdownResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShutdownResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ErrorWire")]
pub struct Error {
    pub schema_version: i64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub request_id: Option<CanonicalId>,
    pub code: ErrorCode,
    pub retryable: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorWire {
    pub schema_version: i64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub request_id: Option<CanonicalId>,
    pub code: ErrorCode,
    pub retryable: bool,
}
impl TryFrom<ErrorWire> for Error {
    type Error = &'static str;
    fn try_from(w: ErrorWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            code: w.code,
            retryable: w.retryable,
        };
        v.validate()?;
        Ok(v)
    }
}
impl Error {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        self.request_id
            .as_ref()
            .map(|value_request_id| -> Result<(), &'static str> {
                if !canonical_id(value_request_id) {
                    return Err("invalid broker UUID");
                }
                if value_request_id == "00000000-0000-0000-0000-000000000000" {
                    return Err("invalid broker identity");
                }
                Ok(())
            })
            .transpose()?;
        {
            let value_retryable = &self.retryable;
            if *value_retryable {
                return Err("invalid broker constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ElicitationMetadataWire")]
pub struct ElicitationMetadata {
    pub yijie_market_call_ref: CanonicalId,
    pub yijie_kind: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ElicitationMetadataWire {
    pub yijie_market_call_ref: CanonicalId,
    pub yijie_kind: String,
}
impl TryFrom<ElicitationMetadataWire> for ElicitationMetadata {
    type Error = &'static str;
    fn try_from(w: ElicitationMetadataWire) -> Result<Self, Self::Error> {
        let v = Self {
            yijie_market_call_ref: w.yijie_market_call_ref,
            yijie_kind: w.yijie_kind,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ElicitationMetadata {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_yijie_market_call_ref = &self.yijie_market_call_ref;
            if !canonical_id(value_yijie_market_call_ref) {
                return Err("invalid broker UUID");
            }
            if value_yijie_market_call_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_yijie_kind = &self.yijie_kind;
            if value_yijie_kind != "gateway_call_approval" {
                return Err("invalid broker constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for ElicitationMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ElicitationMetadata([redacted])")
    }
}
fn canonical_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && s != "00000000-0000-0000-0000-000000000000"
        && b.iter().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                *c == b'-'
            } else {
                c.is_ascii_digit() || (*c >= b'a' && *c <= b'f')
            }
        })
}
fn gateway_url(s: &str) -> bool {
    s.strip_prefix("http://127.0.0.1:")
        .and_then(|x| x.strip_suffix("/mcp"))
        .is_some_and(|p| {
            !p.is_empty()
                && !p.starts_with("0")
                && p.len() <= 5
                && p.bytes().all(|b| b.is_ascii_digit())
                && p.parse::<u16>().is_ok_and(|v| v > 0)
        })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Initialize(Box<InitializeRequest>),
    Prepare(Box<PrepareRequest>),
    BindTurn(Box<BindTurnRequest>),
    Revoke(Box<RevokeRequest>),
    Status(Box<StatusRequest>),
    PendingCall(Box<PendingCallRequest>),
    DecideCall(Box<DecideCallRequest>),
    Shutdown(Box<ShutdownRequest>),
}
impl Request {
    pub fn request_id(&self) -> &str {
        match self {
            Self::Initialize(v) => &v.request_id,
            Self::Prepare(v) => &v.request_id,
            Self::BindTurn(v) => &v.request_id,
            Self::Revoke(v) => &v.request_id,
            Self::Status(v) => &v.request_id,
            Self::PendingCall(v) => &v.request_id,
            Self::DecideCall(v) => &v.request_id,
            Self::Shutdown(v) => &v.request_id,
        }
    }
}
pub const GENERIC_MAX_TOOLS_PER_SERVICE: usize = 512;
pub const GENERIC_MAX_TOOLS_PER_SELECTION: usize = 512;
pub const GENERIC_MAX_SCHEMA_BYTES: usize = 65536;
pub const GENERIC_MAX_SCHEMAS_BYTES: usize = 2097152;
pub const GENERIC_MAX_REVIEW_ARGUMENTS_BYTES: usize = 16384;
pub const GENERIC_MAX_RESULT_BYTES: usize = 65536;
pub const GENERIC_MAX_DEPTH: usize = 16;
pub const MAX_FRAME_BYTES: usize = 65536;
pub const MAX_LEASE_TTL_MS: usize = 300000;
pub const MAX_UNBOUND_WAIT_MS: usize = 5000;
pub const MAX_APPROVAL_TTL_MS: usize = 60000;
pub const MAX_RETAINED_LEASES: usize = 64;
pub const MAX_PENDING_CALLS: usize = 128;
pub const MAX_MUTATION_RECEIPTS: usize = 1024;
pub const MAX_ARGUMENTS_BYTES: usize = 32768;
pub const MAX_ARGUMENTS_DEPTH: usize = 16;
pub const GATEWAY_ERROR_OWNER_REJECTED_OUTCOME: &str = "not_sent";
pub const GATEWAY_ERROR_OWNER_REJECTED_CODE: &str = "approval_invalid";
pub const GATEWAY_ERROR_OWNER_REJECTED_MESSAGE: &str =
    "用户已拒绝本次连接器调用，本次外部请求未发送。不会自动重试。";
pub const GATEWAY_ERROR_ADMISSION_STOPPED_OUTCOME: &str = "not_sent";
pub const GATEWAY_ERROR_ADMISSION_STOPPED_CODE: &str = "approval_invalid";
pub const GATEWAY_ERROR_ADMISSION_STOPPED_MESSAGE: &str =
    "本次连接器调用在进入外部执行前已停止，外部请求未发送。不会自动重试。";
pub const GATEWAY_ERROR_EXECUTION_UNVERIFIED_OUTCOME: &str = "unknown";
pub const GATEWAY_ERROR_EXECUTION_UNVERIFIED_CODE: &str = "temporarily_unavailable";
pub const GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE:&str="本次连接器调用未取得可验证的结果，无法确认外部请求是否已发送或完成。不能据此判断没有数据或已执行；不会自动重试。";
