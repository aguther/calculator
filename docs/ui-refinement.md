# Oberfläche und PWA-Korrekturen

Die Gestaltung folgt weiterhin dem Referenzprojekt `simple-calculator`: einzeilige Verlaufszeilen mit Minuten vor dem rechtsbündigen Wert, stärkere Ziffern, getrennte Karten für Verlauf und Summe sowie eine gut erreichbare Moduswahl. Gruppen erhalten eine Kopfzeile, Einrückungen, Verbindungslinien und eine hervorgehobene Ergebniszeile. Der Rechnerkern und seine Referenzregeln bleiben unverändert.

Im Querformat schaltet „Verlauf rechts (Querformat)“ die Position des gesamten Verlaufs um. Neue Installationen verwenden rechts; eine ausdrücklich gespeicherte linke Position bleibt erhalten. Die numerische Ausrichtung bleibt unabhängig davon rechtsbündig. Große Fenster verwenden eine zentrierte, begrenzte Oberfläche; Tasten wachsen höchstens auf 76 logische Pixel Höhe. Kleine Fenster bleiben skalierbar und der Verlauf scrollbar.

Die PWA positioniert den Canvas innerhalb der iOS-Safe-Areas; eine Update-Meldung reserviert zusätzlich Platz. Haptik wird nur angeboten, wenn die Browserfunktion vorhanden ist. Fehlende Funktionen und geworfene Fehler dürfen die WASM-Berechnung nicht unterbrechen. Safari/iPhone bietet über die hier verwendete Browser-Schnittstelle keine Vibration; eine echte Geräteprüfung bleibt erforderlich.

GitHub-Builds ändern die laufende Cloudflare-Version nicht. Erst nach erfolgreichen Builds/Tests wird das vollständige Web-Artefakt hochgeladen. Der Service Worker hält die bestehende App aktiv, bis die neue Version vollständig geladen und das Update bewusst bestätigt wird. Ein unveränderter zweiter Tab speichert seinen veralteten Zustand beim gemeinsamen Neuladen nicht erneut über eine neuere Berechnung. Dies ist kein allgemeiner Mehrbenutzer-/Tab-Synchronisationsdienst; aktiv veränderte Tabs behalten weiterhin ihren eigenen Zustand.

## Prüfung

Automatisierte Browser-Prüfung über Playwright (Browser-Plugin in dieser Sitzung nicht verfügbar): Chromium, Firefox, WebKit. Die Tests decken unter anderem fehlende/fehlerhafte Haptik, simulierte Safe-Areas, Speicherung, Touch, Themewechsel und Offline-Neustart ab. Der Update-Test hält den neuen WASM-Download absichtlich an, rechnet in der alten App weiter, prüft die wartende Version vor Freigabe und öffnet anschließend beide Tabs mit dem neuesten Rechenstand offline.

Rust-GUI-Tests prüfen einzeilige Minuten/Wert-Darstellung, Rechtsausrichtung, Panel-Seitenwechsel und sichtbare Tasten bei 300×400, 320×568, 400×760, 568×320, 844×390 und 1440×900. Zusätzlich gelten die bisherigen synthetischen JavaScript-/Rust-Referenzfälle. Visuell geprüft werden Original/Hell/Dunkel, kleine und große Ansichten und verschachtelte Gruppen. Installierte iOS-PWAs benötigen weiterhin eine ergänzende Prüfung auf einem echten iPhone.

Schriftquelle und Lizenz: [eingebettete Noto-Sans-Schriften](../assets/fonts/README.md).

## Dialogfuß und sichtbarer Viewport

Die Überschrift und der Fertig-Button bleiben außerhalb des inneren Scrollbereichs. Der GUI-Test prüft Sichtbarkeit und unveränderte Position beim Scrollen bei 400, 568 und 760 logischen Pixeln Höhe; ein Browser-Test schließt den Dialog nach dem Scrollen über den sichtbaren Button.

