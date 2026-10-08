// Generated from market-provider source; DO NOT EDIT.
use serde::{Deserialize, Serialize};

fn optional_non_null<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
pub use crate::broker_generated::ScopeBinding;
pub use crate::generated::CanonicalId;
pub use crate::generated::SelectionRef;
pub use crate::generated::ServiceId;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProviderBindingWire")]
pub struct ProviderBinding {
    pub reference: SelectionRef,
    pub service_id: ServiceId,
    pub credential_ref: CanonicalId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderBindingWire {
    pub reference: SelectionRef,
    pub service_id: ServiceId,
    pub credential_ref: CanonicalId,
}
impl TryFrom<ProviderBindingWire> for ProviderBinding {
    type Error = &'static str;
    fn try_from(w: ProviderBindingWire) -> Result<Self, Self::Error> {
        let v = Self {
            reference: w.reference,
            service_id: w.service_id,
            credential_ref: w.credential_ref,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProviderBinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_reference = &self.reference;
            value_reference.validate()?;
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
                return Err("invalid provider service ID");
            }
        }
        {
            let value_credential_ref = &self.credential_ref;
            if !canonical_id(value_credential_ref) {
                return Err("invalid provider UUID");
            }
            if value_credential_ref == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProviderBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProviderBinding([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProviderPayloadWire")]
pub struct ProviderPayload {
    pub operation_id: CanonicalId,
    pub host_instance_id: CanonicalId,
    pub scope: ScopeBinding,
    pub binding: ProviderBinding,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderPayloadWire {
    pub operation_id: CanonicalId,
    pub host_instance_id: CanonicalId,
    pub scope: ScopeBinding,
    pub binding: ProviderBinding,
}
impl TryFrom<ProviderPayloadWire> for ProviderPayload {
    type Error = &'static str;
    fn try_from(w: ProviderPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            host_instance_id: w.host_instance_id,
            scope: w.scope,
            binding: w.binding,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProviderPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid provider UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_host_instance_id = &self.host_instance_id;
            if !canonical_id(value_host_instance_id) {
                return Err("invalid provider UUID");
            }
            if value_host_instance_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_scope = &self.scope;
            value_scope.validate()?;
        }
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProviderPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProviderPayload([redacted])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    #[serde(rename = "starting")]
    Starting,
    #[serde(rename = "awaiting_user")]
    AwaitingUser,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationStatus {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "unauthorized")]
    Unauthorized,
    #[serde(rename = "authorizing")]
    Authorizing,
    #[serde(rename = "authorized")]
    Authorized,
    #[serde(rename = "error")]
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "connecting")]
    Connecting,
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "error")]
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Qualification {
    #[serde(rename = "not_qualified")]
    NotQualified,
    #[serde(rename = "qualified")]
    Qualified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "authority_mismatch")]
    AuthorityMismatch,
    #[serde(rename = "scope_expired")]
    ScopeExpired,
    #[serde(rename = "binding_mismatch")]
    BindingMismatch,
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "request_conflict")]
    RequestConflict,
    #[serde(rename = "capacity_exceeded")]
    CapacityExceeded,
    #[serde(rename = "unknown_service")]
    UnknownService,
    #[serde(rename = "unsupported_auth")]
    UnsupportedAuth,
    #[serde(rename = "keyring_unavailable")]
    KeyringUnavailable,
    #[serde(rename = "metadata_unavailable")]
    MetadataUnavailable,
    #[serde(rename = "authorization_rejected")]
    AuthorizationRejected,
    #[serde(rename = "authorization_timeout")]
    AuthorizationTimeout,
    #[serde(rename = "not_qualified")]
    NotQualified,
    #[serde(rename = "cleanup_pending")]
    CleanupPending,
    #[serde(rename = "temporarily_unavailable")]
    TemporarilyUnavailable,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProviderStatusWire")]
