# Paged Dialoguer Selection Clarity

## Trigger

The numbered text-selection replacement removed the expected interactive
Dialoguer checkbox experience. The original flicker came from redrawing an
unbounded result list rather than from the selection model itself.

## Decision

Restore Dialoguer's `MultiSelect` and cap its visible page to 15 items. This
preserves checkbox keyboard interaction while limiting each terminal redraw to
a small region.

## Supersedes

The numbered, comma-separated selection decision in
`04-stable-selection-progress-clarity.md`.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to describe and track
the paged Dialoguer selection.
