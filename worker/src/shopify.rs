//! Shopify Global Catalog is keyless. UCP metadata is part of the reviewed
//! arguments, never injected after approval or supplied by a hidden request.
use crate::tushare_oauth::Failure;
use rmcp::model::Tool;
use serde_json::{Value, json};
use std::sync::Arc;

pub const SERVICE: &str = "shopify";
pub const PROFILE: &str =
    "https://shopify.dev/ucp/agent-profiles/2026-08-25/valid-with-capabilities.json";
const CATALOG_TOOLS: [&str; 3] = ["search_catalog", "lookup_catalog", "get_product"];

pub fn catalog_tools(tools: Vec<Tool>) -> Result<Vec<Tool>, Failure> {
    let mut selected = Vec::new();
    for mut tool in tools {
        if !CATALOG_TOOLS.contains(&tool.name.as_ref()) {
            continue;
        }
        let mut schema = (*tool.input_schema).clone();
        let properties = schema
            .entry("properties")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or(Failure::InvalidMetadata)?;
        let upstream_meta = properties.remove("meta").unwrap_or_else(|| json!({}));
        // Intersect with the provider's full meta schema. Pin only the public
        // profile identity; do not erase provider constraints or add authority.
        properties.insert(
            "meta".into(),
            json!({"allOf": [upstream_meta, {
                "type": "object", "required": ["ucp-agent"],
                "properties": {"ucp-agent": {"type": "object", "required": ["profile"],
                    "properties": {"profile": {"type": "string", "const": PROFILE}}}}
            }]}),
        );
        let required = schema
            .entry("required")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or(Failure::InvalidMetadata)?;
        if !required.contains(&Value::String("meta".into())) {
            required.push(Value::String("meta".into()));
        }
        tool.input_schema = Arc::new(schema);
        tool.description = Some(format!(
            "{}\nShopify Global Catalog: include meta.ucp-agent.profile = {PROFILE}. Only public catalog discovery is supported; this connector does not expose carts, checkout or orders.",
            tool.description.as_deref().unwrap_or_default()
        ).into());
        selected.push(tool);
    }
    if selected.len() != CATALOG_TOOLS.len()
        || CATALOG_TOOLS
            .iter()
            .any(|name| selected.iter().filter(|t| t.name.as_ref() == *name).count() != 1)
    {
        return Err(Failure::InvalidMetadata);
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_requires_full_upstream_schema_and_visible_fixed_profile() {
        let source = CATALOG_TOOLS
            .iter()
            .chain(["create_checkout"].iter())
            .map(|name| {
                Tool::new(*name, "ordinary local catalog schema", json!({
                "type": "object", "required": ["catalog"], "additionalProperties": false,
                "properties": {"catalog": {"type": "object", "required": ["query"],
                    "properties": {"query": {"type": "string"}}}, "meta": {"type": "object"}}
            }).as_object().unwrap().clone())
            })
            .collect();
        let definitions =
            crate::generic::definitions(SERVICE, catalog_tools(source).unwrap()).unwrap();
        assert_eq!(definitions.len(), 3);
        let args = json!({"catalog": {"query": "cotton sweater"}, "meta": {"ucp-agent": {"profile": PROFILE}}});
        let review = definitions[0].review(args.as_object().unwrap()).unwrap();
        assert!(review.arguments_json.unwrap().contains(PROFILE));
        assert!(
            definitions[0]
                .review(
                    json!({"catalog": {"query": "cotton sweater"}})
                        .as_object()
                        .unwrap()
                )
                .is_err()
        );
        assert!(
            definitions[0]
                .review(
                    json!({"meta": {"ucp-agent": {"profile": PROFILE}}})
                        .as_object()
                        .unwrap()
                )
                .is_err()
        );
    }
}
