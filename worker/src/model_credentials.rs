//! One owned startup pipe, consumed before the product worker starts threads.
//! No vault, filesystem key lookup, network request, or public secret field.
use crate::{broker_generated as wire, credentials::HeaderCredential};
use std::{fs::File, io::Read, os::fd::FromRawFd, os::unix::fs::FileTypeExt, sync::OnceLock};

static KIMI: OnceLock<Option<HeaderCredential>> = OnceLock::new();

// Called exactly once by synchronous product main, before the Tokio runtime.
pub fn capture_startup_source() {
    let hint = std::env::var(wire::KIMI_MODEL_CREDENTIAL_ENVIRONMENT_KEY).ok();
    // SAFETY: product main has not created threads or child processes yet.
    unsafe { std::env::remove_var(wire::KIMI_MODEL_CREDENTIAL_ENVIRONMENT_KEY) };
    let credential = (|| {
        if hint.as_deref() != Some("3") {
            return None;
        }
        // Opening /dev/fd duplicates an existing descriptor without adopting an
        // invalid fd. Adopt the original before checking its type so every
        // validated-descriptor return path closes the inherited handle.
        let duplicate = File::open(format!(
            "/dev/fd/{}",
            wire::KIMI_MODEL_CREDENTIAL_DESCRIPTOR
        ))
        .ok()?;
        // SAFETY: fd 3 was inherited exclusively by this single-threaded entry;
        // the successful duplicate proves it exists. No Rust owner precedes us.
        let original = unsafe { File::from_raw_fd(wire::KIMI_MODEL_CREDENTIAL_DESCRIPTOR as i32) };
        drop(duplicate);
        if !original.metadata().ok()?.file_type().is_fifo() {
            return None; // the adopted original is closed on every return
        }
        parse_source(original)
    })();
    let _ = KIMI.set(credential);
}

fn parse_source(source: impl Read) -> Option<HeaderCredential> {
    let limit = wire::KIMI_MODEL_CREDENTIAL_PREAMBLE.len()
        + wire::KIMI_MODEL_CREDENTIAL_MAX_KEY_BYTES
        + wire::KIMI_MODEL_CREDENTIAL_TERMINATOR.len();
    let mut bytes = Vec::new();
    source
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > limit {
        return None;
    }
    let text = std::str::from_utf8(&bytes).ok()?;
    let key = text
        .strip_prefix(wire::KIMI_MODEL_CREDENTIAL_PREAMBLE)?
        .strip_suffix(wire::KIMI_MODEL_CREDENTIAL_TERMINATOR)?;
    HeaderCredential::from_kimi_model(key).ok()
}

pub fn kimi() -> Option<HeaderCredential> {
    KIMI.get().and_then(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_source_preserves_the_value_and_requires_complete_framing() {
        let ordinary = format!(
            "{}ordinary-model-value{}",
            wire::KIMI_MODEL_CREDENTIAL_PREAMBLE,
            wire::KIMI_MODEL_CREDENTIAL_TERMINATOR
        );
        let value = parse_source(ordinary.as_bytes()).unwrap();
        assert_eq!(value.name, "Authorization");
        assert_eq!(value.value, "Bearer ordinary-model-value");
        assert!(parse_source(&b""[..]).is_none());
        assert!(parse_source(ordinary.trim_end().as_bytes()).is_none());
        assert!(
            parse_source(format!("{}\n", wire::KIMI_MODEL_CREDENTIAL_PREAMBLE).as_bytes())
                .is_none()
        );
        let key = "a".repeat(wire::KIMI_MODEL_CREDENTIAL_MAX_KEY_BYTES);
        assert!(
            parse_source(format!("{}{key}\n", wire::KIMI_MODEL_CREDENTIAL_PREAMBLE).as_bytes())
                .is_some()
        );
    }
}
