# Contributing

Read [AGENTS.md](AGENTS.md) for repository workflow rules and the
[README](README.md) for prerequisites and run commands. This checkout is independent
of `simple-calculator`; do not modify or deploy the original project.

## First development session

1. Install Rust through rustup; the repository pins the toolchain and WASM target.
   Install platform build dependencies from [CI](.github/workflows/ci.yml).
2. Run `cargo test --workspace --locked` and `cargo run -p calculator` to establish
   a native baseline. A graphical session is required for the latter.
3. Install Trunk 0.21.14 with `cargo install trunk --version 0.21.14 --locked`.
   Use `trunk serve` for UI iteration, or `python scripts/build-web.py` followed
   by `node scripts/server.cjs` to exercise offline installation and updates.
4. Run `npm ci` and `npx playwright install chromium firefox webkit` before browser
   testing. Use Python >=3.11 and Node.js >=24.

## Find the right code

See [architecture](docs/architecture.md) for the source map and behavior contracts.
Calculation changes belong in the pure core; browser lifecycle changes belong in
the web shell. Keep code, comments, documentation and meaningful identifiers in
English. User-visible text supports German and English: the initial default comes
from the system/browser locale, while an explicit choice in Settings is persisted.
Keep both translations consistent when adding or changing UI text. Serialized
compatibility names require separate consideration; do not rename stored fields
merely to change prose.

Preserve both modes' calculations, existing preferences, stable touch hit targets,
and release-on-touch-end behavior. Changes to persisted data must retain old JSON
compatibility or provide an explicit migration with regression coverage. Do not
regenerate reference fixtures to conceal a behavior change; understand differences
against the original first.

## Verify and review

Use the checks listed in the README. For a complete local gate after installing
tools/browser engines, run `python scripts/check.py`; its native smoke test needs
a graphical session. Focused Rust tests are useful during iteration. Web changes
also need `python scripts/test_web.py`, the Node lifecycle tests, a fresh web build
and the Chromium/Firefox/WebKit browser matrix. Windows packaging must pass its
DLL-import checks; Linux window startup is covered with Xvfb/Mesa in CI.

Document observable changes and remaining device limitations. Do not claim an
iPhone fix from desktop emulation alone. Historical benchmarks are evidence for
the measured environment, not language-wide performance claims.

Pushes to `main` automatically deploy only the checked `calculator-dev` artifact.
Production publication requires an explicitly requested version release and the
[release procedure](docs/releases.md). Include version-specific release notes;
do not move an existing version tag or replace differing release assets.
