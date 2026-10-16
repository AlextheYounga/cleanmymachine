use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicUsize, Ordering},
};

use cleanmymachine::scanner::{Item, scan_directory_children_with_progress, scan_large_items_with_progress};

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
    let mut file = File::create(path)?;
    file.write_all(&vec![0; usize::try_from(size)?])?;
    Ok(())
}

fn scan(root: &Path, threshold: u64) -> Vec<Item> {
    scan_large_items_with_progress(root, threshold, 10, &mut |_| {})
}

#[test]
fn reports_large_files_largest_first() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    create_file(&directory.path().join("small"), 1024 * 1024)?;
    create_file(&directory.path().join("medium"), 2 * 1024 * 1024)?;
    create_file(&directory.path().join("large"), 3 * 1024 * 1024)?;

    let items = scan(directory.path(), 2 * 1024 * 1024);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].path, directory.path().join("large"));
    assert_eq!(items[0].size, 3 * 1024 * 1024);
    assert_eq!(items[1].path, directory.path().join("medium"));
    Ok(())
}

#[test]
fn reports_a_qualifying_directory_without_its_children() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let nested = directory.path().join("nested");
    fs::create_dir(&nested)?;
    create_file(&nested.join("large-file"), 2 * 1024 * 1024)?;

    let items = scan(directory.path(), 1024 * 1024);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, nested);
    assert!(items[0].is_directory);
    assert_eq!(items[0].size, 2 * 1024 * 1024);
    Ok(())
}

#[cfg(unix)]
#[test]
fn ignores_symbolic_links() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new()?;
    let target = directory.path().join("target");
    create_file(&target, 2 * 1024 * 1024)?;
    symlink(&target, directory.path().join("link"))?;

    let items = scan(directory.path(), 1024 * 1024);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, target);
    Ok(())
}

#[test]
fn reports_paths_to_the_progress_callback() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let file = directory.path().join("file");
    create_file(&file, 2 * 1024 * 1024)?;
    let mut inspected = Vec::new();

    let _ = scan_large_items_with_progress(directory.path(), 1024 * 1024, 10, &mut |path| {
        inspected.push(path.to_path_buf());
    });

    assert!(inspected.contains(&file));
    Ok(())
}

#[cfg(unix)]
#[test]
fn uses_allocated_size_for_sparse_files() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    File::create(directory.path().join("sparse"))?.set_len(1024 * 1024 * 1024)?;

    let items = scan(directory.path(), 1024 * 1024);

    assert!(items.is_empty());
    Ok(())
}

#[cfg(unix)]
#[test]
fn counts_hard_linked_files_once_per_directory() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let nested = directory.path().join("nested");
    fs::create_dir(&nested)?;
    let original = nested.join("original");
    create_file(&original, 2 * 1024 * 1024)?;
    fs::hard_link(&original, nested.join("duplicate"))?;

    let items = scan(directory.path(), 1024 * 1024);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].size, 2 * 1024 * 1024);
    Ok(())
}

#[test]
fn traverses_home_containers_instead_of_reporting_them() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let work = directory.path().join("Work");
    let dependencies = work.join("rust/project/node_modules");
    fs::create_dir_all(&dependencies)?;
    create_file(&dependencies.join("package"), 2 * 1024 * 1024)?;

    let items = scan(directory.path(), 1024 * 1024);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, dependencies);
    Ok(())
}

#[test]
fn traverses_common_hidden_home_directories_instead_of_reporting_them() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let config = directory.path().join(".config");
    fs::create_dir(&config)?;
    let application = config.join("application");
    fs::create_dir(&application)?;
    create_file(&application.join("settings"), 2 * 1024 * 1024)?;

    let items = scan(directory.path(), 1024 * 1024);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, application.join("settings"));
    Ok(())
}

#[test]
fn reports_direct_children_when_browsing_a_directory() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let child_directory = directory.path().join("child-directory");
    fs::create_dir(&child_directory)?;
    create_file(&child_directory.join("file"), 2 * 1024 * 1024)?;
    let child_file = directory.path().join("child-file");
    create_file(&child_file, 1024 * 1024)?;

    let items = scan_directory_children_with_progress(directory.path(), 10, &mut |_| {});

    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|item| item.path == child_directory && item.is_directory));
    assert!(items.iter().any(|item| item.path == child_file && !item.is_directory));
    Ok(())
}
