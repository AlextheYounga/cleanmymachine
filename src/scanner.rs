use std::{
    cmp::Reverse,
    collections::HashSet,
    env, fs,
    fs::Metadata,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

pub const CACHE_THRESHOLD: u64 = 512 * 1024 * 1024;
pub const HOME_ENVIRONMENT_VARIABLE: &str = "HOME";
pub const LARGE_ITEM_THRESHOLD: u64 = 1024 * 1024 * 1024;
pub const RESULT_LIMIT: usize = 1_000;

const HOME_CONTAINER_DIRECTORIES: [&str; 20] = [
    "Desktop",
    "Documents",
    "Downloads",
    "Music",
    "Pictures",
    "Public",
    "Templates",
    "Videos",
    "Work",
    ".cache",
    ".cargo",
    ".config",
    ".docker",
    ".local",
    ".npm",
    ".ollama",
    ".rustup",
    ".vscode",
    ".vscode-server",
    ".windows",
];
pub const DEVELOPMENT_CLEANUP_DIRECTORIES: [&str; 14] = [
    "target",
    "node_modules",
    ".venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".next",
    ".nuxt",
    ".turbo",
    ".parcel-cache",
    ".gradle",
    ".build",
    ".cxx",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
}

pub fn scan_cache_items_with_progress<F>(threshold: u64, limit: usize, progress: &mut F) -> Vec<Item>
where
    F: FnMut(&Path),
{
    let mut scan = Scan { threshold, limit, items: Vec::new(), progress };

    for root in cache_roots() {
        scan.collect_direct_children(&root);
        if scan.items.len() >= limit {
            break;
        }
    }

    sort_items(&mut scan.items);
    scan.items
}

pub fn scan_large_items_with_progress<F>(root: &Path, threshold: u64, limit: usize, progress: &mut F) -> Vec<Item>
where
    F: FnMut(&Path),
{
    let mut scan = Scan { threshold, limit, items: Vec::new(), progress };
    scan.scan_directory(root, true);
    sort_items(&mut scan.items);
    scan.items
}

fn cache_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let Some(home) = home_directory() else {
            return vec![PathBuf::from("/Library/Caches")];
        };
        let application_support = home.join("Library/Application Support");
        let mut roots = vec![home.join("Library/Caches"), home.join("Library/Logs"), PathBuf::from("/Library/Caches")];

        if let Ok(entries) = fs::read_dir(application_support) {
            roots.extend(entries.flatten().map(|entry| entry.path().join("Caches")));
        }

        roots
    }

    #[cfg(target_os = "linux")]
    {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home_directory().map(|home| home.join(".cache")))
            .into_iter()
            .collect()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Vec::new()
    }
}

fn home_directory() -> Option<PathBuf> {
    env::var_os(HOME_ENVIRONMENT_VARIABLE).map(PathBuf::from)
}

struct Scan<'a, F>
where
    F: FnMut(&Path),
{
    threshold: u64,
    limit: usize,
    items: Vec<Item>,
    progress: &'a mut F,
}

impl<F> Scan<'_, F>
where
    F: FnMut(&Path),
{
    fn collect_direct_children(&mut self, root: &Path) {
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };

        for entry in entries.flatten() {
            if self.items.len() >= self.limit {
                return;
            }
            self.add_item_if_large(entry.path());
        }
    }

    fn scan_directory(&mut self, root: &Path, skip_container_directories: bool) {
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };

        for entry in entries.flatten() {
            if self.items.len() >= self.limit {
                return;
            }

            let path = entry.path();
            (self.progress)(&path);
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                if skip_container_directories && is_home_container_directory(&path) {
                    self.scan_container_directory(&path);
                    continue;
                }
                let size = self.directory_size(&path);
                if size >= self.threshold {
                    self.items.push(Item { path, size, is_directory: true });
                } else {
                    self.scan_directory(&path, false);
                }
            } else if metadata.is_file() {
                let size = file_size(&metadata);
                if size >= self.threshold {
                    self.items.push(Item { path, size, is_directory: false });
                }
            }
        }
    }

    fn scan_container_directory(&mut self, root: &Path) {
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };

        for entry in entries.flatten() {
            if self.items.len() >= self.limit {
                return;
            }

            let path = entry.path();
            (self.progress)(&path);
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                if is_cleanup_directory(&path) {
                    let size = self.directory_size(&path);
                    if size >= self.threshold {
                        self.items.push(Item { path, size, is_directory: true });
                        continue;
                    }
                }
                self.scan_container_directory(&path);
            } else if metadata.is_file() {
                let size = file_size(&metadata);
                if size >= self.threshold {
                    self.items.push(Item { path, size, is_directory: false });
                }
            }
        }
    }

    fn add_item_if_large(&mut self, path: PathBuf) {
        (self.progress)(&path);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            return;
        };
        if metadata.file_type().is_symlink() {
            return;
        }

        let size = if metadata.is_dir() {
            self.directory_size(&path)
        } else if metadata.is_file() {
            file_size(&metadata)
        } else {
            return;
        };

        if size >= self.threshold {
            self.items.push(Item { path, size, is_directory: metadata.is_dir() });
        }
    }

    fn directory_size(&mut self, path: &Path) -> u64 {
        self.directory_size_once(path, &mut HashSet::new())
    }

    fn directory_size_once(&mut self, path: &Path, seen_files: &mut HashSet<(u64, u64)>) -> u64 {
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };

        entries
            .flatten()
            .map(|entry| {
                let path = entry.path();
                (self.progress)(&path);
                let Ok(metadata) = fs::symlink_metadata(&path) else {
                    return 0;
                };
                if metadata.file_type().is_symlink() {
                    0
                } else if metadata.is_dir() {
                    self.directory_size_once(&path, seen_files)
                } else if metadata.is_file() && remember_file(&metadata, seen_files) {
                    file_size(&metadata)
                } else {
                    0
                }
            })
            .sum()
    }
}

fn is_home_container_directory(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).is_some_and(|name| HOME_CONTAINER_DIRECTORIES.contains(&name))
}

fn is_cleanup_directory(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).is_some_and(|name| DEVELOPMENT_CLEANUP_DIRECTORIES.contains(&name))
}

#[cfg(unix)]
fn file_size(metadata: &Metadata) -> u64 {
    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn file_size(metadata: &Metadata) -> u64 {
    metadata.len()
}

#[cfg(unix)]
fn remember_file(metadata: &Metadata, seen_files: &mut HashSet<(u64, u64)>) -> bool {
    seen_files.insert((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn remember_file(_metadata: &Metadata, _seen_files: &mut HashSet<(u64, u64)>) -> bool {
    true
}

fn sort_items(items: &mut [Item]) {
    items.sort_by_key(|item| Reverse(item.size));
}
