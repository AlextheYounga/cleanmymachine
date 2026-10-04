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
GiB. Results will be sorted largest first, summarized, offered in a Dialoguer
multi-select, and permanently removed only after a negative-by-default
confirmation.

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
symlink exclusion.

## Responsibilities and boundaries

`src/main.rs` owns Clap setup, the interactive menu, user prompts, rendering,
and deletion reporting. `src/scanner.rs` owns filesystem discovery, size
calculation, platform cache-root selection, and the `Item` data type. The
entry point does not contain filesystem traversal logic.

## Affected areas

`Cargo.toml` will configure Clap derive support. `src/main.rs` will become the
interactive application. `src/scanner.rs` will contain scanner behavior and
unit tests. `README.md` will document installation, workflows, thresholds, and
the permanent-deletion warning.

## Decisions

Use the standard library rather than a directory-walking dependency because
the required traversal and error policy are small and explicit. Use MiB and
GiB binary thresholds to match the reference behavior. Limit each scan to 1,000
reported items as in the reference CLI. Use the Linux XDG user cache directory
because it is the standard writable cache location and does not require
privileges. Do not scan `/var/cache` because it normally requires elevated
access and raises the risk of removing system-managed data.

## Risks

Scanning large directory trees can take substantial time and may encounter
permission-denied or concurrently removed paths; these paths will be skipped.
Permanent deletion can cause data loss; explicit item selection and a default
negative confirmation reduce this risk but do not eliminate it. Directory
sizes may change between scanning and deletion.

## Validation

Run scanner unit tests against temporary filesystem fixtures, including a
symlink case on Unix. Run `cargo fmt --check`, `cargo clippy --all-targets
--all-features`, and `cargo test`. Verify `cargo run -- --help` exposes the
program description and standard help/version flags. Inspect the final diff to
confirm deletion remains confirmation-gated and that unrelated files are not
changed.
