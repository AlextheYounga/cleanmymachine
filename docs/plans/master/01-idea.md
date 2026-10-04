# Idea

## Request

Create a Rust CLI using Clap and Dialoguer that provides the CleanMyMac-style
macOS workflow from the linked JavaScript CLI and adds Linux support.

## Problem

The Rust crate contains only a dependency scaffold and prints "Hello, world!".
It cannot identify large items or cache data for cleanup on either supported
platform.

## Definitions

**Cache item:** A direct child of a configured platform cache root whose total
size meets the cache threshold. It can be a regular file or a directory.

**Large item:** A regular file or directory in the user-selected scan root
whose total size meets the large-item threshold. A qualifying directory is
reported as one item and is not traversed further for reporting.

**Deletion:** Permanent removal of a selected regular file or directory. It
does not move an item to a trash or recycle-bin location.

## Desired outcome

On macOS and Linux, users can interactively scan common cache locations or a
chosen directory, review the largest matching items, select items to remove,
and confirm before permanent deletion. Inaccessible paths are skipped and
reported items are sorted by size.

## Scope

The CLI will provide a menu for cache scanning, large-item scanning, and exit.
It will discover macOS cache roots equivalent to the JavaScript CLI and Linux
user cache roots using the XDG cache directory. It will display a summary,
offer results in a 15-row paged Dialoguer checkbox list, show the path currently
being scanned, and report deletion successes and failures.

## Constraints

The implementation uses Rust, Clap, and Dialoguer. It supports macOS and
Linux. Scanning does not follow symbolic links, and deletion is only available
through the interactive confirmation flow.

## Exclusions

Windows support, trash/recycle-bin integration, privileged scanning, automated
cleanup, shell completions, and a non-interactive deletion interface are not
part of this change.
