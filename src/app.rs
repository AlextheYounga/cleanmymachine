mod deletion;
mod dev_scan;
mod docker;
mod presentation;
mod progress;
mod selection;

use std::{
    any::Any,
    env,
    error::Error,
    io,
    path::{Path, PathBuf},
    process, thread,
};

use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use self::{
    deletion::delete_items,
    presentation::{display_items, display_selected_items},
    progress::{ScanProgress, ScanProgressDisplay},
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
    /// Find development build and dependency directories inside Git projects.
    DevScan,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    install_interrupt_handler()?;
    let cli = Cli::parse();
    println!("\nCleanMyMachine\n");

    match cli.command {
        Command::Scan => run_scan(&ColorfulTheme::default())?,
        Command::DevScan => run_dev_scan(&ColorfulTheme::default())?,
    }
    Ok(())
}

fn run_dev_scan(theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    let directory = select_scan_directory(theme)?;
    dev_scan::run(&directory, theme)
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

fn scan_cache_items(display: ScanProgressDisplay) -> Vec<Item> {
    let mut progress = ScanProgress::new(0, display);
    let items = scanner::scan_cache_items_with_progress(CACHE_THRESHOLD, RESULT_LIMIT, &mut |path| {
        progress.update(path);
    });
    progress.finish();
    items
}

fn scan_large_items(root: &Path, index: usize, display: ScanProgressDisplay) -> Vec<Item> {
    let mut progress = ScanProgress::new(index, display);
    let items = scanner::scan_large_items_with_progress(root, LARGE_ITEM_THRESHOLD, RESULT_LIMIT, &mut |path| {
        progress.update(path);
    });
    progress.finish();
    items
}

fn default_scan_directory() -> String {
    env::var_os(HOME_ENVIRONMENT_VARIABLE).map_or_else(|| PathBuf::from("."), PathBuf::from).display().to_string()
}

struct ScanCategory {
    name: &'static str,
    items: Vec<Item>,
}

enum ScanOption {
    Files(ScanCategory),
    Docker(docker::DockerCleanup),
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
    let categories = scan_categories(&directory)?;
    select_category(&categories, theme)
}

fn scan_categories(directory: &Path) -> Result<Vec<ScanOption>, Box<dyn Error>> {
    let display = ScanProgressDisplay::new();
    let (cache_items, large_files, large_directories, docker_cleanup) = thread::scope(|scope| {
        let cache_display = display.clone();
        let files_display = display.clone();
        let directories_display = display.clone();
        let docker_display = display.clone();
        let cache_scan = scope.spawn(|| scan_cache_items(cache_display));
        let large_files_scan = scope.spawn(|| scan_large_files(directory, files_display));
        let large_directories_scan = scope.spawn(|| scan_large_directories(directory, directories_display));
        let docker_scan = scope.spawn(|| docker::scan(docker_display));
        let cache_items = cache_scan.join().map_err(scanner_thread_error)?;
        let large_files = large_files_scan.join().map_err(scanner_thread_error)?;
        let large_directories = large_directories_scan.join().map_err(scanner_thread_error)?;
        let docker_cleanup = docker_scan.join().map_err(scanner_thread_error)??;
        display.clear();
        Ok::<_, io::Error>((cache_items, large_files, large_directories, docker_cleanup))
    })?;

    let mut options = vec![
        ScanOption::Files(ScanCategory { name: "Cache files", items: cache_items }),
        ScanOption::Files(ScanCategory { name: "Large files", items: large_files }),
        ScanOption::Files(ScanCategory { name: "Large directories", items: large_directories }),
    ];
    if let Some(docker_cleanup) = docker_cleanup {
        options.push(ScanOption::Docker(docker_cleanup));
    }
    Ok(options)
}

fn scan_large_files(root: &Path, display: ScanProgressDisplay) -> Vec<Item> {
    scan_large_items(root, 1, display).into_iter().filter(|item| !item.is_directory).collect()
}

fn scan_large_directories(root: &Path, display: ScanProgressDisplay) -> Vec<Item> {
    scan_large_items(root, 2, display).into_iter().filter(|item| item.is_directory).collect()
}

fn scanner_thread_error(_: Box<dyn Any + Send>) -> io::Error {
    io::Error::from(io::ErrorKind::BrokenPipe)
}

fn select_category(categories: &[ScanOption], theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    loop {
        let labels: Vec<String> = categories.iter().map(ScanOption::label).collect();
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

        match &categories[selected] {
            ScanOption::Files(category) => clean_items(&category.items, theme)?,
            ScanOption::Docker(cleanup) => clean_docker(cleanup, theme)?,
        }
    }
}

impl ScanOption {
    fn label(&self) -> String {
        match self {
            Self::Files(category) => category.label(),
            Self::Docker(cleanup) => {
                format!("Docker storage ({})", presentation::format_size(cleanup.reclaimable_size))
            }
        }
    }
}

fn clean_docker(cleanup: &docker::DockerCleanup, theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    let confirmed = dialoguer::Confirm::with_theme(theme)
        .with_prompt(format!(
            "Prune Docker storage (up to {} reclaimable)?",
            presentation::format_size(cleanup.reclaimable_size)
        ))
        .default(false)
        .interact()?;
    if !confirmed {
        return Ok(());
    }
    docker::prune()?;
    println!("Docker storage cleanup complete.");
    Ok(())
}

fn clean_items(items: &[Item], theme: &ColorfulTheme) -> Result<(), Box<dyn Error>> {
    if items.is_empty() {
        println!("No items found matching the criteria.");
        return Ok(());
    }

    display_items(items);
    let Some(selected_items) = prompt_for_selection(items, theme)? else {
        return Ok(());
    };
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

    use super::{ScanCategory, ScanOption};
    use crate::scanner::Item;

    fn item(path: &str, size: u64, is_directory: bool) -> Item {
        Item { path: PathBuf::from(path), size, is_directory }
    }

    #[test]
    fn category_label_includes_combined_size() {
        let category =
            ScanCategory { name: "Large files", items: vec![item("first", 1024, false), item("second", 1024, false)] };

        assert_eq!(category.label(), "Large files (2.0 KiB)");
    }

    #[test]
    fn labels_docker_cleanup_with_reclaimable_size() {
        let option = ScanOption::Docker(super::docker::DockerCleanup { reclaimable_size: 2 * 1024 * 1024 });

        assert_eq!(option.label(), "Docker storage (2.0 MiB)");
    }
}
