use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

const RELEASES_API: &str = "https://api.github.com/repos/ctctoo/FatFish/releases/latest";
/// 回退数据源：Releases Atom feed。它走 github.com 而非 api.github.com，
/// 不受匿名 API「60 次/小时/IP」的限流约束——本机多开、公司/校园共享出口 IP
/// 时匿名配额会长期耗尽，导致检查更新必定失败。
const RELEASES_ATOM: &str = "https://github.com/ctctoo/FatFish/releases.atom";
const RELEASES_PAGE: &str = "https://github.com/ctctoo/FatFish/releases";
const REPO_TAG_URL: &str = "https://github.com/ctctoo/FatFish/releases/tag/";
const USER_AGENT_VALUE: &str = "FatFish";

/// 单次请求超时。没有它时网络异常会让界面一直停在「正在检查…」。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

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

/// 构造带超时的 HTTP 客户端。
fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("初始化 HTTP 客户端失败：{e}"))
}

/// 从 Atom feed 文本里解析出版本号最高的 release。
///
/// Atom 按更新时间倒序，但补发旧版本补丁时顺序会与版本号不一致，
/// 所以取全部 tag 中语义化版本最高者，而不是简单取第一条。
/// 返回 (版本号, 原始 tag)。
fn latest_from_atom(body: &str) -> Option<(String, String)> {
    let mut best: Option<(String, String)> = None;
    for chunk in body.split(REPO_TAG_URL).skip(1) {
        let tag: String = chunk
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '<')
            .collect();
        if tag.is_empty() {
            continue;
        }
        let version = tag.trim_start_matches(['v', 'V']).to_string();
        if parse_version(&version).is_empty() {
            continue;
        }
        let is_better = match &best {
            None => true,
            Some((best_version, _)) => is_newer(&version, best_version),
        };
        if is_better {
            best = Some((version, tag));
        }
    }
    best
}

/// 取 Atom feed 里第一条 <updated> 作为发布时间。
fn atom_published_at(body: &str) -> Option<String> {
    body.split_once("<updated>")
        .and_then(|(_, rest)| rest.split_once("</updated>"))
        .map(|(value, _)| value.trim().to_string())
}

/// 主路径：GitHub Releases API（信息最全，含更新说明）。
/// 匿名限流、网络异常等情况返回 Err，交由调用方回退到 Atom。
fn fetch_latest_api(client: &Client) -> Result<Option<(String, String, Option<String>, Option<String>)>, String> {
    let resp = client
        .get(RELEASES_API)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let release: RawRelease = resp
        .json()
        .map_err(|e| format!("解析发布信息失败：{e}"))?;

    let tag = release.tag_name.unwrap_or_default().trim().to_string();
    if tag.is_empty() {
        return Ok(None);
    }
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    Ok(Some((
        version,
        release.html_url.unwrap_or_else(|| RELEASES_PAGE.to_string()),
        release.body,
        release.published_at,
    )))
}

/// 回退路径：Releases Atom feed，规避匿名 API 限流。
/// Atom 不含更新说明，故 notes 为 None。
fn fetch_latest_atom(client: &Client) -> Result<Option<(String, String, Option<String>, Option<String>)>, String> {
    let resp = client
        .get(RELEASES_ATOM)
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body = resp
        .text()
        .map_err(|e| format!("读取发布信息失败：{e}"))?;

    Ok(latest_from_atom(&body).map(|(version, tag)| {
        (
            version,
            format!("{REPO_TAG_URL}{tag}"),
            None,
            atom_published_at(&body),
        )
    }))
}

/// 比较版本，仅在确实更新时返回 UpdateInfo。
fn build_update_info(
    current_version: &str,
    latest_version: String,
    release_url: String,
    release_notes: Option<String>,
    published_at: Option<String>,
) -> Result<Option<UpdateInfo>, String> {
    if latest_version.is_empty() || !is_newer(&latest_version, current_version) {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version,
        release_url,
        release_notes,
        published_at,
    }))
}

