use crate::scanner::Item;

pub(super) fn display_items(items: &[Item]) {
    println!("\nTop 5 largest items:");
    for item in items.iter().take(5) {
        println!("  {:>10}  {:<9} {}", format_size(item.size), item_type(item), item.path.display());
    }
    println!("\nTotal size: {}", format_size(items.iter().map(|item| item.size).sum()));
    println!("\nAvailable items:");
    for (index, item) in items.iter().enumerate() {
        println!("  {:>4}. {}", index + 1, item_label(item));
    }
    println!();
}

pub(super) fn item_label(item: &Item) -> String {
    format!("{}  {}  ({})", format_size(item.size), item.path.display(), item_type(item))
}

fn item_type(item: &Item) -> &str {
    if item.is_directory {
        return "directory";
    }
    "file"
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
        return format!("{size} B");
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn formats_binary_sizes() {
        assert_eq!(format_size(1_536), "1.5 KiB");
    }
}
