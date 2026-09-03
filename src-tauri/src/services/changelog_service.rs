use crate::services::git_service::CommitEntry;

/// conventional commits 分组后的类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitKind {
    Feature,
    Fix,
    Performance,
    Refactor,
    Docs,
    Other,
    Breaking,
}

/// 从 subject 前缀解析提交类别。支持 `feat(scope):`、`feat!:` 两种形态；
/// body 中含 `BREAKING CHANGE:` 时视为 breaking。
pub fn classify(entry: &CommitEntry) -> CommitKind {
    let subject = entry.subject.trim();
    let lower = subject.to_lowercase();
    let type_part = lower
        .split(|c: char| c == '(' || c == ':' || c == '!')
        .next()
        .unwrap_or("")
        .trim();

    if entry.body.contains("BREAKING CHANGE") || lower.starts_with("feat!") || lower.starts_with("fix!") {
        return CommitKind::Breaking;
    }

    match type_part {
        "feat" | "feature" => CommitKind::Feature,
        "fix" | "bugfix" => CommitKind::Fix,
        "perf" => CommitKind::Performance,
        "refactor" => CommitKind::Refactor,
        "docs" => CommitKind::Docs,
        _ => CommitKind::Other,
    }
}

fn clean_subject(subject: &str) -> String {
    // 去掉 "type(scope): " 前缀，让说明更面向读者
    let s = subject.trim();
    match s.find(':') {
        Some(idx) if idx < 40 => s[idx + 1..].trim().to_string(),
        _ => s.to_string(),
    }
}

fn kind_meta(kind: CommitKind) -> (&'static str, &'static str) {
    match kind {
        CommitKind::Feature => ("✨ Features", "feat"),
        CommitKind::Fix => ("🐛 Fixes", "fix"),
        CommitKind::Performance => ("⚡ Performance", "perf"),
        CommitKind::Refactor => ("♻ Refactor", "refactor"),
        CommitKind::Docs => ("📝 Docs", "docs"),
        CommitKind::Other => ("🔧 Others", "other"),
        CommitKind::Breaking => ("⚠ BREAKING CHANGES", "breaking"),
    }
}

const ORDER: [CommitKind; 7] = [
    CommitKind::Breaking,
    CommitKind::Feature,
    CommitKind::Fix,
    CommitKind::Performance,
    CommitKind::Refactor,
    CommitKind::Docs,
    CommitKind::Other,
];

/// 将提交列表分组生成 Markdown Changelog。
/// 每条形如 `- 说明 (short_hash)`，breaking 条目额外加 `**BREAKING:**` 提示。
pub fn generate_changelog(entries: &[CommitEntry], version: &str) -> String {
    let mut sections: Vec<(CommitKind, Vec<&CommitEntry>)> = Vec::new();
    for kind in ORDER {
        let items: Vec<&CommitEntry> = entries.iter().filter(|e| classify(e) == kind).collect();
        if !items.is_empty() {
            sections.push((kind, items));
        }
    }
    if sections.is_empty() {
        return format!("## {version}\n\n此版本没有新的提交。");
    }

    let mut out = String::new();
    out.push_str(&format!("## {version}\n\n"));
    for (kind, items) in sections {
        let (title, _) = kind_meta(kind);
        out.push_str(&format!("### {title}\n\n"));
        for e in items {
            let short: String = e.hash.chars().take(7).collect();
            let subject = clean_subject(&e.subject);
            if kind == CommitKind::Breaking && !e.subject.to_lowercase().starts_with("breaking") {
                out.push_str(&format!("- **BREAKING:** {subject} ({short})\n"));
            } else {
                out.push_str(&format!("- {subject} ({short})\n"));
            }
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(subject: &str, body: &str) -> CommitEntry {
        CommitEntry {
            hash: "abcdef1234567890".to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        }
    }

    #[test]
    fn classifies_conventional_prefixes() {
        assert_eq!(classify(&entry("feat: add x", "")), CommitKind::Feature);
        assert_eq!(classify(&entry("feat(ui): add y", "")), CommitKind::Feature);
        assert_eq!(classify(&entry("fix: crash on load", "")), CommitKind::Fix);
        assert_eq!(classify(&entry("perf: speed up scan", "")), CommitKind::Performance);
        assert_eq!(classify(&entry("refactor: split module", "")), CommitKind::Refactor);
        assert_eq!(classify(&entry("docs: update readme", "")), CommitKind::Docs);
        assert_eq!(classify(&entry("chore: bump deps", "")), CommitKind::Other);
        assert_eq!(classify(&entry("update stuff", "")), CommitKind::Other);
    }

    #[test]
    fn detects_breaking_change() {
        assert_eq!(classify(&entry("feat!: new api", "")), CommitKind::Breaking);
        assert_eq!(
            classify(&entry("feat: new api", "BREAKING CHANGE: config renamed")),
            CommitKind::Breaking
        );
    }

    #[test]
    fn groups_and_orders_sections() {
        let entries = vec![
            entry("fix: bug A", ""),
            entry("feat: feature B", ""),
            entry("random commit", ""),
            entry("feat!: removed C", ""),
            entry("docs: readme", ""),
        ];
        let md = generate_changelog(&entries, "1.1.0");
        assert!(md.contains("## 1.1.0"));
        // 顺序：Breaking -> Features -> Fixes -> Docs -> Others
        let breaking = md.find("BREAKING CHANGES").unwrap();
        let features = md.find("Features").unwrap();
        let fixes = md.find("Fixes").unwrap();
        let docs = md.find("Docs").unwrap();
        let others = md.find("Others").unwrap();
        assert!(breaking < features && features < fixes && fixes < docs && docs < others);
        // breaking 有提示前缀，且 subject 前缀已清理
        assert!(md.contains("**BREAKING:** removed C (abcdef1)"));
        assert!(md.contains("- bug A (abcdef1)"));
        assert!(!md.contains("- feat: feature B"));
    }

    #[test]
    fn empty_input_yields_placeholder() {
        let md = generate_changelog(&[], "0.1.0");
        assert!(md.contains("没有新的提交"));
    }
}
