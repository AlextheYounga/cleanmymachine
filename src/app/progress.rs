use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const SCANNER_COUNT: usize = 4;

#[derive(Clone)]
pub(super) struct ScanProgressDisplay {
    state: Arc<Mutex<DisplayState>>,
}

struct DisplayState {
    paths: [Option<String>; SCANNER_COUNT],
    completed: [bool; SCANNER_COUNT],
    rendered: bool,
}

impl ScanProgressDisplay {
    pub(super) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DisplayState {
                paths: [None, None, None, None],
                completed: [false, false, false, false],
                rendered: false,
            })),
        }
    }

    fn update(&self, index: usize, path: &Path) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.paths[index] = Some(abbreviated_path(path));
        render(&mut state);
    }

    fn finish(&self, index: usize) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.completed[index] = true;
        render(&mut state);
    }

    pub(super) fn clear(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if !state.rendered {
            return;
        }
        eprint!("\x1b[{}A", SCANNER_COUNT - 1);
        for index in 0..SCANNER_COUNT {
            eprint!("\r\x1b[2K");
            if index + 1 < SCANNER_COUNT {
                eprint!("\n");
            }
        }
        let _ = io::stderr().flush();
        state.rendered = false;
    }
}

pub(super) struct ScanProgress {
    last_rendered: Option<Instant>,
    index: usize,
    display: ScanProgressDisplay,
}

pub(super) struct SingleLineScanProgress {
    last_rendered: Option<Instant>,
    label: &'static str,
}

impl SingleLineScanProgress {
    pub(super) fn new(label: &'static str) -> Self {
        Self { last_rendered: None, label }
    }

    pub(super) fn update(&mut self, path: &Path) {
        if self.last_rendered.is_some_and(|last| last.elapsed() < Duration::from_millis(80)) {
            return;
        }
        self.last_rendered = Some(Instant::now());
        eprint!("\r\x1b[2KScanning {}: {}", self.label, abbreviated_path(path));
        let _ = io::stderr().flush();
    }

    pub(super) fn clear(&self) {
        if self.last_rendered.is_none() {
            return;
        }
        eprint!("\r\x1b[2K");
        let _ = io::stderr().flush();
    }
}

impl ScanProgress {
    pub(super) fn new(index: usize, display: ScanProgressDisplay) -> Self {
        Self { last_rendered: None, index, display }
    }

    pub(super) fn update(&mut self, path: &Path) {
        if self.should_wait() {
            return;
        }
        self.last_rendered = Some(Instant::now());
        self.display.update(self.index, path);
    }

    pub(super) fn finish(&self) {
        self.display.finish(self.index);
    }

    fn should_wait(&self) -> bool {
        self.last_rendered.is_some_and(|last| last.elapsed() < Duration::from_millis(80))
    }
}

fn render(state: &mut DisplayState) {
    if state.rendered {
        eprint!("\x1b[{}A", SCANNER_COUNT - 1);
    }
    for (line_index, path) in state.paths.iter().enumerate() {
        let line_label = match line_index {
            0 => "Cache files",
            1 => "Large files",
            2 => "Large directories",
            _ => "Docker storage",
        };
        let status = if state.completed[line_index] {
            "Done".to_string()
        } else {
            path.as_deref().unwrap_or("waiting...").to_string()
        };
        eprint!("\r\x1b[2KScanning {line_label}: {status}");
        if line_index + 1 < SCANNER_COUNT {
            eprint!("\n");
        }
    }
    state.rendered = true;
    let _ = io::stderr().flush();
}

fn abbreviated_path(path: &Path) -> String {
    const MAXIMUM_CHARACTERS: usize = 100;

    let path = path.display().to_string();
    if path.chars().count() <= MAXIMUM_CHARACTERS {
        return path;
    }
    let suffix: String =
        path.chars().rev().take(MAXIMUM_CHARACTERS - 3).collect::<Vec<_>>().into_iter().rev().collect();
    format!("...{suffix}")
}
