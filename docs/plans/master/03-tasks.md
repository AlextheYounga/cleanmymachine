# Tasks

## Implementation

- [x] Split CLI interaction into orchestration, deletion, presentation,
  progress, and selection modules.
- [x] Move scanner behavior behind the library boundary and scanner coverage
  into an integration test.
- [x] Replace the redrawing multi-select with validated, comma-separated
  numbered selection.
- [x] Pass scan progress callbacks through filesystem traversal and render the
  active path on one throttled terminal line.
- [x] Configure Clap derive support and introduce the platform-aware scanner
  module with bounded recursive discovery and size calculation.
- [x] Replace the greeting with the interactive cache scan, large-item scan,
  multi-select, confirmation, and deletion workflow.
- [x] Add runnable scanner tests using temporary filesystem fixtures.
- [x] Document the command, platform cache roots, thresholds, and permanent
  deletion behavior in the README.

## Validation

- [x] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`,
  and `cargo test` after moving module boundaries.
- [x] Verify scanner progress callbacks and numbered-selection parsing with
  unit tests.
- [ ] Manually verify that scan status is updated in place and selection leaves
  the displayed results stable.
- [x] Run `cargo fmt --check` without formatting differences.
- [x] Run `cargo clippy --all-targets --all-features` without warnings.
- [x] Run `cargo test` with all scanner tests passing.
- [x] Run `cargo run -- --help` and verify Clap help output.

## Completion

- [x] Review the final diff for accidental changes and confirmation-gated
  deletion.

## Completion notes

`cargo fmt --check`, `cargo clippy --all-targets --all-features`, and `cargo
test` passed. Clap help and version output were verified with `cargo run --
--help` and `cargo run -- --version`. No deviations from the plan.

The progress callback and selection parser are covered by unit tests. A visual
terminal check of the in-place status rendering remains pending.
