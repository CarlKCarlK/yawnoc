import init, { WasmApp } from "../wasm/pkg/yawnoc_wasm.js";

let appPromise = null;
let satWorker = null;
let pendingPrevId = null;

async function getApp() {
  if (!appPromise) {
    appPromise = (async () => {
      await init();
      return new WasmApp(0x9e3779b9);
    })();
  }
  return appPromise;
}

function terminateSatWorker() {
  if (satWorker) {
    satWorker.terminate();
    satWorker = null;
  }
}

function resolvePendingPrev(app) {
  if (pendingPrevId !== null) {
    const frame = JSON.parse(app.frame_json());
    self.postMessage({ id: pendingPrevId, result: frame });
    pendingPrevId = null;
  }
}

self.addEventListener("message", async (event) => {
  const { id, cmd, args } = event.data;
  try {
    const app = await getApp();
    let json;
    switch (cmd) {
      case "frame":
        json = app.frame_json();
        break;
      case "tick":
        json = app.tick_json();
        break;
      case "pressKey": {
        const key = args.key;
        if (key === "prev") {
          terminateSatWorker();
          // Transition to searching state (non-blocking).
          app.press_key_json("prev");
          const board_json = app.board_json();
          // Hold the promise open — resolve it when SAT finishes or is cancelled.
          pendingPrevId = id;
          satWorker = new Worker(new URL("./sat-worker.js", import.meta.url), {
            type: "module",
          });
          satWorker.addEventListener("message", async (e) => {
            const innerApp = await getApp();
            if (e.data.type === "progress") {
              // Show intermediate best board; keep is_searching = true
              // so tick() renders it in red and doesn't advance the sim.
              innerApp.show_progress_json(e.data.result);
            } else {
              // type === "done"
              satWorker = null;
              if (e.data.result !== null) {
                innerApp.apply_predecessor_json(e.data.result);
              } else {
                innerApp.search_not_found_json();
              }
              resolvePendingPrev(innerApp);
            }
          });
          satWorker.addEventListener("error", async (e) => {
            console.error("sat-worker onerror:", e.message);
            satWorker = null;
            const innerApp = await getApp();
            innerApp.search_not_found_json();
            resolvePendingPrev(innerApp);
          });
          satWorker.postMessage({ board_json });
          return; // Don't post a response yet.
        } else if (key === "cancel") {
          terminateSatWorker();
          json = app.press_key_json("cancel");
          resolvePendingPrev(app); // Unblock any awaiting pressKey("prev").
        } else if (key.length === 1 && key >= "0" && key <= "9" && satWorker) {
          // Digit pressed during a search — cancel the search and load the new pattern.
          terminateSatWorker();
          json = app.press_key_json(key); // Rust clears is_searching for digit keys
          resolvePendingPrev(app);
        } else {
          json = app.press_key_json(key);
        }
        break;
      }
      case "toggleCell":
        json = app.toggle_cell_json(args.row, args.col);
        break;
      case "getState":
        json = app.state_json();
        self.postMessage({ id, result: json });
        return;
      case "setState": {
        const ok = app.replace_state_json(args.state);
        self.postMessage({ id, result: ok });
        return;
      }
      default:
        throw new Error(`unknown worker command: ${cmd}`);
    }
    self.postMessage({ id, result: JSON.parse(json) });
  } catch (error) {
    self.postMessage({
      id,
      error: error?.message ?? String(error),
    });
  }
});


