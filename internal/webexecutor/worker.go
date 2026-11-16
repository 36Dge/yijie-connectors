// Package webexecutor owns service-side Kimi execution. Supplier keys never
// enter the public API, Desktop, Host, MCP arguments or Runtime environment.
package webexecutor

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"os"
	"os/exec"
	"strings"
	"sync"
	"syscall"
	"time"

	wire "github.com/36Dge/yijie-connectors/internal/contracts/builtinweb"
	"github.com/36Dge/yijie-connectors/internal/worker"
)

var unavailable = errors.New("managed web executor unavailable")
var stopPending = errors.New("managed web executor normal exit pending")

type Worker struct {
	artifact worker.Artifact
	key      []byte
	slots    chan struct{}
}

// Wait retains ownership of normally stopping workers. It never kills a
// process or releases a concurrency slot before Wait observes its real exit.
func (w *Worker) Wait(ctx context.Context) error {
	tick := time.NewTicker(20 * time.Millisecond)
	defer tick.Stop()
	for len(w.slots) > 0 {
		select {
		case <-ctx.Done():
			return stopPending
		case <-tick.C:
		}
	}
	return nil
}

func NewWorker(manifest, keyPath string) (*Worker, error) {
	artifact, err := worker.LoadArtifact(manifest)
	if err != nil || artifact.ProviderProfile != "generic-mcp-v1" {
		return nil, unavailable
	}
	// Reuse the existing owned startup pipe; load only an operator-mounted,
	// owner-only regular secret file. No client-side credential fallback.
	file, err := os.OpenFile(keyPath, os.O_RDONLY|syscall.O_NOFOLLOW, 0)
	if err != nil {
		return nil, unavailable
	}
	defer file.Close()
	info, err := file.Stat()
	if err != nil || !info.Mode().IsRegular() || info.Mode().Perm()&0077 != 0 || info.Size() < 1 || info.Size() > 16384 {
		return nil, unavailable
	}
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !ok || stat.Uid != uint32(os.Getuid()) || stat.Nlink != 1 {
		return nil, unavailable
	}
	key, err := io.ReadAll(io.LimitReader(file, 16385))
	if err != nil {
		return nil, unavailable
	}
	key = bytes.TrimSpace(key)
	if len(key) == 0 || len(key) > 16384 || bytes.IndexFunc(key, func(r rune) bool { return r <= 32 || r == 127 }) >= 0 {
		return nil, unavailable
	}
	return &Worker{artifact: artifact, key: append([]byte(nil), key...), slots: make(chan struct{}, 2)}, nil
}

type output struct {
	mu    sync.Mutex
	data  []byte
	done  bool
	ready chan []byte
}

func (o *output) Write(p []byte) (int, error) {
	o.mu.Lock()
	defer o.mu.Unlock()
	if o.done {
		return len(p), nil
	}
	remaining := 65537 - len(o.data)
	if len(p) > remaining {
		o.data = append(o.data, p[:remaining]...)
	} else {
		o.data = append(o.data, p...)
	}
	if bytes.ContainsRune(o.data, '\n') || len(o.data) > 65536 {
		o.done = true
		o.ready <- append([]byte(nil), o.data...)
	}
	return len(p), nil
}
func (w *Worker) Execute(ctx context.Context, q wire.ExecutorWorkerRequest) ([]byte, error) {
	select {
	case w.slots <- struct{}{}:
	default:
		return nil, unavailable
	}
	started := false
	defer func() {
		if !started {
			<-w.slots
		}
	}()
	payload, err := json.Marshal(q)
	if err != nil || wire.Validate("ExecutorWorkerRequest", payload) != nil {
		return nil, unavailable
	}
	if ctx.Err() != nil || time.Now().UnixMilli() >= int64(q.Receipt.DeadlineUnixMs) {
		return nil, unavailable
	}
	keyReader, keyWriter, err := os.Pipe()
	if err != nil {
		return nil, unavailable
	}
	defer keyReader.Close()
	command := exec.Command(w.artifact.Binary, "--builtin-web-execute-stdio")
	command.Env = []string{wire.CredentialPipeEnvironmentKey + "=3"}
	command.ExtraFiles = []*os.File{keyReader}
	input, err := command.StdinPipe()
	if err != nil {
		keyWriter.Close()
		return nil, unavailable
	}
	defer input.Close()
	out := &output{ready: make(chan []byte, 1)}
	command.Stdout = out
	command.Stderr = newDiagnosticWriter(q.Receipt.OperationId, os.Stderr)
	if err = command.Start(); err != nil {
		keyWriter.Close()
		return nil, unavailable
	}
	started = true
	done := make(chan error, 1)
	go func() { done <- command.Wait(); <-w.slots }()
	go func() {
		defer keyWriter.Close()
		_, _ = io.Copy(keyWriter, strings.NewReader(wire.CredentialPipePreamble+string(w.key)+wire.CredentialPipeTerminator))
	}()
	if _, err = input.Write(append(payload, '\n')); err != nil {
		input.Close()
		return nil, unavailable
	}
	var body []byte
	select {
	case body = <-out.ready:
	case <-ctx.Done():
		input.Close()
		select {
		case <-done:
			return nil, ctx.Err()
		case <-time.After(2 * time.Second):
			return nil, stopPending
		}
	case <-done:
		return nil, unavailable
	}
	input.Close() // EOF completes the worker's normal ownership handshake.
	select {
	case err = <-done:
	case <-ctx.Done():
		return nil, stopPending
	}
	if err != nil || len(body) > 65536 || wire.Validate("ToolOutput", body) != nil {
		return nil, unavailable
	}
	return bytes.TrimSpace(body), nil
}
