# Settings redesign

Approved design: the interactive preview in the conversation, with subsequent user changes.

- Three generous tabs: Allgemein, Bedienung, Diagnose. Fixed tab header and Fertig footer; only content scrolls.
- Allgemein retains themes and the three existing switches. Existing saved settings remain compatible.
- Bedienung lists useful shortcuts only: F1 settings, F2 time, F3 numbers, Backspace, Delete, Escape. Omit obvious digits and arithmetic operators.
- Diagnose presents readable app/device information and a generous copy button retaining the full diagnostic JSON.
- Buttons gain a subtle raised face and animated depression on press/release, instead of a white active outline. Preserve visible keyboard focus and hit targets, including touch cancellation.
- F2/F3 select their modes outside settings, preserving each mode's calculation. Calculator input remains blocked inside settings.
- Use shared Rust/egui implementation for desktop and browser; no new dependencies or release/deployment.

## Approaches
User selected three tabs over expanding cards, for easier access and room for future grouped settings. Keep the existing theme palettes and switch style. Use one shared painted button for consistent press feedback.

Latest user refinement: version text uses 14px instead of small print; omit the on-device storage sentence.
