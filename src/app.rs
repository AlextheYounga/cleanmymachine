mod deletion;
mod presentation;
mod progress;
mod selection;

use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process,
};

use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use self::{
    deletion::delete_items,
    presentation::{display_items, display_selected_items},
    progress::ScanProgress,
    selection::prompt_for_selection,
};
use crate::scanner::{self, CACHE_THRESHOLD, HOME_ENVIRONMENT_VARIABLE, Item, LARGE_ITEM_THRESHOLD, RESULT_LIMIT};

#[derive(Parser)]
#[command(name = "cleanmymachine", version, about = "Find large cache files and directories for interactive cleanup")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan cache locations and a directory for cleanup candidates.
    Scan,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    install_interrupt_handler()?;
    let cli = Cli::parse();
    println!("\nCleanMyMachine\n");

    match cli.command {
        Command::Scan => run_scan(&ColorfulTheme::default())?,
    }
    Ok(())
}

fn install_interrupt_handler() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(|| {
        let terminal = Term::stderr();
        let _ = terminal.show_cursor();
        let _ = terminal.flush();
        process::exit(130);
    })
}

fn select_scan_directory(theme: &ColorfulTheme) -> Result<PathBuf, dialoguer::Error> {
    let directory: String =
        Input::with_theme(theme).with_prompt("Directory to scan").default(default_scan_directory()).interact_text()?;
    Ok(PathBuf::from(directory))
}

fn scan_cache_items() -> Vec<Item> {
    let mut progress = ScanProgress::new();
    let items = scanner::scan_cache_items_with_progress(CACHE_THRESHOLD, RESULT_LIMIT, &mut |path| {
        progress.update(path);
    });
    progress.clear();
    items
}

fn scan_large_items(root: &Path) -> Vec<Item> {
    let mut progress = ScanProgress::new();
    let items = scanner::scan_large_items_with_progress(root, LARGE_ITEM_THRESHOLD, RESULT_LIMIT, &mut |path| {
        progress.update(path);
    });
    progress.clear();
    items
}

fn default_scan_directory() -> String {
    env::var_os(HOME_ENVIRONMENT_VARIABLE).map_or_else(|| PathBuf::from("."), PathBuf::from).display().to_string()
}

struct ScanCategory {
    name: &'static str,
    items: Vec<Item>,
}

impl ScanCategory {
    fn total_size(&self) -> u64 {
        self.items.iter().map(|item| item.size).sum()
    }

    fn label(&self) -> String {
        format!("{} ({})", self.name, presentation::format_size(self.total_size()))
    }
}

fn run_scan(theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    let directory = select_scan_directory(theme)?;
    let categories = scan_categories(&directory);
    select_category(&categories, theme)
}

fn scan_categories(directory: &Path) -> Vec<ScanCategory> {
    let cache_items = scan_cache_items();
    let large_items = scan_large_items(directory);
    let (large_directories, large_files) = split_large_items(large_items);

    vec![
        ScanCategory { name: "Cache files", items: cache_items },
        ScanCategory { name: "Large files", items: large_files },
        ScanCategory { name: "Large directories", items: large_directories },
    ]
}

fn split_large_items(items: Vec<Item>) -> (Vec<Item>, Vec<Item>) {
    items.into_iter().partition(|item| item.is_directory)
}

fn select_category(categories: &[ScanCategory], theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    loop {
        let labels: Vec<String> = categories.iter().map(ScanCategory::label).collect();
        let mut options = labels;
        options.push(String::from("Exit"));
        let selected = Select::with_theme(theme)
            .with_prompt("What would you like to review?")
            .items(options)
            .default(0)
            .interact()?;

        if selected == categories.len() {
            return Ok(());
        }

        clean_items(&categories[selected].items, theme)?;
    }
}

fn clean_items(items: &[Item], theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    if items.is_empty() {
        println!("No items found matching the criteria.");
        return Ok(());
    }

    display_items(items);
    let selected_items = prompt_for_selection(items, theme)?;
    if selected_items.is_empty() {
        println!("No items selected for deletion.");
        return Ok(());
    }

    display_selected_items(&selected_items);
    delete_items(&selected_items, theme)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ScanCategory, split_large_items};
    use crate::scanner::Item;

    fn item(path: &str, size: u64, is_directory: bool) -> Item {
        Item { path: PathBuf::from(path), size, is_directory }
    }

    #[test]
    fn separates_large_files_and_directories() {
        let (directories, files) = split_large_items(vec![item("directory", 20, true), item("file", 10, false)]);

        assert_eq!(directories, vec![item("directory", 20, true)]);
        assert_eq!(files, vec![item("file", 10, false)]);
    }

    #[test]
    fn category_label_includes_combined_size() {
        let category =
            ScanCategory { name: "Large files", items: vec![item("first", 1024, false), item("second", 1024, false)] };

        assert_eq!(category.label(), "Large files (2.0 KiB)");
    }
}
