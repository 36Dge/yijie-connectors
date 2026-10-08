// Generated from market-connectors source; DO NOT EDIT.
use serde::{Deserialize, Serialize};
fn optional_non_null<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
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
fn catalog_identifier(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b[0].is_ascii_alphanumeric()
        && b.iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}
pub const CONNECTOR_CAPABILITIES: [&str; 4] = [
    "connector.read",
    "connector.manage",
    "connector.credentials.manage",
    "connector.use",
];
pub type CanonicalId = String;
pub type ServiceId = String;
pub type Revision = i64;
pub type InitialRevision = i64;
pub type SchemaVersion = i64;
pub type CredentialReference = String;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "connector.read")]
    Read,
    #[serde(rename = "connector.manage")]
    Manage,
    #[serde(rename = "connector.credentials.manage")]
    CredentialsManage,
    #[serde(rename = "connector.use")]
    Use,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoryId {
    #[serde(rename = "knowledge_docs")]
    KnowledgeDocs,
    #[serde(rename = "ecommerce_retail")]
    EcommerceRetail,
    #[serde(rename = "data_analytics")]
    DataAnalytics,
    #[serde(rename = "productivity")]
    Productivity,
    #[serde(rename = "industry_data")]
    IndustryData,
    #[serde(rename = "marketing")]
    Marketing,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "stdio")]
    Stdio,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMode {
    #[serde(rename = "oauth")]
    Oauth,
    #[serde(rename = "api_key")]
    ApiKey,
    #[serde(rename = "provider_credentials")]
    ProviderCredentials,
    #[serde(rename = "local_oauth")]
    LocalOauth,
    #[serde(rename = "stdio_api_key")]
    StdioApiKey,
    #[serde(rename = "provider_gateway")]
    ProviderGateway,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "needs_setup")]
    NeedsSetup,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "unverified")]
    Unverified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationStatus {
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "removing")]
    Removing,
    #[serde(rename = "removed")]
    Removed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationStatus {
    #[serde(rename = "unconfigured")]
    Unconfigured,
    #[serde(rename = "configured")]
    Configured,
    #[serde(rename = "invalid")]
    Invalid,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationStatus {
    #[serde(rename = "not_required")]
    NotRequired,
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "authorizing")]
    Authorizing,
    #[serde(rename = "authorized")]
    Authorized,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "connecting")]
    Connecting,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationAction {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "configure")]
    Configure,
    #[serde(rename = "authorize")]
    Authorize,
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "uninstall")]
    Uninstall,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "context_invalid")]
    ContextInvalid,
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "request_conflict")]
    RequestConflict,
    #[serde(rename = "not_configured")]
    NotConfigured,
    #[serde(rename = "authorization_required")]
    AuthorizationRequired,
    #[serde(rename = "authorization_cancelled")]
    AuthorizationCancelled,
    #[serde(rename = "dependency_missing")]
    DependencyMissing,
    #[serde(rename = "provider_onboarding_required")]
    ProviderOnboardingRequired,
    #[serde(rename = "permission_denied")]
    PermissionDenied,
    #[serde(rename = "revision_conflict")]
    RevisionConflict,
    #[serde(rename = "unsupported_capability")]
    UnsupportedCapability,
    #[serde(rename = "rate_limited")]
    RateLimited,
    #[serde(rename = "temporarily_unavailable")]
    TemporarilyUnavailable,
    #[serde(rename = "operation_pending")]
    OperationPending,
    #[serde(rename = "outcome_unknown")]
    OutcomeUnknown,
    #[serde(rename = "cleanup_pending")]
    CleanupPending,
    #[serde(rename = "execution_unavailable")]
    ExecutionUnavailable,
    #[serde(rename = "selection_stale")]
    SelectionStale,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "CatalogEntryWire")]
