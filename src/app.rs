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

use clap::Parser;
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
struct Cli;

pub fn run() -> Result<(), Box<dyn Error>> {
    install_interrupt_handler()?;
    Cli::parse();
    println!("\nCleanMyMachine\n");

    let theme = ColorfulTheme::default();
    loop {
        match select_action(&theme)? {
            Action::ScanCache => clean_items(&scan_cache_items(), &theme)?,
            Action::ScanLargeItems => clean_items(&scan_selected_directory(&theme)?, &theme)?,
            Action::Exit => return Ok(()),
        }
    }
}

fn install_interrupt_handler() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(|| {
        let terminal = Term::stderr();
        let _ = terminal.show_cursor();
        let _ = terminal.flush();
        process::exit(130);
    })
}

enum Action {
    ScanCache,
    ScanLargeItems,
    Exit,
}

fn select_action(theme: &ColorfulTheme) -> Result<Action, dialoguer::Error> {
    let selected = Select::with_theme(theme)
        .with_prompt("What would you like to do?")
        .items(["Scan for cache files", "Scan for large files", "Exit"])
        .default(0)
        .interact()?;

    Ok(match selected {
        0 => Action::ScanCache,
        1 => Action::ScanLargeItems,
        _ => Action::Exit,
    })
}

fn scan_selected_directory(theme: &ColorfulTheme) -> Result<Vec<Item>, dialoguer::Error> {
    let directory: String =
        Input::with_theme(theme).with_prompt("Directory to scan").default(default_scan_directory()).interact_text()?;
    Ok(scan_large_items(&PathBuf::from(directory)))
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
    env::var_os(HOME_ENVIRONMENT_VARIABLE)
        .map_or_else(|| PathBuf::from("."), |home| PathBuf::from(home).join("Documents"))
        .display()
        .to_string()
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
