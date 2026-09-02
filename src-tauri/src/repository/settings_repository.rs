use rusqlite::{params, Connection, OptionalExtension};

/// 读取一个应用级设置项（不存在时返回 None）。
pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

/// 写入（upsert）一个应用级设置项。
pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// 读取布尔设置项。
pub fn get_bool(conn: &Connection, key: &str) -> rusqlite::Result<bool> {
    Ok(get(conn, key)?.map(|v| v == "1").unwrap_or(false))
}

/// 写入布尔设置项。
pub fn set_bool(conn: &Connection, key: &str, value: bool) -> rusqlite::Result<()> {
    set(conn, key, if value { "1" } else { "0" })
}
