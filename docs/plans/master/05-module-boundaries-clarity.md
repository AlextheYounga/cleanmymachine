# Module Boundaries Clarity

## Trigger

The CLI and scanner source files combine several responsibilities and are
becoming difficult to navigate.

## Decision

Expose `scanner` through `src/lib.rs` and keep `src/main.rs` as a thin binary
entry point. Move CLI orchestration to `src/app.rs`; place deletion,
presentation, scan progress, and selection parsing in dedicated `src/app/`
modules. Move scanner filesystem coverage to `tests/scanner.rs`.

## Supersedes

The earlier plan's placement of all terminal interaction in `src/main.rs` and
scanner unit tests in `src/scanner.rs`.

## Required Authoritative Updates

`02-plan.md` and `03-tasks.md` are updated to describe and track the module
boundaries.
