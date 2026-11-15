const update = document.querySelector('#update');
let reloading = false;
let hadController = !!navigator.serviceWorker?.controller;
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.addEventListener('controllerchange', () => {
    if (!hadController) { hadController = true; return; }
    if (reloading) return;
    reloading = true;
    location.reload();
  });
  // Release builds supply sw.js; Trunk development deliberately has no worker.
  fetch('/sw.js', {cache:'no-store'}).then(response => {
    if (!response.ok || !response.headers.get('content-type')?.includes('javascript')) return;
    return navigator.serviceWorker.register('/sw.js', {updateViaCache:'none'}).then(registration => {
      const offer = () => { if (registration.waiting && navigator.serviceWorker.controller) {update.hidden = false;update.textContent='↻';update.title='Neue Version verfügbar · jetzt aktualisieren';update.setAttribute('aria-label',update.title);} };
      offer();
      registration.addEventListener('updatefound', () => registration.installing?.addEventListener('statechange', offer));
      update.addEventListener('click', () => registration.waiting?.postMessage({type:'ACTIVATE_UPDATE'}));
    });
  }).catch(() => { /* Offline and unavailable storage do not block the calculator. */ });
}
