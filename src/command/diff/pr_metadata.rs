use serde_json::Value;

use super::pr_intent::{self, PrIntent};

pub struct PrMetadata {
    pub node_id: String,
    pub base_ref: String,
    pub head_ref: String,
    pub base_repo_owner: Option<String>,
    pub head_repo_owner: Option<String>,
    pub intent: PrIntent,
}

pub fn parse_pr_metadata(json: &str) -> Result<PrMetadata, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let pr = &value["data"]["repository"]["pullRequest"];
    Ok(PrMetadata {
        node_id: required_text(pr, "id")?,
        base_ref: required_text(pr, "baseRefName")?,
        head_ref: required_text(pr, "headRefName")?,
        base_repo_owner: nested_owner(pr, "baseRepository"),
        head_repo_owner: nested_owner(pr, "headRepository"),
        intent: pr_intent::parse_pr_intent(json)?,
    })
}

fn required_text(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("Missing PR {}", key))
}

fn nested_owner(value: &Value, repository_key: &str) -> Option<String> {
    value[repository_key]["owner"]["login"]
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_escaped_refs_and_deleted_fork() {
        let json = r#"{"data":{"repository":{"pullRequest":{"id":"node","title":"Fix","body":"Intent","url":"https://github.com/o/r/pull/7","baseRefName":"release\"quoted","headRefName":"feature\\path","baseRepository":{"owner":{"login":"base-owner"}},"headRepository":null}}}}"#;
        let metadata = parse_pr_metadata(json).unwrap();
        assert_eq!(metadata.node_id, "node");
        assert_eq!(metadata.base_ref, "release\"quoted");
        assert_eq!(metadata.head_ref, "feature\\path");
        assert_eq!(metadata.base_repo_owner.as_deref(), Some("base-owner"));
        assert_eq!(metadata.head_repo_owner, None);
    }
}
