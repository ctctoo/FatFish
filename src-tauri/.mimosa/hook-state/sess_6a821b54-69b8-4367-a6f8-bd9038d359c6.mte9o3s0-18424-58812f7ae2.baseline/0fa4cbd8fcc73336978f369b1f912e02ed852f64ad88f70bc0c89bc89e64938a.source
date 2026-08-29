use std::path::Path;

use crate::models::scanned::ScannedProject;

const MARKERS_JAVA: [&str; 1] = ["pom.xml"];
const MARKERS_GRADLE: [&str; 4] = ["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"];
const MARKERS_JS: [&str; 1] = ["package.json"];
const MARKERS_RUST: [&str; 1] = ["Cargo.toml"];
const MARKERS_GO: [&str; 1] = ["go.mod"];
const MARKERS_PYTHON: [&str; 3] = ["requirements.txt", "pyproject.toml", "setup.py"];

const SKIP_DIRS: [&str; 8] = [
    "node_modules", "target", "build", "dist", ".gradle", ".idea", ".vscode", "venv",
];

/// 扫描根目录下的一级子目录，依据项目特征文件识别项目。
/// MVP 不做无限递归。
pub fn scan_directory(root: &str) -> Result<Vec<ScannedProject>, String> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Err(format!("目录不存在或不是文件夹: {root}"));
    }

    let entries = std::fs::read_dir(root_path).map_err(|e| format!("读取目录失败: {e}"))?;

    let mut scanned = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || SKIP_DIRS.contains(&name.to_lowercase().as_str()) {
            continue;
        }
        if let Some(language) = detect_language(&path) {
            scanned.push(ScannedProject {
                name,
                path: path.to_string_lossy().to_string(),
                language: Some(language),
                already_imported: false,
            });
        }
    }

    scanned.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(scanned)
}

/// 依据特征文件判断项目类型 / 语言
pub fn detect_language(project_path: &Path) -> Option<String> {
    let has = |file: &str| project_path.join(file).exists();

    if MARKERS_JAVA.iter().any(|f| has(f)) {
        return Some("Java".into());
    }
    if MARKERS_GRADLE.iter().any(|f| has(f)) {
        if has("src/main/AndroidManifest.xml") || has("app") && has_android_manifest(project_path) {
            return Some("Android".into());
        }
        return Some("Java".into());
    }
    if MARKERS_JS.iter().any(|f| has(f)) {
        if has("tsconfig.json") {
            return Some("TypeScript".into());
        }
        return Some("JavaScript".into());
    }
    if MARKERS_RUST.iter().any(|f| has(f)) {
        return Some("Rust".into());
    }
    if MARKERS_GO.iter().any(|f| has(f)) {
        return Some("Go".into());
    }
    if MARKERS_PYTHON.iter().any(|f| has(f)) {
        return Some("Python".into());
    }
    None
}

fn has_android_manifest(project_path: &Path) -> bool {
    project_path.join("app/src/main/AndroidManifest.xml").exists()
}

/// 仅给出项目的显示名（末级目录名）
pub fn project_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::project_name_from_path;

    #[test]
    fn extracts_last_segment() {
        assert_eq!(project_name_from_path("D:\\Projects\\AiCooking"), "AiCooking");
    }
}
