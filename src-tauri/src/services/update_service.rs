use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

const RELEASES_API: &str = "https://api.github.com/repos/ctctoo/FatFish/releases/latest";
const USER_AGENT_VALUE: &str = "FatFish";

/// 可用更新信息（latest 比 current 新时才返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawRelease {
    #[serde(default)]
    tag_name: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
}

/// 查询 GitHub 最新 Release，与当前版本比较。
/// 无新版本时返回 Ok(None)；网络失败返回 Err（调用方决定是否提示）。
pub fn check_update(current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let resp = Client::new()
        .get(RELEASES_API)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("检查更新失败（HTTP {}）", resp.status()));
    }
    let release: RawRelease = resp
        .json()
        .map_err(|e| format!("解析发布信息失败：{e}"))?;

    let latest = release
        .tag_name
        .unwrap_or_default()
        .trim()
        .trim_start_matches(['v', 'V'])
        .to_string();
    if latest.is_empty() {
        return Ok(None);
    }
    if !is_newer(&latest, current_version) {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: latest,
        release_url: release
            .html_url
            .unwrap_or_else(|| "https://github.com/ctctoo/FatFish/releases".into()),
        release_notes: release.body,
        published_at: release.published_at,
    }))
}

/// 拆出版本号里的数字段：v0.3.0-beta.1 -> [0, 3, 0, 1]
fn parse_version(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-'])
        .filter_map(|p| p.parse::<u64>().ok())
        .collect()
}

/// 语义化比较：candidate 是否比 current 更新
fn is_newer(candidate: &str, current: &str) -> bool {
    let a = parse_version(candidate);
    let b = parse_version(current);
    let len = a.len().max(b.len());
    for i in 0..len {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        if av != bv {
            return av > bv;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{is_newer, parse_version};

    #[test]
    fn parses_numeric_segments() {
        assert_eq!(parse_version("v0.3.0"), vec![0, 3, 0]);
        assert_eq!(parse_version("0.3.0-beta.1"), vec![0, 3, 0, 1]);
    }

    #[test]
    fn compares_semver() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(is_newer("0.2.1", "0.2.0"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.9", "0.2.0"));
    }
}
