package worker

import (
	"context"
	"errors"
	"os"
	"testing"
	"time"

	wire "github.com/36Dge/yijie-connectors/internal/contracts/marketconnectors"
)

func TestRealStatusWorkerNormalEOF(t *testing.T) {
	manifest := os.Getenv("YIJIE_MARKET_WORKER_TEST_MANIFEST")
	if manifest == "" {
		t.Skip("explicit locally built artifact required; no helper executable substituted")
	}
	owner := NewOwner(manifest)
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()
	request := wire.WorkerAuthStatusRequest{SchemaVersion: 1, RequestId: "11111111-1111-4111-8111-111111111111", Method: "auth_status", ServiceId: "cue"}
	response, failure, err := owner.Query(ctx, request)
	if err != nil || failure != nil || response.Data.ExecutionAvailable || response.Data.Qualification != "not_qualified" {
		t.Fatalf("normal status query failed: error=%v failure=%v", err, failure)
	}
	request.ServiceId = "unlisted-service"
	_, failure, err = owner.Query(ctx, request)
	if err != nil || failure == nil || failure.Code != wire.WorkerErrorCodeUnknownService {
		t.Fatalf("unknown service classification failed: error=%v failure=%v", err, failure)
	}
	if err := owner.AwaitNormalExit(ctx); err != nil {
		t.Fatal(err)
	}
	if err := owner.Close(ctx); err != nil {
		t.Fatal(err)
	}
	if _, _, err := owner.Query(ctx, request); !errors.Is(err, ErrClosed) {
		t.Fatalf("closed owner admitted another query: %v", err)
	}
	if err := owner.Close(ctx); err != nil {
		t.Fatalf("normal repeated Close failed: %v", err)
	}
}

func TestCancelledQueryDoesNotReadArtifactOrStart(t *testing.T) {
	owner := NewOwner("unused-manifest-because-operation-was-cancelled")
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	request := wire.WorkerAuthStatusRequest{SchemaVersion: 1, RequestId: "11111111-1111-4111-8111-111111111111", Method: "auth_status", ServiceId: "cue"}
	if _, _, err := owner.Query(ctx, request); !errors.Is(err, context.Canceled) {
		t.Fatalf("pre-cancelled query did not stop before artifact access: %v", err)
	}
	if owner.pending != nil {
		t.Fatal("cancelled query acquired a child")
	}
}

func TestQueuedQueryRespectsCancellationAndClose(t *testing.T) {
	owner := NewOwner("unused-manifest-because-owner-admission-is-closed")
	owner.gate <- struct{}{} // Ordinary serialization queue, no subprocess fixture.
	ctx, cancel := context.WithCancel(context.Background())
	result := make(chan error, 1)
	request := wire.WorkerAuthStatusRequest{SchemaVersion: 1, RequestId: "11111111-1111-4111-8111-111111111111", Method: "auth_status", ServiceId: "cue"}
	go func() { _, _, err := owner.Query(ctx, request); result <- err }()
	cancel()
	if err := <-result; !errors.Is(err, context.Canceled) {
		t.Fatalf("queued query ignored cancellation: %v", err)
	}
	if err := owner.Close(context.Background()); err != nil {
		t.Fatal(err)
	}
	<-owner.gate
	if _, _, err := owner.Query(context.Background(), request); !errors.Is(err, ErrClosed) {
		t.Fatalf("Close failed to close admission: %v", err)
	}
}
