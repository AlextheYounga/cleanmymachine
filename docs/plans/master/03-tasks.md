# Tasks

## Implementation

- [ ] Configure Clap derive support and introduce the platform-aware scanner
  module with bounded recursive discovery and size calculation.
- [ ] Replace the greeting with the interactive cache scan, large-item scan,
  multi-select, confirmation, and deletion workflow.
- [ ] Add runnable scanner tests using temporary filesystem fixtures.
- [ ] Document the command, platform cache roots, thresholds, and permanent
  deletion behavior in the README.

## Validation

- [ ] Run `cargo fmt --check` without formatting differences.
- [ ] Run `cargo clippy --all-targets --all-features` without warnings.
- [ ] Run `cargo test` with all scanner tests passing.
- [ ] Run `cargo run -- --help` and verify Clap help output.

## Completion

- [ ] Review the final diff for accidental changes and confirmation-gated
  deletion.

## Completion notes

Pending implementation approval.
