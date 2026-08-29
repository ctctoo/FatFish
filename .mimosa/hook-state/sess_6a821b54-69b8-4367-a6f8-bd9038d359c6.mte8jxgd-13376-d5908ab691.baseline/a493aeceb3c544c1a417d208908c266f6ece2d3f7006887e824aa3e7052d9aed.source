/// 打开项目文件夹（系统文件管理器）
#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("目录不存在: {path}"));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
    }
    Ok(())
}

/// 在项目目录打开系统终端
#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("目录不存在: {path}"));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd"])
            .current_dir(dir)
            .spawn()
            .map_err(|e| format!("打开终端失败: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "Terminal", &path])
            .spawn()
            .map_err(|e| format!("打开终端失败: {e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("x-terminal-emulator")
            .current_dir(dir)
            .spawn()
            .or_else(|_| {
                std::process::Command::new("xterm")
                    .current_dir(dir)
                    .spawn()
            })
            .map_err(|e| format!("打开终端失败: {e}"))?;
    }
    Ok(())
}
