# Calculator Desktop

Entpacken und calculator.exe (Windows) beziehungsweise ./calculator (Linux) starten.
Windows: x64, Windows 10/11; keine Rust-, WebView- oder Visual-C++-Runtime-Installation erforderlich.
Ein Grafiktreiber mit OpenGL >= 2.0 ist erforderlich.
Die Windows-Version bindet die C-Laufzeit statisch ein; die Paketierung prüft
die tatsächlichen DLL-Importe und lehnt separate VC-Runtime-Abhängigkeiten ab.
Linux: x64, glibc >= 2.35 (Ubuntu 22.04 oder neuer), OpenGL, X11 oder Wayland,
libxkbcommon und ALSA. Beispiel Ubuntu: libgl1 libxkbcommon0 libasound2 sowie
libx11-6 libxi6 libxcursor1 libxrandr2 oder libwayland-client0.
Eine grafische Sitzung und ein funktionierender Grafiktreiber sind erforderlich.

Das Programm ist noch nicht signiert; Windows kann eine SmartScreen-Warnung zeigen.
Themes und Berechnungen werden lokal im benutzerspezifischen eframe-Datenverzeichnis
unter der Anwendungskennung Calculator gespeichert. Keine Synchronisierung.
Escape löscht alles, Entf die Eingabe, Rücktaste die letzte Stelle; Enter rechnet.

GitHub-hosted Windows-Runner stellen nur OpenGL 1.1 bereit. Dort werden Build,
Rechner-/GUI-Tests ohne Fenster und DLL-Importe geprüft; der echte Fensterstart
wird lokal unter Windows und im Linux-CI-Job mit Xvfb/Mesa geprüft.
