# UI and PWA refinement notes

These notes preserve the investigation history. Later sections supersede earlier
visual choices where stated; real-device observations remain distinct from
browser emulation.

## Layout and initial PWA corrections

The design follows `simple-calculator`: single-line history rows with minutes
before a right-aligned value, stronger digits, separate history/total cards and
an accessible mode selector. Groups have headers, indentation, connector lines
and an emphasized result row. Core calculation/reference rules are unchanged.

In landscape, the history-position setting moves the entire history panel.
New installations place it on the right; an explicitly saved left preference is
retained. Values remain right-aligned independently. Large windows use a centered,
width-limited interface; keys grow to at most 76 logical pixels high. Small
windows remain scalable with scrollable history.

The PWA positions the canvas inside iOS safe areas and reserves extra space for
an update notice. Haptics are offered only when the browser API exists. Missing
APIs and thrown errors must not interrupt WASM calculation. Safari/iPhone does
not provide vibration through this API; real-device testing remains necessary.

Build jobs do not themselves change the running Cloudflare version. Deployment
uses the complete artifact only after successful builds/tests. The service worker
keeps the current app active until the new version is fully loaded and explicitly
accepted. An unchanged second tab does not overwrite newer calculations with its
stale state during a coordinated reload. This is not general multi-user/tab
synchronization: actively edited tabs retain independent state.

### Verification

Playwright covered Chromium, Firefox and WebKit; the browser plugin was unavailable
in the original session. Coverage includes absent/failing haptics, simulated safe
areas, persistence, touch, theme changes and offline restart. The update test
intentionally stalls the new WASM download, calculates in the old app, checks the
waiting version before activation, then opens both tabs offline with the latest
calculation.

Rust GUI tests cover single-line minutes/value rendering, right alignment, panel
side switching and visible keys at 300×400, 320×568, 400×760, 568×320, 844×390 and
1440×900. Existing synthetic JavaScript/Rust reference cases also apply. Visual
checks cover Original/Light/Dark, small/large layouts and nested groups. Installed
iOS PWAs still need a physical iPhone check.

Font source/license: [embedded Noto Sans](../assets/fonts/README.md).

## Dialog footer and visible viewport

The heading and Done button stay outside the inner scroll area. GUI tests check
visibility and unchanged position while scrolling at heights of 400, 568 and
760 logical pixels; a browser test closes the scrolled dialog with its visible
button.

The PWA measures visible height with
[VisualViewport](https://developer.mozilla.org/en-US/docs/Web/API/VisualViewport),
falling back to `window.innerHeight`. CSS retains `100dvh` as the baseline.
Resize, redisplay and background-return events refresh the measurement. Safe-area
insets remain intact. Diagnostics include browser height, visible height, offset
and system insets.

The iPhone diagnostic from 0.1.3 reported a canvas of 402×716 logical pixels at
scale 3. That alone did not identify the exact cause of the visible-screen
mismatch. Automated coverage protects the height strategy; the affected device
must confirm behavior after publication.

## Lucide icons and audio activation

Original `undo-2`, `delete` and `refresh-cw` SVGs come from Lucide; revision and
license are in `assets/icons/`. `egui_extras` SVG support renders the original
paths on desktop and WASM. The update button loads its icon from the local
offline package. Distribution notices include the ISC license.

When key sounds are enabled, direct pointer/touch/keyboard handlers create or
resume the browser audio context. The later WASM action uses that prepared shared
context, following [Web Audio user-activation guidance](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Best_practices).
Errors and pending resume promises never block calculation. Native audio is
unchanged.

The previous WASM-render-loop call was a plausible reason for missing iOS audio,
but the device-specific cause was not confirmed. WebKit also documents
[iOS PWA resume issues](https://bugs.webkit.org/show_bug.cgi?id=291892). The
Windows WebKit test runtime used here lacked Web Audio, so its real audio gesture
test was explicitly skipped. Chromium and Firefox use the real AudioContext;
unit tests enforce direct gesture activation and cover initialization failure.
Audible output on a physical iPhone still needs confirmation after publication.

## Reported iOS standalone margin in 0.1.4

Device diagnostics reported `innerHeight` and `visualViewport.height` as 812px,
`safeTop=62px`, `safeBottom=34px` and a 716px canvas: exactly 812−62−34. No space
was reserved for an update button.

[WebKit 313800](https://bugs.webkit.org/show_bug.cgi?id=313800) reports the same
numbers, plus `screen.height=874`, and a bottom strip outside the DOM/canvas area.
[WebKit 301994](https://bugs.webkit.org/show_bug.cgi?id=301994) was reopened as a
regression. The affected device's screen height was not yet measured; diagnostics
now include `screenWidth`, `screenHeight`, `documentElement.clientHeight` and zoom.

The 0.1.4 viewport adjustment cannot fix a platform case when both browser APIs
already report reduced height. The app does not blindly extend to `screen.height`,
which could place controls outside the reachable area. The report says a regular
Safari tab is unaffected by the same standalone issue. Installed status-bar
changes and reinstallation were not a confirmed general fix.

## Comparison with the working original PWA

The subsequently supplied original diagnostic reported the same 812px for window,
document and visual viewport, but `safeArea.top=0` instead of 62; both had a 34px
bottom inset. The original app ended at 812px and its final key at 778px. Neither
document was scrolled. Artificially increasing height to 874px therefore was not
the original's solution.

The concrete configuration difference was in `index.html`: `simple-calculator`
used `apple-mobile-web-app-status-bar-style=black`, while Calculator used
`black-translucent`. Calculator now uses `black`. A build-contract test prevents
regression; a browser test simulates 402×812 with 0px top/34px bottom insets and
activates the bottom key.

The earlier blanket conclusion that no code fix was possible was therefore
insufficient for this comparison. A WebKit report alone does not rule out an
installed status-bar configuration difference. After publication, the affected
iPhone must confirm whether it picks up the changed installation metadata;
re-adding the PWA may be necessary. Back up calculations before reinstalling,
because device storage can be lost. The original PWA remains unchanged.

## PWA updates and touch feedback

The PWA checks for updates on return (`pageshow`, focus, visibility), reconnection,
and every 60 seconds while visible and online. Existing service-worker
registrations remain monitored after offline startup. Concurrent checks coalesce;
network errors do not stop subsequent attempts. Installation and activation stay
separate: offer only a fully loaded version, and activate only through the update
button.

Touch input is processed before drawing. Even start/end events within one frame
leave visible press feedback; canceled contacts are discarded. At this stage the
key face immediately depressed by a size-dependent 3.5–6 pixels, changed contrast
and retained width. It returned within 60 ms after release with no extra hold.
Key shape/rounding, hit targets and release-triggered actions were preserved.

Coverage includes worker lifecycle tests, browser updates without restart and
after offline startup, overlapping contacts and rendering of short touches,
including same-frame start/end events. Perceived behavior on a physical iPhone
still needs manual verification.

## Flat keys and the reference web version

The permanent 3D appearance was removed to match 0.1.5: no base, shadow edge or
raised key face. Shape, rounding and colors remain. Feedback follows the original
web version (`simple-calculator`, CSS reference `0274b425`): immediate scaling to
92 percent and a visible contrast change, then return within 60 ms. There is no
depth or slanted vertical displacement. Hit targets and touch cancellation remain
unchanged. This supersedes the earlier vertical-depression visual treatment.
