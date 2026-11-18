// nrzPaper – Service Worker. Einzige Aufgabe: Dateien entgegennehmen, die auf Android über
// „Teilen → nrzPaper“ kommen (Web Share Target). Sie landen kurz im Cache „nrz-share“; die
// App holt sie nach dem Öffnen ab und legt sie in die Inbox (static/mobile.js, takeShared).
// Alles andere geht unverändert ans Netz – nichts wird offline zwischengespeichert.

const SHARE_CACHE = 'nrz-share';

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()));

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== 'POST' || url.pathname !== '/share-target') return;
  event.respondWith((async () => {
    try {
      const form = await event.request.formData();
      const files = form.getAll('file').filter((f) => f && typeof f !== 'string');
      const cache = await caches.open(SHARE_CACHE);
      for (const key of await cache.keys()) await cache.delete(key);
      let i = 0;
      for (const f of files) {
        await cache.put(`/share-target/file/${i++}`, new Response(f, {
          headers: {
            'Content-Type': f.type || 'application/octet-stream',
            'X-File-Name': encodeURIComponent(f.name || 'Dokument'),
          },
        }));
      }
      return Response.redirect(files.length ? '/?shared=1' : '/?shared=failed', 303);
    } catch (e) {
      return Response.redirect('/?shared=failed', 303);
    }
  })());
});