Für die PWA wird die sichtbare Höhe über [VisualViewport](https://developer.mozilla.org/en-US/docs/Web/API/VisualViewport) abgeglichen, mit `window.innerHeight` als Ersatz, falls die Schnittstelle fehlt. CSS behält `100dvh` als Ausgangswert. Größenänderungen, Wiederanzeigen und Rückkehr aus dem Hintergrund aktualisieren die Messung. Safe-Area-Abstände bleiben erhalten. Die Diagnose enthält ergänzend Browserhöhe, sichtbare Höhe, Offset und Systemabstände.

Aus der iPhone-Diagnose von Version 0.1.3 (402×716 logische Pixel, Skalierung 3) ist die gemeldete Canvas-Größe bekannt; die genaue Ursache der Abweichung zum sichtbaren Bildschirm war damit noch nicht eindeutig bestimmt. Die neue Höhenstrategie ist automatisiert abgesichert, muss aber nach Veröffentlichung auf dem betroffenen Gerät bestätigt werden.

## Lucide und Audiofreigabe

`undo-2`, `delete` und `refresh-cw` stammen als Original-SVGs aus Lucide; Quellstand und Lizenz stehen unter `assets/icons/`. `egui_extras` mit SVG-Unterstützung zeichnet die Originalpfade in Desktop und WASM. Die Update-Schaltfläche lädt ihr Symbol aus dem eigenen Offline-Paket. Die ISC-Lizenz liegt den Distributionshinweisen bei.

Der Browser-Audiokontext wird bei aktivierten Tastenklicks in direkten Pointer-/Touch-/Tastatur-Handlern erstellt beziehungsweise fortgesetzt. Die spätere WASM-Rechenaktion verwendet nur den bereits vorbereiteten gemeinsamen Kontext. Das folgt den [Web-Audio-Empfehlungen zur Benutzeraktivierung](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Best_practices). Fehler und ausstehende Resume-Promises blockieren keine Berechnung. Native Audioausgabe bleibt unverändert.

Der frühere Aufruf im WASM-Renderzyklus war ein plausibler Grund für fehlenden Ton auf iOS; die konkrete Ursache auf dem Gerät ist damit noch nicht bestätigt. Zusätzlich sind [iOS-PWA-Resume-Probleme bei WebKit dokumentiert](https://bugs.webkit.org/show_bug.cgi?id=291892). Die hiesige Windows-WebKit-Testlaufzeit besitzt keine Web-Audio-Schnittstelle; dort ist der echte Audio-Gestentest explizit übersprungen. Chromium und Firefox prüfen ihn mit dem echten AudioContext; Unit-Tests erzwingen die direkte Gestenfreigabe und prüfen fehlgeschlagene Audioinitialisierung. Tatsächlich hörbare Ausgabe auf einem iPhone muss nach Veröffentlichung ergänzt werden.

## Gemeldeter iOS-Standalone-Rand in 0.1.4

Die Gerätediagnose meldet innerHeight und visualViewport.height beide als 812px, safeTop=62px, safeBottom=34px und einen Canvas von 716px. Das entspricht exakt unserer Rechnung 812−62−34. Es bleibt kein Platz für die Update-Schaltfläche reserviert.

[WebKit 313800](https://bugs.webkit.org/show_bug.cgi?id=313800) beschreibt dieselben Zahlen (zusätzlich screen.height=874) und einen unteren Streifen außerhalb der für DOM/Canvas erreichbaren Fläche. [WebKit 301994](https://bugs.webkit.org/show_bug.cgi?id=301994) ist als Regression wieder geöffnet. Die Bildschirmhöhe des konkreten Geräts ist noch nicht gemessen; die Diagnose enthält dafür jetzt auch screenWidth/screenHeight, documentElement.clientHeight und den Zoomfaktor.

Der Viewport-Abgleich aus 0.1.4 kann diesen Plattformfall nicht beheben, wenn beide Browser-APIs bereits die reduzierte Höhe liefern. Die App verlängert den Canvas nicht blind auf screen.height, weil dadurch bedienbare Tasten außerhalb der tatsächlich erreichbaren Fläche landen könnten. Ein normaler Safari-Tab ist laut Fehlerbericht nicht von demselben Standalone-Problem betroffen. Änderungen an installierten Statusbar-Einstellungen und eine erneute Installation sind kein bestätigter allgemeiner Fix.

## Abgleich mit der funktionierenden ursprünglichen PWA

Die nachgereichte Original-Diagnose zeigt dieselben 812px für window, document und visualViewport, aber safeArea.top=0 statt62; unten sind beide34px. Die Original-App endet bei812px, ihre letzte Taste bei778px. Beide Dokumente sind ungescrollt. Eine künstliche Erhöhung auf screen.height=874 ist daher nicht die Lösung des Originals.

Der konkrete Konfigurationsunterschied liegt in index.html: simple-calculator verwendet apple-mobile-web-app-status-bar-style=black, während calculator bisher black-translucent verwendete. Calculator übernimmt jetzt black. Der Build-Vertragstest verhindert eine unbeabsichtigte Rückkehr zur abweichenden Konfiguration; der Browser-Test bildet402×812 mit0px oberem und34px unterem Inset nach und betätigt die unterste Taste.

Damit ist die frühere pauschale Schlussfolgerung „kein Code-Fix möglich“ für diesen Vergleich nicht ausreichend gewesen. Der WebKit-Bericht allein schließt einen Unterschied der installierten Statusbar-Konfiguration nicht aus. Nach Veröffentlichung muss das betroffene iPhone bestätigen, ob es die geänderten Installations-Metadaten übernimmt; möglicherweise muss die PWA dafür neu hinzugefügt werden. Vor einer Neuinstallation Berechnungen sichern, da Gerätespeicher verloren gehen kann. Die vorhandene Original-PWA bleibt unverändert.
