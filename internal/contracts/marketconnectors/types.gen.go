// Generated from market-connectors source; DO NOT EDIT.
package marketconnectors

import (
	"bytes"
	"encoding/json"
	"errors"
	"io"
	"regexp"
	"unicode/utf8"
)

var ConnectorCapabilities = []Permission{PermissionConnectorRead, PermissionConnectorManage, PermissionConnectorCredentialsManage, PermissionConnectorUse}
var pattern0 = regexp.MustCompile("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
var pattern1 = regexp.MustCompile("^[A-Za-z0-9][A-Za-z0-9_-]{0,127}$")

type CanonicalId string

func (v CanonicalId) Validate() error {
	if !pattern0.MatchString(string(v)) {
		return errors.New("invalid connector identifier")
	}
	if string(v) == "00000000-0000-0000-0000-000000000000" {
		return errors.New("invalid connector UUID")
	}
	return nil
}

type ServiceId string

func (v ServiceId) Validate() error {
	if utf8.RuneCountInString(string(v)) < 1 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v)) > 128 {
		return errors.New("invalid connector text length")
	}
	if !pattern1.MatchString(string(v)) {
		return errors.New("invalid connector identifier")
	}
	return nil
}

type Revision int64

func (v Revision) Validate() error {
	if v < 1 {
		return errors.New("invalid connector number")
	}
	if v > 9007199254740991 {
		return errors.New("invalid connector number")
	}
	return nil
}

type InitialRevision int64

func (v InitialRevision) Validate() error {
	if v != 0 {
		return errors.New("invalid connector constant")
	}
	return nil
}

type SchemaVersion int64

func (v SchemaVersion) Validate() error {
	if v != 1 {
		return errors.New("invalid connector constant")
	}
	return nil
}

type CredentialReference string

func (v CredentialReference) Validate() error {
	if utf8.RuneCountInString(string(v)) < 1 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v)) > 128 {
		return errors.New("invalid connector text length")
	}
	if !pattern1.MatchString(string(v)) {
		return errors.New("invalid connector identifier")
	}
	return nil
}

type Permission string

const (
	PermissionConnectorRead              Permission = "connector.read"
	PermissionConnectorManage            Permission = "connector.manage"
	PermissionConnectorCredentialsManage Permission = "connector.credentials.manage"
	PermissionConnectorUse               Permission = "connector.use"
)

