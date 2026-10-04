use dialoguer::{MultiSelect, theme::ColorfulTheme};

use super::presentation::item_label;
use crate::scanner::Item;

const VISIBLE_ITEM_COUNT: usize = 15;

pub(super) fn prompt_for_selection(items: &[Item], theme: &ColorfulTheme) -> Result<Vec<Item>, dialoguer::Error> {
    let labels = items.iter().map(item_label);
    let selected = MultiSelect::with_theme(theme)
        .with_prompt("Select files or folders to delete")
        .items(labels)
        .max_length(VISIBLE_ITEM_COUNT)
        .interact()?;

    Ok(selected.into_iter().map(|index| items[index].clone()).collect())
}
