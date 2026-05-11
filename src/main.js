const keyMap = new Map([
  [" ", "play_pause"],
  ["n", "next"],
  ["N", "next"],
  ["p", "prev"],
  ["P", "prev"],
  ["Escape", "cancel"],
  ["m", "mode"],
  ["M", "mode"],
  ["]", "speed_up"],
  ["[", "speed_down"],
]);

const panel = document.querySelector("#panel");
const status = document.querySelector("#status");
const ctx = panel.getContext("2d");
let tickTimer = null;
let backend = null;
let isPrevSearching = false;

async function createBackend() {
  if (window.__TAURI__?.core?.invoke) {
    const { invoke } = window.__TAURI__.core;
    return {
      frame: () => invoke("frame"),
      tick: () => invoke("tick"),
      pressKey: (key) => invoke("press_key", { key }),
      toggleCell: (row, col) => invoke("toggle_cell", { row, col }),
    };
  }

  const makeWorker = () =>
    new Worker(new URL("./wasm-worker.js", import.meta.url), {
      type: "module",
    });
  let worker = makeWorker();
  let requestId = 0;
  const pending = new Map();

  function attachWorkerListener(w) {
    w.addEventListener("message", (event) => {
      if (event.data.type === "progress") {
        renderFrame(event.data.frame);
        return;
      }
      const { id, result, error } = event.data;
      const request = pending.get(id);
      if (!request) return;
      pending.delete(id);
      if (error) {
        request.reject(new Error(error));
        return;
      }
      request.resolve(result);
    });
  }
  attachWorkerListener(worker);

  function restartWorker() {
    worker.terminate();
    worker = makeWorker();
    attachWorkerListener(worker);
  }

  const call = (cmd, args = {}, timeoutMs = 0) =>
    new Promise((resolve, reject) => {
      const id = requestId++;
      pending.set(id, { resolve, reject });
      worker.postMessage({ id, cmd, args });
      if (timeoutMs > 0) {
        setTimeout(() => {
          if (!pending.has(id)) return;
          pending.delete(id);
          reject(new Error("worker_timeout"));
        }, timeoutMs);
      }
    });

  return {
    frame: () => call("frame"),
    tick: () => call("tick"),
    pressKey: (key, timeoutMs = 0) => call("pressKey", { key }, timeoutMs),
    toggleCell: (row, col) => call("toggleCell", { row, col }),
    getState: () => call("getState"),
    setState: (state) => call("setState", { state }),
    restartWorker,
  };
}

function drawLed(ctx, x, y, size, color, alive) {
  const cx = x + size / 2;
  const cy = y + size / 2;
  const radius = size * 0.32;

  ctx.beginPath();
  ctx.arc(cx, cy, radius, 0, Math.PI * 2);
  ctx.fillStyle = alive ? color : "#11181c";
  ctx.fill();

  ctx.beginPath();
  ctx.arc(cx - radius * 0.35, cy - radius * 0.35, radius * 0.34, 0, Math.PI * 2);
  ctx.fillStyle = alive ? "rgba(255, 255, 255, 0.45)" : "rgba(255, 255, 255, 0.05)";
  ctx.fill();

  if (alive) {
    const glow = ctx.createRadialGradient(cx, cy, radius * 0.1, cx, cy, radius * 1.7);
    glow.addColorStop(0, `${color}bb`);
    glow.addColorStop(1, "rgba(0, 0, 0, 0)");
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(cx, cy, radius * 1.7, 0, Math.PI * 2);
    ctx.fill();
  }
}

function renderFrame(frame) {
  const width = panel.width;
  const height = panel.height;
  const padding = 26;
  const grid = width - padding * 2;
  const cell = grid / frame.width;

  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#050608";
  ctx.fillRect(0, 0, width, height);

  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "rgba(89, 215, 255, 0.10)");
  gradient.addColorStop(1, "rgba(255, 255, 255, 0.02)");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);

  for (let row = 0; row < frame.height; row += 1) {
    for (let col = 0; col < frame.width; col += 1) {
      const index = row * frame.width + col;
      const rawColor = frame.cells[index];
      const color = rawColor === null ? null : (isPrevSearching ? "#dc1e1e" : rawColor);
      drawLed(ctx, padding + col * cell, padding + row * cell, cell, color, color !== null);
    }
  }

  ctx.strokeStyle = "#2c3a42";
  ctx.lineWidth = 2;
  ctx.strokeRect(padding - 8, padding - 8, grid + 16, grid + 16);
}

function restartTimer(intervalMs) {
  if (tickTimer !== null) {
    clearInterval(tickTimer);
  }
  tickTimer = setInterval(tick, intervalMs);
}

function stopTimer() {
  if (tickTimer !== null) {
    clearInterval(tickTimer);
    tickTimer = null;
  }
}

function syncTimer(frame) {
  if (frame.status === "ok") {
    restartTimer(frame.tick_interval_ms);
  } else {
    stopTimer();
  }
}

async function refresh() {
  const frame = await backend.frame();
  renderFrame(frame);
  syncTimer(frame);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
}

async function tick() {
  const frame = await backend.tick();
  renderFrame(frame);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
}

async function handleKey(key) {
  if (key === "prev") {
    isPrevSearching = true;
    status.textContent = "searching";
    const frameBeforeSearch = await backend.frame();
    renderFrame(frameBeforeSearch);
    try {
      const frame = await backend.pressKey(key);
      isPrevSearching = false;
      renderFrame(frame);
      syncTimer(frame);
      status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
      return;
    } catch (error) {
      isPrevSearching = false;
      throw error;
    }
  }
  const frame = await backend.pressKey(key);
  isPrevSearching = false;
  renderFrame(frame);
  syncTimer(frame);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
}

document.addEventListener("keydown", async (event) => {
  const key = keyMap.get(event.key) ?? (/^[0-9]$/.test(event.key) ? event.key : null);
  if (!key) {
    return;
  }
  event.preventDefault();
  await handleKey(key);
});

for (const button of document.querySelectorAll("button[data-key]")) {
  button.addEventListener("click", async () => {
    await handleKey(button.dataset.key);
  });
}

createBackend()
  .then((createdBackend) => {
    backend = createdBackend;
    return refresh();
  })
  .catch((error) => {
    console.error(error);
    status.textContent = `load failed: ${error.message ?? error}`;
  });

panel.addEventListener("click", async (event) => {
  const padding = 26;
  const rect = panel.getBoundingClientRect();
  // canvas logical size vs CSS display size
  const scaleX = panel.width / rect.width;
  const scaleY = panel.height / rect.height;
  const lx = (event.clientX - rect.left) * scaleX;
  const ly = (event.clientY - rect.top) * scaleY;
  const grid = panel.width - padding * 2;
  const cell = grid / 16;
  const col = Math.floor((lx - padding) / cell);
  const row = Math.floor((ly - padding) / cell);
  if (col < 0 || col >= 16 || row < 0 || row >= 16) return;
  const frame = await backend.toggleCell(row, col);
  renderFrame(frame);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
});
