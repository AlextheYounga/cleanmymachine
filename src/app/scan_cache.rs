use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "macos"))]
use crate::scanner::DEFAULT_CACHE_DIRECTORY;
#[cfg(target_os = "macos")]
use crate::scanner::MACOS_USER_CACHE_DIRECTORY;
#[cfg(target_os = "linux")]
use crate::scanner::XDG_CACHE_HOME_ENVIRONMENT_VARIABLE;
use crate::scanner::{HOME_ENVIRONMENT_VARIABLE, Item};

const CACHE_DIRECTORY: &str = "cleanmymachine";
const CACHE_FILE: &str = "scan-results.json";
const CACHE_FORMAT_VERSION: u8 = 2;
const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Deserialize, Serialize)]
pub(super) struct ScanResults {
    version: u8,
    pub(super) root: PathBuf,
    scanned_at: u64,
    pub(super) cache_items: Vec<Item>,
    pub(super) large_files: Vec<Item>,
    pub(super) large_directories: Vec<Item>,
}

pub(super) fn load(root: &Path) -> Option<ScanResults> {
    let contents = fs::read_to_string(cache_path()?).ok()?;
    let entries: Vec<ScanResults> = serde_json::from_str(&contents).ok()?;
    let now = current_time()?;
    entries
        .into_iter()
        .find(|entry| entry.version == CACHE_FORMAT_VERSION && entry.root == root && is_fresh(entry.scanned_at, now))
}

pub(super) fn store(root: &Path, cache_items: Vec<Item>, large_files: Vec<Item>, large_directories: Vec<Item>) {
    let Some(path) = cache_path() else {
        return;
    };
    let Some(now) = current_time() else {
        return;
    };

    let mut entries = fs::read_to_string(&path)
        .ok()
        .and_then(|contents| serde_json::from_str::<Vec<ScanResults>>(&contents).ok())
        .unwrap_or_default();
    entries.retain(|entry| entry.root != root && is_fresh(entry.scanned_at, now));
    entries.push(ScanResults {
        version: CACHE_FORMAT_VERSION,
        root: root.to_path_buf(),
        scanned_at: now,
        cache_items,
        large_files,
        large_directories,
    });

    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(contents) = serde_json::to_vec(&entries) else {
        return;
    };

    let pending_path = path.with_extension("tmp");
    if fs::write(&pending_path, contents).is_ok() {
        let _ = fs::rename(pending_path, path);
    }
}

pub(super) fn clear() {
    if let Some(path) = cache_path() {
        let _ = fs::remove_file(path);
    }
}

fn cache_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let parent = home_directory()?.join(MACOS_USER_CACHE_DIRECTORY);

    #[cfg(target_os = "linux")]
    let parent = env::var_os(XDG_CACHE_HOME_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or(home_directory()?.join(DEFAULT_CACHE_DIRECTORY));

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let parent = home_directory()?.join(DEFAULT_CACHE_DIRECTORY);

    Some(parent.join(CACHE_DIRECTORY).join(CACHE_FILE))
}

fn home_directory() -> Option<PathBuf> {
    env::var_os(HOME_ENVIRONMENT_VARIABLE).map(PathBuf::from)
}

fn current_time() -> Option<u64> {
    SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|duration| duration.as_secs())
}

fn is_fresh(scanned_at: u64, now: u64) -> bool {
    now.saturating_sub(scanned_at) < CACHE_TTL.as_secs()
}

#[cfg(test)]
mod tests {
    use super::is_fresh;

    #[test]
    fn expires_scan_results_after_one_hour() {
        assert!(is_fresh(1_000, 4_599));
        assert!(!is_fresh(1_000, 4_600));
    }
}
