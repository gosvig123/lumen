use super::*;

#[test]
fn parses_escaped_pr_text() {
    let json = r#"{"data":{"repository":{"pullRequest":{"title":"Fix auth","body":"Line 1\nLine 2","url":"https://github.com/o/r/pull/7"}}}}"#;
    let intent = parse_pr_intent(json).unwrap();
    assert_eq!(intent.title, "Fix auth");
    assert_eq!(intent.body, "Line 1\nLine 2");
}

#[test]
fn sanitizes_terminal_controls() {
    let pr = sample_pr("safe\u{1b}[31m body".into());
    let (_, message) = modal_content(&pr);
    assert!(!message.contains('\u{1b}'));
}

#[test]
fn bounds_long_description() {
    let body = (0..1_000).map(|_| "word").collect::<Vec<_>>().join(" ");
    let pr = sample_pr(body);
    let (_, message) = modal_content(&pr);
    let lines = message.lines().collect::<Vec<_>>();
    assert!(lines
        .iter()
        .any(|line| line.contains("description truncated")));
    assert!(lines.len() <= MODAL_INNER_LINES);
    assert!(lines
        .iter()
        .all(|line| line.chars().count() <= MODAL_INNER_WIDTH));
}

fn sample_pr(body: String) -> PrInfo {
    PrInfo {
        number: 7,
        node_id: "id".into(),
        repo_owner: "o".into(),
        repo_name: "r".into(),
        base_ref: "main".into(),
        head_ref: "feature".into(),
        base_repo_owner: "o".into(),
        head_repo_owner: Some("o".into()),
        intent: PrIntent {
            title: "Fix auth".into(),
            body,
            url: "https://github.com/o/r/pull/7".into(),
        },
    }
}
