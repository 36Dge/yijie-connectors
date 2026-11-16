"""Normal local entry checks against actual Cargo-built binaries.

Only initialize, a read-only HTTP route check, local capability clearing and
normal shutdown/EOF are used. No provider setup, credentials, tools/call or model.
"""
import argparse
import http.client
import json
import os
from pathlib import Path
import secrets
import select
import subprocess
import uuid
from urllib.parse import urlsplit


def identifier():
    return str(uuid.uuid4())


def check_entry(binary, qualification, explicit_shutdown):
    capability = secrets.token_hex(32)
    # Do not forward provider credentials, model settings or inherited FD hints.
    environment = {
        "PATH": os.defpath,
        "LANG": "C.UTF-8",
        "YIJIE_MARKET_RUNTIME_CAPABILITY": capability,
    }
    child = subprocess.Popen(
        [str(binary), "--broker-control-stdio"],
        env=environment,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        close_fds=True,
    )
    print(json.dumps({"started_pid": child.pid, "qualification": qualification}), flush=True)

    def exchange(value):
        child.stdin.write(json.dumps(value).encode() + b"\n")
        child.stdin.flush()
        readable, _, _ = select.select([child.stdout], [], [], 10)
        if not readable:
            raise RuntimeError("Local control reply did not arrive")
        line = child.stdout.readline(65537)
        if not line.endswith(b"\n") or len(line) > 65536:
            raise RuntimeError("Local control reply is incomplete")
        return json.loads(line)

    def request(method, payload):
        request_id = identifier()
        response = exchange({"schemaVersion": 1, "requestId": request_id,
                             "method": method, "payload": payload})
        assert response.get("requestId") == request_id
        assert "error" not in response, response
        return response["data"]

    try:
        process = {"hostInstanceId": identifier(), "runtimeGeneration": identifier()}
        initialized = request("initialize", {"operationId": identifier(), "process": process})
        assert initialized["process"] == process
        assert initialized["externalCallsEnabled"] is (not qualification)
        endpoint = urlsplit(initialized["gatewayUrl"])
        assert endpoint.scheme == "http" and endpoint.hostname == "127.0.0.1"
        assert endpoint.port and endpoint.path == "/mcp"

        connection = http.client.HTTPConnection(endpoint.hostname, endpoint.port, timeout=5)
        try:
            # No MCP session or tool request is created by this route check.
            connection.request("GET", "/builtin-web", headers={"Authorization": "Bearer " + capability})
            response = connection.getresponse()
            status = response.status
            response.close()
        finally:
            connection.close()
        if qualification:
            assert status == 404, status
        else:
            # The official rmcp route handles the request and can reject its
            # missing session/Accept header. A missing route must never pass.
            assert status in {200, 400, 405, 406}, status

        cleared = exchange({"method": "web_capability_clear",
                            "process_epoch": identifier(), "binding_epoch": identifier()})
        if qualification:
            # Broker safe_error is the existing flat ErrorResponse, not a
            # JSON-RPC error envelope or a Web control acknowledgement.
            assert cleared == {"schemaVersion": 1, "code": "invalid_request", "retryable": False}, cleared
        else:
            assert cleared["state"] == "applied", cleared

        if explicit_shutdown:
            request("shutdown", {"operationId": identifier(), "process": process,
                                 "reason": "owner_shutdown"})
    finally:
        child.stdin.close()
        delayed = False
        try:
            code = child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            delayed = True
            print(json.dumps({"state": "STOP_PENDING", "pid": child.pid,
                              "action": "stdin closed; retain owner and wait; no signal sent"}), flush=True)
            # Keep the owner and its pipes alive; never escalate to terminate/kill.
            code = child.wait()
        stderr = child.stderr.read()
        child.stdout.close()
        child.stderr.close()
        assert not delayed, "Normal cleanup exceeded the observation window"
        assert code == 0, f"Normal exit failed: code={code}; stderr_bytes={len(stderr)}"
    return {"binary": binary.name, "qualification": qualification,
            "stop": "shutdown_then_eof" if explicit_shutdown else "eof",
            "builtin_web_http_status": status, "exit_code": code,
            "provider_or_model_calls": 0, "signals_sent": 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True,
                        help="The explicit isolated Cargo target directory just built")
    args = parser.parse_args()
    target = args.target_dir.resolve(strict=True)
    results = []
    for name, qualification in [("market-broker-qualification", True), ("yijie-mcp-worker", False)]:
        binary = target / "debug" / name
        if not binary.is_file() or binary.is_symlink():
            raise ValueError("Expected a regular Cargo-built binary: " + str(binary))
        for shutdown in (False, True):
            results.append(check_entry(binary, qualification, shutdown))
    print(json.dumps({"passed": len(results), "results": results}, indent=2))


if __name__ == "__main__":
    main()