pub struct ProviderStatus {
    pub operation_id: CanonicalId,
    pub host_instance_id: CanonicalId,
    pub scope: ScopeBinding,
    pub binding: ProviderBinding,
    pub operation_state: OperationState,
    pub authorization_status: AuthorizationStatus,
    pub connection_status: ConnectionStatus,
    pub qualification: Qualification,
    pub execution_available: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub authorization_url: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderStatusWire {
    pub operation_id: CanonicalId,
    pub host_instance_id: CanonicalId,
    pub scope: ScopeBinding,
    pub binding: ProviderBinding,
    pub operation_state: OperationState,
    pub authorization_status: AuthorizationStatus,
    pub connection_status: ConnectionStatus,
    pub qualification: Qualification,
    pub execution_available: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub authorization_url: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
impl TryFrom<ProviderStatusWire> for ProviderStatus {
    type Error = &'static str;
    fn try_from(w: ProviderStatusWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            host_instance_id: w.host_instance_id,
            scope: w.scope,
            binding: w.binding,
            operation_state: w.operation_state,
            authorization_status: w.authorization_status,
            connection_status: w.connection_status,
            qualification: w.qualification,
            execution_available: w.execution_available,
            authorization_url: w.authorization_url,
            error_code: w.error_code,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProviderStatus {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid provider UUID");
            }
            if value_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_host_instance_id = &self.host_instance_id;
            if !canonical_id(value_host_instance_id) {
                return Err("invalid provider UUID");
            }
            if value_host_instance_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_scope = &self.scope;
            value_scope.validate()?;
        }
        {
            let value_binding = &self.binding;
            value_binding.validate()?;
        }
        self.authorization_url
            .as_ref()
            .map(|value_authorization_url| -> Result<(), &'static str> {
                if value_authorization_url.chars().count() < 1 {
                    return Err("invalid broker text");
                }
                if value_authorization_url.chars().count() > 4096 {
                    return Err("invalid broker text");
                }
                Ok(())
            })
            .transpose()?;
        if self.authorization_url.is_some() && self.operation_state != OperationState::AwaitingUser
        {
            return Err("authorization URL outside pending state");
        }
        if self.execution_available
            && (self.qualification != Qualification::Qualified
                || self.authorization_status != AuthorizationStatus::Authorized
                || self.connection_status != ConnectionStatus::Connected)
        {
            return Err("provider readiness lacks facts");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProviderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProviderStatus([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthBeginRequestWire")]
pub struct AuthBeginRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthBeginRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
impl TryFrom<AuthBeginRequestWire> for AuthBeginRequest {
    type Error = &'static str;
    fn try_from(w: AuthBeginRequestWire) -> Result<Self, Self::Error> {
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
impl AuthBeginRequest {
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
                return Err("invalid provider UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "provider_auth_begin" {
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
impl std::fmt::Debug for AuthBeginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthBeginRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthBeginResponseWire")]
pub struct AuthBeginResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthBeginResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
impl TryFrom<AuthBeginResponseWire> for AuthBeginResponse {
    type Error = &'static str;
    fn try_from(w: AuthBeginResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl AuthBeginResponse {
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
                return Err("invalid provider UUID");
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
impl std::fmt::Debug for AuthBeginResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthBeginResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthPollRequestWire")]
pub struct AuthPollRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthPollRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
impl TryFrom<AuthPollRequestWire> for AuthPollRequest {
    type Error = &'static str;
    fn try_from(w: AuthPollRequestWire) -> Result<Self, Self::Error> {
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
impl AuthPollRequest {
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
                return Err("invalid provider UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "provider_auth_poll" {
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
impl std::fmt::Debug for AuthPollRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthPollRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthPollResponseWire")]
pub struct AuthPollResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthPollResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
impl TryFrom<AuthPollResponseWire> for AuthPollResponse {
    type Error = &'static str;
    fn try_from(w: AuthPollResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl AuthPollResponse {
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
                return Err("invalid provider UUID");
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
impl std::fmt::Debug for AuthPollResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthPollResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthCancelRequestWire")]
pub struct AuthCancelRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthCancelRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
impl TryFrom<AuthCancelRequestWire> for AuthCancelRequest {
    type Error = &'static str;
    fn try_from(w: AuthCancelRequestWire) -> Result<Self, Self::Error> {
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
impl AuthCancelRequest {
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
                return Err("invalid provider UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "provider_auth_cancel" {
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
impl std::fmt::Debug for AuthCancelRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthCancelRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthCancelResponseWire")]
pub struct AuthCancelResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthCancelResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
impl TryFrom<AuthCancelResponseWire> for AuthCancelResponse {
    type Error = &'static str;
    fn try_from(w: AuthCancelResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl AuthCancelResponse {
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
                return Err("invalid provider UUID");
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
impl std::fmt::Debug for AuthCancelResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthCancelResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ForgetRequestWire")]
pub struct ForgetRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ForgetRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
impl TryFrom<ForgetRequestWire> for ForgetRequest {
    type Error = &'static str;
    fn try_from(w: ForgetRequestWire) -> Result<Self, Self::Error> {
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
impl ForgetRequest {
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
                return Err("invalid provider UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "provider_forget" {
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
impl std::fmt::Debug for ForgetRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ForgetRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ForgetResponseWire")]
pub struct ForgetResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ForgetResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
impl TryFrom<ForgetResponseWire> for ForgetResponse {
    type Error = &'static str;
    fn try_from(w: ForgetResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ForgetResponse {
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
                return Err("invalid provider UUID");
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
impl std::fmt::Debug for ForgetResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ForgetResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProbeRequestWire")]
pub struct ProbeRequest {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProbeRequestWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub method: String,
    pub payload: ProviderPayload,
}
impl TryFrom<ProbeRequestWire> for ProbeRequest {
    type Error = &'static str;
    fn try_from(w: ProbeRequestWire) -> Result<Self, Self::Error> {
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
impl ProbeRequest {
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
                return Err("invalid provider UUID");
            }
            if value_request_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "provider_probe" {
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
impl std::fmt::Debug for ProbeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProbeRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProbeResponseWire")]
pub struct ProbeResponse {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProbeResponseWire {
    pub schema_version: i64,
    pub request_id: CanonicalId,
    pub data: ProviderStatus,
}
impl TryFrom<ProbeResponseWire> for ProbeResponse {
    type Error = &'static str;
    fn try_from(w: ProbeResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProbeResponse {
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
                return Err("invalid provider UUID");
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
impl std::fmt::Debug for ProbeResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProbeResponse([redacted])")
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
                    return Err("invalid provider UUID");
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
#[serde(rename_all = "camelCase", try_from = "TushareDailyArgumentsWire")]
pub struct TushareDailyArguments {
    #[serde(rename = "ts_code")]
    pub ts_code: String,
    #[serde(rename = "trade_date")]
    pub trade_date: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TushareDailyArgumentsWire {
    #[serde(rename = "ts_code")]
    pub ts_code: String,
    #[serde(rename = "trade_date")]
    pub trade_date: String,
}
impl TryFrom<TushareDailyArgumentsWire> for TushareDailyArguments {
    type Error = &'static str;
    fn try_from(w: TushareDailyArgumentsWire) -> Result<Self, Self::Error> {
        let v = Self {
            ts_code: w.ts_code,
            trade_date: w.trade_date,
        };
        v.validate()?;
        Ok(v)
    }
}
impl TushareDailyArguments {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_ts_code = &self.ts_code;
            if value_ts_code.chars().count() < 9 {
                return Err("invalid broker text");
            }
            if value_ts_code.chars().count() > 9 {
                return Err("invalid broker text");
            }
            if !daily_code(value_ts_code) {
                return Err("invalid daily instrument code");
            }
        }
        {
            let value_trade_date = &self.trade_date;
            if value_trade_date.chars().count() < 8 {
                return Err("invalid broker text");
            }
            if value_trade_date.chars().count() > 8 {
                return Err("invalid broker text");
            }
            if !daily_date(value_trade_date) {
                return Err("invalid daily date");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for TushareDailyArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TushareDailyArguments([redacted])")
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
fn daily_code(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 9
        && b[..6].iter().all(u8::is_ascii_digit)
        && b[6] == b'.'
        && matches!(&b[7..], b"SH" | b"SZ" | b"BJ")
}
fn daily_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 8 || !b.iter().all(u8::is_ascii_digit) {
        return false;
    }
    let decimal = |v: &[u8]| v.iter().fold(0u32, |n, d| n * 10 + u32::from(d - b'0'));
    let year = decimal(&b[..4]);
    let month = decimal(&b[4..6]);
    let day = decimal(&b[6..]);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    year > 0 && day > 0 && day <= days
}
pub const MAX_FRAME_BYTES: usize = 65536;
pub const MAX_OPERATIONS: usize = 64;
pub const MAX_OAUTH_TTL_MS: usize = 300000;
pub const TUSHARE_DAILY_PROFILE: &str = "tushare-daily-v1";
pub const TUSHARE_DAILY_TOOL_NAME: &str = "tushare_daily";
pub const TUSHARE_DAILY_UPSTREAM_TOOL_NAME: &str = "daily";
pub const TUSHARE_DAILY_UPSTREAM_INPUT_SCHEMA_SHA256: &str =
    "ec10409543d1ae690de2b5da5893a0e69387cdd5a398f976fb04d187fc632ae1";
pub const TUSHARE_DAILY_RISK: &str = "read";
pub const TUSHARE_DAILY_PERMISSION_MODE: &str = "ask";
pub const TUSHARE_DAILY_RESULT_ROWS_FIELD: &str = "rows";
pub const TUSHARE_DAILY_INPUT_SCHEMA_JSON:&str="{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"ts_code\",\"trade_date\"],\"properties\":{\"ts_code\":{\"type\":\"string\",\"minLength\":9,\"maxLength\":9,\"pattern\":\"^[0-9]{6}\\\\.(SH|SZ|BJ)$\",\"description\":\"One six-digit A-share instrument code with SH, SZ or BJ suffix. This checks syntax, not current listing existence. No lists, separators or whitespace.\"},\"trade_date\":{\"type\":\"string\",\"minLength\":8,\"maxLength\":8,\"pattern\":\"^(?:(?:[0-9]{3}[1-9]|[0-9]{2}[1-9][0-9]|[0-9][1-9][0-9]{2}|[1-9][0-9]{3})(?:(?:01|03|05|07|08|10|12)(?:0[1-9]|[12][0-9]|3[01])|(?:04|06|09|11)(?:0[1-9]|[12][0-9]|30)|02(?:0[1-9]|1[0-9]|2[0-8]))|(?:[0-9]{2}(?:0[48]|[2468][048]|[13579][26])|(?:0[48]|[2468][048]|[13579][26])00)0229)$\",\"description\":\"Exactly one valid Gregorian date in YYYYMMDD, years 0001 through 9999, including leap-year validation. No date range.\"}},\"description\":\"Yijie-owned normalized input for the tushare-daily-v1 Gateway tool tushare_daily. Only these two strings are forwarded to exact upstream daily after validation and one-call approval; no fields, range, limit or other provider arguments are accepted. This is not the full upstream input schema.\"}";
pub const TUSHARE_DAILY_OUTPUT_SCHEMA_JSON:&str="{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"rows\"],\"properties\":{\"rows\":{\"type\":\"array\",\"maxItems\":1,\"items\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"ts_code\",\"trade_date\",\"close\"],\"properties\":{\"ts_code\":{\"type\":\"string\",\"minLength\":9,\"maxLength\":9,\"pattern\":\"^[0-9]{6}\\\\.(SH|SZ|BJ)$\",\"description\":\"One six-digit A-share instrument code with SH, SZ or BJ suffix. This checks syntax, not current listing existence. No lists, separators or whitespace.\"},\"trade_date\":{\"type\":\"string\",\"minLength\":8,\"maxLength\":8,\"pattern\":\"^(?:(?:[0-9]{3}[1-9]|[0-9]{2}[1-9][0-9]|[0-9][1-9][0-9]{2}|[1-9][0-9]{3})(?:(?:01|03|05|07|08|10|12)(?:0[1-9]|[12][0-9]|3[01])|(?:04|06|09|11)(?:0[1-9]|[12][0-9]|30)|02(?:0[1-9]|1[0-9]|2[0-8]))|(?:[0-9]{2}(?:0[48]|[2468][048]|[13579][26])|(?:0[48]|[2468][048]|[13579][26])00)0229)$\",\"description\":\"Exactly one valid Gregorian date in YYYYMMDD, years 0001 through 9999, including leap-year validation. No date range.\"},\"open\":{\"type\":\"number\"},\"high\":{\"type\":\"number\"},\"low\":{\"type\":\"number\"},\"close\":{\"type\":\"number\"},\"pre_close\":{\"type\":\"number\"},\"change\":{\"type\":\"number\"},\"pct_chg\":{\"type\":\"number\"},\"vol\":{\"type\":\"number\"},\"amount\":{\"type\":\"number\"}}}}}}";
pub const TUSHARE_DAILY_MAX_ROWS: usize = 1;
pub const TUSHARE_DAILY_MAX_CALLS_PER_APPROVAL: usize = 1;
pub const TUSHARE_DAILY_RESULT_IDENTITY_FIELDS: [&str; 2] = ["ts_code", "trade_date"];
pub const TUSHARE_DAILY_RESULT_NUMERIC_FIELDS: [&str; 9] = [
    "open",
    "high",
    "low",
    "close",
    "pre_close",
    "change",
    "pct_chg",
    "vol",
    "amount",
];
pub const TUSHARE_DAILY_RESULT_REQUIRED_NUMERIC_FIELDS: [&str; 1] = ["close"];
