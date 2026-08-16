/**
 * Exists only so the app qualifies as installable. It caches nothing on purpose: Rekall is
 * the frontend for a local database that can change out from under it (a switched database
 * folder, a rebuilt jar), and a service worker that served a cached response in that moment
 * would be indistinguishable from data loss. Every request goes to the network, always.
 */
self.addEventListener('install', () => {
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim())
})

self.addEventListener('fetch', () => {
  // No-op: declining to call event.respondWith() falls through to the network as if this
  // listener were not here at all.
})
