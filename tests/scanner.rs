use std::{
    env,
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicUsize, Ordering},
};

use cleanmymachine::scanner::{Item, scan_large_items_with_progress};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Result<Self, Box<dyn Error>> {
        let path = env::temp_dir().join(format!(
            "cleanmymachine-test-{}-{}",
            process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn create_file(path: &Path, size: u64) -> Result<(), Box<dyn Error>> {
    File::create(path)?.set_len(size)?;
    Ok(())
}

fn scan(root: &Path, threshold: u64) -> Vec<Item> {
    scan_large_items_with_progress(root, threshold, 10, &mut |_| {})
}

#[test]
fn reports_large_files_largest_first() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    create_file(&directory.path().join("small"), 9)?;
    create_file(&directory.path().join("medium"), 20)?;
    create_file(&directory.path().join("large"), 30)?;

    let items = scan(directory.path(), 10);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].path, directory.path().join("large"));
    assert_eq!(items[0].size, 30);
    assert_eq!(items[1].path, directory.path().join("medium"));
    Ok(())
}

#[test]
fn reports_a_qualifying_directory_without_its_children() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let nested = directory.path().join("nested");
    fs::create_dir(&nested)?;
    create_file(&nested.join("large-file"), 20)?;

    let items = scan(directory.path(), 10);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, nested);
    assert!(items[0].is_directory);
    assert_eq!(items[0].size, 20);
    Ok(())
}

#[cfg(unix)]
#[test]
fn ignores_symbolic_links() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new()?;
    let target = directory.path().join("target");
    create_file(&target, 20)?;
    symlink(&target, directory.path().join("link"))?;

    let items = scan(directory.path(), 10);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, target);
    Ok(())
}

#[test]
fn reports_paths_to_the_progress_callback() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let file = directory.path().join("file");
    create_file(&file, 20)?;
    let mut inspected = Vec::new();

    let _ = scan_large_items_with_progress(directory.path(), 10, 10, &mut |path| {
        inspected.push(path.to_path_buf());
    });

    assert!(inspected.contains(&file));
    Ok(())
}
