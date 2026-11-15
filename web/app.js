// Use the visible browser area; standalone iOS can report different CSS viewport units.
function syncViewport() {
  const viewport = window.visualViewport;
  const height = viewport?.height || window.innerHeight;
  const top = viewport?.offsetTop || 0;
  const root = document.documentElement;
  root.style.setProperty('--viewport-height', `${height}px`);
  root.style.setProperty('--viewport-top', `${top}px`);
  const style = window.getComputedStyle?.(root);
  root.setAttribute('data-viewport', JSON.stringify({innerHeight:window.innerHeight,visibleHeight:viewport?.height,offsetTop:top,dpr:window.devicePixelRatio,safeTop:style?.getPropertyValue('--safe-top'),safeBottom:style?.getPropertyValue('--safe-bottom'),standalone:!!navigator.standalone}));
}
syncViewport();
window.addEventListener('resize', syncViewport);
window.addEventListener('pageshow', syncViewport);
window.visualViewport?.addEventListener('resize', syncViewport);
window.visualViewport?.addEventListener('scroll', syncViewport);
document.addEventListener('visibilitychange', () => {if (!document.hidden) syncViewport();});

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
      const offer = () => { if (registration.waiting && navigator.serviceWorker.controller) {update.hidden = false;update.title='Neue Version verfügbar · jetzt aktualisieren';update.setAttribute('aria-label',update.title);} };
      offer();
      registration.addEventListener('updatefound', () => registration.installing?.addEventListener('statechange', offer));
      update.addEventListener('click', () => registration.waiting?.postMessage({type:'ACTIVATE_UPDATE'}));
    });
  }).catch(() => { /* Offline and unavailable storage do not block the calculator. */ });
}
