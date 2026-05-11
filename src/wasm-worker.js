import init, { WasmApp } from "../wasm/pkg/yawnoc_wasm.js";

let appPromise = null;
let satWorker = null;

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
          // Cancel any running search first.
          terminateSatWorker();
          // Transition app state to Searching (non-blocking).
          json = app.press_key_json("prev");
          const board_json = app.board_json();
          // Spawn child worker to run SAT off this event loop.
          satWorker = new Worker(new URL("./sat-worker.js", import.meta.url), {
            type: "module",
          });
          satWorker.addEventListener("message", async (e) => {
            satWorker = null;
            const innerApp = await getApp();
            if (e.data.result !== null) {
              innerApp.apply_predecessor_json(e.data.result);
            } else {
              innerApp.search_not_found_json();
            }
          });
          satWorker.postMessage({ board_json });
        } else if (key === "cancel") {
          terminateSatWorker();
          json = app.press_key_json("cancel");
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

