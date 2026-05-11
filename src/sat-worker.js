// Runs the SAT predecessor search in an isolated worker so the main
// wasm-worker can stay responsive and handle cancel while this runs.
import init, { find_predecessor_json_with_progress } from "../wasm/pkg/yawnoc_wasm.js";

let ready = init();

self.addEventListener("message", async (event) => {
  const { board_json } = event.data;
  try {
    await ready;
    const on_progress = (intermediate_board_json) => {
      self.postMessage({ type: "progress", result: intermediate_board_json });
    };
    const outcome = JSON.parse(find_predecessor_json_with_progress(board_json, on_progress));
    self.postMessage({ type: "done", ...outcome });
  } catch (err) {
    console.error("sat-worker error:", err);
    self.postMessage({ type: "done", kind: "solver_error", result: null, error: String(err) });
  }
});
