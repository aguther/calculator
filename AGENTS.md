# Repository workflow

The repository owner explicitly authorizes implementation directly on `main`.
Use the existing primary checkout for future work; a dedicated task branch and
worktree are not required for this repository. Preserve unrelated user changes.

Pushes to `main` run checks but must not deploy production. Publish only when the
user requests a release, using the version-tag workflow after successful checks.
Keep the original `simple-calculator` repository and deployment unchanged.
