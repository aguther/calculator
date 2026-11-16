# Release-Verfahren

## Einmalige Einrichtung

1. App-Arbeitsbranch nach Review in `main` integrieren. `main` veröffentlicht nach erfolgreichen Prüfungen nur auf der Entwicklungsadresse.
2. In GitHub unter Settings → Secrets and variables → Actions die Secrets
   `CLOUDFLARE_API_TOKEN` und `CLOUDFLARE_ACCOUNT_ID` hinterlegen.
   Das Token benötigt Zugriff zum Deployment von Workers/Static Assets im
   vorgesehenen Konto. Werte nie in Dateien oder Workflow-Ausgaben schreiben.
3. Kein automatisches Cloudflare-Git-Deployment einrichten. Falls `calculator`
   bereits mit Git verbunden wird, dessen automatische Production-/Preview-Builds
   deaktivieren. Der bestehende Worker `simple-calculator` bleibt unberührt.
4. GitHub Actions erlauben; Release-Workflow erhält nur im Publish-Job Schreibrecht.

## Eine Version freigeben

Version in `[workspace.package]` erhöhen und Cargo.lock aktualisieren. Dazu die
Änderungen für Nutzer in `docs/release-notes/VERSION.md` beschreiben: neue
Funktionen, Korrekturen und gegebenenfalls Hinweise zum Update. Die Datei wird
als GitHub-Release-Beschreibung verwendet; eine fehlende oder leere Beschreibung
bricht die Freigabe ab. Bei einem wiederaufgenommenen Release wird die
Beschreibung aus derselben Datei übernommen. Änderung
prüfen und in `main` integrieren. Dann einen neuen, unveränderlichen Tag setzen:

```sh
git checkout main
git pull --ff-only
git tag -a v0.1.0 -m "Calculator 0.1.0"
git push origin v0.1.0
```

Der Workflow prüft Tag/Version und `main`-Abstammung. Alle Plattform-Builds und
Tests müssen vor Veröffentlichung erfolgreich sein. Die Binärarchive und
SHA256SUMS.txt werden zuerst in ein Draft-Release geladen, die exakt geprüfte
PWA auf Cloudflare deployt, anschließend wird das Release veröffentlicht.
Die Veröffentlichung ist global serialisiert; ältere Versionen dürfen eine
bereits veröffentlichte oder durch einen Draft reservierte neuere Version nicht
überschreiben. Damit bleibt der Schutz auch bei fehlgeschlagener Finalisierung
nach einem erfolgreichen Cloudflare-Deployment erhalten.

Downloads: `calculator-VERSION-windows-x64.zip`,
`calculator-VERSION-linux-x64.tar.gz`, `SHA256SUMS.txt`.
Windows enthält eine unsignierte einzelne EXE; Linux eine ausführbare Datei
mit dokumentierten Systembibliotheken. Keine WebView oder Rust-Installation nötig.
Die initiale Zielbasis für Linux ist Ubuntu 22.04 (glibc 2.35).

## Fehlgeschlagene Veröffentlichungen

In GitHub **Re-run failed jobs** verwenden. Damit werden bereits gebaute Artefakte
weiterverwendet. Vorhandene Release-Dateien werden anhand SHA-256 verglichen;
abweichende Bytes werden niemals automatisch ersetzt. Ein vollständiger Neubuild
kann andere Archivbytes erzeugen und wird dann bewusst abgewiesen. Für korrigierten
Quellcode eine neue Version und einen neuen Tag erstellen; keinen Tag verschieben.

Fehlt ein Cloudflare-Secret, bleibt das GitHub Release im Draft. Bei erfolgreichem
Deployment und fehlgeschlagener Release-Finalisierung kann die Browser-Version
bereits neuer sein; der erneute fehlgeschlagene Job vervollständigt die Freigabe.
GitHub und Cloudflare können nicht atomar veröffentlicht werden.

## PWA-Updates

Ein neuer Service Worker lädt die komplette Anwendung vor. Bei Fehlschlag wird
sein Cache entfernt und die bisherige Version bleibt aktiv. Ein verfügbares Update
wird per Schaltfläche angeboten. Nach Bestätigung wechseln die offenen Tabs durch
Neuladen auf die neue Version. Persistierter Rechnerzustand bleibt erhalten.

## Entwicklungsadresse für main

Nach jedem Push auf `main` laufen die vollständigen Windows-, Linux- und Web-Prüfungen. Erst wenn alle erfolgreich sind, veröffentlicht CI das bereits geprüfte Web-Artefakt unter:

https://calculator-dev.andreas-7f3.workers.dev

Der Job verwendet ausschließlich `wrangler.dev.jsonc` und den Worker `calculator-dev`. Pull Requests und Release-Tags lösen dieses Deployment nicht aus. Ein inzwischen überholter Main-Build wird übersprungen. Nach dem Deployment wird der Build-Hash der Adresse mit dem geprüften Artefakt verglichen.

Die Entwicklungsadresse ist öffentlich. Sie hat aufgrund der eigenen Origin getrennte Berechnungen, Einstellungen und PWA-Caches. Produktion bleibt `calculator` und wird ausschließlich mit Versions-Tags aktualisiert. Der ursprüngliche Worker `simple-calculator` bleibt unberührt.

Alle verwendeten GitHub-Actions laufen mit Node.js 24; auch die npm-Buildschritte verwenden Node.js 24.
