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

#[cfg(test)]
mod tests {
    use super::normalize_remote_url;

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
}
