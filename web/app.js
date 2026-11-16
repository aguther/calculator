/** Select the first supported browser language; unsupported locales use English. */
function browserLanguage() {
  for (const locale of navigator.languages || [navigator.language]) {
    const primary = typeof locale === 'string' ? locale.toLowerCase().split(/[-_]/)[0] : '';
    if (primary === 'de' || primary === 'en') return primary;
  }
  return 'en';
}

/** Keep the HTML shell aligned with the language selected by the Rust frontend. */
function updateShellLabels() {
  const german = document.documentElement.lang === 'de';
  document.title = german ? 'Calculator · Zeit- und Zahlenrechner' : 'Calculator · Time and number calculator';
  const loading = document.getElementById?.('loading');
  if (loading) loading.textContent = german ? 'Calculator wird geladen …' : 'Loading calculator…';
  const canvas = document.getElementById?.('calculator-canvas');
  canvas?.setAttribute('aria-label', german
    ? 'Zeit- und Zahlenrechner. Bedienung mit Ziffern, Rechenzeichen und Enter.'
    : 'Time and number calculator. Use digits, operators and Enter.');
  const updateButton = document.querySelector('#update');
  if (updateButton) {
    updateButton.title = german ? 'Neue Version verfügbar · jetzt aktualisieren' : 'New version available · update now';
    updateButton.setAttribute?.('aria-label', updateButton.title);
  }
}
document.documentElement.lang = browserLanguage();
updateShellLabels();
if (typeof MutationObserver !== 'undefined') {
  new MutationObserver(updateShellLabels).observe(document.documentElement, {attributes: true, attributeFilter: ['lang']});
}

// Keep app shortcuts from opening browser Help or Find while the canvas is focused.
document.addEventListener('keydown', event => {
  if (event.target?.id === 'calculator-canvas' && !event.ctrlKey && !event.metaKey && !event.altKey && ['F1', 'F2', 'F3'].includes(event.key)) {
    event.preventDefault();
  }
}, {capture: true});

// Use the visible browser area; standalone iOS can report different CSS viewport units.
function syncViewport() {
  const viewport = window.visualViewport;
  const height = viewport?.height || window.innerHeight;
  const top = viewport?.offsetTop || 0;
  const root = document.documentElement;
  root.style.setProperty('--viewport-height', `${height}px`);
  root.style.setProperty('--viewport-top', `${top}px`);
  const style = window.getComputedStyle?.(root);
  root.setAttribute('data-viewport', JSON.stringify({innerHeight:window.innerHeight,screenWidth:window.screen?.width,screenHeight:window.screen?.height,clientHeight:root.clientHeight,viewportScale:viewport?.scale,visibleHeight:viewport?.height,offsetTop:top,dpr:window.devicePixelRatio,safeTop:style?.getPropertyValue('--safe-top'),safeBottom:style?.getPropertyValue('--safe-bottom'),standalone:!!navigator.standalone}));
}
syncViewport();
window.addEventListener('resize', syncViewport);
window.addEventListener('pageshow', syncViewport);
window.visualViewport?.addEventListener('resize', syncViewport);
window.visualViewport?.addEventListener('scroll', syncViewport);
document.addEventListener('visibilitychange', () => {if (!document.hidden) syncViewport();});

const updateButton = document.querySelector('#update');
let reloading = false;
let hadController = !!navigator.serviceWorker?.controller;
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.addEventListener('controllerchange', () => {
    if (!hadController) { hadController = true; return; }
    if (reloading) return;
    reloading = true;
    location.reload();
  });
  /** Attach lifecycle checks to an existing registration, even after offline launch. */
  const watchUpdates = registration => {
    const offer = () => {
      if (registration.waiting && navigator.serviceWorker.controller) {
        updateButton.hidden = false;
        updateShellLabels();
      }
    };
    const watchInstalling = () => registration.installing?.addEventListener('statechange', offer);
    registration.addEventListener('updatefound', watchInstalling);
    watchInstalling();
    updateButton.addEventListener('click', () => registration.waiting?.postMessage({type:'ACTIVATE_UPDATE'}));
    let checking = false;
    const check = async () => {
      offer();
      if (checking || document.hidden || navigator.onLine === false) return;
      checking = true;
      try { await registration.update(); }
      catch { /* Offline checks are retried on reconnect or the next visible interval. */ }
      finally { checking = false; offer(); }
    };
    window.addEventListener('pageshow', check);
    window.addEventListener('focus', check);
    window.addEventListener('online', check);
    document.addEventListener('visibilitychange', check);
    setInterval(check, 60000);
    void check();
  };
  // An existing PWA registration remains usable even when launched offline.
  navigator.serviceWorker.getRegistration('/').then(async existing => {
    if (existing) { watchUpdates(existing); return; }
    // Release builds supply sw.js; Trunk development deliberately has no worker.
    const response = await fetch('/sw.js', {cache:'no-store'});
    if (!response.ok || !response.headers.get('content-type')?.includes('javascript')) return;
    watchUpdates(await navigator.serviceWorker.register('/sw.js', {updateViaCache:'none'}));
  }).catch(() => { /* Unavailable storage does not block the calculator. */ });
}
