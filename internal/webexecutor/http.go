package webexecutor

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"net/http"
	"time"

	wire "github.com/36Dge/yijie-connectors/internal/contracts/builtinweb"
	"github.com/google/uuid"
)

type Runner interface {
	Execute(context.Context, wire.ExecutorWorkerRequest) ([]byte, error)
}
type Handler struct {
	Client    *http.Client
	APIOrigin string
	Worker    Runner
	Instance  uuid.UUID
}

func (h *Handler) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /readyz", func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusNoContent) })
	mux.HandleFunc("POST /internal/v1/web-retrieval/execute", h.execute)
	return mux
}
func write(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-store")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}
func failure(w http.ResponseWriter, code string, attempt bool) {
	write(w, 503, wire.APIError{Error: wire.AppError{Code: wire.AppErrorCode(code), Message: wire.ErrorMessages[code]}, AttemptDispatched: attempt})
}
func (h *Handler) call(ctx context.Context, path, name string, request, response any) error {
	body, err := json.Marshal(request)
	if err != nil {
		return unavailable
	}
	req, err := http.NewRequestWithContext(ctx, "POST", h.APIOrigin+path, bytes.NewReader(body))
	if err != nil {
		return unavailable
	}
	req.Header.Set("Content-Type", "application/json")
	res, err := h.Client.Do(req)
	if err != nil {
		return unavailable
	}
	defer res.Body.Close()
	if res.StatusCode != 200 {
		return unavailable
	}
	raw, err := io.ReadAll(io.LimitReader(res.Body, 16385))
	if err != nil || len(raw) > 16384 || wire.Validate(name, raw) != nil {
		return unavailable
	}
	if json.Unmarshal(raw, response) != nil {
		return unavailable
	}
	return nil
}
func (h *Handler) execute(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Origin") != "" {
		failure(w, "not_entitled", false)
		return
	}
	raw, err := io.ReadAll(io.LimitReader(r.Body, 16385))
	defer r.Body.Close()
	if err != nil || len(raw) > 16384 || wire.Validate("InternalExecute", raw) != nil {
		failure(w, "invalid_arguments", false)
		return
	}
	var q wire.InternalExecute
	if json.Unmarshal(raw, &q) != nil {
		failure(w, "invalid_arguments", false)
		return
	}
	encoded, _ := q.Request.MarshalJSON()
	var fields map[string]json.RawMessage
	_ = json.Unmarshal(encoded, &fields)
	var id, tool string
	_ = json.Unmarshal(fields["operation_id"], &id)
	_ = json.Unmarshal(fields["tool"], &tool)
	var arguments any
	decoder := json.NewDecoder(bytes.NewReader(fields["arguments"]))
	decoder.UseNumber()
	if decoder.Decode(&arguments) != nil {
		failure(w, "invalid_arguments", false)
		return
	}
	canonical, err := json.Marshal(arguments)
	if err != nil {
		failure(w, "invalid_arguments", false)
		return
	}
	sum := sha256.Sum256(append(append([]byte(tool), 0), canonical...))
	deadline := time.UnixMilli(int64(q.DeadlineUnixMs))
	if hex.EncodeToString(sum[:]) != q.ParameterDigest || !deadline.After(time.Now()) || time.Until(deadline) > 45*time.Second {
		failure(w, "operation_conflict", false)
		return
	}
	ctx, cancel := context.WithDeadline(r.Context(), deadline)
	defer cancel()
	var receipt wire.ClaimReceipt
	if h.call(ctx, "/internal/v1/web-retrieval/claim", "ClaimReceipt", wire.ClaimRequest{OperationId: id, ParameterDigest: q.ParameterDigest, ExecutorInstance: h.Instance}, &receipt) != nil {
		failure(w, "upstream_unavailable", false)
		return
	}
	if receipt.OperationId != id || receipt.ParameterDigest != q.ParameterDigest || receipt.ExecutorInstance != h.Instance || receipt.DeadlineUnixMs != q.DeadlineUnixMs {
		failure(w, "outcome_unknown", true)
		return
	}
	result, executionErr := h.Worker.Execute(ctx, wire.ExecutorWorkerRequest{Execute: q, Receipt: receipt})
	state, dispatch := "completed", "completed"
	if executionErr != nil || wire.Validate("ToolOutput", result) != nil {
		state, dispatch = "unknown", "may_have_dispatched"
	}
	// A transport/cancellation error after the claim is never proof of no charge.
	var content map[string]json.RawMessage
	if json.Unmarshal(result, &content) == nil && content["error"] != nil {
		state, dispatch = "unknown", "may_have_dispatched"
	}
	settlement, stop := context.WithTimeout(context.WithoutCancel(ctx), 3*time.Second)
	defer stop()
	var status wire.OperationStatus
	if h.call(settlement, "/internal/v1/web-retrieval/finish", "OperationStatus", wire.ClaimFinish{Receipt: receipt, State: wire.ClaimFinishState(state), Dispatch: wire.DispatchFact(dispatch)}, &status) != nil {
		failure(w, "outcome_unknown", true)
		return
	}
	response := wire.ExecutionResult{Operation: status}
	if executionErr == nil && wire.Validate("ToolOutput", result) == nil {
		var output wire.ToolOutput
		if output.UnmarshalJSON(result) == nil {
			response.Result = &output
			response.Operation.ResultAvailability = "available"
		}
	}
	write(w, 200, response)
}
