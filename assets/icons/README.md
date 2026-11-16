# Lucide icons

Original undo-2, delete and refresh-cw SVGs from https://github.com/lucide-icons/lucide at revision 55782beff3a0133df79c417eb68f96e8f19e2404. ISC license, full text in LICENSE and included in distribution notices. Native/WASM GUI renders the same SVG paths, with a white base for theme tinting. The PWA update button embeds refresh-cw locally. No runtime CDN.

## Windows application icon

The application icon is separate from these Lucide action icons. The canonical
image is [`web/icon-512.png`](../../web/icon-512.png). The checked-in
[`assets/windows/calculator.ico`](../windows/calculator.ico) contains 16, 20, 24,
32, 40, 48, 64, 96, 128 and 256 px images derived from that PNG. Native window
rendering continues to use the 512 px PNG; Windows builds embed the ICO in the PE
resources through `winresource` in the app's `build.rs`.

After changing the canonical PNG, install the pinned developer-only dependency
and regenerate from the repository root:

```sh
python -m pip install -r scripts/requirements-icons.txt
python scripts/generate_windows_icon.py
python scripts/test_windows_icon.py
```

Pillow is needed only for regeneration. Standard-library tests check the ICO;
Windows packaging/CI also checks embedded `RT_GROUP_ICON` and `RT_ICON` resources
alongside DLL imports with `scripts/windows_binary.py`.
