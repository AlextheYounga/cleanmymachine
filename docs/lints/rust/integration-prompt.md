# Agent Integration Prompt

Please help me integrate our linters with my existing project.

## What These Linters Do

These linters are opinionated, Clean Code-first configurations designed to help AI coding agents produce readable, maintainable, and correct code by default.

They enforce strict standards for:
- Code structure and organization
- Naming conventions
- Function complexity and size
- Dead code elimination
- Import ordering
- Explicit over clever implementations
- Safety measures (denying panics, unsafe patterns, unwraps)

## What to Check

Please review the following files that were installed:

  - clippy.toml
  - rustfmt.toml
  - tests/cleancode_file_too_long.rs
  - tests/cleancode_no_duplicated_state.rs
  - tests/cleancode_no_legacy_terms.rs
  - tests/cleancode_no_literal_wrapped_fallback.rs
  - tests/cleancode_no_manufactured_success.rs
  - tests/cleancode_no_vague_naming.rs
  - Cargo.toml

For each linter config you find, please:

1. Understand the specific rules it enforces
2. Scan the project's source code for violations
3. Fix violations systematically where possible
4. If a violation requires architectural changes, document your reasoning

## Integration Considerations

- These configs are self-contained when used at their documented paths
- You may need to adjust specific rules to match project conventions
- Consider the Boy Scout Rule: every lint run should leave the code cleaner than it found it
- Focus on high-signal rules; false negatives in edge cases are acceptable
- Make code self-explanatory by default; avoid code comments explaining obvious intent

## Action Items

After reviewing:
- Identify any direct violations
- Suggest architectural improvements if violations require them
- Run the configured linters and confirm they pass
- Update documentation if the project needs additional setup instructions