const CACHE_NAME = "yawnoc-v9";

// All assets the app needs to work offline after first load.
// The .wasm binary is cached lazily on first fetch to avoid blocking SW install.
const PRECACHE = [
  "./",
  "./manifest.json",
  "./sw.js",
  "./src/styles.css",
  "./src/main.js",
  "./src/wasm-worker.js",
  "./wasm/pkg/yawnoc_wasm.js",
  "./public/remote.jpg",
  "./public/icon.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(PRECACHE))
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
      )
    )
  );
  self.clients.claim();
});

// Cache-first: serve from cache, fall back to network and cache the response.
self.addEventListener("fetch", (event) => {
  // Only handle GET requests for same-origin or relative assets.
  if (event.request.method !== "GET") return;

  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;
      return fetch(event.request).then((response) => {
        // Only cache valid same-origin responses.
        if (
          response.ok &&
          (event.request.url.startsWith(self.location.origin) ||
            event.request.url.startsWith("./"))
        ) {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
        }
        return response;
      });
    })
  );
});
