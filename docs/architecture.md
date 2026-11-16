# Architecture and source map

The workspace shares one Rust calculator and egui UI between native desktops and
a WASM PWA. The browser shell supplies installation, viewport handling, audio
activation, accessibility output and service-worker updates.

| Area | Entry points | Responsibility |
| --- | --- | --- |
| Pure calculation | [core](../crates/calculator-core/src/lib.rs) | Parsing, time/number modes, actions, history and undo; no GUI dependency |
| Application | [app](../crates/calculator-app/src/lib.rs), [native entry](../crates/calculator-app/src/main.rs) | State restoration, keyboard routing, layout, rendering and persistence |
| Interaction | [touch](../crates/calculator-app/src/touch.rs), [buttons](../crates/calculator-app/src/buttons.rs) | Contact ownership/cancellation and visual feedback with stable hit rectangles |
| Settings/feedback | [settings](../crates/calculator-app/src/settings.rs), [feedback](../crates/calculator-app/src/feedback.rs), [audio](../crates/calculator-app/src/audio.js) | Preferences, diagnostics, haptics and key sounds |
| Browser shell | [index](../index.html), [app.js](../web/app.js), [CSS](../web/app.css) | Canvas lifecycle, safe areas, viewport, keyboard integration and updates |
| Windows icon | [build script](../crates/calculator-app/build.rs), [generator](../scripts/generate_windows_icon.py), [maintenance](../assets/icons/README.md#windows-application-icon) | Windows-only PE icon resources; window artwork uses the canonical PNG |
| Offline build | [build-web.py](../scripts/build-web.py), [worker template](../web/sw.template.js) | Versioned asset manifest, complete app precache and update integrity |
| Delivery | [CI](../.github/workflows/ci.yml), [release](../.github/workflows/release.yml), [scripts](../scripts/) | Verification, packaging, checksums and deployment |

## State and persistence

`CalculatorApp` owns the core calculator plus preferences, including the explicit
German/English language selection. Without a saved manual choice, the initial
language follows the system/browser locale. Settings allows a persisted override.
Both the app and core state
use serde JSON. The eframe storage key is `calculator-v1`; browser storage is
origin-specific and native storage uses the `Calculator` application identity.
Development and production origins therefore have separate data. There is no
server synchronization or automatic import from the JavaScript app.

Serde defaults allow missing preference/state fields; transient UI fields, touch
contacts and the last-saved snapshot are skipped. Restore tolerates unusable
stored JSON by falling back to defaults. Preserve field/enum representations and
defaults when changing Rust names or adding settings; use explicit serde names or
migrations where necessary. Keep old-state restoration and missing-field tests.

The saved JSON snapshot prevents an unchanged stale tab from overwriting a newer
calculation when an update reloads all tabs. It is not a general multi-tab merge:
actively edited tabs still have independent state. Storage failure must leave
calculation usable for the current session.

## Input and rendering

The eframe update loop handles keyboard and touch input before drawing. Touch
contacts resolve against registered button targets; release commits an action,
while cancellation or focus loss discards interrupted contacts. Native/emulated
pointer input must not double-commit a touch action. Painting scales and recolors
the button face without shrinking its hit rectangle. Same-frame start/end events
still produce visible feedback. Settings retain fixed navigation/footer controls
around scrollable content. See [UI notes](ui-refinement.md) for historical changes
and real-device limitations.

## PWA lifecycle

Trunk alone is a development server. `scripts/build-web.py` produces the complete
versioned PWA and service worker. A new worker precaches the complete app before
offering activation; a failed installation discards its incomplete cache and
leaves the existing app running. The user confirms the update button, then open
tabs reload into the new version with persisted state retained.

Update checks run on return to the app, reconnection and every 60 seconds while
visible and online. Registration monitoring also works after an offline start;
concurrent checks coalesce and network failures do not prevent later retries.
Do not activate partial builds or skip the user's update confirmation.

## Test layers and release gates

Core behavior, reference arithmetic and state transitions are covered by
[Rust tests](../crates/calculator-core/tests/) and the synthetic JavaScript
[reference fixtures](../tests/reference/). App unit tests and
[UI tests](../crates/calculator-app/tests/ui.rs) cover keyboard, serialization,
touch and layout. [Node tests](../tests/) exercise service-worker, viewport, audio
and update lifecycle contracts; [Python checks](../scripts/test_web.py) verify the
build contract. [Playwright](../tests/browser/calculator.spec.cjs) tests real browser
engines, including offline startup, updates, settings and touch.

CI runs Windows, Linux and web checks before deploying `main` to `calculator-dev`.
Version tags additionally validate version/main ancestry, build desktop packages,
verify imports/licenses/checksums, stage a draft release, deploy the exact checked
web artifact and finalize the release. See [release gates and recovery](releases.md).