pub struct CatalogEntry {
    pub service_id: ServiceId,
    pub server_name: ServiceId,
    pub display_name: String,
    pub category_id: CategoryId,
    pub category_label: String,
    pub description: String,
    pub icon_asset_id: ServiceId,
    pub transport: Transport,
    pub auth_mode: AuthMode,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub authorization_available: Option<bool>,
    pub availability: Availability,
    pub blocker_codes: Vec<ErrorCode>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogEntryWire {
    pub service_id: ServiceId,
    pub server_name: ServiceId,
    pub display_name: String,
    pub category_id: CategoryId,
    pub category_label: String,
    pub description: String,
    pub icon_asset_id: ServiceId,
    pub transport: Transport,
    pub auth_mode: AuthMode,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub authorization_available: Option<bool>,
    pub availability: Availability,
    pub blocker_codes: Vec<ErrorCode>,
}
impl TryFrom<CatalogEntryWire> for CatalogEntry {
    type Error = &'static str;
    fn try_from(w: CatalogEntryWire) -> Result<Self, Self::Error> {
        let v = Self {
            service_id: w.service_id,
            server_name: w.server_name,
            display_name: w.display_name,
            category_id: w.category_id,
            category_label: w.category_label,
            description: w.description,
            icon_asset_id: w.icon_asset_id,
            transport: w.transport,
            auth_mode: w.auth_mode,
            authorization_available: w.authorization_available,
            availability: w.availability,
            blocker_codes: w.blocker_codes,
        };
        v.validate()?;
        Ok(v)
    }
}
impl CatalogEntry {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_server_name = &self.server_name;
            if value_server_name.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_server_name.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_server_name) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_display_name = &self.display_name;
            if value_display_name.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_display_name.chars().count() > 160 {
                return Err("invalid connector text length");
            }
        }
        {
            let value_category_label = &self.category_label;
            if value_category_label.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_category_label.chars().count() > 64 {
                return Err("invalid connector text length");
            }
        }
        {
            let value_description = &self.description;
            if value_description.chars().count() > 2000 {
                return Err("invalid connector text length");
            }
        }
        {
            let value_icon_asset_id = &self.icon_asset_id;
            if value_icon_asset_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_icon_asset_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_icon_asset_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_blocker_codes = &self.blocker_codes;
            if value_blocker_codes.len() > 20 {
                return Err("invalid connector array length");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for CatalogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CatalogEntry([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationWire")]
pub struct Operation {
    pub operation_id: CanonicalId,
    pub installation_id: CanonicalId,
    pub service_id: ServiceId,
    pub action: OperationAction,
    pub status: OperationStatus,
    pub revision: Revision,
    pub cancellable: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperationWire {
    pub operation_id: CanonicalId,
    pub installation_id: CanonicalId,
    pub service_id: ServiceId,
    pub action: OperationAction,
    pub status: OperationStatus,
    pub revision: Revision,
    pub cancellable: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
impl TryFrom<OperationWire> for Operation {
    type Error = &'static str;
    fn try_from(w: OperationWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            installation_id: w.installation_id,
            service_id: w.service_id,
            action: w.action,
            status: w.status,
            revision: w.revision,
            cancellable: w.cancellable,
            error_code: w.error_code,
        };
        v.validate()?;
        Ok(v)
    }
}
impl Operation {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_revision = &self.revision;
            if *value_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Operation([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InstallationWire")]
pub struct Installation {
    pub installation_id: CanonicalId,
    pub service_id: ServiceId,
    pub revision: Revision,
    pub generation: Revision,
    pub status: InstallationStatus,
    pub desired_enabled: bool,
    pub effective_enabled: bool,
    pub configuration_status: ConfigurationStatus,
    pub authorization_status: AuthorizationStatus,
    pub connection_status: ConnectionStatus,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub credential_ref: Option<CredentialReference>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub active_operation: Option<Operation>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallationWire {
    pub installation_id: CanonicalId,
    pub service_id: ServiceId,
    pub revision: Revision,
    pub generation: Revision,
    pub status: InstallationStatus,
    pub desired_enabled: bool,
    pub effective_enabled: bool,
    pub configuration_status: ConfigurationStatus,
    pub authorization_status: AuthorizationStatus,
    pub connection_status: ConnectionStatus,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub credential_ref: Option<CredentialReference>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub active_operation: Option<Operation>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub error_code: Option<ErrorCode>,
}
impl TryFrom<InstallationWire> for Installation {
    type Error = &'static str;
    fn try_from(w: InstallationWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            service_id: w.service_id,
            revision: w.revision,
            generation: w.generation,
            status: w.status,
            desired_enabled: w.desired_enabled,
            effective_enabled: w.effective_enabled,
            configuration_status: w.configuration_status,
            authorization_status: w.authorization_status,
            connection_status: w.connection_status,
            credential_ref: w.credential_ref,
            active_operation: w.active_operation,
            error_code: w.error_code,
        };
        v.validate()?;
        Ok(v)
    }
}
impl Installation {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_revision = &self.revision;
            if *value_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        {
            let value_generation = &self.generation;
            if *value_generation < 1 {
                return Err("invalid connector number");
            }
            if *value_generation > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        self.credential_ref
            .as_ref()
            .map(|value_credential_ref| -> Result<(), &'static str> {
                if value_credential_ref.chars().count() < 1 {
                    return Err("invalid connector text length");
                }
                if value_credential_ref.chars().count() > 128 {
                    return Err("invalid connector text length");
                }
                if !catalog_identifier(value_credential_ref) {
                    return Err("invalid connector identifier");
                }
                Ok(())
            })
            .transpose()?;
        self.active_operation
            .as_ref()
            .map(|value_active_operation| -> Result<(), &'static str> {
                value_active_operation.validate()?;
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for Installation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Installation([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SnapshotWire")]
pub struct Snapshot {
    pub catalog_revision: Revision,
    pub catalog: Vec<CatalogEntry>,
    pub installations: Vec<Installation>,
    pub capabilities: Vec<Permission>,
    pub execution_available: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotWire {
    pub catalog_revision: Revision,
    pub catalog: Vec<CatalogEntry>,
    pub installations: Vec<Installation>,
    pub capabilities: Vec<Permission>,
    pub execution_available: bool,
}
impl TryFrom<SnapshotWire> for Snapshot {
    type Error = &'static str;
    fn try_from(w: SnapshotWire) -> Result<Self, Self::Error> {
        let v = Self {
            catalog_revision: w.catalog_revision,
            catalog: w.catalog,
            installations: w.installations,
            capabilities: w.capabilities,
            execution_available: w.execution_available,
        };
        v.validate()?;
        Ok(v)
    }
}
impl Snapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_catalog_revision = &self.catalog_revision;
            if *value_catalog_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_catalog_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        {
            let value_catalog = &self.catalog;
            if value_catalog.len() > 51 {
                return Err("invalid connector array length");
            }
            for item in value_catalog.iter() {
                item.validate()?;
            }
        }
        {
            let value_installations = &self.installations;
            if value_installations.len() > 51 {
                return Err("invalid connector array length");
            }
            for item in value_installations.iter() {
                item.validate()?;
            }
        }
        {
            let value_capabilities = &self.capabilities;
            if value_capabilities.len() > 4 {
                return Err("invalid connector array length");
            }
            for (i, v) in value_capabilities.iter().enumerate() {
                if value_capabilities[..i].contains(v) {
                    return Err("duplicate connector value");
                }
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Snapshot([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "MutationResultWire")]
pub struct MutationResult {
    pub installation: Installation,
    pub operation: Operation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MutationResultWire {
    pub installation: Installation,
    pub operation: Operation,
}
impl TryFrom<MutationResultWire> for MutationResult {
    type Error = &'static str;
    fn try_from(w: MutationResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation: w.installation,
            operation: w.operation,
        };
        v.validate()?;
        Ok(v)
    }
}
impl MutationResult {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation = &self.installation;
            value_installation.validate()?;
        }
        {
            let value_operation = &self.operation;
            value_operation.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for MutationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MutationResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionRefWire")]
pub struct SelectionRef {
    pub installation_id: CanonicalId,
    pub revision: Revision,
    pub generation: Revision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectionRefWire {
    pub installation_id: CanonicalId,
    pub revision: Revision,
    pub generation: Revision,
}
impl TryFrom<SelectionRefWire> for SelectionRef {
    type Error = &'static str;
    fn try_from(w: SelectionRefWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            revision: w.revision,
            generation: w.generation,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionRef {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_revision = &self.revision;
            if *value_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        {
            let value_generation = &self.generation;
            if *value_generation < 1 {
                return Err("invalid connector number");
            }
            if *value_generation > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionRef([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionDisplayWire")]
pub struct SelectionDisplay {
    pub reference: SelectionRef,
    pub service_id: ServiceId,
    pub display_name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectionDisplayWire {
    pub reference: SelectionRef,
    pub service_id: ServiceId,
    pub display_name: String,
}
impl TryFrom<SelectionDisplayWire> for SelectionDisplay {
    type Error = &'static str;
    fn try_from(w: SelectionDisplayWire) -> Result<Self, Self::Error> {
        let v = Self {
            reference: w.reference,
            service_id: w.service_id,
            display_name: w.display_name,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionDisplay {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_reference = &self.reference;
            value_reference.validate()?;
        }
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_display_name = &self.display_name;
            if value_display_name.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_display_name.chars().count() > 160 {
                return Err("invalid connector text length");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionDisplay([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionValidationWire")]
pub struct SelectionValidation {
    pub selection: Vec<SelectionDisplay>,
    pub execution_available: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectionValidationWire {
    pub selection: Vec<SelectionDisplay>,
    pub execution_available: bool,
}
impl TryFrom<SelectionValidationWire> for SelectionValidation {
    type Error = &'static str;
    fn try_from(w: SelectionValidationWire) -> Result<Self, Self::Error> {
        let v = Self {
            selection: w.selection,
            execution_available: w.execution_available,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionValidation {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_selection = &self.selection;
            if value_selection.len() > 51 {
                return Err("invalid connector array length");
            }
            for item in value_selection.iter() {
                item.validate()?;
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionValidation([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "EmptyPayloadWire")]
pub struct EmptyPayload {}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EmptyPayloadWire {}
impl TryFrom<EmptyPayloadWire> for EmptyPayload {
    type Error = &'static str;
    fn try_from(_w: EmptyPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {};
        v.validate()?;
        Ok(v)
    }
}
impl EmptyPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
impl std::fmt::Debug for EmptyPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EmptyPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InstallPayloadWire")]
pub struct InstallPayload {
    pub service_id: ServiceId,
    pub operation_id: CanonicalId,
    pub expected_revision: InitialRevision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstallPayloadWire {
    pub service_id: ServiceId,
    pub operation_id: CanonicalId,
    pub expected_revision: InitialRevision,
}
impl TryFrom<InstallPayloadWire> for InstallPayload {
    type Error = &'static str;
    fn try_from(w: InstallPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            service_id: w.service_id,
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InstallPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision != 0 {
                return Err("invalid connector constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for InstallPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstallPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    try_from = "InstallationOperationPayloadWire"
)]
pub struct InstallationOperationPayload {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstallationOperationPayloadWire {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
}
impl TryFrom<InstallationOperationPayloadWire> for InstallationOperationPayload {
    type Error = &'static str;
    fn try_from(w: InstallationOperationPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InstallationOperationPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for InstallationOperationPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstallationOperationPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "CredentialOperationPayloadWire")]
pub struct CredentialOperationPayload {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub expected_generation: Revision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CredentialOperationPayloadWire {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub expected_generation: Revision,
}
impl TryFrom<CredentialOperationPayloadWire> for CredentialOperationPayload {
    type Error = &'static str;
    fn try_from(w: CredentialOperationPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
            expected_generation: w.expected_generation,
        };
        v.validate()?;
        Ok(v)
    }
}
impl CredentialOperationPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        {
            let value_expected_generation = &self.expected_generation;
            if *value_expected_generation < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_generation > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for CredentialOperationPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CredentialOperationPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SetEnabledPayloadWire")]
pub struct SetEnabledPayload {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub desired_enabled: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetEnabledPayloadWire {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub desired_enabled: bool,
}
impl TryFrom<SetEnabledPayloadWire> for SetEnabledPayload {
    type Error = &'static str;
    fn try_from(w: SetEnabledPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
            desired_enabled: w.desired_enabled,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SetEnabledPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SetEnabledPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SetEnabledPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "UninstallPayloadWire")]
pub struct UninstallPayload {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub confirmed: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UninstallPayloadWire {
    pub installation_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
    pub confirmed: bool,
}
impl TryFrom<UninstallPayloadWire> for UninstallPayload {
    type Error = &'static str;
    fn try_from(w: UninstallPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            installation_id: w.installation_id,
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
            confirmed: w.confirmed,
        };
        v.validate()?;
        Ok(v)
    }
}
impl UninstallPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_installation_id = &self.installation_id;
            if !canonical_id(value_installation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        {
            let value_confirmed = &self.confirmed;
            if !*value_confirmed {
                return Err("invalid connector constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for UninstallPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UninstallPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationReadPayloadWire")]
pub struct OperationReadPayload {
    pub operation_id: CanonicalId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperationReadPayloadWire {
    pub operation_id: CanonicalId,
}
impl TryFrom<OperationReadPayloadWire> for OperationReadPayload {
    type Error = &'static str;
    fn try_from(w: OperationReadPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationReadPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationReadPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationReadPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationCancelPayloadWire")]
pub struct OperationCancelPayload {
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperationCancelPayloadWire {
    pub operation_id: CanonicalId,
    pub expected_revision: Revision,
}
impl TryFrom<OperationCancelPayloadWire> for OperationCancelPayload {
    type Error = &'static str;
    fn try_from(w: OperationCancelPayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            operation_id: w.operation_id,
            expected_revision: w.expected_revision,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationCancelPayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_operation_id = &self.operation_id;
            if !canonical_id(value_operation_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_expected_revision = &self.expected_revision;
            if *value_expected_revision < 1 {
                return Err("invalid connector number");
            }
            if *value_expected_revision > 9007199254740991 {
                return Err("invalid connector number");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationCancelPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationCancelPayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionValidatePayloadWire")]
pub struct SelectionValidatePayload {
    pub selection: Vec<SelectionRef>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectionValidatePayloadWire {
    pub selection: Vec<SelectionRef>,
}
impl TryFrom<SelectionValidatePayloadWire> for SelectionValidatePayload {
    type Error = &'static str;
    fn try_from(w: SelectionValidatePayloadWire) -> Result<Self, Self::Error> {
        let v = Self {
            selection: w.selection,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionValidatePayload {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_selection = &self.selection;
            if value_selection.len() > 51 {
                return Err("invalid connector array length");
            }
            for (i, v) in value_selection.iter().enumerate() {
                if value_selection[..i].contains(v) {
                    return Err("duplicate connector value");
                }
            }
            for item in value_selection.iter() {
                item.validate()?;
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionValidatePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionValidatePayload([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SnapshotRequestWire")]
pub struct SnapshotRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: EmptyPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnapshotRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: EmptyPayload,
}
impl TryFrom<SnapshotRequestWire> for SnapshotRequest {
    type Error = &'static str;
    fn try_from(w: SnapshotRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SnapshotRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for SnapshotRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SnapshotRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SnapshotResponseWire")]
pub struct SnapshotResponse {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: Snapshot,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotResponseWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: Snapshot,
}
impl TryFrom<SnapshotResponseWire> for SnapshotResponse {
    type Error = &'static str;
    fn try_from(w: SnapshotResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SnapshotResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for SnapshotResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SnapshotResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "InstallRequestWire")]
pub struct InstallRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: InstallPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstallRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: InstallPayload,
}
impl TryFrom<InstallRequestWire> for InstallRequest {
    type Error = &'static str;
    fn try_from(w: InstallRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl InstallRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for InstallRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstallRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "MutationResponseWire")]
pub struct MutationResponse {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: MutationResult,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MutationResponseWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: MutationResult,
}
impl TryFrom<MutationResponseWire> for MutationResponse {
    type Error = &'static str;
    fn try_from(w: MutationResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl MutationResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for MutationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MutationResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SetEnabledRequestWire")]
pub struct SetEnabledRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: SetEnabledPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetEnabledRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: SetEnabledPayload,
}
impl TryFrom<SetEnabledRequestWire> for SetEnabledRequest {
    type Error = &'static str;
    fn try_from(w: SetEnabledRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SetEnabledRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for SetEnabledRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SetEnabledRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "UninstallRequestWire")]
pub struct UninstallRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: UninstallPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UninstallRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: UninstallPayload,
}
impl TryFrom<UninstallRequestWire> for UninstallRequest {
    type Error = &'static str;
    fn try_from(w: UninstallRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl UninstallRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for UninstallRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UninstallRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ConfigureRequestWire")]
pub struct ConfigureRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: CredentialOperationPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigureRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: CredentialOperationPayload,
}
impl TryFrom<ConfigureRequestWire> for ConfigureRequest {
    type Error = &'static str;
    fn try_from(w: ConfigureRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ConfigureRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ConfigureRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConfigureRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AuthorizeRequestWire")]
pub struct AuthorizeRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: CredentialOperationPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthorizeRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: CredentialOperationPayload,
}
impl TryFrom<AuthorizeRequestWire> for AuthorizeRequest {
    type Error = &'static str;
    fn try_from(w: AuthorizeRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl AuthorizeRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for AuthorizeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthorizeRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationReadRequestWire")]
pub struct OperationReadRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationReadPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperationReadRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationReadPayload,
}
impl TryFrom<OperationReadRequestWire> for OperationReadRequest {
    type Error = &'static str;
    fn try_from(w: OperationReadRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationReadRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationReadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationReadRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationResponseWire")]
pub struct OperationResponse {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: Operation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperationResponseWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: Operation,
}
impl TryFrom<OperationResponseWire> for OperationResponse {
    type Error = &'static str;
    fn try_from(w: OperationResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationCancelRequestWire")]
pub struct OperationCancelRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationCancelPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperationCancelRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationCancelPayload,
}
impl TryFrom<OperationCancelRequestWire> for OperationCancelRequest {
    type Error = &'static str;
    fn try_from(w: OperationCancelRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationCancelRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationCancelRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationCancelRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionValidateRequestWire")]
pub struct SelectionValidateRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: SelectionValidatePayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectionValidateRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: SelectionValidatePayload,
}
impl TryFrom<SelectionValidateRequestWire> for SelectionValidateRequest {
    type Error = &'static str;
    fn try_from(w: SelectionValidateRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionValidateRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionValidateRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionValidateRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionValidateResponseWire")]
pub struct SelectionValidateResponse {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: SelectionValidation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectionValidateResponseWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: SelectionValidation,
}
impl TryFrom<SelectionValidateResponseWire> for SelectionValidateResponse {
    type Error = &'static str;
    fn try_from(w: SelectionValidateResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionValidateResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionValidateResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionValidateResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ErrorWire")]
pub struct Error {
    pub schema_version: SchemaVersion,
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
    pub schema_version: SchemaVersion,
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
                return Err("invalid connector constant");
            }
        }
        self.request_id
            .as_ref()
            .map(|value_request_id| -> Result<(), &'static str> {
                if !canonical_id(value_request_id) {
                    return Err("invalid connector UUID");
                }
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error([redacted])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerErrorCode {
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "not_qualified")]
    NotQualified,
    #[serde(rename = "unknown_service")]
    UnknownService,
    #[serde(rename = "temporarily_unavailable")]
    TemporarilyUnavailable,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "WorkerLibraryPolicyWire")]
pub struct WorkerLibraryPolicy {
    pub mcp_library: String,
    pub oauth_store: String,
    pub credential_boundary: String,
    pub stdio_shutdown: String,
    pub external_calls_enabled: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerLibraryPolicyWire {
    pub mcp_library: String,
    pub oauth_store: String,
    pub credential_boundary: String,
    pub stdio_shutdown: String,
    pub external_calls_enabled: bool,
}
impl TryFrom<WorkerLibraryPolicyWire> for WorkerLibraryPolicy {
    type Error = &'static str;
    fn try_from(w: WorkerLibraryPolicyWire) -> Result<Self, Self::Error> {
        let v = Self {
            mcp_library: w.mcp_library,
            oauth_store: w.oauth_store,
            credential_boundary: w.credential_boundary,
            stdio_shutdown: w.stdio_shutdown,
            external_calls_enabled: w.external_calls_enabled,
        };
        v.validate()?;
        Ok(v)
    }
}
impl WorkerLibraryPolicy {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_mcp_library = &self.mcp_library;
            if value_mcp_library != "codex-rmcp-client" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_oauth_store = &self.oauth_store;
            if value_oauth_store != "keyring_only" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_credential_boundary = &self.credential_boundary;
            if value_credential_boundary != "connectors_only" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_stdio_shutdown = &self.stdio_shutdown;
            if value_stdio_shutdown != "eof_only" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_external_calls_enabled = &self.external_calls_enabled;
            if *value_external_calls_enabled {
                return Err("invalid connector constant");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for WorkerLibraryPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerLibraryPolicy([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "WorkerAuthStatusRequestWire")]
pub struct WorkerAuthStatusRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub method: String,
    pub service_id: ServiceId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerAuthStatusRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub method: String,
    pub service_id: ServiceId,
}
impl TryFrom<WorkerAuthStatusRequestWire> for WorkerAuthStatusRequest {
    type Error = &'static str;
    fn try_from(w: WorkerAuthStatusRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            method: w.method,
            service_id: w.service_id,
        };
        v.validate()?;
        Ok(v)
    }
}
impl WorkerAuthStatusRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_method = &self.method;
            if value_method != "auth_status" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for WorkerAuthStatusRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerAuthStatusRequest([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "WorkerAuthStatusWire")]
pub struct WorkerAuthStatus {
    pub service_id: ServiceId,
    pub qualification: String,
    pub authorization_status: String,
    pub connection_status: String,
    pub execution_available: bool,
    pub library_policy: WorkerLibraryPolicy,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerAuthStatusWire {
    pub service_id: ServiceId,
    pub qualification: String,
    pub authorization_status: String,
    pub connection_status: String,
    pub execution_available: bool,
    pub library_policy: WorkerLibraryPolicy,
}
impl TryFrom<WorkerAuthStatusWire> for WorkerAuthStatus {
    type Error = &'static str;
    fn try_from(w: WorkerAuthStatusWire) -> Result<Self, Self::Error> {
        let v = Self {
            service_id: w.service_id,
            qualification: w.qualification,
            authorization_status: w.authorization_status,
            connection_status: w.connection_status,
            execution_available: w.execution_available,
            library_policy: w.library_policy,
        };
        v.validate()?;
        Ok(v)
    }
}
impl WorkerAuthStatus {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_service_id = &self.service_id;
            if value_service_id.chars().count() < 1 {
                return Err("invalid connector text length");
            }
            if value_service_id.chars().count() > 128 {
                return Err("invalid connector text length");
            }
            if !catalog_identifier(value_service_id) {
                return Err("invalid connector identifier");
            }
        }
        {
            let value_qualification = &self.qualification;
            if value_qualification != "not_qualified" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_authorization_status = &self.authorization_status;
            if value_authorization_status != "unknown" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_connection_status = &self.connection_status;
            if value_connection_status != "disconnected" {
                return Err("invalid connector constant");
            }
        }
        {
            let value_execution_available = &self.execution_available;
            if *value_execution_available {
                return Err("invalid connector constant");
            }
        }
        {
            let value_library_policy = &self.library_policy;
            value_library_policy.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for WorkerAuthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerAuthStatus([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "WorkerAuthStatusResponseWire")]
pub struct WorkerAuthStatusResponse {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: WorkerAuthStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerAuthStatusResponseWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub data: WorkerAuthStatus,
}
impl TryFrom<WorkerAuthStatusResponseWire> for WorkerAuthStatusResponse {
    type Error = &'static str;
    fn try_from(w: WorkerAuthStatusResponseWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            data: w.data,
        };
        v.validate()?;
        Ok(v)
    }
}
impl WorkerAuthStatusResponse {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_data = &self.data;
            value_data.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for WorkerAuthStatusResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerAuthStatusResponse([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "WorkerErrorWire")]
pub struct WorkerError {
    pub schema_version: SchemaVersion,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub request_id: Option<CanonicalId>,
    pub code: WorkerErrorCode,
    pub retryable: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerErrorWire {
    pub schema_version: SchemaVersion,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub request_id: Option<CanonicalId>,
    pub code: WorkerErrorCode,
    pub retryable: bool,
}
impl TryFrom<WorkerErrorWire> for WorkerError {
    type Error = &'static str;
    fn try_from(w: WorkerErrorWire) -> Result<Self, Self::Error> {
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
impl WorkerError {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        self.request_id
            .as_ref()
            .map(|value_request_id| -> Result<(), &'static str> {
                if !canonical_id(value_request_id) {
                    return Err("invalid connector UUID");
                }
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WorkerError([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "OperationReopenRequestWire")]
pub struct OperationReopenRequest {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationCancelPayload,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperationReopenRequestWire {
    pub schema_version: SchemaVersion,
    pub request_id: CanonicalId,
    pub context_id: CanonicalId,
    pub payload: OperationCancelPayload,
}
impl TryFrom<OperationReopenRequestWire> for OperationReopenRequest {
    type Error = &'static str;
    fn try_from(w: OperationReopenRequestWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            request_id: w.request_id,
            context_id: w.context_id,
            payload: w.payload,
        };
        v.validate()?;
        Ok(v)
    }
}
impl OperationReopenRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid connector constant");
            }
        }
        {
            let value_request_id = &self.request_id;
            if !canonical_id(value_request_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_context_id = &self.context_id;
            if !canonical_id(value_context_id) {
                return Err("invalid connector UUID");
            }
        }
        {
            let value_payload = &self.payload;
            value_payload.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for OperationReopenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationReopenRequest([redacted])")
    }
}
