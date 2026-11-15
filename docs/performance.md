# Vergleichsmethodik

Verglichen werden Release-Builds des JavaScript-Originals (0274b42) und der
Rust-WASM-App. Keine pauschale Annahme eines Geschwindigkeitsvorteils.

- Download: gesamte App-Shell, rohe Bytes und gzip-komprimierte Bytes separat.
- Start: Navigation bis erfolgreiche erste Tastaturberechnung, kalter Kontext
  und Offline-/Cache-Neustart; gleiche Browserengine, Viewport und Hardware.
- Eingabe: wiederholte Berechnungen sowie langer Verlauf bis 1.000 Zeilen.
- Ressourcen: CPU-Zeit und Speicher getrennt erfassen. JavaScript-Heap misst
  weder die gesamte WASM-Nutzung noch GPU-/Prozessspeicher und wird nicht als
  Gesamt-RAM interpretiert.
- Mindestens zehn Durchläufe; Median und Messumgebung angeben. Lokale HTTP-Werte
  sind keine Aussage über mobile Netze oder Cloudflare-Latenzen.

Mit `CALCULATOR_REFERENCE_ROOT` den vorhandenen Original-Checkout auswählen
(Standard: Geschwisterverzeichnis `../simple-calculator`).
`node scripts/benchmark.cjs` schreibt lokale Messungen nach `.cache/benchmark.json`.
Der zweite Server für das Original wird nur lesend mit dessen vorhandenem
Produktionsbuild gestartet. Ergebnisse und Grenzen des geprüften Standes werden
im Verifikationsbericht festgehalten.

## Erste lokale Messung · 15. November 2026

Windows, Intel i9-13900K, Chromium 159 headless mit Software-Grafik, 400 × 760,
HTTP auf Loopback, zehn Start-/Eingabedurchläufe. Beide vorhandenen Release-Builds;
kein mobiles Netzwerk und kein Cold-Compiler-Cache nach Browser-Prozessstart.

| Messwert | JavaScript | Rust/WASM |
| --- | ---: | ---: |
| App-Dateien unkomprimiert | 334 KB | 3,61 MB |
| Summe gzip-komprimierter Dateien | 273 KB | 1,68 MB |
| Navigation bis erste erfolgreiche Rechnung, Median | 88 ms | 79 ms |
| Warmer Neustart, Median | 42 ms | 46 ms |
| Einzelne Eingabe, kurzer Verlauf | 9 ms | 17 ms |
| Einzelne Eingabe, 900 Verlaufseinträge | 2,4 ms | 16,6 ms |
| 900 Einträge aufbauen, einmaliger Lastlauf | 20,75 s | 0,63 s |
| Renderer-Taskzeit dieses Lastlaufs | 19,13 s | 0,23 s |

Die Zahl von 900 Einträgen wurde in beiden gespeicherten Zuständen geprüft.
Die Eingabezeiten enthalten Playwright-Aufwand und Frame-Timing. Der Lastlauf
misst bewusst auch die unterschiedlichen Render-/Speicherstrategien; er beweist
keinen allgemeinen Vorteil von Rust als Sprache. Seine CPU-Werte sind ein einzelner
Durchlauf, keine Medianwerte. Die Downloadsumme zählt alle App-Dateien einschließlich
Icons; separate Lizenzdokumente gehören nicht zur vorgeladenen App-Shell.

JavaScript-Heap nach dem Lastlauf: etwa 2,6 MB versus 6,0 MB. Dies ist **kein**
Gesamtspeichervergleich: WASM-Speicher, Grafikressourcen und Prozess-RAM sind damit
nicht vollständig erfasst. Ein belastbarer RAM-/Akkuvergleich und Messungen auf
realen mobilen Geräten bleiben ergänzende Prüfungen.
