use std::collections::HashSet;

use dialoguer::{Input, theme::ColorfulTheme};

use crate::scanner::Item;

pub(super) fn prompt_for_selection(items: &[Item], theme: &ColorfulTheme) -> Result<Vec<Item>, dialoguer::Error> {
    loop {
        let selection: String = Input::with_theme(theme)
            .with_prompt("Item numbers to delete (for example: 1, 3, 5; blank to cancel)")
            .allow_empty(true)
            .interact_text()?;
        match parse_selection(&selection, items.len()) {
            Ok(indices) => return Ok(indices.into_iter().map(|index| items[index].clone()).collect()),
            Err(message) => eprintln!("Invalid selection: {message}"),
        }
    }
}

fn parse_selection(input: &str, item_count: usize) -> Result<Vec<usize>, String> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for value in input.split(',').map(str::trim) {
        let number: usize = value.parse().map_err(|_| format!("`{value}` is not a number"))?;
        let index = selected_index(number, item_count)?;
        if !seen.insert(index) {
            return Err(format!("`{number}` was selected more than once"));
        }
        selected.push(index);
    }
    Ok(selected)
}

fn selected_index(number: usize, item_count: usize) -> Result<usize, String> {
    if (1..=item_count).contains(&number) {
        return Ok(number - 1);
    }
    Err(format!("`{number}` is outside the available range 1-{item_count}"))
}

#[cfg(test)]
mod tests {
    use super::parse_selection;

    #[test]
    fn parses_unique_one_based_item_numbers() {
        assert_eq!(parse_selection("1, 3, 2", 3), Ok(vec![0, 2, 1]));
    }

    #[test]
    fn rejects_invalid_item_numbers() {
        assert!(parse_selection("1, 1", 3).is_err());
        assert!(parse_selection("4", 3).is_err());
        assert!(parse_selection("one", 3).is_err());
    }
}