func (v Permission) Validate() error {
	switch v {
	case PermissionConnectorRead, PermissionConnectorManage, PermissionConnectorCredentialsManage, PermissionConnectorUse:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *Permission) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := Permission(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type CategoryId string

const (
	CategoryIdKnowledgeDocs   CategoryId = "knowledge_docs"
	CategoryIdEcommerceRetail CategoryId = "ecommerce_retail"
	CategoryIdDataAnalytics   CategoryId = "data_analytics"
	CategoryIdProductivity    CategoryId = "productivity"
	CategoryIdIndustryData    CategoryId = "industry_data"
	CategoryIdMarketing       CategoryId = "marketing"
)

func (v CategoryId) Validate() error {
	switch v {
	case CategoryIdKnowledgeDocs, CategoryIdEcommerceRetail, CategoryIdDataAnalytics, CategoryIdProductivity, CategoryIdIndustryData, CategoryIdMarketing:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *CategoryId) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := CategoryId(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Transport string

const (
	TransportHttp  Transport = "http"
	TransportStdio Transport = "stdio"
)

func (v Transport) Validate() error {
	switch v {
	case TransportHttp, TransportStdio:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *Transport) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := Transport(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type AuthMode string

const (
	AuthModeOauth               AuthMode = "oauth"
	AuthModeApiKey              AuthMode = "api_key"
	AuthModeProviderCredentials AuthMode = "provider_credentials"
	AuthModeLocalOauth          AuthMode = "local_oauth"
	AuthModeStdioApiKey         AuthMode = "stdio_api_key"
	AuthModeProviderGateway     AuthMode = "provider_gateway"
	AuthModeUnknown             AuthMode = "unknown"
)

func (v AuthMode) Validate() error {
	switch v {
	case AuthModeOauth, AuthModeApiKey, AuthModeProviderCredentials, AuthModeLocalOauth, AuthModeStdioApiKey, AuthModeProviderGateway, AuthModeUnknown:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *AuthMode) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := AuthMode(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Availability string

const (
	AvailabilityAvailable  Availability = "available"
	AvailabilityNeedsSetup Availability = "needs_setup"
	AvailabilityBlocked    Availability = "blocked"
	AvailabilityUnverified Availability = "unverified"
)

func (v Availability) Validate() error {
	switch v {
	case AvailabilityAvailable, AvailabilityNeedsSetup, AvailabilityBlocked, AvailabilityUnverified:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *Availability) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := Availability(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type InstallationStatus string

const (
	InstallationStatusInstalled InstallationStatus = "installed"
	InstallationStatusRemoving  InstallationStatus = "removing"
	InstallationStatusRemoved   InstallationStatus = "removed"
)

func (v InstallationStatus) Validate() error {
	switch v {
	case InstallationStatusInstalled, InstallationStatusRemoving, InstallationStatusRemoved:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *InstallationStatus) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := InstallationStatus(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type ConfigurationStatus string

const (
	ConfigurationStatusUnconfigured ConfigurationStatus = "unconfigured"
	ConfigurationStatusConfigured   ConfigurationStatus = "configured"
	ConfigurationStatusInvalid      ConfigurationStatus = "invalid"
	ConfigurationStatusUnknown      ConfigurationStatus = "unknown"
)

func (v ConfigurationStatus) Validate() error {
	switch v {
	case ConfigurationStatusUnconfigured, ConfigurationStatusConfigured, ConfigurationStatusInvalid, ConfigurationStatusUnknown:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *ConfigurationStatus) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := ConfigurationStatus(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type AuthorizationStatus string

const (
	AuthorizationStatusNotRequired AuthorizationStatus = "not_required"
	AuthorizationStatusRequired    AuthorizationStatus = "required"
	AuthorizationStatusAuthorizing AuthorizationStatus = "authorizing"
	AuthorizationStatusAuthorized  AuthorizationStatus = "authorized"
	AuthorizationStatusExpired     AuthorizationStatus = "expired"
	AuthorizationStatusFailed      AuthorizationStatus = "failed"
	AuthorizationStatusUnknown     AuthorizationStatus = "unknown"
)

func (v AuthorizationStatus) Validate() error {
	switch v {
	case AuthorizationStatusNotRequired, AuthorizationStatusRequired, AuthorizationStatusAuthorizing, AuthorizationStatusAuthorized, AuthorizationStatusExpired, AuthorizationStatusFailed, AuthorizationStatusUnknown:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *AuthorizationStatus) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := AuthorizationStatus(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type ConnectionStatus string

const (
	ConnectionStatusDisconnected ConnectionStatus = "disconnected"
	ConnectionStatusConnecting   ConnectionStatus = "connecting"
	ConnectionStatusReady        ConnectionStatus = "ready"
	ConnectionStatusFailed       ConnectionStatus = "failed"
	ConnectionStatusUnknown      ConnectionStatus = "unknown"
)

func (v ConnectionStatus) Validate() error {
	switch v {
	case ConnectionStatusDisconnected, ConnectionStatusConnecting, ConnectionStatusReady, ConnectionStatusFailed, ConnectionStatusUnknown:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *ConnectionStatus) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := ConnectionStatus(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationAction string

const (
	OperationActionInstall   OperationAction = "install"
	OperationActionConfigure OperationAction = "configure"
	OperationActionAuthorize OperationAction = "authorize"
	OperationActionEnable    OperationAction = "enable"
	OperationActionDisable   OperationAction = "disable"
	OperationActionUninstall OperationAction = "uninstall"
)

func (v OperationAction) Validate() error {
	switch v {
	case OperationActionInstall, OperationActionConfigure, OperationActionAuthorize, OperationActionEnable, OperationActionDisable, OperationActionUninstall:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *OperationAction) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := OperationAction(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationStatus string

const (
	OperationStatusPending   OperationStatus = "pending"
	OperationStatusSucceeded OperationStatus = "succeeded"
	OperationStatusFailed    OperationStatus = "failed"
	OperationStatusCancelled OperationStatus = "cancelled"
	OperationStatusUnknown   OperationStatus = "unknown"
)

func (v OperationStatus) Validate() error {
	switch v {
	case OperationStatusPending, OperationStatusSucceeded, OperationStatusFailed, OperationStatusCancelled, OperationStatusUnknown:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *OperationStatus) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := OperationStatus(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type ErrorCode string

const (
	ErrorCodeInvalidRequest             ErrorCode = "invalid_request"
	ErrorCodeContextInvalid             ErrorCode = "context_invalid"
	ErrorCodeNotFound                   ErrorCode = "not_found"
	ErrorCodeRequestConflict            ErrorCode = "request_conflict"
	ErrorCodeNotConfigured              ErrorCode = "not_configured"
	ErrorCodeAuthorizationRequired      ErrorCode = "authorization_required"
	ErrorCodeAuthorizationCancelled     ErrorCode = "authorization_cancelled"
	ErrorCodeDependencyMissing          ErrorCode = "dependency_missing"
	ErrorCodeProviderOnboardingRequired ErrorCode = "provider_onboarding_required"
	ErrorCodePermissionDenied           ErrorCode = "permission_denied"
	ErrorCodeRevisionConflict           ErrorCode = "revision_conflict"
	ErrorCodeUnsupportedCapability      ErrorCode = "unsupported_capability"
	ErrorCodeRateLimited                ErrorCode = "rate_limited"
	ErrorCodeTemporarilyUnavailable     ErrorCode = "temporarily_unavailable"
	ErrorCodeOperationPending           ErrorCode = "operation_pending"
	ErrorCodeOutcomeUnknown             ErrorCode = "outcome_unknown"
	ErrorCodeCleanupPending             ErrorCode = "cleanup_pending"
	ErrorCodeExecutionUnavailable       ErrorCode = "execution_unavailable"
	ErrorCodeSelectionStale             ErrorCode = "selection_stale"
)

func (v ErrorCode) Validate() error {
	switch v {
	case ErrorCodeInvalidRequest, ErrorCodeContextInvalid, ErrorCodeNotFound, ErrorCodeRequestConflict, ErrorCodeNotConfigured, ErrorCodeAuthorizationRequired, ErrorCodeAuthorizationCancelled, ErrorCodeDependencyMissing, ErrorCodeProviderOnboardingRequired, ErrorCodePermissionDenied, ErrorCodeRevisionConflict, ErrorCodeUnsupportedCapability, ErrorCodeRateLimited, ErrorCodeTemporarilyUnavailable, ErrorCodeOperationPending, ErrorCodeOutcomeUnknown, ErrorCodeCleanupPending, ErrorCodeExecutionUnavailable, ErrorCodeSelectionStale:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *ErrorCode) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := ErrorCode(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type CatalogEntry struct {
	ServiceId              ServiceId    `json:"serviceId"`
	ServerName             ServiceId    `json:"serverName"`
	DisplayName            string       `json:"displayName"`
	CategoryId             CategoryId   `json:"categoryId"`
	CategoryLabel          string       `json:"categoryLabel"`
	Description            string       `json:"description"`
	IconAssetId            ServiceId    `json:"iconAssetId"`
	Transport              Transport    `json:"transport"`
	AuthMode               AuthMode     `json:"authMode"`
	AuthorizationAvailable *bool        `json:"authorizationAvailable,omitempty"`
	Availability           Availability `json:"availability"`
	BlockerCodes           []ErrorCode  `json:"blockerCodes"`
}

func (v CatalogEntry) Validate() error {
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if err := v.ServerName.Validate(); err != nil {
		return err
	}
	if utf8.RuneCountInString(string(v.DisplayName)) < 1 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v.DisplayName)) > 160 {
		return errors.New("invalid connector text length")
	}
	if err := v.CategoryId.Validate(); err != nil {
		return err
	}
	if utf8.RuneCountInString(string(v.CategoryLabel)) < 1 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v.CategoryLabel)) > 64 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v.Description)) > 2000 {
		return errors.New("invalid connector text length")
	}
	if err := v.IconAssetId.Validate(); err != nil {
		return err
	}
	if err := v.Transport.Validate(); err != nil {
		return err
	}
	if err := v.AuthMode.Validate(); err != nil {
		return err
	}
	if v.AuthorizationAvailable != nil {
	}
	if err := v.Availability.Validate(); err != nil {
		return err
	}
	if v.BlockerCodes == nil {
		return errors.New("null connector array")
	}
	if len(v.BlockerCodes) > 20 {
		return errors.New("invalid connector array length")
	}
	for _, item := range v.BlockerCodes {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *CatalogEntry) UnmarshalJSON(b []byte) error {
	type wire CatalogEntry
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["serverName"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serverName"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["displayName"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["displayName"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["categoryId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["categoryId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["categoryLabel"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["categoryLabel"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["description"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["description"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["iconAssetId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["iconAssetId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["transport"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["transport"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["authMode"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["authMode"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["authorizationAvailable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["availability"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["availability"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["blockerCodes"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["blockerCodes"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := CatalogEntry(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Operation struct {
	OperationId    CanonicalId     `json:"operationId"`
	InstallationId CanonicalId     `json:"installationId"`
	ServiceId      ServiceId       `json:"serviceId"`
	Action         OperationAction `json:"action"`
	Status         OperationStatus `json:"status"`
	Revision       Revision        `json:"revision"`
	Cancellable    bool            `json:"cancellable"`
	ErrorCode      *ErrorCode      `json:"errorCode,omitempty"`
}

func (v Operation) Validate() error {
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if err := v.Action.Validate(); err != nil {
		return err
	}
	if err := v.Status.Validate(); err != nil {
		return err
	}
	if err := v.Revision.Validate(); err != nil {
		return err
	}
	if v.ErrorCode != nil {
		if err := (*v.ErrorCode).Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *Operation) UnmarshalJSON(b []byte) error {
	type wire Operation
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["action"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["action"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["status"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["status"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["revision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["revision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["cancellable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["cancellable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["errorCode"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := Operation(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Installation struct {
	InstallationId      CanonicalId          `json:"installationId"`
	ServiceId           ServiceId            `json:"serviceId"`
	Revision            Revision             `json:"revision"`
	Generation          Revision             `json:"generation"`
	Status              InstallationStatus   `json:"status"`
	DesiredEnabled      bool                 `json:"desiredEnabled"`
	EffectiveEnabled    bool                 `json:"effectiveEnabled"`
	ConfigurationStatus ConfigurationStatus  `json:"configurationStatus"`
	AuthorizationStatus AuthorizationStatus  `json:"authorizationStatus"`
	ConnectionStatus    ConnectionStatus     `json:"connectionStatus"`
	CredentialRef       *CredentialReference `json:"credentialRef,omitempty"`
	ActiveOperation     *Operation           `json:"activeOperation,omitempty"`
	ErrorCode           *ErrorCode           `json:"errorCode,omitempty"`
}

func (v Installation) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if err := v.Revision.Validate(); err != nil {
		return err
	}
	if err := v.Generation.Validate(); err != nil {
		return err
	}
	if err := v.Status.Validate(); err != nil {
		return err
	}
	if err := v.ConfigurationStatus.Validate(); err != nil {
		return err
	}
	if err := v.AuthorizationStatus.Validate(); err != nil {
		return err
	}
	if err := v.ConnectionStatus.Validate(); err != nil {
		return err
	}
	if v.CredentialRef != nil {
		if err := (*v.CredentialRef).Validate(); err != nil {
			return err
		}
	}
	if v.ActiveOperation != nil {
		if err := (*v.ActiveOperation).Validate(); err != nil {
			return err
		}
	}
	if v.ErrorCode != nil {
		if err := (*v.ErrorCode).Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *Installation) UnmarshalJSON(b []byte) error {
	type wire Installation
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["revision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["revision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["generation"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["generation"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["status"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["status"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["desiredEnabled"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["desiredEnabled"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["effectiveEnabled"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["effectiveEnabled"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["configurationStatus"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["configurationStatus"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["authorizationStatus"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["authorizationStatus"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["connectionStatus"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["connectionStatus"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["credentialRef"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["activeOperation"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["errorCode"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := Installation(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Snapshot struct {
	CatalogRevision    Revision       `json:"catalogRevision"`
	Catalog            []CatalogEntry `json:"catalog"`
	Installations      []Installation `json:"installations"`
	Capabilities       []Permission   `json:"capabilities"`
	ExecutionAvailable bool           `json:"executionAvailable"`
}

func (v Snapshot) Validate() error {
	if err := v.CatalogRevision.Validate(); err != nil {
		return err
	}
	if v.Catalog == nil {
		return errors.New("null connector array")
	}
	if len(v.Catalog) > 51 {
		return errors.New("invalid connector array length")
	}
	for _, item := range v.Catalog {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	if v.Installations == nil {
		return errors.New("null connector array")
	}
	if len(v.Installations) > 51 {
		return errors.New("invalid connector array length")
	}
	for _, item := range v.Installations {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	if v.Capabilities == nil {
		return errors.New("null connector array")
	}
	if len(v.Capabilities) > 4 {
		return errors.New("invalid connector array length")
	}
	for i, uniqueItem := range v.Capabilities {
		for _, prior := range v.Capabilities[:i] {
			if uniqueItem == prior {
				return errors.New("duplicate connector value")
			}
		}
	}
	for _, item := range v.Capabilities {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *Snapshot) UnmarshalJSON(b []byte) error {
	type wire Snapshot
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["catalogRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["catalogRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["catalog"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["catalog"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["installations"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installations"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["capabilities"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["capabilities"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["executionAvailable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["executionAvailable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := Snapshot(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type MutationResult struct {
	Installation Installation `json:"installation"`
	Operation    Operation    `json:"operation"`
}

func (v MutationResult) Validate() error {
	if err := v.Installation.Validate(); err != nil {
		return err
	}
	if err := v.Operation.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *MutationResult) UnmarshalJSON(b []byte) error {
	type wire MutationResult
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installation"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installation"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operation"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operation"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := MutationResult(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionRef struct {
	InstallationId CanonicalId `json:"installationId"`
	Revision       Revision    `json:"revision"`
	Generation     Revision    `json:"generation"`
}

func (v SelectionRef) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.Revision.Validate(); err != nil {
		return err
	}
	if err := v.Generation.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SelectionRef) UnmarshalJSON(b []byte) error {
	type wire SelectionRef
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["revision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["revision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["generation"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["generation"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionRef(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionDisplay struct {
	Reference   SelectionRef `json:"reference"`
	ServiceId   ServiceId    `json:"serviceId"`
	DisplayName string       `json:"displayName"`
}

func (v SelectionDisplay) Validate() error {
	if err := v.Reference.Validate(); err != nil {
		return err
	}
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if utf8.RuneCountInString(string(v.DisplayName)) < 1 {
		return errors.New("invalid connector text length")
	}
	if utf8.RuneCountInString(string(v.DisplayName)) > 160 {
		return errors.New("invalid connector text length")
	}
	return nil
}
func (v *SelectionDisplay) UnmarshalJSON(b []byte) error {
	type wire SelectionDisplay
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["reference"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["reference"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["displayName"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["displayName"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionDisplay(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionValidation struct {
	Selection          []SelectionDisplay `json:"selection"`
	ExecutionAvailable bool               `json:"executionAvailable"`
}

func (v SelectionValidation) Validate() error {
	if v.Selection == nil {
		return errors.New("null connector array")
	}
	if len(v.Selection) > 51 {
		return errors.New("invalid connector array length")
	}
	for _, item := range v.Selection {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *SelectionValidation) UnmarshalJSON(b []byte) error {
	type wire SelectionValidation
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["selection"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["selection"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["executionAvailable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["executionAvailable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionValidation(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type EmptyPayload struct {
}

func (v EmptyPayload) Validate() error {
	return nil
}
func (v *EmptyPayload) UnmarshalJSON(b []byte) error {
	type wire EmptyPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := EmptyPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type InstallPayload struct {
	ServiceId        ServiceId       `json:"serviceId"`
	OperationId      CanonicalId     `json:"operationId"`
	ExpectedRevision InitialRevision `json:"expectedRevision"`
}

func (v InstallPayload) Validate() error {
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *InstallPayload) UnmarshalJSON(b []byte) error {
	type wire InstallPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := InstallPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type InstallationOperationPayload struct {
	InstallationId   CanonicalId `json:"installationId"`
	OperationId      CanonicalId `json:"operationId"`
	ExpectedRevision Revision    `json:"expectedRevision"`
}

func (v InstallationOperationPayload) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *InstallationOperationPayload) UnmarshalJSON(b []byte) error {
	type wire InstallationOperationPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := InstallationOperationPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type CredentialOperationPayload struct {
	InstallationId     CanonicalId `json:"installationId"`
	OperationId        CanonicalId `json:"operationId"`
	ExpectedRevision   Revision    `json:"expectedRevision"`
	ExpectedGeneration Revision    `json:"expectedGeneration"`
}

func (v CredentialOperationPayload) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedGeneration.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *CredentialOperationPayload) UnmarshalJSON(b []byte) error {
	type wire CredentialOperationPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedGeneration"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedGeneration"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := CredentialOperationPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SetEnabledPayload struct {
	InstallationId   CanonicalId `json:"installationId"`
	OperationId      CanonicalId `json:"operationId"`
	ExpectedRevision Revision    `json:"expectedRevision"`
	DesiredEnabled   bool        `json:"desiredEnabled"`
}

func (v SetEnabledPayload) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SetEnabledPayload) UnmarshalJSON(b []byte) error {
	type wire SetEnabledPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["desiredEnabled"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["desiredEnabled"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SetEnabledPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type UninstallPayload struct {
	InstallationId   CanonicalId `json:"installationId"`
	OperationId      CanonicalId `json:"operationId"`
	ExpectedRevision Revision    `json:"expectedRevision"`
	Confirmed        bool        `json:"confirmed"`
}

func (v UninstallPayload) Validate() error {
	if err := v.InstallationId.Validate(); err != nil {
		return err
	}
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	if v.Confirmed != true {
		return errors.New("invalid connector constant")
	}
	return nil
}
func (v *UninstallPayload) UnmarshalJSON(b []byte) error {
	type wire UninstallPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["installationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["installationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["confirmed"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["confirmed"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := UninstallPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationReadPayload struct {
	OperationId CanonicalId `json:"operationId"`
}

func (v OperationReadPayload) Validate() error {
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationReadPayload) UnmarshalJSON(b []byte) error {
	type wire OperationReadPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationReadPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationCancelPayload struct {
	OperationId      CanonicalId `json:"operationId"`
	ExpectedRevision Revision    `json:"expectedRevision"`
}

func (v OperationCancelPayload) Validate() error {
	if err := v.OperationId.Validate(); err != nil {
		return err
	}
	if err := v.ExpectedRevision.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationCancelPayload) UnmarshalJSON(b []byte) error {
	type wire OperationCancelPayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["operationId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["operationId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["expectedRevision"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["expectedRevision"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationCancelPayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionValidatePayload struct {
	Selection []SelectionRef `json:"selection"`
}

func (v SelectionValidatePayload) Validate() error {
	if v.Selection == nil {
		return errors.New("null connector array")
	}
	if len(v.Selection) > 51 {
		return errors.New("invalid connector array length")
	}
	for i, uniqueItem := range v.Selection {
		for _, prior := range v.Selection[:i] {
			if uniqueItem == prior {
				return errors.New("duplicate connector value")
			}
		}
	}
	for _, item := range v.Selection {
		if err := item.Validate(); err != nil {
			return err
		}
	}
	return nil
}
func (v *SelectionValidatePayload) UnmarshalJSON(b []byte) error {
	type wire SelectionValidatePayload
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["selection"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["selection"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionValidatePayload(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SnapshotRequest struct {
	SchemaVersion SchemaVersion `json:"schemaVersion"`
	RequestId     CanonicalId   `json:"requestId"`
	ContextId     CanonicalId   `json:"contextId"`
	Payload       EmptyPayload  `json:"payload"`
}

func (v SnapshotRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SnapshotRequest) UnmarshalJSON(b []byte) error {
	type wire SnapshotRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SnapshotRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SnapshotResponse struct {
	SchemaVersion SchemaVersion `json:"schemaVersion"`
	RequestId     CanonicalId   `json:"requestId"`
	Data          Snapshot      `json:"data"`
}

func (v SnapshotResponse) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.Data.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SnapshotResponse) UnmarshalJSON(b []byte) error {
	type wire SnapshotResponse
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["data"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["data"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SnapshotResponse(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type InstallRequest struct {
	SchemaVersion SchemaVersion  `json:"schemaVersion"`
	RequestId     CanonicalId    `json:"requestId"`
	ContextId     CanonicalId    `json:"contextId"`
	Payload       InstallPayload `json:"payload"`
}

func (v InstallRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *InstallRequest) UnmarshalJSON(b []byte) error {
	type wire InstallRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := InstallRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type MutationResponse struct {
	SchemaVersion SchemaVersion  `json:"schemaVersion"`
	RequestId     CanonicalId    `json:"requestId"`
	Data          MutationResult `json:"data"`
}

func (v MutationResponse) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.Data.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *MutationResponse) UnmarshalJSON(b []byte) error {
	type wire MutationResponse
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["data"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["data"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := MutationResponse(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SetEnabledRequest struct {
	SchemaVersion SchemaVersion     `json:"schemaVersion"`
	RequestId     CanonicalId       `json:"requestId"`
	ContextId     CanonicalId       `json:"contextId"`
	Payload       SetEnabledPayload `json:"payload"`
}

func (v SetEnabledRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SetEnabledRequest) UnmarshalJSON(b []byte) error {
	type wire SetEnabledRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SetEnabledRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type UninstallRequest struct {
	SchemaVersion SchemaVersion    `json:"schemaVersion"`
	RequestId     CanonicalId      `json:"requestId"`
	ContextId     CanonicalId      `json:"contextId"`
	Payload       UninstallPayload `json:"payload"`
}

func (v UninstallRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *UninstallRequest) UnmarshalJSON(b []byte) error {
	type wire UninstallRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := UninstallRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type ConfigureRequest struct {
	SchemaVersion SchemaVersion              `json:"schemaVersion"`
	RequestId     CanonicalId                `json:"requestId"`
	ContextId     CanonicalId                `json:"contextId"`
	Payload       CredentialOperationPayload `json:"payload"`
}

func (v ConfigureRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *ConfigureRequest) UnmarshalJSON(b []byte) error {
	type wire ConfigureRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := ConfigureRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type AuthorizeRequest struct {
	SchemaVersion SchemaVersion              `json:"schemaVersion"`
	RequestId     CanonicalId                `json:"requestId"`
	ContextId     CanonicalId                `json:"contextId"`
	Payload       CredentialOperationPayload `json:"payload"`
}

func (v AuthorizeRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *AuthorizeRequest) UnmarshalJSON(b []byte) error {
	type wire AuthorizeRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := AuthorizeRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationReadRequest struct {
	SchemaVersion SchemaVersion        `json:"schemaVersion"`
	RequestId     CanonicalId          `json:"requestId"`
	ContextId     CanonicalId          `json:"contextId"`
	Payload       OperationReadPayload `json:"payload"`
}

func (v OperationReadRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationReadRequest) UnmarshalJSON(b []byte) error {
	type wire OperationReadRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationReadRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationResponse struct {
	SchemaVersion SchemaVersion `json:"schemaVersion"`
	RequestId     CanonicalId   `json:"requestId"`
	Data          Operation     `json:"data"`
}

func (v OperationResponse) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.Data.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationResponse) UnmarshalJSON(b []byte) error {
	type wire OperationResponse
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["data"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["data"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationResponse(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationCancelRequest struct {
	SchemaVersion SchemaVersion          `json:"schemaVersion"`
	RequestId     CanonicalId            `json:"requestId"`
	ContextId     CanonicalId            `json:"contextId"`
	Payload       OperationCancelPayload `json:"payload"`
}

func (v OperationCancelRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationCancelRequest) UnmarshalJSON(b []byte) error {
	type wire OperationCancelRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationCancelRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionValidateRequest struct {
	SchemaVersion SchemaVersion            `json:"schemaVersion"`
	RequestId     CanonicalId              `json:"requestId"`
	ContextId     CanonicalId              `json:"contextId"`
	Payload       SelectionValidatePayload `json:"payload"`
}

func (v SelectionValidateRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SelectionValidateRequest) UnmarshalJSON(b []byte) error {
	type wire SelectionValidateRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionValidateRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type SelectionValidateResponse struct {
	SchemaVersion SchemaVersion       `json:"schemaVersion"`
	RequestId     CanonicalId         `json:"requestId"`
	Data          SelectionValidation `json:"data"`
}

func (v SelectionValidateResponse) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.Data.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *SelectionValidateResponse) UnmarshalJSON(b []byte) error {
	type wire SelectionValidateResponse
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["data"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["data"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := SelectionValidateResponse(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type Error struct {
	SchemaVersion SchemaVersion `json:"schemaVersion"`
	RequestId     *CanonicalId  `json:"requestId,omitempty"`
	Code          ErrorCode     `json:"code"`
	Retryable     bool          `json:"retryable"`
}

func (v Error) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if v.RequestId != nil {
		if err := (*v.RequestId).Validate(); err != nil {
			return err
		}
	}
	if err := v.Code.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *Error) UnmarshalJSON(b []byte) error {
	type wire Error
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["code"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["code"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["retryable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["retryable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := Error(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerErrorCode string

const (
	WorkerErrorCodeInvalidRequest         WorkerErrorCode = "invalid_request"
	WorkerErrorCodeNotQualified           WorkerErrorCode = "not_qualified"
	WorkerErrorCodeUnknownService         WorkerErrorCode = "unknown_service"
	WorkerErrorCodeTemporarilyUnavailable WorkerErrorCode = "temporarily_unavailable"
)

func (v WorkerErrorCode) Validate() error {
	switch v {
	case WorkerErrorCodeInvalidRequest, WorkerErrorCodeNotQualified, WorkerErrorCodeUnknownService, WorkerErrorCodeTemporarilyUnavailable:
		return nil
	default:
		return errors.New("invalid connector enum")
	}
}
func (v *WorkerErrorCode) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return errors.New("invalid connector enum")
	}
	x := WorkerErrorCode(s)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerLibraryPolicy struct {
	McpLibrary           string `json:"mcpLibrary"`
	OauthStore           string `json:"oauthStore"`
	CredentialBoundary   string `json:"credentialBoundary"`
	StdioShutdown        string `json:"stdioShutdown"`
	ExternalCallsEnabled bool   `json:"externalCallsEnabled"`
}

func (v WorkerLibraryPolicy) Validate() error {
	if string(v.McpLibrary) != "codex-rmcp-client" {
		return errors.New("invalid connector constant")
	}
	if string(v.OauthStore) != "keyring_only" {
		return errors.New("invalid connector constant")
	}
	if string(v.CredentialBoundary) != "connectors_only" {
		return errors.New("invalid connector constant")
	}
	if string(v.StdioShutdown) != "eof_only" {
		return errors.New("invalid connector constant")
	}
	if v.ExternalCallsEnabled != false {
		return errors.New("invalid connector constant")
	}
	return nil
}
func (v *WorkerLibraryPolicy) UnmarshalJSON(b []byte) error {
	type wire WorkerLibraryPolicy
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["mcpLibrary"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["mcpLibrary"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["oauthStore"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["oauthStore"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["credentialBoundary"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["credentialBoundary"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["stdioShutdown"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["stdioShutdown"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["externalCallsEnabled"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["externalCallsEnabled"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := WorkerLibraryPolicy(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerAuthStatusRequest struct {
	SchemaVersion SchemaVersion `json:"schemaVersion"`
	RequestId     CanonicalId   `json:"requestId"`
	Method        string        `json:"method"`
	ServiceId     ServiceId     `json:"serviceId"`
}

func (v WorkerAuthStatusRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if string(v.Method) != "auth_status" {
		return errors.New("invalid connector constant")
	}
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *WorkerAuthStatusRequest) UnmarshalJSON(b []byte) error {
	type wire WorkerAuthStatusRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["method"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["method"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := WorkerAuthStatusRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerAuthStatus struct {
	ServiceId           ServiceId           `json:"serviceId"`
	Qualification       string              `json:"qualification"`
	AuthorizationStatus string              `json:"authorizationStatus"`
	ConnectionStatus    string              `json:"connectionStatus"`
	ExecutionAvailable  bool                `json:"executionAvailable"`
	LibraryPolicy       WorkerLibraryPolicy `json:"libraryPolicy"`
}

func (v WorkerAuthStatus) Validate() error {
	if err := v.ServiceId.Validate(); err != nil {
		return err
	}
	if string(v.Qualification) != "not_qualified" {
		return errors.New("invalid connector constant")
	}
	if string(v.AuthorizationStatus) != "unknown" {
		return errors.New("invalid connector constant")
	}
	if string(v.ConnectionStatus) != "disconnected" {
		return errors.New("invalid connector constant")
	}
	if v.ExecutionAvailable != false {
		return errors.New("invalid connector constant")
	}
	if err := v.LibraryPolicy.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *WorkerAuthStatus) UnmarshalJSON(b []byte) error {
	type wire WorkerAuthStatus
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["serviceId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["serviceId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["qualification"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["qualification"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["authorizationStatus"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["authorizationStatus"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["connectionStatus"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["connectionStatus"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["executionAvailable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["executionAvailable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["libraryPolicy"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["libraryPolicy"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := WorkerAuthStatus(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerAuthStatusResponse struct {
	SchemaVersion SchemaVersion    `json:"schemaVersion"`
	RequestId     CanonicalId      `json:"requestId"`
	Data          WorkerAuthStatus `json:"data"`
}

func (v WorkerAuthStatusResponse) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.Data.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *WorkerAuthStatusResponse) UnmarshalJSON(b []byte) error {
	type wire WorkerAuthStatusResponse
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["data"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["data"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := WorkerAuthStatusResponse(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type WorkerError struct {
	SchemaVersion SchemaVersion   `json:"schemaVersion"`
	RequestId     *CanonicalId    `json:"requestId,omitempty"`
	Code          WorkerErrorCode `json:"code"`
	Retryable     bool            `json:"retryable"`
}

func (v WorkerError) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if v.RequestId != nil {
		if err := (*v.RequestId).Validate(); err != nil {
			return err
		}
	}
	if err := v.Code.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *WorkerError) UnmarshalJSON(b []byte) error {
	type wire WorkerError
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["code"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["code"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["retryable"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["retryable"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := WorkerError(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}

type OperationReopenRequest struct {
	SchemaVersion SchemaVersion          `json:"schemaVersion"`
	RequestId     CanonicalId            `json:"requestId"`
	ContextId     CanonicalId            `json:"contextId"`
	Payload       OperationCancelPayload `json:"payload"`
}

func (v OperationReopenRequest) Validate() error {
	if err := v.SchemaVersion.Validate(); err != nil {
		return err
	}
	if err := v.RequestId.Validate(); err != nil {
		return err
	}
	if err := v.ContextId.Validate(); err != nil {
		return err
	}
	if err := v.Payload.Validate(); err != nil {
		return err
	}
	return nil
}
func (v *OperationReopenRequest) UnmarshalJSON(b []byte) error {
	type wire OperationReopenRequest
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil || raw == nil {
		return errors.New("invalid connector object")
	}
	if _, ok := raw["schemaVersion"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["schemaVersion"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["requestId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["requestId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["contextId"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["contextId"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	if _, ok := raw["payload"]; !ok {
		return errors.New("missing connector field")
	}
	if value, ok := raw["payload"]; ok && bytes.Equal(bytes.TrimSpace(value), []byte("null")) {
		return errors.New("null connector field")
	}
	var w wire
	d := json.NewDecoder(bytes.NewReader(b))
	d.DisallowUnknownFields()
	if err := d.Decode(&w); err != nil {
		return errors.New("invalid connector object")
	}
	if err := d.Decode(new(any)); err != io.EOF {
		return errors.New("invalid connector object")
	}
	x := OperationReopenRequest(w)
	if err := x.Validate(); err != nil {
		return err
	}
	*v = x
	return nil
}
