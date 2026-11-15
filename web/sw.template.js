const VERSION = __VERSION__;
const CACHE = 'calculator-' + VERSION;
const ASSETS = __ASSETS__;
self.addEventListener('install', event => {
  event.waitUntil((async () => {
    try {
      const cache = await caches.open(CACHE);
      await cache.addAll(ASSETS.map(path => new Request(path, {cache:'reload'})));
    } catch (error) {
      await caches.delete(CACHE);
      throw error;
    }
  })());
});
self.addEventListener('message', event => {
  if(event.data?.type === 'ACTIVATE_UPDATE') self.skipWaiting();
  if(event.data?.type === 'GET_BUILD_VERSION') event.ports[0]?.postMessage({version:VERSION});
});
self.addEventListener('activate', event => {
  event.waitUntil((async () => {
    // Existing clients reload on controllerchange before using the new shell.
    await self.clients.claim();
    for(const key of await caches.keys()) if(key.startsWith('calculator-') && key !== CACHE) await caches.delete(key);
  })());
});
self.addEventListener('fetch', event => {
  if(event.request.method !== 'GET' || new URL(event.request.url).origin !== self.location.origin) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    const cached = await cache.match(event.request, {ignoreSearch:event.request.mode === 'navigate'});
    if(cached) return cached;
    if(event.request.mode === 'navigate') return await cache.match('/index.html') || fetch(event.request);
    return fetch(event.request);
  })());
});
