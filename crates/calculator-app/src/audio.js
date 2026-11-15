// Shared Web Audio context: unlock only in a direct DOM input handler.
let enabled = false;
let context = null;
let listening = false;
function unlock() {
  if (!enabled) return;
  try {
    const Audio = window.AudioContext || window.webkitAudioContext;
    if (!Audio) return;
    if (!context || context.state === 'closed') context = new Audio();
    if (context.state !== 'running') {
      context.resume().catch(() => {});
      const buffer = context.createBufferSource();
      buffer.buffer = context.createBuffer(1, 1, context.sampleRate);
      buffer.connect(context.destination);
      buffer.onended = () => buffer.disconnect();
      buffer.start(0);
    }
  } catch (_) { /* Audio failures never interrupt calculator input. */ }
}
export function configureSound(value) {
  enabled = value;
  if (!listening) {
    listening = true;
    for (const event of ['pointerdown', 'pointerup', 'touchend', 'keydown']) {
      document.addEventListener(event, unlock, {capture:true, passive:true});
    }
  }
}
export function playClick() {
  if (!enabled || !context || context.state !== 'running') return;
  try {
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.frequency.value = 900;
    gain.gain.value = 0.04;
    oscillator.connect(gain);
    gain.connect(context.destination);
    oscillator.onended = () => {oscillator.disconnect();gain.disconnect();};
    oscillator.start();
    oscillator.stop(context.currentTime + 0.012);
  } catch (_) { /* Audio failures never interrupt calculator input. */ }
}
