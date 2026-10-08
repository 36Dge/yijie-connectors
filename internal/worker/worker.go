// Package worker owns the status-only child through stdin EOF. It never uses
// exec.CommandContext, Process.Kill, signals or a substituted executable.
package worker

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	wire "github.com/36Dge/yijie-connectors/internal/contracts/marketconnectors"
)

var ErrStopPending = errors.New("worker normal EOF exit is not confirmed")
var ErrClosed = errors.New("worker owner is closed")

// Artifact is a private canonical build manifest, never a renderer DTO.
type Artifact struct {
	SchemaVersion        int    `json:"schemaVersion"`
	CandidateOnly        bool   `json:"candidateOnly"`
	Binary               string `json:"binary"`
	SHA256               string `json:"sha256"`
	SizeBytes            int64  `json:"sizeBytes"`
	CodexSource          string `json:"codexSource"`
	ExternalCallsEnabled bool   `json:"externalCallsEnabled"`
	QualificationOnly    bool   `json:"qualificationOnly,omitempty"`
	ProviderProfile      string `json:"providerProfile,omitempty"`
}

func LoadArtifact(path string) (Artifact, error) {
	var value Artifact
	data, err := os.ReadFile(path)
	if err != nil {
		return value, errors.New("worker build manifest unavailable")
	}
	decoder := json.NewDecoder(bytes.NewReader(data))
	decoder.DisallowUnknownFields()
	if decoder.Decode(&value) != nil || !value.CandidateOnly || value.QualificationOnly || !((value.SchemaVersion == 1 && !value.ExternalCallsEnabled && value.ProviderProfile == "") || (value.SchemaVersion == 2 && value.ExternalCallsEnabled && (value.ProviderProfile == "tushare-oauth-v1" || value.ProviderProfile == "tushare-daily-v1"))) || value.CodexSource != "7fd463bcef07f37b0211acd9f62b9f93ea0a4b12" {
		return Artifact{}, errors.New("worker build manifest is unsupported")
	}
	if !filepath.IsAbs(value.Binary) || filepath.Base(value.Binary) != "yijie-mcp-worker" {
		return Artifact{}, errors.New("worker path is invalid")
	}
	info, err := os.Lstat(value.Binary)
	if err != nil || !info.Mode().IsRegular() || info.Size() != value.SizeBytes {
		return Artifact{}, errors.New("worker artifact is unavailable")
	}
	file, err := os.Open(value.Binary)
	if err != nil {
		return Artifact{}, errors.New("worker artifact is unreadable")
	}
	defer file.Close()
	hash := sha256.New()
	if _, err := io.Copy(hash, file); err != nil || hex.EncodeToString(hash.Sum(nil)) != value.SHA256 {
		return Artifact{}, errors.New("worker artifact differs from build manifest")
	}
	return value, nil
}

type boundedOutput struct {
	data     []byte
	limit    int
	exceeded bool
}

func (b *boundedOutput) Write(data []byte) (int, error) {
	remaining := b.limit - len(b.data)
	if len(data) > remaining {
		b.exceeded = true
		b.data = append(b.data, data[:remaining]...)
	} else {
		b.data = append(b.data, data...)
	}
	// Keep draining output so normal EOF cleanup cannot deadlock on a full pipe.
	return len(data), nil
}

type pending struct {
	command *exec.Cmd
	done    chan struct{}
	err     error // Written before done closes; all readers wait for done.
	stdout  *boundedOutput
	request wire.WorkerAuthStatusRequest
}

// Owner serializes operations and retains a pending child on timeout. A later
// query observes normal exit before another child may be created.
type Owner struct {
	mu       sync.Mutex
	gate     chan struct{}
	manifest string
	pending  *pending
	closed   bool
}

func NewOwner(manifest string) *Owner {
	return &Owner{manifest: manifest, gate: make(chan struct{}, 1)}
}

