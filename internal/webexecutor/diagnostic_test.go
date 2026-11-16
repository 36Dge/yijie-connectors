package webexecutor

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func TestWebDiagnosticPreservesTimeoutOriginWithBoundedTypedRecords(t *testing.T) {
	var output bytes.Buffer
	operation := "019a8811-0000-7000-8000-000000000175"
	w := newDiagnosticWriter(operation, &output)
	response := `{"schemaVersion":1,"unixMs":1794811000000,"stage":"web_http_response","event":"response","httpStatus":504,"safeFailure":null}` + "\n"
	_, _ = w.Write([]byte(response[:30]))
	_, _ = w.Write([]byte(response[30:]))
	_, _ = w.Write([]byte(`{"schemaVersion":1,"unixMs":1794811000001,"stage":"web_http_response","event":"expired","httpStatus":null,"safeFailure":null}` + "\n"))
	// Ordinary non-web startup output is not a diagnostic event.
	_, _ = w.Write([]byte("worker ready\n"))
	_, _ = w.Write([]byte(strings.Repeat(response, 10)))
	lines := bytes.Split(bytes.TrimSpace(output.Bytes()), []byte("\n"))
	if len(lines) != 8 {
		t.Fatalf("unexpected bounded diagnostic count: %d", len(lines))
	}
	for i, line := range lines {
		var record map[string]any
		if json.Unmarshal(line, &record) != nil || record["operation_id"] != operation || record["component"] != "builtin_web" {
			t.Fatal("diagnostic lost its owner-supplied operation association")
		}
		if i == 1 {
			if record["event"] != "expired" || record["httpStatus"] != nil {
				t.Fatal("local timeout must remain distinct from an HTTP status")
			}
		} else if record["event"] != "response" || record["httpStatus"] != float64(504) {
			t.Fatal("upstream timeout status was lost")
		}
	}
}
