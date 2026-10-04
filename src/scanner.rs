use std::{
    cmp::Reverse,
    fs,
    path::{Path, PathBuf},
};

pub const CACHE_THRESHOLD: u64 = 512 * 1024 * 1024;
pub const LARGE_ITEM_THRESHOLD: u64 = 1024 * 1024 * 1024;
pub const RESULT_LIMIT: usize = 1_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
}

pub fn scan_cache_items(threshold: u64, limit: usize) -> Vec<Item> {
    let mut items = Vec::new();

    for root in cache_roots() {
        collect_direct_children(&root, threshold, limit, &mut items);
        if items.len() >= limit {
            break;
        }
    }

    sort_items(&mut items);
    items
}

pub fn scan_large_items(root: &Path, threshold: u64, limit: usize) -> Vec<Item> {
    let mut items = Vec::new();
    scan_directory(root, threshold, limit, &mut items);
    sort_items(&mut items);
    items
}

fn cache_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let Some(home) = home_directory() else {
            return vec![PathBuf::from("/Library/Caches")];
        };
        let application_support = home.join("Library/Application Support");
        let mut roots = vec![
            home.join("Library/Caches"),
            home.join("Library/Logs"),
            PathBuf::from("/Library/Caches"),
        ];

        if let Ok(entries) = fs::read_dir(application_support) {
            roots.extend(entries.flatten().map(|entry| entry.path().join("Caches")));
        }

        roots
    }

    #[cfg(target_os = "linux")]
    {
        let cache_home = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home_directory().map(|home| home.join(".cache")));

        cache_home.into_iter().collect()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Vec::new()
    }
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn collect_direct_children(root: &Path, threshold: u64, limit: usize, items: &mut Vec<Item>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        if items.len() >= limit {
            return;
        }
        add_item_if_large(entry.path(), threshold, items);
    }
}

fn scan_directory(root: &Path, threshold: u64, limit: usize, items: &mut Vec<Item>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        if items.len() >= limit {
            return;
        }

        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            let size = directory_size(&path);
            if size >= threshold {
                items.push(Item {
                    path,
                    size,
                    is_directory: true,
                });
            } else {
                scan_directory(&path, threshold, limit, items);
            }
        } else if metadata.is_file() && metadata.len() >= threshold {
            items.push(Item {
                path,
                size: metadata.len(),
                is_directory: false,
            });
        }
    }
}

fn add_item_if_large(path: PathBuf, threshold: u64, items: &mut Vec<Item>) {
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }

    let size = if metadata.is_dir() {
        directory_size(&path)
    } else if metadata.is_file() {
        metadata.len()
    } else {
        return;
    };

    if size >= threshold {
        items.push(Item {
            path,
            size,
            is_directory: metadata.is_dir(),
        });
    }
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                return 0;
            };
            if metadata.file_type().is_symlink() {
                0
            } else if metadata.is_dir() {
                directory_size(&path)
            } else if metadata.is_file() {
                metadata.len()
            } else {
                0
            }
        })
        .sum()
}

fn sort_items(items: &mut [Item]) {
    items.sort_by_key(|item| Reverse(item.size));
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::scan_large_items;

    static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "cleanmymachine-test-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create temporary directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn create_file(path: &Path, size: u64) {
        File::create(path)
            .expect("create test file")
            .set_len(size)
            .expect("set test file length");
    }

    #[test]
    fn reports_large_files_largest_first() {
        let directory = TemporaryDirectory::new();
        create_file(&directory.path().join("small"), 9);
        create_file(&directory.path().join("medium"), 20);
        create_file(&directory.path().join("large"), 30);

        let items = scan_large_items(directory.path(), 10, 10);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].path, directory.path().join("large"));
        assert_eq!(items[0].size, 30);
        assert_eq!(items[1].path, directory.path().join("medium"));
    }

    #[test]
    fn reports_a_qualifying_directory_without_its_children() {
        let directory = TemporaryDirectory::new();
        let nested = directory.path().join("nested");
        fs::create_dir(&nested).expect("create nested directory");
        create_file(&nested.join("large-file"), 20);

        let items = scan_large_items(directory.path(), 10, 10);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, nested);
        assert!(items[0].is_directory);
        assert_eq!(items[0].size, 20);
    }

    #[test]
    fn descends_into_directories_below_the_threshold() {
        let directory = TemporaryDirectory::new();
        let nested = directory.path().join("nested");
        fs::create_dir(&nested).expect("create nested directory");
        create_file(&nested.join("large-file"), 20);

        let items = scan_large_items(directory.path(), 30, 10);

        assert!(items.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn ignores_symbolic_links() {
        use std::os::unix::fs::symlink;

        let directory = TemporaryDirectory::new();
        let target = directory.path().join("target");
        create_file(&target, 20);
        symlink(&target, directory.path().join("link")).expect("create symbolic link");

        let items = scan_large_items(directory.path(), 10, 10);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, target);
    }
}
