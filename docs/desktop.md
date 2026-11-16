# Calculator Desktop

Extract the archive and run `calculator.exe` on Windows or `./calculator` on Linux.

Windows: x64, Windows 10/11, and a graphics driver with OpenGL >=2.0. No Rust,
WebView or Visual C++ runtime installation is required. The Windows build links
the C runtime statically; packaging checks actual DLL imports and rejects separate
VC runtime dependencies.

Linux: x64, glibc >=2.35 (Ubuntu 22.04 or newer), OpenGL, X11 or Wayland,
libxkbcommon and ALSA. Example Ubuntu runtime packages: `libgl1`, `libxkbcommon0`,
`libxkbcommon-x11-0`, `libasound2`, and `libx11-6`, `libxi6`, `libxcursor1`,
`libxrandr2` or `libwayland-client0`. A graphical session and working graphics
driver are required.

The program is unsigned; Windows may show a SmartScreen warning. Themes and
calculations are saved locally in eframe's per-user data directory under the
application identifier `Calculator`. There is no synchronization. The interface
supports German and English, initially following the system locale. A manual
language selection in Settings is saved with the other preferences.
Escape clears everything, Delete clears the input, Backspace removes the last
character, and Enter calculates.

GitHub-hosted Windows runners provide only OpenGL 1.1. Windows CI checks the
build, headless calculator/GUI tests and DLL imports. Actual window startup is
checked locally on Windows and in Linux CI with Xvfb/Mesa.

The Windows executable embeds the application icon for Explorer and shortcuts;
the native window uses the same artwork from the 512 px PNG. See
[icon maintenance](../assets/icons/README.md#windows-application-icon) when changing
that artwork.
