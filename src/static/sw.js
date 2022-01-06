var cacheName = 't27frapp';
var assets = [
  './',
  './t27patch.png',
];


/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function(e) {
    e.waitUntil(
        caches.open(cacheName).then(function(cache) {
            return cache.addAll(assets);
        })
    );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function(e) {
    e.respondWith(
        fetch(e.request).catch(function() {
            return caches.match(e.request)
        })
    );
});
