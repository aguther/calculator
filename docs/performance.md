# Comparison methodology

Compare release builds of the JavaScript original (`0274b42`) and the Rust/WASM
app without assuming that either language is faster.

- Download: measure the complete app shell, reporting raw and gzip bytes separately.
- Startup: navigation to the first successful keyboard calculation, both in a cold
  context and after an offline/cache restart; use the same browser engine,
  viewport and hardware.
- Input: repeated calculations and long histories up to 1,000 rows.
- Resources: record CPU time and memory separately. JavaScript heap excludes parts
  of WASM, GPU and process memory and must not be interpreted as total RAM.
- Run at least ten trials; report the median and environment. Local HTTP timings
  do not characterize mobile networks or Cloudflare latency.

Set `CALCULATOR_REFERENCE_ROOT` to an existing original checkout (default:
`../simple-calculator`). `node scripts/benchmark.cjs` writes local measurements
to `.cache/benchmark.json`. The original's second server reads its existing
production build without modifying it. Record results and limitations of the
measured revision in the verification report.

## First local measurement · 15 November 2026

Windows, Intel i9-13900K, headless Chromium 159 with software graphics, 400 × 760,
loopback HTTP, ten startup/input trials. Both existing release builds; no mobile
network and no cold compiler cache after browser process startup.

| Measurement | JavaScript | Rust/WASM |
| --- | ---: | ---: |
| Uncompressed app files | 334 KB | 3.61 MB |
| Sum of gzip-compressed files | 273 KB | 1.68 MB |
| Navigation to first successful calculation, median | 88 ms | 79 ms |
| Warm restart, median | 42 ms | 46 ms |
| Single input, short history | 9 ms | 17 ms |
| Single input, 900 history entries | 2.4 ms | 16.6 ms |
| Build 900 entries, single load run | 20.75 s | 0.63 s |
| Renderer task time for that load run | 19.13 s | 0.23 s |

Both saved states were checked to contain 900 entries. Input timings include
Playwright overhead and frame timing. The load run intentionally includes the
different rendering/storage strategies; it does not establish a general language
advantage for Rust. Its CPU values are from one run, not medians. Download totals
include all app files, including icons; separate license documents are outside
the precached app shell.

JavaScript heap after the load run: about 2.6 MB versus 6.0 MB. This is **not** a
total memory comparison: WASM memory, graphics resources and process RAM are not
fully included. Reliable RAM/battery comparisons and real mobile device
measurements remain additional work.
