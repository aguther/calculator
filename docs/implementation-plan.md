# Calculator implementation plan
Approved scope: egui/eframe Rust GUI, WASM PWA, Windows/Linux x64; original application unchanged.
1. Isolated repository and reproducible toolchain; platform feasibility.
2. Pure state machine with JavaScript differential fixtures and boundary tests.
3. Shared compact responsive GUI; Original, Light and Dark themes; local persistence.
4. Offline PWA with version-consistent updates and Cloudflare static configuration.
5. CI for all targets; tag-only releases and Cloudflare deployment, no first tag before acceptance.
Validation: cargo test, fmt, clippy, native/WASM builds, browser engine matrix and desktop smoke checks.
