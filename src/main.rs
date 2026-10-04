mod scanner;

use std::{error::Error, fs, path::PathBuf};

use clap::Parser;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use scanner::{CACHE_THRESHOLD, Item, LARGE_ITEM_THRESHOLD, RESULT_LIMIT};

#[derive(Parser)]
#[command(
    name = "cleanmymachine",
    version,
    about = "Find large cache files and directories for interactive cleanup"
)]
struct Cli;

fn main() -> Result<(), Box<dyn Error>> {
    Cli::parse();
    println!("\nCleanMyMachine\n");

    let theme = ColorfulTheme::default();
    loop {
        let action = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(["Scan for cache files", "Scan for large files", "Exit"])
            .default(0)
            .interact()?;

        match action {
            0 => clean_items(
                scanner::scan_cache_items(CACHE_THRESHOLD, RESULT_LIMIT),
                &theme,
            )?,
            1 => {
                let directory: String = Input::with_theme(&theme)
                    .with_prompt("Directory to scan")
                    .default(default_scan_directory())
                    .interact_text()?;
                clean_items(
                    scanner::scan_large_items(
                        &PathBuf::from(directory),
                        LARGE_ITEM_THRESHOLD,
                        RESULT_LIMIT,
                    ),
                    &theme,
                )?;
            }
            _ => return Ok(()),
        }
    }
}

fn default_scan_directory() -> String {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Documents"))
        .unwrap_or_else(|| PathBuf::from("."))
        .display()
        .to_string()
}

fn clean_items(items: Vec<Item>, theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    if items.is_empty() {
        println!("No items found matching the criteria.");
        return Ok(());
    }

    display_summary(&items);
    let labels: Vec<String> = items.iter().map(item_label).collect();
    let selected = MultiSelect::with_theme(theme)
        .with_prompt("Select files or folders to delete")
        .items(&labels)
        .interact()?;
    let selected_items: Vec<Item> = selected
        .into_iter()
        .map(|index| items[index].clone())
        .collect();

    if selected_items.is_empty() {
        println!("No items selected for deletion.");
        return Ok(());
    }

    if !Confirm::with_theme(theme)
        .with_prompt(format!(
            "Permanently delete {} selected item(s)?",
            selected_items.len()
        ))
        .default(false)
        .interact()?
    {
        println!("Operation cancelled.");
        return Ok(());
    }

    delete_items(&selected_items);
    Ok(())
}

fn display_summary(items: &[Item]) {
    println!("\nTop 5 largest items:");
    for item in items.iter().take(5) {
        let kind = if item.is_directory {
            "directory"
        } else {
            "file"
        };
        println!(
            "  {:>10}  {:<9} {}",
            format_size(item.size),
            kind,
            item.path.display()
        );
    }
    let total_size: u64 = items.iter().map(|item| item.size).sum();
    println!("\nTotal size: {}\n", format_size(total_size));
}

fn item_label(item: &Item) -> String {
    let kind = if item.is_directory {
        "directory"
    } else {
        "file"
    };
    format!(
        "{}  {}  ({kind})",
        format_size(item.size),
        item.path.display()
    )
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn delete_items(items: &[Item]) {
    let mut deleted = 0;
    let mut failed = 0;

    for item in items {
        match delete_item(&item.path) {
            Ok(()) => deleted += 1,
            Err(error) => {
                failed += 1;
                eprintln!("Could not delete {}: {error}", item.path.display());
            }
        }
    }

    println!("Deleted {deleted} item(s). Failed to delete {failed} item(s).");
}

fn delete_item(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err("refusing to delete a symbolic link".into());
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else if metadata.is_file() {
        fs::remove_file(path)?;
    } else {
        return Err("path is neither a regular file nor a directory".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn formats_binary_sizes() {
        assert_eq!(format_size(1_536), "1.5 KiB");
    }
}
