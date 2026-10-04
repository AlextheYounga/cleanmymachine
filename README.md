# CleanMyMachine

A CleanMyMac-style interactive CLI for macOS and Linux. It finds large cache
items and large files or directories, then lets you select items to delete.

## Install

```bash
cargo install --path .
```

## Usage

```bash
cleanmymachine
```

The menu provides two scans:

- **Cache files**: scans direct children of common macOS cache and log roots,
  or the Linux XDG cache directory (`$XDG_CACHE_HOME` or `~/.cache`). Items at
  least 512 MiB are reported.
- **Large files**: recursively scans a directory you choose, defaulting to
  `~/Documents`. Files and directories at least 1 GiB are reported.

Results are ordered largest first. While scanning, the active path is shown on
one updating terminal line. The scanner does not follow symbolic links and
silently skips inaccessible paths.

Enter comma-separated item numbers to select results for deletion, such as
`1, 3, 5`. The result list remains visible while you enter your selection.

## Safety

Deletion is permanent. The CLI requires selecting individual items and a final
confirmation before it removes anything. Review every selected path carefully.

The tool does not scan `/var/cache` on Linux because it is normally managed by
the system and requires elevated permissions.
