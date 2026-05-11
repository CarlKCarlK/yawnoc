// Runs the SAT predecessor search in an isolated worker so the main
// wasm-worker can stay responsive and handle cancel while this runs.
import init, { find_predecessor_json } from "../wasm/pkg/yawnoc_wasm.js";

let ready = init();

self.addEventListener("message", async (event) => {
  const { board_json } = event.data;
  try {
    await ready;
    const result = find_predecessor_json(board_json) ?? null;
    self.postMessage({ result });
  } catch (err) {
    console.error("sat-worker error:", err);
    self.postMessage({ result: null, error: String(err) });
  }
});
