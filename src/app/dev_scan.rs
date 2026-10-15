use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use dialoguer::{Confirm, theme::ColorfulTheme};

use super::presentation::format_size;
use super::progress::SingleLineScanProgress;
use crate::scanner::DEVELOPMENT_CLEANUP_DIRECTORIES;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[derive(Debug, Eq, PartialEq)]
struct CleanupDirectory {
    path: PathBuf,
    size: u64,
}

pub(super) fn run(root: &Path, theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    let mut progress = SingleLineScanProgress::new("development projects");
    let directories = scan(root, &mut progress);
    progress.clear();
    if directories.is_empty() {
        println!("No development cleanup folders found.");
        return Ok(());
    }

    let total_size: u64 = directories.iter().map(|directory| directory.size).sum();
    println!("Development cleanup candidates:");
    for directory in &directories {
        println!("  {:>10}  {}", format_size(directory.size), directory.path.display());
    }
    println!("\nTotal storage to clear: {}\n", format_size(total_size));

    let confirmed = Confirm::with_theme(theme)
        .with_prompt(format!("Delete all {} development folders?", directories.len()))
        .default(false)
        .interact()?;
    if !confirmed {
        return Ok(());
    }

    delete(&directories);
    Ok(())
}

fn scan(root: &Path, progress: &mut SingleLineScanProgress) -> Vec<CleanupDirectory> {
    let project_roots = find_git_roots(root, progress);
    let mut directories = HashMap::new();
    for project_root in project_roots {
        find_cleanup_directories(&project_root, &mut directories, progress);
    }

    let mut directories: Vec<_> = directories.into_iter().map(|(path, size)| CleanupDirectory { path, size }).collect();
    directories.sort_by_key(|directory| Reverse(directory.size));
    directories
}

fn find_git_roots(root: &Path, progress: &mut SingleLineScanProgress) -> Vec<PathBuf> {
    let mut project_roots = Vec::new();
    collect_git_roots(root, &mut project_roots, progress);
    project_roots
}

fn collect_git_roots(path: &Path, project_roots: &mut Vec<PathBuf>, progress: &mut SingleLineScanProgress) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        progress.update(&entry_path);
        let Ok(metadata) = fs::symlink_metadata(&entry_path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        if entry.file_name() == ".git" {
            project_roots.push(path.to_path_buf());
            continue;
        }
        collect_git_roots(&entry_path, project_roots, progress);
    }
}

fn find_cleanup_directories(
    project_root: &Path,
    directories: &mut HashMap<PathBuf, u64>,
    progress: &mut SingleLineScanProgress,
) {
    let Ok(entries) = fs::read_dir(project_root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        progress.update(&path);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        if is_cleanup_directory(&path) {
            let size = directory_size(&path, progress);
            directories.entry(path).or_insert(size);
            continue;
        }
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        find_cleanup_directories(&path, directories, progress);
    }
}

fn is_cleanup_directory(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).is_some_and(|name| DEVELOPMENT_CLEANUP_DIRECTORIES.contains(&name))
}

fn directory_size(path: &Path, progress: &mut SingleLineScanProgress) -> u64 {
    directory_size_with_seen_files(path, &mut HashSet::new(), progress)
}

fn directory_size_with_seen_files(
    path: &Path,
    seen_files: &mut HashSet<(u64, u64)>,
    progress: &mut SingleLineScanProgress,
) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            progress.update(&path);
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                return 0;
            };
            if metadata.file_type().is_symlink() {
                return 0;
            }
            if metadata.is_dir() {
                return directory_size_with_seen_files(&path, seen_files, progress);
            }
            if metadata.is_file() && remember_file(&metadata, seen_files) {
                return file_size(&metadata);
            }
            0
        })
        .sum()
}

#[cfg(unix)]
fn remember_file(metadata: &fs::Metadata, seen_files: &mut HashSet<(u64, u64)>) -> bool {
    seen_files.insert((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn remember_file(_: &fs::Metadata, _: &mut HashSet<(u64, u64)>) -> bool {
    true
}

#[cfg(unix)]
fn file_size(metadata: &fs::Metadata) -> u64 {
    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn file_size(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

fn delete(directories: &[CleanupDirectory]) {
    for directory in directories {
        match fs::remove_dir_all(&directory.path) {
            Ok(()) => println!("Deleted {}", directory.path.display()),
            Err(error) => report_delete_error(&directory.path, &error),
        }
    }
}

fn report_delete_error(path: &Path, error: &io::Error) {
    eprintln!("Could not delete {}: {error}", path.display());
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::Path,
        process,
    };

    use super::{Error, SingleLineScanProgress, scan};

    #[test]
    fn finds_cleanup_directories_only_inside_git_projects() -> Result<(), Box<dyn Error>> {
        let root = env::temp_dir().join(format!("cleanmymachine-dev-scan-{}", process::id()));
        let project = root.join("project");
        fs::create_dir_all(project.join(".git"))?;
        fs::create_dir_all(project.join("target/nested"))?;
        fs::create_dir_all(root.join("not-a-project/target"))?;
        let mut file = File::create(project.join("target/nested/build"))?;
        file.write_all(&vec![0; 1024])?;

        let mut progress = SingleLineScanProgress::new("test");
        let directories = scan(&root, &mut progress);

        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0].path, project.join("target"));
        assert!(directories[0].size > 0);
        remove_directory(&root);
        Ok(())
    }

    fn remove_directory(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
