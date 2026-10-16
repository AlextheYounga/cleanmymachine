use std::path::{Path, PathBuf};

use console::{Key, Term, truncate_str};
use dialoguer::theme::ColorfulTheme;

use super::presentation::item_label;
use crate::scanner::{Item, RESULT_LIMIT, scan_directory_children_with_progress};

const VISIBLE_ITEM_COUNT: usize = 15;

struct CursorRestorer<'a>(&'a Term);

impl Drop for CursorRestorer<'_> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
        let _ = self.0.flush();
    }
}

pub(super) fn prompt_for_selection(
    items: &[Item],
    _theme: &ColorfulTheme,
) -> Result<Option<Vec<Item>>, dialoguer::Error> {
    let terminal = Term::stderr();
    let _cursor_restorer = CursorRestorer(&terminal);
    let mut session = SelectionSession { selected: Vec::new(), terminal: &terminal, rendered_lines: 0 };
    if !session.browse(items)? {
        return Ok(None);
    }
    session.clear()?;
    Ok(Some(remove_descendants_of_selected_items(session.selected)))
}

struct SelectionSession<'a> {
    selected: Vec<Item>,
    terminal: &'a Term,
    rendered_lines: usize,
}

impl SelectionSession<'_> {
    fn browse(&mut self, root_items: &[Item]) -> Result<bool, dialoguer::Error> {
        let mut directories = vec![DirectoryView { path: None, items: root_items.to_vec(), cursor: 0 }];
        loop {
            let Some(view) = directories.last() else {
                return Ok(false);
            };
            self.render(view)?;

            match self.terminal.read_key()? {
                Key::ArrowDown | Key::Char('j') => move_cursor_down(directories.last_mut()),
                Key::ArrowUp | Key::Char('k') => move_cursor_up(directories.last_mut()),
                Key::Char(' ') => {
                    if let Some(item) = current_item(directories.last()) {
                        toggle_selected(item, &mut self.selected);
                    }
                }
                Key::ArrowRight | Key::Enter => open_current_directory(&mut directories),
                Key::ArrowLeft | Key::Escape => {
                    if directories.len() == 1 {
                        self.clear()?;
                        return Ok(false);
                    }
                    directories.pop();
                }
                Key::Char('c' | 'C') => return Ok(true),
                _ => {}
            }
        }
    }

    fn render(&mut self, view: &DirectoryView) -> Result<(), dialoguer::Error> {
        self.clear()?;
        let location =
            view.path.as_ref().map_or_else(|| String::from("scan results"), |path| path.display().to_string());
        self.write_line(&format!("Select items to delete: {location}"))?;
        self.write_line("Up/down: move  Right/Enter: open  Left/Esc: back  Space: select  C: continue")?;

        let start = view.cursor.saturating_sub(VISIBLE_ITEM_COUNT - 1);
        for (index, item) in view.items.iter().enumerate().skip(start).take(VISIBLE_ITEM_COUNT) {
            let pointer = if index == view.cursor { ">" } else { " " };
            self.write_line(&format!("{pointer} {}", selectable_item_label(item, &self.selected)))?;
        }
        self.terminal.flush()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), dialoguer::Error> {
        if self.rendered_lines > 0 {
            self.terminal.clear_last_lines(self.rendered_lines)?;
            self.rendered_lines = 0;
        }
        Ok(())
    }

    fn write_line(&mut self, text: &str) -> Result<(), dialoguer::Error> {
        let (_, columns) = self.terminal.size();
        let width = usize::from(columns).saturating_sub(1);
        self.terminal.write_line(&truncate_str(text, width, "..."))?;
        self.rendered_lines += 1;
        Ok(())
    }
}

struct DirectoryView {
    path: Option<PathBuf>,
    items: Vec<Item>,
    cursor: usize,
}

fn move_cursor_down(view: Option<&mut DirectoryView>) {
    if let Some(view) = view.filter(|view| !view.items.is_empty()) {
        view.cursor = (view.cursor + 1) % view.items.len();
    }
}

fn move_cursor_up(view: Option<&mut DirectoryView>) {
    if let Some(view) = view.filter(|view| !view.items.is_empty()) {
        view.cursor = (view.cursor + view.items.len() - 1) % view.items.len();
    }
}

fn current_item(view: Option<&DirectoryView>) -> Option<&Item> {
    view.and_then(|view| view.items.get(view.cursor))
}

fn open_current_directory(directories: &mut Vec<DirectoryView>) {
    let Some(item) = current_item(directories.last()) else {
        return;
    };
    if !item.is_directory {
        return;
    }

    let path = item.path.clone();
    let items = directory_children(&path);
    directories.push(DirectoryView { path: Some(path), items, cursor: 0 });
}

fn directory_children(path: &Path) -> Vec<Item> {
    scan_directory_children_with_progress(path, RESULT_LIMIT, &mut |_| {})
}

fn selectable_item_label(item: &Item, selected: &[Item]) -> String {
    let marker = if is_selected(item, selected) { "x" } else { " " };
    let suffix = if item.is_directory { " [open]" } else { "" };
    format!("[{marker}] {}{suffix}", item_label(item))
}

fn is_selected(item: &Item, selected: &[Item]) -> bool {
    selected.iter().any(|selected_item| selected_item.path == item.path)
}

fn toggle_selected(item: &Item, selected: &mut Vec<Item>) {
    if let Some(index) = selected.iter().position(|selected_item| selected_item.path == item.path) {
        selected.remove(index);
    } else {
        selected.push(item.clone());
    }
}

fn remove_descendants_of_selected_items(items: Vec<Item>) -> Vec<Item> {
    let selected_paths: Vec<_> = items.iter().map(|item| item.path.clone()).collect();
    items
        .into_iter()
        .filter(|item| {
            !item.path.ancestors().skip(1).any(|ancestor| selected_paths.iter().any(|selected| selected == ancestor))
        })
        .collect()
}
