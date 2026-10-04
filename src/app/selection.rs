use console::Term;
use dialoguer::{MultiSelect, theme::ColorfulTheme};

use super::presentation::item_label;
use crate::scanner::Item;

const VISIBLE_ITEM_COUNT: usize = 15;

struct CursorRestorer<'a>(&'a Term);

impl Drop for CursorRestorer<'_> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
        let _ = self.0.flush();
    }
}

pub(super) fn prompt_for_selection(items: &[Item], theme: &ColorfulTheme) -> Result<Vec<Item>, dialoguer::Error> {
    let labels = items.iter().map(item_label);
    let terminal = Term::stderr();
    let _cursor_restorer = CursorRestorer(&terminal);
    let selected = MultiSelect::with_theme(theme)
        .with_prompt("Select files or folders to delete")
        .items(labels)
        .max_length(VISIBLE_ITEM_COUNT)
        .report(false)
        .interact_on(&terminal)?;

    Ok(selected.into_iter().map(|index| items[index].clone()).collect())
}
