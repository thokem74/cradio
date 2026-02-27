use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteEntry {
    pub stationuuid: String,
    pub name: String,
    pub url: String,
}

fn favorites_path() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .map_err(|_| "HOME is not set; favorites persistence is unavailable".to_string())?;
    Ok(Path::new(&home).join(".cradio").join("favorites.json"))
}

fn load_favorites_from_path(path: &Path) -> Result<Vec<FavoriteEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read favorites file {}: {}", path.display(), e))?;

    let entries: Vec<FavoriteEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse favorites JSON {}: {}", path.display(), e))?;

    let mut deduped: Vec<FavoriteEntry> = Vec::new();
    for entry in entries {
        if entry.stationuuid.trim().is_empty() {
            continue;
        }
        if let Some(existing) = deduped
            .iter_mut()
            .find(|fav| fav.stationuuid == entry.stationuuid)
        {
            existing.name = entry.name;
            existing.url = entry.url;
        } else {
            deduped.push(entry);
        }
    }

    deduped.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.stationuuid.cmp(&b.stationuuid))
    });

    Ok(deduped)
}

fn save_favorites_to_path(path: &Path, favorites: &[FavoriteEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create favorites directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let mut deduped: Vec<FavoriteEntry> = Vec::new();
    for entry in favorites {
        if entry.stationuuid.trim().is_empty() {
            continue;
        }
        if let Some(existing) = deduped
            .iter_mut()
            .find(|fav| fav.stationuuid == entry.stationuuid)
        {
            existing.name = entry.name.clone();
            existing.url = entry.url.clone();
        } else {
            deduped.push(entry.clone());
        }
    }

    deduped.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.stationuuid.cmp(&b.stationuuid))
    });

    let json = serde_json::to_string_pretty(&deduped)
        .map_err(|e| format!("Failed to serialize favorites: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Failed to write favorites file {}: {}", path.display(), e))
}

pub fn load_favorites() -> Result<Vec<FavoriteEntry>, String> {
    let path = favorites_path()?;
    load_favorites_from_path(&path)
}

pub fn save_favorites(favorites: &[FavoriteEntry]) -> Result<(), String> {
    let path = favorites_path()?;
    save_favorites_to_path(&path, favorites)
}

#[cfg(test)]
mod tests {
    use super::{FavoriteEntry, load_favorites_from_path, save_favorites_to_path};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("cradio-favorites-test-{}-{}", name, stamp))
            .join("favorites.json")
    }

    fn fav(id: &str, name: &str, url: &str) -> FavoriteEntry {
        FavoriteEntry {
            stationuuid: id.to_string(),
            name: name.to_string(),
            url: url.to_string(),
        }
    }

    #[test]
    fn load_missing_file_returns_empty_vec() {
        let path = temp_path("missing");
        let favorites = load_favorites_from_path(&path).expect("load should succeed");
        assert!(favorites.is_empty());
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let path = temp_path("invalid");
        fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
        fs::write(&path, "{not-json]").expect("write invalid json");

        let err = load_favorites_from_path(&path).expect_err("expected parse error");
        assert!(err.contains("Failed to parse favorites JSON"));

        let _ = fs::remove_dir_all(path.parent().expect("parent").parent().expect("root"));
    }

    #[test]
    fn save_and_load_roundtrip_object_entries() {
        let path = temp_path("roundtrip");
        let favorites = vec![
            fav("uuid-b", "Beta", "https://b"),
            fav("uuid-a", "Alpha", "https://a"),
        ];

        save_favorites_to_path(&path, &favorites).expect("save should work");
        let loaded = load_favorites_from_path(&path).expect("load should work");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].stationuuid, "uuid-a");
        assert_eq!(loaded[1].stationuuid, "uuid-b");

        let _ = fs::remove_dir_all(path.parent().expect("parent").parent().expect("root"));
    }

    #[test]
    fn save_and_load_dedups_same_uuid_with_latest_data() {
        let path = temp_path("dedup");
        let favorites = vec![
            fav("uuid-a", "Old Name", "https://old"),
            fav("uuid-a", "New Name", "https://new"),
            fav("uuid-b", "Beta", "https://b"),
        ];

        save_favorites_to_path(&path, &favorites).expect("save should work");
        let loaded = load_favorites_from_path(&path).expect("load should work");

        assert_eq!(loaded.len(), 2);
        let updated = loaded
            .iter()
            .find(|entry| entry.stationuuid == "uuid-a")
            .expect("uuid-a must exist");
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.url, "https://new");

        let _ = fs::remove_dir_all(path.parent().expect("parent").parent().expect("root"));
    }
}
