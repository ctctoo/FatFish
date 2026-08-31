use rusqlite::{params, Connection};

/// 保存（或覆盖）本地 GitHub 账号。token 只落 SQLite，不落 localStorage。
pub fn save(
    conn: &Connection,
    login: &str,
    token: &str,
    user_json: &str,
    logged_in_at: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO github_account (login, token, user_json, logged_in_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(login) DO UPDATE SET
            token = excluded.token,
            user_json = excluded.user_json,
            logged_in_at = excluded.logged_in_at",
        params![login, token, user_json, logged_in_at],
    )
}

/// 读取已保存的账号，返回 (login, token, user_json, logged_in_at)。
pub fn load(conn: &Connection) -> rusqlite::Result<Option<(String, String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT login, token, user_json, logged_in_at FROM github_account ORDER BY logged_in_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))),
        None => Ok(None),
    }
}

/// 删除本地账号（退出登录）。
pub fn delete(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM github_account", [])
}