func (o *Owner) Query(ctx context.Context, request wire.WorkerAuthStatusRequest) (wire.WorkerAuthStatusResponse, *wire.WorkerError, error) {
	if err := ctx.Err(); err != nil {
		return wire.WorkerAuthStatusResponse{}, nil, err
	}
	if err := request.Validate(); err != nil {
		return wire.WorkerAuthStatusResponse{}, nil, errors.New("invalid worker request")
	}
	select {
	case o.gate <- struct{}{}:
		defer func() { <-o.gate }()
	case <-ctx.Done():
		return wire.WorkerAuthStatusResponse{}, nil, ctx.Err()
	}
	o.mu.Lock()
	if o.closed {
		o.mu.Unlock()
		return wire.WorkerAuthStatusResponse{}, nil, ErrClosed
	}
	if err := ctx.Err(); err != nil {
		o.mu.Unlock()
		return wire.WorkerAuthStatusResponse{}, nil, err
	}
	if o.pending != nil {
		select {
		case <-o.pending.done:
			o.pending = nil
		default:
			o.mu.Unlock()
			return wire.WorkerAuthStatusResponse{}, nil, ErrStopPending
		}
	}
	o.mu.Unlock()
	artifact, err := LoadArtifact(o.manifest)
	if err != nil {
		return wire.WorkerAuthStatusResponse{}, nil, err
	}
	encoded, err := json.Marshal(request)
	if err != nil {
		return wire.WorkerAuthStatusResponse{}, nil, errors.New("worker request unavailable")
	}
	command := exec.Command(artifact.Binary)
	command.Env = []string{"PATH=/usr/bin:/bin:/usr/sbin:/sbin", "LANG=en_US.UTF-8"}
	command.Stdin = strings.NewReader(string(encoded) + "\n") // EOF after this one request.
	stdout := &boundedOutput{limit: 32 * 1024}
	command.Stdout = stdout
	command.Stderr = &boundedOutput{limit: 4096} // Never publish child diagnostic text.
	// Close and process admission have one linearization point. Artifact IO
	// happens outside this lock; a cancelled or closed owner never starts after it.
	o.mu.Lock()
	if o.closed {
		o.mu.Unlock()
		return wire.WorkerAuthStatusResponse{}, nil, ErrClosed
	}
	if err := ctx.Err(); err != nil {
		o.mu.Unlock()
		return wire.WorkerAuthStatusResponse{}, nil, err
	}
	if err := command.Start(); err != nil {
		o.mu.Unlock()
		return wire.WorkerAuthStatusResponse{}, nil, errors.New("worker start unavailable")
	}
	entry := &pending{command: command, done: make(chan struct{}), stdout: stdout, request: request}
	o.pending = entry
	o.mu.Unlock()
	go func() {
		entry.err = command.Wait()
		close(entry.done)
	}()
	timer := time.NewTimer(10 * time.Second)
	defer timer.Stop()
	select {
	case <-entry.done:
		o.clearExited(entry)
		if entry.err != nil || stdout.exceeded {
			return wire.WorkerAuthStatusResponse{}, nil, errors.New("worker normal response unavailable")
		}
	case <-ctx.Done():
		return wire.WorkerAuthStatusResponse{}, nil, ErrStopPending
	case <-timer.C:
		return wire.WorkerAuthStatusResponse{}, nil, ErrStopPending
	}
	var reply wire.WorkerAuthStatusResponse
	if err := json.Unmarshal(stdout.data, &reply); err == nil {
		if reply.RequestId != request.RequestId || reply.Data.ServiceId != request.ServiceId {
			return wire.WorkerAuthStatusResponse{}, nil, errors.New("worker response binding mismatch")
		}
		return reply, nil, nil
	}
	var failure wire.WorkerError
	if err := json.Unmarshal(stdout.data, &failure); err == nil && failure.RequestId != nil && *failure.RequestId == request.RequestId {
		return wire.WorkerAuthStatusResponse{}, &failure, nil
	}
	return wire.WorkerAuthStatusResponse{}, nil, errors.New("worker response is invalid")
}

// AwaitNormalExit is used by the owning application during its ordinary exit.
// On timeout ownership stays retained; the caller must report cleanup pending.
func (o *Owner) AwaitNormalExit(ctx context.Context) error {
	o.mu.Lock()
	entry := o.pending
	o.mu.Unlock()
	if entry == nil {
		return nil
	}
	select {
	case <-entry.done:
		o.clearExited(entry)
		return nil
	case <-ctx.Done():
		return ErrStopPending
	}
}

// Close permanently closes admission before waiting for the already EOF-bound
// child. A timeout retains ownership; another Close may finish waiting, but no
// subsequent Query can revive this owner. AwaitNormalExit alone only waits.
func (o *Owner) Close(ctx context.Context) error {
	o.mu.Lock()
	o.closed = true
	o.mu.Unlock()
	return o.AwaitNormalExit(ctx)
}

func (o *Owner) clearExited(entry *pending) {
	o.mu.Lock()
	defer o.mu.Unlock()
	if o.pending == entry {
		o.pending = nil
	}
}
