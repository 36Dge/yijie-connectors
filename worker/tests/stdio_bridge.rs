//! Qualification of the fixed public in-process byte interface, with a separate
//! owner controlling real child stdin/normal wait. No Codex stdio launcher.
use codex_rmcp_client::{ElicitationResponse, InProcessTransportFactory, RmcpClient};
use futures::future::BoxFuture;
use rmcp::model::{ClientCapabilities, Implementation, InitializeRequestParams};
use sha2::{Digest, Sha256};
use std::io;
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncWriteExt, DuplexStream};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;

struct ChildOwner {
    child: Child,
    close_input: Option<oneshot::Sender<()>>,
    input_copy: JoinHandle<io::Result<u64>>,
    output_copy: JoinHandle<io::Result<u64>>,
}

struct ByteFactory {
    owner: Arc<Mutex<Option<ChildOwner>>>,
    opens: Arc<AtomicUsize>,
}

impl InProcessTransportFactory for ByteFactory {
    fn open(&self) -> BoxFuture<'static, io::Result<DuplexStream>> {
        let state = Arc::clone(&self.owner);
        let opens = Arc::clone(&self.opens);
        Box::pin(async move {
            let mut locked = state.lock().await;
            if locked.is_some() {
                return Err(io::Error::other(
                    "existing child must finish normal cleanup before reopen",
                ));
            }
            let mut child = Command::new(env!("CARGO_BIN_EXE_stdio-qualification-server"))
                .env_clear()
                .kill_on_drop(false)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("child stdin unavailable"))?;
            let mut stdout = child
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("child stdout unavailable"))?;
            let (client, bridge) = tokio::io::duplex(16 * 1024);
            let (mut from_client, mut to_client) = tokio::io::split(bridge);
            let (close_input, closed) = oneshot::channel();
            let input_copy = tokio::spawn(async move {
                let result = tokio::select! {
                    result = tokio::io::copy(&mut from_client, &mut stdin) => result,
                    _ = closed => Ok(0),
                };
                stdin.shutdown().await?;
                drop(stdin); // The server's documented normal EOF exit.
                result
            });
            let output_copy = tokio::spawn(async move {
                let result = tokio::io::copy(&mut stdout, &mut to_client).await;
                let _ = to_client.shutdown().await;
                result
            });
            opens.fetch_add(1, Ordering::SeqCst);
            *locked = Some(ChildOwner {
                child,
                close_input: Some(close_input),
                input_copy,
                output_copy,
            });
            Ok(client)
        })
    }
}

impl ByteFactory {
    async fn close_normally(&self) -> Result<serde_json::Value, String> {
        let mut state = self.owner.lock().await;
        let owner = state.as_mut().ok_or("child owner unavailable")?;
        let pid = owner.child.id();
        if let Some(close) = owner.close_input.take() {
            let _ = close.send(()); // Close stdin, not a process signal.
        }
        let exit = tokio::time::timeout(Duration::from_secs(5), owner.child.wait())
            .await
            .map_err(|_| format!("STOP_PENDING: pid={pid:?}; no force-kill fallback"))?
            .map_err(|_| "normal child wait failed".to_string())?;
        let input = tokio::time::timeout(Duration::from_secs(5), &mut owner.input_copy)
            .await
            .map_err(|_| "input byte bridge cleanup pending")?
            .map_err(|_| "input bridge task failed")?;
        let output = tokio::time::timeout(Duration::from_secs(5), &mut owner.output_copy)
            .await
            .map_err(|_| "output byte bridge cleanup pending")?
            .map_err(|_| "output bridge task failed")?;
        let observation = serde_json::json!({"pid":pid,"normalEOF":true,"exitCode":exit.code(),"inputBridgeEnded":input.is_ok(),"outputBridgeEnded":output.is_ok(),"childReaped":true});
        *state = None;
        if !exit.success() {
            return Err("ordinary server returned unsuccessful exit".into());
        }
        Ok(observation)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fixed_library_byte_bridge_uses_real_server_and_normal_eof() {
    let factory = Arc::new(ByteFactory {
        owner: Arc::new(Mutex::new(None)),
        opens: Arc::new(AtomicUsize::new(0)),
    });
    let client = RmcpClient::new_in_process_client(factory.clone())
        .await
        .expect("ordinary bridge start");
    let outcome = async {
        let initialized = client.initialize(
            InitializeRequestParams::new(ClientCapabilities::default(), Implementation::new("yijie-stdio-qualification", "0.1.0")),
            Some(Duration::from_secs(5)),
            Box::new(|_, _| Box::pin(async { Ok(ElicitationResponse { action: codex_rmcp_client::ElicitationAction::Decline, content: None, meta: None }) })),
        ).await.map_err(|_| "initialize failed")?;
        let tools = client.list_tools(None, Some(Duration::from_secs(5))).await.map_err(|_| "tools/list failed")?;
        if tools.tools.len() != 1 || tools.tools[0].name != "lookup" {
            return Err("unexpected tool catalog");
        }
        let result = client.call_tool("lookup".into(), Some(serde_json::json!({})), None, Some(Duration::from_secs(5))).await.map_err(|_| "lookup failed")?;
        let serialized = serde_json::to_value(result).map_err(|_| "result serialization failed")?;
        if serialized["content"][0]["text"] != "public-stdio-value" {
            return Err("lookup result differs");
        }
        Ok(serde_json::json!({"serverInfo":initialized.server_info,"tools":tools.tools,"lookup":serialized}))
    }.await;
    // The in-process recipe has no stdio_process and cannot reach Codex's
    // LocalProcessTerminator. The independent owner closes the real pipe.
    client.shutdown().await;
    drop(client);
    let cleanup = factory.close_normally().await;
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_stdio-qualification-server"));
    let bytes = std::fs::read(binary).expect("project-built server bytes");
    let report = serde_json::json!({
        "status": if outcome.is_ok() && cleanup.is_ok() {"PASS"} else {"INCOMPLETE"},
        "codexSource":"7fd463bcef07f37b0211acd9f62b9f93ea0a4b12",
        "transport":"public InProcessTransportFactory DuplexStream to separately owned stdio pipes",
        "serverBinary":binary,"serverSha256":format!("{:x}",Sha256::digest(bytes)),
        "spawnCount":factory.opens.load(Ordering::SeqCst),"operation":outcome,"cleanup":cleanup,
        "externalCalls":0,"credentialsReadOrWritten":0,"forceKillFallback":false,
        "realNpmPackages":"NOT RUN", "productionBridge":"NOT IMPLEMENTED"
    });
    if let Ok(path) = std::env::var("YIJIE_STDIO_QUALIFICATION_EVIDENCE") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap())
            .expect("save safe qualification evidence");
    }
    assert!(
        outcome.is_ok(),
        "ordinary MCP operation did not complete: {outcome:?}"
    );
    assert!(cleanup.is_ok(), "normal cleanup incomplete: {cleanup:?}");
    assert_eq!(factory.opens.load(Ordering::SeqCst), 1);
    eprintln!(
        "PASS: fixed Codex public byte bridge; initialize/list/lookup; real child normal EOF exit0 and bridge tasks joined; no external IO"
    );
}
