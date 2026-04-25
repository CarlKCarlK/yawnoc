import init, { WasmApp } from "../wasm/pkg/yawnoc_wasm.js";

let appPromise = null;

async function getApp() {
  if (!appPromise) {
    appPromise = (async () => {
      await init();
      return new WasmApp(0x9e3779b9);
    })();
  }
  return appPromise;
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
      case "pressKey":
        json = app.press_key_json(args.key);
        break;
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
