# Plan

## Current behavior

`src/main.rs` prints a greeting. `Cargo.toml` declares Clap and Dialoguer but
does not use them. The linked `cleanmymac/index.js` implements an interactive
menu, recursively measures files and directories, scans four macOS cache
locations, prompts for selection, and permanently deletes confirmed items.

## Intended behavior

The executable will parse its name, version, and help with Clap, then run the
interactive menu. Cache scans will inspect platform cache roots and return
direct children at or above 512 MiB. Large-item scans will prompt for a root
(defaulting to `~/Documents`) and return files or directories at or above 1
GiB. Results will be sorted largest first, summarized, selected through a
Dialoguer checkbox list capped at 15 visible items, and permanently removed
only after a negative-by-default confirmation. During a scan, the active path
will update on one terminal line.

## Approach

Replace the greeting entry point with focused modules for item discovery and
terminal interaction. The scanner will use standard-library filesystem APIs
and `symlink_metadata` to avoid following symbolic links. Recursive size
calculation will skip inaccessible descendants, treat symlinks as zero size,
and accumulate regular-file lengths. Traversal will stop below a qualifying
directory so users never receive overlapping deletion choices. Platform cache
roots will be selected with conditional compilation: the macOS roots from the
reference CLI and the XDG cache directory on Linux. Tests will use temporary
directories to verify thresholding, directory reporting, sorting inputs, and
symlink exclusion. Scanner entry points will receive a path callback; the CLI
will throttle the callback to update one status line rather than write each
path separately.

## Responsibilities and boundaries

`src/main.rs` owns only process startup. `src/app.rs` owns Clap setup and the
interactive menu. Its focused child modules own deletion, presentation,
progress rendering, and paged checkbox selection. `src/scanner.rs` owns filesystem
discovery, size calculation, platform cache-root selection, and the `Item` data
type. `tests/scanner.rs` owns filesystem scanner coverage.

## Affected areas

`Cargo.toml` configures Clap derive support. `src/main.rs` is the binary entry
point and `src/lib.rs` exposes application modules. `src/app.rs` and
`src/app/` contain CLI behavior. `src/scanner.rs` contains scanner behavior,
and `tests/scanner.rs` contains scanner coverage. `README.md` documents
installation, workflows, thresholds, scan progress, and permanent deletion.

## Decisions

Use the standard library rather than a directory-walking dependency because
the required traversal and error policy are small and explicit. Use MiB and
GiB binary thresholds to match the reference behavior. Limit each scan to 1,000
reported items as in the reference CLI. Use the Linux XDG user cache directory
because it is the standard writable cache location and does not require
privileges. Do not scan `/var/cache` because it normally requires elevated
access and raises the risk of removing system-managed data. Limit Dialoguer's
checkbox list to 15 visible items so navigation redraws only a small terminal
region.

## Risks

Scanning large directory trees can take substantial time and may encounter
permission-denied or concurrently removed paths; these paths will be skipped.
Permanent deletion can cause data loss; explicit item selection and a default
negative confirmation reduce this risk but do not eliminate it. Directory
sizes may change between scanning and deletion. Progress status can expose the
names of paths on the active terminal, so it is emitted only during an
interactive scan.

## Validation

Run scanner unit tests against temporary filesystem fixtures, including a
symlink case on Unix. Run `cargo fmt --check`, `cargo clippy --all-targets
--all-features`, and `cargo test`. Verify `cargo run -- --help` exposes the
program description and standard help/version flags. Manually verify that a
scan updates one status line and selection does not redraw the result list.
Inspect the final diff to confirm deletion remains confirmation-gated and that
unrelated files are not changed.
