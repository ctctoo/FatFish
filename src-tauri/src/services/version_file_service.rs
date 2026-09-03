use std::path::{Path, PathBuf};

/// 探测项目目录中所有可写入版本号的文件。
pub fn detect_version_files(project_path: &str) -> Vec<PathBuf> {
    let dir = Path::new(project_path);
    [
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "pom.xml",
    ]
    .iter()
    .map(|f| dir.join(f))
    .filter(|p| p.is_file())
    .collect()
}

/// 把新版本号写入项目文件，返回实际被改动的文件列表。
/// 文件不存在或格式不支持时跳过；全部失败才返回 Err。
pub fn bump_version(project_path: &str, new_version: &str) -> Result<Vec<PathBuf>, String> {
    let files = detect_version_files(project_path);
    if files.is_empty() {
        return Err("项目中没有找到可同步版本号的文件（package.json / Cargo.toml / pyproject.toml / pom.xml）".to_string());
    }
    let mut changed = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for file in &files {
        match write_version(file, new_version) {
            Ok(true) => changed.push(file.clone()),
            Ok(false) => {} // 版本未变化
            Err(e) => errors.push(format!("{}: {e}", file.file_name().unwrap_or_default().to_string_lossy())),
        }
    }
    if changed.is_empty() && !errors.is_empty() {
        return Err(errors.join("；"));
    }
    Ok(changed)
}

/// 写入单个文件。返回 Ok(true) 表示有改动。
fn write_version(path: &Path, new_version: &str) -> Result<bool, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("无法读取文件: {e}"))?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let updated = match name.as_ref() {
        "package.json" => bump_package_json(&content, new_version)?,
        "Cargo.toml" => bump_toml_section(&content, new_version, "[package]")?,
        "pyproject.toml" => bump_toml_section(&content, new_version, "[project]")?,
        "pom.xml" => bump_pom_xml(&content, new_version)?,
        other => return Err(format!("不支持的文件类型: {other}")),
    };
    if updated == content {
        return Ok(false);
    }
    std::fs::write(path, updated).map_err(|e| format!("无法写入文件: {e}"))?;
    Ok(true)
}

/// package.json：JSON 解析后仅替换 version 字段，其余内容原样保留（2 空格缩进重排）。
fn bump_package_json(content: &str, new_version: &str) -> Result<String, String> {
    let mut value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("JSON 解析失败: {e}"))?;
    let obj = value
        .as_object_mut()
        .ok_or("package.json 顶层必须是对象")?;
    match obj.get("version") {
        Some(v) if v.as_str() == Some(new_version) => return Ok(content.to_string()),
        _ => {}
    }
    obj.insert(
        "version".to_string(),
        serde_json::Value::String(new_version.to_string()),
    );
    serde_json::to_string_pretty(&value)
        .map(|s| s + "\n")
        .map_err(|e| format!("JSON 序列化失败: {e}"))
}

/// TOML 文件：仅在指定 section（如 [package] / [project]）内替换 version = "..." 行。
fn bump_toml_section(content: &str, new_version: &str, section: &str) -> Result<String, String> {
    let mut in_section = false;
    let mut found = false;
    let mut out = String::with_capacity(content.len() + 16);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
        }
        if in_section && trimmed.starts_with("version") {
            let replaced = replace_kv_line(line, new_version);
            if replaced.is_some() {
                found = true;
                out.push_str(&replaced.unwrap());
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    // 去掉因 lines() 展开引入的多余末尾换行
    if content.ends_with('\n') && out.ends_with("\n\n") {
        out.truncate(out.len() - 1);
    }
    if !found {
        return Err(format!("{} 中未找到 version 字段", section));
    }
    Ok(out)
}

/// pom.xml：替换 <project> 头部附近（前 15 行内）的第一个 <version>...</version>。
fn bump_pom_xml(content: &str, new_version: &str) -> Result<String, String> {
    let mut found = false;
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    for i in 0..lines.len().min(15) {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
            let inner_start = trimmed.find('>').unwrap() + 1;
            let inner_end = trimmed.rfind('<').unwrap();
            if &trimmed[inner_start..inner_end] == new_version {
                return Ok(content.to_string());
            }
            lines[i] = format!("<version>{new_version}</version>");
            found = true;
            break;
        }
    }
    if !found {
        return Err("pom.xml 头部未找到 <version> 字段".to_string());
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// 对 `version = "x.y.z"` 形态的行做值替换，不匹配则返回 None。
fn replace_kv_line(line: &str, new_version: &str) -> Option<String> {
    let eq = line.find('=')?;
    let key = line[..eq].trim();
    if key != "version" {
        return None;
    }
    let value = line[eq + 1..].trim().trim_matches('"');
    if value == new_version {
        return Some(line.to_string());
    }
    Some(format!("{key} = \"{new_version}\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fatfish-ver-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn bumps_package_json() {
        let dir = temp_project();
        let file = dir.join("package.json");
        std::fs::write(&file, r#"{
  "name": "demo",
  "version": "1.0.0",
  "scripts": { "build": "vite build" }
}"#)
        .unwrap();

        let changed = bump_version(dir.to_str().unwrap(), "1.1.0").unwrap();
        assert_eq!(changed, vec![file.clone()]);

        let content = std::fs::read_to_string(&file).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["version"], "1.1.0");
        assert_eq!(v["name"], "demo");
        assert_eq!(v["scripts"]["build"], "vite build");

        // 相同版本号 -> 无改动
        let changed2 = bump_version(dir.to_str().unwrap(), "1.1.0").unwrap();
        assert!(changed2.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bumps_cargo_toml_only_in_package_section() {
        let dir = temp_project();
        let file = dir.join("Cargo.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n",
        )
        .unwrap();

        bump_version(dir.to_str().unwrap(), "0.2.0").unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.contains("version = \"0.2.0\""));
        // dependencies 段的 version 不能被误改
        assert!(content.contains("version = \"1\""));
        assert!(content.contains("edition = \"2021\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bumps_pyproject_toml() {
        let dir = temp_project();
        let file = dir.join("pyproject.toml");
        std::fs::write(
            &file,
            "[project]\nname = \"demo\"\nversion = \"1.2.2\"\n\n[build-system]\nrequires = [\"setuptools\"]\n",
        )
        .unwrap();

        bump_version(dir.to_str().unwrap(), "1.3.0").unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.contains("version = \"1.3.0\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bumps_pom_xml() {
        let dir = temp_project();
        let file = dir.join("pom.xml");
        std::fs::write(
            &file,
            "<?xml version=\"1.0\"?>\n<project>\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>com.demo</groupId>\n  <artifactId>demo</artifactId>\n  <version>2.0.0</version>\n  <dependencies>\n    <dependency>\n      <groupId>junit</groupId>\n      <version>4.13.2</version>\n    </dependency>\n  </dependencies>\n</project>\n",
        )
        .unwrap();

        bump_version(dir.to_str().unwrap(), "2.1.0").unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.contains("<version>2.1.0</version>"));
        // 依赖的版本不受影响
        assert!(content.contains("<version>4.13.2</version>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn errors_when_no_version_files() {
        let dir = temp_project();
        let result = bump_version(dir.to_str().unwrap(), "9.9.9");
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
