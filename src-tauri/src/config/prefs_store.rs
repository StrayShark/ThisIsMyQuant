use std::fs;
use std::path::{Path, PathBuf};

use crate::config::UserPreferences;
use crate::db::Database;
use crate::error::{AppError, AppResult};

const FILE_NAME: &str = "user_preferences.json";

pub fn preferences_path(database_path: &Path) -> PathBuf {
    database_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| super::project_root().join("data"))
        .join(FILE_NAME)
}

/// 从 JSON 文件加载；若不存在则尝试自 SQLite 迁移，否则返回默认值。
pub fn load_user_preferences(db: &Database, database_path: &Path) -> AppResult<UserPreferences> {
    let path = preferences_path(database_path);
    if path.exists() {
        let text = fs::read_to_string(&path)
            .map_err(|e| AppError::Msg(format!("read {}: {e}", path.display())))?;
        let prefs: UserPreferences = serde_json::from_str(&text)?;
        return Ok(prefs.normalize());
    }

    if let Ok(Some(legacy)) = db.load_user_preferences_legacy() {
        let prefs = legacy.normalize();
        save_user_preferences(&path, &prefs)?;
        log::info!("migrated preferences SQLite → {}", path.display());
        return Ok(prefs);
    }

    Ok(UserPreferences::default().normalize())
}

pub fn save_user_preferences(path: &Path, prefs: &UserPreferences) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::Msg(format!("create {}: {e}", parent.display())))?;
    }
    let prefs = prefs.clone().normalize();
    let text = serde_json::to_string_pretty(&prefs)?;
    fs::write(path, text).map_err(|e| AppError::Msg(format!("write {}: {e}", path.display())))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_path_sits_next_to_database() {
        let db = PathBuf::from("/tmp/quant/data/quant.db");
        assert_eq!(
            preferences_path(&db),
            PathBuf::from("/tmp/quant/data/user_preferences.json")
        );
    }
}
