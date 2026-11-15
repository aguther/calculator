# Oberfläche und PWA-Korrekturen

Die Gestaltung folgt weiterhin dem Referenzprojekt `simple-calculator`: einzeilige Verlaufszeilen mit Minuten vor dem rechtsbündigen Wert, stärkere Ziffern, getrennte Karten für Verlauf und Summe sowie eine gut erreichbare Moduswahl. Gruppen erhalten eine Kopfzeile, Einrückungen, Verbindungslinien und eine hervorgehobene Ergebniszeile. Der Rechnerkern und seine Referenzregeln bleiben unverändert.

Im Querformat schaltet „Verlauf rechts (Querformat)“ die Position des gesamten Verlaufs um. Neue Installationen verwenden rechts; eine ausdrücklich gespeicherte linke Position bleibt erhalten. Die numerische Ausrichtung bleibt unabhängig davon rechtsbündig. Große Fenster verwenden eine zentrierte, begrenzte Oberfläche; Tasten wachsen höchstens auf 76 logische Pixel Höhe. Kleine Fenster bleiben skalierbar und der Verlauf scrollbar.

Die PWA positioniert den Canvas innerhalb der iOS-Safe-Areas; eine Update-Meldung reserviert zusätzlich Platz. Haptik wird nur angeboten, wenn die Browserfunktion vorhanden ist. Fehlende Funktionen und geworfene Fehler dürfen die WASM-Berechnung nicht unterbrechen. Safari/iPhone bietet über die hier verwendete Browser-Schnittstelle keine Vibration; eine echte Geräteprüfung bleibt erforderlich.

GitHub-Builds ändern die laufende Cloudflare-Version nicht. Erst nach erfolgreichen Builds/Tests wird das vollständige Web-Artefakt hochgeladen. Der Service Worker hält die bestehende App aktiv, bis die neue Version vollständig geladen und das Update bewusst bestätigt wird. Ein unveränderter zweiter Tab speichert seinen veralteten Zustand beim gemeinsamen Neuladen nicht erneut über eine neuere Berechnung. Dies ist kein allgemeiner Mehrbenutzer-/Tab-Synchronisationsdienst; aktiv veränderte Tabs behalten weiterhin ihren eigenen Zustand.

## Prüfung

Automatisierte Browser-Prüfung über Playwright (Browser-Plugin in dieser Sitzung nicht verfügbar): Chromium, Firefox, WebKit. Die Tests decken unter anderem fehlende/fehlerhafte Haptik, simulierte Safe-Areas, Speicherung, Touch, Themewechsel und Offline-Neustart ab. Der Update-Test hält den neuen WASM-Download absichtlich an, rechnet in der alten App weiter, prüft die wartende Version vor Freigabe und öffnet anschließend beide Tabs mit dem neuesten Rechenstand offline.

Rust-GUI-Tests prüfen einzeilige Minuten/Wert-Darstellung, Rechtsausrichtung, Panel-Seitenwechsel und sichtbare Tasten bei 300×400, 320×568, 400×760, 568×320, 844×390 und 1440×900. Zusätzlich gelten die bisherigen synthetischen JavaScript-/Rust-Referenzfälle. Visuell geprüft werden Original/Hell/Dunkel, kleine und große Ansichten und verschachtelte Gruppen. Installierte iOS-PWAs benötigen weiterhin eine ergänzende Prüfung auf einem echten iPhone.

Schriftquelle und Lizenz: [eingebettete Noto-Sans-Schriften](../assets/fonts/README.md).
