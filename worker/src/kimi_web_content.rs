//! Normalization for explicitly opted-in body/Pro results. Legacy stays separate.
use super::*;

fn string(row: &Value, key: &str) -> Result<String, ErrorCode> {
    match row.get(key) {
        None => Ok(String::new()),
        Some(Value::String(s)) => Ok(s.clone()),
        _ => Err(ErrorCode::InvalidResponse),
    }
}

fn page(row: &Value, pro: bool) -> Result<Value, ErrorCode> {
    if !row.is_object() {
        return Err(ErrorCode::InvalidResponse);
    }
    let url = string(row, "url")?;
    if !public_url(&url) {
        return Err(ErrorCode::InvalidResponse);
    }
    let mut out = json!({"url":url,"title":string(row,"title")?,"snippet":string(row,"snippet")?});
    let available;
    if pro {
        for key in ["site_name", "date", "authority"] {
            out[key] = json!(string(row, key)?);
        }
        let empty = Vec::new();
        let chunks = match row.get("chunks") {
            None => &empty,
            Some(Value::Array(items)) => items,
            _ => return Err(ErrorCode::InvalidResponse),
        };
        let mut normalized = Vec::new();
        for chunk in chunks {
            let text = chunk
                .get("text")
                .and_then(Value::as_str)
                .ok_or(ErrorCode::InvalidResponse)?;
            let score = chunk
                .get("score")
                .and_then(Value::as_number)
                .filter(|s| s.as_f64().is_some_and(f64::is_finite))
                .ok_or(ErrorCode::InvalidResponse)?;
            normalized.push(json!({"text":text,"score":score}));
        }
        available = normalized
            .iter()
            .any(|c| !c["text"].as_str().unwrap().trim().is_empty());
        out["chunks"] = json!(normalized);
    } else {
        let text = string(row, "text")?;
        available = !text.trim().is_empty();
        out["text"] = json!(text);
    }
    out["content_status"] = json!(if available {
        "available"
    } else {
        "unavailable"
    });
    Ok(out)
}

// The returned flag marks a content edit, not an upstream absence. Only actual
// content/metadata is reducible; URL and the provenance markers are never clipped.
fn shrink_page(row: &mut Value) -> bool {
    let mut largest = (0usize, String::new(), None);
    for key in ["text", "title", "snippet", "site_name", "date", "authority"] {
        if let Some(s) = row.get(key).and_then(Value::as_str)
            && s.len() > largest.0
        {
            largest = (s.len(), key.into(), None);
        }
    }
    if let Some(chunks) = row.get("chunks").and_then(Value::as_array) {
        for (i, c) in chunks.iter().enumerate() {
            if let Some(s) = c.get("text").and_then(Value::as_str)
                && s.len() > largest.0
            {
                largest = (s.len(), "chunks".into(), Some(i));
            }
        }
    }
    if largest.0 == 0 {
        if let Some(chunks) = row.get_mut("chunks").and_then(Value::as_array_mut)
            && !chunks.is_empty()
        {
            chunks.pop();
            if row["content_status"] != "unavailable" {
                row["content_status"] = json!("truncated");
            }
            return true;
        }
        return false;
    }
    let content = largest.1 == "text" || largest.2.is_some();
    let target = if let Some(i) = largest.2 {
        &mut row["chunks"][i]["text"]
    } else {
        &mut row[&largest.1]
    };
    let mut s = target.as_str().unwrap().to_owned();
    shrink(&mut s);
    *target = json!(s);
    if content && row["content_status"] != "unavailable" {
        row["content_status"] = json!("truncated");
    }
    true
}

pub(super) fn normalize(
    value: &Value,
    limit: usize,
    pro: bool,
) -> Result<CallToolResult, ErrorCode> {
    let source = value
        .get("search_results")
        .and_then(Value::as_array)
        .ok_or(ErrorCode::InvalidResponse)?;
    let mut rows: Vec<Value> = source
        .iter()
        .filter_map(|r| page(r, pro).ok())
        .take(limit)
        .collect();
    if !source.is_empty() && rows.is_empty() {
        return Err(ErrorCode::InvalidResponse);
    }
    let mut truncated = rows.len() != source.len();
    loop {
        let output =
            json!({"results":rows,"truncated":truncated,"dropped_results":source.len()-rows.len()});
        let valid = if pro {
            serde_json::from_value::<wire::ProSuccess>(output.clone()).is_ok()
        } else {
            serde_json::from_value::<wire::ContentSuccess>(output.clone()).is_ok()
        };
        let result = CallToolResult::structured(output);
        if valid && fits(&result) {
            return Ok(result);
        }
        truncated = true;
        // Work on the largest ordinary page first. No truncation of serialized
        // JSON or source URLs; markers are recalculated in every iteration.
        let largest = rows
            .iter()
            .enumerate()
            .max_by_key(|(_, r)| serde_json::to_vec(r).map_or(0, |b| b.len()))
            .map(|(i, _)| i);
        if let Some(i) = largest {
            if shrink_page(&mut rows[i]) {
                continue;
            }
            if rows.len() > 1 {
                rows.pop();
                continue;
            }
        }
        return Err(ErrorCode::ResponseTooLarge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimi_clip_to_empty_and_metadata_clip_keep_origin_distinct() {
        let mut body = json!({"text":"文","content_status":"available"});
        assert!(shrink_page(&mut body));
        assert_eq!(body["text"], "");
        assert_eq!(body["content_status"], "truncated");
        let mut absent = json!({"title":"ordinary title","text":"","content_status":"unavailable"});
        assert!(shrink_page(&mut absent));
        assert_eq!(absent["content_status"], "unavailable");
        let mut metadata =
            json!({"title":"ordinary long title","text":"body","content_status":"available"});
        assert!(shrink_page(&mut metadata));
        assert_eq!(metadata["text"], "body");
        assert_eq!(metadata["content_status"], "available");
    }

    #[test]
    fn kimi_byte_budget_counts_escaped_copies_and_dropped_pages_once() {
        let mut pages = (0..20)
            .map(|i| {
                json!({
                    "url":format!("https://example.org/{i}/{}", "文".repeat(4000)),
                    "title":"ordinary \"quoted\" title", "text":"body"
                })
            })
            .collect::<Vec<_>>();
        pages.push(json!({"url":"", "title":null}));
        let result = normalize(&json!({"search_results":pages}), 19, false).unwrap();
        let value = result.structured_content.as_ref().unwrap();
        let kept = value["results"].as_array().unwrap();
        assert!(!kept.is_empty());
        assert_eq!(value["dropped_results"], 21 - kept.len());
        assert_eq!(value["truncated"], true);
        assert!(serde_json::to_vec(&result).unwrap().len() <= wire::MAX_RESULT_BYTES);
        let text = result.content[0].as_text().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&text.text).unwrap(), *value);
        for row in kept {
            assert!(row["url"].as_str().unwrap().ends_with(&"文".repeat(4000)));
        }
    }
}
