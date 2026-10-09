// Generated from market-broker-control source and declared borrowed authorities; DO NOT EDIT.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub type CanonicalId = String;
pub use crate::generated::SelectionRef;
pub type SelectionDigest = String;
pub type Selection = Vec<SelectionRef>;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SelectionSnapshotWire")]
pub struct SelectionSnapshot {
    pub schema_version: i64,
    pub turn_operation_id: CanonicalId,
    pub selection: Selection,
    pub selection_digest: SelectionDigest,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectionSnapshotWire {
    pub schema_version: i64,
    pub turn_operation_id: CanonicalId,
    pub selection: Selection,
    pub selection_digest: SelectionDigest,
}
impl TryFrom<SelectionSnapshotWire> for SelectionSnapshot {
    type Error = &'static str;
    fn try_from(w: SelectionSnapshotWire) -> Result<Self, Self::Error> {
        let v = Self {
            schema_version: w.schema_version,
            turn_operation_id: w.turn_operation_id,
            selection: w.selection,
            selection_digest: w.selection_digest,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SelectionSnapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_schema_version = &self.schema_version;
            if *value_schema_version != 1 {
                return Err("invalid broker constant");
            }
        }
        {
            let value_turn_operation_id = &self.turn_operation_id;
            if !canonical_id(value_turn_operation_id) {
                return Err("invalid broker UUID");
            }
            if value_turn_operation_id == "00000000-0000-0000-0000-000000000000" {
                return Err("invalid broker identity");
            }
        }
        {
            let value_selection = &self.selection;
            if value_selection.len() > 58 {
                return Err("invalid broker collection");
            }
            for item in value_selection.iter() {
                item.validate()?;
            }
            for (i, item) in value_selection.iter().enumerate() {
                if value_selection[..i].contains(item) {
                    return Err("duplicate broker collection item");
                }
            }
        }
        {
            let value_selection_digest = &self.selection_digest;
            if value_selection_digest.len() != 64
                || !value_selection_digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("invalid broker digest");
            }
        }
        if self
            .selection
            .windows(2)
            .any(|w| w[0].installation_id >= w[1].installation_id)
        {
            return Err("selection snapshot is not canonical");
        }
        if selection_digest(&self.turn_operation_id, &self.selection)? != self.selection_digest {
            return Err("selection snapshot digest mismatch");
        }
        Ok(())
    }
}
impl std::fmt::Debug for SelectionSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectionSnapshot([redacted])")
    }
}
fn canonical_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && s != "00000000-0000-0000-0000-000000000000"
        && b.iter().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                *c == b'-'
            } else {
                c.is_ascii_digit() || (*c >= b'a' && *c <= b'f')
            }
        })
}
pub fn validate_selection(refs: &[SelectionRef]) -> Result<(), &'static str> {
    if refs.len() > 58 {
        return Err("too many connector references");
    }
    for (i, r) in refs.iter().enumerate() {
        r.validate()?;
        if refs[..i]
            .iter()
            .any(|p| p.installation_id == r.installation_id)
        {
            return Err("duplicate connector installation");
        }
    }
    Ok(())
}
pub fn canonical_selection_bytes(
    turn_operation_id: &str,
    refs: &[SelectionRef],
) -> Result<Vec<u8>, &'static str> {
    if !canonical_id(turn_operation_id) {
        return Err("invalid turn operation UUID");
    }
    validate_selection(refs)?;
    let mut sorted = refs.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.installation_id.cmp(&b.installation_id));
    let mut text = format!(
        "yijie.market-selection/v1\n{}\n{}\n",
        turn_operation_id,
        sorted.len()
    );
    for r in sorted {
        text.push_str(&format!(
            "{} {} {}\n",
            r.installation_id, r.revision, r.generation
        ));
    }
    Ok(text.into_bytes())
}
pub fn selection_digest(
    turn_operation_id: &str,
    refs: &[SelectionRef],
) -> Result<String, &'static str> {
    Ok(format!(
        "{:x}",
        Sha256::digest(canonical_selection_bytes(turn_operation_id, refs)?)
    ))
}
pub fn freeze_selection(
    turn_operation_id: String,
    mut selection: Vec<SelectionRef>,
) -> Result<SelectionSnapshot, &'static str> {
    let digest = selection_digest(&turn_operation_id, &selection)?;
    selection.sort_by(|a, b| a.installation_id.cmp(&b.installation_id));
    Ok(SelectionSnapshot {
        schema_version: 1,
        turn_operation_id,
        selection,
        selection_digest: digest,
    })
}
