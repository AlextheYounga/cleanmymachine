use std::{error::Error, fs};

use dialoguer::{Confirm, theme::ColorfulTheme};

use crate::scanner::Item;

pub(super) fn delete_items(items: &[Item], theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    if !confirm_deletion(items.len(), theme)? {
        println!("Operation cancelled.");
        return Ok(());
    }

    let (deleted, failed) = remove_items(items);
    println!("Deleted {deleted} item(s). Failed to delete {failed} item(s).");
    Ok(())
}

fn confirm_deletion(item_count: usize, theme: &ColorfulTheme) -> Result<bool, dialoguer::Error> {
    Confirm::with_theme(theme)
        .with_prompt(format!("Permanently delete {item_count} selected item(s)?"))
        .default(false)
        .interact()
}

fn remove_items(items: &[Item]) -> (usize, usize) {
    let mut deleted = 0;
    let mut failed = 0;

    for item in items {
        if let Err(error) = remove_item(item) {
            failed += 1;
            eprintln!("Could not delete {}: {error}", item.path.display());
            continue;
        }
        deleted += 1;
    }

    (deleted, failed)
}

fn remove_item(item: &Item) -> Result<(), Box<dyn Error>> {
    let metadata = fs::symlink_metadata(&item.path)?;
    if metadata.file_type().is_symlink() {
        return Err("refusing to delete a symbolic link".into());
    }
    if metadata.is_dir() {
        fs::remove_dir_all(&item.path)?;
        return Ok(());
    }
    if metadata.is_file() {
        fs::remove_file(&item.path)?;
        return Ok(());
    }
    Err("path is neither a regular file nor a directory".into())
}
