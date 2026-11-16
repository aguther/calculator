# Calculator

A Rust time calculator built with egui/eframe for browsers, Windows and Linux.
This is an independent comparison implementation of
[simple-calculator](https://github.com/aguther/simple-calculator). The original
repository and deployment remain unchanged.

## Features

Time and number modes, shorthand (`145` → `1:45`), seconds, scalar
multiplication/division, operator precedence, parentheses, subtotals, history and
undo. Original, Light and Dark themes work in the PWA and desktop app. The interface supports German and English. The initial language follows the
system or browser locale; a manual choice in Settings is saved on the device.

Calculations and preferences stay on the device: no telemetry or synchronization.
Inputs are limited to 64 characters and history to 1,000 entries.

## Run locally

Rustup installs the pinned toolchain and WASM target from `rust-toolchain.toml`.
Windows builds also require Visual Studio C++ Build Tools. Install Python >=3.11
and Node.js >=24 for web builds and tests. Linux build dependencies are listed in
[CI](.github/workflows/ci.yml); see [desktop requirements](docs/desktop.md).

```sh
cargo run -p calculator
cargo install trunk --version 0.21.14 --locked
trunk serve
```

Open http://127.0.0.1:8080. For the complete offline PWA, build and serve it with:

```sh
python scripts/build-web.py
node scripts/server.cjs
```

Open http://127.0.0.1:8765. Trunk alone intentionally does not generate a service
worker. Start with [Contributing](CONTRIBUTING.md) for setup, checks and a source
map, or read the [architecture](docs/architecture.md) for persistence and update
contracts.

## Check changes

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p calculator --target wasm32-unknown-unknown --locked -- -D warnings
cargo test --workspace --locked
node scripts/reference-fixtures.cjs --check
python scripts/test_web.py
node --test tests/sw.test.cjs tests/viewport.test.cjs tests/audio.test.cjs tests/pwa-update.test.cjs
python scripts/build-web.py
npm ci
npx playwright install chromium firefox webkit
npm test
```

Browser tests require the complete web build. Set `PLAYWRIGHT_BROWSERS_PATH` to
`.cache/ms-playwright` to isolate browser downloads if needed.
`cargo run --release -p calculator --example smoke` opens a test window and exits
after three frames; it needs a working graphical session.

## Deployment and releases

Pushes to `main` deploy the verified web artifact to the separate
[calculator-dev site](https://calculator-dev.andreas-7f3.workers.dev) only after
all Windows, Linux and web checks pass. Pull requests do not deploy.
Production (`calculator`) and GitHub desktop downloads are published only by a
stable version tag, such as `v0.1.10`, after successful release checks.
See the [release procedure](docs/releases.md) for secrets, gates and recovery.
Do not create a first production tag before app acceptance.

## Differences and limitations

All variants use an egui canvas. Browsers require WebAssembly, JavaScript and
WebGL. The original HTML/CSS is not reused, so fonts, scrolling and touch gestures
can differ. The canvas provides an HTML live output and keyboard controls, but
not the original DOM app's full semantic screen-reader interaction. Browser zoom
and operating-system scaling are supported; installed iOS PWAs still need real
device verification.

Haptics depend on browser/device support. Desktop offers optional key sounds but
no general haptics interface. Calculation remains available without audio and,
when local storage is blocked, for the current session. Data is not automatically
imported from the original app.

## Project documentation

- [Architecture and source navigation](docs/architecture.md)
- [Developer onboarding](CONTRIBUTING.md)
- [Desktop installation](docs/desktop.md)
- [Release procedure](docs/releases.md) and [version notes](docs/release-notes/)
- [UI and PWA investigation notes](docs/ui-refinement.md)
- [Performance methodology and historical measurements](docs/performance.md)

## Attribution

MIT, Copyright 2026 Andreas Guther. Calculation rules, reference code, icons and
design originate from `aguther/simple-calculator`, reference commit `0274b42`.
The JavaScript core and extracted state functions in `tests/reference/` are used
only for synthetic comparisons; shipped calculations run in Rust.

Noto Sans Regular/SemiBold (SIL OFL 1.1) and egui's fallback fonts are embedded.
Available full Cargo dependency licenses, including font licenses, accompany the
desktop archives and ship as `THIRD_PARTY_LICENSES.txt` in the web build.
