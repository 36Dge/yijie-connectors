//! Ordinary project-built MCP qualification server. No external IO or fixtures
//! replace any installed executable. stdin EOF is its normal exit protocol.
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let input = io::stdin().lock();
    let mut output = io::stdout().lock();
    for line in input.lines() {
        let request: serde_json::Value = serde_json::from_str(&line?)?;
        let Some(id) = request.get("id") else {
            continue;
        };
        let result = match request["method"].as_str() {
            Some("initialize") => serde_json::json!({
                "protocolVersion": request["params"]["protocolVersion"],
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "yijie-ordinary-stdio-qualification", "version": "1.0.0"}
            }),
            Some("tools/list") => serde_json::json!({"tools": [{
                "name": "lookup", "description": "Read an ordinary public synthetic value.",
                "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
                "annotations": {"readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false}
            }]}),
            Some("tools/call") if request["params"]["name"] == "lookup" => {
                serde_json::json!({"content": [{"type": "text", "text": "public-stdio-value"}], "isError": false})
            }
            Some("ping") => serde_json::json!({}),
            _ => {
                let response = serde_json::json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32601,"message":"Unsupported ordinary qualification method"}});
                serde_json::to_writer(&mut output, &response)?;
                output.write_all(b"\n")?;
                output.flush()?;
                continue;
            }
        };
        serde_json::to_writer(
            &mut output,
            &serde_json::json!({"jsonrpc":"2.0","id":id,"result":result}),
        )?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
