# Allocated Size And Container Clarity

## Trigger

Sparse Docker files were reported at their virtual capacity, producing totals
larger than the machine's storage. Large top-level home folders also hid useful
cleanup candidates below them.

## Decision

On Unix, measure regular files using allocated filesystem blocks and count each
hard-linked inode once while sizing a directory. Treat common user folders such
as `Work`, `Documents`, `Downloads`, and `Videos` as top-level containers: scan
inside them, but do not offer the containers themselves for deletion. Traverse
these containers deeply so large files and known cleanup directories such as
`node_modules` and `target` are not hidden by qualifying ancestors.

## Safety

The scanner continues to avoid overlapping deletion candidates. A qualifying
directory is still reported without separately reporting its descendants.
