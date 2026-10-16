# CleanMyMachine

A CleanMyMac-style interactive CLI for macOS and Linux. It finds large cache
items and large files or directories, then lets you select items to delete.

## Install

```bash
cargo install --path .
```

## Usage

```bash
cleanmymachine scan
```

Use `cleanmymachine dev-scan` to find `target`, `node_modules`, and `.venv`
folders inside Git projects under a selected directory. It lists every folder,
shows the combined storage total, and asks for confirmation before deleting
them.

The scan prompts for a directory (defaulting to the home directory), scans
cache locations and that directory, then presents three categories with their
combined storage sizes:

- **Cache files**: scans direct children of common macOS cache and log roots,
  or the Linux XDG cache directory (`$XDG_CACHE_HOME` or `~/.cache`). Items at
  least 512 MiB are reported.
- **Large files**: files at least 1 GiB found while recursively scanning the
  chosen directory.
- **Large directories**: directories at least 1 GiB found while recursively
  scanning the chosen directory.
- **Docker storage**: Docker-reported reclaimable storage, excluding volumes.
  Selecting this category runs Docker's default safe prune operations for
  stopped containers, dangling images, unused networks, and build cache.

Selecting a category opens the paged checkbox list for that category. The
category list remains available until you choose Exit.

Docker cleanup requires the Docker CLI and a running Docker daemon. The CLI
does not use Docker's `--all` options, prune volumes, or delete Docker storage
files directly.

Common home containers such as `Work`, `Documents`, `Downloads`, and `Videos`
are traversed but are not offered as deletion candidates themselves. This
allows large content inside them, including project dependency and build
directories such as `node_modules` and `target`, to be discovered without
offering their project ancestors for deletion.

Results are ordered largest first. While scanning, the active path is shown on
one updating terminal line. The scanner does not follow symbolic links and
silently skips inaccessible paths.
On macOS and Linux, displayed sizes use allocated disk space rather than sparse
files' virtual capacity, and hard-linked files are counted once per directory.

Filesystem scan results are cached for one hour per selected directory to avoid
repeating expensive scans. Successful file or folder deletions clear the cache.
Docker storage is always queried live.

Use the paged checkbox list to select results for deletion. The list displays
15 items at a time to keep terminal redraws small and responsive.

## Safety

Deletion is permanent. The CLI requires selecting individual items and a final
confirmation before it removes anything. Review every selected path carefully.

The tool does not scan `/var/cache` on Linux because it is normally managed by
the system and requires elevated permissions.
