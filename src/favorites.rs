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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Linux,
    Windows,
}

fn favorites_path() -> Result<PathBuf, String> {
    let env_vars: Vec<_> = env::vars_os().collect();
    let platform = if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Linux
    };

    favorites_path_for(platform, &env_vars)
}

fn favorites_path_for(
    platform: Platform,
    env_vars: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<PathBuf, String> {
    match platform {
        Platform::Linux => linux_favorites_dir(env_vars)
            .map(|dir| dir.join("favorites.json"))
            .ok_or_else(|| {
                "Favorites persistence is unavailable: set XDG_CONFIG_HOME or HOME.".to_string()
            }),
        Platform::Windows => windows_favorites_dir(env_vars)
            .map(|dir| windows_path(dir.as_os_str(), &["favorites.json"]))
            .ok_or_else(|| {
                "Favorites persistence is unavailable: set APPDATA, LOCALAPPDATA, or USERPROFILE."
                    .to_string()
            }),
    }
}

fn linux_favorites_dir(env_vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Option<PathBuf> {
    env_lookup(env_vars, "XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .map(|dir| dir.join("cradio"))
        .or_else(|| {
            env_lookup(env_vars, "HOME")
                .map(PathBuf::from)
                .map(|dir| dir.join(".cradio"))
        })
}

fn windows_favorites_dir(env_vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Option<PathBuf> {
    env_lookup(env_vars, "APPDATA")
        .map(|dir| windows_path(dir, &["cradio"]))
        .or_else(|| env_lookup(env_vars, "LOCALAPPDATA").map(|dir| windows_path(dir, &["cradio"])))
        .or_else(|| {
            env_lookup(env_vars, "USERPROFILE")
                .map(|dir| windows_path(dir, &["AppData", "Roaming", "cradio"]))
        })
}

fn env_lookup<'a>(
    env_vars: &'a [(std::ffi::OsString, std::ffi::OsString)],
    key: &str,
) -> Option<&'a std::ffi::OsStr> {
    env_vars
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_os_str())
}

fn windows_path(base: &std::ffi::OsStr, segments: &[&str]) -> PathBuf {
    let mut path = base
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_string();
    for segment in segments {
        if !path.is_empty() {
            path.push('\\');
        }
        path.push_str(segment);
    }
    PathBuf::from(path)
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
    use super::{
        FavoriteEntry, Platform, favorites_path_for, load_favorites_from_path,
        save_favorites_to_path,
    };
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
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
    fn linux_path_prefers_xdg_config_home() {
        let path = favorites_path_for(
            Platform::Linux,
            &[
                (
                    OsString::from("XDG_CONFIG_HOME"),
                    OsString::from("/tmp/cradio-config"),
                ),
                (OsString::from("HOME"), OsString::from("/tmp/home")),
            ],
        )
        .expect("path should resolve");

        assert_eq!(path, Path::new("/tmp/cradio-config/cradio/favorites.json"));
    }

    #[test]
    fn windows_path_uses_appdata() {
        let path = favorites_path_for(
            Platform::Windows,
            &[(
                OsString::from("APPDATA"),
                OsString::from(r"C:\Users\Test\AppData\Roaming"),
            )],
        )
        .expect("path should resolve");

        assert_eq!(
            path,
            Path::new(r"C:\Users\Test\AppData\Roaming\cradio\favorites.json")
        );
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
