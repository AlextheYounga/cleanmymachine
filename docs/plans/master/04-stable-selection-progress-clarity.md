# Stable Selection And Progress Clarity

## Trigger

The interactive selection UI flickers when a user changes a selection, and
scans provide no indication of the path currently being inspected.

## Decision

Display results as a stable numbered list and collect comma-separated item
numbers through a Dialoguer text prompt instead of Dialoguer's redrawing
multi-select widget. Invalid, duplicate, and out-of-range entries are rejected
before deletion confirmation.

Scanner entry points accept a progress callback and invoke it for each
filesystem path inspected. The CLI throttles callback rendering and updates one
stderr terminal line with the active path, clearing it before displaying
results or errors.

## Supersedes

The multi-select decision in `02-plan.md`. Dialoguer remains the CLI input
library.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to include stable
numbered selection and scan progress.
