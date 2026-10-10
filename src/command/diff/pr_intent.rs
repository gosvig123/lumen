use serde_json::Value;

use super::PrInfo;

const MODAL_INNER_WIDTH: usize = 74;
const MODAL_INNER_LINES: usize = 17;
const INLINE_VALUE_WIDTH: usize = 62;
const BODY_WIDTH: usize = 72;
const BODY_LINES: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct PrIntent {
    pub title: String,
    pub body: String,
    pub url: String,
}

pub fn parse_pr_intent(json: &str) -> Result<PrIntent, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let pr = &value["data"]["repository"]["pullRequest"];
    Ok(PrIntent {
        title: required_text(pr, "title")?,
        body: optional_text(pr, "body"),
        url: required_text(pr, "url")?,
    })
}

pub fn modal_content(pr: &PrInfo) -> (String, String) {
    let mut lines = metadata_lines(pr);
    lines.extend(description_lines(&pr.intent.body));
    lines.push(String::new());
    lines.push("esc close · p reopen".to_string());
    debug_assert!(lines.len() <= MODAL_INNER_LINES);
    debug_assert!(lines
        .iter()
        .all(|line| line.chars().count() <= MODAL_INNER_WIDTH));
    (format!("PR #{} intent", pr.number), lines.join("\n"))
}

fn metadata_lines(pr: &PrInfo) -> Vec<String> {
    vec![
        format!(
            "Title: {}",
            bounded_inline(&pr.intent.title, INLINE_VALUE_WIDTH)
        ),
        format!(
            "Branches: {}",
            bounded_inline(&branch_summary(pr), INLINE_VALUE_WIDTH)
        ),
        format!(
            "URL: {}",
            bounded_inline(&pr.intent.url, INLINE_VALUE_WIDTH)
        ),
        String::new(),
        "Description (author-provided context):".to_string(),
    ]
}

fn description_lines(body: &str) -> Vec<String> {
    let (mut lines, truncated) = wrap_body(body);
    if truncated {
        lines.push("… description truncated; full text returns to Pi".to_string());
    }
    lines
}

fn branch_summary(pr: &PrInfo) -> String {
    format!(
        "{} → {}",
        sanitize_inline(&pr.base_ref),
        sanitize_inline(&pr.head_ref)
    )
}

fn bounded_inline(value: &str, width: usize) -> String {
    truncate(&sanitize_inline(value), width)
}

fn required_text(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("Missing PR {}", key))
}

fn optional_text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}

fn wrap_body(body: &str) -> (Vec<String>, bool) {
    let sanitized = sanitize_body(body);
    let source = if sanitized.trim().is_empty() {
        "(No PR description provided.)"
    } else {
        sanitized.trim()
    };
    let mut lines = source.lines().flat_map(wrap_paragraph).collect::<Vec<_>>();
    let truncated = lines.len() > BODY_LINES;
    lines.truncate(BODY_LINES);
    (lines, truncated)
}

fn wrap_paragraph(paragraph: &str) -> Vec<String> {
    if paragraph.trim().is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in paragraph.split_whitespace() {
        append_word(&mut lines, &mut current, word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn append_word(lines: &mut Vec<String>, current: &mut String, word: &str) {
    let joined = current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
    if joined <= BODY_WIDTH {
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
        return;
    }
    if !current.is_empty() {
        lines.push(std::mem::take(current));
    }
    append_chunks(lines, current, word);
}

fn append_chunks(lines: &mut Vec<String>, current: &mut String, word: &str) {
    let chars = word.chars().collect::<Vec<_>>();
    for chunk in chars.chunks(BODY_WIDTH) {
        let value = chunk.iter().collect::<String>();
        if chunk.len() == BODY_WIDTH {
            lines.push(value);
        } else {
            *current = value;
        }
    }
}

fn sanitize_inline(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn sanitize_body(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' => '\n',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect()
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    value
        .chars()
        .take(width.saturating_sub(1))
        .collect::<String>()
        + "…"
}

#[cfg(test)]
#[path = "pr_intent_tests.rs"]
mod tests;
