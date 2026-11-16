# Release procedure

## One-time setup

1. Integrate reviewed app changes into `main`. After successful checks, `main`
   publishes only to the development address.
2. Add `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` under GitHub Settings →
   Secrets and variables → Actions. The token needs Workers/Static Assets
   deployment access in the intended account. Never write values to files or logs.
3. Do not configure automatic Cloudflare Git deployment. If `calculator` is already
   Git-connected, disable its automatic production/preview builds. Keep the
   existing `simple-calculator` Worker unchanged.
4. Enable GitHub Actions. The release workflow grants write access only to its
   publish job.

## Publish a version

Update `[workspace.package]` and `Cargo.lock`. Describe user-visible features,
fixes and any update instructions in `docs/release-notes/VERSION.md`. This file
becomes the GitHub release description; a missing or empty description fails
publication. Resumed releases use the same file. Review the change and integrate
it into `main`, then create a new immutable tag:

```sh
git checkout main
git pull --ff-only
git tag -a v0.1.10 -m "Calculator 0.1.10"
git push origin v0.1.10
```

Use the actual new version, not an already published tag. The workflow checks the
tag/version match and ancestry from `main`. Every platform build and test must
pass before publication. It first uploads binary archives and `SHA256SUMS.txt`
to a draft release, deploys the exact verified PWA to Cloudflare, then publishes
the release.

Publication is globally serialized. An older version cannot overwrite a newer
published version or one reserved by a draft. This protects production even if
finalization fails after a successful Cloudflare deployment.

Downloads: `calculator-VERSION-windows-x64.zip`,
`calculator-VERSION-linux-x64.tar.gz`, `SHA256SUMS.txt`. Windows contains one
unsigned EXE; Linux contains an executable with documented system libraries.
Neither requires WebView or Rust. The initial Linux baseline is Ubuntu 22.04
(glibc 2.35).

## Recover a failed publication

Use GitHub **Re-run failed jobs** to reuse existing build artifacts. Existing
release files are compared by SHA-256; differing bytes are never automatically
replaced. A full rebuild may produce different archive bytes and is deliberately
rejected in that case. Corrected source requires a new version and tag; do not
move an existing tag.

Missing Cloudflare secrets leave the GitHub release as a draft. If deployment
succeeds but release finalization fails, the browser version may already be newer;
rerun the failed job to complete publication. GitHub and Cloudflare publication
cannot be atomic.

## PWA updates

A new service worker preloads the complete app. If loading fails, its cache is
removed and the previous version stays active. A button offers a ready update.
After confirmation, open tabs reload into the new version. Persisted calculator
state is retained. See [architecture](architecture.md) for lifecycle details.

## Development deployment from main

Every push to `main` runs the full Windows, Linux and web checks. Only after all
pass does CI publish the already verified web artifact to:

https://calculator-dev.andreas-7f3.workers.dev

The job uses only `wrangler.dev.jsonc` and the `calculator-dev` Worker. Pull
requests and release tags do not trigger this deployment. Superseded main builds
are skipped. After deployment, the served build hash is compared with the checked
artifact.

The development address is public. Its separate origin gives it independent
calculations, settings and PWA caches. Production remains `calculator` and changes
only through version tags. The original `simple-calculator` Worker stays unchanged.
All GitHub Actions and npm build steps use Node.js 24.
