use super::Failure;
use crate::credentials::escape;
use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use uuid::Uuid;

enum Mark {
    Success,
    Attention,
    Time,
    Closed,
}

pub(super) fn result(name: &str, outcome: Result<(), Failure>) -> Response {
    let connector = format!("<strong><bdi>{}</bdi></strong>", escape(name));
    let (title, detail, mark) = match outcome {
        Ok(()) => (
            "授权成功",
            format!("你已授权易界访问{connector}。"),
            Mark::Success,
        ),
        Err(Failure::Cancelled) => (
            "授权已取消",
            format!("如需连接{connector}，请返回易界重新授权。"),
            Mark::Closed,
        ),
        Err(Failure::Expired) => (
            "授权已过期",
            format!("请返回易界，重新发起{connector}的授权。"),
            Mark::Time,
        ),
        Err(Failure::VaultUnavailable) => (
            "授权未能保存",
            format!("请检查本机钥匙串，然后返回易界重新连接{connector}。"),
            Mark::Attention,
        ),
        Err(Failure::CleanupPending) => (
            "请在易界确认授权状态",
            format!("易界仍在处理{connector}的授权，请返回应用查看进度。"),
            Mark::Time,
        ),
        Err(Failure::NotAuthorized | Failure::InvalidMetadata | Failure::Unavailable) => (
            "授权未完成",
            format!("易界未能完成{connector}的授权。请返回应用后重试。"),
            Mark::Attention,
        ),
    };
    // A 200 acknowledges the browser redirect; the page shows the actual outcome.
    render(StatusCode::OK, title, &detail, mark)
}

pub(super) fn already_handled(name: &str) -> Response {
    render(
        StatusCode::GONE,
        "此授权链接已使用",
        &format!(
            "请返回易界，查看<strong><bdi>{}</bdi></strong>的授权状态。",
            escape(name)
        ),
        Mark::Time,
    )
}

fn render(status: StatusCode, title: &str, detail: &str, mark: Mark) -> Response {
    let (tone, icon) = match mark {
        Mark::Success => ("success", "M5 12l4 4L19 6"),
        Mark::Attention => ("attention", "M12 5v9m0 4v.1"),
        Mark::Time => (
            "neutral",
            "M12 7v5l3 2 M21 12a9 9 0 1 1-18 0a9 9 0 1 1 18 0",
        ),
        Mark::Closed => ("neutral", "m7 7 10 10M17 7 7 17"),
    };
    let nonce = Uuid::new_v4().simple().to_string();
    let page = format!(
        include_str!("callback.html"),
        title = title,
        detail = detail,
        tone = tone,
        icon = icon,
        nonce = nonce,
        brand = include_str!("../credentials/assets/yijie-mark.svg"),
        tokens = include_str!("../credentials/assets/tokens.css"),
        style = include_str!("callback.css"),
    );
    let policy = format!(
        "default-src 'none'; style-src 'nonce-{nonce}'; script-src 'nonce-{nonce}'; img-src data:; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
    );
    (
        status,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CONTENT_SECURITY_POLICY, &policy),
        ],
        Html(page),
    )
        .into_response()
}
