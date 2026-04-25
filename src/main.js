const { invoke } = window.__TAURI__.core;

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
      const color = frame.cells[index];
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

async function refresh() {
  const frame = await invoke("frame");
  renderFrame(frame);
  restartTimer(frame.tick_interval_ms);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
}

async function tick() {
  const frame = await invoke("tick");
  renderFrame(frame);
  status.textContent = `${frame.status} | ${frame.live_cells} live | ${frame.speed}`;
}

async function handleKey(key) {
  const frame = await invoke("press_key", { key });
  renderFrame(frame);
  restartTimer(frame.tick_interval_ms);
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

refresh().catch((error) => {
  console.error(error);
  status.textContent = `load failed: ${error.message ?? error}`;
});
