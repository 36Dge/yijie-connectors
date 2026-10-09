//! Private worker-owned UI: secrets never enter the Desktop renderer.
use super::{FormState, escape, reply};
use axum::{http::StatusCode, response::Response};
use uuid::Uuid;

const STYLE: &str = include_str!("page.css");
const TOKENS: &str = include_str!("assets/tokens.css");
const SCRIPT: &str = include_str!("page.js");
const BRAND_LIGHT: &str = include_str!("assets/yijie-mark.svg");
const BRAND_DARK: &str = include_str!("assets/yijie-mark-dark.svg");
const SORFTIME: &str = include_str!("assets/sorftime.svg");

pub(super) enum Status {
    Submitted,
    Cancelled,
    Expired,
    Processed,
}

fn document(content: &str, script_nonce: &str) -> String {
    let artwork = &SORFTIME[SORFTIME.find("<svg").expect("reviewed SVG")..];
    format!(
        r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="color-scheme" content="light dark"><title>连接 Sorftime · 易界</title><style>{TOKENS}{STYLE}</style></head><body><header class="brand" aria-label="易界"><span class="brand-light" aria-hidden="true">{BRAND_LIGHT}</span><span class="brand-dark" aria-hidden="true">{BRAND_DARK}</span><span>易界</span></header><main><section class="card" aria-labelledby="title"><div class="artwork" aria-hidden="true">{artwork}</div>{content}</section></main><script nonce="{script_nonce}">{SCRIPT}</script></body></html>"#,
    )
}

fn response(status: StatusCode, content: &str) -> Response {
    let nonce = Uuid::new_v4().to_string();
    let mut result = reply(status, document(content, &nonce));
    result.headers_mut().insert("content-security-policy", format!("default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-{nonce}'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'").parse().expect("owned nonce"));
    result
}

pub(super) fn form(state: &FormState, error: Option<&str>) -> Response {
    let message = escape(error.unwrap_or_default());
    response(
        if error.is_some() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::OK
        },
        &format!(
            r#"<h1 id="title" aria-live="polite">连接 Sorftime</h1><p class="description" id="description">连接你的账号，在易界中使用 Sorftime 数据。</p>
<form method="post" action="{path}" autocomplete="off" data-remaining-ms="{remaining}">
<input type="hidden" name="nonce" value="{nonce}"><input type="hidden" name="name" value="Authorization"><input type="hidden" name="scheme" value="bearer">
<div class="field-heading"><label for="account-sk">Account-SK</label><a class="key-link" href="https://open.sorftime.com/mcp" target="_blank" rel="noopener noreferrer">获取 Account-SK ↗</a></div>
<div class="input-shell"><input id="account-sk" name="value" type="password" placeholder="粘贴你的 Account-SK" maxlength="4096" required autocomplete="off" autocapitalize="off" spellcheck="false" aria-describedby="key-hint field-error" aria-invalid="{invalid}"><button id="visibility" class="visibility" type="button" aria-label="显示 Account-SK" aria-pressed="false" aria-controls="account-sk" hidden>显示</button></div>
<p class="hint" id="key-hint">登录 Sorftime 后，在 MCP 页面复制 Account-SK。</p><p id="field-error" class="error" role="alert" {error_hidden}>{message}</p>
<div class="actions"><button id="cancel" class="button" type="submit" name="action" value="cancel" formnovalidate>取消</button><button id="save" class="button primary" type="submit">保存并连接</button></div></form>
<p class="privacy" id="privacy"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/></svg><span>密钥保存在本机系统钥匙串，不上传至易界云端。<br>仅用于向 Sorftime 验证身份。</span></p><p class="status-note" id="status-note" hidden>此页面可以关闭。</p>"#,
            path = escape(&state.path),
            nonce = escape(&state.nonce),
            remaining = state
                .deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis(),
            invalid = error.is_some(),
            error_hidden = if error.is_some() { "" } else { "hidden" },
        ),
    )
}

pub(super) fn status(code: StatusCode, state: Status) -> Response {
    let (title, description) = match state {
        Status::Submitted => ("连接请求已提交", "请回到易界查看 Sorftime 的连接结果。"),
        Status::Cancelled => ("已取消连接", "未保存本次输入的密钥。"),
        Status::Expired => ("连接页面已过期", "请回到易界，重新打开 Sorftime 连接页面。"),
        Status::Processed => (
            "连接页面已结束",
            "请回到易界查看连接状态，或重新打开连接页面。",
        ),
    };
    response(
        code,
        &format!(
            r#"<h1 id="title">{title}</h1><p class="description">{description}</p><p class="status-note">此页面可以关闭。</p>"#
        ),
    )
}
