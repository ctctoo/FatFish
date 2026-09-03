use std::path::Path;
use std::process::Command;

use crate::models::git::GitInfo;

/// 汇总一个项目目录的只读 Git 信息。
/// 核心约束：Git 不可用 / 失败时返回空字段，绝不向上抛错导致项目加载失败。
pub fn collect_git_info(project_path: &str) -> GitInfo {
    let mut info = GitInfo::default();
    let dir = Path::new(project_path);
    let git_dir = dir.join(".git");

    if !git_dir.exists() {
        return info;
    }

    // 1. 解析 .git/config 中的 remote.origin.url（无 git CLI 也可用）
    info.remote_url = read_origin_url(&git_dir).map(|url| normalize_remote_url(&url));

    // 2. 优先用 git CLI 读取 branch / commit / dirty 状态
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
    {
        if output.status.success() {
            info.branch = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["log", "-1", "--pretty=format:%H%x1f%s%x1f%cI"])
        .current_dir(dir)
        .output()
    {
        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() == 3 {
                info.commit_hash = Some(parts[0].trim().to_string());
                info.commit_message = Some(parts[1].trim().to_string());
                info.commit_time = Some(parts[2].trim().to_string());
            }
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
    {
        if output.status.success() {
            info.is_dirty = Some(!output.stdout.is_empty());
        }
    }

    // 3. git CLI 不可用时，退回解析 .git 内部文件，至少给出 branch
    if info.branch.is_none() {
        info.branch = read_branch_from_head(&git_dir);
    }

    info
}

/// 解析 INI 风格的 .git/config，取 [remote "origin"] 段的 url
fn read_origin_url(git_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(git_dir.join("config")).ok()?;
    let mut in_origin_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_origin_section = line.starts_with("[remote \"origin\"]")
                || line.starts_with("[remote \"origin\" ");
            continue;
        }
        if in_origin_section {
            if let Some(rest) = line.strip_prefix("url") {
                let rest = rest.trim_start();
                if let Some(value) = rest.strip_prefix('=') {
                    let value = value.trim();
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}

/// https://github.com/example/AiCooking.git -> https://github.com/example/AiCooking
/// git@github.com:example/AiCooking.git -> https://github.com/example/AiCooking
pub fn normalize_remote_url(url: &str) -> String {
    let url = url.trim();
    let mut url = url.to_string();
    if let Some(rest) = url.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            url = format!("https://{host}/{path}");
        }
    }
    if let Some(stripped) = url.strip_suffix("ssh://") {
        url = stripped.to_string();
    }
    let url = url.trim_end_matches('/');
    url.strip_suffix(".git").unwrap_or(url).to_string()
}

/// git CLI 缺席时的兜底：从 .git/HEAD 读当前分支
fn read_branch_from_head(git_dir: &Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    head.strip_prefix("ref: refs/heads/").map(str::to_string)
}

// ---------------------------------------------------------------------------
// 发布流程专用能力：tag / log / commit / push。
// 与 collect_git_info 不同，这些操作失败要向上返回结构化错误，供向导展示。
// ---------------------------------------------------------------------------

/// 单条提交摘要
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub subject: String,
    pub body: String,
}

/// git 输出中常见的多行/编码干扰处理
fn stdout_string(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace('\r', "")
}

fn run_git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|_| "git 命令不可用，请确认已安装 Git 并加入 PATH".to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {} 失败", args.join(" "))
        } else {
            stderr
        });
    }
    Ok(stdout_string(&output))
}

/// 从 tag 名中提取语义化版本号（v1.2.3 / 1.2.3 -> 1.2.3）
fn extract_version(tag: &str) -> Option<(u64, u64, u64)> {
    let v = tag.trim().trim_start_matches(['v', 'V']);
    let mut it = v.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

/// 找出最新的语义化版本 tag（按版本号降序取第一个）。没有匹配 tag 时返回 None。
pub fn latest_version_tag(project_path: &str) -> Option<String> {
    let dir = Path::new(project_path);
    let out = run_git(dir, &["tag", "--list", "--sort=-v:refname"]).ok()?;
    out.lines()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .find(|t| extract_version(t).is_some())
        .map(str::to_string)
}

/// 读取从某 tag 到 HEAD 的提交。tag 为 None 时退化为全量提交（最多 200 条）。
pub fn log_since_tag(project_path: &str, tag: Option<&str>) -> Result<Vec<CommitEntry>, String> {
    let dir = Path::new(project_path);
    let range = match tag {
        Some(t) => format!("{t}..HEAD"),
        None => "HEAD".to_string(),
    };
    let max = if tag.is_some() { "100000" } else { "200" };
    let out = run_git(
        dir,
        &[
            "log",
            &range,
            &format!("--max-count={max}"),
            "--pretty=format:%H%x1f%s%x1f%b%x1e",
        ],
    )?;
    Ok(out
        .split('\x1e')
        .filter_map(|chunk| {
            let chunk = chunk.trim_start_matches('\n').trim();
            if chunk.is_empty() {
                return None;
            }
            let mut parts = chunk.splitn(3, '\x1f');
            let hash = parts.next()?.trim().to_string();
            let subject = parts.next().unwrap_or("").trim().to_string();
            let body = parts.next().unwrap_or("").trim().to_string();
            if hash.is_empty() {
                return None;
            }
            Some(CommitEntry { hash, subject, body })
        })
        .collect())
}

/// 提交指定文件（仅包含给定路径），返回 commit hash。
pub fn commit_files(project_path: &str, files: &[std::path::PathBuf], message: &str) -> Result<String, String> {
    if files.is_empty() {
        return Err("没有要提交的文件".to_string());
    }
    let dir = Path::new(project_path);
    let mut args: Vec<&str> = vec!["add", "--"];
    let paths: Vec<String> = files.iter().map(|f| f.to_string_lossy().to_string()).collect();
    for p in &paths {
        args.push(p);
    }
    run_git(dir, &args)?;
    let out = run_git(dir, &["commit", "-m", message])?;
    // git commit 输出首行通常带 [branch hash] 提示，取不出来也无妨
    Ok(out.lines().next().unwrap_or("").to_string())
}

/// 创建 tag 并推送到 origin。tag 已存在时返回明确错误。
pub fn tag_and_push(project_path: &str, tag: &str) -> Result<(), String> {
    let dir = Path::new(project_path);
    run_git(dir, &["tag", tag]).map_err(|e| {
        if e.contains("already exists") {
            format!("tag {tag} 已存在，请换一个版本号")
        } else {
            e
        }
    })?;
    run_git(dir, &["push", "origin", tag]).map_err(|e| {
        format!("tag 已在本地创建，但推送失败：{e}")
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{extract_version, normalize_remote_url};

    #[test]
    fn strips_git_suffix() {
        assert_eq!(
            normalize_remote_url("https://github.com/example/AiCooking.git"),
            "https://github.com/example/AiCooking"
        );
    }

    #[test]
    fn converts_ssh_to_https() {
        assert_eq!(
            normalize_remote_url("git@github.com:example/AiCooking.git"),
            "https://github.com/example/AiCooking"
        );
    }

    #[test]
    fn parses_versions_from_tags() {
        assert_eq!(extract_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(extract_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(extract_version("V0.1.0"), Some((0, 1, 0)));
        assert_eq!(extract_version("release-1"), None);
        assert_eq!(extract_version("1.2"), Some((1, 2, 0)));
    }
}
