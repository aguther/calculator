# Calculator

Rust-Zeitrechner mit egui/eframe für Browser, Windows und Linux. Eigenständige
Vergleichsversion von [simple-calculator](https://github.com/aguther/simple-calculator).
Das ursprüngliche Projekt und dessen Deployment werden nicht verändert.

## Funktionen

Zeit- und Zahlenmodus, Kurzschreibweise (`145` → `1:45`), Sekunden, Skalare bei
Multiplikation/Division, Vorrangregeln, Klammern, Zwischensummen, Verlauf und Undo.
Themes **Original**, **Hell**, **Dunkel** funktionieren in PWA und Desktop-App.
Berechnungen und Einstellungen werden lokal gespeichert; keine Telemetrie und
keine Synchronisierung. 64 Zeichen pro Eingabe, 1.000 Verlaufseinträge.

## Lokal starten

Rustup installiert die in `rust-toolchain.toml` festgeschriebene Toolchain und das
WASM-Ziel. Windows benötigt zum Bauen die Visual Studio C++ Build Tools.

```sh
cargo run -p calculator
cargo install trunk --version 0.21.14 --locked
trunk serve
```

Die Entwicklungsseite läuft auf http://127.0.0.1:8080. Für den produktiven
Offline-Modus den vollständigen PWA-Build verwenden:

```sh
python scripts/build-web.py
node scripts/server.cjs
```

Dann http://127.0.0.1:8765 öffnen. Trunk allein erzeugt bewusst keinen
Service Worker. Python >=3.11 und Node >=24 werden für Build/Tests benötigt.
Linux-Buildpakete siehe `.github/workflows/ci.yml`; Laufzeitvoraussetzungen unter
[Desktop](docs/desktop.md).

## Prüfen

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p calculator --target wasm32-unknown-unknown --locked -- -D warnings
cargo test --workspace --locked
node scripts/reference-fixtures.cjs --check
python scripts/test_web.py
npm ci
npx playwright install chromium firefox webkit
npm test
```

Die Browser-Tests benötigen zuvor `python scripts/build-web.py`. Für isolierte
Browser-Downloads kann `PLAYWRIGHT_BROWSERS_PATH` auf `.cache/ms-playwright`
gesetzt werden. `cargo run --release -p calculator --example smoke` öffnet ein
Testfenster und beendet es nach drei Frames.

## Veröffentlichen

**`main` und Pull Requests veröffentlichen nichts.** Nur ein stabiler Versions-Tag
wie `v0.1.0` führt nach erfolgreichen Prüfungen zu Desktop-Downloads auf GitHub
Releases und zum Cloudflare-Deployment `calculator`.

Vorgehen, erforderliche Secrets und Fehlerbehebung: [Release-Anleitung](docs/releases.md).
Den ersten produktiven Tag erst nach Abnahme der App erstellen.

## Unterschiede und Grenzen

Die GUI wird in allen Varianten durch egui gezeichnet. Browser benötigen
WebAssembly, JavaScript und WebGL. HTML/CSS des Originals werden nicht verwendet;
Schrift, Scrollverhalten und Touch-Gesten können deshalb abweichen.
Die Browser-Canvas besitzt eine HTML-Live-Ausgabe und Tastaturbedienung, bietet
aber nicht die vollständige semantische Screenreader-Bedienung des DOM-Originals.
Browserzoom und Betriebssystem-Skalierung werden unterstützt; reale installierte
iOS-PWAs müssen ergänzend auf einem Gerät geprüft werden.

Haptik funktioniert nur bei Browser-/Geräteunterstützung. Desktop bietet optionale
Tastentöne, aber keine allgemeine Haptik-Schnittstelle. Ohne Audioausgabe bleibt
der Rechner bedienbar. Bei blockiertem lokalen Speicher bleibt die Berechnung
für die aktuelle Sitzung verfügbar. Keine automatische Datenübernahme aus der
ursprünglichen App.

## Herkunft

MIT, Copyright 2026 Andreas Guther. Rechenregeln, Referenzcode, Icons und Gestaltung
stammen aus `aguther/simple-calculator` (Referenzcommit `0274b42`). Der originale
JavaScript-Kern und extrahierte Zustandsfunktionen unter `tests/reference/` dienen
nur dem synthetischen Vergleich; ausgelieferte Berechnungen laufen in Rust.
Noto Sans (Regular/SemiBold, SIL OFL 1.1) und die mit egui gelieferten Fallback-Schriften werden eingebettet. Vollständige verfügbare
Lizenztexte der Cargo-Abhängigkeiten einschließlich Schriften werden den
Desktop-Archiven beigelegt und als THIRD_PARTY_LICENSES.txt im Web-Build ausgeliefert.