/// 查询 GitHub 上的最新 Release，与当前版本比较。
/// 无新版本时返回 Ok(None)；两条数据源都失败才返回 Err。
pub fn check_update(current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let client = build_client()?;

    // 优先走 API（能带出更新说明）；限流或失败则回退 Atom feed。
    match fetch_latest_api(&client) {
        Ok(Some((version, url, notes, published))) => {
            return build_update_info(current_version, version, url, notes, published);
        }
        Ok(None) => return Ok(None),
        // API 不可用（匿名限流 / 网络异常）时回退 Atom；两条路都不通才报错，
        // 并把两个原因都带出来便于排查。
        Err(api_err) => match fetch_latest_atom(&client) {
            Ok(Some((version, url, notes, published))) => {
                return build_update_info(current_version, version, url, notes, published);
            }
            Ok(None) => return Ok(None),
            Err(atom_err) => {
                return Err(format!("检查更新失败（API：{api_err}；Atom：{atom_err}）"));
            }
        },
    }
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
    use super::{atom_published_at, is_newer, latest_from_atom, parse_version, REPO_TAG_URL};

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

    /// 构造一段最小 Atom，tag 之间顺序可任意打乱。
    fn atom_with(tags: &[&str], updated: &str) -> String {
        let entries: Vec<String> = tags
            .iter()
            .map(|t| {
                format!(
                    "<entry><id>tag:github.com,2008:Repository/1/{t}</id>\
                     <updated>{updated}</updated>\
                     <link rel=\"alternate\" type=\"text/html\" href=\"{REPO_TAG_URL}{t}\"/>\
                     <title>{t}</title></entry>"
                )
            })
            .collect();
        format!("<feed><updated>{updated}</updated>{}</feed>", entries.concat())
    }

    #[test]
    fn picks_highest_version_from_atom() {
        // Atom 按更新时间倒序：补发旧版补丁后，最高版本不在第一条
        let body = atom_with(&["v0.2.2", "v0.3.0", "v0.2.1"], "2026-09-02T01:22:33Z");
        let (version, tag) = latest_from_atom(&body).expect("应解析出版本");
        assert_eq!(version, "0.3.0");
        assert_eq!(tag, "v0.3.0");
    }

    #[test]
    fn parses_single_atom_entry() {
        let body = atom_with(&["v0.3.0"], "2026-09-02T01:22:33Z");
        let (version, _) = latest_from_atom(&body).expect("应解析出版本");
        assert_eq!(version, "0.3.0");
        assert_eq!(
            atom_published_at(&body).as_deref(),
            Some("2026-09-02T01:22:33Z")
        );
    }

    #[test]
    fn ignores_non_version_tags() {
        // 无关链接或无法解析的数字段不应被当作版本
        let body = atom_with(&["nightly", "v0.3.0"], "2026-09-02T01:22:33Z");
        let (version, _) = latest_from_atom(&body).expect("应解析出版本");
        assert_eq!(version, "0.3.0");
    }

    #[test]
    fn returns_none_when_no_tag_present() {
        assert!(latest_from_atom("<feed><entry><title>x</title></entry></feed>").is_none());
    }

    /// 需要联网，默认跳过：`cargo test -- --ignored`
    #[test]
    #[ignore]
    fn e2e_real_network() {
        use super::check_update;
        // 当前版本落后时应解析出 0.3.0（API 限流时自动走 Atom 回退）
        match check_update("0.2.0") {
            Ok(Some(info)) => {
                println!(
                    "latest={} url={} published={:?}",
                    info.latest_version, info.release_url, info.published_at
                );
                assert_eq!(info.latest_version, "0.3.0");
            }
            Ok(None) => panic!("应检出 0.3.0，实际返回无更新"),
            Err(e) => panic!("检查失败：{e}"),
        }
        // 已是最新版时应返回 None
        assert!(
            check_update("0.3.0").expect("不应报错").is_none(),
            "0.3.0 应报告为已是最新"
        );
    }
}
