package app

import (
	"context"

	"github.com/36Dge/yijie-connectors/catalog"
	wire "github.com/36Dge/yijie-connectors/internal/contracts/marketconnectors"
	"github.com/36Dge/yijie-connectors/internal/worker"
)

// MarketFoundation is a private gateway assembly point. It does not expose a
// public HTTP management API, persist installations or grant tool execution.
// Native owns product state; an authenticated, versioned management API remains
// a subsequent Contracts-first integration step.
type MarketFoundation struct{ Worker *worker.Owner }

func NewMarketFoundation(workerManifest string) *MarketFoundation {
	return &MarketFoundation{Worker: worker.NewOwner(workerManifest)}
}

func (m *MarketFoundation) Catalog() (wire.Revision, []wire.CatalogEntry, error) {
	return catalog.Read()
}

func (m *MarketFoundation) AuthStatus(ctx context.Context, request wire.WorkerAuthStatusRequest) (wire.WorkerAuthStatusResponse, *wire.WorkerError, error) {
	return m.Worker.Query(ctx, request)
}

func (m *MarketFoundation) Close(ctx context.Context) error {
	return m.Worker.Close(ctx)
}
