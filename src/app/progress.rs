use std::{
    io::{self, Write},
    path::Path,
    time::{Duration, Instant},
};

pub(super) struct ScanProgress {
    last_rendered: Option<Instant>,
}

impl ScanProgress {
    pub(super) fn new() -> Self {
        Self { last_rendered: None }
    }

    pub(super) fn update(&mut self, path: &Path) {
        if self.should_wait() {
            return;
        }
        self.last_rendered = Some(Instant::now());
        eprint!("\r\x1b[2KScanning: {}", abbreviated_path(path));
        let _ = io::stderr().flush();
    }

    pub(super) fn clear(&self) {
        if self.last_rendered.is_none() {
            return;
        }
        eprint!("\r\x1b[2K");
        let _ = io::stderr().flush();
    }

    fn should_wait(&self) -> bool {
        self.last_rendered.is_some_and(|last| last.elapsed() < Duration::from_millis(80))
    }
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
