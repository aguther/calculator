# English repository and Windows icon implementation plan

**Goal:** Make the repository approachable to English-speaking contributors and embed the existing brand icon in Windows executables at appropriate DPI sizes.

**Architecture:** Keep the Rust state machine and serialized representation compatible. Improve names and Rustdoc at the API and state-transition boundaries, use PEP 8/docstrings for Python tooling and focused comments for JavaScript lifecycle code. Translate developer-facing Markdown and release descriptions. Store a multi-resolution ICO resource separately from the high-resolution runtime window icon.

**Scope:** Documentation and maintainability improvements, Windows branding resources, existing CI verification. No production release is requested. `main` continues deploying only to development after successful checks. The user clarified that the UI must support German and English, select an initial default from the platform locale and persist an explicit Settings override.

## Tasks

- [x] Translate all developer documentation and versioned release notes to English; update stale deployment guidance.
- [x] Add an architecture map and contributor onboarding, including persistence, rendering/input, PWA lifecycle, tests and release procedures.
- [x] Add Rustdoc for public calculator types/functions and explain non-obvious internal invariants. Replace ambiguous local names where this improves readability without changing persisted field names.
- [x] Format/document the Python tools and improve meaningful identifiers while preserving CLI contracts and release integrity checks.
- [x] Generate a multi-resolution Windows ICO from the existing icon, embed it in the PE resources, and use the high-resolution source for the runtime window.
- [x] Verify icon directory dimensions and actual Windows executable icon resources in CI; retain the portable-runtime import check.
- [x] Run Rust formatting, Clippy, Rustdoc and tests; Python/JavaScript checks, browser tests and Windows/Linux CI. Independently review the combined diff.
- [x] Update published release descriptions from their reviewed English source files. Push the changes to `main`, verify development deployment, and leave production release artifacts unchanged.

## Review focus

- Existing JSON persistence and JavaScript differential reference fixtures remain compatible.
- Linux and WASM do not require Windows resource tools.
- Windows includes both the shell resource and high-DPI runtime icon.
- English documentation preserves historical measurements and product limitations.
- German localized UI text is not confused with developer prose.

## Verification status

Local Rust, Python, JavaScript, documentation and all 25 Chromium browser tests passed. The combined diff received independent review. Native Windows resource checks, the complete browser matrix and the development deployment are enforced by the main CI run; inspect that run after pushing. Historical published release descriptions have been translated without changing their tags or artifacts.
