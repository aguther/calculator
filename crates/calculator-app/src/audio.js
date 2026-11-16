// Share one Web Audio context, unlocked only inside a direct DOM input handler.
let soundEnabled = false;
let audioContext = null;
let gestureListenersInstalled = false;
function unlockAudioForGesture() {
  if (!soundEnabled) return;
  try {
    const AudioContextConstructor = window.AudioContext || window.webkitAudioContext;
    if (!AudioContextConstructor) return;
    if (!audioContext || audioContext.state === 'closed') audioContext = new AudioContextConstructor();
    if (audioContext.state !== 'running') {
      audioContext.resume().catch(() => {});
      const buffer = audioContext.createBufferSource();
      buffer.buffer = audioContext.createBuffer(1, 1, audioContext.sampleRate);
      buffer.connect(audioContext.destination);
      buffer.onended = () => buffer.disconnect();
      buffer.start(0);
    }
  } catch (_) { /* AudioContextConstructor failures never interrupt calculator input. */ }
}
/** Set the preference and install gesture listeners once; never unlock from WASM. */
export function configureSound(value) {
  soundEnabled = value;
  if (!gestureListenersInstalled) {
    gestureListenersInstalled = true;
    for (const event of ['pointerdown', 'pointerup', 'touchend', 'keydown']) {
      document.addEventListener(event, unlockAudioForGesture, {capture:true, passive:true});
    }
  }
}
/** Play feedback only after a real gesture has unlocked a usable audio context. */
export function playClick() {
  if (!soundEnabled || !audioContext || audioContext.state !== 'running') return;
  try {
    const oscillator = audioContext.createOscillator();
    const gain = audioContext.createGain();
    oscillator.frequency.value = 900;
    gain.gain.value = 0.04;
    oscillator.connect(gain);
    gain.connect(audioContext.destination);
    oscillator.onended = () => {oscillator.disconnect();gain.disconnect();};
    oscillator.start();
    oscillator.stop(audioContext.currentTime + 0.012);
  } catch (_) { /* AudioContextConstructor failures never interrupt calculator input. */ }
}
