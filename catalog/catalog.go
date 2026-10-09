// Package catalog owns the non-secret market directory. It exposes no provider
// endpoints, credential storage, activation or HTTP management authority.
package catalog

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"errors"
	"io"

	wire "github.com/36Dge/yijie-connectors/internal/contracts/marketconnectors"
)

//go:embed market-catalog.v1.json
var source []byte

// document is this repository's packaged asset format, not a service wire DTO.
// Every public entry uses the Contracts-generated DTO and validator.
type document struct {
	CatalogRevision wire.Revision       `json:"catalogRevision"`
	Catalog         []wire.CatalogEntry `json:"catalog"`
}

func Read() (wire.Revision, []wire.CatalogEntry, error) {
	var value document
	decoder := json.NewDecoder(bytes.NewReader(source))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return 0, nil, errors.New("managed connector catalog is invalid")
	}
	if err := value.CatalogRevision.Validate(); err != nil || len(value.Catalog) != 58 {
		return 0, nil, errors.New("managed connector catalog revision or size is invalid")
	}
	var trailing any
	if err := decoder.Decode(&trailing); !errors.Is(err, io.EOF) {
		return 0, nil, errors.New("managed connector catalog contains trailing data")
	}
	seen := make(map[wire.ServiceId]bool, len(value.Catalog))
	for _, entry := range value.Catalog {
		if err := entry.Validate(); err != nil || seen[entry.ServiceId] || entry.IconAssetId != entry.ServiceId {
			return 0, nil, errors.New("managed connector catalog entry is invalid")
		}
		seen[entry.ServiceId] = true
		if entry.Availability == wire.AvailabilityAvailable || len(entry.BlockerCodes) == 0 {
			return 0, nil, errors.New("unqualified connector cannot be available")
		}
	}
	return value.CatalogRevision, value.Catalog, nil
}

// PublicJSON validates before returning an independent copy for packaging.
func PublicJSON() ([]byte, error) {
	if _, _, err := Read(); err != nil {
		return nil, err
	}
	return append([]byte(nil), source...), nil
}
