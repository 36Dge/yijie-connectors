package webexecutor

import (
	"bytes"
	"encoding/json"
	"io"
	"sync"

	"github.com/google/uuid"
)

type webDiagnostic struct {
	SchemaVersion int     `json:"schemaVersion"`
	UnixMS        int64   `json:"unixMs"`
	Stage         string  `json:"stage"`
	Event         string  `json:"event"`
	HTTPStatus    *int    `json:"httpStatus"`
	SafeFailure   *string `json:"safeFailure"`
}

// Only the existing adapter's typed web observations cross the stderr boundary.
// Do not forward raw worker output, provider bodies, URLs, headers or credentials.
type diagnosticWriter struct {
	mu        sync.Mutex
	operation string
	target    io.Writer
	line      []byte
	dropped   bool
	count     int
}

func newDiagnosticWriter(operation string, target io.Writer) io.Writer {
	id, err := uuid.Parse(operation)
	if err != nil || id.Version() != 7 || id.String() != operation {
		return io.Discard
	}
	return &diagnosticWriter{operation: operation, target: target}
}

func (w *diagnosticWriter) Write(p []byte) (int, error) {
	w.mu.Lock()
	defer w.mu.Unlock()
	for _, b := range p {
		if b == '\n' {
			if !w.dropped && w.count < 8 {
				w.emit()
			}
			w.line, w.dropped = w.line[:0], false
		} else if len(w.line) < 512 && !w.dropped {
			w.line = append(w.line, b)
		} else {
			w.dropped = true
		}
	}
	return len(p), nil
}

func (w *diagnosticWriter) emit() {
	var record webDiagnostic
	decoder := json.NewDecoder(bytes.NewReader(w.line))
	decoder.DisallowUnknownFields()
	if decoder.Decode(&record) != nil || record.SchemaVersion != 1 || record.UnixMS <= 0 || record.SafeFailure != nil {
		return
	}
	if decoder.Decode(new(any)) != io.EOF {
		return
	}
	valid := false
	switch record.Stage {
	case "web_http_send":
		valid = record.Event == "request" && record.HTTPStatus == nil
	case "web_http_response":
		valid = record.Event == "response" && record.HTTPStatus != nil && *record.HTTPStatus >= 100 && *record.HTTPStatus <= 599
		valid = valid || ((record.Event == "expired" || record.Event == "transport_unavailable") && record.HTTPStatus == nil)
	case "web_normalize":
		valid = record.Event == "succeeded" && record.HTTPStatus == nil
	}
	if !valid {
		return
	}
	entry := struct {
		Component   string `json:"component"`
		OperationID string `json:"operation_id"`
		webDiagnostic
	}{"builtin_web", w.operation, record}
	encoded, err := json.Marshal(entry)
	if err == nil {
		w.count++
		_, _ = w.target.Write(append(encoded, '\n'))
	}
}
