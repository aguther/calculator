# Repository workflow

The repository owner explicitly authorizes implementation directly on `main`.
Use the existing primary checkout for future work; a dedicated task branch and
worktree are not required for this repository. Preserve unrelated user changes.

Pushes to `main` run the complete checks and deploy the verified web artifact to
the separate `calculator-dev` Worker. They must not deploy production. Publish only when the
user requests a release, using the version-tag workflow after successful checks.
Keep the original `simple-calculator` repository and deployment unchanged.

Keep source identifiers, comments, developer documentation and release notes in
English. The application intentionally supports German and English; preserve
locale detection, the saved language override and existing persistence keys.
